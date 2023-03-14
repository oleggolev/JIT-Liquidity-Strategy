use std::{num::ParseIntError, thread};

use callback::transaction_callback;
use ethers::{
    providers::{Middleware, Provider, StreamExt, Ws},
    types::H256,
    utils::Ganache,
};
use ethers_providers::stream::tx_stream::GetTransactionError;

use crate::config::read_config;

mod callback;
mod config;

const DEFAULT_CONFIG_PATH: &str = "config.yaml";

const INFURA_MAINNET: &str = "wss://mainnet.infura.io/ws/v3";
const INFURA_TESTNET: &str = "wss://goerli.infura.io/ws/v3";
const LLAMA_NODES_MAINNET: &str = "wss://eth.llamarpc.com";

#[tokio::main]
async fn main() {
    let config = read_config(
        std::env::args()
            .collect::<Vec<String>>()
            .get(1)
            .unwrap_or(&DEFAULT_CONFIG_PATH.to_owned()),
    );

    let provider = Provider::<Ws>::connect(match config.provider {
        config::Provider::LlamaNodes => {
            if config.is_test {
                unimplemented!(); // LlamaNodes does not yet provide access to the Goerli testnet
            } else {
                LLAMA_NODES_MAINNET.to_string()
            }
        }
        config::Provider::Infura => format!(
            "{}/{}",
            if config.is_test {
                INFURA_TESTNET
            } else {
                INFURA_MAINNET
            },
            config.api_key.unwrap()
        ),
        config::Provider::Ganache => {
            let ganache = Ganache::new()
                .fork("https://goerli.infura.io/v3/4250fb4ff74c4fa5b27e19fa82451925")
                .spawn();
            let ganache_ws_endpoint = ganache.ws_endpoint();
            thread::spawn(move || {
                println!("Running a local Ganache node: {}", ganache.ws_endpoint());
                thread::park();
            });
            ganache_ws_endpoint
        }
    })
    .await
    .unwrap();

    let mut tx_stream = provider
        .subscribe_pending_txs()
        .await
        .unwrap()
        .transactions_unordered(10)
        .fuse();

    pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
            .collect()
    }

    loop {
        println!("HERE");
        let latest_block_number = provider.get_block_number().await.unwrap();
        println!("{:?}", provider.get_block(latest_block_number).await);
        println!(
            "{:?}",
            provider
                .get_transaction(H256::from(
                    std::convert::TryInto::<[u8; 32]>::try_into(
                        decode_hex(
                            "a62cff0c2868bab929de26c079d4356a72998eb4a1dc48358ccebc4e95b85977"
                        )
                        .unwrap()
                        .as_slice()
                    )
                    .unwrap()
                ))
                .await
        );

        let pcc = provider.clone();
        let tx = tx_stream.next().await.unwrap().map_err(|err| match err {
            GetTransactionError::ProviderError(_, _) => panic!("provider failed"),
            GetTransactionError::NotFound(hash) => hash,
        });
        tokio::spawn(async move {
            transaction_callback(pcc, tx).await;
        });
    }
}
