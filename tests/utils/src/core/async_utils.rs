//! # Async Testing Helpers
//!
//! This module provides standardized utilities for testing async code in
//! Lithos, following best practices for Tokio-based async operations.

use std::{future::Future, path::PathBuf, sync::Arc, time::Duration};

use tokio::{
    sync::{Mutex, RwLock, Semaphore},
    time::timeout,
};

use crate::fs::temp::TempDir;

/// Isolated test context providing unique resources for parallel testing.
pub struct IsolatedTestContext {
    /// Unique temporary directory for this test
    pub temp_dir: TempDir,
    /// Unique database name for this test
    pub db_name: String,
    /// Name of the test
    pub test_name: String,
}

impl IsolatedTestContext {
    /// Creates a new isolated test context.
    ///
    /// # Panics
    ///
    /// Panics if the temporary directory cannot be created.
    // # LINT_DISABLE_REASON: Test context initialization uses expect for
    // simplicity. # LINT_DISABLE_REASON: Options tried: manual Result
    // propagation. # LINT_DISABLE_REASON: Justification: fatal
    // initialization error in tests should panic.
    #[allow(clippy::expect_used, clippy::disallowed_methods)]
    pub fn new(test_name: &str) -> Self {
        let temp_dir = TempDir::with_prefix(test_name)
            .expect("Failed to create isolated temp dir");
        let db_name = format!("{}.redb", test_name);
        Self {
            temp_dir,
            db_name,
            test_name: test_name.to_string(),
        }
    }

    /// Returns the absolute path to the database file within the isolated
    /// context.
    #[must_use]
    pub fn db_path(&self) -> PathBuf {
        self.temp_dir.path().join(&self.db_name)
    }
}

/// Factory for creating isolated test contexts.
pub struct TestContextFactory {
    base_name: String,
}

impl TestContextFactory {
    /// Creates a new test context factory.
    #[must_use]
    pub fn new(base_name: &str) -> Self {
        Self {
            base_name: base_name.to_string(),
        }
    }

    /// Generates a new unique isolated test context.
    #[must_use]
    pub fn create_context(&self) -> IsolatedTestContext {
        let unique_name =
            crate::fs::temp::generate_unique_name(&self.base_name);
        IsolatedTestContext::new(&unique_name)
    }
}

/// Helper to wrap a future with a timeout, preventing hanging tests.
///
/// # Arguments
///
/// * `duration` - Maximum duration to wait for the future to complete
/// * `future` - The async future to execute with a timeout
///
/// # Returns
///
/// Returns `Ok(T)` if the future completes within the timeout, or
/// `Err(Elapsed)` if the timeout is exceeded.
///
/// # Usage
///
/// ```rust
/// use lithos_test_utils::with_timeout;
/// use tokio::time::Duration;
///
/// # #[tokio::main]
/// # async fn main() {
/// let result = with_timeout(Duration::from_secs(5), async {
///     // Some async operation
///     tokio::time::sleep(Duration::from_millis(100)).await;
///     42
/// })
/// .await;
///
/// assert_eq!(result.unwrap(), 42);
/// # }
/// ```
///
/// # Why Timeouts?
///
/// Async tests can hang indefinitely if:
/// - Deadlocks occur in synchronization primitives
/// - Await points never resolve
/// - Channels are never closed
///
/// Timeouts prevent individual tests from blocking the entire test suite and
/// provide clear error messages about which test timed out.
pub async fn with_timeout<F, T>(
    duration: Duration,
    future: F,
) -> Result<T, tokio::time::error::Elapsed>
where
    F: Future<Output = T>,
{
    timeout(duration, future).await
}

/// Helper to execute blocking operations in async tests using `spawn_blocking`.
///
/// Use this for CPU-intensive tasks, blocking I/O operations, or Redb
/// transactions that should not block the async runtime threads.
///
/// # Arguments
///
/// * `f` - A closure containing blocking operations
///
/// # Returns
///
/// Returns the result of the blocking operation.
///
/// # Usage
///
/// ```rust
/// use lithos_test_utils::spawn_blocking_test;
///
/// # #[tokio::main]
/// # async fn main() {
/// let result = spawn_blocking_test(|| {
///     // Blocking operation here (e.g., heavy computation, std::fs operations)
///     std::thread::sleep(std::time::Duration::from_millis(100));
///     42
/// })
/// .await;
///
/// assert_eq!(result.unwrap(), 42);
/// # }
/// ```
///
/// # Safety Invariant
///
/// According to Lithos project rules:
/// - NEVER block an async thread for >10ms
/// - Use `spawn_blocking` for all `std::fs` operations, heavy CPU rendering, or
///   `Redb` write transactions
/// - Blocking operations in async tests must use this helper to prevent runtime
///   thread starvation
///
/// # Examples of operations that need `spawn_blocking`:
///
/// - `std::fs::File` operations (reading/writing files)
/// - Heavy CPU computations
/// - `Redb` write transactions (which can block for extended periods)
/// - Synchronous HTTP requests
pub async fn spawn_blocking_test<F, R>(
    f: F,
) -> Result<R, tokio::task::JoinError>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(f).await
}

/// Helper to wrap a test with cancellation support for graceful shutdown.
///
/// This is useful for testing async operations that should respond to shutdown
/// signals, such as actors, background tasks, or event bus subscribers.
///
/// # Arguments
///
/// * `duration` - Timeout duration for the test
/// * `test_fn` - Async test function that receives a CancellationToken
///
/// # Returns
///
/// Returns `Ok(T)` if the test completes successfully within the timeout,
/// or `Err` if the timeout is exceeded or the test panics.
///
/// # Usage
///
/// ```rust
/// use lithos_test_utils::with_cancellation;
/// use tokio::time::Duration;
///
/// # #[tokio::main]
/// # async fn main() {
/// let result =
///     with_cancellation(Duration::from_secs(5), |cancel| async move {
///         // Test code that respects cancellation
///         tokio::select! {
///             _ = cancel.cancelled() => {
///                 Ok("Cancelled")
///             }
///             result = async { Ok("Done") } => {
///                 result
///             }
///         }
///     })
///     .await;
///
/// assert!(result.is_ok());
/// # }
/// ```
///
/// # Why Cancellation Testing?
///
/// According to Lithos project rules:
/// - All actors must use `tokio::select!` to listen for a global
///   `broadcast::Receiver` shutdown signal
/// - On shutdown, actors MUST complete the current atomic transaction before
///   exiting
///
/// This helper ensures that async operations properly handle cancellation
/// signals and clean up resources (e.g., complete transactions, release locks)
/// when tests complete.
pub async fn with_cancellation<F, Fut, T>(
    duration: Duration,
    test_fn: F,
) -> Result<T, Box<dyn std::error::Error>>
where
    F: FnOnce(tokio_util::sync::CancellationToken) -> Fut,
    Fut: Future<Output = Result<T, Box<dyn std::error::Error>>>,
{
    let token = tokio_util::sync::CancellationToken::new();

    match with_timeout(duration, test_fn(token.clone())).await {
        Ok(result) => result,
        Err(err) => {
            token.cancel();
            Err(err.into())
        }
    }
}

/// Helper to create a timeout for tests with sensible defaults.
///
/// Returns a 5-second timeout, which is appropriate for most unit tests.
/// Adjust this duration based on the specific operation being tested.
///
/// # Usage
///
/// ```rust
/// use lithos_test_utils::default_test_timeout;
///
/// let timeout = default_test_timeout(); // Duration::from_secs(5)
/// ```
#[must_use]
pub fn default_test_timeout() -> Duration {
    Duration::from_secs(5)
}

/// Helper to create a short timeout for quick operations (e.g., simple
/// calculations).
///
/// Returns a 1-second timeout, suitable for tests that should complete very
/// quickly.
#[must_use]
pub fn short_test_timeout() -> Duration {
    Duration::from_secs(1)
}

/// Helper to create a long timeout for complex operations (e.g., indexing,
/// heavy processing).
///
/// Returns a 30-second timeout, suitable for tests involving heavy computation
/// or multiple async operations.
#[must_use]
pub fn long_test_timeout() -> Duration {
    Duration::from_secs(30)
}

/// Helper to create shared mutex state for race-free testing.
#[must_use]
pub fn shared_mutex<T>(value: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(value))
}

/// Helper to create shared RwLock state for concurrent tests.
#[must_use]
pub fn shared_rwlock<T>(value: T) -> Arc<RwLock<T>> {
    Arc::new(RwLock::new(value))
}

/// Helper to create a shared semaphore for concurrency throttling.
#[must_use]
pub fn shared_semaphore(permits: usize) -> Arc<Semaphore> {
    Arc::new(Semaphore::new(permits))
}

/// Polls a condition until it becomes true or the timeout expires.
///
/// # Errors
///
/// Returns an error if the condition does not become true within the timeout.
pub async fn poll_condition<F, Fut>(
    mut condition: F,
    timeout_duration: Duration,
    interval: Duration,
) -> Result<(), String>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = bool>,
{
    let deadline = tokio::time::Instant::now() + timeout_duration;

    loop {
        if condition().await {
            return Ok(());
        }

        if tokio::time::Instant::now() >= deadline {
            return Err(format!(
                "Condition not met within {timeout_duration:?}"
            ));
        }

        tokio::time::sleep(interval).await;
    }
}

/// Macro for async tests with proper runtime configuration.
///
/// This macro wraps tests with `#[tokio::test(flavor = "multi_thread",
/// worker_threads = 2)]` to ensure consistent test behavior and surface race
/// conditions in async operations.
///
/// # Usage
///
/// ```rust
/// use lithos_test_utils::async_test;
///
/// async_test!(
///     async fn my_async_function_test() {
///         // Your test code here
///         assert_eq!(1 + 1, 2);
///     }
/// );
/// ```
///
/// # Why multi_thread?
///
/// Using `multi_thread` flavor with multiple worker threads helps surface race
/// conditions that might not appear in single-threaded tests. This is critical
/// for testing async code that involves concurrent operations, event buses, or
/// shared state.
///
/// # Safety Invariants
///
/// - NEVER perform blocking I/O or heavy CPU tasks inside an async fn without
///   `spawn_blocking`
/// - This macro ensures proper runtime setup for each test
/// - Tests are properly isolated and can run concurrently
#[macro_export]
macro_rules! async_test {
    ($(#[$meta:meta])* $vis:vis async fn $name:ident() $body:block) => {
        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        $(#[$meta])*
        $vis async fn $name() $body
    };
}

/// Macro for async tests with a paused virtual clock.
///
/// This macro wraps tests with `#[tokio::test(flavor = "multi_thread",
/// worker_threads = 2)]` and automatically pauses the virtual clock, allowing
/// deterministic testing of timeouts and delays using `tokio::time::advance`.
///
/// # Usage
///
/// ```rust
/// use lithos_test_utils::time_test;
/// use tokio::time::Duration;
///
/// time_test!(
///     async fn test_with_delay() {
///         let (tx, mut rx) = tokio::sync::mpsc::channel(1);
///
///         tokio::spawn(async move {
///             tokio::time::sleep(Duration::from_secs(10)).await;
///             tx.send(42).await.unwrap();
///         });
///
///         tokio::time::advance(Duration::from_secs(11)).await;
///         assert_eq!(rx.recv().await.unwrap(), 42);
///     }
/// );
/// ```
#[macro_export]
macro_rules! time_test {
    ($(#[$meta:meta])* $vis:vis async fn $name:ident() $body:block) => {
        #[tokio::test(flavor = "current_thread")]
        $(#[$meta])*
        $vis async fn $name() {
            tokio::time::pause();
            $body
        }
    };
}

#[cfg(test)]
// # LINT_DISABLE_REASON: Assertion macros in tests trigger disallowed-method
// linting. # LINT_DISABLE_REASON: Options tried: explicit matches/guarded
// Result handling. # LINT_DISABLE_REASON: Justification: keep tests readable
// without unwrap/expect.
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn with_timeout_returns_value_before_deadline() {
        let result =
            with_timeout(Duration::from_millis(100), async { 42 }).await;

        assert!(matches!(result, Ok(42)), "expected Ok(42), got {result:?}");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn with_timeout_errors_after_deadline() {
        let result = with_timeout(Duration::from_millis(10), async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            42
        })
        .await;

        assert!(result.is_err(), "expected timeout error, got {result:?}");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn spawn_blocking_returns_result() {
        let result = spawn_blocking_test(|| {
            let mut sum = 0;
            for i in 0..1_000 {
                sum += i;
            }
            sum
        })
        .await;

        assert!(
            matches!(result, Ok(499_500)),
            "unexpected spawn result: {result:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn with_cancellation_returns_value_before_cancel() {
        let result = with_cancellation(Duration::from_millis(100), |cancel| async move {
            tokio::select! {
                _ = cancel.cancelled() => {
                    Err("cancelled".into())
                }
                result = async { Ok::<_, Box<dyn std::error::Error>>(42) } => result
            }
        })
        .await;

        assert!(
            matches!(result, Ok(42)),
            "unexpected cancellation result: {result:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn with_cancellation_times_out() {
        let result = with_cancellation(
            Duration::from_millis(10),
            |_cancel| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                Ok::<_, Box<dyn std::error::Error>>(42)
            },
        )
        .await;

        assert!(result.is_err(), "expected timeout error, got {result:?}");
    }

    #[test]
    fn default_test_timeout_is_five_seconds() {
        let timeout = default_test_timeout();

        assert_eq!(timeout, Duration::from_secs(5));
    }

    #[test]
    fn short_test_timeout_is_one_second() {
        let timeout = short_test_timeout();

        assert_eq!(timeout, Duration::from_secs(1));
    }

    #[test]
    fn long_test_timeout_is_thirty_seconds() {
        let timeout = long_test_timeout();

        assert_eq!(timeout, Duration::from_secs(30));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shared_mutex_allows_mutation() {
        let mutex = shared_mutex(0);
        *mutex.lock().await += 1;

        assert_eq!(*mutex.lock().await, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shared_rwlock_allows_write_access() {
        let rwlock = shared_rwlock(0);
        *rwlock.write().await += 1;

        assert_eq!(*rwlock.read().await, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shared_semaphore_allows_acquire() {
        let semaphore = shared_semaphore(2);
        let permit = semaphore.acquire().await;

        assert!(permit.is_ok(), "expected semaphore permit, got {permit:?}");
    }

    #[test]
    fn isolated_test_context_provides_unique_paths() {
        let ctx1 = IsolatedTestContext::new("test1");
        let ctx2 = IsolatedTestContext::new("test2");

        assert_ne!(ctx1.temp_dir.path(), ctx2.temp_dir.path());
        assert_ne!(ctx1.db_path(), ctx2.db_path());
        assert!(ctx1.db_path().ends_with("test1.redb"));
    }

    #[test]
    fn test_context_factory_generates_unique_contexts() {
        let factory = TestContextFactory::new("my_test");
        let ctx1 = factory.create_context();
        let ctx2 = factory.create_context();

        assert_ne!(ctx1.temp_dir.path(), ctx2.temp_dir.path());
        assert!(ctx1.test_name.contains("my_test"));
    }
}
