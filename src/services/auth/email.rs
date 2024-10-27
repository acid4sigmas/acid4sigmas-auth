/*use acid4sigmas_models::{
    secrets::SECRET_KEY,
    utils::jwt::{JwtToken, UserClaims},
};
use actix_web::HttpResponse;

pub async fn send_verify_email_service(token: &str) -> Result<HttpResponse, (String, u16)> {
    let jwt_token = JwtToken::new(SECRET_KEY.get().unwrap());

    let claims: UserClaims = jwt_token
        .decode_jwt::<UserClaims>(&token)
        .map_err(|e| (format!("failed to decode token: {}", e), 500))?;

    Ok(HttpResponse::Ok().finish())
}
*/
