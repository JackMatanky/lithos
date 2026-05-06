//! Cached base properties for schema files.
//!
//! Contains the fully converted (hydrated) property map for a schema,
//! excluding any inherited properties. Stored in the `schema_base_properties`
//! table with `SchemaId` as the key.
//!
//! The `hash` field contains per-property hashes that must match
//! `RawSchemaView.current().hashes().properties()` to ensure the
//! cached properties coincide with the latest snapshot.

use std::time::SystemTime;

use rkyv::{Archive, Deserialize, Serialize};

use super::RawPropertyMapHash;
use crate::schema::property::PropertyMap;

/// Cached base properties for a schema file.
///
/// Contains the fully converted `PropertyMap` (local properties only,
/// with all `$ref` entries resolved against the property bank).
/// Stored in the `schema_base_properties` table (key: `SchemaId`).
///
/// The `hash` field must match `RawSchemaView.current().hashes().properties()`
/// to ensure this cache coincides with the latest file snapshot.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BasePropertiesView {
    /// Fully converted `PropertyMap` (local properties only, $ref resolved).
    properties: PropertyMap,

    /// Per-property hashes matching the current snapshot.
    /// Must match `RawSchemaView.current().hashes().properties()`.
    hash: RawPropertyMapHash,

    /// When this cache entry was recorded.
    #[rkyv(with = rkyv::with::AsUnixTime)]
    recorded_at: SystemTime,
}

impl BasePropertiesView {
    /// Creates a new `BasePropertiesView`.
    #[inline]
    #[must_use]
    pub fn new(properties: PropertyMap, hash: RawPropertyMapHash) -> Self {
        Self {
            properties,
            hash,
            recorded_at: SystemTime::now(),
        }
    }

    /// Returns the cached property map.
    #[inline]
    #[must_use]
    pub const fn properties(&self) -> &PropertyMap {
        &self.properties
    }

    /// Returns the per-property hashes.
    #[inline]
    #[must_use]
    pub const fn hash(&self) -> &RawPropertyMapHash {
        &self.hash
    }

    /// Returns when this cache entry was recorded.
    #[inline]
    #[must_use]
    pub const fn recorded_at(&self) -> &SystemTime {
        &self.recorded_at
    }
}
