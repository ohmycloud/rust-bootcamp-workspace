use anyhow::{anyhow, Result};
use std::time::Duration;
use std::{sync::mpsc, thread};

#[derive(Debug)]
struct Msg {
    idx: usize,
    value: u8,
}
fn producer(idx: usize, tx: mpsc::Sender<Msg>) -> Result<()> {
    loop {
        let rand_value: u8 = rand::random::<u8>();
        let msg = Msg {
            idx,
            value: rand_value,
        };

        tx.send(msg)?;
        let sleep_time = rand::random::<u8>() as u64 * 10;
        thread::sleep(Duration::from_millis(sleep_time));

        if rand::random::<u8>() % 5 == 0 {
            println!("producer {} exit!", idx);
            break Ok(());
        }
    }
}

fn consumer(rx: mpsc::Receiver<Msg>) -> Result<()> {
    // create consumer
    let consumer = thread::spawn(move || {
        for msg in rx {
            println!("consumer: {:?}", msg);
        }
        println!("consumer exit!");
    });
    consumer
        .join()
        .map_err(|e| anyhow!("Thread join error: {:?}", e))
}

const NUM_PRODUCES: usize = 4;
fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel();

    // create producers
    for idx in 0..=NUM_PRODUCES {
        let tx = tx.clone();
        thread::spawn(move || producer(idx, tx));
    }

    drop(tx);

    consumer(rx)?;

    Ok(())
}
