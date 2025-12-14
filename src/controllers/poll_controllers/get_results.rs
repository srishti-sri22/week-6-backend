use std::sync::Arc;
use std::time::Duration;
use axum::{
    extract::{Extension, Path},
    response::sse::{Event, Sse},
};
use futures::stream::{self, Stream};
use mongodb::{
    Database,
    bson::{doc, oid::ObjectId},
};
use tokio::time::sleep;

use crate::models::poll_models::Poll;
use crate::controllers::poll_controllers::models::PollResponse;
use crate::utils::error::{AppError, AppResult};

pub async fn poll_updates_stream(
    Path(poll_id): Path<String>,
    Extension(db): Extension<Arc<Database>>,
) -> AppResult<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>>> {
    
    let obj_id = ObjectId::parse_str(&poll_id)
        .map_err(|_| AppError::BadRequest("Invalid Poll id".to_string()))?;

    let stream = stream::unfold((db, obj_id), |(db, poll_id)| async move {
        sleep(Duration::from_secs(2)).await;

        let coll = db.collection::<Poll>("polls");
        
        match coll.find_one(doc! { "_id": poll_id }).await {
            Ok(Some(poll)) => {
                let poll_response = PollResponse {
                    id: poll.id.to_hex(),
                    question: poll.question,
                    creator_id: poll.creator_id.to_hex(),
                    options: poll.options,
                    is_closed: poll.is_closed,
                    created_at: poll.created_at,
                    total_votes: poll.total_votes,
                };

                match serde_json::to_string(&poll_response) {
                    Ok(json_data) => {
                        Some((Ok(Event::default().data(json_data)), (db, poll_id)))
                    }
                    Err(_) => None,
                }
            }
            _ => None,
        }
    });

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}