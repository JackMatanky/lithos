//! Table definitions for schema storage v2.

use crate::{
    db::{PathTable, UuidTable},
    impl_redb_uuid,
    schema::identifier::SchemaId,
};

impl_redb_uuid!(SchemaId);
/// Schema aggregates (key: `SchemaId`, value: serialized `&[u8]`).
///
/// Uses zero-copy serialization via `rkyv`.
pub const SCHEMAS: UuidTable<SchemaId, &[u8]> = UuidTable::new("schemas_v2");

/// Path-to-SchemaId index (key: path string, value: `SchemaId`).
///
/// Maps file paths to their corresponding schema IDs for path-based lookups.
/// Keys are path strings (validated at insert time), values are serialized
/// `SchemaId`s.
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
/// Maps property bank file paths to their raw views for staleness detection.
pub const RAW_PROPERTY_BANK_VIEW: PathTable<&[u8]> =
    PathTable::new("raw_property_bank_view_v2");
