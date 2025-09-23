#[allow(dead_code)]
use anyhow::Result;
use notify_server::{get_router, setup_pg_listener};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    setup_pg_listener().await?;

    let addr = "127.0.0.1:6687";
    let app = get_router().await;
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: http://{}", addr);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
