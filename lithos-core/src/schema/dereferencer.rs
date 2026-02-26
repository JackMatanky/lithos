//! `Dereferencer` — resolves raw property bank `$ref` pointers into
//! validated [`Property`] values.
//!
//! # Pipeline position
//!
//! ```text
//! Ingestor → Vec<(SchemaId, RawSchema)>
//! Dereferencer  ← here
//! → Vec<(SchemaId, DereferencedSchema)>
//! Extender
//! Resolver
//! ```
//!
//! After `Dereferencer` completes, the [`PropertyBank`] is no longer
//! referenced anywhere in the pipeline.
//!
//! # Design
//!
//! - **Input**: stale `Vec<(SchemaId, RawSchema)>` + `&PropertyBank`
//! - **Output**: `Vec<(SchemaId, DereferencedSchema)>`
//! - Properties in each `DereferencedSchema` are **sorted by name** so
//!   downstream components (`Extender`, `Resolver`) can use two-pointer merges
//!   without re-sorting.

use super::{
    bank::PropertyBank,
    error::SchemaError,
    property::{
        Cardinality, Multiplicity, Property, PropertyId, PropertyName,
        PropertyRef,
    },
    property_spec::PropertySpec,
    raw::{RawProperty, RawPropertyRef, RawSchema},
};
use crate::schema::aggregate::SchemaId;

// ─────────────────────────────────────────────────────────────────────────────
//  DereferencedSchema
// ─────────────────────────────────────────────────────────────────────────────

/// A raw schema with all `$ref` pointers resolved against the property bank.
///
/// Properties are sorted by name for efficient merging by `Extender` and
/// `Resolver`.
///
/// Visibility is `pub(crate)` because `DereferencedSchema` is an internal
/// pipeline type — it is never exposed to callers of the public API.
pub(crate) struct DereferencedSchema {
    /// Schema name string (carried forward from `RawSchema`).
    pub name: Box<str>,
    /// Optional parent schema name (carried forward from `RawSchema.extends`).
    pub extends: Option<Box<str>>,
    /// Property names to exclude from the parent schema.
    pub excludes: Vec<Box<str>>,
    /// Fully resolved and sorted own properties.
    pub properties: Vec<Property>,
}

// ─────────────────────────────────────────────────────────────────────────────
//  Dereferencer
// ─────────────────────────────────────────────────────────────────────────────

/// Resolves raw `$ref` property pointers against the [`PropertyBank`] and
/// validates inline property definitions.
///
/// Holds a shared reference to the bank so callers can pre-build the bank
/// once and reuse the dereferencer across multiple schemas.
pub(crate) struct Dereferencer<'bank> {
    bank: &'bank PropertyBank,
}

impl<'bank> Dereferencer<'bank> {
    /// Create a new `Dereferencer` bound to the given [`PropertyBank`].
    #[inline]
    #[must_use]
    pub(crate) const fn new(bank: &'bank PropertyBank) -> Self {
        Self {
            bank,
        }
    }

    /// Dereference a batch of stale raw schemas.
    ///
    /// For each `(SchemaId, RawSchema)` pair, all properties are resolved
    /// against the bank and the results are returned in the same order.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaError`] if any property fails validation or a `$ref`
    /// path cannot be resolved.
    #[inline]
    pub(crate) fn deref(
        &self,
        schemas: Vec<(SchemaId, RawSchema)>,
    ) -> Result<Vec<(SchemaId, DereferencedSchema)>, SchemaError> {
        schemas
            .into_iter()
            .map(|(id, raw)| {
                let derefed = self.deref_one(raw)?;
                Ok((id, derefed))
            })
            .collect()
    }

    /// Dereference a single [`RawSchema`].
    fn deref_one(
        &self,
        raw: RawSchema,
    ) -> Result<DereferencedSchema, SchemaError> {
        // Collect and sort entries by name for deterministic output.
        let mut entries: Vec<(Box<str>, RawProperty)> =
            raw.properties.into_iter().collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        let mut properties = Vec::with_capacity(entries.len());
        for (prop_name, entry) in entries {
            properties.push(self.deref_entry(&prop_name, entry)?);
        }

        Ok(DereferencedSchema {
            name: raw.name,
            extends: raw.extends,
            excludes: raw.excludes,
            properties,
        })
    }

    /// Resolve a single raw property entry into a validated [`Property`].
    fn deref_entry(
        &self,
        name: &str,
        entry: RawProperty,
    ) -> Result<Property, SchemaError> {
        match entry {
            RawProperty::Inline(inline) => {
                let prop_name = PropertyName::new(name)?;
                let spec = inline.spec.try_into_validated()?;
                let cardinality = Cardinality::from(inline.required);
                let multiplicity = Multiplicity::from(inline.multi);
                Property::new(
                    PropertyId::new(),
                    prop_name,
                    cardinality,
                    multiplicity,
                    spec,
                )
            }

            RawProperty::Ref(ref_entry) => {
                Self::apply_ref_overrides(self.bank, name, &ref_entry)
            }
        }
    }

    /// Resolve a `$ref` entry: look up the base property in the bank, then
    /// apply any `required`/`multi` and type-specific overrides.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaError::PropertyRefNotFound`] if the referenced property
    /// is absent from the bank, or [`SchemaError::PropertyTypeMismatch`] if
    /// any override field is incompatible with the base property type (R-10).
    fn apply_ref_overrides(
        bank: &PropertyBank,
        name: &str,
        ref_entry: &RawPropertyRef,
    ) -> Result<Property, SchemaError> {
        let prop_ref = PropertyRef::try_from(ref_entry.ref_path.as_ref())?;
        let base = bank.get_by_name(prop_ref.name()).ok_or_else(|| {
            SchemaError::PropertyRefNotFound(ref_entry.ref_path.to_string())
        })?;

        let cardinality =
            ref_entry.required.map_or(base.cardinality(), Cardinality::from);
        let multiplicity =
            ref_entry.multi.map_or(base.multiplicity(), Multiplicity::from);

        let spec = Self::apply_spec_overrides(base.spec(), ref_entry)?;

        let prop_name = PropertyName::new(name)?;
        Property::new(base.id(), prop_name, cardinality, multiplicity, spec)
    }

    /// Apply type-specific spec overrides, rejecting type changes (R-10).
    ///
    /// Returns [`SchemaError::PropertyTypeMismatch`] if any override field
    /// targets a type incompatible with the base spec type.
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &PropertySpec are intentional for \
                  readability; dereferencing every arm adds noise"
    )]
    fn apply_spec_overrides(
        base: &PropertySpec,
        ref_entry: &RawPropertyRef,
    ) -> Result<PropertySpec, SchemaError> {
        let has_number = ref_entry.number.min.is_some()
            || ref_entry.number.max.is_some()
            || ref_entry.number.step.is_some();
        let has_string = ref_entry.string.options.is_some()
            || ref_entry.string.pattern.is_some();
        let has_date = ref_entry.date.format.is_some();
        let has_file = ref_entry.file.directory.is_some()
            || ref_entry.file.file_class.is_some();

        match base {
            PropertySpec::Bool(_) => {
                if has_number {
                    return Err(type_mismatch("bool", "number"));
                }
                if has_string {
                    return Err(type_mismatch("bool", "string"));
                }
                if has_date {
                    return Err(type_mismatch("bool", "date"));
                }
                if has_file {
                    return Err(type_mismatch("bool", "file"));
                }
                Ok(base.clone())
            }

            PropertySpec::Number(spec) => {
                if has_string {
                    return Err(type_mismatch("number", "string"));
                }
                if has_date {
                    return Err(type_mismatch("number", "date"));
                }
                if has_file {
                    return Err(type_mismatch("number", "file"));
                }
                Ok(PropertySpec::Number(
                    spec.clone().apply_overrides(&ref_entry.number)?,
                ))
            }

            PropertySpec::String(spec) => {
                if has_number {
                    return Err(type_mismatch("string", "number"));
                }
                if has_date {
                    return Err(type_mismatch("string", "date"));
                }
                if has_file {
                    return Err(type_mismatch("string", "file"));
                }
                Ok(PropertySpec::String(
                    spec.clone().apply_overrides(&ref_entry.string)?,
                ))
            }

            PropertySpec::Date(spec) => {
                if has_number {
                    return Err(type_mismatch("date", "number"));
                }
                if has_string {
                    return Err(type_mismatch("date", "string"));
                }
                if has_file {
                    return Err(type_mismatch("date", "file"));
                }
                Ok(PropertySpec::Date(
                    spec.clone().apply_overrides(&ref_entry.date)?,
                ))
            }

            PropertySpec::File(spec) => {
                if has_number {
                    return Err(type_mismatch("file", "number"));
                }
                if has_string {
                    return Err(type_mismatch("file", "string"));
                }
                if has_date {
                    return Err(type_mismatch("file", "date"));
                }
                Ok(PropertySpec::File(
                    spec.clone().apply_overrides(&ref_entry.file)?,
                ))
            }
        }
    }
}

#[inline]
fn type_mismatch(expected: &str, actual: &str) -> SchemaError {
    SchemaError::PropertyTypeMismatch {
        expected: expected.into(),
        actual: actual.into(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures before test sub-modules for \
              readability"
)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use super::*;
    use crate::schema::{
        aggregate::SchemaId,
        bank::PropertyBank,
        property::{Cardinality, Multiplicity, PropertyId, PropertyName},
        property_spec::{BoolSpec, PropertySpec},
        raw::{
            RawBoolSpec, RawDateSpec, RawFileSpec, RawNumberSpec, RawProperty,
            RawPropertyInline, RawPropertyRef, RawPropertySpec, RawSchema,
            RawStringSpec,
        },
    };

    mod fixtures {
        use super::*;

        pub fn bool_property(name: &str) -> Result<Property, SchemaError> {
            Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID),
                PropertyName::new(name)?,
                Cardinality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            )
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
            RawProperty::Ref(RawPropertyRef {
                ref_path: ref_path.into(),
                required: None,
                multi: None,
                number: RawNumberSpec::default(),
                string: RawStringSpec::default(),
                date: RawDateSpec::default(),
                file: RawFileSpec::default(),
            })
        }
    }

    const TEST_PROPERTY_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0F01);

    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test functions use assert! macros which may panic; this is \
                  standard test practice"
    )]
    mod deref_entry {
        use super::*;

        #[test]
        fn inline_bool_resolves_correctly() -> Result<(), SchemaError> {
            let bank = PropertyBank::new();
            let deref = Dereferencer::new(&bank);
            let prop =
                deref.deref_entry("flag", fixtures::inline_bool_entry())?;
            assert_eq!(prop.name().as_str(), "flag");
            assert_eq!(prop.cardinality(), Cardinality::Required);
            assert_eq!(prop.multiplicity(), Multiplicity::Single);
            Ok(())
        }

        #[test]
        fn ref_resolves_from_bank() -> Result<(), SchemaError> {
            let base = fixtures::bool_property("status")?;
            let bank = fixtures::bank_with(base)?;
            let deref = Dereferencer::new(&bank);
            let prop = deref.deref_entry(
                "status",
                fixtures::ref_entry("property_bank#/status"),
            )?;
            assert_eq!(prop.name().as_str(), "status");
            Ok(())
        }

        #[test]
        fn ref_overrides_cardinality_and_multiplicity()
        -> Result<(), SchemaError> {
            let base = fixtures::bool_property("status")?;
            let bank = fixtures::bank_with(base)?;
            let deref = Dereferencer::new(&bank);
            let entry = RawProperty::Ref(RawPropertyRef {
                ref_path: "property_bank#/status".into(),
                required: Some(false),
                multi: Some(true),
                number: RawNumberSpec::default(),
                string: RawStringSpec::default(),
                date: RawDateSpec::default(),
                file: RawFileSpec::default(),
            });
            let prop = deref.deref_entry("status", entry)?;
            assert_eq!(prop.cardinality(), Cardinality::Optional);
            assert_eq!(prop.multiplicity(), Multiplicity::Many);
            Ok(())
        }

        #[test]
        fn ref_type_mismatch_returns_error() {
            let base =
                fixtures::bool_property("status").expect("valid property");
            let bank = fixtures::bank_with(base).expect("valid bank");
            let deref = Dereferencer::new(&bank);
            let entry = RawProperty::Ref(RawPropertyRef {
                ref_path: "property_bank#/status".into(),
                required: None,
                multi: None,
                number: RawNumberSpec {
                    min: Some(0.0f64),
                    max: None,
                    step: None,
                },
                string: RawStringSpec::default(),
                date: RawDateSpec::default(),
                file: RawFileSpec::default(),
            });
            let result = deref.deref_entry("status", entry);
            assert!(
                matches!(result, Err(SchemaError::PropertyTypeMismatch { .. })),
                "Expected PropertyTypeMismatch, got: {result:?}"
            );
        }

        #[test]
        fn ref_missing_in_bank_returns_error() {
            let bank = PropertyBank::new();
            let deref = Dereferencer::new(&bank);
            let entry = fixtures::ref_entry("property_bank#/missing");
            let result = deref.deref_entry("missing", entry);
            assert!(
                matches!(result, Err(SchemaError::PropertyRefNotFound(_))),
                "Expected PropertyRefNotFound, got: {result:?}"
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
    mod deref_batch {
        use super::*;

        #[test]
        fn empty_batch_returns_empty() -> Result<(), SchemaError> {
            let bank = PropertyBank::new();
            let deref = Dereferencer::new(&bank);
            let result = deref.deref(vec![])?;
            assert!(result.is_empty());
            Ok(())
        }

        #[test]
        fn properties_sorted_by_name() -> Result<(), SchemaError> {
            let bank = PropertyBank::new();
            let deref = Dereferencer::new(&bank);
            let mut props = HashMap::new();
            props.insert("z".into(), fixtures::inline_bool_entry());
            props.insert("a".into(), fixtures::inline_bool_entry());
            let raw = RawSchema {
                version: crate::schema::raw::SCHEMA_VERSION.into(),
                name: "test".into(),
                extends: None,
                excludes: Vec::new(),
                properties: props,
            };
            let id = SchemaId::new();
            let result = deref.deref(vec![(id, raw)])?;
            let derefed = &result[0].1;
            assert_eq!(derefed.properties[0].name().as_str(), "a");
            assert_eq!(derefed.properties[1].name().as_str(), "z");
            Ok(())
        }

        #[test]
        fn extends_and_excludes_carried_forward() -> Result<(), SchemaError> {
            let bank = PropertyBank::new();
            let deref = Dereferencer::new(&bank);
            let raw = RawSchema {
                version: crate::schema::raw::SCHEMA_VERSION.into(),
                name: "child".into(),
                extends: Some("parent".into()),
                excludes: vec!["old-prop".into()],
                properties: HashMap::new(),
            };
            let id = SchemaId::new();
            let result = deref.deref(vec![(id, raw)])?;
            let derefed = &result[0].1;
            assert_eq!(derefed.extends.as_deref(), Some("parent"));
            assert_eq!(derefed.excludes.len(), 1);
            assert_eq!(derefed.excludes[0].as_ref(), "old-prop");
            Ok(())
        }
    }
}
