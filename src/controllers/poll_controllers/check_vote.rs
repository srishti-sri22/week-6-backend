use axum::{
    Json,
    extract::{Extension, Path, Query},
    http::StatusCode,
};
use std::sync::Arc;
use std::collections::HashMap;
use mongodb::{
    Database,
    bson::{doc, oid::ObjectId},
};
use serde_json::json;

use crate::models::vote_record_models::VoteRecord;

pub async fn check_user_vote(
    Path(poll_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    Extension(db): Extension<Arc<Database>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    
    // Get user_id from query params
    let user_id_str = params.get("user_id")
        .ok_or((StatusCode::BAD_REQUEST, "user_id query parameter is required".to_string()))?;
    
    // Parse poll_id
    let poll_obj_id = ObjectId::parse_str(&poll_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid poll_id".to_string()))?;
    
    // Parse user_id
    let user_obj_id = ObjectId::parse_str(user_id_str)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user_id".to_string()))?;
    
    // Query VoteRecord collection
    let vote_coll = db.collection::<VoteRecord>("vote_records");
    
    let vote_record = vote_coll
        .find_one(doc! {
            "poll_id": poll_obj_id,
            "user_id": user_obj_id
        })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
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