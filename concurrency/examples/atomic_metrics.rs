use anyhow::Result;
use mpsc::metrics::AtomicMetrics;
use rand::Rng;
use std::thread;
use std::time::Duration;

const N: usize = 2;
const M: usize = 4;

fn task_worker(idx: usize, metrics: AtomicMetrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();
            thread::sleep(Duration::from_millis(rng.gen_range(100..5000)));
            metrics.inc(format!("call.thread.worker.{}", idx).as_str())?;
        }
        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });
    Ok(())
}

fn request_worker(metrics: AtomicMetrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();

            thread::sleep(Duration::from_millis(rng.gen_range(50..800)));
            let page = rng.gen_range(1..256);
            metrics.inc(format!("req.page.{}", page).as_str())?;
        }

        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });

    Ok(())
}
fn main() -> Result<()> {
    let metrics = AtomicMetrics::new(&[
        "call.thread.worker.0",
        "call.thread.worker.1",
        "call.thread.worker.2",
        "call.thread.worker.1",
        "req.page.1",
        "req.page.2",
        "req.page.3",
        "req.page.4",
    ]);
    metrics.inc("req.page.1")?;
    metrics.inc("call.thread.worker.1")?;

    for _ in 0..100 {
        metrics.inc("call.thread.worker.1")?;
    }

    println!("{:?}", metrics);

    // start N workers and M requesters
    for idx in 0..N {
        task_worker(idx, metrics.clone())?;
    }

    for _ in 0..M {
        request_worker(metrics.clone())?
    }

    loop {
        thread::sleep(Duration::from_millis(1000));
        println!("{}", metrics);
    }
}
