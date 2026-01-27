//! Asynchronous backfill engine for decoupled cache coordination.
//!
//! This module implements the **Submission Handle** pattern to achieve strict
//! CQRS compliance. It separates the "intent to update" (Reading side) from
//! the "execution of update" (Writing side) using a non-blocking MPSC channel.
//!
//! Internal types drop the `Backfill` prefix for brevity, as they are
//! encapsulated within this module, but are re-exported with the prefix
//! at the `cache` module level.

use std::sync::Arc;

use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::spi::cache::CacheWriter;

/// Submission handle for triggering background backfills.
///
/// This handle is cheaply cloneable and provides a non-blocking API for the
/// `Reader` to notify the system about cache misses that should be backfilled
/// to the fast memory layer.
///
/// # Example
///
/// ```rust
/// # use lithos_adapters::spi::cache::BackfillHandle;
/// # use tokio::sync::mpsc;
/// # // In practice, use new_backfiller()
/// ```
pub struct Handle<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    tx: mpsc::Sender<Request<K, V>>,
}

impl<K, V> Handle<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Non-blocking submission of a backfill request.
    ///
    /// This method uses `try_send` to ensure that the caller (typically a
    /// cache reader) is never blocked by backfill processing or channel
    /// congestion. If the internal buffer is full or the worker has stopped,
    /// the request is dropped and a diagnostic event is logged.
    ///
    /// # Performance
    /// Constant time O(1) complexity.
    #[inline]
    pub fn trigger(&self, key: K, value: V) {
        let request = Request {
            key,
            value,
        };

        match self.tx.try_send(request) {
            Ok(()) => {
                debug!(operation = "backfill", status = "triggered");
            }
            Err(mpsc::error::TrySendError::Full(_)) => {
                info!(
                    operation = "backfill",
                    status = "dropped",
                    reason = "channel full"
                );
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                info!(
                    operation = "backfill",
                    status = "dropped",
                    reason = "channel closed"
                );
            }
        }
    }
}

impl<K, V> Clone for Handle<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.tx.clone_from(&source.tx);
    }
}

/// Lifecycle-managed worker that processes backfill requests.
///
/// The worker owns the receiving end of the backfill channel and is responsible
/// for executing the actual `put` operations on the fast cache layer.
pub struct Worker<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    rx: mpsc::Receiver<Request<K, V>>,
}

impl<K, V> Worker<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Starts the background task with the provided writer.
    ///
    /// This method consumes the worker to ensure that the background task is
    /// only started once. It spawns a new Tokio task that processes requests
    /// until the channel is closed.
    ///
    /// # Errors
    /// Errors occurring during the `writer.put` operation are logged but do not
    /// stop the worker or affect the caller.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::sync::Arc;
    /// # use async_trait::async_trait;
    /// # use lithos_adapters::spi::cache::{BackfillWorker, CacheWriter, new_backfiller};
    /// # use lithos_adapters::spi::errors::CacheError;
    /// # struct DummyWriter;
    /// # #[async_trait]
    /// # impl CacheWriter<String, String> for DummyWriter {
    /// #     async fn put(&self, _: String, _: String) -> Result<(), CacheError> { Ok(()) }
    /// #     async fn delete(&self, _: &String) -> Result<bool, CacheError> { Ok(true) }
    /// #     async fn clear(&self) -> Result<(), CacheError> { Ok(()) }
    /// # }
    /// # tokio::runtime::Runtime::new().unwrap().block_on(async {
    /// let (handle, worker) = new_backfiller::<String, String>(10);
    /// worker.start(Arc::new(DummyWriter));
    /// # });
    /// ```
    #[inline]
    pub fn start(self, writer: Arc<dyn CacheWriter<K, V>>) {
        let mut rx = self.rx;

        tokio::spawn(async move {
            debug!(operation = "backfill", status = "started");

            while let Some(request) = rx.recv().await {
                if let Err(e) =
                    writer.put(request.key.clone(), request.value).await
                {
                    error!(
                        operation = "backfill",
                        status = "error",
                        error = ?e,
                        "Failed to backfill key"
                    );
                } else {
                    debug!(operation = "backfill", status = "success");
                }
            }

            debug!(operation = "backfill", status = "stopped");
        });
    }
}

/// Internal request type for backfill.
struct Request<K, V> {
    key: K,
    value: V,
}

/// Type alias for the decoupled handle/worker pair returned by `new`.
pub type HandleWorkerPair<K, V> = (Handle<K, V>, Worker<K, V>);

/// Factory function to create the decoupled handle/worker pair.
///
/// Returns a `(Handle, Worker)` tuple where the handle is used for submission
/// and the worker is used for execution.
///
/// # Example
///
/// ```rust
/// use lithos_adapters::spi::cache::new_backfiller;
///
/// let (handle, worker) = new_backfiller::<String, String>(1024);
/// ```
#[inline]
#[must_use]
pub fn new<K, V>(capacity: usize) -> HandleWorkerPair<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    let (tx, rx) = mpsc::channel(capacity);
    (
        Handle {
            tx,
        },
        Worker {
            rx,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    mod execution {
        use super::*;
        use crate::spi::cache::MockCacheWriter;

        #[tokio::test]
        async fn worker_processes_requests() {
            // GIVEN: a worker and a mock writer
            let (tx, rx) = mpsc::channel(1);
            let worker = Worker {
                rx,
            };
            let mut mock_writer = MockCacheWriter::<String, String>::new();

            mock_writer
                .expect_put()
                .with(
                    mockall::predicate::eq("key".to_owned()),
                    mockall::predicate::eq("value".to_owned()),
                )
                .returning(|_, _| Box::pin(async { Ok(()) }))
                .times(1);

            let writer = Arc::new(mock_writer);

            // WHEN: the worker is started and a request is sent
            worker.start(writer);

            tx.send(Request {
                key: "key".to_owned(),
                value: "value".to_owned(),
            })
            .await
            .unwrap();

            // THEN: the writer should receive the put command
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }

    mod initialization {
        use super::*;

        #[test]
        fn factory_creates_handle_worker_pair() {
            // GIVEN: a desired channel capacity
            let capacity = 10;

            // WHEN: the factory is called
            let (_handle, _worker) = new::<String, String>(capacity);

            // THEN: it should return a valid handle/worker pair
        }

        #[test]
        fn verifies_compilation() {
            // GIVEN: the module is correctly linked
            // WHEN: the test runs
            // THEN: it should compile and pass
        }
    }

    mod submission {
        use super::*;

        #[tokio::test]
        async fn drops_requests_on_full_channel() {
            // GIVEN: a handle with a full channel
            let (tx, _rx) = mpsc::channel(1);
            let handle = Handle {
                tx,
            };
            handle.trigger("key1".to_owned(), "value1".to_owned());

            // WHEN: another backfill is triggered
            // THEN: it should not block and should drop the request
            handle.trigger("key2".to_owned(), "value2".to_owned());
        }

        #[test]
        fn handle_is_cloneable() {
            // GIVEN: a backfill handle
            let (handle, _worker) = new::<String, String>(10);

            // WHEN: the handle is cloned
            let _clone = handle.clone();

            // THEN: it should succeed
        }

        #[tokio::test]
        async fn triggers_request_to_channel() {
            // GIVEN: a handle and its receiving end
            let (tx, mut rx) = mpsc::channel(1);
            let handle = Handle {
                tx,
            };

            // WHEN: a backfill is triggered
            handle.trigger("key".to_owned(), "value".to_owned());

            // THEN: the request should be received on the channel
            let req = rx.recv().await.expect("Should receive request");
            assert_eq!(req.key, "key");
            assert_eq!(req.value, "value");
        }
    }
}
