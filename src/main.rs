// mod config;
// mod simulation;

// use crate::config::read_config;
// use std::thread;

// const DEFAULT_CONFIG_PATH: &str = "config.yaml";

// #[tokio::main]
// async fn main() {
//     let config = read_config(
//         std::env::args()
//             .collect::<Vec<String>>()
//             .get(1)
//             .unwrap_or(&DEFAULT_CONFIG_PATH.to_owned()),
//     );
//     let anvil_ws_endpoint = simulation::start(config).await;
//     println!("Anvil blockchain simulation node started at: {anvil_ws_endpoint}");
//     thread::park();
// }

///////////////////////////////////////////////////////////////////////////////////////////

// use ethers::{
//     core::{types::TransactionRequest, utils::Anvil},
//     providers::{Http, Middleware, Provider},
// };
// use eyre::Result;

// #[tokio::main]
// async fn main() -> Result<()> {
//     // fork mainnet
//     let anvil = Anvil::new().fork("https://eth.llamarpc.com").spawn();
//     let from = anvil.addresses()[0];
//     // connect to the network
//     let provider = Provider::<Http>::try_from(anvil.endpoint())
//         .unwrap()
//         .with_sender(from);

//     // craft the transaction
//     let tx = TransactionRequest::new().to("vitalik.eth").value(100_000);

//     // send it!
//     let receipt = provider
//         .send_transaction(tx, None)
//         .await?
//         .await?
//         .ok_or_else(|| eyre::format_err!("tx dropped from mempool"))?;
//     let tx = provider.get_transaction(receipt.transaction_hash).await?;

//     println!("{}", serde_json::to_string(&tx)?);
//     println!("{}", serde_json::to_string(&receipt)?);

//     Ok(())
// }

/////////////////////////////////////////////////////////////////////////////////////////////////////////

use std::sync::Arc;

use ethers::{
    contract::abigen,
    core::types::{Address, U256},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    utils::Anvil,
};
use eyre::Result;

abigen!(
    UniswapV2Router,
    r#"[
        removeLiquidity(address tokenA,address tokenB, uint liquidity,uint amountAMin, uint amountBMin, address to, uint ) external returns (uint amountA, uint amountB)
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

#[tokio::main]
async fn main() {
    // fork mainnet
    let anvil = Anvil::new().fork("https://eth.llamarpc.com").spawn();
    let from = anvil.addresses()[0];
    // connect to the network
    let provider = Arc::new({
        let provider = Provider::<Http>::try_from(anvil.endpoint())
            .unwrap()
            .with_sender(from);
        let chain_id = provider.get_chainid().await.unwrap();
        let wallet: LocalWallet = anvil.keys()[0].clone().into();
        let wallet = wallet.with_chain_id(chain_id.as_u64());
        SignerMiddleware::new(provider, wallet)
    });

    let pair = "0xc6a45ecdc8bcef94c476647be1303fd83d438cd0"
        .parse::<Address>()
        .unwrap();
    let pair = UniswapV2Pair::new(pair, provider.clone());

    let router = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"
        .parse::<Address>()
        .unwrap();
    let router = UniswapV2Router::new(router, provider.clone());

    let (reserve0, reserve1, _) = pair.get_reserves().call().await.unwrap();

    println!("Reserves (token A, Token B): ({reserve0}, {reserve1})");

    let price = if reserve0 > reserve1 {
        1000 * reserve0 / reserve1
    } else {
        1000 * reserve1 / reserve0
    } / 1000;
    println!("token0 / token1 price = {price}");

    let liquidity = 1000000.into();

    println!("Approving the transaction!");
    let receipt = pair
        .approve(
            "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"
                .parse::<Address>()
                .unwrap(),
            liquidity,
        )
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .expect("no receipt found");
    // let receipt = pair
    //     .approve(pair.address(), liquidity)
    //     .send()
    //     .await
    //     .unwrap()
    //     .await
    //     .unwrap()
    //     .expect("no receipt found");
    // let receipt = pair
    //     .approve(provider.address(), liquidity)
    //     .send()
    //     .await
    //     .unwrap()
    //     .await
    //     .unwrap()
    //     .expect("no receipt found");

    println!("contract approved succesfully!");
    println!("{receipt:?}");

    println!("Removing {liquidity} liquidity!");

    let token0 = pair.token_0().call().await.unwrap();
    let token1 = pair.token_1().call().await.unwrap();

    let receipt = router
        .add_liquidity(
            token0,
            token1,
            1.into(),
            1.into(),
            0.into(),
            0.into(),
            provider.address(),
            U256::MAX,
        )
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .expect("no receipt for remove_liquidity");

    // let receipt = router
    //     .remove_liquidity(
    //         token0,
    //         token1,
    //         liquidity,
    //         0.into(),
    //         0.into(),
    //         provider.address(),
    //         U256::MAX,
    //     )
    //     .send()
    //     .await
    //     .unwrap()
    //     .await
    //     .unwrap()
    //     .expect("no receipt for remove_liquidity");
    println!("liquidity removed succesfully!");
    println!("{receipt:?}");
}
