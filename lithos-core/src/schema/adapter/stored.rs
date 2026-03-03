//! Storage representation for schema aggregates.
//!
//! [`StoredSchema`] is the rkyv-serialized adapter type persisted to the
//! `schema_by_id` table. Metadata for staleness checking lives in the
//! `schema_metadata` table via [`StoredMetadata`].
//!
//! Property bank storage uses:
//! - `bank_metadata` for version/timestamp tracking
//! - `bank_property_by_id` for ID-keyed snapshots
//! - `bank_property_by_name` for name-keyed snapshots

use super::super::{
    aggregate::{Schema, SchemaId, SchemaName, Timestamp},
    bank::{BankVersion, PropertyBank},
    error::SchemaError,
    property::{Multiplicity, Optionality, Property, PropertyId, PropertyName},
    property_spec::PropertySpec,
};

/// Adapter storage representation of a resolved schema.
///
/// Persisted to the `schema_by_id` table. Contains all fields required
/// for staleness checking and `SchemaTree` reconstruction.
///
/// This type lives in the adapter layer and is never exposed to the domain.
/// Conversions between `Schema` and `StoredSchema` are the responsibility
/// of the command and query adapters.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub(crate) struct StoredSchema {
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
    /// Build a [`StoredSchema`] from a domain [`Schema`].
    ///
    /// Called by the command adapter before persisting.
    ///
    /// The `parent_id` parameter is now sourced from `schema.parent_id()`
    /// instead of being passed separately (as of the `parent_id` domain
    /// migration).
    pub(crate) fn from_schema(schema: &Schema) -> Self {
        let properties = schema
            .properties()
            .map(|p| StoredProperty {
                id: p.id(),
                name: p.name().as_str().into(),
                required: p.optionality() == Optionality::Required,
                multi: p.multiplicity() == Multiplicity::Many,
                spec: p.spec().clone(),
            })
            .collect();

        Self {
            id: schema.id(),
            name: schema.name().as_str().into(),
            parent_id: schema.parent_id(),
            properties,
        }
    }
}

impl TryFrom<StoredSchema> for Schema {
    type Error = SchemaError;

    #[inline]
    fn try_from(stored: StoredSchema) -> Result<Self, Self::Error> {
        let name = SchemaName::new(&stored.name)?;
        let properties: Result<Vec<_>, _> = stored
            .properties
            .into_iter()
            .map(|sp| {
                let prop_name = PropertyName::new(&sp.name)?;
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
        let properties = properties?;
        Ok(Schema::reconstruct(stored.id, name, stored.parent_id, properties))
    }
}

/// Adapter storage representation of a property bank snapshot.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub(crate) struct StoredPropertyBank {
    /// Bank version at time of persistence.
    pub bank_version: BankVersion,
    /// Wall-clock timestamp when this record was written.
    pub recorded_at: Timestamp,
    /// Flattened properties in the bank.
    pub properties: Vec<StoredProperty>,
}

/// Adapter storage representation of property bank metadata.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub struct StoredMetadata {
    /// Bank version at time of persistence.
    pub bank_version: BankVersion,
    /// Filesystem birthtime (from `Metadata::created()`), if available.
    pub created_at: Option<Timestamp>,
    /// Filesystem mtime (from `Metadata::modified()`), if available.
    pub modified_at: Option<Timestamp>,
    /// Wall-clock timestamp when this record was written.
    pub recorded_at: Timestamp,
}

impl StoredMetadata {
    /// Build metadata for storage.
    #[inline]
    pub(crate) fn new(
        bank_version: BankVersion,
        created_at: Option<Timestamp>,
        modified_at: Option<Timestamp>,
    ) -> Self {
        Self {
            bank_version,
            created_at,
            modified_at,
            recorded_at: Timestamp::now(),
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
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub(crate) struct StoredChildSchema {
    /// Child schema ID.
    pub child_id: SchemaId,
    /// Property names this child excludes from parent's properties.
    pub excludes: Vec<Box<str>>,
    /// Timestamp when this inheritance relationship was last resolved.
    pub resolved_at: Timestamp,
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
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub(crate) struct StoredParentSchema {
    /// Parent schema ID, or None for root schemas.
    pub parent_id: Option<SchemaId>,
    /// Property names excluded from parent (cached for multimap removal).
    pub excludes: Vec<Box<str>>,
    /// Timestamp when relationship was resolved (cached for multimap removal).
    pub resolved_at: Timestamp,
}

/// Adapter storage representation of a single bank property snapshot.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub(crate) struct StoredBankProperty {
    /// Bank version at time of persistence.
    pub bank_version: BankVersion,
    /// Wall-clock timestamp when this record was written.
    pub recorded_at: Timestamp,
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
                let prop_name = PropertyName::try_from(sp.name)?;
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
        PropertyBank::reconstruct(properties?, stored.bank_version)
    }
}

/// Flat storage representation of a single property.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub(crate) struct StoredProperty {
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

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::schema::{
        aggregate::SchemaId,
        property_spec::{BoolSpec, PropertySpec},
    };

    const TEST_SCHEMA_ID: SchemaId = SchemaId::from_uuid(Uuid::from_u128(
        0x018C_0000_0000_7000_8000_0000_0000_0801,
    ));
    const TEST_PROP_ID: PropertyId = PropertyId::from_uuid(Uuid::from_u128(
        0x018C_0000_0000_7000_8000_0000_0000_0802,
    ));

    fn make_schema() -> Schema {
        let name = SchemaName::new("test-stored").expect("valid name");
        Schema::reconstruct(TEST_SCHEMA_ID, name, None, vec![])
    }

    #[test]
    fn to_stored_round_trips_to_schema() {
        let schema = make_schema();
        let stored = StoredSchema::from_schema(&schema);

        assert_eq!(stored.id, TEST_SCHEMA_ID, "ID should match");
        assert_eq!(stored.name.as_ref(), "test-stored", "Name should match");
        assert!(stored.parent_id.is_none(), "No parent");
        assert!(stored.properties.is_empty(), "No properties");
        let recovered =
            Schema::try_from(stored).expect("Round-trip should succeed");
        assert_eq!(
            recovered.name().as_str(),
            "test-stored",
            "Name round-trip should match"
        );
    }

    #[test]
    fn to_stored_includes_properties() {
        let prop_name = PropertyName::new("flag").expect("valid name");
        let prop = Property::new(
            TEST_PROP_ID,
            prop_name,
            Optionality::Required,
            Multiplicity::Single,
            PropertySpec::Bool(BoolSpec::default()),
        );

        let schema_name = SchemaName::new("with-props").expect("valid name");
        let schema =
            Schema::reconstruct(TEST_SCHEMA_ID, schema_name, None, vec![prop]);

        let stored = StoredSchema::from_schema(&schema);

        assert_eq!(stored.properties.len(), 1, "One property stored");
        let sp = stored.properties.first().expect("One property stored");
        assert_eq!(sp.name.as_ref(), "flag", "Property name stored");
        assert!(sp.required, "Required flag stored correctly");
        assert!(!sp.multi, "Multi flag stored correctly");
    }
}
