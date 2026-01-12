# Async Testing Guidelines

This document provides comprehensive guidelines and patterns for testing async code in Lithos using Tokio.

## Table of Contents

1. [Overview](#overview)
2. [Testing Patterns](#testing-patterns)
3. [Runtime Configuration](#runtime-configuration)
4. [Blocking Operations](#blocking-operations)
5. [Timeouts and Cancellation](#timeouts-and-cancellation)
6. [Race Condition Prevention](#race-condition-prevention)
7. [Test Isolation](#test-isolation)
8. [Error Handling](#error-handling)
9. [Best Practices](#best-practices)
10. [Examples](#examples)

## Overview

Lithos uses Tokio for async operations throughout the hexagonal architecture. Async testing must follow standardized patterns to ensure:

- **Reliability**: Tests are deterministic and don't flake
- **Safety**: Proper runtime behavior without blocking issues
- **Isolation**: Tests don't interfere with each other
- **Speed**: Tests run efficiently without unnecessary delays

## Testing Patterns

### Using `#[tokio::test]` Macro

All async tests must use `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]` (or the `async_test!` helper from `lithos-test-utils`):

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn my_async_function_test() {
    // Your test code here
    let result = my_async_function().await;
    assert_eq!(result, expected);
}
```

**Why multi_thread?**

The macro configures `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]` to:
- Surface race conditions that might not appear in single-threaded tests
- Test concurrent operations like event buses and shared state
- Ensure tests behave similarly to production runtime

### Testing Async Functions and Futures

When testing async functions, follow these patterns:

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_async_function() {
    // Given
    let input = 42;

    // When
    let result = my_async_function(input).await;

    // Then
    assert_eq!(result, expected_value);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_async_futures() {
    // Test Future types directly
    let future = my_async_future();

    // You can await multiple futures concurrently
    let (result1, result2) = tokio::join!(future, another_async_function());

    assert!(result1.is_ok());
}
```

## Runtime Configuration

### Tokio Test Runtime

The test runtime is configured via `lithos-test-utils`:

```rust
#[macro_export]
macro_rules! async_test {
    ($vis:vis async fn $name:ident() $body:block) => {
        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        $vis async fn $name() $body
    };
}
```

**Key Configuration Points:**

1. **flavor = "multi_thread"**: Uses multiple worker threads
2. **worker_threads = 2**: Limits concurrent threads to balance speed with race condition detection

### Adjusting Worker Threads

For specific tests that need different thread configurations:

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_highly_concurrent_operations() {
    // Test with 4 worker threads
}
```

## Blocking Operations

### The Blocking Problem

NEVER perform blocking operations in async tests without proper handling:

```rust
// ❌ BAD - Blocks the async runtime thread
#[tokio::test]
async fn test_blocking_file_read() {
    let content = std::fs::read_to_string("file.txt").unwrap(); // BLOCKING!
}
```

### Using `spawn_blocking_test`

Use the helper from `lithos-test-utils`:

```rust
use lithos_test_utils::spawn_blocking_test;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_non_blocking_file_read() {
    // ✅ GOOD - Offloads to blocking thread pool
    let content = spawn_blocking_test(|| {
        std::fs::read_to_string("file.txt").unwrap()
    }).await.unwrap();

    assert!(!content.is_empty());
}
```

### When to Use `spawn_blocking`

**Always use for:**
- `std::fs` operations (reading/writing files)
- Heavy CPU computations (e.g., parsing large documents)
- Redb write transactions
- Synchronous HTTP requests
- Database operations without async drivers

**Never use for:**
- `tokio::fs` operations (already async)
- `tokio::sync` primitives (designed for async)
- `tokio::net` operations (already async)

### Safety Invariant

According to Lithos project rules:
- NEVER block an async thread for >10ms
- Use `tokio::task::spawn_blocking` for all blocking operations
- This prevents runtime thread starvation and maintains responsiveness

## Timeouts and Cancellation

### Preventing Hanging Tests

Always wrap async operations with timeouts:

```rust
use lithos_test_utils::{with_timeout, default_test_timeout};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_with_timeout() {
    let result = with_timeout(default_test_timeout(), async {
        // Operation that should complete
        my_async_operation().await
    }).await;

    assert!(result.is_ok());
}
```

### Timeout Durations

Choose appropriate timeouts based on operation complexity:

```rust
use lithos_test_utils::{short_test_timeout, default_test_timeout, long_test_timeout};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_quick_operation() {
    // Use for simple calculations or checks
    with_timeout(short_test_timeout(), async {
        quick_operation().await
    }).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_normal_operation() {
    // Use for most operations (default: 5 seconds)
    with_timeout(default_test_timeout(), async {
        normal_operation().await
    }).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_heavy_operation() {
    // Use for indexing, parsing, heavy computations (30 seconds)
    with_timeout(long_test_timeout(), async {
        heavy_operation().await
    }).await.unwrap();
}
```

### Cancellation Testing

Test that operations respect shutdown signals:

```rust
use lithos_test_utils::with_cancellation;
use tokio::sync::broadcast;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_actor_shutdown() {
    let result = with_cancellation(default_test_timeout(), |cancel| async move {
        // Test actor shutdown behavior
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);

        tokio::select! {
            _ = cancel.cancelled() => {
                // Test was cancelled
                Ok("cancelled".to_string())
            }
            _ = shutdown_rx.recv() => {
                // Actor received shutdown signal
                Ok("graceful_shutdown".to_string())
            }
            result = async {
                // Normal operation
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                Ok("completed".to_string())
            } => result
        }
    }).await;

    assert!(result.is_ok());
}
```

## Race Condition Prevention

### Testing Concurrent Operations

Use multi-threaded tests to surface race conditions:

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent_counter() {
    let counter = Arc::new(AtomicUsize::new(0));

    // Spawn multiple tasks updating the same counter
    let mut tasks = Vec::new();
    for _ in 0..10 {
        let counter = counter.clone();
        tasks.push(tokio::spawn(async move {
            for _ in 0..100 {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }

    // Wait for all tasks
    for task in tasks {
        task.await.unwrap();
    }

    // Verify no race conditions occurred
    assert_eq!(counter.load(Ordering::SeqCst), 1000);
}
```

### Synchronization Primitives

Use appropriate synchronization primitives:

```rust
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, Semaphore};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mutex_protection() {
    let shared_data = Arc::new(Mutex::new(0));

    // Multiple tasks accessing shared data
    let mut handles = vec![];
    for _ in 0..10 {
        let data = shared_data.clone();
        handles.push(tokio::spawn(async move {
            let mut guard = data.lock().await;
            *guard += 1;
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(*shared_data.lock().await, 10);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_semaphore_throttling() {
    let semaphore = Arc::new(Semaphore::new(2)); // Max 2 concurrent operations

    let mut join_set = tokio::task::JoinSet::new();
    for i in 0..5 {
        let sem = semaphore.clone();
        join_set.spawn(async move {
            let _permit = sem.acquire().await;
            // Simulate operation
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            i
        });
    }

    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        results.push(result.unwrap());
    }

    assert_eq!(results.len(), 5);
}
```

### Lock Discipline

**CRITICAL**: NEVER hold a `std::sync::MutexGuard` across an `.await`:

```rust
// ❌ BAD - Deadlock risk
use std::sync::Mutex;

#[tokio::test]
async fn test_deadlock_bad() {
    let guard = std::sync::Mutex::new(1).lock().unwrap(); // Acquires lock
    tokio::time::sleep(Duration::from_millis(10)).await; // AWAIT WITH LOCK HELD!
    // Guard is held across await - may cause deadlock
}

// ✅ GOOD - Use tokio::sync::Mutex
use tokio::sync::Mutex;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_no_deadlock_good() {
    let mutex = Mutex::new(1);
    let guard = mutex.lock().await; // Acquires lock (async)
    tokio::time::sleep(Duration::from_millis(10)).await;
    // Guard can be held across await with tokio::sync::Mutex
}
```

## Test Isolation

### No Shared State Between Tests

Each test should be independent:

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_isolated_operation() {
    // Setup isolated for this test
    let test_data = create_test_data();

    // Test
    let result = operation(test_data).await;
    assert!(result.is_ok());

    // Cleanup (optional, will be dropped when test ends)
}
```

### Using Fixtures for Test Data

Create reusable test fixtures:

```rust
// In a fixtures module
pub async fn create_test_vault() -> TestVault {
    // Create isolated test vault
    TestVault::new().await
}

// In tests
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_with_fixture() {
    let vault = create_test_vault().await;
    let result = vault.process_note("test.md").await;
    assert!(result.is_ok());
}
```

## Error Handling

### Testing Error Cases

Async functions return `Result`, so test both success and failure paths:

```rust
use anyhow::Result;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_success_path() {
    let result = my_async_operation("valid_input").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_value);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_error_path() {
    let result = my_async_operation("invalid_input").await;
    assert!(result.is_err());

    // Optionally check specific error
    let err = result.unwrap_err();
    assert!(err.to_string().contains("expected error"));
}
```

### Using `miette` for Rich Errors

When testing error handling, check that errors include helpful information:

```rust
use miette::{Diagnostic, SourceSpan};
use std::fmt;

#[derive(Debug, Diagnostic)]
#[error("Test error")]
struct TestError {
    #[source_code]
    src: String,
    #[label("here")]
    span: SourceSpan,
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_error_with_diagnostic() {
    let result = operation_that_errors().await;

    assert!(result.is_err());
    let err = result.unwrap_err();

    // Verify error has diagnostic information
    assert!(err.to_string().contains("Test error"));
}
```

## Best Practices

### 1. Always Use Timeouts

```rust
use lithos_test_utils::with_timeout;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_operation() {
    // Always wrap with timeout
    with_timeout(Duration::from_secs(5), async {
        my_operation().await
    }).await.unwrap();
}
```

### 2. Never Block Async Threads

```rust
// ❌ BAD
#[tokio::test]
async fn test() {
    std::fs::read_to_string("file").unwrap(); // Blocks!
}

// ✅ GOOD
use lithos_test_utils::spawn_blocking_test;

#[tokio::test]
async fn test() {
    spawn_blocking_test(|| {
        std::fs::read_to_string("file").unwrap()
    }).await.unwrap();
}
```

### 3. Use Multi-Threaded Tests for Concurrency

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent() {
    // Uses multi_thread flavor by default
    // Surfaces race conditions
}
```

### 4. Test Cancellation Paths

```rust
use lithos_test_utils::with_cancellation;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_graceful_shutdown() {
    with_cancellation(Duration::from_secs(5), |cancel| async move {
        tokio::select! {
            _ = cancel.cancelled() => Ok("cancelled"),
            result = operation() => result,
        }
    }).await.unwrap();
}
```

### 5. Avoid Global State

Each test should create its own test data and resources:

```rust
// ❌ BAD - Global state
static GLOBAL_DATA: OnceCell<Data> = OnceCell::new();

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_with_global() {
    let data = GLOBAL_DATA.get_or_init(|| create_data());
    // Tests share state - bad!
}

// ✅ GOOD - Local state
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_with_local() {
    let data = create_data(); // Each test has its own
    // Tests are isolated
}
```

### 6. Test Edge Cases

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_empty_input() {
    let result = operation("").await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_large_input() {
    let large_input = vec![0; 1_000_000];
    let result = operation(&large_input).await;
    assert!(result.is_ok());
}
```

## Examples

### Complete Example: Async Repository Test

```rust
use lithos_test_utils::{with_timeout, spawn_blocking_test};
use std::sync::Arc;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_note_repository_crud() {
    with_timeout(Duration::from_secs(5), async {
        // Setup - create test repository
        let repo = Arc::new(TestNoteRepository::new().await);

        // Create
        let note = Note::new("test", "content");
        let created = repo.create(note.clone()).await.unwrap();
        assert_eq!(created.id(), note.id());

        // Read
        let found = repo.get(note.id()).await.unwrap();
        assert_eq!(found.title(), "test");

        // Update
        let updated = Note::new(note.id(), "updated content");
        repo.update(updated.clone()).await.unwrap();

        // Delete
        repo.delete(note.id()).await.unwrap();
        let result = repo.get(note.id()).await;
        assert!(result.is_err());

        Ok::<(), anyhow::Error>(())
    }).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_note_repository_with_blocking() {
    // Test with blocking operations (e.g., file I/O)
    let path = "/tmp/test-notes.db";
    let repo = spawn_blocking_test(move || {
        // Blocking initialization
        std::fs::File::create(path).unwrap();
        NoteRepository::new(path)
    }).await.unwrap();

    // Async operations continue
    repo.create(Note::new("test", "content")).await.unwrap();
}
```

### Complete Example: Event Bus Testing

```rust
use lithos_test_utils::with_timeout;
use tokio::sync::broadcast;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_event_bus_publish_subscribe() {
    with_timeout(Duration::from_secs(2), async {
        let (tx, mut rx) = broadcast::channel(16);

        // Spawn subscriber task
        let subscriber_task = tokio::spawn(async move {
            let mut received = Vec::new();
            while let Ok(event) = rx.recv().await {
                received.push(event);
                if received.len() >= 3 {
                    break;
                }
            }
            received
        });

        // Publish events
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();

        // Wait for subscriber
        let events = subscriber_task.await.unwrap();
        assert_eq!(events, vec![1, 2, 3]);

        Ok::<(), anyhow::Error>(())
    }).await.unwrap();
}
```

### Complete Example: Cancellation Testing

```rust
use lithos_test_utils::{with_cancellation, default_test_timeout};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_actor_graceful_shutdown() {
    let result = with_cancellation(default_test_timeout(), |cancel| async move {
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);

        // Spawn an actor task
        let actor_task = tokio::spawn(async move {
            let mut counter = 0;

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        // Gracefully shutdown - complete current work
                        counter += 1;
                        return Ok(counter);
                    }
                    _ = tokio::time::sleep(Duration::from_millis(10)) => {
                        // Normal operation
                        counter += 1;
                        if counter > 10 {
                            return Ok(counter);
                        }
                    }
                }
            }
        });

        // Test: Cancel after a short time
        tokio::time::sleep(Duration::from_millis(50)).await;
        cancel.cancel();

        let result = actor_task.await.unwrap()?;
        assert!(result > 0);

        Ok::<_, anyhow::Error>(())
    }).await;

    assert!(result.is_ok());
}
```

## Summary

### Key Takeaways

1. **Always use `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`** macro for proper runtime configuration
2. **Never block async threads** - use `spawn_blocking_test` for blocking operations
3. **Wrap with timeouts** to prevent hanging tests
4. **Test cancellation paths** to ensure graceful shutdown
5. **Use multi-threaded tests** to surface race conditions
6. **Maintain test isolation** - no shared state between tests
7. **Follow lock discipline** - use `tokio::sync::Mutex`, never `std::sync::Mutex` across awaits

### Common Pitfalls to Avoid

- ❌ Blocking operations in async context without `spawn_blocking`
- ❌ Holding `std::sync::MutexGuard` across `.await`
- ❌ Tests that can hang indefinitely (no timeouts)
- ❌ Single-threaded tests for concurrent code
- ❌ Global state between tests
- ❌ Not testing error paths
- ❌ Not testing cancellation/shutdown paths

### Resources

- [Tokio Testing Documentation](https://tokio.rs/tokio/topics/testing)
- [Async/Await Book](https://rust-lang.github.io/async-book/)
- [Lithos Project Context](../../project-context.md)
