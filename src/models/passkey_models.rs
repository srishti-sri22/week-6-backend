use serde::{Serialize, Deserialize};
use mongodb::bson::{oid::ObjectId, DateTime};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasskeyDocument {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub credential_id: String,
    pub user_id: ObjectId,
    pub passkey: String,
    pub created_at: DateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasskeyData {
    pub cred: String,
    pub needs_update: bool,
    pub user_verified: bool,
    pub backup_state: bool,
    pub backup_eligible: bool,
    pub counter: u32,
    pub extensions: std::collections::HashMap<String, serde_json::Value>,
}