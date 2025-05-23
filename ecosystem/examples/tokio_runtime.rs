use anyhow::Result;
use std::{thread, time::Duration};

#[tokio::main]
async fn main() -> Result<()> {
    let handle = thread::spawn(|| {
        // execute future
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.spawn(async {
            println!("Future 1!");
            let content = tokio::fs::read_to_string("Cargo.toml").await.unwrap();
            println!("Future 1 content length: {}", content.len());
        });

        rt.spawn(async {
            println!("Future 2!");
            let ret = expensive_blocking_task("Future 2".to_string());
            println!("Future 2 result: {}", ret);
        });

        // there is no reactor running,
        // must be called from the context of a Tokio 1.x runtime
        // rt.block_on(tokio::time::sleep(Duration::from_millis(900)));
        rt.block_on(sleep(Duration::from_millis(900)));
    });
    handle.join().unwrap();

    Ok(())
}

async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

fn expensive_blocking_task(name: String) -> String {
    thread::sleep(Duration::from_millis(800));
    blake3::hash(name.as_bytes()).to_string()
}
