use axum::{Json, extract::Extension, http::StatusCode};
use futures::stream::TryStreamExt;
use mongodb::{
    Database,
    bson::{DateTime as BsonDateTime, Document, doc},
};
use std::sync::Arc;
use webauthn_rs::prelude::*;
use crate::controllers::auth_controllers::models::AuthStartRequest;

pub async fn auth_start(
    Extension(db): Extension<Arc<Database>>,
    Extension(webauthn): Extension<Arc<Webauthn>>,
    Json(body): Json<AuthStartRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let users = db.collection::<Document>("users");

    println!("🔍 Looking for user: {}", &body.username);

    let user_doc = users
        .find_one(doc! { "username": &body.username })
        .await
        .map_err(|e| {
            eprintln!("❌ Database error finding user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            eprintln!("❌ User not found: {}", &body.username);
            StatusCode::NOT_FOUND
        })?;

    println!("✅ User found: {:?}", user_doc);

    let user_id = user_doc
        .get_object_id("_id")
        .map_err(|e| {
            eprintln!("❌ Error getting user_id: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("✅ User ID: {}", user_id);

    let passkeys_collection = db.collection::<Document>("passkeys");

    println!("🔍 Looking for passkeys for user_id: {}", user_id);

    let passkey_docs: Vec<Document> = passkeys_collection
        .find(doc! { "user_id": user_id })
        .await
        .map_err(|e| {
            eprintln!("❌ Database error finding passkeys: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .try_collect()
        .await
        .map_err(|e| {
            eprintln!("❌ Error collecting passkey docs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("✅ Found {} passkey documents", passkey_docs.len());

    if passkey_docs.is_empty() {
        eprintln!("❌ No passkeys found for user: {}", &body.username);
        return Err(StatusCode::NOT_FOUND);
    }

    let mut passkeys: Vec<Passkey> = Vec::new();

    for (index, doc) in passkey_docs.iter().enumerate() {
        println!("🔍 Processing passkey #{}", index);
        
        let passkey_doc = doc
            .get_document("passkey")
            .map_err(|e| {
                eprintln!("❌ Error getting passkey document: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let passkey: Passkey =
            mongodb::bson::from_document(passkey_doc.clone())
                .map_err(|e| {
                    eprintln!("❌ Error deserializing passkey: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

        passkeys.push(passkey);
    }

    println!("✅ Successfully loaded {} passkeys", passkeys.len());

    let (rcr, auth_state) = webauthn
        .start_passkey_authentication(&passkeys)
        .map_err(|e| {
            eprintln!("❌ Webauthn error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let state_json =
        serde_json::to_string(&auth_state).map_err(|e| {
            eprintln!("❌ Error serializing auth state: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let challenge_collection = db.collection::<Document>("auth_challenges");

    challenge_collection
        .delete_many(doc! { "username": &body.username })
        .await
        .map_err(|e| {
            eprintln!("❌ Error deleting old challenges: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    challenge_collection
        .insert_one(
            doc! {
                "username": &body.username,
                "state": state_json,
                "created_at": BsonDateTime::now(),
            },
        )
        .await
        .map_err(|e| {
            eprintln!("❌ Error inserting challenge: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("✅ Auth challenge created successfully");

    Ok(Json(
        serde_json::to_value(rcr).map_err(|e| {
            eprintln!("❌ Error converting rcr to JSON: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?,
    ))
}