use axum::{
    Json,
    extract::{Extension, Path},
};
use std::sync::Arc;
use mongodb::{
    Database,
    bson::{doc, oid::ObjectId},
};
use futures::TryStreamExt;

use crate::{controllers::poll_controllers::models::PollResponse, models::poll_models::Poll};
use crate::utils::error::{AppError, AppResult};

pub async fn get_polls_by_user(
    Path(user_id): Path<String>,
    Extension(db): Extension<Arc<Database>>
) -> AppResult<Json<Vec<PollResponse>>> {

    let coll = db.collection::<Poll>("polls");

    let object_id = ObjectId::parse_str(&user_id)
        .map_err(|e| AppError::BadRequest(format!("Invalid user ID: {}", e)))?;

    let cursor = coll
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