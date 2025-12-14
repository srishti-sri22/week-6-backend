use std::sync::Arc;

use axum::{
    Json,
    extract::{Extension},
};
use chrono::Utc;
use mongodb::{
    Database,
    bson::{ oid::ObjectId},
};


use crate::models::{
    poll_models::{Poll, PollOption}
};
use crate::controllers::poll_controllers::models::{CreatePollRequest, PollResponse};
use crate::utils::error::{AppError, AppResult};

pub async fn create_poll(
    Extension(db): Extension<Arc<Database>>,
    Json(payload): Json<CreatePollRequest>,
) -> AppResult<Json<PollResponse>> {
    
    let coll = db.collection::<Poll>("polls");

    let now = Utc::now();

    let unique_options: Vec<String> = (&payload.options).into_iter()
        .map(|opt| opt.trim().to_string())
        .collect::<Vec<String>>();
    
    if unique_options.len()<2 {
        return Err(AppError::ValidationError("Enter atleast 2 options for the user to select from".to_string()));
    }

    let mut deduped_options = Vec::new();
    for option in &unique_options {
        if !deduped_options.contains(option) {
            deduped_options.push(option.clone());
        }
    }

    if deduped_options.len() < 2 {
        return Err(AppError::ValidationError("Poll must have at least 2 unique options".to_string()));
    }

    if deduped_options.len() != unique_options.len() {
        return Err(AppError::ValidationError("Poll options must be unique".to_string()));
    }
    
    dbg!(&payload);
    
    let creator_id = ObjectId::parse_str(&payload.creator_id)
        .map_err(|e| AppError::BadRequest(format!("Invalid creator_id: {}", e)))?;
    
    let new_poll = Poll {
        id: ObjectId::new(),
        question: payload.question,
        creator_id,
        options: payload
            .options
            .into_iter()
            .map(|text| PollOption {
                id: ObjectId::new().to_hex(),
                text,
                votes: 0,   
                voter: creator_id
            })
            .collect(),
        is_closed: false,
        created_at: now,
        total_votes: 0
    };

    coll.insert_one(&new_poll)
        .await?;
    
        
    let poll_res = PollResponse {
        id: new_poll.id.to_hex(),
        question: new_poll.question,
        creator_id: payload.creator_id.clone(),
        options: new_poll.options,
        is_closed: new_poll.is_closed,
        created_at: new_poll.created_at,
        total_votes: new_poll.total_votes,
    };

    Ok(Json(poll_res))
}