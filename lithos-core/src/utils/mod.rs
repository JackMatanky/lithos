//! Shared utility contracts exposed by `lithos-core`.
//!
//! This module provides domain-agnostic primitives with stable semantics that
//! can be consumed across bounded contexts without importing crate-internal
//! implementation details.

#![expect(
    clippy::pub_use,
    reason = "utils module re-exports stable contract types intentionally"
)]

pub mod error;

/// UUID v7 support primitive shared across contexts.
pub mod uuid;

pub use error::UuidV7Error;
pub use uuid::UuidV7;
