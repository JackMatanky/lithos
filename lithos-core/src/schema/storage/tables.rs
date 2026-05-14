//! Table definitions for schema storage.

use crate::{
    db::{PathTable, Table, UuidTable},
    impl_redb_uuid,
    schema::identifier::SchemaId,
};

impl_redb_uuid!(SchemaId);
/// Schema aggregates with zero-copy serialization.
///
/// Stores full `Schema` structures indexed by schema ID for efficient
/// retrieval by identity.
///
/// Key: `SchemaId`
/// Value: rkyv-serialized `Schema`
pub const SCHEMAS: UuidTable<SchemaId, &[u8]> = UuidTable::new("schemas");

/// Raw schema views indexed by schema ID.
///
/// Stores full `RawSchemaView` structures containing path, version history,
/// and content hashes for staleness detection and batch retrieval.
///
/// Key: `SchemaId`
/// Value: serialized `RawSchemaView`
pub const RAW_SCHEMA_VIEWS: UuidTable<SchemaId, &[u8]> =
    UuidTable::new("raw_schema_views");

/// Property Bank singleton with all registered property definitions.
///
/// Stores the global registry of reusable property specs that schemas can
/// reference. Uses a singleton pattern with a constant key.
///
/// Key: `PROPERTY_BANK_KEY` (constant string)
/// Value: serialized `PropertyBank`
pub const PROPERTY_BANK: Table<&str, &[u8]> = Table::new("property_bank");

/// Constant key for the Property Bank singleton table.
pub const PROPERTY_BANK_KEY: &str = "singleton";

/// Raw property bank view for staleness detection.
///
/// Maps property bank file paths (e.g., "property-bank.toml",
/// "property-bank.json") to their raw views, enabling change detection
/// without re-parsing the full property bank.
///
/// Key: path string
/// Value: serialized `RawPropertyBankView`
pub const RAW_PROPERTY_BANK_VIEW: PathTable<&[u8]> =
    PathTable::new("raw_property_bank_view");

/// Cached base properties for schema files.
///
/// Stores the fully converted (hydrated) property map for each schema,
/// excluding any inherited properties. This enables skipping the
/// `RefExpander` when the property bank has not changed.
///
/// Key: `SchemaId`.
/// Value: rkyv-serialized `BasePropertiesView`.
pub const SCHEMA_BASE_PROPERTIES: UuidTable<SchemaId, &[u8]> =
    UuidTable::new("schema_base_properties");

/// Topological inheritance graph singleton with DAG structure.
///
/// Stores the persisted directed acyclic graph used for schema inheritance
/// resolution. Contains `SchemaId` links and adjacency lists.
/// Rebuilt or patched when inheritance relationships change.
///
/// Key: `TOPOLOGICAL_GRAPH_KEY` (constant string)
/// Value: serialized `InheritanceGraph<()>`
pub const SCHEMA_TOPOLOGICAL_GRAPH: Table<&str, &[u8]> =
    Table::new("schema_topological_graph");

/// Constant key for the topological inheritance graph singleton table.
pub const TOPOLOGICAL_GRAPH_KEY: &str = "graph_singleton";

/// Schema name-to-ID index for fast name-based lookup.
///
/// Enables resolving schema IDs by name without loading full schema data.
/// Maintained atomically with `SCHEMAS` during save operations.
///
/// Key: schema name string
/// Value: serialized `SchemaId`
pub const SCHEMA_ID_BY_NAME: Table<&str, &[u8]> =
    Table::new("schema_id_by_name");

/// Path-to-SchemaId index for raw view lookup.
///
/// Maps file paths (e.g., "note.toml", "task.json") to their corresponding
/// schema IDs, enabling path-based schema retrieval.
///
/// Key: path string with extension
/// Value: `SchemaId`
pub const SCHEMA_ID_BY_PATH: PathTable<&[u8]> =
    PathTable::new("schema_id_by_path");
