use acid4sigmas_models::{
    secrets::SECRET_KEY,
    utils::{
        jwt::{JwtToken, UserClaims},
        token_handler::{TokenHandler, UserTokenHandler},
    },
};
use actix_web::HttpResponse;

use crate::services::ws::get_ws_client;

pub async fn send_verify_email_service(token: &str) -> Result<HttpResponse, (String, u16)> {
    println!("hello world");

    let client_lock = get_ws_client().await;

    let mut client = client_lock.lock().await;

    let mut token_handler = UserTokenHandler::new(SECRET_KEY.get().unwrap(), client).await;

    let claims = token_handler.verify_token(&token).await?;

    println!("claims: {:?}", claims);

    Ok(HttpResponse::Ok().finish())
}
