//! Asynchronous backfill engine for decoupled cache coordination.
//!
//! This module implements the **Submission Handle** pattern to achieve strict
//! CQRS compliance. It separates the "intent to update" (Reading side) from
//! the "execution of update" (Writing side) using a non-blocking MPSC channel.
//!
//! Internal types drop the `Backfill` prefix for brevity, as they are
//! encapsulated within this module, but are re-exported with the prefix
//! at the `cache` module level.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use tokio::sync::mpsc;
use tracing::{debug, error};

use crate::spi::cache::CacheWriter;

/// Channel capacity presets for different workload profiles.
///
/// These presets balance memory usage with throughput requirements. Use
/// `Custom` for fine-tuned control when workload characteristics don't
/// match the standard profiles.
///
/// # Examples
///
/// ```rust
/// use lithos_adapters::spi::cache::{BackfillCapacity, new_backfiller};
///
/// // Use a preset
/// let capacity = BackfillCapacity::Heavy;
/// let (handle, worker) = new_backfiller::<String, String>(capacity.into());
///
/// // Or custom value
/// let capacity = BackfillCapacity::custom(2048).unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Capacity {
    /// Custom capacity with validation (128-16384 range).
    Custom(usize),
    /// Heavy workloads (> 1000 ops/sec) - 4096 slots.
    Heavy,
    /// Light workloads (< 100 ops/sec) - 256 slots.
    Light,
    /// Medium workloads (< 1000 ops/sec) - 1024 slots (default).
    Medium,
}

impl Capacity {
    /// Maximum allowed capacity for custom values.
    pub const MAX: usize = 0x4000;
    /// Minimum allowed capacity for custom values.
    pub const MIN: usize = 128;

    /// Create a validated custom capacity.
    ///
    /// # Errors
    /// Returns error string if capacity is outside valid range.
    ///
    /// # Example
    /// ```rust
    /// # use lithos_adapters::spi::cache::BackfillCapacity;
    /// let capacity = BackfillCapacity::custom(2048).unwrap();
    /// assert_eq!(capacity.value(), 2048);
    /// ```
    #[inline]
    pub const fn custom(capacity: usize) -> Result<Self, &'static str> {
        match Self::validate(capacity) {
            Ok(()) => Ok(Self::Custom(capacity)),
            Err(e) => Err(e),
        }
    }

    /// Validate a capacity value against allowed bounds.
    #[inline]
    const fn validate(capacity: usize) -> Result<(), &'static str> {
        if capacity < Self::MIN {
            return Err("capacity below minimum (128)");
        }
        if capacity > Self::MAX {
            return Err("capacity above maximum (16384)");
        }
        Ok(())
    }

    /// Convert to actual capacity value.
    ///
    /// # Example
    /// ```rust
    /// # use lithos_adapters::spi::cache::BackfillCapacity;
    /// assert_eq!(BackfillCapacity::Light.value(), 256);
    /// assert_eq!(BackfillCapacity::Medium.value(), 1024);
    /// assert_eq!(BackfillCapacity::Heavy.value(), 4096);
    /// ```
    #[inline]
    #[must_use]
    pub const fn value(self) -> usize {
        match self {
            Self::Custom(n) => n,
            Self::Heavy => 4096,
            Self::Light => 256,
            Self::Medium => 1024,
        }
    }
}

impl Default for Capacity {
    #[inline]
    fn default() -> Self {
        Self::Medium
    }
}

impl From<Capacity> for usize {
    #[inline]
    fn from(capacity: Capacity) -> Self {
        capacity.value()
    }
}

/// Metrics snapshot for backfill operations.
///
/// Provides insight into backfill health and performance for monitoring
/// and debugging purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Metrics {
    /// Total number of backfill requests successfully queued.
    pub triggered: u64,
    /// Total number of backfill requests dropped due to channel full/closed.
    pub dropped: u64,
    /// Current channel capacity (max buffered requests).
    pub channel_capacity: usize,
    /// Current number of available slots in the channel.
    pub channel_available: usize,
}

/// Internal atomic metrics shared between handle clones.
struct AtomicMetrics {
    triggered: AtomicU64,
    dropped: AtomicU64,
}

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
    metrics: Arc<AtomicMetrics>,
}

impl<K, V> Handle<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Get a snapshot of current backfill metrics.
    ///
    /// # Example
    /// ```rust
    /// # use lithos_adapters::spi::cache::new_backfiller;
    /// let (handle, _worker) = new_backfiller::<String, String>(1024);
    /// handle.trigger("key".to_string(), "value".to_string());
    /// let metrics = handle.metrics();
    /// assert_eq!(metrics.triggered, 1);
    /// assert_eq!(metrics.dropped, 0);
    /// ```
    #[inline]
    #[must_use]
    pub fn metrics(&self) -> Metrics {
        let capacity = self.tx.capacity();
        let max_capacity = self.tx.max_capacity();
        let available = max_capacity.saturating_sub(capacity);

        Metrics {
            triggered: self.metrics.triggered.load(Ordering::Relaxed),
            dropped: self.metrics.dropped.load(Ordering::Relaxed),
            channel_capacity: max_capacity,
            channel_available: available,
        }
    }

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
        _ = self.try_trigger(key, value);
    }

    /// Attempt to submit a backfill request, returning whether it was queued.
    ///
    /// This is useful for metrics or backpressure visibility.
    #[inline]
    pub fn try_trigger(&self, key: K, value: V) -> bool {
        let request = Request {
            key,
            value,
        };

        match self.tx.try_send(request) {
            Ok(()) => {
                self.metrics.triggered.fetch_add(1, Ordering::Relaxed);
                debug!(operation = "backfill", status = "triggered");
                true
            }
            Err(mpsc::error::TrySendError::Full(_)) => {
                self.metrics.dropped.fetch_add(1, Ordering::Relaxed);
                debug!(
                    operation = "backfill",
                    status = "dropped",
                    reason = "channel full"
                );
                false
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                self.metrics.dropped.fetch_add(1, Ordering::Relaxed);
                debug!(
                    operation = "backfill",
                    status = "dropped",
                    reason = "channel closed"
                );
                false
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
            metrics: Arc::clone(&self.metrics),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.tx.clone_from(&source.tx);
        self.metrics = Arc::clone(&source.metrics);
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
    let metrics = Arc::new(AtomicMetrics {
        triggered: AtomicU64::new(0),
        dropped: AtomicU64::new(0),
    });

    (
        Handle {
            tx,
            metrics,
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

    mod capacity {
        use super::*;

        #[test]
        fn rejects_capacity_below_minimum() {
            // GIVEN: a capacity below MIN
            let capacity = Capacity::MIN - 1;

            // WHEN: attempting to create custom capacity
            let result = Capacity::custom(capacity);

            // THEN: it should return an error
            let err =
                result.expect_err("Expected error for capacity below minimum");
            assert!(
                err.contains("below minimum"),
                "Error message should mention 'below minimum', got: {err}"
            );
        }

        #[test]
        fn rejects_capacity_above_maximum() {
            // GIVEN: a capacity above MAX
            let capacity = Capacity::MAX + 1;

            // WHEN: attempting to create custom capacity
            let result = Capacity::custom(capacity);

            // THEN: it should return an error
            let err =
                result.expect_err("Expected error for capacity above maximum");
            assert!(
                err.contains("above maximum"),
                "Error message should mention 'above maximum', got: {err}"
            );
        }

        #[test]
        fn accepts_capacity_at_boundaries() {
            // GIVEN: capacities at MIN and MAX boundaries
            // WHEN: creating custom capacities
            let min_capacity = Capacity::custom(Capacity::MIN)
                .expect("MIN boundary should be valid");
            let max_capacity = Capacity::custom(Capacity::MAX)
                .expect("MAX boundary should be valid");

            // THEN: both should have correct values
            assert_eq!(min_capacity.value(), Capacity::MIN);
            assert_eq!(max_capacity.value(), Capacity::MAX);
        }

        #[test]
        fn accepts_capacity_within_range() {
            // GIVEN: a capacity within valid range
            let capacity = 2048;

            // WHEN: creating custom capacity
            let result = Capacity::custom(capacity)
                .expect("Valid capacity should succeed");

            // THEN: it should return correct value
            assert_eq!(result.value(), capacity);
        }

        #[test]
        fn presets_have_correct_values() {
            // GIVEN: preset capacities
            // WHEN: getting their values
            // THEN: they should match expected values
            assert_eq!(Capacity::Light.value(), 256);
            assert_eq!(Capacity::Medium.value(), 1024);
            assert_eq!(Capacity::Heavy.value(), 4096);
        }

        #[test]
        fn default_is_medium() {
            // GIVEN: default capacity
            let capacity = Capacity::default();

            // WHEN: checking its value
            // THEN: it should be Medium (1024)
            assert_eq!(capacity, Capacity::Medium);
            assert_eq!(capacity.value(), 1024);
        }

        #[test]
        fn converts_to_usize() {
            // GIVEN: various capacities
            let capacities = [
                Capacity::Light,
                Capacity::Medium,
                Capacity::Heavy,
                Capacity::custom(2048).unwrap(),
            ];

            // WHEN: converting to usize
            // THEN: it should use the From trait
            for capacity in capacities {
                let value: usize = capacity.into();
                assert_eq!(value, capacity.value());
            }
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
            let (handle, _worker) = new::<String, String>(1);
            handle.trigger("key1".to_owned(), "value1".to_owned());

            // WHEN: another backfill is triggered
            // THEN: it should not block and should drop the request
            handle.trigger("key2".to_owned(), "value2".to_owned());

            // Metrics should reflect one triggered and one dropped
            let metrics = handle.metrics();
            assert_eq!(metrics.triggered, 1);
            assert_eq!(metrics.dropped, 1);
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
            let (handle, _worker) = new::<String, String>(1);

            // WHEN: a backfill is triggered
            handle.trigger("key".to_owned(), "value".to_owned());

            // THEN: the metrics should reflect it
            let metrics = handle.metrics();
            assert_eq!(metrics.triggered, 1);
            assert_eq!(metrics.dropped, 0);
        }

        #[tokio::test]
        async fn metrics_track_multiple_operations() {
            // GIVEN: a handle with capacity for 2
            let (handle, _worker) = new::<String, String>(2);

            // WHEN: we trigger 3 backfills
            assert!(handle.try_trigger("k1".to_owned(), "v1".to_owned()));
            assert!(handle.try_trigger("k2".to_owned(), "v2".to_owned()));
            assert!(!handle.try_trigger("k3".to_owned(), "v3".to_owned())); // Should drop

            // THEN: metrics should reflect 2 triggered and 1 dropped
            let metrics = handle.metrics();
            assert_eq!(metrics.triggered, 2);
            assert_eq!(metrics.dropped, 1);
            assert_eq!(metrics.channel_capacity, 2);
        }

        #[test]
        fn metrics_shared_across_clones() {
            // GIVEN: a handle and its clone
            let (handle1, _worker) = new::<String, String>(10);
            let handle2 = handle1.clone();

            // WHEN: both handles trigger backfills
            handle1.trigger("k1".to_owned(), "v1".to_owned());
            handle2.trigger("k2".to_owned(), "v2".to_owned());

            // THEN: both should report the same metrics
            let m1 = handle1.metrics();
            let m2 = handle2.metrics();
            assert_eq!(m1.triggered, 2);
            assert_eq!(m2.triggered, 2);
            assert_eq!(m1, m2);
        }
    }
}
