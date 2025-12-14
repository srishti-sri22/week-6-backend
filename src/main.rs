use axum::{Router, extract::Extension, http::{HeaderValue, Method}, response::Json, routing::get};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use dotenvy::dotenv;
use tower_http::cors::CorsLayer;

mod db;
mod routes;
mod controllers;
mod models;
mod utils;


#[tokio::main]
async fn main() {
    dotenv().ok();

    let database = match db::connection::init_db().await {
        Ok(db) => Arc::new(db),
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    let webauthn = match utils::webauthn::init_webauthn() {
        Ok(wa) => wa,
        Err(e) => {
            eprintln!("Failed to initialize webauthn: {}", e);
            std::process::exit(1);
        }
    };

    let origin = "http://localhost:3000".parse::<HeaderValue>()
        .unwrap_or_else(|_| {
            eprintln!("Failed to parse CORS origin");
            std::process::exit(1);
        });

    let cors = CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::header::USER_AGENT,
            axum::http::header::HeaderName::from_static("x-requested-with"),
        ])
        .allow_credentials(true);

    let app = Router::new()
        .route("/", get(root))
        .nest("/api/auth", routes::auth_routes::auth_routes())
        .nest("/api/polls", routes::poll_routes::poll_routes())
        .layer(cors)
        .layer(Extension(database.clone()))
        .layer(Extension(webauthn));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Server running at http://{}", addr);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to address {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}

async fn root() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "message": "Backend is running!"
    }))
}