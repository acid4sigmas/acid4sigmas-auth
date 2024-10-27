use crate::services::auth::{login::login_service, registration::register_service};
use acid4sigmas_models::{
    error_response,
    models::auth::{LoginRequest, RegisterRequest},
};
use actix_web::{http::header::AUTHORIZATION, post, web, HttpRequest, HttpResponse};

#[post("/register")]
pub async fn register(req_body: web::Json<RegisterRequest>) -> HttpResponse {
    let body: RegisterRequest = req_body.into_inner();

    match register_service(body).await {
        Ok(response) => response,
        Err((error_msg, error_code)) => return error_response!(error_code, error_msg),
    }
}

#[post("/login")]
pub async fn login(req_body: web::Json<LoginRequest>) -> HttpResponse {
    let body: LoginRequest = req_body.into_inner();

    match login_service(body).await {
        Ok(response) => response,
        Err((error_msg, error_code)) => return error_response!(error_code, error_msg),
    }
}

#[post("/send_verify_email")]
pub async fn send_verify_email(req: HttpRequest) -> HttpResponse {
    match req.headers().get(AUTHORIZATION) {
        Some(header_value) => match header_value.to_str() {
            Ok(_token) => {}
            Err(e) => return error_response!(400, &format!("failed to convert: {}", e)),
        },
        None => return error_response!(401, "Authorization header missing"),
    }

    HttpResponse::Ok().finish()
}
