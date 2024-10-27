use acid4sigmas_models::secrets::{DB_WS_URL, SECRET_KEY};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use acid4sigmas_models::utils::ws::WsClient;

pub static WS_CLIENT: OnceCell<Arc<Mutex<WsClient>>> = OnceCell::new();
pub static URL: OnceCell<Arc<RwLock<String>>> = OnceCell::new();

pub async fn init_ws_client(initial_url: &str) {
    URL.set(Arc::new(RwLock::new(initial_url.to_string())))
        .unwrap();
    let client = WsClient::new(initial_url).await.unwrap();
    let _ = WS_CLIENT.set(client);

    tokio::spawn(reconnect_task());
}

use tokio::time::{self, Duration};

async fn reconnect_task() {
    let mut interval = time::interval(Duration::from_secs(RECONNECT_AFTER));
    let url_lock = URL.get().unwrap().clone();
    let client_lock = WS_CLIENT.get().unwrap().clone();

    tokio::task::spawn(heartbeat());

    loop {
        interval.tick().await;

        let new_url = create_url().unwrap();

        *url_lock.write().await = new_url.clone();

        // lock() the client for mutable access
        let mut client = client_lock.lock().await;

        if let Err(e) = client.reconnect(&new_url).await {
            eprintln!("Failed to reconnect WebSocket: {}", e);
        } else {
            println!("WebSocket reconnected with new URL: {}", new_url);
        }
    }
}

pub async fn get_ws_client() -> Arc<Mutex<WsClient>> {
    WS_CLIENT.get().expect("Client not initialized").clone()
}

use crate::{HEARTBEAT_INTERVAL, RECONNECT_AFTER};
use acid4sigmas_models::utils::jwt::{BackendClaims, JwtToken};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn create_url() -> anyhow::Result<String> {
    let now = SystemTime::now();
    let duration_since_epoch = now.duration_since(UNIX_EPOCH)?;

    let timestamp: u64 = duration_since_epoch.as_secs();

    let exp = (timestamp + 3600) as usize;

    let claims = BackendClaims { timestamp, exp };

    let jwt_token =
        JwtToken::new(SECRET_KEY.get().unwrap()).create_jwt::<BackendClaims>(&claims)?;

    println!("{}", jwt_token);

    Ok(format!("{}?token={}", DB_WS_URL.get().unwrap(), jwt_token))
}

// send a heartbeat to keep the client alive.
async fn heartbeat() {
    let mut interval = time::interval(Duration::from_secs(HEARTBEAT_INTERVAL));

    loop {
        interval.tick().await;

        let client_lock = get_ws_client().await;

        let mut client = client_lock.lock().await;

        let _ = client.send_ping().await;

        if let Some(_message) = client.receive().await {}

        drop(client);
    }
}
