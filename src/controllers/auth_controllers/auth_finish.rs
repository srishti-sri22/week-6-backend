use axum::{Json, extract::State, http::{HeaderValue, header::SET_COOKIE}};
use axum::response::IntoResponse;
use mongodb::bson::{Document, doc};
use webauthn_rs::prelude::*;
use crate::{
    controllers::auth_controllers::models::{AuthFinishRequest, AuthResponse}, 
    utils::{session, error::{AppError, AppResult}},
    state::AppState,
};

pub async fn auth_finish(
    State(state): State<AppState>,
    Json(body): Json<AuthFinishRequest>,
) -> AppResult<impl IntoResponse> {
    if body.username.is_empty() {
        return Err(AppError::ValidationError("Username is required".to_string()));
    }

    let auth_challenge_collection = state.db.collection::<Document>("auth_challenges");
    let challenge_doc = auth_challenge_collection
        .find_one(doc! { "username": &body.username })
        .await?
        .ok_or_else(|| AppError::NotFound("Authentication challenge not found".to_string()))?;

    let state_json = challenge_doc
        .get_str("state")
        .map_err(|e| AppError::InternalError(format!("Failed to get state: {}", e)))?;

    let auth_state: PasskeyAuthentication = serde_json::from_str(state_json)?;

    let credential_json: PublicKeyCredential = serde_json::from_value(body.credential)
        .map_err(|e| AppError::BadRequest(format!("Invalid credential format: {}", e)))?;

    let auth_result = state.webauthn
        .finish_passkey_authentication(&credential_json, &auth_state)
        .map_err(|e| AppError::AuthenticationError(format!("Authentication verification failed: {}", e)))?;

    let credential_id = auth_result.cred_id().to_vec();
    let credential_id_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &credential_id);

    let passkeys_collection = state.db.collection::<Document>("passkeys");
    
    let passkey_doc = passkeys_collection
        .find_one(doc! { "credential_id": &credential_id_base64 })
        .await?
        .ok_or_else(|| AppError::NotFound("Passkey not found".to_string()))?;

    let user_id = passkey_doc
        .get_object_id("user_id")
        .map_err(|e| AppError::InternalError(format!("Failed to get user_id: {}", e)))?;

    let username = passkey_doc
        .get_str("username")
        .map_err(|e| AppError::InternalError(format!("Failed to get username: {}", e)))?
        .to_string();

    let users_collection = state.db.collection::<Document>("users");
    let user_doc = users_collection
        .find_one(doc! { "_id": user_id })
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let display_name = user_doc
        .get_str("display_name")
        .map_err(|e| AppError::InternalError(format!("Failed to get display_name: {}", e)))?
        .to_string();

    let updated_passkey_json = serde_json::to_string(&auth_result)?;

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
        .await?;

    auth_challenge_collection
        .delete_one(doc! { "username": &body.username })
        .await?;

    let token = session::create_token(&username)
        .map_err(|e| AppError::InternalError(format!("Failed to create session token: {}", e)))?;

    let response = AuthResponse {
        success: true,
        username: username.clone(),
        display_name,
        token: token.clone(),
        user_id: user_id.to_hex(),
    };

    let cookie_value = format!(
        "token={}; Path=/; HttpOnly; Secure; SameSite=None; Max-Age=86400",
        token
    );

    let mut resp = Json(response).into_response();
    resp.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&cookie_value)
            .map_err(|e| AppError::InternalError(format!("Failed to create cookie header: {}", e)))?
    );

    Ok(resp)
}