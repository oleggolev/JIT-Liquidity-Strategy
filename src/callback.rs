use std::{thread, time};

use ethers::providers::Provider;
use ethers::types::{Transaction, H256};
use ethers_providers::{Middleware, Ws};

pub async fn transaction_callback(provider: Provider<Ws>, tx: Result<Transaction, H256>) {
    let tx = match tx {
        Ok(tx) => tx,
        Err(tx_hash) => loop {
            let tx = provider.get_transaction(tx_hash).await.unwrap();
            if let Some(tx) = tx {
                break tx;
            } else {
                thread::sleep(time::Duration::from_millis(100));
            }
        },
    };
    println!("{tx:?}");
}
