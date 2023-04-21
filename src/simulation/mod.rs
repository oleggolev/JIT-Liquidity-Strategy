mod callback;

use std::sync::Arc;

use ethers::{
    prelude::{gas_oracle::GasNow, MiddlewareBuilder},
    providers::{Middleware, Provider, Ws},
    signers::{LocalWallet, Signer},
    types::{BlockNumber, Filter, ValueOrArray, H160, H256},
    utils::{keccak256, Anvil, AnvilInstance},
};
use ethers_providers::StreamExt;
use ethers_providers::{rpc::transports::ws::WsClient, Http};
use tokio::sync::Mutex;

use crate::config::Config;

use self::callback::event_callback;

const INFURA_MAINNET_WS: &str = "wss://mainnet.infura.io/ws/v3";
const INFURA_TESTNET_WS: &str = "wss://goerli.infura.io/ws/v3";

const INFURA_MAINNET_HTTP: &str = "https://mainnet.infura.io/v3";
const INFURA_TESTNET_HTTP: &str = "https://goerli.infura.io/v3";

const LOCAL_WALLET_KEY: &str = "725fd1619b2653b7ff1806bf29ae11d0568606d83777afd5b1f2e649bd5132a9";
const LOCAL_WALLET_KEY_PREFIXED: &str =
    "0x725fd1619b2653b7ff1806bf29ae11d0568606d83777afd5b1f2e649bd5132a9";
const LOCAL_WALLET_BALANCE: u64 = i64::MAX as u64;
const UNISWAP_V2_ROUTER: &str = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D";

const GAS_LIMIT: u64 = 30000000;
// const GAS_PRICE: u64 = 10000000;

pub async fn start(config: Config) -> String {
    // Launch the local anvil network, instantiated as a fork of the connected provider node's blockchain.
    // let anvil = Anvil::new()
    //     .block_time(config.block_time)
    //     .fork(format!(
    //         "{}/{}",
    //         if config.is_test {
    //             INFURA_TESTNET_HTTP
    //         } else {
    //             INFURA_MAINNET_HTTP
    //         },
    //         config.api_key,
    //     ))
    //     .args([format!("--gas-limit={GAS_LIMIT}")])
    //     .spawn();

    let anvil = Anvil::new().fork("https://eth.llamarpc.com").spawn();
    let from = anvil.addresses()[0];
    // connect to the network
    let anvil_provider = Provider::<Http>::try_from(anvil.endpoint())
        .unwrap()
        .with_sender(from);

    let anvil_http_endpoint = anvil.endpoint();
    let wallet: LocalWallet = anvil.keys()[0].clone().into();
    let gas_oracle = GasNow::new();

    // Connect to the anvil node.
    // let anvil_provider = Provider::<Http>::try_from(anvil_http_endpoint.clone()).unwrap();
    // .with_sender(from);

    // Connect to the provider through which we will process real-time events on the local anvil network.
    let external_provider = Provider::<Ws>::connect(match config.provider {
        crate::config::Provider::Infura => format!(
            "{}/{}",
            if config.is_test {
                INFURA_TESTNET_WS
            } else {
                INFURA_MAINNET_WS
            },
            config.api_key
        ),
    })
    .await
    .unwrap();

    // Process and forward all events in a separate thread.
    // This also keeps the anvil instance alive since it moves.
    tokio::spawn(async move {
        forward(anvil, external_provider, anvil_provider, config, wallet).await;
    });

    anvil_http_endpoint
}

async fn forward(
    _anvil: AnvilInstance,
    ep: Provider<WsClient>,
    ip: Provider<Http>,
    config: Config,
    wallet: LocalWallet,
) {
    let mut events_stream = ep
        .subscribe_logs(
            &Filter::new()
                .from_block(
                    ep.clone()
                        .get_block(BlockNumber::Latest)
                        .await
                        .unwrap()
                        .unwrap()
                        .number
                        .unwrap(),
                )
                .topic0(ValueOrArray::Value(H256::from(keccak256(
                    "Swap(address,uint256,uint256,uint256,uint256,address)",
                )))),
        )
        .await
        .unwrap();

    let nonce = Arc::new(Mutex::new(0_u64));
    loop {
        // Clone provider nodes.
        let ep = ep.clone();
        let ip = ip.clone();
        let event = events_stream.next().await.unwrap();
        let config = config.clone();

        // Process each new event
        let wallet_cl = wallet.clone();
        let nonce_cl = Arc::clone(&nonce);
        tokio::spawn(async move {
            _ = event_callback(
                event.clone(),
                ep,
                ip,
                config.tx_retry_times,
                config.tx_retry_interval,
                wallet_cl,
                nonce_cl,
            )
            .await
            .map_err(|err| println!("could not process event {}: {:?}", event.address, err));
        });
    }
}
