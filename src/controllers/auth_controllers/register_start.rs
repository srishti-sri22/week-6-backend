use axum::{Json, extract::Extension, http::StatusCode};
use mongodb::{
    Database,
    bson::{DateTime as BsonDateTime, Document, doc},
};
use std::sync::Arc;
use webauthn_rs::prelude::*;

use crate::controllers::auth_controllers::models::RegisterStartRequest;

pub async fn register_start(
    Extension(db): Extension<Arc<Database>>,
    Extension(webauthn): Extension<Arc<Webauthn>>,
    Json(body): Json<RegisterStartRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    
    let users = db.collection::<crate::models::user_models::User>("users");

    let existing = users.find_one(doc! { "username": &body.username }).await;
    if existing.is_ok() && existing.unwrap().is_some() {
        return Err(StatusCode::CONFLICT);
    }

    let user_unique_id = Uuid::new_v4();

    let (ccr, reg_state) = webauthn
        .start_passkey_registration(user_unique_id, &body.username, &body.display_name, None)
        .map_err(|e| {
            eprintln!("Passkey registration error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let state_json = serde_json::to_string(&reg_state).map_err(|e| {
        eprintln!("State serialization error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let challenge_collection = db.collection::<Document>("registration_challenges");
    challenge_collection
        .insert_one(doc! {
        "username": &body.username,
        "display_name": &body.display_name,
        "user_unique_id": user_unique_id.to_string(),
        "state": state_json,
        "created_at": BsonDateTime::now(),
        })
        .await
        .map_err(|e| {
            eprintln!("Database insert error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::to_value(ccr).unwrap()))
}