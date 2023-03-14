use std::{thread, time};

use ethers::providers::Provider;
use ethers::types::{Transaction, H256};
use ethers_providers::{Middleware, Ws};

pub async fn transaction_callback(provider: Provider<Ws>, tx: Result<Transaction, H256>) {
    let tx = match tx {
        Ok(tx) => tx,
        Err(tx_hash) => loop {
            // TODO: Add a timeout.
            let tx = provider.get_transaction(tx_hash).await.unwrap();
            if let Some(tx) = tx {
                break tx;
            } else {
                thread::sleep(time::Duration::from_millis(100));
            }
        },
    };
    // How do we get a lot of money for bundling?
    // What about flash loans?
    // TODO: Flashbots bundling.
    println!("{tx:?}");
}
