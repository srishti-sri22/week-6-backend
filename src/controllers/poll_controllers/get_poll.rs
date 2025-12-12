use axum::{
    Json,
    extract::{Extension, Path},
    http::StatusCode,
};
use std::sync::Arc;
use mongodb::{
    Database,
    bson::{doc, oid::ObjectId},
};


use crate::{controllers::poll_controllers::models::PollResponse, models::poll_models::Poll};

pub async fn get_poll(
    Path(poll_id): Path<String>,
    Extension(db): Extension<Arc<Database>>,
) -> Result<Json<PollResponse>, (StatusCode, String)> {
    
    let coll = db.collection::<Poll>("polls");

    let obj_id = ObjectId::parse_str(&poll_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid poll id".to_string()))?;

    let new_poll = coll
        .find_one(doc! { "_id": obj_id })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Poll not found".to_string()))?;

    let poll_res = PollResponse {
        id: new_poll.id.to_hex(),
        question: new_poll.question,
        creator_id: new_poll.creator_id.to_hex(),
        options: new_poll.options,
        is_closed: new_poll.is_closed,
        created_at: new_poll.created_at,
        updated_at: new_poll.updated_at,
        total_votes: new_poll.total_votes,
    };

    Ok(Json(poll_res))
}