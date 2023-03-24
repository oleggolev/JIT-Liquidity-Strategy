use std::{thread, time};

use ethers::providers::Provider;
use ethers::types::Transaction;
use ethers_providers::stream::GetTransactionError;
use ethers_providers::{Middleware, Ws};

pub async fn transaction_callback(
    ep: Provider<Ws>,
    ip: Provider<Ws>,
    tx: Result<Transaction, GetTransactionError>,
    retry_times: u64,
    retry_period: u64,
) {
    // If the transaction was not yet known to the node, request it again.
    let tx = match tx {
        Ok(tx) => tx,
        Err(err) => match err {
            GetTransactionError::ProviderError(_, _) => panic!("external provider failed"),
            GetTransactionError::NotFound(hash) =>
            // Attempt to get the transaction `retry` times before failing this thread.
            {
                let mut i: u64 = 0;
                loop {
                    if i == retry_times {
                        return;
                    }
                    let tx = ep.get_transaction(hash).await.unwrap();
                    if let Some(tx) = tx {
                        break tx;
                    } else {
                        thread::sleep(time::Duration::from_millis(retry_period));
                    }
                    i += 1;
                }
            }
        },
    };

    // Pass the transaction to the Ganache network.
    let _receipt = ip
        .send_transaction(&tx, None)
        .await
        .unwrap()
        .confirmations(3)
        .await
        .unwrap()
        .unwrap();
}
