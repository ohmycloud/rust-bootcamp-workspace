mod sse;

use crate::sse::sse_handler;
use axum::Router;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use futures::StreamExt;
use sqlx::postgres::PgListener;
use tracing::info;

const INDEX_HTML: &str = include_str!("../index.html");

pub async fn get_router() -> Router {
    Router::new().route("/events", get(sse_handler))
}

pub async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}

pub async fn setup_pg_listener() -> anyhow::Result<()> {
    // Implementation of setting up PostgreSQL listener
    let mut listener =
        PgListener::connect("postgres://postgres:password@127.0.0.1:5432/chat").await?;
    listener.listen("user_added_to_chat").await?;
    listener.listen("new_chat_message_created").await?;

    let mut stream = listener.into_stream();
    tokio::spawn(async move {
        while let Some(Ok(notification)) = stream.next().await {
            info!("Received notification: {:?}", notification);
        }
    });
    Ok(())
}
