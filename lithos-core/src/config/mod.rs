//! Configuration bounded context.
//!
//! This module contains configuration domain entities, business logic, and
//! events.

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

// ============================================================================
// Core Aggregate Modules
// ============================================================================

/// Configuration aggregate root.
pub mod aggregate;
/// Global configuration types and validation.
pub mod global;
/// Path configuration types.
pub mod paths;
/// Vault-scoped configuration types.
pub mod vault;

// ============================================================================
// Logic & Infrastructure Modules
// ============================================================================

/// Configuration command implementations (CQRS write operations).
pub mod command;
/// Figment ingestion boundary for raw config.
pub mod ingest;
/// Configuration ports for CQRS.
pub mod ports;
/// Configuration query implementations (CQRS read operations).
pub mod query;

// ============================================================================
// Supporting Domain Modules
// ============================================================================

/// Configuration error types.
pub mod error;
/// Configuration domain events.
pub mod events;
/// Frontmatter configuration types.
pub mod frontmatter;
/// Logging configuration types.
pub mod logging;
/// Raw (serde) configuration input types.
pub mod raw;
/// Task configuration schema and validation.
pub mod task;

// ============================================================================
// Concrete Implementation Aliases (Redb)
// ============================================================================

use crate::db::{
    Database,
    config_adapter::{CommandAdapter, QueryAdapter},
};

/// Redb-backed config command alias.
pub type RedbConfigCommand<'db> = command::Command<CommandAdapter<'db>>;

/// Redb-backed config query alias.
pub type RedbConfigQuery<'db> = query::Query<QueryAdapter<'db>>;

impl<'db> RedbConfigCommand<'db> {
    #[inline]
    #[must_use]
    /// Create a redb-backed config command.
    pub fn new_redb(db: &'db Database) -> Self {
        Self::new(CommandAdapter::new(db))
    }
}

impl<'db> RedbConfigQuery<'db> {
    #[inline]
    #[must_use]
    /// Create a redb-backed config query.
    pub fn new_redb(db: &'db Database) -> Self {
        Self::new(QueryAdapter::new(db))
    }
}
