use anyhow::Result;
use std::net::SocketAddr;
use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tracing::{info, warn};

const BUF_SIZE: usize = 4096;

async fn process_redis_connection(
    mut stream: tokio::net::TcpStream,
    raddr: SocketAddr,
) -> Result<()> {
    loop {
        stream.readable().await?;
        let mut buf = Vec::with_capacity(BUF_SIZE);

        // Try to read data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive
        match stream.try_read_buf(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                info!("read {} bytes", n);
                let line = String::from_utf8_lossy(&buf);
                info!("{:?}", line);
                stream.write_all(b"+OKK\r\n").await.expect("write failed");
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    warn!("Connection {:?} closed", raddr);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // build a listener
    let addr = "0.0.0.0:6379";
    let listener = TcpListener::bind(addr).await?;
    info!("listening on: {}", addr);

    loop {
        let (stream, socket_addr) = listener.accept().await?;
        info!("Accepted connection from: {}", socket_addr);

        tokio::spawn(async move {
            if let Err(e) = process_redis_connection(stream, socket_addr).await {
                warn!("Connection error with {}: {:?}", socket_addr, e);
            }
        });
    }
}
