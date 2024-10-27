mod controller;
mod router;
mod services;

use crate::services::ws::create_url;
use acid4sigmas_models::secrets::init_secrets;
use actix_web::{App, HttpServer};
use services::ws::init_ws_client;

pub const RECONNECT_AFTER: u64 = 360;
pub const HEARTBEAT_INTERVAL: u64 = 30;
pub const USER_TOKEN_EXPIRY: usize = 31536000; // a year in seconds

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_secrets("Secrets.toml");

    let url = create_url().unwrap();

    init_ws_client(&url).await;

    HttpServer::new(|| App::new().configure(router::routes))
        .bind(("127.0.0.1", 1234))?
        .run()
        .await
}
