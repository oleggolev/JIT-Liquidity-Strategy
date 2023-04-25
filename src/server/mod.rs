use actix_web::{
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Clone)]
pub struct DataPoint {
    pub tx_hash: String,
    pub from_token_qty: String,
    pub from_token_symbol: String,
    pub to_token_qty: String,
    pub to_token_symbol: String,
    pub balance1: u128,
    pub balance2: u128,
    pub timestamp: i64,
}

async fn get_live_data(data: web::Data<Arc<Mutex<Vec<DataPoint>>>>) -> HttpResponse {
    let mut data = data.lock().unwrap();
    let response = HttpResponse::Ok().json(&*data);
    data.clear();
    response
}

pub fn start(data: Arc<Mutex<Vec<DataPoint>>>) {
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(data.clone()))
            .service(web::resource("/live_data").route(web::get().to(get_live_data)))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run();
}