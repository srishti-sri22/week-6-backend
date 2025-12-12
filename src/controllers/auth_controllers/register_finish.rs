// src/controllers/auth_controllers/register_finish.rs

use axum::{extract::Extension, http::StatusCode, Json};
use mongodb::{bson::doc, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use webauthn_rs::prelude::*;
use mongodb::bson::oid::ObjectId;
use crate::{controllers::auth_controllers::models::RegisterResponse, utils::session};

#[derive(Debug, Deserialize)]
pub struct RegisterFinishRequest {
    pub username: String,
    pub credential: serde_json::Value,
}

pub async fn register_finish(
    Extension(db): Extension<Arc<Database>>,
    Extension(webauthn): Extension<Arc<Webauthn>>,
    Json(body): Json<RegisterFinishRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    println!("=== Register finish for: {} ===", body.username);

    // Get the challenge from database
    let challenge_collection = db.collection::<mongodb::bson::Document>("registration_challenges");
    let challenge_doc = challenge_collection
        .find_one(doc! { "username": &body.username })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    let state_json = challenge_doc
        .get_str("state")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let reg_state: PasskeyRegistration =
        serde_json::from_str(state_json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Parse the credential
    let credential_json: RegisterPublicKeyCredential =
        serde_json::from_value(body.credential).map_err(|e| {
            eprintln!("Credential parsing error: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;

    // Verify the credential
    let passkey = webauthn
        .finish_passkey_registration(&credential_json, &reg_state)
        .map_err(|e| {
            eprintln!("Registration verification failed: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    println!("✓ Registration verified successfully");

    // Create user document
    let user_id = ObjectId::new();
    let users_collection = db.collection::<mongodb::bson::Document>("users");
    
    users_collection
        .insert_one(doc! {
            "_id": user_id,
            "username": &body.username,
            "created_at": chrono::Utc::now().to_rfc3339(),
        })
        .await
        .map_err(|e| {
            eprintln!("Failed to create user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("✓ User created with ID: {}", user_id);

    // Store the passkey
    let credential_id = passkey.cred_id().to_vec();
    let credential_id_base64 =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &credential_id);

    let passkeys_collection = db.collection::<mongodb::bson::Document>("passkeys");
    passkeys_collection
        .insert_one(doc! {
            "credential_id": &credential_id_base64,
            "user_id": user_id,
            "username": &body.username,
            "passkey": serde_json::to_string(&passkey).unwrap(),
            "created_at": chrono::Utc::now().to_rfc3339(),
        })
        .await
        .map_err(|e| {
            eprintln!("Failed to store passkey: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("✓ Passkey stored");

    // Clean up the challenge
    challenge_collection
        .delete_one(doc! { "username": &body.username })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate JWT token
    let token = session::create_token(&body.username)
        .map_err(|e| {
            eprintln!("Failed to create token: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("=== Registration complete for: {} ===", body.username);
    println!("User ID: {}", user_id.to_hex());

    // Return response with user_id as hex string
    let response = RegisterResponse {
        success: true,
        username: body.username,
        token,
        user_id: user_id // Convert ObjectId to hex string
    };

    Ok(Json(response))
}