use axum::{Router, routing::{get,post}};
use crate::controllers::poll_controllers::{cast_vote, change_vote, check_vote, close_poll, create_poll, get_poll, get_results, get_user_polls, polls, reset_poll};


pub fn poll_routes()-> Router<>{
    return Router::new()
    .route("/create",post(create_poll::create_poll))
    .route("/:pollId", get(get_poll::get_poll))
    .route("/:pollId/vote", post(cast_vote::cast_vote))
    .route("/:pollId/close",post(close_poll::close_poll))
    .route("/:pollId/reset",post(reset_poll::reset_poll))
    .route("/:pollId/stream", get(get_results::poll_updates_stream))
    .route("/:pollId/change/vote", post(change_vote::change_vote))
    .route("/",get(polls::get_all_polls))
    .route("/user/:user_id", get( get_user_polls::get_polls_by_user))
    .route("/:pollId/vote/check", get(check_vote::check_user_vote))
    ;

}