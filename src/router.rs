use actix_web::web;

use crate::controller::auth::{login, register, send_verify_email};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(register)
            .service(login)
            .service(send_verify_email),
    );
}
