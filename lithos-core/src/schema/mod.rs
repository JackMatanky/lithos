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

/// Zstd compression utilities for raw file storage.
pub mod compression;
/// Schema errors.
pub mod error;
/// Schema domain events.
pub mod events;
/// Blake3 hash types for content addressing.
pub mod hash;
/// Raw file storage types for versioned schema files.
pub mod raw_file;

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
/// Fixed-size ring buffer for versioned storage.
pub mod ring_buffer;

/// Schema resolution service.
///
/// **Benchmark access**: This module is `#[doc(hidden)] pub` to allow
/// benchmarks to measure individual pipeline stages while hiding from public
/// documentation.
#[doc(hidden)]
pub mod resolver;

pub(crate) mod db_table {
    use redb::{MultimapTableDefinition, TableDefinition};

    pub(crate) const SCHEMA_BY_ID: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_by_id");
    pub(crate) const SCHEMA_ID_BY_NAME: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_id_by_name");
    pub(crate) const SCHEMA_METADATA: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_metadata");
    pub(crate) const BANK_METADATA: TableDefinition<&str, &[u8]> =
        TableDefinition::new("bank_metadata");
    pub(crate) const BANK_PROPERTY_BY_ID: TableDefinition<&str, &[u8]> =
        TableDefinition::new("bank_property_by_id");
    pub(crate) const BANK_PROPERTY_BY_NAME: TableDefinition<&str, &[u8]> =
        TableDefinition::new("bank_property_by_name");
    pub(crate) const PROPERTY_BANK_KEY: &str = "singleton";

    // Raw file storage tables
    /// Raw schema files (key: `file_path`, value: rkyv-serialized
    /// `RawSchemaFile`).
    pub(crate) const RAW_SCHEMA_FILES: TableDefinition<&str, &[u8]> =
        TableDefinition::new("raw_schema_files");

    /// Raw property bank file (singleton: key = `"property-bank"`).
    pub(crate) const RAW_PROPERTY_BANK_FILE: TableDefinition<&str, &[u8]> =
        TableDefinition::new("raw_property_bank_file");

    /// Key for raw property bank singleton.
    pub(crate) const RAW_PROPERTY_BANK_KEY: &str = "property-bank";

    // Inheritance tracking tables
    /// Multimap: parent `SchemaId` → multiple child schema records.
    /// Enables O(1) cascade staleness queries: "find all children of parent P".
    pub(crate) const SCHEMA_CHILDREN: MultimapTableDefinition<&str, &[u8]> =
        MultimapTableDefinition::new("schema_children");

    /// Regular table: child `SchemaId` → parent schema reference.
    /// Enables O(1) updates (know old parent to remove from multimap).
    /// Also tracks all schemas including roots (`parent_id` = None).
    pub(crate) const SCHEMA_PARENT: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_parent");
}

// --- Public API ---

/// Generic command type alias to remove path stuttering: `schema::Command` vs
/// `schema::command::Command`.
pub type Command<C> = command::Command<C>;

/// Generic query type alias to remove path stuttering: `schema::Query` vs
/// `schema::query::Query`.
pub type Query<Q> = query::Query<Q>;
