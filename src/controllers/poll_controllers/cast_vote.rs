use std::sync::Arc;

use axum::{
    Json,
    extract::{Extension, Path},
};
use chrono::Utc;
use mongodb::{
    Database,
    bson::{doc, oid::ObjectId},
};

use crate::controllers::poll_controllers::models::{CastVoteRequest, PollResponse};
use crate::utils::error::{AppError, AppResult};
use crate::models::{
    poll_models::Poll,
    vote_record_models::VoteRecord,
};


pub async fn cast_vote(
    Path(poll_id): Path<String>,
    Extension(db): Extension<Arc<Database>>,
    Json(payload): Json<CastVoteRequest>,
) -> AppResult<Json<PollResponse>> {

    let coll = db.collection::<Poll>("polls");
    let vote_coll = db.collection::<VoteRecord>("vote_records");

    let obj_id = ObjectId::parse_str(&poll_id)
        .map_err(|_| AppError::BadRequest("Invalid Poll id".to_string()))?;

    let user_obj_id = ObjectId::parse_str(&payload.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let poll = coll.find_one(doc! { "_id": obj_id })
        .await?
        .ok_or_else(|| AppError::NotFound("Poll not found".to_string()))?;

    if poll.is_closed {
        return Err(AppError::BadRequest("Poll is Closed. Voting is not allowed".to_string()));
    }

    let already_voted = vote_coll
        .find_one(doc! { "poll_id": obj_id, "user_id": user_obj_id.clone() })
        .await?;

    if already_voted.is_some() {
        return Err(AppError::Conflict("You have already voted for this poll and can't vote again, Bye Byee.".to_string()));
    }

    let filter = doc! { "_id": obj_id, "options.id": &payload.option_id  };
    let update = doc! {
        "$inc": {
            "options.$.votes": 1,
            "total_votes": 1
        }
    };

    let update_result = coll
        .update_one(filter, update)
        .await?;

    if update_result.matched_count == 0 {
        return Err(AppError::BadRequest("Option not found for this poll".to_string()));
    }
    if update_result.modified_count == 0 {
        return Err(AppError::InternalError("Failed to increment vote for option".to_string()));
    }

    let vote = VoteRecord {
        id: ObjectId::new(),
        poll_id: obj_id,
        user_id: Some(user_obj_id),
        option_id: payload.option_id.clone(),
        created_at: Utc::now(),
    };

    vote_coll.insert_one(vote)
        .await?;

    let new_poll = coll
        .find_one(doc! { "_id": obj_id })
        .await?
        .ok_or_else(|| AppError::NotFound("Poll not found".to_string()))?;
    
    let poll_res = PollResponse {
        id: new_poll.id.to_hex(),
        question: new_poll.question,
        creator_id: new_poll.creator_id.to_hex(),
        options: new_poll.options,
        is_closed: new_poll.is_closed,
        created_at: new_poll.created_at,
        total_votes: new_poll.total_votes,
    };

    Ok(Json(poll_res))
}