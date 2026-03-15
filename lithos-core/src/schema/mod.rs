//! Schema bounded context domain models.
//!
//! This module contains all entities and services for the schema system,
//! organized into focused modules for better maintainability.

#![expect(
    clippy::module_name_repetitions,
    reason = "Schema* types are namespaced intentionally for clarity"
)]

/// Schema aggregate and identifier types.
pub mod aggregate;

/// Unified repository trait and implementations for schema persistence.
///
/// Provides the `Repository` trait and `RedbRepository` implementation,
/// replacing the old CQRS Command/Query pattern.
pub mod storage;

/// View types for storage and queries.
///
/// **Migration Status**: Placeholder structure created.
/// Raw file types currently re-exported from `storage.rs`.
pub mod views;

/// PropertyBank domain aggregate for centralized property registration.
pub mod bank;
/// Property-bank reference expansion pipeline stage.
///
/// **Benchmark access**: This module is `#[doc(hidden)] pub` to allow
/// benchmarks to measure individual pipeline stages while hiding from public
/// documentation.
#[doc(hidden)]
pub mod expander;

/// Schema errors.
pub mod error;
/// Schema domain events, pipeline events, and event handlers.
pub mod events;
/// File ingestion pipeline for schemas and property banks.
///
/// **Benchmark/Test access**: This module is `#[doc(hidden)] pub` to allow
/// benchmarks and tests to access the ingestor while hiding from public
/// documentation.
#[doc(hidden)]
pub mod ingestor;

/// Schema inheritance-tree builder pipeline stage.
///
/// **Benchmark access**: This module is `#[doc(hidden)] pub` to allow
/// benchmarks to measure individual pipeline stages while hiding from public
/// documentation.
#[doc(hidden)]
pub mod extender;

/// Schema loader — orchestrates file ingestion and resolution.
pub mod loader;
/// Property domain entities.
pub mod property;
/// Property specification variants.
pub mod property_spec;
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
    use redb::{MultimapTableDefinition, TableDefinition};

    pub(crate) const SCHEMA_BY_ID: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_by_id");
    pub(crate) const SCHEMA_ID_BY_NAME: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_id_by_name");
    #[expect(
        dead_code,
        reason = "Reserved for future schema metadata storage - part of \
                  planned database schema"
    )]
    pub(crate) const SCHEMA_METADATA: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_metadata");
    pub(crate) const BANK_METADATA: TableDefinition<&str, &[u8]> =
        TableDefinition::new("bank_metadata");
    pub(crate) const BANK_PROPERTY_BY_ID: TableDefinition<&str, &[u8]> =
        TableDefinition::new("bank_property_by_id");
    #[expect(
        dead_code,
        reason = "Reserved for property lookup by name - part of planned \
                  database schema"
    )]
    pub(crate) const BANK_PROPERTY_BY_NAME: TableDefinition<&str, &[u8]> =
        TableDefinition::new("bank_property_by_name");
    pub(crate) const PROPERTY_BANK_KEY: &str = "singleton";

    // Raw file storage tables (old format - to be deprecated)
    /// Raw schema files (key: `file_path`, value: rkyv-serialized
    /// `RawSchemaFile`).
    #[expect(
        dead_code,
        reason = "Deprecated table - kept for backwards compatibility, will \
                  be removed in future version"
    )]
    pub(crate) const RAW_SCHEMA_FILES: TableDefinition<&str, &[u8]> =
        TableDefinition::new("raw_schema_files");

    /// Raw property bank file (singleton: key = `"property-bank"`).
    #[expect(
        dead_code,
        reason = "Deprecated table - kept for backwards compatibility, will \
                  be removed in future version"
    )]
    pub(crate) const RAW_PROPERTY_BANK_FILE: TableDefinition<&str, &[u8]> =
        TableDefinition::new("raw_property_bank_file");

    /// Key for raw property bank singleton.
    pub(crate) const RAW_PROPERTY_BANK_KEY: &str = "property-bank";

    // Raw view storage tables (new format - for staleness detection)
    /// Raw schema views (key: `SchemaId` as UUID string, value: rkyv-serialized
    /// `RawSchemaView`).
    pub(crate) const RAW_SCHEMA_VIEWS: TableDefinition<&str, &[u8]> =
        TableDefinition::new("raw_schema_views");

    /// Raw property bank view (singleton: key = `"property-bank"`).
    pub(crate) const RAW_PROPERTY_BANK_VIEW: TableDefinition<&str, &[u8]> =
        TableDefinition::new("raw_property_bank_view");

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
// Use explicit paths like `schema::aggregate::Schema` or
// `schema::storage::Repository` instead of re-exports to maintain clear module
// boundaries
