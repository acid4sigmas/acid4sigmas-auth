use acid4sigmas_models::db::TableModel;
use acid4sigmas_models::error_response;
use acid4sigmas_models::models::api::users::User;
use acid4sigmas_models::models::auth::AuthUser;
use acid4sigmas_models::models::db::{
    DatabaseAction, DatabaseRequest, DatabaseResponse, Filters, WhereClause,
};
use acid4sigmas_models::utils::hasher::Hasher;
use acid4sigmas_models::utils::timer::Timer;
use acid4sigmas_models::utils::util::generate_uid;
use acid4sigmas_models::{models::auth::RegisterRequest, to_string_};
use actix_web::HttpResponse;

use crate::services::ws::get_ws_client;

// return Result with T = HttpResposne and E = (String, u16).
// u16 is the http error code, String is the error message -> E = (String, u16)
pub async fn register_service(body: RegisterRequest) -> Result<HttpResponse, (String, u16)> {
    body.validate().map_err(|e| (e, 400))?;

    let generated_uid = generate_uid();

    let mut auth_user = AuthUser {
        uid: generated_uid,
        email: body.email.clone(),
        email_verified: false,
        username: body.username.clone(),
        password_hash: to_string_!(""), // insert no data here to save performance cost.
    };

    let timer = Timer::new(); // start a timer [debugging]

    let client_lock = get_ws_client().await;

    let mut client = client_lock.lock().await; // prepare the ws client

    let db_request = DatabaseRequest {
        table: to_string_!("auth_users"),
        action: DatabaseAction::Retrieve,
        values: None,
        filters: Some(Filters {
            where_clause: Some(WhereClause::Or(
                auth_user.get_keys_as_hashmap(vec!["email", "username"]),
            )), // if here occurs an typo, the value will be skipped!, make sure that the entered key matches 1:1 the desired key defined in the hashmap
            ..Default::default()
        }),
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
                if !users.is_empty() {
                    return Err((
                        to_string_!("an user with this email or username already exists."),
                        409,
                    ));
                }
                // handle the data returned by the database
            }
            _ => (),
        }
    }

    auth_user.password_hash = Hasher::hash(&body.password).map_err(|e| (e.to_string(), 500))?; // hash a password for the user. hashing now will save us performance in the long run

    let db_request = DatabaseRequest {
        table: to_string_!("auth_users"),
        action: DatabaseAction::Insert,
        values: Some(auth_user.as_hash_map()),
        ..Default::default()
    }; // initialize a new database request to insert the data

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

        // here we dont need to match the db_response, because we know it is either {"status": "success"} or {"error": "some error"}
    }

    //// now we will create the actual user ////

    let user = User {
        uid: generated_uid,
        email: body.email,
        email_verified: false,
        owner: false,
        username: body.username,
    };

    let db_request = DatabaseRequest {
        table: to_string_!("users"),
        action: DatabaseAction::Insert,
        values: Some(user.as_hash_map()),
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
    }

    println!("{} ms", timer.elapsed_as_millis());

    Ok(HttpResponse::Ok().finish())
}
