//! Asynchronous backfill implementation using the Submission Handle pattern.
//!
//! Internal types drop the `Backfill` prefix for brevity, as they are
//! encapsulated within this module.

use std::sync::Arc;

use tokio::sync::mpsc;

use crate::spi::cache::CacheWriter;

/// Submission handle for triggering background backfills.
pub struct Handle<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[allow(dead_code)]
    tx: mpsc::Sender<Request<K, V>>,
}

/// Lifecycle-managed worker that processes backfill requests.
pub struct Worker<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[allow(dead_code)]
    rx: mpsc::Receiver<Request<K, V>>,
}

impl<K, V> Worker<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Starts the background task with the provided writer.
    pub fn start(self, _writer: Arc<dyn CacheWriter<K, V>>) {
        // Implementation will follow in later subtasks
    }
}

/// Internal request type for backfill.
struct Request<K, V> {
    #[allow(dead_code)]
    key: K,
    #[allow(dead_code)]
    value: V,
}

#[cfg(test)]
mod tests {
    #[test]
    fn verifies_compilation() {
        assert!(true);
    }
}
