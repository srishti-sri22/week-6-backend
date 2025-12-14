use std::sync::Arc;

use axum::{
    Json,
    extract::{Extension, Path},
};

use mongodb::{
    Database,
    bson::{doc, oid::ObjectId},
};


use crate::models::{
    poll_models::Poll,
    vote_record_models::VoteRecord,
};
use crate::controllers::poll_controllers::models::CreatorOnly;
use crate::utils::error::{AppError, AppResult};

pub async fn reset_poll(
    Path(poll_id): Path<String>,
    Extension(db): Extension<Arc<Database>>,
    Json(payload): Json<CreatorOnly>,
) -> AppResult<Json<Poll>> {
    let coll = db.collection::<Poll>("polls");

    let obj_id = ObjectId::parse_str(&poll_id)
        .map_err(|_| AppError::BadRequest("Invalid poll id".to_string()))?;

    let poll = coll
        .find_one(doc! {"_id":obj_id})
        .await?
        .ok_or_else(|| AppError::NotFound("The Poll id does not exist".to_string()))?;

    let current_user = ObjectId::parse_str(&payload.user_id)
        .map_err(|e| AppError::BadRequest(format!("Invalid user id: {}", e)))?;

    if poll.creator_id != current_user {
        return Err(AppError::BadRequest("Only the Creator of the Poll is allowed to RESET that Poll".to_string()));
    }

    coll.update_one(
        doc! {"_id":obj_id},
        doc! {
            "$set":{
                "options.$[].votes":0,
                "is_closed":false,
                "total_votes":0,
            }
        },
    )
    .await?;

    let vote_coll = db.collection::<VoteRecord>("vote_records");

    vote_coll
        .delete_many(doc! { "poll_id": obj_id })
        .await?;

    let updated_poll = coll
        .find_one(doc! { "_id": obj_id })
        .await?
        .ok_or_else(|| AppError::NotFound("Poll not found".to_string()))?;
    
    Ok(Json(updated_poll))
}