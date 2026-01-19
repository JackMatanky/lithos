//! Configuration bounded context.
//!
//! This module contains configuration domain entities, business logic, and events.

/// Configuration aggregate types and business logic.
pub mod aggregate;
/// Configuration domain events.
pub mod events;
/// Global configuration types.
pub mod global;
pub mod types;
/// Vault configuration types.
pub mod vault;

// Re-export main types for convenience
pub use aggregate::Config;
pub use types::*;
