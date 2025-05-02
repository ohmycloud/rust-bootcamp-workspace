use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use dashmap::DashMap;
use futures::{SinkExt, StreamExt, stream::SplitStream};
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

const MAX_MESSAGE_LENGTH: usize = 128;

#[tokio::main]
pub async fn main() -> Result<()> {
    let layer = fmt::Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    // console_subscriber::init();

    let addr = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Starting chat server on {}", addr);
    let state = Arc::new(State::default());

    loop {
        let (stream, addr) = listener.accept().await?;
        let state = state.clone();
        info!("Accepted connect from {}", addr);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(state, addr, stream).await {
                info!("Error handling connection: {}", e);
            }
        });
    }
    #[allow(unreachable_code)]
    Ok::<(), anyhow::Error>(())
}

#[derive(Debug)]
enum Message {
    Joined(String),
    Left(String),
    Chat { sender: String, content: String },
}

impl Message {
    fn user_joined(username: &str) -> Self {
        let content = format!("{} has joined the chat", username);
        Self::Joined(content)
    }

    fn user_left(username: &str) -> Self {
        let content = format!("{} has left the chat", username);
        Self::Left(content)
    }

    fn chat(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Chat {
            sender: sender.into(),
            content: content.into(),
        }
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Joined(username) => write!(f, "User {} joined", username),
            Message::Left(username) => write!(f, "User {} left", username),
            Message::Chat { sender, content } => write!(f, "[{}] {}", sender, content),
        }
    }
}

#[derive(Debug)]
struct Peer {
    username: String,
    stream: SplitStream<Framed<tokio::net::TcpStream, LinesCodec>>,
}

#[derive(Debug, Default)]
struct State {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}

impl State {
    async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        for peer in self.peers.iter() {
            if peer.key() == &addr {
                continue;
            }
            if let Err(e) = peer.value().send(message.clone()).await {
                warn!("Error sending message to {}: {}", peer.key(), e);
            }
        }
    }

    async fn add(
        &self,
        addr: SocketAddr,
        username: String,
        stream: Framed<tokio::net::TcpStream, LinesCodec>,
    ) -> Peer {
        let (tx, mut rx) = mpsc::channel(MAX_MESSAGE_LENGTH);
        self.peers.insert(addr, tx);

        let (mut stream_tx, stream_rx) = stream.split();

        // receive messages from the peer
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = stream_tx.send(message.to_string()).await {
                    warn!("Error sending message to {}: {}", addr, e);
                    break;
                }
            }
        });

        // return peer
        Peer {
            username,
            stream: stream_rx,
        }
    }
}

async fn handle_connection(
    state: Arc<State>,
    addr: SocketAddr,
    stream: tokio::net::TcpStream,
) -> Result<()> {
    let mut stream = Framed::new(stream, LinesCodec::new());
    stream.send("Enter your username:").await?;

    let username = match stream.next().await {
        Some(Ok(username)) => username,
        Some(Err(e)) => return Err(e.into()),
        None => return Ok(()),
    };
    let mut peer = state.add(addr, username, stream).await;
    let message = Arc::new(Message::user_joined(&peer.username));
    state.broadcast(addr, message).await;

    while let Some(line) = peer.stream.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("Error receiving message from {}: {}", addr, e);
                break;
            }
        };

        let message = Arc::new(Message::chat(&peer.username, line));
        state.broadcast(addr, message).await;
    }

    // when the loop exit, peer has left the chat or line reading failed
    // remove the peer from the state
    state.peers.remove(&addr);

    // notify other peers that the peer has left the chat
    let message = Arc::new(Message::user_left(&peer.username));
    state.broadcast(addr, message).await;

    Ok(())
}
