//! Table definitions for schema storage v2.

use crate::{
    db::{PathTable, UuidTable},
    impl_redb_uuid,
    schema::identifier::SchemaId,
};

impl_redb_uuid!(SchemaId);
/// Schema aggregates (key: `SchemaId`, value: rkyv-serialized `Schema`).
///
/// Uses zero-copy serialization via `rkyv`.
pub const SCHEMAS: UuidTable<SchemaId, &[u8]> = UuidTable::new("schemas_v2");

/// Path-to-SchemaId index for raw view lookup (key: path string, value:
/// `SchemaId`).
///
/// Maps file paths to their corresponding schema IDs for path-based lookups.
/// Keys are path strings with extension (e.g., "note.toml", "task.json"),
/// validated at insert time.
pub const SCHEMA_ID_BY_PATH: PathTable<&[u8]> =
    PathTable::new("schema_id_by_path_v2");

/// Raw schema views by ID (key: `SchemaId`, value: serialized `RawSchemaView`).
///
/// Stores full `RawSchemaView` structures indexed by schema ID for efficient
/// batch retrieval. Views contain path, version history, and hashes for
/// staleness detection.
pub const RAW_SCHEMA_VIEWS: UuidTable<SchemaId, &[u8]> =
    UuidTable::new("raw_schema_views_v2");

/// Property Bank singleton (key: constant string, value: serialized
/// `PropertyBank`).
///
/// The Property Bank stores all registered property definitions that schemas
/// can reference. Uses a singleton pattern with a constant key.
pub const PROPERTY_BANK: PathTable<&[u8]> = PathTable::new("property_bank_v2");

/// Constant key for Property Bank singleton table.
pub const PROPERTY_BANK_KEY: &str = "singleton";

/// Raw property bank view by path (key: path string, value: serialized
/// `RawPropertyBankView`).
///
/// Maps property bank file paths (e.g., "property-bank.toml",
/// "property-bank.json") to their raw views for staleness detection.
pub const RAW_PROPERTY_BANK_VIEW: PathTable<&[u8]> =
    PathTable::new("raw_property_bank_view_v2");

/// Topological inheritance graph singleton.
///
/// Key: Constant `TOPOLOGICAL_GRAPH_KEY` (singleton)
/// Value: serialized `InheritanceGraph<()>`.
///
/// Contains DAG structure with `SchemaId` links and adjacency lists.
/// Rebuilt/patched when inheritance relationships change.
pub const SCHEMA_TOPOLOGICAL_GRAPH: PathTable<&[u8]> =
    PathTable::new("schema_topological_graph_v2");

/// Constant key for topological graph singleton table.
pub const TOPOLOGICAL_GRAPH_KEY: &str = "graph_singleton";

/// Schema name→ID index (key: schema name string, value: serialized
/// `SchemaId`).
///
/// Enables fast ID lookup by schema name without loading full schema data.
/// Maintained atomically with `SCHEMAS` table during save operations.
pub const SCHEMA_ID_BY_NAME: PathTable<&[u8]> =
    PathTable::new("schema_id_by_name_v2");

/// Cached base properties for schema files.
///
/// Stores the fully converted (hydrated) property map for each schema,
/// excluding any inherited properties. This enables skipping the
/// `RefExpander` when the property bank has not changed.
///
/// Key: `SchemaId` as UUID string.
/// Value: rkyv-serialized `BasePropertiesView`.
pub const SCHEMA_BASE_PROPERTIES: PathTable<&[u8]> =
    PathTable::new("schema_base_properties");
