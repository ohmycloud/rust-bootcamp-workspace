use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = fmt::Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    // proxy client traffic to upstream
    let config = resolve_config();
    let config = Arc::new(config);
    info!(
        "Upstream: {}, Listening on {}",
        config.upstream_addr, config.listen_addr
    );
    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    info!("Listening on {}", listener.local_addr()?);
    loop {
        let config = config.clone();
        let (client, addr) = listener.accept().await?;
        info!("Accepted connection from: {}", addr);

        tokio::spawn(async move {
            let upstream = tokio::net::TcpStream::connect(&config.upstream_addr).await?;
            proxy(client, upstream).await?;
            Ok::<(), anyhow::Error>(())
        });
    }
    #[allow(unreachable_code)]
    Ok::<(), anyhow::Error>(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    listen_addr: String,
    upstream_addr: String,
}

fn resolve_config() -> Config {
    Config {
        listen_addr: "127.0.0.1:8080".to_string(),
        upstream_addr: "127.0.0.1:8081".to_string(),
    }
}

async fn proxy(
    mut client: tokio::net::TcpStream,
    mut upstream: tokio::net::TcpStream,
) -> Result<()> {
    let (mut client_reader, mut client_writer) = client.split();
    let (mut upstream_reader, mut upstream_writer) = upstream.split();
    let client_to_upstream = tokio::io::copy(&mut client_reader, &mut upstream_writer);
    let upstream_to_client = tokio::io::copy(&mut upstream_reader, &mut client_writer);

    match tokio::try_join!(client_to_upstream, upstream_to_client) {
        Ok((client_bytes, upstream_bytes)) => {
            info!(
                "Client sent {} bytes to upstream, upstream sent {} bytes to client",
                client_bytes, upstream_bytes
            );
        }
        Err(e) => {
            warn!("Error proxying: {:?}", e);
        }
    }
    Ok(())
}
