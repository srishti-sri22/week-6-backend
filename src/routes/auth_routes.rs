use axum::{Router, routing::post};
use crate::controllers::auth_controllers::{auth_finish, auth_start, register_finish, register_start, logout};
use crate::state::AppState;

pub fn auth_routes(state: AppState) -> Router {
    Router::new()
        .route("/register/start", post(register_start::register_start))
        .route("/register/finish", post(register_finish::register_finish))
        .route("/login/start", post(auth_start::auth_start))
        .route("/login/finish", post(auth_finish::auth_finish))
        .route("/logout", post(logout::logout))
        .with_state(state)
}