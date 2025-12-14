use axum::{
    Json,
    extract::{Extension, Path, Query},
};
use std::sync::Arc;
use std::collections::HashMap;
use mongodb::{
    Database,
    bson::{doc, oid::ObjectId},
};
use serde_json::json;

use crate::models::vote_record_models::VoteRecord;
use crate::utils::error::{AppError, AppResult};

pub async fn check_user_vote(
    Path(poll_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    Extension(db): Extension<Arc<Database>>,
) -> AppResult<Json<serde_json::Value>> {
    
    let user_id_str = params.get("user_id")
        .ok_or_else(|| AppError::BadRequest("user_id query parameter is required".to_string()))?;
    
    let poll_obj_id = ObjectId::parse_str(&poll_id)
        .map_err(|_| AppError::BadRequest("Invalid poll_id".to_string()))?;
    
    let user_obj_id = ObjectId::parse_str(user_id_str)
        .map_err(|_| AppError::BadRequest("Invalid user_id".to_string()))?;
    
    let vote_coll = db.collection::<VoteRecord>("vote_records");
    
    let vote_record = vote_coll
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