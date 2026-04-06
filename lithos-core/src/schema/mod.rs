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
/// PropertyBank state machine for incremental loading and staleness detection.
pub mod property_bank_processor;

/// Batch-based schema processor pipeline.
///
/// **Pipeline utility**: This module is `#[doc(hidden)] pub` to allow
/// builder and tests to use the new batch processor.
#[doc(hidden)]
pub mod schema_processor;

/// DAG structures and algorithms for schema inheritance.
pub mod graph;

/// Schema errors.
pub mod error;
/// Schema domain events, pipeline events, and event handlers.
pub mod events;

/// Facade for schema orchestration.
pub mod builder;
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

pub(crate) mod db_table {
    use redb::TableDefinition;

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

    /// Maps schema filename to `SchemaId` for raw view lookup.
    /// Key: filename with extension (e.g., "note.toml", "task.json")
    /// Value: rkyv-serialized `SchemaId`.
    pub(crate) const SCHEMA_ID_BY_PATH: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_id_by_path");

    // ========================================================================
    // PropertyBank Storage
    // ========================================================================

    /// `PropertyBank` singleton (key: `PROPERTY_BANK_KEY`, value:
    /// rkyv-serialized `PropertyBank`).
    pub(crate) const PROPERTY_BANK: TableDefinition<&str, &[u8]> =
        TableDefinition::new("property_bank");

    /// Key for `PropertyBank` singleton table.
    pub(crate) const PROPERTY_BANK_KEY: &str = "singleton";

    // ========================================================================
    // Raw View Storage (for staleness detection)
    // ========================================================================

    /// Raw schema views (key: `SchemaId` as UUID string, value: rkyv-serialized
    /// `RawSchemaView`).
    pub(crate) const RAW_SCHEMA_VIEWS: TableDefinition<&str, &[u8]> =
        TableDefinition::new("raw_schema_views");

    /// Raw property bank view (key: filename with extension, value:
    /// rkyv-serialized `RawPropertyBankView`).
    /// Key examples: "property-bank.toml", "property-bank.json".
    pub(crate) const RAW_PROPERTY_BANK_VIEW: TableDefinition<&str, &[u8]> =
        TableDefinition::new("raw_property_bank_view");

    // ========================================================================
    // Inheritance Tracking Tables
    // ========================================================================

    /// Topologically sorted inheritance graph singleton.
    ///
    /// Key: Constant `TOPOLOGICAL_GRAPH_KEY` (singleton)
    /// Value: rkyv-serialized `TopologicalGraph<InheritanceNode>`.
    ///
    /// Contains DAG structure with `SchemaId` links, child lists, and depth
    /// information. Rebuilt/patched when inheritance relationships change.
    pub(crate) const SCHEMA_TOPOLOGICAL_GRAPH: TableDefinition<&str, &[u8]> =
        TableDefinition::new("schema_topological_graph");

    /// Key for `TopologicalGraph` singleton table.
    pub(crate) const TOPOLOGICAL_GRAPH_KEY: &str = "graph_singleton";
}

// --- Public API ---
// Use explicit paths like `schema::aggregate::Schema` or
// `schema::storage::Repository` instead of re-exports to maintain clear module
// boundaries
