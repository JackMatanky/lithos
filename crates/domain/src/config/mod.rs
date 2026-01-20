//! Configuration bounded context.
//!
//! This module contains configuration domain entities, business logic, and
//! events.

pub(crate) mod aggregate;
pub(crate) mod events;
pub(crate) mod global;
pub(crate) mod types;
pub(crate) mod vault;

// Re-export main types for internal use and lib.rs re-exports
