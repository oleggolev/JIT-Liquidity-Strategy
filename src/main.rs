mod config;
mod simulation;

use crate::config::read_config;
use std::thread;

const DEFAULT_CONFIG_PATH: &str = "config.yaml";

#[tokio::main]
async fn main() {
    let config = read_config(
        std::env::args()
            .collect::<Vec<String>>()
            .get(1)
            .unwrap_or(&DEFAULT_CONFIG_PATH.to_owned()),
    );
    let anvil_ws_endpoint = simulation::start(config).await;
    println!("Anvil blockchain simulation node started at: {anvil_ws_endpoint}");
    thread::park();
}
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
