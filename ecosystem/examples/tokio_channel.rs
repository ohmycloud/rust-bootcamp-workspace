use anyhow::Result;
use chrono::Utc;
use std::{thread, time::Duration};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = tokio::sync::mpsc::channel(32);
    let handle = worker(rx);

    tokio::spawn(async move {
        let mut i = 0;
        loop {
            i += 1;
            let message = format!("Send task {} - {}", i, Utc::now().to_rfc3339());
            println!("{}", message);
            tx.send(message).await?;
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });

    handle.join().unwrap();

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
