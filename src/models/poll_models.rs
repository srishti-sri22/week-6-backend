use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use uuid::Uuid;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Poll {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub question: String,
    pub creator_id: ObjectId,
    pub options: Vec<PollOption>,
    pub is_closed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub total_votes: i32
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PollOption {
    pub id: String,
    pub text: String,
    pub votes: u32,
    pub voter: ObjectId
}
