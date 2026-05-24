//! Testing infrastructure primitives for in-memory repository adapters.
//!
//! This module provides **infrastructure-only primitives** for building
//! context-local in-memory repository adapters. It does NOT contain domain
//! logic or projection semantics—those remain in each context (Schema, Note,
//! Template, Config).
//!
//! # Architecture Boundaries
//!
//! - **DB owns**: Lock helpers, operation counters, failure injectors, error
//!   types for in-memory testing infrastructure.
//! - **Contexts own**: Domain projection logic, entity-specific indices,
//!   business invariants, in-memory repository adapter implementations.
//!
//! # Explicit Non-Goals
//!
//! - Do NOT add a generic `db::InMemoryRepository` domain adapter.
//! - Do NOT add shared domain index maps in `db`.
//! - Do NOT move context-specific invariants into DB.
//!
//! These non-goals protect DB context locality and prevent creating a shallow
//! cross-context fake storage module.

#![cfg(test)]

use std::sync::{
    Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
    atomic::{AtomicUsize, Ordering},
};

// ============================================================================
// Error Types
// ============================================================================

/// Error type for in-memory testing infrastructure failures.
///
/// This error type is used by `db::testing` primitives and can be converted
/// into context-specific error types via `From<InMemoryDbError>`.
#[derive(Debug, thiserror::Error)]
pub(crate) enum InMemoryDbError {
    /// Lock was poisoned (another thread panicked while holding the lock).
    #[error("Lock poisoned: {context}")]
    LockPoisoned {
        /// Context describing which operation attempted to acquire the lock.
        context: &'static str,
    },

    /// Failure was injected for testing purposes.
    #[error("Injected failure at {point:?}: {reason}")]
    InjectedFailure {
        /// The failure point where injection occurred.
        point: FailurePoint,
        /// Human-readable reason for the failure.
        reason: Box<str>,
    },

    /// Test invariant was violated.
    #[error("Invariant violation: {message}")]
    #[expect(dead_code, reason = "Reserved for future test scenarios")]
    InvariantViolation {
        /// Description of the invariant that was violated.
        message: Box<str>,
    },
}

// ============================================================================
// Test Harness (Primary API)
// ============================================================================

/// Test harness that holds operation counters and an optional failure injector.
///
/// Contexts embed this in their in-memory repository adapter state to gain
/// shared instrumentation and failure injection capabilities.
///
/// # Usage
///
/// Embed `InMemoryHarness` in your in-memory repository adapter:
///
/// ```rust,no_run
/// use std::sync::RwLock;
///
/// use lithos_core::db::testing::{InMemoryHarness, read_lock, write_lock};
///
/// struct InMemoryRepository {
///     data: RwLock<Vec<String>>,
///     harness: InMemoryHarness,
/// }
///
/// impl InMemoryRepository {
///     fn read_all(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
///         let guard = read_lock(&self.data, "read_all")?;
///         Ok(guard.clone())
///     }
/// }
/// ```
#[derive(Default)]
pub(crate) struct InMemoryHarness {
    counters: OpCounters,
    injector: Option<Box<dyn FailureInjector + Send + Sync>>,
}

impl std::fmt::Debug for InMemoryHarness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryHarness")
            .field("counters", &self.counters)
            .field("has_injector", &self.injector.is_some())
            .finish()
    }
}

impl InMemoryHarness {
    /// Creates a new harness with no failure injector.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Creates a new harness with the specified failure injector.
    pub(crate) fn with_injector(
        injector: Box<dyn FailureInjector + Send + Sync>,
    ) -> Self {
        Self {
            counters: OpCounters::default(),
            injector: Some(injector),
        }
    }

    /// Attempts to inject a failure at the specified point.
    ///
    /// If an injector is configured, delegates to it. Otherwise returns
    /// `Ok(())`.
    pub(crate) fn fail_at(
        &self,
        point: FailurePoint,
    ) -> Result<(), InMemoryDbError> {
        self.injector.as_ref().map_or(Ok(()), |inj| inj.fail_at(point))
    }

    /// Returns a reference to the operation counters.
    pub(crate) fn counters(&self) -> &OpCounters {
        &self.counters
    }
}

// ============================================================================
// Operation Instrumentation
// ============================================================================

/// Operation counters for tracking in-memory repository activity.
///
/// Uses `AtomicUsize` for lock-free, thread-safe counting. Suitable for
/// instrumenting test operations without introducing contention.
#[derive(Debug, Default)]
pub(crate) struct OpCounters {
    reads: AtomicUsize,
    writes: AtomicUsize,
    batches: AtomicUsize,
    deletes: AtomicUsize,
    injected_failures: AtomicUsize,
}

impl OpCounters {
    /// Increments the read operation counter.
    pub(crate) fn inc_read(&self) {
        self.reads.fetch_add(1, Ordering::Relaxed);
    }

    /// Increments the write operation counter.
    pub(crate) fn inc_write(&self) {
        self.writes.fetch_add(1, Ordering::Relaxed);
    }

    /// Increments the batch operation counter.
    pub(crate) fn inc_batch(&self) {
        self.batches.fetch_add(1, Ordering::Relaxed);
    }

    /// Increments the delete operation counter.
    #[expect(dead_code, reason = "Reserved for future test scenarios")]
    pub(crate) fn inc_delete(&self) {
        self.deletes.fetch_add(1, Ordering::Relaxed);
    }

    /// Increments the injected failure counter.
    #[expect(dead_code, reason = "Reserved for future test scenarios")]
    pub(crate) fn inc_injected_failure(&self) {
        self.injected_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// Takes a snapshot of all counters for assertions.
    ///
    /// Returns a `OpCountersSnapshot` with plain `usize` values.
    pub(crate) fn snapshot(&self) -> OpCountersSnapshot {
        OpCountersSnapshot {
            reads: self.reads.load(Ordering::Relaxed),
            writes: self.writes.load(Ordering::Relaxed),
            batches: self.batches.load(Ordering::Relaxed),
            deletes: self.deletes.load(Ordering::Relaxed),
            injected_failures: self.injected_failures.load(Ordering::Relaxed),
        }
    }

    /// Resets all counters to zero.
    ///
    /// Useful for test isolation when reusing counter instances across
    /// test cases.
    pub(crate) fn reset(&self) {
        self.reads.store(0, Ordering::Relaxed);
        self.writes.store(0, Ordering::Relaxed);
        self.batches.store(0, Ordering::Relaxed);
        self.deletes.store(0, Ordering::Relaxed);
        self.injected_failures.store(0, Ordering::Relaxed);
    }
}

/// Snapshot of operation counters at a point in time.
///
/// Used for assertions in tests. All fields are plain `usize` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OpCountersSnapshot {
    /// Number of read operations.
    pub(crate) reads: usize,
    /// Number of write operations.
    pub(crate) writes: usize,
    /// Number of batch operations.
    pub(crate) batches: usize,
    /// Number of delete operations.
    pub(crate) deletes: usize,
    /// Number of injected failures.
    pub(crate) injected_failures: usize,
}

// ============================================================================
// Failure Injection
// ============================================================================

/// Points in execution where failures can be injected for testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FailurePoint {
    /// Before a read operation.
    BeforeRead,
    /// Before a write operation.
    BeforeWrite,
    /// After serialization but before commit.
    #[expect(dead_code, reason = "Reserved for future test scenarios")]
    AfterSerialize,
    /// Before committing a transaction.
    #[expect(dead_code, reason = "Reserved for future test scenarios")]
    BeforeCommit,
}

/// Trait for injecting failures at specific points in test execution.
///
/// Implement this trait to create custom failure injection strategies for
/// testing error handling and rollback logic.
pub(crate) trait FailureInjector {
    /// Attempts to inject a failure at the specified point.
    ///
    /// Returns `Ok(())` if no failure should be injected, or
    /// `Err(InMemoryDbError::InjectedFailure)` to simulate a failure.
    fn fail_at(&self, point: FailurePoint) -> Result<(), InMemoryDbError>;
}

// ============================================================================
// Lock Helpers
// ============================================================================

/// Acquires a read lock on an `RwLock`, mapping poison errors to
/// `InMemoryDbError`.
///
/// # Errors
///
/// Returns `InMemoryDbError::LockPoisoned` if the lock is poisoned.
///
/// # Examples
///
/// ```rust,no_run
/// use std::sync::RwLock;
///
/// use lithos_core::db::testing::read_lock;
///
/// let data = RwLock::new(42);
/// let guard = read_lock(&data, "my_operation").unwrap();
/// assert_eq!(*guard, 42);
/// ```
pub(crate) fn read_lock<'a, T>(
    lock: &'a RwLock<T>,
    ctx: &'static str,
) -> Result<RwLockReadGuard<'a, T>, InMemoryDbError> {
    lock.read().map_err(|_| InMemoryDbError::LockPoisoned {
        context: ctx,
    })
}

/// Acquires a write lock on an `RwLock`, mapping poison errors to
/// `InMemoryDbError`.
///
/// # Errors
///
/// Returns `InMemoryDbError::LockPoisoned` if the lock is poisoned.
pub(crate) fn write_lock<'a, T>(
    lock: &'a RwLock<T>,
    ctx: &'static str,
) -> Result<RwLockWriteGuard<'a, T>, InMemoryDbError> {
    lock.write().map_err(|_| InMemoryDbError::LockPoisoned {
        context: ctx,
    })
}

/// Acquires a lock on a `Mutex`, mapping poison errors to `InMemoryDbError`.
///
/// # Errors
///
/// Returns `InMemoryDbError::LockPoisoned` if the lock is poisoned.
pub(crate) fn mutex_lock<'a, T>(
    lock: &'a Mutex<T>,
    ctx: &'static str,
) -> Result<MutexGuard<'a, T>, InMemoryDbError> {
    lock.lock().map_err(|_| InMemoryDbError::LockPoisoned {
        context: ctx,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread};

    use super::*;

    mod fixtures {
        use super::*;

        /// Poisons a lock by panicking while holding it.
        #[expect(
            clippy::disallowed_methods,
            reason = "catch_unwind required to poison lock for testing"
        )]
        #[expect(clippy::panic, reason = "Intentional panic to poison lock")]
        pub(crate) fn poison_lock<T>(lock: &RwLock<T>) {
            let _ =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let _guard = lock.write().unwrap();
                    panic!("poison");
                }));
        }

        /// Test injector that always fails.
        pub(crate) struct AlwaysFailInjector;

        impl FailureInjector for AlwaysFailInjector {
            fn fail_at(
                &self,
                point: FailurePoint,
            ) -> Result<(), InMemoryDbError> {
                Err(InMemoryDbError::InjectedFailure {
                    point,
                    reason: "test failure".into(),
                })
            }
        }

        /// Test injector that fails selectively based on failure point.
        pub(crate) struct SelectiveInjector {
            pub(crate) fail_on_write: bool,
        }

        impl FailureInjector for SelectiveInjector {
            fn fail_at(
                &self,
                point: FailurePoint,
            ) -> Result<(), InMemoryDbError> {
                match point {
                    FailurePoint::BeforeWrite if self.fail_on_write => {
                        Err(InMemoryDbError::InjectedFailure {
                            point,
                            reason: "write disabled".into(),
                        })
                    }
                    _ => Ok(()),
                }
            }
        }
    }

    mod locking {
        use super::*;

        #[test]
        fn returns_read_guard_when_not_poisoned() {
            // Arrange
            let data = RwLock::new(42);

            // Act
            let guard = read_lock(&data, "test").unwrap();

            // Assert
            assert_eq!(*guard, 42);
        }

        #[test]
        fn returns_error_when_read_lock_poisoned() {
            // Arrange
            let data = RwLock::new(0);
            fixtures::poison_lock(&data);

            // Act
            let result = read_lock(&data, "test");

            // Assert
            assert!(
                matches!(result, Err(InMemoryDbError::LockPoisoned { context }) if context == "test")
            );
        }

        #[test]
        fn returns_write_guard_when_not_poisoned() {
            // Arrange
            let data = RwLock::new(42);

            // Act
            let mut guard = write_lock(&data, "test").unwrap();

            // Assert
            *guard = 100;
            assert_eq!(*guard, 100);
        }

        #[test]
        fn returns_mutex_guard_when_not_poisoned() {
            // Arrange
            let data = Mutex::new(42);

            // Act
            let guard = mutex_lock(&data, "test").unwrap();

            // Assert
            assert_eq!(*guard, 42);
        }

        #[test]
        fn allows_concurrent_read_locks() {
            // Arrange
            let data = Arc::new(RwLock::new(42));

            // Act
            let guard1 = read_lock(&data, "reader1").unwrap();
            let guard2 = read_lock(&data, "reader2").unwrap();

            // Assert
            assert_eq!(*guard1, 42);
            assert_eq!(*guard2, 42);
        }

        #[test]
        fn captures_context_in_write_lock_error() {
            // Arrange
            let data = RwLock::new(0);
            fixtures::poison_lock(&data);

            // Act
            let result = write_lock(&data, "my_write_op");

            // Assert
            assert!(
                matches!(result, Err(InMemoryDbError::LockPoisoned { context }) if context == "my_write_op")
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "catch_unwind required to poison mutex for testing"
        )]
        #[expect(clippy::panic, reason = "Intentional panic to poison mutex")]
        fn captures_context_in_mutex_lock_error() {
            // Arrange
            let data = Mutex::new(0);
            // Poison the mutex
            let _ =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let _guard = data.lock().unwrap();
                    panic!("poison");
                }));

            // Act
            let result = mutex_lock(&data, "my_mutex_op");

            // Assert
            assert!(
                matches!(result, Err(InMemoryDbError::LockPoisoned { context }) if context == "my_mutex_op")
            );
        }
    }

    mod counters {
        use super::*;

        #[test]
        fn starts_with_zero() {
            // Arrange
            let counters = OpCounters::default();

            // Act
            let snapshot = counters.snapshot();

            // Assert
            assert_eq!(snapshot.reads, 0);
            assert_eq!(snapshot.writes, 0);
            assert_eq!(snapshot.batches, 0);
            assert_eq!(snapshot.deletes, 0);
            assert_eq!(snapshot.injected_failures, 0);
        }

        #[test]
        fn increments_read_count() {
            // Arrange
            let counters = OpCounters::default();

            // Act
            counters.inc_read();
            counters.inc_read();

            // Assert
            assert_eq!(counters.snapshot().reads, 2);
        }

        #[test]
        fn increments_write_and_batch_counts() {
            // Arrange
            let counters = OpCounters::default();

            // Act
            counters.inc_write();
            counters.inc_batch();

            // Assert
            let snapshot = counters.snapshot();
            assert_eq!(snapshot.writes, 1);
            assert_eq!(snapshot.batches, 1);
        }

        #[test]
        #[allow(clippy::excessive_nesting, reason = "Test concurrency setup")]
        fn increments_concurrently_from_multiple_threads() {
            // Arrange
            let counters = Arc::new(OpCounters::default());

            // Act
            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let c = Arc::clone(&counters);
                    thread::spawn(move || {
                        for _ in 0..100 {
                            c.inc_read();
                        }
                    })
                })
                .collect();

            for h in handles {
                h.join().unwrap();
            }

            // Assert
            assert_eq!(counters.snapshot().reads, 1000);
        }

        #[test]
        fn resets_all_counters_to_zero() {
            // Arrange
            let counters = OpCounters::default();
            counters.inc_read();
            counters.inc_write();
            counters.inc_batch();

            // Act
            counters.reset();

            // Assert
            let snapshot = counters.snapshot();
            assert_eq!(snapshot.reads, 0);
            assert_eq!(snapshot.writes, 0);
            assert_eq!(snapshot.batches, 0);
        }
    }

    mod failure_injection {
        use super::*;

        #[test]
        fn succeeds_when_no_injector_configured() {
            // Arrange
            let harness = InMemoryHarness::new();

            // Act
            let result = harness.fail_at(FailurePoint::BeforeWrite);

            // Assert
            assert!(result.is_ok());
        }

        #[test]
        fn injects_failure_when_injector_configured() {
            // Arrange
            let harness = InMemoryHarness::with_injector(Box::new(
                fixtures::AlwaysFailInjector,
            ));

            // Act
            let result = harness.fail_at(FailurePoint::BeforeWrite);

            // Assert
            assert!(matches!(
                result,
                Err(InMemoryDbError::InjectedFailure { .. })
            ));
        }

        #[test]
        fn injects_selectively_by_failure_point() {
            // Arrange
            let harness = InMemoryHarness::with_injector(Box::new(
                fixtures::SelectiveInjector {
                    fail_on_write: true,
                },
            ));

            // Act & Assert
            assert!(harness.fail_at(FailurePoint::BeforeRead).is_ok());
            assert!(harness.fail_at(FailurePoint::BeforeWrite).is_err());
        }
    }
}
