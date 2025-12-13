use axum::{Router, extract::Extension, http::{HeaderValue, Method}, response::Json, routing::get};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use dotenvy::dotenv;
use tower_http::cors::{CorsLayer, Any};

mod db;
mod routes;
mod controllers;
mod models;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database = Arc::new(db::connection::init_db().await);
    let webauthn = utils::webauthn::init_webauthn();

    let cors = CorsLayer::new()
    .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
    
    .allow_headers([
        axum::http::header::CONTENT_TYPE,
        axum::http::header::ACCEPT,
        axum::http::header::USER_AGENT,
        axum::http::header::HeaderName::from_static("x-requested-with"),
    ]).allow_credentials(true)
    ;

    let app = Router::new()
        .route("/", get(root))
        .nest("/api/auth", routes::auth_routes::auth_routes())
        .nest("/api/polls", routes::poll_routes::poll_routes())
        .layer(cors)
        .layer(Extension(database.clone()))
        .layer(Extension(webauthn));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();


}

async fn root() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "message": "Backend is running!"
    }))
}