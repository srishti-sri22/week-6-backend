use axum::{
    Json,
    extract::{Extension, Path, State},
};
use mongodb::{
    bson::{doc, oid::ObjectId},
};
use serde_json::json;

use crate::models::vote_record_models::VoteRecord;
use crate::utils::error::{AppError, AppResult};
use crate::utils::session::Claims;
use crate::state::AppState;

pub async fn check_user_vote(
    Path(poll_id): Path<String>,
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {
    
    let poll_obj_id = ObjectId::parse_str(&poll_id)
        .map_err(|_| AppError::BadRequest("Invalid poll_id".to_string()))?;
    
    let user_obj_id = ObjectId::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user_id".to_string()))?;
    
    let vote_collection = state.db.collection::<VoteRecord>("vote_records");
    
    let vote_record = vote_collection
        .find_one(doc! {
            "poll_id": poll_obj_id,
            "user_id": user_obj_id
        })
        .await?;
    
    match vote_record {
        Some(record) => Ok(Json(json!({
            "has_voted": true,
            "option_id": record.option_id
        }))),
        None => Ok(Json(json!({
            "has_voted": false
        })))
    }
}