//! Storage representation for resolved schemas (read model pattern).
//!
//! ## Read Model Architecture
//!
//! [`StoredSchema`] is a **read model** - it contains no behavior, no events,
//! and no domain logic. It is purely structured data optimized for storage
//! and retrieval.
//!
//! - **Not a DDD Aggregate**: No state transitions, no invariant enforcement
//! - **Orchestration**: All pipeline logic lives in [`crate::schema::loader`]
//! - **Zero-Copy Reads**: Uses `rkyv` for fast deserialization-free access
//!
//! ## Storage Tables
//!
//! Schema storage uses:
//! - `schema_by_id` - Resolved schemas (rkyv-serialized)
//! - `schema_metadata` - Staleness metadata (hash, timestamps, bank version)
//!
//! Property bank storage uses:
//! - `bank_metadata` - Version/timestamp tracking
//! - `bank_property_by_id` - ID-keyed property snapshots
//! - `bank_property_by_name` - Name-keyed property snapshots

// Clippy false positive: Archive macro generates internal types that trigger
// exhaustive_structs, but our public types are marked #[non_exhaustive].
// This cannot be fixed without changes to rkyv.
#![expect(
    clippy::exhaustive_structs,
    reason = "False positive from rkyv Archive macro - all public types use \
              #[non_exhaustive]"
)]

use std::time::SystemTime;

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

use super::{
    aggregate::SchemaId,
    bank::{BankVersion, PropertyBank},
    error::SchemaError,
    hash::Blake3Hash,
    property::{Multiplicity, Optionality, Property, PropertyId, PropertyName},
    property_spec::PropertySpec,
};

/// Storage representation of a resolved schema (read model).
///
/// ## Read Model Pattern
///
/// This type is a **read model** - it has no behavior, no events, and no
/// domain logic. It exists purely to store and retrieve resolved schema data.
///
/// - **No Methods**: Only field accessors (getters)
/// - **No State Transitions**: Immutable after resolution
/// - **No Events**: Event emission happens in [`crate::schema::loader`]
///
/// ## Storage
///
/// Persisted to the `schema_by_id` table using `rkyv` serialization.
/// Contains all fields required for staleness checking and inheritance
/// tree reconstruction.
///
/// This is now the primary schema type used throughout the system.
/// Files are the source of truth; schemas are loaded, resolved, and stored
/// as `StoredSchema` values.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StoredSchema {
    /// Schema identity.
    pub id: SchemaId,
    /// Schema name (flattened from `SchemaName` newtype).
    pub name: Box<str>,
    /// Parent schema ID, for `SchemaTree` reconstruction.
    pub parent_id: Option<SchemaId>,
    /// Resolved properties (flattened).
    pub properties: Vec<StoredProperty>,
}

impl StoredSchema {
    /// Create a new `StoredSchema` for testing purposes.
    #[inline]
    #[must_use]
    pub fn new(
        id: SchemaId,
        name: Box<str>,
        parent_id: Option<SchemaId>,
        properties: Vec<StoredProperty>,
    ) -> Self {
        Self {
            id,
            name,
            parent_id,
            properties,
        }
    }
}

/// Adapter storage representation of a property bank snapshot.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub(crate) struct StoredPropertyBank {
    /// Bank version at time of persistence.
    pub bank_version: BankVersion,
    /// Wall-clock timestamp when this record was written.
    #[rkyv(with = AsUnixTime)]
    pub recorded_at: SystemTime,
    /// Flattened properties in the bank.
    pub properties: Vec<StoredProperty>,
}

/// Adapter storage representation of property bank metadata.
///
/// # Timestamps
///
/// Uses `SystemTime` with rkyv's `AsUnixTime` wrapper for safe serialization.
/// This stores timestamps as Unix epoch seconds internally while preserving
/// `SystemTime`'s type safety.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StoredMetadata {
    /// Bank version at time of persistence.
    pub bank_version: BankVersion,
    /// Blake3 hash of source file content (for accurate staleness detection).
    pub source_file_hash: Blake3Hash,
    /// Filesystem birthtime (from `Metadata::created()`), if available.
    #[rkyv(with = Map<AsUnixTime>)]
    pub created_at: Option<SystemTime>,
    /// Filesystem mtime (from `Metadata::modified()`), if available.
    #[rkyv(with = Map<AsUnixTime>)]
    pub modified_at: Option<SystemTime>,
    /// Wall-clock timestamp when this record was written.
    #[rkyv(with = AsUnixTime)]
    pub recorded_at: SystemTime,
}

impl StoredMetadata {
    /// Build metadata for storage.
    #[inline]
    pub(crate) fn new(
        bank_version: BankVersion,
        source_file_hash: Blake3Hash,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Self {
        let recorded_at = SystemTime::now();
        Self {
            bank_version,
            source_file_hash,
            created_at,
            modified_at,
            recorded_at,
        }
    }
}

/// Child schema metadata stored in the `schema_children` multimap.
///
/// **Storage pattern:**
/// - Table: `schema_children` (multimap)
/// - Key: Parent `SchemaId` (as UUID string)
/// - Values: Multiple `StoredChildSchema` entries (one per child)
///
/// Each parent can have many children. This structure stores each child's
/// inheritance metadata including which properties it excludes from the parent.
///
/// **Cascade staleness:** When a parent schema changes, query this multimap
/// to find all children that must be re-resolved.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub(crate) struct StoredChildSchema {
    /// Child schema ID.
    pub child_id: SchemaId,
    /// Property names this child excludes from parent's properties.
    pub excludes: Vec<Box<str>>,
    /// Timestamp when this inheritance relationship was last resolved.
    #[rkyv(with = AsUnixTime)]
    pub resolved_at: SystemTime,
}

impl StoredChildSchema {
    /// Serialize to bytes for multimap storage.
    ///
    /// # Errors
    /// Returns serialization error if rkyv encoding fails.
    pub(crate) fn to_bytes(&self) -> Result<Vec<u8>, crate::db::DbError> {
        rkyv::to_bytes(self).map(|bytes| bytes.to_vec()).map_err(
            |e: rkyv::rancor::Error| {
                crate::db::DbError::Serialization(e.to_string())
            },
        )
    }
}

/// Parent schema reference, stored in `schema_parent` table.
///
/// **Storage pattern:**
/// - Table: `schema_parent` (regular table, not multimap)
/// - Key: Child `SchemaId` (as UUID string)
/// - Value: `StoredParentSchema`
///
/// This table tracks ALL schemas (both roots and children):
/// - Root schemas: `parent_id = None`
/// - Child schemas: `parent_id = Some(parent_id)`
///
/// **Update optimization:** When updating a child's parent, this table
/// provides O(1) lookup of the old parent plus the old excludes/timestamp
/// needed to reconstruct the exact bytes for removing the old entry from
/// the `schema_children` multimap.
///
/// **Data redundancy:** `excludes` and `resolved_at` are stored in both
/// `schema_parent` and `schema_children`. This trades ~10KB of storage
/// (for typical 100-schema vaults) for simpler, faster update logic.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub(crate) struct StoredParentSchema {
    /// Parent schema ID, or None for root schemas.
    pub parent_id: Option<SchemaId>,
    /// Property names excluded from parent (cached for multimap removal).
    pub excludes: Vec<Box<str>>,
    /// Timestamp when relationship was resolved (cached for multimap removal).
    #[rkyv(with = AsUnixTime)]
    pub resolved_at: SystemTime,
}

/// Adapter storage representation of a single bank property snapshot.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub(crate) struct StoredBankProperty {
    /// Bank version at time of persistence.
    pub bank_version: BankVersion,
    /// Wall-clock timestamp when this record was written.
    #[rkyv(with = AsUnixTime)]
    pub recorded_at: SystemTime,
    /// Flattened property payload.
    pub property: StoredProperty,
}

impl StoredBankProperty {
    /// Format a bank property key for the given version.
    #[inline]
    pub(crate) fn key(version: BankVersion, suffix: &str) -> String {
        format!("{}:{suffix}", version.as_u64())
    }

    /// Format a bank property key prefix for the given version.
    #[inline]
    pub(crate) fn prefix(version: BankVersion) -> String {
        format!("{}:", version.as_u64())
    }
}

impl TryFrom<StoredPropertyBank> for PropertyBank {
    type Error = SchemaError;

    #[inline]
    fn try_from(stored: StoredPropertyBank) -> Result<Self, Self::Error> {
        let properties: Result<Vec<_>, _> = stored
            .properties
            .into_iter()
            .map(|sp| {
                let prop_name = PropertyName::try_new(&sp.name)?;
                let optionality = Optionality::from(sp.required);
                let multiplicity = Multiplicity::from(sp.multi);
                Ok(Property::new(
                    sp.id,
                    prop_name,
                    optionality,
                    multiplicity,
                    sp.spec,
                ))
            })
            .collect();
        PropertyBank::try_reconstruct(properties?, stored.bank_version)
    }
}

/// Flat storage representation of a single property.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StoredProperty {
    /// Property identity.
    pub id: PropertyId,
    /// Property name (flattened from `PropertyName` newtype).
    pub name: Box<str>,
    /// Whether the property is required (flattened from `Optionality`).
    pub required: bool,
    /// Whether the property accepts multiple values (flattened from
    /// `Multiplicity`).
    pub multi: bool,
    /// Type-specific validation constraints.
    pub spec: PropertySpec,
}

impl StoredProperty {
    /// Create a new `StoredProperty`.
    #[inline]
    #[must_use]
    pub fn new(
        id: PropertyId,
        name: Box<str>,
        required: bool,
        multi: bool,
        spec: PropertySpec,
    ) -> Self {
        Self {
            id,
            name,
            required,
            multi,
            spec,
        }
    }
}
