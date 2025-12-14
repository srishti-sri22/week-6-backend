use std::{str::FromStr, sync::Arc};

use axum::{
    Json,
    extract::{Extension},
    http::StatusCode,
};
use chrono::Utc;
use mongodb::{
    Database,
    bson::{ DateTime, oid::ObjectId},
};


use crate::models::{
    poll_models::{Poll, PollOption}
};
use crate::controllers::poll_controllers::models::{CreatePollRequest, PollResponse};

pub async fn create_poll(
    Extension(db): Extension<Arc<Database>>,
    Json(payload): Json<CreatePollRequest>,
) -> Result<Json<PollResponse>, (StatusCode, String)> {
    
    let coll = db.collection::<Poll>("polls");

    let now = Utc::now();

    let unique_options: Vec<String> = (&payload.options).into_iter()
        .map(|opt| opt.trim().to_string())
        .collect::<Vec<String>>();
    
    //dekho minimum 2 options to bheje hmara user
    if unique_options.len()<2 {
        return Err((StatusCode::BAD_REQUEST, "Enter atleast 2 options for the user to select from".to_string()));
    }

    let mut deduped_options = Vec::new();
    for option in &unique_options {
        if !deduped_options.contains(option) {
            deduped_options.push(option.clone());
        }
    }

    if deduped_options.len() < 2 {
        return Err((StatusCode::BAD_REQUEST, "Poll must have at least 2 unique options".to_string()));
    }

    if deduped_options.len() != unique_options.len() {
        return Err((StatusCode::BAD_REQUEST, "Poll options must be unique".to_string()));
    }
    dbg!(&payload);
    let new_poll = Poll {
        id: ObjectId::new(),
        question: payload.question,
        creator_id: ObjectId::parse_str(&payload.creator_id).unwrap(),
        options: payload
            .options
            .into_iter()
            .map(|text| PollOption {
                id: ObjectId::new().to_hex(),
                text,
                votes: 0,   
                voter: ObjectId::from_str(&payload.creator_id
                ).unwrap()
            })
            .collect(),
        is_closed: false,
        created_at: now,
        total_votes: 0
    };

    coll.insert_one(&new_poll)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
        
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