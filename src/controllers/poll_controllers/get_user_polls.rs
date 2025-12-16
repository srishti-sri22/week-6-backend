use axum::{
    Json,
    extract::{Extension, State},
};
use mongodb::{
    bson::{doc, oid::ObjectId},
};
use futures::TryStreamExt;

use crate::{controllers::poll_controllers::models::PollResponse, models::poll_models::Poll};
use crate::utils::error::{AppError, AppResult};
use crate::utils::session::Claims;
use crate::state::AppState;

pub async fn get_polls_by_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<Vec<PollResponse>>> {

    let polls_collection = state.db.collection::<Poll>("polls");

    let object_id = ObjectId::parse_str(&claims.sub)
        .map_err(|e| AppError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let cursor = polls_collection
        .find(doc! { "creator_id": object_id })
        .await?;

    let polls: Vec<Poll> = cursor
        .try_collect()
        .await?;

    let poll_responses: Vec<PollResponse> = polls
        .into_iter()
        .map(|poll| PollResponse {
            id: poll.id.to_hex(),
            question: poll.question,
            creator_id: poll.creator_id.to_hex(),
            options: poll.options,
            is_closed: poll.is_closed,
            created_at: poll.created_at,
            total_votes: poll.total_votes,
        })
        .collect();

    Ok(Json(poll_responses))
}