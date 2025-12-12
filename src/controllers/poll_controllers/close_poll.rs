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
    poll_models::{Poll}
};
use crate::controllers::poll_controllers::models::{CreatorOnly};

pub async fn close_poll(
    Path(poll_id): Path<String>,
    Extension(db): Extension<Arc<Database>>,
    Json(payload): Json<CreatorOnly>,
) -> Result<Json<Poll>, (StatusCode, String)> {
    let obj_id = ObjectId::parse_str(&poll_id).map_err(|e|(StatusCode::BAD_REQUEST, e.to_string()))?;
    let coll = db.collection::<Poll>("polls");

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

    let current_user = ObjectId::parse_str(&payload.user_id).map_err(|e|(StatusCode::BAD_REQUEST, e.to_string()))?;

    if poll.creator_id != current_user {
        return Err((
            StatusCode::FORBIDDEN,
            "Only the Creator of the Poll is allowed to CLOSE that Poll".to_string(),
        ));
    }

    coll.update_one(
        doc! { "_id": obj_id },
        doc! { "$set": { "is_closed": true } },
    )
    .await
    .map_err(|e|(StatusCode::BAD_REQUEST, e.to_string()))?;

    let updated_poll = coll
        .find_one(doc! { "_id": obj_id })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Poll not found".to_string()))?;
    Ok(Json(updated_poll))
}
