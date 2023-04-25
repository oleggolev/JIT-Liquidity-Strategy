mod abi;

use ethereum_abi::Value;
use ethers::contract::abigen;
use ethers::signers::{LocalWallet, Signer};
use ethers::types::H160;
use ethers::{
    providers::{Middleware, Provider, Ws},
    types::{Address, Transaction, H256},
    utils::Anvil,
};
use ethers_contract::Contract;
use ethers_providers::StreamExt;
use ethers_providers::{rpc::transports::ws::WsClient, Http};
use std::{
    ops::Mul,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::config::Config;
use crate::datapoint::DataPoint;

use self::abi::AbiWrapper;

const UNISWAP_V2_ROUTER: &str = "0x7a250d5630b4cf539739df2c5dacb4c659f2488d";
const UNISWAP_V2_FACTORY: &str = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f";

const LLAMA_WS: &str = "wss://eth.llamarpc.com";
const LLAMA_HTTP: &str = "https://eth.llamarpc.com";

const INFURA_WS: &str = "wss://mainnet.infura.io/ws/v3";

abigen!(
    UniswapV2Pair,
    r#"[
        approve(address,uint256)(bool)
        getReserves()(uint112,uint112,uint32)
    ]"#
);

pub async fn start(config: Config, data: Arc<Mutex<Vec<DataPoint>>>) {
    // Launch the local anvil network, instantiated as a fork of the connected provider node's blockchain.
    let anvil = Anvil::new().fork(LLAMA_HTTP).spawn();
    let wallet: LocalWallet = anvil.keys()[0].clone().into();
    let anvil_provider = Provider::<Http>::try_from(anvil.endpoint()).unwrap();

    // Connect to the external provider from which we will process raw pending transactions.
    let external_provider = Provider::<Ws>::connect(match config.provider {
        crate::config::Provider::Infura => format!(
            "{}/{}",
            INFURA_WS,
            config
                .api_key
                .clone()
                .expect("API key is required to listen on Infura nodes"),
        ),
        crate::config::Provider::Llama => LLAMA_WS.to_owned(),
    })
    .await
    .unwrap();

    // Subscribe to a feed of all pending transactions from the external provider.
    let mut tx_stream = external_provider.subscribe_pending_txs().await.unwrap();

    // Create ABI elements.
    let abi = AbiWrapper::new(config.abi_json_path.clone());

    // Forward every pending transaction to the local Anvil node.
    loop {
        let tx_hash = tx_stream.next().await.unwrap();
        let ep = external_provider.clone();
        let ip = anvil_provider.clone();
        let config = config.clone();
        let abi = abi.clone();
        let data = data.clone();
        let wallet = wallet.clone();

        tokio::spawn(async move {
            let _ = collect(tx_hash, ep, ip, config, abi, data, wallet).await;
            // .map_err(|err| println!("{err:?}"));
        });
    }
}

async fn collect(
    tx_hash: H256,
    ep: Provider<WsClient>,
    ip: Provider<Http>,
    config: Config,
    abi: AbiWrapper,
    data: Arc<Mutex<Vec<DataPoint>>>,
    wallet: LocalWallet,
) -> Result<(), String> {
    let tx = try_get_transaction(
        tx_hash,
        ep.clone(),
        config.tx_retry_times,
        config.tx_retry_period,
    )
    .await?;

    // Capture all UNISWAP V2 transactions.
    if let Some(to) = tx.to {
        if format!("{to:?}") == UNISWAP_V2_ROUTER {
            // Decode transaction input to figure out which tokens are getting swapped.
            let (_, decoded_input) = abi
                .tx_input_decoder
                .decode_input_from_hex(
                    tx.input
                        .to_string()
                        .split('x')
                        .collect::<Vec<&str>>()
                        .get(1)
                        .unwrap()
                        .trim(),
                )
                .expect("failed decoding input");

            // Extract the two tokens being swapped, their quantity, and value.
            let params_reader = decoded_input.reader();
            let from_token_qty = match params_reader.by_name.get("amountIn") {
                Some(amount_in) => match amount_in.value {
                    Value::Uint(amount_in, _) => amount_in,
                    _ => Err("unexpected type for `amountIn` value (not a Uint)")?,
                },
                None => Err("not a swap (`amountIn` absent)")?,
            };
            let to_token_qty = match params_reader.by_name.get("amountOutMin") {
                Some(amount_out) => match amount_out.value {
                    Value::Uint(amount_out, _) => amount_out,
                    _ => Err("unexpected type for `amountOutMin` value (not a Uint)")?,
                },
                None => Err("not a swap (`amountOutMin` absent)")?,
            };
            let (from_token_addr, to_token_addr) = match params_reader.by_name.get("path") {
                Some(path) => {
                    let mut token_addresses: Vec<H160> = Vec::new();
                    match path.value.clone() {
                        ethereum_abi::Value::Array(arr, _) => {
                            for val in arr {
                                match val {
                                    ethereum_abi::Value::Address(v) => {
                                        token_addresses.push(v.0.into())
                                    }
                                    _ => Err("value inside `path` Array is not Address")?,
                                }
                            }
                        }
                        _ => Err("unexpected DecodedParam type for `path`")?,
                    };
                    if token_addresses.len() != 2 {
                        Err("`path` param does not contain exactly 2 token addresses")?
                    }
                    (
                        token_addresses.get(0).unwrap().to_owned(),
                        token_addresses.get(1).unwrap().to_owned(),
                    )
                }
                None => Err("not a swap (`path` absent)")?,
            };

            // Get the contract information of the two tokens.
            let arc_ip = Arc::new(ip);
            let from_token_contract =
                Contract::new(from_token_addr, abi.erc20_token_abi.clone(), arc_ip.clone());
            let from_token_symbol: String = from_token_contract
                .method::<_, String>("symbol", ())
                .map_err(|err| format!("{err:?}"))?
                .call()
                .await
                .map_err(|err| format!("{err:?}"))?;
            let to_token_contract =
                Contract::new(to_token_addr, abi.erc20_token_abi.clone(), arc_ip.clone());
            let to_token_symbol: String = to_token_contract
                .method::<_, String>("symbol", ())
                .map_err(|err| format!("{err:?}"))?
                .call()
                .await
                .map_err(|err| format!("{err:?}"))?;

            // Get the information about the token pair's liquidity pool.
            let factory: Address = UNISWAP_V2_FACTORY.parse::<Address>().unwrap();
            let factory_contract =
                Contract::new(factory, abi.uniswap_v2_factory_abi.clone(), arc_ip.clone());
            let pair_addr: Address = factory_contract
                .method::<_, Address>("getPair", (from_token_addr, to_token_addr))
                .map_err(|err| format!("{err:?}"))?
                .call()
                .await
                .map_err(|err| format!("{err:?}"))?;
            let pair = UniswapV2Pair::new(pair_addr, arc_ip.clone());
            let (balance1, balance2, _) = pair
                .get_reserves()
                .call()
                .await
                .map_err(|err| format!("{err}"))?;

            // Use the router contract to estimate gas fees.
            let router: Address = UNISWAP_V2_ROUTER.parse::<Address>().unwrap();
            let router_contract =
                Contract::new(router, abi.uniswap_v2_router_abi.clone(), arc_ip.clone());

            // Estimate gas fees for approve.
            let approve_tx = pair.approve(
                router_contract.address(),
                ethers::types::U256(to_token_qty.0),
            );
            let approve_fee = approve_tx
                .estimate_gas()
                .await
                .map_err(|err| format!("{err:?}"))?;

            // // Estimate gas fees for adding and removing liquidity.
            // println!("To token: {to_token_symbol}");
            // let liq_fee = router_contract
            //     .method::<_, Address>(
            //         "removeLiquidity",
            //         (
            //             from_token_addr,
            //             to_token_addr,
            //             ethers::types::U256(to_token_qty.0),
            //             std::convert::Into::<ethers::types::U256>::into(0),
            //             std::convert::Into::<ethers::types::U256>::into(0),
            //             wallet.address(),
            //             ethers::types::U256::MAX,
            //         ),
            //     )
            //     .map_err(|err| format!("{err:?}"))?
            //     .estimate_gas()
            //     .await
            //     .map_err(|err| format!("{err:?}"))?;

            // Make the data available for consumption through the API.
            let mut data = data.lock().unwrap();
            data.push(DataPoint {
                tx_hash: tx_hash.to_string(),
                from_token_qty: from_token_qty.to_string(),
                from_token_symbol,
                to_token_qty: to_token_qty.to_string(),
                to_token_symbol,
                balance1,
                balance2,
                approve_fee: approve_fee.mul(2_i64).to_string(),
                // liq_fee: liq_fee.mul(2_i64).to_string(),
                liq_fee: "0".to_owned(),
                timestamp: chrono::Utc::now().timestamp_millis(),
            });
            drop(data);
        }
    }
    Ok(())
}

async fn try_get_transaction(
    tx_hash: H256,
    external_provider: Provider<Ws>,
    retry_times: u64,
    retry_period: u64,
) -> Result<Transaction, String> {
    let mut i: u64 = 0;
    loop {
        if i == retry_times {
            break Err("retry_times exceeded".to_owned());
        }
        let tx = external_provider.get_transaction(tx_hash).await.unwrap();
        if let Some(tx) = tx {
            break Ok(tx);
        } else {
            thread::sleep(Duration::from_millis(retry_period));
        }
        i += 1;
    }
}
