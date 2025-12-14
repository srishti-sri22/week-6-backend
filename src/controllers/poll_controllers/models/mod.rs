
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::models::poll_models::{PollOption};

#[derive(Deserialize,Debug)]
pub struct CreatePollRequest {
    pub question: String,
    pub options: Vec<String>,
    pub creator_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PollResponse {
    pub id: String,
    pub question: String,
    pub creator_id: String,
    pub options: Vec<PollOption>,
    pub is_closed: bool,
    pub created_at: DateTime<Utc>,
    pub total_votes: i32
}


#[derive(Deserialize)]
pub struct CastVoteRequest {
    pub option_id: String,
    pub user_id: String,
}


#[derive(Deserialize)]
pub struct CreatorOnly {
    pub user_id: String,
}




