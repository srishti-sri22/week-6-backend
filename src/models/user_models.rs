use serde::{Deserialize, Serialize};
use mongodb::bson::{oid::ObjectId};
use chrono::{DateTime, Utc};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub username: String,
    pub created_at: DateTime<Utc>,
}
