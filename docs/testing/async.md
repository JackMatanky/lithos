# Async Testing Guidelines

Tactical specification for testing asynchronous code in Lithos using Tokio.

## 1. Key Principles

### Determinism First
Async tests must be deterministic. Flakiness is unacceptable in a CI environment.
- **Avoid Wall-Clock**: Never use `std::thread::sleep`. It relies on the OS scheduler and varies by machine load (e.g., CI runners are slower than dev machines).
- **Virtual Time**: Use Tokio's test runtime to pause and advance time programmatically. This allows testing a "1-hour timeout" in milliseconds with 100% reliability.
- **Fixed Seeds**: Any randomness (UUIDs, jitter, exponential backoff) must be seeded with a fixed value during testing.

### Safety Invariants
The Tokio runtime behaves differently than standard threads. Violating its invariants leads to difficult-to-debug hangs.
- **No Blocking**: Blocking the async runtime thread for >10ms (Rule 103) starves other tasks on the same worker thread. Use `spawn_blocking_test` for `std::fs`, heavy JSON parsing, `bcrypt`, or large `Redb` write transactions.
- **Lock Discipline**: NEVER hold a `std::sync::MutexGuard` across an `.await` point. This causes deadlocks because the task yields to the executor while holding the lock, preventing other tasks from acquiring it. Use `tokio::sync::Mutex` instead.
- **Resource Bounds**: Unbounded concurrency (e.g., `join_all` on a list of 10k items) can exhaust file descriptors or memory. Use `Semaphore` or `JoinSet` to bound concurrency.

### Isolation
- **Fresh Runtime**: Each test gets its own Tokio runtime instance created by the `#[tokio::test]` macro.
- **Fresh Filesystem**: Use `tempfile::TempDir` to get a unique, randomized temporary directory per test.
- **No Global State**: Avoid `static` mutable state. Use dependency injection for shared resources (e.g., pass `Arc<Bus>` rather than accessing a `lazy_static` Bus).

## 2. Golden Rules (The Invariants)

1.  **Strict Flavor**: All tests must use `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`. Single-threaded tests mask race conditions by executing tasks sequentially. Multi-threaded tests force actual parallelism.
2.  **Mandatory Timeouts**: All async operations must be wrapped in `tokio::time::timeout` or explicit cancellation. Hanging CI pipelines are a major productivity killer.
3.  **Cancellation Paths**: Every long-running process (Actor/Service) must be tested for graceful shutdown via cancellation token or channel close.
4.  **Error Paths**: Async results must be verified for both success (`Ok`) and failure (`Err`) conditions. Panics in spawned tasks must be caught and reported, not silently ignored.

## 3. Implementation Reference

### Runtime Configuration
The `multi_thread` flavor with 2 workers is the project standard. It balances speed with the ability to detect common async bugs like sending non-`Send` types across threads or deadlocking shared resources.

```rust
// Project standard: multi_thread runtime with 2 workers
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn my_concurrent_test() {
    // Test logic here
}

// Manual (if specific config needed, e.g., simulating high contention)
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn heavy_concurrency_test() {
    // ...
}
```

### Handling Blocking Operations
If you must use `std::fs` (e.g., legacy code, initial setup, or Redb internals), offload it.

```rust
use tokio::task::spawn_blocking;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_non_blocking_file_read() {
    // spawn_blocking offloads to a dedicated blocking thread pool
    let content = spawn_blocking(|| {
        // This closure runs on a thread where blocking is safe
        std::fs::read_to_string("vault_config.toml")
    })
    .await
    .expect("Task failed/panicked")
    .expect("File read failed");

    assert!(!content.is_empty());
}
```

### Deterministic Time Control
Eliminate flakiness by pausing the global virtual clock.

```rust
use tokio::time::{advance, pause, Duration};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn validates_cache_expiry() {
    pause();

    // 1. Set a key with 60s TTL
    // The creation timestamp is captured relative to the frozen clock
    cache.set("key", "val", Duration::from_secs(60)).await;

    // 2. Advance 30s (should still exist)
    // This jumps forward instantly. No sleeping.
    advance(Duration::from_secs(30)).await;
    assert!(cache.get("key").await.is_some());

    // 3. Advance another 31s (total 61s - expired)
    advance(Duration::from_secs(31)).await;
    assert!(cache.get("key").await.is_none());
}
```

### Resource Throttling & JoinSets
When testing indexing or batch operations, use `tokio::task::JoinSet` and `Semaphore` to prevent overwhelming the system.

```rust
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

#[tokio::test]
async fn test_batch_processing_under_load() {
    let semaphore = Arc::new(Semaphore::new(10)); // Max 10 concurrent tasks
    let mut join_set = JoinSet::new();

    for i in 0..100 {
        // Acquire permit BEFORE spawning. This exerts backpressure.
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        join_set.spawn(async move {
            // Permit is moved into the task and dropped when task completes
            let _permit = permit;
            process_vault_file(i).await
        });
    }

    // Drain the set and check for failures
    while let Some(res) = join_set.join_next().await {
        match res {
            Ok(app_result) => assert!(app_result.is_ok()),
            Err(join_err) => panic!("Task panicked: {:?}", join_err),
        }
    }
}
```

### Graceful Shutdown
Actors must respect cancellation signals. Verify this using `tokio::select!`.

```rust
use tokio::sync::broadcast;

#[tokio::test]
async fn test_actor_shutdown() {
    let (tx, _) = broadcast::channel(1);
    let mut rx = tx.subscribe();

    let actor_handle = tokio::spawn(async move {
        // Simulating an actor loop
        loop {
            tokio::select! {
                // Priority 1: Shutdown signal
                _ = rx.recv() => {
                    return Ok::<_, anyhow::Error>("shutdown_cleanly");
                }
                // Priority 2: Work (mocked with sleep)
                _ = tokio::time::sleep(Duration::from_secs(10)) => {
                    // This branch should NOT be taken if cancellation works
                    return Err(anyhow::anyhow!("Timed out waiting for shutdown"));
                }
            }
        }
    });

    // Simulate shutdown signal
    tx.send(()).unwrap();

    let result = actor_handle.await.unwrap().unwrap();
    assert_eq!(result, "shutdown_cleanly");
}
```

## 4. Advanced Scenarios & Debugging

### Testing Async Streams
When testing `Stream` implementations (e.g., from `tokio-stream` or `futures`), use `StreamExt` traits.

```rust
use futures::StreamExt;

#[tokio::test]
async fn test_stream_yields_values() {
    let mut stream = my_async_stream();

    // Assert the first value
    let val1 = stream.next().await.expect("Stream ended prematurely");
    assert_eq!(val1, 1);

    // Assert the stream ends
    assert!(stream.next().await.is_none());
}
```

### Race Condition Detection
Race conditions often only appear under load or specific thread scheduling.
- **Pattern**: Run the same logic in a loop within the test to increase probability of collision.
- **Pattern**: Use `tokio::sync::Barrier` to align starting points of concurrent tasks.

```rust
let barrier = Arc::new(Barrier::new(2));
let c = barrier.clone();

let task_a = tokio::spawn(async move {
    c.wait().await; // Wait for both tasks to be ready
    // Perform Action A
});

barrier.wait().await; // Release the barrier
// Perform Action B immediately concurrent with Action A
```

### Debugging Hung Tests
If `mise run test` hangs, it usually means a task is:
1.  Waiting on a channel that will never receive a message.
2.  Waiting on a lock (`Mutex`) that is held by a suspended task (Deadlock).
3.  Running an infinite loop without `.await` points (CPU starvation).

**Mitigation**:
- Enable `console-subscriber` in the test environment when debugging async scheduling.
- Use `#[tokio::test(flavor = "multi_thread")]` to prevent single-thread starvation.
- Ensure strict timeouts (`with_timeout`) are applied.

### Async Traits and Mockall
When mocking `#[async_trait]`, ensure the mock is `Send + Sync`.
- `mockall::automock` handles this automatically if configured correctly.
- Ensure return types are `BoxFuture` compliant if manually mocking.

### Error Handling
Async tests often fail with `JoinError` (panic in task) vs. `AppError` (logic failure).
- Differentiate between the two:
  - `JoinError`: The code crashed/panicked. This is usually a bug in the code or test setup.
  - `AppError`: The code handled an error condition. This is what you assert against.

```rust
let handle = tokio::spawn(async { panic!("oops") });
let err = handle.await.unwrap_err();
assert!(err.is_panic()); // Verify it was a panic
```

## 5. Anti-Patterns (Do Not Do This)

### ❌ The "Sleep and Pray"
```rust
// BAD: Flaky and slow
tokio::spawn(async_work());
tokio::time::sleep(Duration::from_millis(50)).await;
assert!(check_work_done());
```
**Fix**: Use channels or `Notify` to signal completion deterministically.

### ❌ The "Blocking Mutex"
```rust
// BAD: Deadlock risk
let lock = std::sync::Mutex::new(0);
let guard = lock.lock().unwrap();
some_async_fn().await; // Executor switches task, but lock is still held!
```
**Fix**: Use `tokio::sync::Mutex` if you must hold it across await, or drop the guard before awaiting.

### ❌ The "Unawaited Future"
```rust
// BAD: Does nothing
my_async_fn(); // Returns a Future that is dropped immediately
```
**Fix**: Always `.await` or `tokio::spawn` the future. `#[must_use]` on Futures helps catch this.

---
*For high-level guides, see [docs/test_guide.md](../test_guide.md)*
