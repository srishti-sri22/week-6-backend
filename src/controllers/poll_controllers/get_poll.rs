use axum::{
    Json,
    extract::{Path, State},
};
use mongodb::{
    bson::{doc, oid::ObjectId},
};

use crate::{controllers::poll_controllers::models::PollResponse, models::{poll_models::Poll}};
use crate::utils::error::{AppError, AppResult};
use crate::state::AppState;

pub async fn get_poll(
    Path(poll_id): Path<String>,
    State(state): State<AppState>,
) -> AppResult<Json<PollResponse>> {

    let poll_collection = state.db.collection::<Poll>("polls");

    let poll_obj_id = ObjectId::parse_str(&poll_id)
        .map_err(|_| AppError::BadRequest("Invalid poll id".to_string()))?;
    
    let poll = poll_collection
        .find_one(doc! { "_id": poll_obj_id })
        .await?
        .ok_or_else(|| AppError::NotFound("Poll not found".to_string()))?;

    let poll_res = PollResponse {
        id: poll.id.to_hex(),
        question: poll.question,
        creator_id: poll.creator_id.to_hex(),           
        options: poll.options,
        is_closed: poll.is_closed,
        created_at: poll.created_at,
        total_votes: poll.total_votes,
    };
    Ok(Json(poll_res))
}