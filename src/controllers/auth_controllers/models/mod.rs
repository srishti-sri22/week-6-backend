use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

fn serialize_object_id_as_string<S>(oid: &ObjectId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&oid.to_hex())
}

#[derive(Deserialize)]
pub struct RegisterStartRequest {
    pub username: String,
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct RegisterFinishRequest {
    pub username: String,
    pub credential: serde_json::Value,
}

#[derive(Deserialize)]
pub struct AuthStartRequest {
    pub username: String,
}

#[derive(Deserialize)]
pub struct AuthFinishRequest {
    pub username: String,
    pub credential: serde_json::Value,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub username: String,
    pub display_name: String,
    pub token: String,
    pub user_id: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub username: String,
    pub display_name: String,
    pub token: String,
    #[serde(serialize_with = "serialize_object_id_as_string")]
    pub user_id: ObjectId, 
}