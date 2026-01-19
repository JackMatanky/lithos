//! Configuration bounded context.
//!
//! This module contains configuration domain entities, business logic, and events.

/// Configuration aggregate types and business logic.
pub(crate) mod aggregate;
/// Configuration domain events.
pub(crate) mod events;
/// Global configuration types.
pub(crate) mod global;
pub(crate) mod types;
/// Vault configuration types.
pub(crate) mod vault;

// Re-export main types for internal use and lib.rs re-exports
pub use aggregate::Config;
