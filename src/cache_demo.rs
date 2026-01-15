use dashmap::DashMap;
use pyo3::prelude::*;
use std::sync::Arc;

/// Demonstrates the cache pattern: synchronous reads, async background updates.
///
/// This class shows the core architectural pattern for driftpyrs:
/// - Python reads from the cache synchronously (instant, no await)
/// - Tokio background tasks write to the cache asynchronously
/// - DashMap provides lock-free concurrent access
#[pyclass]
pub struct CacheDemo {
    cache: Arc<DashMap<String, String>>,
}

#[pymethods]
impl CacheDemo {
    #[new]
    fn new() -> Self {
        #[cfg(feature = "observability")]
        tracing::info!("CacheDemo::new - creating new instance");

        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    /// Get a value from the cache synchronously.
    /// This is instant - no await needed, just reads from memory.
    fn get(&self, key: &str) -> Option<String> {
        self.cache.get(key).map(|v| v.clone())
    }

    /// Start background updates to the cache.
    /// This spawns a Tokio task that updates the cache every 100ms.
    /// Returns immediately once the background task is spawned.
    fn start_updates<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let cache = Arc::clone(&self.cache);

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            #[cfg(feature = "observability")]
            tracing::info!("start_updates - spawning background task");

            // Spawn background task that runs forever
            tokio::spawn(async move {
                let mut counter = 0u64;
                loop {
                    cache.insert("counter".to_string(), counter.to_string());

                    #[cfg(feature = "observability")]
                    if counter % 10 == 0 {
                        tracing::debug!(counter, "background task updating cache");
                    }

                    counter += 1;
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            });

            #[cfg(feature = "observability")]
            tracing::info!("start_updates - background task spawned");

            Ok(())
        })
    }

    /// Get all keys in the cache.
    fn keys(&self) -> Vec<String> {
        self.cache.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Get the number of entries in the cache.
    fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty.
    fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Clear all entries from the cache.
    fn clear(&self) {
        self.cache.clear();
    }
}
