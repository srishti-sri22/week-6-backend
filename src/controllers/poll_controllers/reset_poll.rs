use std::sync::Arc;

use axum::{
    Json,
    extract::{Extension, Path},
    http::StatusCode,
};

use mongodb::{
    Database,
    bson::{doc, oid::ObjectId},
};


use crate::models::{
    poll_models::{Poll},
    vote_record_models::VoteRecord,
};
use crate::controllers::poll_controllers::models::{CreatorOnly};

pub async fn reset_poll(
    Path(poll_id): Path<String>,
    Extension(db): Extension<Arc<Database>>,
    Json(payload): Json<CreatorOnly>,
) -> Result<Json<Poll>, (StatusCode, String)> {
    let coll = db.collection::<Poll>("polls");

    let obj_id = ObjectId::parse_str(&poll_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid poll id".to_string()))?;

    let poll = coll
        .find_one(doc! {"_id":obj_id})
        .await
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "The given Poll does not exist".to_string(),
            )
        })?
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "The Poll id does not exist".to_string(),
        ))?;

    let current_user = ObjectId::parse_str(&payload.user_id).unwrap();

    if poll.creator_id != current_user {
        return Err((
            StatusCode::FORBIDDEN,
            "Only the Creator of the Poll is allowed to RESET that Poll".to_string(),
        ));
    }

    coll.update_one(
        doc! {"_id":obj_id},
        doc! {
        "$set":{
            "options.$[].votes":0,
            "is_closed":false
        }
        },
    )
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "The Resetting ood the poll details failed".to_string(),
        )
    })?;

    let vote_coll = db.collection::<VoteRecord>("vote_records");

    vote_coll
        .delete_many(doc! { "poll_id": obj_id })
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to clear vote records".to_string(),
            )
        })?;

    let updated_poll = coll
        .find_one(doc! { "_id": obj_id })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Poll not found".to_string()))?;
    
    return Ok(Json(updated_poll));
}