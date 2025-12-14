use axum::{Json, extract::Extension};
use futures::stream::TryStreamExt;
use mongodb::{
    Database,
    bson::{DateTime as BsonDateTime, Document, doc},
};
use std::sync::Arc;
use webauthn_rs::prelude::*;
use crate::{
    controllers::auth_controllers::models::AuthStartRequest,
    utils::error::{AppError, AppResult}
};

pub async fn auth_start(
    Extension(db): Extension<Arc<Database>>,
    Extension(webauthn): Extension<Arc<Webauthn>>,
    Json(body): Json<AuthStartRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if body.username.is_empty() {
        return Err(AppError::ValidationError("Username is required".to_string()));
    }

    let users = db.collection::<Document>("users");

    println!("🔍 Looking for user: {}", &body.username);

    let user_doc = users
        .find_one(doc! { "username": &body.username })
        .await?
        .ok_or_else(|| {
            eprintln!("❌ User not found: {}", &body.username);
            AppError::NotFound(format!("User '{}' not found", &body.username))
        })?;

    println!("✅ User found: {:?}", user_doc);

    let user_id = user_doc
        .get_object_id("_id")
        .map_err(|e| AppError::InternalError(format!("Failed to get user_id: {}", e)))?;

    println!("✅ User ID: {}", user_id);

    let passkeys_collection = db.collection::<Document>("passkeys");

    println!("🔍 Looking for passkeys for user_id: {}", user_id);

    let passkey_docs: Vec<Document> = passkeys_collection
        .find(doc! { "user_id": user_id })
        .await?
        .try_collect()
        .await?;

    println!("✅ Found {} passkey documents", passkey_docs.len());

    if passkey_docs.is_empty() {
        eprintln!("❌ No passkeys found for user: {}", &body.username);
        return Err(AppError::NotFound(format!("No passkeys found for user '{}'", &body.username)));
    }

    let mut passkeys: Vec<Passkey> = Vec::new();

    for (index, doc) in passkey_docs.iter().enumerate() {
        println!("🔍 Processing passkey #{}", index);
        
        let passkey_doc = doc
            .get_document("passkey")
            .map_err(|e| AppError::InternalError(format!("Failed to get passkey document: {}", e)))?;

        let passkey: Passkey = mongodb::bson::from_document(passkey_doc.clone())?;

        passkeys.push(passkey);
    }

    println!("✅ Successfully loaded {} passkeys", passkeys.len());

    let (rcr, auth_state) = webauthn
        .start_passkey_authentication(&passkeys)
        .map_err(|e| AppError::WebauthnError(format!("Failed to start authentication: {}", e)))?;

    let state_json = serde_json::to_string(&auth_state)?;

    let challenge_collection = db.collection::<Document>("auth_challenges");

    challenge_collection
        .delete_many(doc! { "username": &body.username })
        .await?;

    challenge_collection
        .insert_one(
            doc! {
                "username": &body.username,
                "state": state_json,
                "created_at": BsonDateTime::now(),
            },
        )
        .await?;

    println!("✅ Auth challenge created successfully");

    let rcr_value = serde_json::to_value(rcr)?;
    
    Ok(Json(rcr_value))
}