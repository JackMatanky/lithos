//! `RefExpander` — expands raw property bank `$ref` pointers into
//! validated [`Property`] values.
//!
//! # Pipeline position
//!
//! ```text
//! Ingestor → Vec<(SchemaId, RawSchema)>
//! RefExpander  ← here
//! → Vec<(SchemaId, RefExpandedSchema)>
//! Extender
//! Merger
//! ```
//!
//! After `RefExpander` completes, the [`PropertyBank`] is no longer
//! referenced anywhere in the pipeline.
//!
//! # Design
//!
//! - **Input**: stale `Vec<(SchemaId, RawSchema)>` + `&PropertyBank`
//! - **Output**: `Vec<(SchemaId, RefExpandedSchema)>`
//! - Properties in each `RefExpandedSchema` are stored in `HashMap` for O(1)
//!   lookup by downstream Merger

use std::collections::HashMap;

use super::{
    aggregate::SchemaName,
    bank::PropertyBank,
    error::SchemaError,
    property::{Multiplicity, Optionality, Property, PropertyId, PropertyName},
    raw::{RawSchema, property::RawProperty},
    resolver::Resolver,
};
use crate::schema::aggregate::SchemaId;

// ─────────────────────────────────────────────────────────────────────────────
//  RefExpandedSchema
// ─────────────────────────────────────────────────────────────────────────────

/// A raw schema with all `$ref` pointers resolved against the property bank.
///
/// Properties are stored in `HashMap` for O(1) lookup by name.
///
/// **Internal API**: This type is public solely for benchmarking purposes.
/// Do not depend on it in production code - use `Loader` instead.
#[doc(hidden)]
#[derive(Clone)]
#[non_exhaustive]
pub struct RefExpandedSchema {
    /// Validated schema name (carried forward from `RawSchema`).
    pub name: SchemaName,
    /// Optional parent schema name (carried forward from `RawSchema.extends`).
    pub extends: Option<SchemaName>,
    /// Validated property names to exclude from the parent schema.
    pub excludes: Vec<PropertyName>,
    /// Fully resolved own properties (`HashMap` for O(1) lookup).
    pub properties: HashMap<PropertyName, Property>,
}

// ─────────────────────────────────────────────────────────────────────────────
//  RefExpander
// ─────────────────────────────────────────────────────────────────────────────

/// Resolves raw `$ref` property pointers against the [`PropertyBank`] and
/// validates inline property definitions.
///
/// Holds a shared reference to the bank so callers can pre-build the bank
/// once and reuse the expander across multiple schemas.
///
/// **Internal API**: This type is public solely for benchmarking purposes.
/// Do not depend on it in production code - use `Loader` instead.
#[doc(hidden)]
pub struct RefExpander<'bank> {
    bank: &'bank PropertyBank,
}

impl<'bank> RefExpander<'bank> {
    /// Create a new `RefExpander` bound to the given [`PropertyBank`].
    ///
    /// **Internal API**: Public for benchmarking only.
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub const fn new(bank: &'bank PropertyBank) -> Self {
        Self {
            bank,
        }
    }

    /// Expand references for a batch of stale raw schemas.
    ///
    /// For each `(SchemaId, RawSchema)` pair, all properties are resolved
    /// against the bank and the results are returned in the same order.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaError`] if any property fails validation or a `$ref`
    /// path cannot be resolved.
    ///
    /// **Internal API**: Public for benchmarking only.
    #[doc(hidden)]
    #[inline]
    pub fn expand_all(
        &self,
        schemas: Vec<(SchemaId, RawSchema)>,
    ) -> Result<Vec<(SchemaId, RefExpandedSchema)>, SchemaError> {
        schemas
            .into_iter()
            .map(|(id, raw)| {
                let expanded = self.expand_schema(&raw)?;
                Ok((id, expanded))
            })
            .collect()
    }

    /// Expand references for a single [`RawSchema`].
    fn expand_schema(
        &self,
        raw: &RawSchema,
    ) -> Result<RefExpandedSchema, SchemaError> {
        let mut properties = HashMap::with_capacity(raw.properties().len());

        for (prop_name, entry) in raw.properties() {
            let (name, prop) = self.expand_property(prop_name, entry)?;
            properties.insert(name.clone(), prop);
        }

        Ok(RefExpandedSchema {
            name: SchemaName::try_new(raw.name())?,
            extends: raw.extends().cloned(),
            excludes: raw.excludes().to_vec(),
            properties,
        })
    }

    /// Resolve a single raw property entry into a validated [`Property`].
    ///
    /// Returns tuple of (`PropertyName`, Property) for `HashMap` insertion.
    fn expand_property(
        &self,
        name: &PropertyName,
        entry: &RawProperty,
    ) -> Result<(PropertyName, Property), SchemaError> {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Match ergonomics is used for cleaner code despite \
                      reference mismatch"
        )]
        match entry {
            RawProperty::Inline(inline) => {
                let spec = inline.spec.clone().try_into()?;
                let optionality = Optionality::from(inline.required);
                let multiplicity = Multiplicity::from(inline.multi);
                Ok((
                    name.clone(),
                    Property::new(
                        PropertyId::new(),
                        name.clone(),
                        optionality,
                        multiplicity,
                        spec,
                    ),
                ))
            }

            RawProperty::Ref(ref_entry) => {
                // Use pre-extracted target_name from RawPropertyRefPath (no
                // parsing needed)
                let bank_name = ref_entry.ref_path.target_name();

                // Look up base property in bank
                let base = self.bank.get(bank_name).ok_or_else(|| {
                    SchemaError::PropertyRef(
                        super::error::PropertyRefError::NotFound {
                            reference: ref_entry.ref_path.as_str().into(),
                        },
                    )
                })?;

                // Apply overrides using Resolver
                let prop = Resolver::from_bank_ref(base, ref_entry)?;

                // Return with the property name from the ref_path
                Ok((bank_name.clone(), prop))
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[expect(
    clippy::unwrap_in_result,
    reason = "Test fixtures use expect/unwrap for simplicity where failure \
              indicates a bug in the test setup"
)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures before test sub-modules for \
              readability"
)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::schema::{
        aggregate::SchemaId,
        bank::PropertyBank,
        property::{Multiplicity, Optionality, PropertyId, PropertyName},
        property_spec::{BoolSpec, PropertySpec},
        raw::{
            RawSchema,
            property::{RawProperty, RawPropertyInline},
            property_spec::{RawBoolSpec, RawPropertySpec},
        },
    };

    mod fixtures {
        use super::*;

        pub fn bool_property(name: &str) -> Result<Property, SchemaError> {
            Ok(Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID),
                PropertyName::try_new(name)?,
                Optionality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            ))
        }

        pub fn bank_with(prop: Property) -> Result<PropertyBank, SchemaError> {
            let mut bank = PropertyBank::new();
            bank.register(prop)?;
            Ok(bank)
        }

        pub fn inline_bool_entry() -> RawProperty {
            RawProperty::Inline(RawPropertyInline {
                required: true,
                multi: false,
                spec: RawPropertySpec::Bool(RawBoolSpec),
            })
        }

        pub fn ref_entry(ref_path: &str) -> RawProperty {
            // RawPropertyRefPath validates during deserialization, so we use
            // JSON
            let json = format!(r#"{{"$ref": "{ref_path}"}}"#);
            serde_json::from_str(&json)
                .expect("Test fixture should create valid RawPropertyRef")
        }
    }

    const TEST_PROPERTY_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0F01);

    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test functions use assert! macros which may panic; this is \
                  standard test practice"
    )]
    mod expand_property {
        use super::*;

        #[test]
        fn inline_bool_resolves_correctly() -> Result<(), SchemaError> {
            let bank = PropertyBank::new();
            let expander = RefExpander::new(&bank);
            let name = PropertyName::try_new("flag")?;
            let entry = fixtures::inline_bool_entry();
            let (_, prop) = expander.expand_property(&name, &entry)?;
            assert_eq!(prop.name().as_str(), "flag");
            assert_eq!(prop.optionality(), Optionality::Required);
            assert_eq!(prop.multiplicity(), Multiplicity::Single);
            Ok(())
        }

        #[test]
        fn ref_resolves_from_bank() -> Result<(), SchemaError> {
            let base = fixtures::bool_property("status")?;
            let bank = fixtures::bank_with(base)?;
            let expander = RefExpander::new(&bank);
            let name = PropertyName::try_new("status")?;
            let entry = fixtures::ref_entry("property_bank#/status");
            let (_, prop) = expander.expand_property(&name, &entry)?;
            assert_eq!(prop.name().as_str(), "status");
            Ok(())
        }

        #[test]
        fn ref_overrides_optionality_and_multiplicity()
        -> Result<(), SchemaError> {
            let base = fixtures::bool_property("status")?;
            let bank = fixtures::bank_with(base)?;
            let expander = RefExpander::new(&bank);
            let json = r#"{
                "$ref": "property_bank#/status",
                "required": false,
                "multi": true
            }"#;
            let entry: RawProperty = serde_json::from_str(json)
                .expect("Valid ref with overrides should deserialize");
            let name = PropertyName::try_new("status")?;
            let (_, prop) = expander.expand_property(&name, &entry)?;
            assert_eq!(prop.optionality(), Optionality::Optional);
            assert_eq!(prop.multiplicity(), Multiplicity::Many);
            Ok(())
        }

        #[test]
        fn ref_type_mismatch_returns_error() {
            let base =
                fixtures::bool_property("status").expect("valid property");
            let bank = fixtures::bank_with(base).expect("valid bank");
            let expander = RefExpander::new(&bank);
            let json = r#"{
                "$ref": "property_bank#/status",
                "min": 0.0
            }"#;
            let entry: RawProperty = serde_json::from_str(json)
                .expect("Valid ref with number override should deserialize");
            let name = PropertyName::try_new("status").expect("valid name");
            let result = expander.expand_property(&name, &entry);
            assert!(
                matches!(
                    result,
                    Err(SchemaError::PropertyRef(
                        crate::schema::error::PropertyRefError::TypeMismatch { .. }
                    ))
                ),
                "Expected PropertyRef::TypeMismatch, got: {result:?}"
            );
        }

        #[test]
        fn ref_missing_in_bank_returns_error() {
            let bank = PropertyBank::new();
            let expander = RefExpander::new(&bank);
            let entry = fixtures::ref_entry("property_bank#/missing");
            let name = PropertyName::try_new("missing").expect("valid name");
            let result = expander.expand_property(&name, &entry);
            assert!(
                matches!(
                    result,
                    Err(SchemaError::PropertyRef(
                        crate::schema::error::PropertyRefError::NotFound { .. }
                    ))
                ),
                "Expected PropertyRef::NotFound, got: {result:?}"
            );
        }
    }

    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test functions use assert! macros; standard test practice"
    )]
    #[expect(
        clippy::indexing_slicing,
        reason = "Tests access known-fixed indices for clarity; bounds \
                  guaranteed by test setup"
    )]
    mod expand_all {
        use super::*;

        #[test]
        fn empty_batch_returns_empty() -> Result<(), SchemaError> {
            let bank = PropertyBank::new();
            let ref_expander = RefExpander::new(&bank);
            let result = ref_expander.expand_all(vec![])?;
            assert!(result.is_empty());
            Ok(())
        }

        #[test]
        fn properties_sorted_by_name() -> Result<(), SchemaError> {
            let bank = PropertyBank::new();
            let ref_expander = RefExpander::new(&bank);

            // Use JSON deserialization to construct RawSchema
            let raw_json = serde_json::json!({
                "$version": "1.0",
                "properties": {
                    "z": { "type": "bool" },
                    "a": { "type": "bool" }
                }
            });
            let raw = serde_json::from_value::<RawSchema>(raw_json)
                .expect("valid schema JSON")
                .with_name("test".into());

            let id = SchemaId::new();
            let result = ref_expander.expand_all(vec![(id, raw)])?;
            let expanded_schemas = &result[0].1;
            assert_eq!(
                expanded_schemas.properties.get("a").map(|p| p.name().as_str()),
                Some("a")
            );
            assert_eq!(
                expanded_schemas.properties.get("z").map(|p| p.name().as_str()),
                Some("z")
            );
            Ok(())
        }

        #[test]
        fn extends_and_excludes_carried_forward() -> Result<(), SchemaError> {
            let bank = PropertyBank::new();
            let ref_expander = RefExpander::new(&bank);

            // Use JSON deserialization to construct RawSchema
            let raw_json = serde_json::json!({
                "$version": "1.0",
                "extends": "parent",
                "excludes": ["old-prop"],
                "properties": {}
            });
            let raw = serde_json::from_value::<RawSchema>(raw_json)
                .expect("valid schema JSON")
                .with_name("child".into());

            let id = SchemaId::new();
            let result = ref_expander.expand_all(vec![(id, raw)])?;
            let expanded_schemas = &result[0].1;
            assert_eq!(
                expanded_schemas
                    .extends
                    .as_ref()
                    .map(std::convert::AsRef::as_ref),
                Some("parent")
            );
            assert_eq!(expanded_schemas.excludes.len(), 1);
            assert_eq!(expanded_schemas.excludes[0].as_ref(), "old-prop");
            Ok(())
        }
    }
}
