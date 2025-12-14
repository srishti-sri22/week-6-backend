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

use crate::{controllers::poll_controllers::models::PollResponse, models::{poll_models::Poll}};

pub async fn get_poll(
    Path(poll_id): Path<String>,
    Extension(db): Extension<Arc<Database>>,
) -> Result<Json<PollResponse>, (StatusCode, String)> {

    let poll_coll = db.collection::<Poll>("polls");

    let obj_id = ObjectId::parse_str(&poll_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid poll id".to_string()))?;

        println!("Poll id in get poll is {:?},", obj_id);
    let poll = poll_coll
        .find_one(doc! { "_id": obj_id })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Poll not found".to_string()))?;

        println!("Poll in get poll is {:?},", poll);

    let poll_res = PollResponse {
        id: poll.id.to_hex(),
        question: poll.question,
        creator_id: poll.creator_id.to_hex(),           
        options: poll.options,
        is_closed: poll.is_closed,
        created_at: poll.created_at,
        total_votes: poll.total_votes,
    };

    println!("Poll response in get poll is {:?},", poll_res);
    Ok(Json(poll_res))
}