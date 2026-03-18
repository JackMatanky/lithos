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

/// Testing and benchmarking utilities for pure unit tests.
///
/// Provides `InMemoryRepository` and test helpers to eliminate filesystem
/// IO from unit tests while maintaining test extent.
#[cfg(test)]
pub mod testing;

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

/// Schema-level property merging for inheritance.
///
/// **Benchmark access**: This module is `#[doc(hidden)] pub` to allow
/// benchmarks to measure individual pipeline stages while hiding from public
/// documentation.
#[doc(hidden)]
pub mod merger;

/// Property-level conflict resolution and override logic.
///
/// **Pipeline utility**: This module is `#[doc(hidden)] pub` to allow
/// benchmarks and pipeline stages (Expander, Merger) to use the
/// Resolver while hiding from public documentation.
#[doc(hidden)]
pub mod resolver;

pub(crate) mod db_table {
    use redb::{MultimapTableDefinition, TableDefinition};

    // ========================================================================
    // Schema Storage Tables
    // ========================================================================

    /// Schema aggregates (key: `SchemaId` as UUID string, value:
    /// rkyv-serialized `Schema`).
    pub(crate) const SCHEMA_BY_ID: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_by_id");

    /// Schema name→ID index (key: `SchemaName`, value: rkyv-serialized
    /// `SchemaId`).
    pub(crate) const SCHEMA_ID_BY_NAME: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_id_by_name");

    // ========================================================================
    // PropertyBank Storage
    // ========================================================================

    /// `PropertyBank` singleton (key: `PROPERTY_BANK_KEY`, value:
    /// rkyv-serialized `PropertyBank`).
    pub(crate) const PROPERTY_BANK: TableDefinition<&str, &[u8]> =
        TableDefinition::new("property_bank");

    /// Key for `PropertyBank` singleton table.
    pub(crate) const PROPERTY_BANK_KEY: &str = "singleton";

    /// Key for `RawPropertyBankView` singleton table.
    pub(crate) const RAW_PROPERTY_BANK_KEY: &str = "property-bank";

    // ========================================================================
    // Raw View Storage (for staleness detection)
    // ========================================================================

    /// Raw schema views (key: `SchemaId` as UUID string, value: rkyv-serialized
    /// `RawSchemaView`).
    pub(crate) const RAW_SCHEMA_VIEWS: TableDefinition<&str, &[u8]> =
        TableDefinition::new("raw_schema_views");

    /// Raw property bank view singleton (key: `PROPERTY_BANK_KEY`, value:
    /// rkyv-serialized `RawPropertyBankView`).
    pub(crate) const RAW_PROPERTY_BANK_VIEW: TableDefinition<&str, &[u8]> =
        TableDefinition::new("raw_property_bank_view");

    /// Maps file path to `SchemaId` for raw view lookup by path.
    /// Key: `file_path` (e.g., "schemas/note.toml")
    /// Value: rkyv-serialized `SchemaId`.
    pub(crate) const RAW_SCHEMA_VIEW_BY_PATH: TableDefinition<&str, &[u8]> =
        TableDefinition::new("raw_schema_view_by_path");

    // ========================================================================
    // Inheritance Tracking Tables
    // ========================================================================

    /// Multimap: parent `SchemaId` → multiple child schema records.
    /// Enables O(1) cascade staleness queries: "find all children of parent P".
    pub(crate) const SCHEMA_CHILDREN: MultimapTableDefinition<&str, &[u8]> =
        MultimapTableDefinition::new("schema_children");

    /// Regular table: child `SchemaId` → parent schema reference.
    /// Enables O(1) updates (know old parent to remove from multimap).
    /// Also tracks all schemas including roots (`parent_id` = None).
    pub(crate) const SCHEMA_PARENT: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_parent");

    /// Schema inheritance metadata cache (key: `SchemaId` as UUID string,
    /// value: rkyv-serialized `SchemaInheritanceView`).
    ///
    /// Stores precomputed inheritance metadata (ancestors, excludes, hash)
    /// to enable fast-path resolution when inheritance chains are unchanged.
    /// Read-heavy workload (every resolution) vs rare writes (schema
    /// restructuring).
    pub(crate) const SCHEMA_INHERITANCE: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_inheritance");
}

// --- Public API ---
// Use explicit paths like `schema::aggregate::Schema` or
// `schema::storage::Repository` instead of re-exports to maintain clear module
// boundaries
