use anyhow::anyhow;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AtomicMetrics {
    data: Arc<HashMap<&'static str, AtomicI64>>,
}

impl fmt::Display for AtomicMetrics {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (key, value) in self.data.iter() {
            writeln!(f, "{}: {}", key, value.load(Ordering::Relaxed))?;
        }
        Ok(())
    }
}

impl AtomicMetrics {
    pub fn new(metric_names: &[&'static str]) -> Self {
        let map = metric_names
            .iter()
            .map(|&name| (name, AtomicI64::new(0)))
            .collect();
        AtomicMetrics {
            data: Arc::new(map),
        }
    }

    pub fn inc(&self, key: impl AsRef<str>) -> anyhow::Result<()> {
        let key = key.as_ref();

        let counter = self
            .data
            .get(key)
            .ok_or_else(|| anyhow!("key {} not found", key))?;
        counter.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub fn dec(&self, key: impl AsRef<str>) -> anyhow::Result<()> {
        let key = key.as_ref();

        let counter = self
            .data
            .get(key)
            .ok_or_else(|| anyhow!("key {} not found", key))?;
        counter.fetch_sub(1, Ordering::Relaxed);
        Ok(())
    }
}
