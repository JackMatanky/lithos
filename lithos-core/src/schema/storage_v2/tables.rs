//! Table definitions for schema storage v2.

use crate::{db::UuidTable, impl_redb_uuid, schema::identifier::SchemaId};

/// Schema aggregates (key: `SchemaId`, value: serialized `&[u8]`).
///
/// Uses zero-copy serialization via `rkyv`.
pub const SCHEMAS: UuidTable<SchemaId, &[u8]> = UuidTable::new("schemas_v2");

impl_redb_uuid!(SchemaId);
