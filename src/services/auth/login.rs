use acid4sigmas_models::{
    models::{
        auth::{AuthUser, LoginIdentifier, LoginRequest},
        db::{DatabaseAction, DatabaseRequest, DatabaseResponse, Filters, WhereClause},
    },
    to_string_, token_response,
    utils::{
        hasher::Hasher,
        token_handler::{TokenHandler, UserTokenHandler},
    },
};
use actix_web::HttpResponse;
use serde_json::{json, Value};
use std::collections::HashMap;

use acid4sigmas_models::secrets::SECRET_KEY;

use crate::{services::ws::get_ws_client, USER_TOKEN_EXPIRY};

// return Result with T = HttpResposne and E = (String, u16).
// u16 is the http error code, String is the error message -> E = (String, u16)
pub async fn login_service(body: LoginRequest) -> Result<HttpResponse, (String, u16)> {
    let client_lock = get_ws_client().await;

    let mut client = client_lock.lock().await;

    // create a hashmap based of the identifier
    let identifier: HashMap<String, Value> = match body.identifier {
        LoginIdentifier::Email(email) => {
            let mut map = HashMap::new();
            map.insert(to_string_!("email"), json!(email));
            map
        }
        LoginIdentifier::Username(username) => {
            let mut map = HashMap::new();
            map.insert(to_string_!("username"), json!(username));
            map
        }
    };

    // get the auth_user based of their email or username to check if they exist
    let db_request = DatabaseRequest {
        table: to_string_!("auth_users"),
        action: DatabaseAction::Retrieve,
        filters: Some(Filters {
            where_clause: Some(WhereClause::Single(identifier)),
            ..Default::default()
        }),
        ..Default::default()
    };

    client
        .send(&db_request.to_string().map_err(|e| (e.to_string(), 500))?)
        .await
        .map_err(|e| (e.to_string(), 500))?;

    if let Some(message) = client.receive().await {
        let db_response = DatabaseResponse::<AuthUser>::parse(&message.to_string())
            .map_err(|e| (e.to_string(), 500))?;

        if db_response.is_error() {
            return Err((to_string_!(db_response.error_message().unwrap()), 500));
        }

        match db_response {
            /*DatabaseResponse::Status { status } => {
                println!("Operation successful: {}", status);
            }*/ // this will never be triggered due of the way how the db-api is designed.
            DatabaseResponse::Data(users) => {
                if users.is_empty() {
                    return Err((
                        to_string_!(
                            "no user exists with this email or username, please register first"
                        ),
                        404,
                    ));
                }

                // get the first user, in a real world application
                //only one user should be returned by your database
                // but our db api always returns an array for Data
                let user = users[0].clone();

                let verify_pw = Hasher::verify(&body.password, &user.password_hash)
                    .map_err(|e| (e.to_string(), 500))?;

                if verify_pw {
                    let token_handler = UserTokenHandler::new(SECRET_KEY.get().unwrap(), client);

                    let gen_token = token_handler
                        .await
                        .generate_token(user.uid, USER_TOKEN_EXPIRY)
                        .await
                        .map_err(|e| (e.to_string(), (500)))?;

                    return Ok(token_response!(gen_token));
                } else {
                    return Err((to_string_!("incorrect password."), 403));
                }
            }
            _ => (),
        }
    }

    Ok(HttpResponse::Ok().finish())
}
