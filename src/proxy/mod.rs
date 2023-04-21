use std::{thread, time::Duration};

use ethers::{
    providers::{Middleware, Provider, Ws},
    types::{Transaction, H256},
    utils::Anvil,
};
use ethers_providers::StreamExt;
use ethers_providers::{rpc::transports::ws::WsClient, Http};

use crate::config::Config;

const NUM_ACCOUNTS: u64 = 100;

const LLAMA_WS: &str = "wss://eth.llamarpc.com";
const LLAMA_HTTP: &str = "https://eth.llamarpc.com";

const INFURA_WS: &str = "wss://mainnet.infura.io/ws/v3";
const INFURA_HTTP: &str = "https://mainnet.infura.io/ws/v3";

pub async fn start(config: Config) {
    // Launch the local anvil network, instantiated as a fork of the connected provider node's blockchain.
    let anvil = Anvil::new()
        .block_time(config.block_time)
        .fork(LLAMA_HTTP)
        .arg(format!("--accounts={NUM_ACCOUNTS}"))
        .spawn();
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

    // Forward every pending transaction to the local Anvil node.
    loop {
        let tx_hash = tx_stream.next().await.unwrap();
        let ep = external_provider.clone();
        let ip = anvil_provider.clone();
        let config = config.clone();

        tokio::spawn(async move {
            let _ = forward(tx_hash, ep, ip, config)
                .await
                .map_err(|err| println!("{err}"));
        });
    }
}

async fn forward(
    tx_hash: H256,
    ep: Provider<WsClient>,
    ip: Provider<Http>,
    config: Config,
) -> Result<(), String> {
    let tx =
        try_get_transaction(tx_hash, ep, config.tx_retry_times, config.tx_retry_period).await?;
    todo!();
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
