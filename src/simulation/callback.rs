use std::{sync::Arc, thread, time};

use ethers::{
    contract::abigen,
    core::types::{Address, Log, Transaction, H256, U256},
    middleware::SignerMiddleware,
    prelude::k256::{Secp256k1, SecretKey},
    providers::{Middleware, Provider, Ws},
    signers::{LocalWallet, Signer},
};

use eyre::Result;

abigen!(
    UniswapV2Router,
    r#"[
        removeLiquidity(address tokenA,address tokenB, uint liquidity,uint amountAMin, uint amountBMin, address to, uint ) external returns (uint amountA, uint amountB)
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

pub async fn try_get_transaction(
    hash: H256,
    ep: Provider<Ws>,
    retry_times: u64,
    retry_period: u64,
) -> Option<Transaction> {
    let mut i: u64 = 0;
    loop {
        if i == retry_times {
            break None;
        }
        let tx = ep.get_transaction(hash).await.unwrap();
        if tx.is_some() {
            break tx;
        } else {
            thread::sleep(time::Duration::from_millis(retry_period));
        }
        i += 1;
    }
}

pub async fn event_callback(
    event: Log,
    ep: Provider<Ws>,
    ip: Provider<Ws>,
    retry_times: u64,
    retry_period: u64,
    private_key: SecretKey,
) -> Result<()> {
    let tx = try_get_transaction(
        event.transaction_hash.unwrap(),
        ep.clone(),
        retry_times,
        retry_period,
    )
    .await;

    let ex_provider = Arc::new({
        let provider = ep.clone();
        let chain_id = provider.get_chainid().await?;
        let wallet = LocalWallet::from(private_key.clone()).with_chain_id(chain_id.as_u64());

        SignerMiddleware::new(provider, wallet)
    });

    // Isolate the two tokens present in the transaction
    // Perform the JIT attack using Flashbots.
    let pair = event.address;
    let pair = UniswapV2Pair::new(pair, ex_provider.clone());

    let router = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D".parse::<Address>()?;
    let router = UniswapV2Router::new(router, ex_provider.clone());

    let (reserve0, reserve1, _) = pair.get_reserves().call().await?;

    println!("Reserves (token A, Token B): ({reserve0}, {reserve1})");

    Ok(())
}
