mod sse;

use crate::sse::sse_handler;
use axum::Router;
use axum::response::{Html, IntoResponse};
use axum::routing::get;

const INDEX_HTML: &str = include_str!("../index.html");

pub async fn get_router() -> Router {
    Router::new().route("/events", get(sse_handler))
}

pub async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}
