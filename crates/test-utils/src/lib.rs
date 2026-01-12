//! # Lithos Test Utilities
//!
//! This crate provides standardized testing utilities and patterns for the Lithos project,
//! with a focus on async testing using Tokio.
//!
//! ## Async Testing Patterns
//!
//! This crate implements standardized patterns for testing Tokio-based async operations:
//! - Proper runtime configuration for consistent test behavior
//! - `spawn_blocking` utilities for CPU-intensive operations in tests
//! - Timeout helpers to prevent hanging tests
//! - `CancellationToken` patterns for graceful test shutdown
//! - Race condition detection and prevention utilities
//!
//! ## Usage
//!
//! Add this crate to your test dependencies and use the provided macros and utilities:
//!
//! ```rust,ignore
//! use lithos_test_utils::async_test;
//! use tokio::time::Duration;
//!
//! async_test!(async fn my_async_test() {
//!     // Your async test code here
//!     tokio::time::sleep(Duration::from_millis(100)).await;
//! });
//! ```

pub mod async_helpers;

pub use async_helpers::{
    default_test_timeout,
    long_test_timeout,
    shared_mutex,
    shared_rwlock,
    shared_semaphore,
    short_test_timeout,
    spawn_blocking_test,
    with_cancellation,
    with_timeout,
};

/// Re-export commonly used tokio testing types
pub use tokio_test::{assert_pending, assert_ready, assert_ready_err, assert_ready_ok};

// The async_test macro is automatically exported at crate root via #[macro_export]
// in the async_helpers module. Use it as:
// use lithos_test_utils::async_test;
