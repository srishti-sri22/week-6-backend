use axum::{Json, extract::Extension, http::{HeaderValue, StatusCode, header::SET_COOKIE}};
use axum::response::IntoResponse;
use mongodb::{Database, bson::{Document, doc}};
use std::sync::Arc;
use webauthn_rs::prelude::*;
use crate::{controllers::auth_controllers::models::{AuthFinishRequest, AuthResponse}, utils::session};

pub async fn auth_finish(
    Extension(db): Extension<Arc<Database>>,
    Extension(webauthn): Extension<Arc<Webauthn>>,
    Json(body): Json<AuthFinishRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    println!("=== Auth finish for: {} ===", body.username);

    let challenge_collection = db.collection::<Document>("auth_challenges");
    let challenge_doc = challenge_collection
        .find_one(doc! { "username": &body.username })
        .await
        .map_err(|e| {
            eprintln!("Failed to query challenge: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            eprintln!("No auth challenge found");
            StatusCode::BAD_REQUEST
        })?;

    let state_json = challenge_doc
        .get_str("state")
        .map_err(|e| {
            eprintln!("Failed to get state: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let auth_state: PasskeyAuthentication = serde_json::from_str(state_json)
        .map_err(|e| {
            eprintln!("Failed to deserialize auth state: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let credential_json: PublicKeyCredential = serde_json::from_value(body.credential)
        .map_err(|e| {
            eprintln!("Credential parsing error: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;

    let auth_result = webauthn
        .finish_passkey_authentication(&credential_json, &auth_state)
        .map_err(|e| {
            eprintln!("Auth verification failed: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    println!("✓ Authentication successful!");

    let credential_id = auth_result.cred_id().to_vec();
    let credential_id_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &credential_id);

    let passkeys_collection = db.collection::<Document>("passkeys");
    
    let passkey_doc = passkeys_collection
        .find_one(doc! { "credential_id": &credential_id_base64 })
        .await
        .map_err(|e| {
            eprintln!("Failed to find passkey: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            eprintln!("Passkey not found");
            StatusCode::NOT_FOUND
        })?;

    let user_id = passkey_doc
        .get_object_id("user_id")
        .map_err(|e| {
            eprintln!("Failed to get user_id: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let updated_passkey_json = serde_json::to_string(&auth_result)
        .map_err(|e| {
            eprintln!("Failed to serialize updated passkey: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    passkeys_collection
        .update_one(
            doc! { "credential_id": &credential_id_base64 },
            doc! { 
                "$set": { 
                    "passkey_data": updated_passkey_json,
                    "counter": auth_result.counter() as i32,
                    "last_used_at": chrono::Utc::now().to_rfc3339(),
                } 
            },
        )
        .await
        .map_err(|e| {
            eprintln!("Failed to update passkey: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("✓ Passkey counter updated");
    println!("User ID: {}", user_id);

    challenge_collection
        .delete_one(doc! { "username": &body.username })
        .await
        .map_err(|e| {
            eprintln!("Failed to delete challenge: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let token = session::create_token(&body.username)
        .map_err(|e| {
            eprintln!("Failed to create token: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("✓ Session token created");

    let response = AuthResponse {
        success: true,
        username: body.username.clone(),
        token: token.clone(),
        user_id: user_id.to_hex(),
    };

    let is_production = std::env::var("ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string()) == "production";
    
    let secure_flag = if is_production { " Secure;" } else { "" };
    
    let cookie_value = format!(
        "session_token={}; Path=/; HttpOnly;{} SameSite=Lax; Max-Age=86400",
        token, secure_flag
    );

    println!("✓ Setting session cookie");
    println!("=== Auth complete for: {} ===", body.username);

    let mut resp = Json(response).into_response();
    resp.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&cookie_value)
            .map_err(|e| {
                eprintln!("Failed to create cookie header: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
    );

    Ok(resp)
}