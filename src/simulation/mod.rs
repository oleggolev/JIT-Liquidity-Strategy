mod callback;

use ethers::{
    providers::{Middleware, Provider, Ws},
    utils::{Ganache, GanacheInstance},
};
use ethers_providers::rpc::transports::ws::WsClient;
use ethers_providers::StreamExt;

use crate::config::Config;

use self::callback::transaction_callback;

const INFURA_MAINNET_WS: &str = "wss://mainnet.infura.io/ws/v3";
const INFURA_TESTNET_WS: &str = "wss://goerli.infura.io/ws/v3";

const INFURA_MAINNET_HTTP: &str = "https://mainnet.infura.io/v3";
const INFURA_TESTNET_HTTP: &str = "https://goerli.infura.io/v3";

pub async fn start(config: Config) -> String {
    // Launch the local Ganache network, instantiated as a fork of the connected provider node's blockchain.
    let ganache = Ganache::new()
        .block_time(config.block_time)
        .fork(format!(
            "{}/{}",
            if config.is_test {
                INFURA_TESTNET_HTTP
            } else {
                INFURA_MAINNET_HTTP
            },
            config.api_key,
        ))
        .spawn();
    let ganache_ws_endpoint = ganache.ws_endpoint();

    // Connect to the ganache node.
    let ganache_provider = Provider::<Ws>::connect(ganache_ws_endpoint.clone())
        .await
        .unwrap();

    // Connect to the provider through which we will process real-time transactions on the local Ganache network.
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

    // Process and forward all transactions in a separate thread.
    // This also keeps the ganache instance alive since it moves.
    tokio::spawn(async move {
        forward(ganache, external_provider, ganache_provider, config).await;
    });

    ganache_ws_endpoint
}

async fn forward(
    _ganache: GanacheInstance,
    ep: Provider<WsClient>,
    ip: Provider<WsClient>,
    config: Config,
) {
    let mut tx_stream = ep
        .subscribe_pending_txs()
        .await
        .unwrap()
        .transactions_unordered(10)
        .fuse();
    let accs = ip.get_accounts().await.unwrap();
    loop {
        // Clone provider nodes.
        let ep = ep.clone();
        let ip = ip.clone();
        let tx = tx_stream.next().await.unwrap();

        // Process each new transaction.
        tokio::spawn(async move {
            transaction_callback(ep, ip, tx, config.tx_retry_times, config.tx_retry_interval).await;
        });
    }
}
