//! Schema bounded context domain models.
//!
//! This module contains all entities and services for the schema system,
//! organized into focused modules for better maintainability.

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

/// Schema identifier value objects (SchemaId, SchemaName).
pub mod id;

/// PropertyBank domain aggregate for centralized property registration.
pub mod bank;
/// Property-bank dereferencer pipeline stage.
///
/// **Benchmark access**: This module is `#[doc(hidden)] pub` to allow
/// benchmarks to measure individual pipeline stages while hiding from public
/// documentation.
#[doc(hidden)]
pub mod dereferencer;

/// Zstd compression utilities for raw file storage.
pub mod compression;
/// Database command adapter for schema CQRS write operations.
///
/// **Benchmark/Test access**: This module is `#[doc(hidden)] pub` to allow
/// benchmarks and tests to access the command adapter while hiding from public
/// documentation.
#[doc(hidden)]
pub mod db_command;
/// Database query adapter for schema CQRS read operations.
///
/// **Benchmark/Test access**: This module is `#[doc(hidden)] pub` to allow
/// benchmarks and tests to access the query adapter while hiding from public
/// documentation.
#[doc(hidden)]
pub mod db_query;
/// Schema errors.
pub mod error;
/// Schema domain events and pipeline events.
pub mod events;
/// Event handler implementations for pipeline observability.
pub mod handlers;
/// Blake3 hash types for content addressing.
pub mod hash;
/// File ingestion pipeline for schemas and property banks.
///
/// **Benchmark/Test access**: This module is `#[doc(hidden)] pub` to allow
/// benchmarks and tests to access the ingestor while hiding from public
/// documentation.
#[doc(hidden)]
pub mod ingestor;
/// Raw file storage types for versioned schema files.
pub mod raw_file;
/// Stored types for database persistence (read model).
pub mod stored;

/// Schema inheritance-tree builder pipeline stage.
///
/// **Benchmark access**: This module is `#[doc(hidden)] pub` to allow
/// benchmarks to measure individual pipeline stages while hiding from public
/// documentation.
#[doc(hidden)]
pub mod extender;

/// User-facing string format specifications.
pub mod formats;
/// Schema loader — orchestrates file ingestion and resolution.
pub mod loader;
/// Schema ports for CQRS.
pub mod ports;
/// Property domain entities.
pub mod property;
/// Property specification variants.
pub mod property_spec;
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
// Note: Generic wrapper boilerplate removed in Phase 6 Part B.
// Applications now use concrete types (db_command::Command, db_query::Query)
// directly.

/// Compatibility re-exports for code using the old `aggregate` module path.
///
/// The Schema aggregate has been removed in Phase 7. Types previously in
/// `schema::aggregate` are now in `schema::id`.
///
/// Note: This uses `pub use` for backward compatibility during the migration
/// period. This module will be removed in a future phase.
#[expect(clippy::pub_use, reason = "Temporary migration compatibility layer")]
pub mod aggregate {
    pub use super::id::{SchemaId, SchemaName};
}
