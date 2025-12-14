use axum::{extract::Extension, http::StatusCode, Json};
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime as BsonDateTime},
    Database,
};
use std::sync::Arc;
use webauthn_rs::prelude::*;
use base64::{engine::general_purpose::STANDARD, Engine};

use crate::{
    controllers::auth_controllers::models::{RegisterFinishRequest, RegisterResponse},
    utils::session,
};

pub async fn register_finish(
    Extension(db): Extension<Arc<Database>>,
    Extension(webauthn): Extension<Arc<Webauthn>>,
    Json(body): Json<RegisterFinishRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    let challenge_collection =
        db.collection::<mongodb::bson::Document>("registration_challenges");

    let challenge_doc = challenge_collection
        .find_one(doc! { "username": &body.username })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    let state_json = challenge_doc
        .get_str("state")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let display_name = challenge_doc
        .get_str("display_name")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    let reg_state: PasskeyRegistration =
        serde_json::from_str(state_json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let credential: RegisterPublicKeyCredential =
        serde_json::from_value(body.credential).map_err(|_| StatusCode::BAD_REQUEST)?;

    let passkey = webauthn
        .finish_passkey_registration(&credential, &reg_state)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let users = db.collection::<mongodb::bson::Document>("users");

    let user_id = match users
        .find_one(doc! { "username": &body.username })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        Some(user) => user
            .get_object_id("_id")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .clone(),
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
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            new_id
        }
    };

    let credential_id_b64 = STANDARD.encode(passkey.cred_id());

    let passkey_bson =
        mongodb::bson::to_document(&passkey).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let passkeys = db.collection::<mongodb::bson::Document>("passkeys");

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
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    challenge_collection
        .delete_one(doc! { "username": &body.username })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let token =
        session::create_token(&body.username).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(RegisterResponse {
        success: true,
        username: body.username,
        display_name,
        token,
        user_id,
    }))
}