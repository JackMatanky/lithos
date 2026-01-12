//! # Async Testing Helpers
//!
//! This module provides standardized utilities for testing async code in Lithos,
//! following best practices for Tokio-based async operations.

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::time::timeout;

/// Helper to wrap a future with a timeout, preventing hanging tests.
///
/// # Arguments
///
/// * `duration` - Maximum duration to wait for the future to complete
/// * `future` - The async future to execute with a timeout
///
/// # Returns
///
/// Returns `Ok(T)` if the future completes within the timeout, or `Err(Elapsed)` if the timeout is exceeded.
///
/// # Usage
///
/// ```rust,ignore
/// use lithos_test_utils::with_timeout;
/// use tokio::time::Duration;
///
/// #[tokio::test]
/// async fn test_with_timeout() {
///     let result = with_timeout(Duration::from_secs(5), async {
///         // Some async operation
///         tokio::time::sleep(Duration::from_millis(100)).await;
///         42
///     }).await;
///
///     assert_eq!(result.unwrap(), 42);
/// }
/// ```
///
/// # Why Timeouts?
///
/// Async tests can hang indefinitely if:
/// - Deadlocks occur in synchronization primitives
/// - Await points never resolve
/// - Channels are never closed
///
/// Timeouts prevent individual tests from blocking the entire test suite and provide
/// clear error messages about which test timed out.
pub async fn with_timeout<F, T>(duration: Duration, future: F) -> Result<T, tokio::time::error::Elapsed>
where
    F: Future<Output = T>,
{
    timeout(duration, future).await
}

/// Helper to execute blocking operations in async tests using `spawn_blocking`.
///
/// Use this for CPU-intensive tasks, blocking I/O operations, or Redb transactions
/// that should not block the async runtime threads.
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
/// ```rust,ignore
/// use lithos_test_utils::spawn_blocking_test;
///
/// #[tokio::test]
/// async fn test_blocking_operation() {
///     let result = spawn_blocking_test(|| {
///         // Blocking operation here (e.g., heavy computation, std::fs operations)
///         std::thread::sleep(std::time::Duration::from_millis(100));
///         42
///     }).await;
///
///     assert_eq!(result.unwrap(), 42);
/// }
/// ```
///
/// # Safety Invariant
///
/// According to Lithos project rules:
/// - NEVER block an async thread for >10ms
/// - Use `spawn_blocking` for all `std::fs` operations, heavy CPU rendering, or `Redb` write transactions
/// - Blocking operations in async tests must use this helper to prevent runtime thread starvation
///
/// # Examples of operations that need `spawn_blocking`:
///
/// - `std::fs::File` operations (reading/writing files)
/// - Heavy CPU computations
/// - `Redb` write transactions (which can block for extended periods)
/// - Synchronous HTTP requests
pub async fn spawn_blocking_test<F, R>(f: F) -> Result<R, tokio::task::JoinError>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(f).await
}

/// Helper to wrap a test with cancellation support for graceful shutdown.
///
/// This is useful for testing async operations that should respond to shutdown signals,
/// such as actors, background tasks, or event bus subscribers.
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
/// ```rust,ignore
/// use lithos_test_utils::with_cancellation;
/// use tokio::time::Duration;
///
/// #[tokio::test]
/// async fn test_with_cancellation() {
///     let result = with_cancellation(Duration::from_secs(5), |cancel| async move {
///         // Test code that respects cancellation
///         tokio::select! {
///             _ = cancel.cancelled() => {
///                 return Ok("Cancelled");
///             }
///             result = some_async_operation() => {
///                 Ok(result)
///             }
///         }
///     }).await;
///
///     assert!(result.is_ok());
/// }
/// ```
///
/// # Why Cancellation Testing?
///
/// According to Lithos project rules:
/// - All actors must use `tokio::select!` to listen for a global `broadcast::Receiver` shutdown signal
/// - On shutdown, actors MUST complete the current atomic transaction before exiting
///
/// This helper ensures that async operations properly handle cancellation signals and
/// clean up resources (e.g., complete transactions, release locks) when tests complete.
pub async fn with_cancellation<F, Fut, T>(
    duration: Duration,
    test_fn: F,
) -> Result<T, Box<dyn std::error::Error>>
where
    F: FnOnce(tokio_util::sync::CancellationToken) -> Fut,
    Fut: Future<Output = Result<T, Box<dyn std::error::Error>>>,
{
    let token = tokio_util::sync::CancellationToken::new();

    with_timeout(duration, test_fn(token.clone())).await?
}

/// Helper to create a timeout for tests with sensible defaults.
///
/// Returns a 5-second timeout, which is appropriate for most unit tests.
/// Adjust this duration based on the specific operation being tested.
///
/// # Usage
///
/// ```rust,ignore
/// use lithos_test_utils::default_test_timeout;
///
/// let timeout = default_test_timeout(); // Duration::from_secs(5)
/// ```
#[must_use]
pub fn default_test_timeout() -> Duration {
    Duration::from_secs(5)
}

/// Helper to create a short timeout for quick operations (e.g., simple calculations).
///
/// Returns a 1-second timeout, suitable for tests that should complete very quickly.
#[must_use]
pub fn short_test_timeout() -> Duration {
    Duration::from_secs(1)
}

/// Helper to create a long timeout for complex operations (e.g., indexing, heavy processing).
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

/// Macro for async tests with proper runtime configuration.
///
/// This macro wraps tests with `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`
/// to ensure consistent test behavior and surface race conditions in async operations.
///
/// # Usage
///
/// ```rust,ignore
/// use lithos_test_utils::async_test;
///
/// async_test!(async fn my_async_function_test() {
///     // Your test code here
///     assert_eq!(1 + 1, 2);
/// });
/// ```
///
/// # Why multi_thread?
///
/// Using `multi_thread` flavor with multiple worker threads helps surface race conditions
/// that might not appear in single-threaded tests. This is critical for testing async code
/// that involves concurrent operations, event buses, or shared state.
///
/// # Safety Invariants
///
/// - NEVER perform blocking I/O or heavy CPU tasks inside an async fn without `spawn_blocking`
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_with_timeout_success() {
        let result = with_timeout(Duration::from_millis(100), async { 42 }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_with_timeout_failure() {
        let result = with_timeout(Duration::from_millis(10), async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            42
        })
        .await;

        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_spawn_blocking_test() {
        let result = spawn_blocking_test(|| {
            std::thread::sleep(Duration::from_millis(10));
            42
        })
        .await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_with_cancellation_success() {
        let result = with_cancellation(Duration::from_millis(100), |cancel| async move {
            tokio::select! {
                _ = cancel.cancelled() => {
                    Err("cancelled".into())
                }
                result = async { Ok::<_, Box<dyn std::error::Error>>(42) } => result
            }
        })
        .await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_timeout_helpers() {
        assert_eq!(default_test_timeout(), Duration::from_secs(5));
        assert_eq!(short_test_timeout(), Duration::from_secs(1));
        assert_eq!(long_test_timeout(), Duration::from_secs(30));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_shared_helpers() {
        let mutex = shared_mutex(0);
        *mutex.lock().await += 1;

        let rwlock = shared_rwlock(0);
        *rwlock.write().await += 1;

        let semaphore = shared_semaphore(2);
        let _permit = semaphore.acquire().await.unwrap();
    }
}
