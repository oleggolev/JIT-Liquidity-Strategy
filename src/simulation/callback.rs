use std::{borrow::BorrowMut, sync::Arc, thread, time};

use ethers::{
    contract::abigen,
    core::types::{Address, Log, Transaction, H256, U256},
    middleware::SignerMiddleware,
    prelude::{gas_oracle::GasNow, MiddlewareBuilder},
    providers::{Middleware, Provider, Ws},
    signers::{LocalWallet, Signer},
    types::H160,
};
use rand::Rng;

use ethers_providers::Http;
use eyre::Result;
use tokio::sync::Mutex;

use crate::simulation::{GAS_LIMIT, UNISWAP_V2_ROUTER};

abigen!(
    UniswapV2Router,
    r#"[
        removeLiquidity(address tokenA, address tokenB, uint liquidity,uint amountAMin, uint amountBMin, address to, uint deadline) external returns (uint amountA, uint amountB)
        addLiquidity(address tokenA, address tokenB, uint amountADesired, uint amountBDesired, uint amountAMin, uint amountBMin, address to, uint deadline) external returns (uint amountA, uint amountB, uint liquidity)
    ]"#,
);

abigen!(
    UniswapV2Pair,
    r#"[
        approve(address,uint256)(bool)
        getReserves()(uint112,uint112,uint32)
        token0()(address)
        token1()(address)
    ]"#
);

// One per block.
fn is_desirable_transaction(_tx: Transaction) -> bool {
    rand::thread_rng().gen_bool(0.05)
}

async fn try_get_transaction(
    hash: H256,
    ep: Provider<Ws>,
    retry_times: u64,
    retry_period: u64,
) -> Result<Transaction, String> {
    let mut i: u64 = 0;
    loop {
        if i == retry_times {
            break Err("retry_times exceeded".to_owned());
        }
        let tx = ep.get_transaction(hash).await.unwrap();
        if let Some(tx) = tx {
            break Ok(tx);
        } else {
            thread::sleep(time::Duration::from_millis(retry_period));
        }
        i += 1;
    }
}

pub async fn event_callback(
    event: Log,
    ep: Provider<Ws>,
    ip: Provider<Http>,
    retry_times: u64,
    retry_period: u64,
    wallet: LocalWallet,
    nonce: Arc<Mutex<u64>>,
) -> Result<(), String> {
    // If the transaction was not posted to the node yet, attempt to acquire it until it's posted.
    let tx = try_get_transaction(
        event.transaction_hash.unwrap(),
        ep.clone(),
        retry_times,
        retry_period,
    )
    .await?;

    // Decide whether the transaction is suitable for a JIT-attack.
    if !is_desirable_transaction(tx) {
        return Ok(());
    }

    // let gas_oracle = GasNow::new();
    // let addr = wallet.address();

    // // Generate an Arc-ed provider to use with the web3 API.
    // let chain_id = ip.get_chainid().await.map_err(|err| format!("{err}"))?;
    // let wallet = wallet.with_chain_id(chain_id.as_u64());
    // let in_provider = Arc::new(SignerMiddleware::new(ip.clone(), wallet.clone()));
    let in_provider = Arc::new(ip.clone());
    // let custom_provider = in_provider
    //     .clone()
    //     .gas_oracle(gas_oracle)
    //     .with_signer(wallet)
    //     .nonce_manager(addr);

    // Get the swapped pair of tokens.
    let pair = UniswapV2Pair::new(event.address, ip.into());

    // Instantiate the UniswapV2 router for controlling asset liquidity.
    let router = UniswapV2Router::new(
        UNISWAP_V2_ROUTER
            .parse::<Address>()
            .map_err(|err| format!("{err}"))?,
        in_provider,
    );

    // Figure out how much of each token exists in the pool.
    let (balance1, balance2, _) = pair
        .get_reserves()
        .call()
        .await
        .map_err(|err| format!("{err}"))?;
    println!("Reserves (token A, Token B): ({balance1}, {balance2})");

    // Define and approve the trasaction to add liquidity.

    // Define and approve the transaction to remove liquidity.
    // let price = if balance1 > balance2 {
    //     1000 * balance1 / balance2
    // } else {
    //     1000 * balance2 / balance1
    // } / 1000;
    // println!("token1 / token2 price = {price}");
    let liquidity = 10.into();

    // // Get the unique nonce.
    // let mut nonce = nonce.lock().await;
    // let this_nonce = *nonce;
    // *nonce += 1;
    // drop(nonce);

    println!("approve for: remove_liquidity");
    let approve_tx = pair.approve(router.address(), liquidity);
    let gas_price = approve_tx
        .estimate_gas()
        .await
        .map_err(|err| format!("approve: {err}"))?;
    let receipt = approve_tx
        // .gas_price(in_provider.get_gas_price().await.unwrap())
        // .gas(GAS_LIMIT)
        .send()
        .await
        .map_err(|err| format!("approve: {err}"))?
        .await
        .map_err(|err| format!("send: {err}"))?
        .expect("no receipt found");
    println!("approve for: remove_liquidity successful");
    println!("{receipt:?}");

    // Actually remove the liquidity.
    println!("remove_liquidity({liquidity})");
    let token0 = pair
        .token_0()
        .call()
        .await
        .map_err(|err| format!("get token0: {err}"))?;
    let token1 = pair
        .token_1()
        .call()
        .await
        .map_err(|err| format!("get token1: {err}"))?;
    let receipt = router
        .remove_liquidity(
            token0,
            token1,
            liquidity,
            0.into(),
            0.into(),
            wallet.address(),
            U256::MAX,
        )
        .send()
        .await
        .map_err(|err| format!("remove liquidity: {err}"))?
        .await
        .map_err(|err| format!("send remove liquidity: {err}"))?
        .expect("no receipt for remove_liquidity");
    println!("remove_liquidity({liquidity}) successful");
    println!("receipt: {receipt:?}");

    // let tx_remove_liquidity = router.remove_liquidity(token1, token_b, liquidity, amount_a_min, amount_b_min, to, deadline)

    // Bundle the transactions into a flashbots bundle.

    // Send the flashbots bundle to the intenal Anvil provider to ensure that the transaction works.
    println!("YEET");
    Ok(())
}
