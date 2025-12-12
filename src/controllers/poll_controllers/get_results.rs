use axum::{
    Json,
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{Response, Sse, IntoResponse},
};
use axum::response::sse::Event;
use mongodb::{
    Database,
    bson::{doc, oid::ObjectId},
};
use crate::models::poll_models::Poll;
use crate::controllers::poll_controllers::models::ResultsParams;
use futures_util::StreamExt;
use tokio_stream::wrappers::IntervalStream;
use tokio::time::interval;
use std::convert::Infallible;
use std::time::Duration;
use async_stream::stream;
use std::sync::Arc;

// Get results for a single poll (with optional SSE)
pub async fn get_results(
    Path(poll_id): Path<String>,
    Query(params): Query<ResultsParams>,
    Extension(db): Extension<Arc<Database>>,
) -> Result<Response, (StatusCode, String)> {
    let obj_id = ObjectId::parse_str(&poll_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid poll id".into()))?;

    let coll = db.collection::<Poll>("polls");

    if params.live.unwrap_or(false) {
        let coll = coll.clone();
        let closed_filter = params.closed.unwrap_or(false);
        let creator_filter = params.creator.clone();

        let stream = stream! {
            let mut ticker = IntervalStream::new(interval(Duration::from_secs(1)));
            while ticker.next().await.is_some() {
                match coll.find_one(doc! { "_id": obj_id }).await {
                    Ok(Some(poll)) => {
                        if closed_filter && !poll.is_closed {
                            continue;
                        }
                        if let Some(c) = &creator_filter {
                            if let Ok(creator_oid) = ObjectId::parse_str(c) {
                                if poll.creator_id != creator_oid {
                                    continue;
                                }
                            }
                        }
                        let data = serde_json::to_string(&poll).unwrap_or_else(|_| "{}".to_string());
                        yield Ok::<Event, Infallible>(Event::default().data(data));
                        if poll.is_closed { break; }
                    }
                    Ok(None) => {
                        yield Ok(Event::default().data(r#"{"error":"poll not found"}"#));
                        break;
                    }
                    Err(e) => {
                        let err_msg = format!(r#"{{"error":"{}"}}"#, e.to_string());
                        yield Ok(Event::default().data(err_msg));
                        break;
                    }
                }
            }
        };

        let sse = Sse::new(stream);
        return Ok(sse.into_response());
    }

    let mut filter = doc! { "_id": obj_id };
    if params.closed.unwrap_or(false) {
        filter.insert("is_closed", true);
    }
    if let Some(c) = params.creator.clone() {
        let cid = ObjectId::parse_str(&c)
            .map_err(|_| (StatusCode::BAD_REQUEST, "invalid creator id".into()))?;
        filter.insert("creator_id", cid);
    }

    let poll = coll
        .find_one(filter)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "poll not found".into()))?;

    Ok(Json(poll).into_response())
}

// NEW: Get live results for ALL polls
pub async fn get_all_results_stream(
    Query(params): Query<ResultsParams>,
    Extension(db): Extension<Arc<Database>>,
) -> Result<Response, (StatusCode, String)> {
    let coll = db.collection::<Poll>("polls");
    let closed_filter = params.closed.unwrap_or(false);
    let creator_filter = params.creator.clone();

    let stream = stream! {
        let mut ticker = IntervalStream::new(interval(Duration::from_secs(1)));
        
        while ticker.next().await.is_some() {
            // Build MongoDB filter
            let mut filter = doc! {};
            if closed_filter {
                filter.insert("is_closed", true);
            }
            if let Some(c) = &creator_filter {
                if let Ok(creator_oid) = ObjectId::parse_str(c) {
                    filter.insert("creator_id", creator_oid);
                }
            }

            // Fetch all polls matching the filter
            match coll.find(filter).await {
                Ok(mut cursor) => {
                    // Collect all polls
                    let mut polls = Vec::new();
                    while let Some(result) = cursor.next().await {
                        match result {
                            Ok(poll) => polls.push(poll),
                            Err(e) => {
                                eprintln!("Error fetching poll: {}", e);
                            }
                        }
                    }

                    // Send each poll as a separate SSE event
                    for poll in &polls {
                        if let Ok(data) = serde_json::to_string(&poll) {
                            yield Ok::<Event, Infallible>(Event::default().data(data));
                        }
                    }

                    // If no polls found, send a heartbeat to keep connection alive
                    if polls.is_empty() {
                        yield Ok(Event::default().data(r#"{"heartbeat":true}"#));
                    }
                }
                Err(e) => {
                    let err_msg = format!(r#"{{"error":"{}"}}"#, e.to_string());
                    yield Ok(Event::default().data(err_msg));
                    // Don't break on error, continue streaming
                }
            }
        }
    };

    let sse = Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(30))
                .text("keep-alive")
        );
    
    Ok(sse.into_response())
}

// Alternative implementation: Send only changed polls (more efficient)
pub async fn get_all_results_stream_optimized(
    Query(params): Query<ResultsParams>,
    Extension(db): Extension<Arc<Database>>,
) -> Result<Response, (StatusCode, String)> {
    let coll = db.collection::<Poll>("polls");
    let closed_filter = params.closed.unwrap_or(false);
    let creator_filter = params.creator.clone();

    let stream = stream! {
        let mut ticker = IntervalStream::new(interval(Duration::from_secs(1)));
        let mut last_polls: std::collections::HashMap<String, Poll> = std::collections::HashMap::new();
        
        while ticker.next().await.is_some() {
            // Build MongoDB filter
            let mut filter = doc! {};
            if closed_filter {
                filter.insert("is_closed", true);
            }
            if let Some(c) = &creator_filter {
                if let Ok(creator_oid) = ObjectId::parse_str(c) {
                    filter.insert("creator_id", creator_oid);
                }
            }

            // Fetch all polls matching the filter
            match coll.find(filter).await {
                Ok(mut cursor) => {
                    let mut current_polls: std::collections::HashMap<String, Poll> = std::collections::HashMap::new();
                    
                    while let Some(result) = cursor.next().await {
                        match result {
                            Ok(poll) => {
                                let poll_id = poll.id.to_hex();
                                current_polls.insert(poll_id.clone(), poll);
                            }
                            Err(e) => {
                                eprintln!("Error fetching poll: {}", e);
                            }
                        }
                    }

                    // Send updates only for changed or new polls
                    for (poll_id, poll) in &current_polls {
                        let has_changed = match last_polls.get(poll_id) {
                            Some(old_poll) => {
                                // Check if votes or status changed
                                old_poll.total_votes != poll.total_votes ||
                                old_poll.is_closed != poll.is_closed
                            }
                            None => true, // New poll
                        };

                        if has_changed {
                            if let Ok(data) = serde_json::to_string(&poll) {
                                yield Ok::<Event, Infallible>(Event::default().data(data));
                            }
                        }
                    }

                    // Update last_polls for next iteration
                    last_polls = current_polls;

                    // Send heartbeat if no changes
                    if last_polls.is_empty() {
                        yield Ok(Event::default().data(r#"{"heartbeat":true}"#));
                    }
                }
                Err(e) => {
                    let err_msg = format!(r#"{{"error":"{}"}}"#, e.to_string());
                    yield Ok(Event::default().data(err_msg));
                }
            }
        }
    };

    let sse = Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(30))
                .text("keep-alive")
        );
    
    Ok(sse.into_response())
}