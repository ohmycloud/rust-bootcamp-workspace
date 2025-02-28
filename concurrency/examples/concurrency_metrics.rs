use anyhow::Result;
use mpsc::metrics::ConcurrencyMetrics;
use rand::Rng;
use std::thread;
use std::time::Duration;

const N: usize = 2;
const M: usize = 4;

fn task_worker(idx: usize, metrics: ConcurrencyMetrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();
            thread::sleep(Duration::from_millis(rng.gen_range(100..5000)));
            metrics.inc(format!("call.thread.worker.{}", idx))?;
        }
        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });
    Ok(())
}

fn request_worker(metrics: ConcurrencyMetrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();

            thread::sleep(Duration::from_millis(rng.gen_range(50..800)));
            let page = rng.gen_range(1..256);
            metrics.inc(format!("req.page.{}", page))?;
        }

        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });

    Ok(())
}
fn main() -> Result<()> {
    let metrics = ConcurrencyMetrics::new();
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
