mod auth;
mod request_id;
mod server_time;

use crate::middlewares::request_id::set_request_id;
use crate::middlewares::server_time::ServerTimeLayer;
use axum::Router;
use axum::middleware::from_fn;
use tower::ServiceBuilder;
use tower_http::LatencyUnit;
use tower_http::compression::CompressionLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

pub use auth::verify_token;

const REQUEST_ID_HEADER: &str = "x-request-id";

pub fn set_layer(app: Router) -> Router {
    app.layer(
        ServiceBuilder::new()
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Micros),
                    ),
            )
            .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
            .layer(from_fn(set_request_id))
            .layer(ServerTimeLayer),
    )
}
