use axum::{
    Json,
    extract::{Extension, Path, State},
};

use mongodb::{
    bson::{doc, oid::ObjectId},
};

use crate::models::{
    poll_models::Poll,
    vote_record_models::VoteRecord,
};
use crate::utils::error::{AppError, AppResult};
use crate::utils::session::Claims;
use crate::state::AppState;

pub async fn reset_poll(
    Path(poll_id): Path<String>,
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<Poll>> {
    let poll_collection = state.db.collection::<Poll>("polls");

    let poll_obj_id = ObjectId::parse_str(&poll_id)
        .map_err(|e| AppError::BadRequest("Invalid poll id".to_string()))?;

    let poll = poll_collection
        .find_one(doc! {"_id":poll_obj_id})
        .await?
        .ok_or_else(|| AppError::NotFound("The Poll id does not exist".to_string()))?;

    let current_user = ObjectId::parse_str(&claims.sub)
        .map_err(|e| AppError::BadRequest(format!("Invalid user id: {}", e)))?;

    if poll.creator_id != current_user {
        return Err(AppError::BadRequest("Only the Creator of the Poll is allowed to RESET that Poll".to_string()));
    }

    poll_collection.update_one(
        doc! {"_id":poll_obj_id},
        doc! {
            "$set":{
                "options.$[].votes":0,
                "is_closed":false,
                "total_votes":0,
            }
        },
    )
    .await?;

    let vote_collection = state.db.collection::<VoteRecord>("vote_records");

    vote_collection
        .delete_many(doc! { "poll_id": poll_obj_id })
        .await?;

    let updated_poll = poll_collection
        .find_one(doc! { "_id": poll_obj_id })
        .await?
        .ok_or_else(|| AppError::NotFound("Poll not found".to_string()))?;
    
    Ok(Json(updated_poll))
}