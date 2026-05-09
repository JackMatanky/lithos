//! Shared support utilities and infrastructure primitives.
//!
//! This module contains cross-cutting concerns like hashing and
//! serialization helpers that are used across multiple bounded contexts.

#![expect(
    clippy::pub_use,
    reason = "support module re-exports stable primitive types intentionally"
)]

/// Error types for support primitives.
pub mod error;

/// BLAKE3 hashing utilities and types.
pub mod hash;

/// UUID v7 support primitive shared across contexts.
pub mod uuid;

pub use error::UuidV7Error;
pub use uuid::UuidV7;
