use std::{sync::Arc, thread, time};

use ethers::{
    contract::abigen,
    core::types::{Address, Log, Transaction, H256},
    middleware::SignerMiddleware,
    prelude::k256::SecretKey,
    providers::{Middleware, Provider, Ws},
    signers::{LocalWallet, Signer},
};

use ethers_flashbots::FlashbotsMiddleware;
use ethers_providers::Http;
use eyre::Result;

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

fn is_desirable_transaction(_tx: Transaction) -> bool {
    true
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
    private_key: SecretKey,
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

    // Generate an Arc-ed provider to use with the web3 API.
    let chain_id = ep.get_chainid().await.map_err(|err| format!("{err}"))?;
    let wallet = LocalWallet::from(private_key).with_chain_id(chain_id.as_u64());
    let ex_provider = Arc::new(SignerMiddleware::new(ip, wallet));

    // // Add signer and Flashbots middleware
    // let provider = Provider::<Http>::try_from(ip.url().as_str())?;
    // let flashbots_middleware = SignerMiddleware::new(
    //     FlashbotsMiddleware::new(provider, Url::parse("https://relay.flashbots.net")?, wallet),
    //     wallet,
    // );

    // Get the swapped pair of tokens.
    let pair = UniswapV2Pair::new(event.address, ex_provider.clone());
    let token0 = pair
        .token_0()
        .call()
        .await
        .map_err(|err| format!("{err}"))?;
    let token1 = pair
        .token_1()
        .call()
        .await
        .map_err(|err| format!("{err}"))?;

    // Instantiate the UniswapV2 router for controlling asset liquidity.
    let router = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"
        .parse::<Address>()
        .map_err(|err| format!("{err}"))?;
    let router = UniswapV2Router::new(router, ex_provider);

    // Figure out how much of each token exists in the pool.
    let (balance1, balance2, _) = pair
        .get_reserves()
        .call()
        .await
        .map_err(|err| format!("{err}"))?;
    println!("Reserves (token A, Token B): ({balance1}, {balance2})");

    // Define and approve the trasaction to add liquidity.

    // Define and approve the transaction to remove liquidity.

    // let tx_remove_liquidity = router.remove_liquidity(token1, token_b, liquidity, amount_a_min, amount_b_min, to, deadline)

    // Bundle the transactions into a flashbots bundle.

    // Send the flashbots bundle to the intenal Ganache provider to ensure that the transaction works.

    Ok(())
}

// Remove liquidity.
//     let price =
//         if reserve0 > reserve1 { 1000 * reserve0 / reserve1 } else { 1000 * reserve1 / reserve0 } /
//             1000;
//     println!("token0 / token1 price = {price}");

//     let liquidity = 100.into();

//     println!("Approving the transaction!");
//     let receipt =
//         pair.approve(router.address(), liquidity).send().await?.await?.expect("no receipt found");
//     println!("contract approved succesfully!");
//     println!("{receipt:?}");

//     println!("Removing {liquidity} liquidity!");
//     let token0 = pair.token_0().call().await?;= event.topic[1]
//     let token1 = pair.token_1().call().await?;= event.topic[2]
//     let receipt = router
//         .remove_liquidity(
//             token0,
//             token1,
//             liquidity,
//             0.into(),
//             0.into(),
//             provider.address(),
//             U256::MAX,
//         )
//         .send()
//         .await?
//         .await?
//         .expect("no receipt for remove_liquidity");
//     println!("liquidity removed succesfully!");
//     println!("{receipt:?}");

//     Ok(())
