use axum::{
    Json,
    extract::{Extension, Path, State},
};
use chrono::Utc;
use mongodb::{
    bson::{doc, oid::ObjectId},
};

use crate::controllers::poll_controllers::models::{CastVoteRequest, PollResponse};
use crate::models::{poll_models::Poll, vote_record_models::VoteRecord};
use crate::utils::error::{AppError, AppResult};
use crate::utils::session::Claims;
use crate::state::AppState;

pub async fn cast_vote(
    Path(poll_id): Path<String>,
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CastVoteRequest>,
) -> AppResult<Json<PollResponse>> {
    let poll_collection = state.db.collection::<Poll>("polls");
    let vote_collection = state.db.collection::<VoteRecord>("vote_records");

    let poll_obj_id = ObjectId::parse_str(&poll_id)
        .map_err(|_| AppError::BadRequest("Invalid Poll id".to_string()))?;

    let user_obj_id = ObjectId::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let poll = poll_collection
        .find_one(doc! { "_id": poll_obj_id })
        .await?
        .ok_or_else(|| AppError::NotFound("Poll not found".to_string()))?;

    let is_valid_option = poll
        .options
        .iter()
        .any(|option| option.id == payload.option_id);

    if !is_valid_option {
        return Err(AppError::BadRequest(
            "Invalid option ID for this poll".to_string(),
        ));
    }

    if poll.is_closed {
        return Err(AppError::BadRequest(
            "Poll is Closed. Voting is not allowed".to_string(),
        ));
    }

    let already_voted = vote_collection
        .find_one(doc! { "poll_id": poll_obj_id, "user_id": user_obj_id.clone() })
        .await?;

    if already_voted.is_some() {
        return Err(AppError::Conflict(
            "You have already voted for this poll and can't vote again, Bye Byee.".to_string(),
        ));
    }

    let filter = doc! { "_id": poll_obj_id, "options.id": &payload.option_id  };
    let update = doc! {
        "$inc": {
            "options.$.votes": 1,
            "total_votes": 1
        }
    };

    let update_result = poll_collection.update_one(filter, update).await?;

    if update_result.matched_count == 0 {
        return Err(AppError::BadRequest(
            "Option not found for this poll".to_string(),
        ));
    }
    if update_result.modified_count == 0 {
        return Err(AppError::InternalError(
            "Failed to increment vote for option".to_string(),
        ));
    }

    let vote = VoteRecord {
        id: ObjectId::new(),
        poll_id: poll_obj_id,
        user_id: Some(user_obj_id),
        option_id: payload.option_id.clone(),
        created_at: Utc::now(),
    };

    vote_collection.insert_one(vote).await?;

    let new_poll = poll_collection
        .find_one(doc! { "_id": poll_obj_id })
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