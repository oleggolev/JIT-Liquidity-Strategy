use std::{thread, time};

use ethers::providers::Provider;
use ethers::types::{Transaction, TransactionRequest};
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

    // Determine whether this transaction is caused by a swap.
    // To read: https://ethereum.stackexchange.com/questions/130715/how-to-determine-whether-a-pending-tx-is-a-result-of-a-swap-in-uniswap

    // Reconstruct the transaction with as a request and pass it to the Ganache network.
    let tx_req = TransactionRequest::new()
        .from(tx.from)
        .to(tx.to.unwrap())
        .gas(tx.gas)
        .gas_price(tx.gas_price.unwrap())
        .value(tx.value);

    // Pass the reconstructed transaction to the Ganache network.
    ip.send_transaction(tx_req, None)
        .await
        .unwrap()
        .confirmations(1)
        .await
        .unwrap()
        .unwrap();
}
