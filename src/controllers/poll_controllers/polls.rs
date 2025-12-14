use axum::{
    Json,
    extract::Extension,
};
use std::sync::Arc;
use mongodb::{
    Database,
    bson::doc,
};
use futures_util::TryStreamExt;

use crate::{controllers::poll_controllers::models::PollResponse, models::poll_models::Poll};
use crate::utils::error::{AppResult};

pub async fn get_all_polls(
    Extension(db): Extension<Arc<Database>>,
) -> AppResult<Json<Vec<PollResponse>>> {
    let coll = db.collection::<Poll>("polls");

    let mut cursor = coll
        .find(doc! {}) 
        .await?;

    let mut new_polls = Vec::new();

    while let Some(poll) = cursor
        .try_next()
        .await?
    {
        new_polls.push(poll);
    }

    let poll_responses: Vec<PollResponse> = new_polls
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