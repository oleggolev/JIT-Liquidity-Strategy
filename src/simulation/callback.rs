use std::{thread, time};

use ethers::providers::Provider;
use ethers::types::{Log, Transaction, H256};
use ethers_providers::{Middleware, Ws};

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
) {
    let tx = try_get_transaction(
        event.transaction_hash.unwrap(),
        ep,
        retry_times,
        retry_period,
    )
    .await;

    if let Some(tx) = tx {
        println!("{event:?}");
        println!("{tx:?}");
    }

    // Isolate the two tokens present in the transaction

    // Perform the JIT attack using Flashbots.
}
