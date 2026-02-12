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
/// Schema inheritance graph logic.
pub mod graph;
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

    pub(crate) const SCHEMAS: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schemas");
}

// --- Public API ---
