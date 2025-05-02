use anyhow::Result;
use std::{thread, time::Duration};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = tokio::sync::mpsc::channel(32);
    worker(rx);

    tokio::spawn(async move {
        loop {
            tx.send("Future 1".to_string()).await?;
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

fn expensive_blocking_task(name: String) -> String {
    thread::sleep(Duration::from_millis(800));
    blake3::hash(name.as_bytes()).to_string()
}

// tokio async task send message to worker for expensive blocking task
fn worker(mut rx: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Some(s) = rx.blocking_recv() {
            let ret = expensive_blocking_task(s);
            println!("Worker received: {}", ret);
        }
    })
}
