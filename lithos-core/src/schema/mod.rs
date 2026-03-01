//! Schema bounded context domain models.
//!
//! This module contains all entities and services for the schema system,
//! organized into focused modules for better maintainability.

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

/// Schema storage adapters.
pub mod adapter;
/// Schema aggregate roots and main entities.
pub mod aggregate;
/// PropertyBank domain aggregate for centralized property registration.
pub mod bank;
/// Schema command implementations (CQRS write operations).
pub mod command;
/// Property-bank dereferencer pipeline stage.
///
/// **Benchmark access**: This module is `#[doc(hidden)] pub` to allow
/// benchmarks to measure individual pipeline stages while hiding from public
/// documentation.
#[doc(hidden)]
pub mod dereferencer;

/// Schema errors.
pub mod error;
/// Schema domain events.
pub mod events;

/// Schema inheritance-tree builder pipeline stage.
///
/// **Benchmark access**: This module is `#[doc(hidden)] pub` to allow
/// benchmarks to measure individual pipeline stages while hiding from public
/// documentation.
#[doc(hidden)]
pub mod extender;

/// User-facing string format specifications.
pub mod formats;
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
///
/// **Benchmark access**: This module is `#[doc(hidden)] pub` to allow
/// benchmarks to measure individual pipeline stages while hiding from public
/// documentation.
#[doc(hidden)]
pub mod resolver;

pub(crate) mod db_table {
    use redb::TableDefinition;

    pub(crate) const SCHEMA_BY_ID: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_by_id");
    pub(crate) const SCHEMA_ID_BY_NAME: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_id_by_name");
    pub(crate) const PROPERTY_BANK: TableDefinition<&str, &[u8]> =
        TableDefinition::new("property_bank");
    pub(crate) const PROPERTY_BANK_KEY: &str = "singleton";
}

// --- Public API ---

use self::adapter::{command::CommandAdapter, query::QueryAdapter};

/// Redb-backed schema command alias.
pub type RedbSchemaCommand<'db> = command::Command<CommandAdapter<'db>>;

/// Redb-backed schema query alias.
pub type RedbSchemaQuery<'db> = query::Query<QueryAdapter<'db>>;
