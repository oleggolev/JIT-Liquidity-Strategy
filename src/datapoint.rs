use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct DataPoint {
    pub tx_hash: String,
    pub from_token_qty: String,
    pub from_token_symbol: String,
    pub to_token_qty: String,
    pub to_token_symbol: String,
    pub balance1: u128,
    pub balance2: u128,
    pub approve_fee: String,
    pub liq_fee: String,
    pub timestamp: i64,
}
