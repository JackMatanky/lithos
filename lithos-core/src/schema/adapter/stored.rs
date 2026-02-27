//! Storage representation for schema aggregates.
//!
//! [`StoredSchema`] is the rkyv-serialized adapter type persisted to the
//! `schema_by_id` table. It carries all metadata needed for staleness
//! checking and tree reconstruction, eliminating the need for a separate
//! `schema_metadata` table.

use super::super::{
    aggregate::{Schema, SchemaId, SchemaName, Timestamp},
    bank::BankVersion,
    error::SchemaError,
    property::{Cardinality, Multiplicity, Property, PropertyId, PropertyName},
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
    /// Bank version at time of resolution; used for staleness detection.
    pub bank_version: BankVersion,
    /// Filesystem birthtime (from `Metadata::created()`), if available.
    pub created_at: Option<Timestamp>,
    /// Filesystem mtime (from `Metadata::modified()`), if available.
    pub modified_at: Option<Timestamp>,
    /// Wall-clock timestamp when this record was written to the database.
    pub recorded_at: Timestamp,
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
    /// Whether the property is required (flattened from `Cardinality`).
    pub required: bool,
    /// Whether the property accepts multiple values (flattened from
    /// `Multiplicity`).
    pub multi: bool,
    /// Type-specific validation constraints.
    pub spec: PropertySpec,
}

/// Build a [`StoredSchema`] from a domain [`Schema`] and storage metadata.
///
/// Called by the command adapter before persisting.
///
/// The `parent_id` parameter is now sourced from `schema.parent_id()` instead
/// of being passed separately (as of the `parent_id` domain migration).
pub(crate) fn to_stored(
    schema: &Schema,
    bank_version: BankVersion,
    created_at: Option<Timestamp>,
    modified_at: Option<Timestamp>,
) -> StoredSchema {
    let properties = schema
        .properties()
        .map(|p| StoredProperty {
            id: p.id(),
            name: p.name().as_str().into(),
            required: p.cardinality() == Cardinality::Required,
            multi: p.multiplicity() == Multiplicity::Many,
            spec: p.spec().clone(),
        })
        .collect();

    StoredSchema {
        id: schema.id(),
        name: schema.name().as_str().into(),
        parent_id: schema.parent_id(),
        properties,
        bank_version,
        created_at,
        modified_at,
        recorded_at: Timestamp::now(),
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
                let cardinality = Cardinality::from(sp.required);
                let multiplicity = Multiplicity::from(sp.multi);
                Ok(Property::new(
                    sp.id,
                    prop_name,
                    cardinality,
                    multiplicity,
                    sp.spec,
                ))
            })
            .collect();
        let properties = properties?;
        Ok(Schema::reconstruct(stored.id, name, stored.parent_id, properties))
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::schema::{
        aggregate::SchemaId,
        bank::BankVersion,
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
        let stored = to_stored(
            &schema,
            BankVersion::initial(),
            Some(Timestamp::from_secs(1_000_000)),
            Some(Timestamp::from_secs(2_000_000)),
        );

        assert_eq!(stored.id, TEST_SCHEMA_ID, "ID should match");
        assert_eq!(stored.name.as_ref(), "test-stored", "Name should match");
        assert!(stored.parent_id.is_none(), "No parent");
        assert!(stored.properties.is_empty(), "No properties");
        assert_eq!(
            stored.created_at,
            Some(Timestamp::from_secs(1_000_000)),
            "Created time should match"
        );
        assert_eq!(
            stored.modified_at,
            Some(Timestamp::from_secs(2_000_000)),
            "Modified time should match"
        );

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
            Cardinality::Required,
            Multiplicity::Single,
            PropertySpec::Bool(BoolSpec::default()),
        );

        let schema_name = SchemaName::new("with-props").expect("valid name");
        let schema =
            Schema::reconstruct(TEST_SCHEMA_ID, schema_name, None, vec![prop]);

        let stored = to_stored(
            &schema,
            BankVersion::initial(),
            Some(Timestamp::from_secs(0)),
            Some(Timestamp::from_secs(0)),
        );

        assert_eq!(stored.properties.len(), 1, "One property stored");
        let sp = stored.properties.first().expect("One property stored");
        assert_eq!(sp.name.as_ref(), "flag", "Property name stored");
        assert!(sp.required, "Required flag stored correctly");
        assert!(!sp.multi, "Multi flag stored correctly");
    }
}
