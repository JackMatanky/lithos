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
//! Resolver
//! ```
//!
//! After `RefExpander` completes, the [`PropertyBank`] is no longer
//! referenced anywhere in the pipeline.
//!
//! # Design
//!
//! - **Input**: stale `Vec<(SchemaId, RawSchema)>` + `&PropertyBank`
//! - **Output**: `Vec<(SchemaId, RefExpandedSchema)>`
//! - Properties in each `RefExpandedSchema` are **sorted by name** so
//!   downstream components (`Extender`, `Resolver`) can use two-pointer merges
//!   without re-sorting.

use super::{
    bank::PropertyBank,
    error::SchemaError,
    property::{
        BankPropertyRef, Multiplicity, Optionality, Property, PropertyId,
        PropertyName,
    },
    property_spec::PropertySpec,
    raw::{
        RawSchema,
        property::{RawProperty, RawPropertyRef},
    },
};
use crate::schema::aggregate::SchemaId;

// ─────────────────────────────────────────────────────────────────────────────
//  RefExpandedSchema
// ─────────────────────────────────────────────────────────────────────────────

/// A raw schema with all `$ref` pointers resolved against the property bank.
///
/// Properties are sorted by name for efficient merging by `Extender` and
/// `Resolver`.
///
/// **Internal API**: This type is public solely for benchmarking purposes.
/// Do not depend on it in production code - use `Loader` instead.
#[doc(hidden)]
#[derive(Clone)]
#[non_exhaustive]
pub struct RefExpandedSchema {
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
                let expanded = self.expand_schema(raw)?;
                Ok((id, expanded))
            })
            .collect()
    }

    /// Expand references for a single [`RawSchema`].
    fn expand_schema(
        &self,
        raw: RawSchema,
    ) -> Result<RefExpandedSchema, SchemaError> {
        // Collect and sort entries by name for deterministic output.
        let mut entries: Vec<(Box<str>, RawProperty)> =
            raw.properties.into_iter().collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        let mut properties = Vec::with_capacity(entries.len());
        for (prop_name, entry) in entries {
            properties.push(self.expand_property(&prop_name, entry)?);
        }

        Ok(RefExpandedSchema {
            name: raw.name,
            extends: raw.extends,
            excludes: raw.excludes,
            properties,
        })
    }

    /// Resolve a single raw property entry into a validated [`Property`].
    fn expand_property(
        &self,
        name: &str,
        entry: RawProperty,
    ) -> Result<Property, SchemaError> {
        match entry {
            RawProperty::Inline(inline) => {
                let prop_name = PropertyName::try_new(name)?;
                let spec = inline.spec.try_into()?;
                let optionality = Optionality::from(inline.required);
                let multiplicity = Multiplicity::from(inline.multi);
                Ok(Property::new(
                    PropertyId::new(),
                    prop_name,
                    optionality,
                    multiplicity,
                    spec,
                ))
            }

            RawProperty::Ref(ref_entry) => {
                Self::apply_ref_overrides(self.bank, &ref_entry)
            }
        }
    }

    /// Resolve a `$ref` entry: look up the base property in the bank, then
    /// apply any `required`/`multi` and type-specific overrides.
    ///
    /// The property name is extracted from the `ref_path` (e.g.,
    /// `property_bank#/original_name`).
    ///
    /// # Errors
    ///
    /// Returns [`SchemaError::PropertyRefNotFound`] if the referenced property
    /// is absent from the bank, or [`SchemaError::PropertyTypeMismatch`] if
    /// any override field is incompatible with the base property type (R-10).
    fn apply_ref_overrides(
        bank: &PropertyBank,
        ref_entry: &RawPropertyRef,
    ) -> Result<Property, SchemaError> {
        // Extract property name from ref_path (e.g., "property_bank#/date" ->
        // "date")
        let prop_ref = BankPropertyRef::try_from(ref_entry.ref_path.as_ref())?;
        let bank_name = prop_ref.name();

        let base = bank.get(bank_name).ok_or_else(|| {
            SchemaError::PropertyRefNotFound(ref_entry.ref_path.to_string())
        })?;

        let optionality =
            ref_entry.required.map_or(base.optionality(), Optionality::from);
        let multiplicity =
            ref_entry.multi.map_or(base.multiplicity(), Multiplicity::from);

        let spec = Self::apply_spec_overrides(base.spec(), ref_entry)?;

        // Use the bank property name as the schema property name
        Ok(Property::new(
            base.id(),
            bank_name.clone(),
            optionality,
            multiplicity,
            spec,
        ))
    }

    /// Apply type-specific spec overrides, rejecting type changes (R-10).
    ///
    /// Returns [`SchemaError::PropertyTypeMismatch`] if any override field
    /// targets a type incompatible with the base spec type.
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on &PropertySpec with value patterns is idiomatic"
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
        property::{Multiplicity, Optionality, PropertyId, PropertyName},
        property_spec::{BoolSpec, PropertySpec},
        raw::{
            RawSchema,
            property::{RawProperty, RawPropertyInline, RawPropertyRef},
            property_spec::{
                RawBoolSpec, RawDateSpec, RawFileSpec, RawNumberSpec,
                RawPropertySpec, RawStringSpec,
            },
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
    mod expand_property {
        use super::*;

        #[test]
        fn inline_bool_resolves_correctly() -> Result<(), SchemaError> {
            let bank = PropertyBank::new();
            let expander = RefExpander::new(&bank);
            let prop = expander
                .expand_property("flag", fixtures::inline_bool_entry())?;
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
            let prop = expander.expand_property(
                "status",
                fixtures::ref_entry("property_bank#/status"),
            )?;
            assert_eq!(prop.name().as_str(), "status");
            Ok(())
        }

        #[test]
        fn ref_overrides_optionality_and_multiplicity()
        -> Result<(), SchemaError> {
            let base = fixtures::bool_property("status")?;
            let bank = fixtures::bank_with(base)?;
            let expander = RefExpander::new(&bank);
            let entry = RawProperty::Ref(RawPropertyRef {
                ref_path: "property_bank#/status".into(),
                required: Some(false),
                multi: Some(true),
                number: RawNumberSpec::default(),
                string: RawStringSpec::default(),
                date: RawDateSpec::default(),
                file: RawFileSpec::default(),
            });
            let prop = expander.expand_property("status", entry)?;
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
            let result = expander.expand_property("status", entry);
            assert!(
                matches!(result, Err(SchemaError::PropertyTypeMismatch { .. })),
                "Expected PropertyTypeMismatch, got: {result:?}"
            );
        }

        #[test]
        fn ref_missing_in_bank_returns_error() {
            let bank = PropertyBank::new();
            let expander = RefExpander::new(&bank);
            let entry = fixtures::ref_entry("property_bank#/missing");
            let result = expander.expand_property("missing", entry);
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
            let mut props = HashMap::new();
            props.insert("z".into(), fixtures::inline_bool_entry());
            props.insert("a".into(), fixtures::inline_bool_entry());
            let raw = RawSchema {
                version: crate::schema::raw::RawSchemaVersion::SUPPORTED.into(),
                name: "test".into(),
                extends: None,
                excludes: Vec::new(),
                properties: props,
                metadata: crate::schema::raw::RawSchemaMetadata::default(),
            };
            let id = SchemaId::new();
            let result = ref_expander.expand_all(vec![(id, raw)])?;
            let expanded_schemas = &result[0].1;
            assert_eq!(expanded_schemas.properties[0].name().as_str(), "a");
            assert_eq!(expanded_schemas.properties[1].name().as_str(), "z");
            Ok(())
        }

        #[test]
        fn extends_and_excludes_carried_forward() -> Result<(), SchemaError> {
            let bank = PropertyBank::new();
            let ref_expander = RefExpander::new(&bank);
            let raw = RawSchema {
                version: crate::schema::raw::RawSchemaVersion::SUPPORTED.into(),
                name: "child".into(),
                extends: Some("parent".into()),
                excludes: vec!["old-prop".into()],
                properties: HashMap::new(),
                metadata: crate::schema::raw::RawSchemaMetadata::default(),
            };
            let id = SchemaId::new();
            let result = ref_expander.expand_all(vec![(id, raw)])?;
            let expanded_schemas = &result[0].1;
            assert_eq!(expanded_schemas.extends.as_deref(), Some("parent"));
            assert_eq!(expanded_schemas.excludes.len(), 1);
            assert_eq!(expanded_schemas.excludes[0].as_ref(), "old-prop");
            Ok(())
        }
    }
}
