use axum::{
    Json,
    extract::{Extension, Path, State},
};

use mongodb::{
    bson::{doc, oid::ObjectId},
};

use crate::{controllers::poll_controllers::models::PollResponse, models::{
    poll_models::Poll,
    vote_record_models::VoteRecord,
}};
use crate::controllers::poll_controllers::models::CastVoteRequest;
use crate::utils::error::{AppError, AppResult};
use crate::utils::session::Claims;
use crate::state::AppState;

pub async fn change_vote(
    Path(poll_id): Path<String>,
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CastVoteRequest>,
) -> AppResult<Json<PollResponse>> {

    let polls_collection = state.db.collection::<Poll>("polls");
    let vote_collection = state.db.collection::<VoteRecord>("vote_records");

    let obj_id = ObjectId::parse_str(&poll_id)
        .map_err(|_| AppError::BadRequest("Invalid poll id".to_string()))?;

    let user_obj_id = ObjectId::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user id".to_string()))?;

    let previous_vote = vote_collection
        .find_one(doc! {
            "poll_id": obj_id,
            "user_id": user_obj_id
        })
        .await?
        .ok_or_else(|| AppError::BadRequest("User has not voted yet".to_string()))?;

    if previous_vote.option_id == payload.option_id {
        return Err(AppError::Conflict("You already voted for this option".to_string()));
    }

    polls_collection.update_one(
        doc! {
            "_id": obj_id,
            "options.id": &previous_vote.option_id   
        },
        doc! {
            "$inc": { "options.$.votes": -1 }
        }
    )
    .await?;

    polls_collection.update_one(
        doc! {
            "_id": obj_id,
            "options.id": &payload.option_id        
        },
        doc! {
            "$inc": { "options.$.votes": 1 }
        }
    )
    .await?;

    vote_collection.update_one(
        doc! { "poll_id": obj_id, "user_id": user_obj_id },
        doc! {
            "$set": {
                "option_id": payload.option_id.clone()
            }
        }
    )
    .await?;

    let new_poll = polls_collection
        .find_one(doc! { "_id": obj_id })
        .await?
        .ok_or_else(|| AppError::NotFound("Poll not found".to_string()))?;

    let poll_response = PollResponse {
        id: new_poll.id.to_hex(),
        question: new_poll.question,
        creator_id: new_poll.creator_id.to_hex(),
        options: new_poll.options,
        is_closed: new_poll.is_closed,
        created_at: new_poll.created_at,
        total_votes: new_poll.total_votes,
    };

    Ok(Json(poll_response))
}