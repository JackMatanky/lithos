//! Schema bounded context domain models.
//!
//! This module contains all entities and services for the schema system,
//! organized into focused modules for better maintainability.

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

/// Schema aggregate roots and main entities.
pub mod aggregate;
/// Schema command implementations (CQRS write operations).
pub mod command;
/// Schema errors.
pub mod error;
/// Schema domain events.
pub mod events;
/// Schema ports for CQRS.
pub mod ports;
/// Property domain entities.
pub mod property;
/// Property specification variants.
pub mod property_spec;
/// Schema query implementations (CQRS read operations).
pub mod query;
/// Raw schema input definitions.
pub mod raw;
/// Schema resolution service.
pub mod resolver;

pub(crate) mod db_table {
    use redb::TableDefinition;

    pub(crate) const SCHEMA_BY_ID: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_by_id");
    pub(crate) const SCHEMA_ID_BY_NAME: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_id_by_name");
}

// --- Public API ---

use crate::db::{
    Database,
    schema_adapter::{CommandAdapter, QueryAdapter},
};

/// Redb-backed schema command alias.
pub type RedbSchemaCommand<'db> = command::Command<CommandAdapter<'db>>;

/// Redb-backed schema query alias.
pub type RedbSchemaQuery<'db> = query::Query<QueryAdapter<'db>>;

impl<'db> RedbSchemaCommand<'db> {
    #[inline]
    #[must_use]
    /// Create a redb-backed schema command.
    pub fn new_redb(db: &'db Database) -> Self {
        Self::new(CommandAdapter::new(db))
    }
}

impl<'db> RedbSchemaQuery<'db> {
    #[inline]
    #[must_use]
    /// Create a redb-backed schema query.
    pub fn new_redb(db: &'db Database) -> Self {
        Self::new(QueryAdapter::new(db))
    }
}
