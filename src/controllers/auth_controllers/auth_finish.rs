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
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    let state_json = challenge_doc
        .get_str("state")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let auth_state: PasskeyAuthentication =
        serde_json::from_str(state_json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let credential_json: PublicKeyCredential =
        serde_json::from_value(body.credential).map_err(|e| {
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
    let credential_id_base64 =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &credential_id);

    let passkeys_collection = db.collection::<Document>("passkeys");
    
    // Get the passkey document to retrieve user_id
    let passkey_doc = passkeys_collection
        .find_one(doc! { "credential_id": &credential_id_base64 })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Update the passkey
    passkeys_collection
        .update_one(
            doc! { "credential_id": &credential_id_base64 },
            doc! { "$set": { "passkey": serde_json::to_string(&auth_result).unwrap() } },
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    challenge_collection
        .delete_one(doc! { "username": &body.username })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // FIXED: Removed duplicate token creation
    let token = session::create_token(&body.username)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    println!("=== Auth complete for: {} ===", body.username);
    println!("Setting session_token cookie with value: {}", &token[..20]);

    // FIXED: Get user_id from passkey document, not challenge document
    // Handle the case where user_id might not exist
    let user_id = passkey_doc
        .get_object_id("user_id")
        .map_err(|e| {
            eprintln!("Failed to get user_id from passkey: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response = AuthResponse {
        success: true,
        username: body.username,
        token: token.clone(),
        user_id: user_id.to_hex(),
    };
    dbg!(&user_id);
    let cookie_value = format!(
        "session_token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400",
        token
    );

    let mut resp = Json(response).into_response();
    resp.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&cookie_value).unwrap(),
    );

    Ok(resp)
}