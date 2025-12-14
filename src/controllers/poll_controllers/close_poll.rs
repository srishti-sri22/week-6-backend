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
    poll_models::Poll
};
use crate::controllers::poll_controllers::models::CreatorOnly;
use crate::utils::error::{AppError, AppResult};

pub async fn close_poll(
    Path(poll_id): Path<String>,
    Extension(db): Extension<Arc<Database>>,
    Json(payload): Json<CreatorOnly>,
) -> AppResult<Json<Poll>> {
    let obj_id = ObjectId::parse_str(&poll_id)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    let coll = db.collection::<Poll>("polls");

    let poll = coll
        .find_one(doc! {"_id":obj_id})
        .await?
        .ok_or_else(|| AppError::NotFound("The Poll id does not exist".to_string()))?;

    let current_user = ObjectId::parse_str(&payload.user_id)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    if poll.creator_id != current_user {
        return Err(AppError::BadRequest("Only the Creator of the Poll is allowed to CLOSE that Poll".to_string()));
    }

    coll.update_one(
        doc! { "_id": obj_id },
        doc! { "$set": { "is_closed": true } },
    )
    .await?;

    let updated_poll = coll
        .find_one(doc! { "_id": obj_id })
        .await?
        .ok_or_else(|| AppError::NotFound("Poll not found".to_string()))?;
    
    Ok(Json(updated_poll))
}