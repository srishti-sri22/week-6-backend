use serde::{Deserialize, Serialize};
use mongodb::bson::{oid::ObjectId};
use chrono::{DateTime,Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VoteRecord {
    #[serde(rename = "_id")]
    pub id: ObjectId,

    pub poll_id: ObjectId,

    pub user_id: Option<ObjectId>,  

    pub option_id: String,

    pub created_at: DateTime<Utc>,
}
