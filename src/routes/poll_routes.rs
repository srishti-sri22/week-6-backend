use axum::{Router, routing::{get,post}};
use crate::controllers::poll_controllers::{create_poll,get_poll,cast_vote,reset_poll,close_poll,change_vote,get_results,polls,get_user_polls};


pub fn poll_routes()-> Router<>{
    return Router::new()
    .route("/create",post(create_poll::create_poll))
    .route("/:pollId", get(get_poll::get_poll))
    .route("/:pollId/vote", post(cast_vote::cast_vote))
    .route("/:pollId/close",post(close_poll::close_poll))
    .route("/:pollId/reset",post(reset_poll::reset_poll))
    .route("/:pollId/results", get(get_results::get_results))
    .route("/:pollId/change/vote", post(change_vote::change_vote))
    .route("/",get(polls::get_all_polls))
    .route("/user/:user_id", get( get_user_polls::get_polls_by_user))
    .route("/results/stream", get(get_results::get_all_results_stream_optimized))
    ;

}