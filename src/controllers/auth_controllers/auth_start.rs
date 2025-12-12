use axum::{Json, extract::Extension, http::StatusCode};
use futures::stream::TryStreamExt;
use mongodb::{
    Database,
    bson::{DateTime as BsonDateTime, Document, doc},
};
use serde::Deserialize;
use std::sync::Arc;
use webauthn_rs::prelude::*;

use crate::models::passkey_models::PasskeyData;

#[derive(Deserialize)]
pub struct AuthStartRequest {
    pub username: String,
}


pub async fn auth_start(
    Extension(db): Extension<Arc<Database>>,
    Extension(webauthn): Extension<Arc<Webauthn>>,
    Json(body): Json<AuthStartRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let users = db.collection::<Document>("users");

    let user_doc = users
        .find_one(doc! { "username": &body.username })
        .await
        .map_err(|e| {
            eprintln!("Database query error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            eprintln!("User not found: {}", body.username);
            StatusCode::NOT_FOUND
        })?;

    let user_id = user_doc
        .get_object_id("_id")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    println!("\nUser ID: {}", user_id);

    let passkeys_collection = db.collection::<Document>("passkeys");

    let passkey_docs: Vec<Document> = passkeys_collection
        .find(doc! { "user_id": user_id })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .try_collect()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if passkey_docs.is_empty() {
        eprintln!("No passkeys found for user: {}", body.username);
        return Err(StatusCode::NOT_FOUND);
    }


    println!("Passkeys = {:?}", passkey_docs);
    
    let mut passkeys: Vec<Passkey> = Vec::new();
    
    for doc in &passkey_docs {
        let passkey_json = doc
            .get_str("passkey")
            .map_err(|e| {
                eprintln!("Failed to get 'passkey' field: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            println!("passkey_json: {}", passkey_json);
        
        let passkey: Passkey = serde_json::from_str(passkey_json)
            .map_err(|e| {
                eprintln!("Failed to parse passkey JSON: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        passkeys.push(passkey);
    }

    println!("Passkeys = {:?}", passkeys);

    let (rcr, auth_state) = webauthn
        .start_passkey_authentication(&passkeys)
        .map_err(|e| {
            eprintln!("Auth start error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let state_json =
        serde_json::to_string(&auth_state).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let challenge_collection = db.collection::<Document>("auth_challenges");
    challenge_collection
        .insert_one(doc! {
            "username": &body.username,
            "state": state_json,
            "created_at": BsonDateTime::now(),
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    println!("✓ Auth challenge created");

    Ok(Json(serde_json::to_value(rcr).unwrap()))
}