//! Schema identifier value objects.
//!
//! **Deprecated**: Use `schema::aggregate` instead. This module exists for
//! backwards compatibility during migration.
//!
//! Core identifier types for the schema domain: `SchemaId` and `SchemaName`.
//! These types have been moved to `schema::aggregate`.

#![allow(
    clippy::pub_use,
    reason = "Temporary compatibility layer during migration"
)]

// Re-export from aggregate module for backwards compatibility
pub use super::aggregate::{SchemaId, SchemaName};
