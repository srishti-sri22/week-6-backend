use axum::{Json, extract::State};
use mongodb::{
    bson::{DateTime as BsonDateTime, Document, doc},
};
use webauthn_rs::prelude::*;

use crate::{
    controllers::auth_controllers::models::RegisterStartRequest,
    utils::error::{AppError, AppResult},
    state::AppState,
};

pub async fn register_start(
    State(state): State<AppState>,
    Json(body): Json<RegisterStartRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if body.username.is_empty() {
        return Err(AppError::ValidationError("Username is required".to_string()));
    }

    if body.username.len() < 3 {
        return Err(AppError::ValidationError("Username must be at least 3 characters long".to_string()));
    }

    if body.display_name.is_empty() {
        return Err(AppError::ValidationError("Display name is required".to_string()));
    }

    if body.display_name.len() < 2 {
        return Err(AppError::ValidationError("Display name must be at least 2 characters long".to_string()));
    }

    let users = state.db.collection::<crate::models::user_models::User>("users");

    let existing = users.find_one(doc! { "username": &body.username }).await?;
    
    if existing.is_some() {
        eprintln!("Username already exists: {}", &body.username);
        return Err(AppError::Conflict("Username already exists".to_string()));
    }

    let user_unique_id = Uuid::new_v4();

    let (ccr, reg_state) = state.webauthn
        .start_passkey_registration(user_unique_id, &body.username, &body.display_name, None)
        .map_err(|e| AppError::WebauthnError(format!("Failed to start passkey registration: {}", e)))?;

    let state_json = serde_json::to_string(&reg_state)?;

    let register_challenge_collection = state.db.collection::<Document>("registration_challenges");
    register_challenge_collection
        .insert_one(doc! {
            "username": &body.username,
            "display_name": &body.display_name,
            "user_unique_id": user_unique_id.to_string(),
            "state": state_json,
            "created_at": BsonDateTime::now(),
        })
        .await?;

    let ccr_value = serde_json::to_value(ccr)?;
    
    Ok(Json(ccr_value))
}