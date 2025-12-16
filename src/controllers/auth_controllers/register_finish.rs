use axum::{extract::State, Json};
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime as BsonDateTime},
};
use webauthn_rs::prelude::*;
use base64::{engine::general_purpose::STANDARD, Engine};

use crate::{
    controllers::auth_controllers::models::{RegisterFinishRequest, RegisterResponse},
    utils::{session, error::{AppError, AppResult}},
    state::AppState,
};

pub async fn register_finish(
    State(state): State<AppState>,
    Json(body): Json<RegisterFinishRequest>,
) -> AppResult<Json<RegisterResponse>> {
    if body.username.is_empty() {
        return Err(AppError::ValidationError("Username is required".to_string()));
    }

    let register_challenge_collection = state.db.collection::<mongodb::bson::Document>("registration_challenges");

    let challenge_doc = register_challenge_collection
        .find_one(doc! { "username": &body.username })
        .await?
        .ok_or_else(|| AppError::NotFound("Registration challenge not found".to_string()))?;

    let state_json = challenge_doc
        .get_str("state")
        .map_err(|e| AppError::InternalError(format!("Failed to get state: {}", e)))?;

    let display_name = challenge_doc
        .get_str("display_name")
        .map_err(|e| AppError::InternalError(format!("Failed to get display_name: {}", e)))?
        .to_string();

    let reg_state: PasskeyRegistration = serde_json::from_str(state_json)?;

    let credential: RegisterPublicKeyCredential = serde_json::from_value(body.credential)
        .map_err(|e| AppError::BadRequest(format!("Invalid credential format: {}", e)))?;

    let passkey = state.webauthn
        .finish_passkey_registration(&credential, &reg_state)
        .map_err(|e| AppError::WebauthnError(format!("Passkey registration failed: {}", e)))?;

    let users = state.db.collection::<mongodb::bson::Document>("users");

    let user_id = match users
        .find_one(doc! { "username": &body.username })
        .await?
    {
        Some(user) => {
            user.get_object_id("_id")
                .map_err(|e| AppError::InternalError(format!("Failed to get user_id: {}", e)))?
                .clone()
        }
        None => {
            let new_id = ObjectId::new();
            users
                .insert_one(
                    doc! {
                        "_id": new_id,
                        "username": &body.username,
                        "display_name": &display_name,
                        "created_at": BsonDateTime::now(),
                    },
                )
                .await?;
            new_id
        }
    };

    let credential_id_b64 = STANDARD.encode(passkey.cred_id());

    let passkey_bson = mongodb::bson::to_document(&passkey)?;

    let passkeys = state.db.collection::<mongodb::bson::Document>("passkeys");

    passkeys
        .insert_one(
            doc! {
                "credential_id": credential_id_b64,
                "user_id": user_id,
                "username": &body.username,
                "passkey": passkey_bson,
                "created_at": BsonDateTime::now(),
                "last_used_at": BsonDateTime::now(),
            },
        )
        .await?;

    register_challenge_collection
        .delete_one(doc! { "username": &body.username })
        .await?;

    let token = session::create_token(&body.username)
        .map_err(|e| AppError::InternalError(format!("Failed to create session token: {}", e)))?;

    Ok(Json(RegisterResponse {
        success: true,
        username: body.username,
        display_name,
        token,
        user_id,
    }))
}