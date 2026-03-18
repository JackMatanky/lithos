//! Property-level conflict resolution and override logic.
//!
//! Handles resolving conflicts between property definitions and applying
//! overrides while maintaining type safety.
//!
//! ## Use Cases
//!
//! ### Expander: PropertyBank Reference Overrides
//! ```ignore
//! // Schema file has: { "$ref": "property_bank#/title", "required": false }
//! // Bank has:        Property { name: "title", required: true, ... }
//! // Result:          Property { name: "title", required: false, ... }
//! ```
//!
//! ### Merger: Schema Inheritance Overrides
//! ```ignore
//! // Parent schema:  Property { name: "title", required: true, ... }
//! // Child schema:   Property { name: "title", required: false, ... }
//! // Result:         Property { name: "title", required: false, ... }
//! ```

use super::{
    error::SchemaError,
    property::{Multiplicity, Optionality, Property},
    property_spec::PropertySpec,
    raw::property::RawPropertyRef,
};

/// Resolves property-level conflicts and applies overrides.
///
/// Stateless utility for property override logic, ensuring type safety
/// and validation when combining property definitions from different sources.
#[non_exhaustive]
pub struct Resolver;

impl Resolver {
    /// Resolve optionality override.
    ///
    /// # Rules
    /// - If override is `Some`, use it
    /// - Otherwise, use base optionality
    ///
    /// # Examples
    /// ```ignore
    /// // Override required → optional
    /// let base = Optionality::Required;
    /// let override_val = Some(false);
    /// let result = Resolver::resolve_optionality(base, override_val);
    /// assert_eq!(result, Optionality::Optional);
    ///
    /// // No override → keep base
    /// let result = Resolver::resolve_optionality(base, None);
    /// assert_eq!(result, Optionality::Required);
    /// ```
    #[inline]
    #[must_use]
    pub fn resolve_optionality(
        base: Optionality,
        override_required: Option<bool>,
    ) -> Optionality {
        override_required.map_or(base, Optionality::from)
    }

    /// Resolve multiplicity override.
    ///
    /// # Rules
    /// - If override is `Some`, use it
    /// - Otherwise, use base multiplicity
    #[inline]
    #[must_use]
    pub fn resolve_multiplicity(
        base: Multiplicity,
        override_multi: Option<bool>,
    ) -> Multiplicity {
        override_multi.map_or(base, Multiplicity::from)
    }

    /// Resolve property spec overrides (type-specific constraints).
    ///
    /// # Rules
    /// - Cannot change property type (bool → number rejected)
    /// - Can override type-specific constraints (min/max, pattern, etc.)
    ///
    /// # Errors
    /// Returns `SchemaError::PropertyTypeMismatch` if override attempts
    /// to change the property type.
    ///
    /// # Examples
    /// ```ignore
    /// // Valid: Override number constraints
    /// let base = PropertySpec::Number(NumberSpec { min: None, max: None });
    /// let overrides = RawPropertyRef { number: RawNumberSpec { min: Some(0.0f64), .. }, .. };
    /// let result = Resolver::resolve_spec(&base, &overrides)?;
    /// // Result: NumberSpec { min: Some(0.0f64), max: None }
    ///
    /// // Invalid: Attempt to change type
    /// let base = PropertySpec::Bool(BoolSpec);
    /// let overrides = RawPropertyRef { number: RawNumberSpec { min: Some(0.0f64), .. }, .. };
    /// let result = Resolver::resolve_spec(&base, &overrides);
    /// // Result: Err(PropertyTypeMismatch { expected: "bool", actual: "number" })
    /// ```
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on &PropertySpec with value patterns is idiomatic"
    )]
    #[inline]
    pub fn resolve_spec(
        base: &PropertySpec,
        ref_entry: &RawPropertyRef,
    ) -> Result<PropertySpec, SchemaError> {
        // Detect which type-specific overrides are present
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

    /// Apply all overrides from a property bank reference.
    ///
    /// Used by Expander when resolving `$ref` entries.
    ///
    /// # Errors
    /// Returns error if type mismatch occurs during spec override.
    #[inline]
    pub fn resolve_from_bank_ref(
        bank_property: &Property,
        ref_entry: &RawPropertyRef,
    ) -> Result<Property, SchemaError> {
        let optionality = Self::resolve_optionality(
            bank_property.optionality(),
            ref_entry.required,
        );
        let multiplicity = Self::resolve_multiplicity(
            bank_property.multiplicity(),
            ref_entry.multi,
        );
        let spec = Self::resolve_spec(bank_property.spec(), ref_entry)?;

        Ok(Property::new(
            bank_property.id(),
            bank_property.name().clone(),
            optionality,
            multiplicity,
            spec,
        ))
    }

    /// Apply child property override to parent property.
    ///
    /// Used by Merger during schema inheritance. In schema inheritance,
    /// child property completely replaces parent property (no field merging).
    ///
    /// # Rules
    /// - Child property wins entirely
    /// - Child can change optionality, multiplicity, spec, and type
    /// - Child's `PropertyId` is used (new property instance)
    ///
    /// # Examples
    /// ```ignore
    /// // Parent: title (required, single, String[max=100])
    /// // Child:  title (optional, multi, String[max=200])
    /// // Result: title (optional, multi, String[max=200]) - child wins completely
    /// ```
    #[inline]
    #[must_use]
    pub fn resolve_child_override(
        _parent: &Property,
        child: &Property,
    ) -> Property {
        // In schema inheritance, child completely replaces parent
        // No merging of fields - child wins entirely
        child.clone()
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
    use super::*;
    use crate::schema::{
        property::{Multiplicity, Optionality, PropertyId, PropertyName},
        property_spec::{BoolSpec, NumberSpec, PropertySpec, StringSpec},
        raw::{
            property::RawPropertyRef,
            property_spec::{
                RawDateSpec, RawFileSpec, RawNumberSpec, RawOptions,
                RawStringSpec,
            },
        },
    };

    // ── Fixtures ────────────────────────────────────────────────────────────

    mod fixtures {
        use super::*;

        pub fn bool_property(name: &str) -> Property {
            Property::new(
                PropertyId::new(),
                PropertyName::try_new(name).unwrap(),
                Optionality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec),
            )
        }

        pub fn number_property(
            name: &str,
            min: Option<f64>,
            max: Option<f64>,
        ) -> Property {
            Property::new(
                PropertyId::new(),
                PropertyName::try_new(name).unwrap(),
                Optionality::Required,
                Multiplicity::Single,
                PropertySpec::Number(
                    NumberSpec::try_new(min, max, None).unwrap(),
                ),
            )
        }

        pub fn ref_entry(ref_path: &str) -> RawPropertyRef {
            RawPropertyRef {
                ref_path: ref_path.into(),
                required: None,
                multi: None,
                number: RawNumberSpec::default(),
                string: RawStringSpec::default(),
                date: RawDateSpec::default(),
                file: RawFileSpec::default(),
            }
        }

        pub fn ref_with_overrides(
            ref_path: &str,
            required: Option<bool>,
            multi: Option<bool>,
        ) -> RawPropertyRef {
            RawPropertyRef {
                ref_path: ref_path.into(),
                required,
                multi,
                number: RawNumberSpec::default(),
                string: RawStringSpec::default(),
                date: RawDateSpec::default(),
                file: RawFileSpec::default(),
            }
        }
    }

    // ── Optionality Resolution ──────────────────────────────────────────────

    mod resolve_optionality {
        use super::*;

        #[test]
        fn uses_override_when_present() {
            let result = Resolver::resolve_optionality(
                Optionality::Required,
                Some(false),
            );
            assert_eq!(result, Optionality::Optional);
        }

        #[test]
        fn uses_base_when_no_override() {
            let result =
                Resolver::resolve_optionality(Optionality::Optional, None);
            assert_eq!(result, Optionality::Optional);
        }

        #[test]
        fn can_make_required_optional() {
            let result = Resolver::resolve_optionality(
                Optionality::Required,
                Some(false),
            );
            assert_eq!(result, Optionality::Optional);
        }

        #[test]
        fn can_make_optional_required() {
            let result = Resolver::resolve_optionality(
                Optionality::Optional,
                Some(true),
            );
            assert_eq!(result, Optionality::Required);
        }
    }

    // ── Multiplicity Resolution ─────────────────────────────────────────────

    mod resolve_multiplicity {
        use super::*;

        #[test]
        fn uses_override_when_present() {
            let result = Resolver::resolve_multiplicity(
                Multiplicity::Single,
                Some(true),
            );
            assert_eq!(result, Multiplicity::Many);
        }

        #[test]
        fn uses_base_when_no_override() {
            let result =
                Resolver::resolve_multiplicity(Multiplicity::Many, None);
            assert_eq!(result, Multiplicity::Many);
        }

        #[test]
        fn can_make_single_many() {
            let result = Resolver::resolve_multiplicity(
                Multiplicity::Single,
                Some(true),
            );
            assert_eq!(result, Multiplicity::Many);
        }

        #[test]
        fn can_make_many_single() {
            let result =
                Resolver::resolve_multiplicity(Multiplicity::Many, Some(false));
            assert_eq!(result, Multiplicity::Single);
        }
    }

    // ── Spec Resolution (Type Safety) ───────────────────────────────────────

    mod resolve_spec {
        use super::*;

        #[test]
        fn bool_rejects_number_override() {
            let base = PropertySpec::Bool(BoolSpec);
            let ref_entry = RawPropertyRef {
                ref_path: "property_bank#/test".into(),
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
            };

            let result = Resolver::resolve_spec(&base, &ref_entry);
            let err = result.unwrap_err();
            assert!(matches!(err, SchemaError::PropertyTypeMismatch { .. }));
        }

        #[test]
        fn bool_rejects_string_override() {
            let base = PropertySpec::Bool(BoolSpec);
            let ref_entry = RawPropertyRef {
                ref_path: "property_bank#/test".into(),
                required: None,
                multi: None,
                number: RawNumberSpec::default(),
                string: RawStringSpec {
                    options: Some(RawOptions::List(vec!["a".into()])),
                    pattern: None,
                },
                date: RawDateSpec::default(),
                file: RawFileSpec::default(),
            };

            let result = Resolver::resolve_spec(&base, &ref_entry);
            let _err = result.unwrap_err();
        }

        #[test]
        fn number_accepts_number_override() {
            let base = PropertySpec::Number(NumberSpec::default());
            let ref_entry = RawPropertyRef {
                ref_path: "property_bank#/test".into(),
                required: None,
                multi: None,
                number: RawNumberSpec {
                    min: Some(0.0f64),
                    max: Some(100.0f64),
                    step: None,
                },
                string: RawStringSpec::default(),
                date: RawDateSpec::default(),
                file: RawFileSpec::default(),
            };

            let result = Resolver::resolve_spec(&base, &ref_entry);
            assert!(result.is_ok());
            // Just verify it returns a Number spec (internal structure is
            // private)
            assert!(matches!(result.unwrap(), PropertySpec::Number(_)));
        }

        #[test]
        fn number_rejects_string_override() {
            let base = PropertySpec::Number(NumberSpec::default());
            let ref_entry = RawPropertyRef {
                ref_path: "property_bank#/test".into(),
                required: None,
                multi: None,
                number: RawNumberSpec::default(),
                string: RawStringSpec {
                    options: Some(RawOptions::List(vec!["a".into()])),
                    pattern: None,
                },
                date: RawDateSpec::default(),
                file: RawFileSpec::default(),
            };

            let result = Resolver::resolve_spec(&base, &ref_entry);
            let _err = result.unwrap_err();
        }

        #[test]
        fn string_accepts_string_override() {
            let base = PropertySpec::String(StringSpec::default());
            let ref_entry = RawPropertyRef {
                ref_path: "property_bank#/test".into(),
                required: None,
                multi: None,
                number: RawNumberSpec::default(),
                string: RawStringSpec {
                    options: Some(RawOptions::List(vec!["valid".into()])),
                    pattern: None,
                },
                date: RawDateSpec::default(),
                file: RawFileSpec::default(),
            };

            let result = Resolver::resolve_spec(&base, &ref_entry);
            assert!(result.is_ok());
            let spec = result.unwrap();
            assert!(
                matches!(spec, PropertySpec::String(_)),
                "Expected String spec, got {spec:?}"
            );
            if let PropertySpec::String(spec) = spec {
                assert!(spec.options().is_some());
            }
        }
    }

    // ── Full Property Resolution (Bank Ref) ─────────────────────────────────

    mod resolve_from_bank_ref {
        use super::*;

        #[test]
        fn preserves_base_when_no_overrides() {
            let base = fixtures::bool_property("test");
            let ref_entry = fixtures::ref_entry("property_bank#/test");

            let result = Resolver::resolve_from_bank_ref(&base, &ref_entry);
            assert!(result.is_ok());
            let prop = result.unwrap();
            assert_eq!(prop.name(), base.name());
            assert_eq!(prop.optionality(), base.optionality());
            assert_eq!(prop.multiplicity(), base.multiplicity());
        }

        #[test]
        fn applies_optionality_override() {
            let base = fixtures::bool_property("test"); // Required by default
            let ref_entry = fixtures::ref_with_overrides(
                "property_bank#/test",
                Some(false), // Override to optional
                None,
            );

            let result = Resolver::resolve_from_bank_ref(&base, &ref_entry);
            assert!(result.is_ok());
            let prop = result.unwrap();
            assert_eq!(prop.optionality(), Optionality::Optional);
        }

        #[test]
        fn applies_multiplicity_override() {
            let base = fixtures::bool_property("test"); // Single by default
            let ref_entry = fixtures::ref_with_overrides(
                "property_bank#/test",
                None,
                Some(true), // Override to multi
            );

            let result = Resolver::resolve_from_bank_ref(&base, &ref_entry);
            assert!(result.is_ok());
            let prop = result.unwrap();
            assert_eq!(prop.multiplicity(), Multiplicity::Many);
        }

        #[test]
        fn applies_all_overrides() {
            let base = fixtures::bool_property("test");
            let ref_entry = fixtures::ref_with_overrides(
                "property_bank#/test",
                Some(false),
                Some(true),
            );

            let result = Resolver::resolve_from_bank_ref(&base, &ref_entry);
            assert!(result.is_ok());
            let prop = result.unwrap();
            assert_eq!(prop.optionality(), Optionality::Optional);
            assert_eq!(prop.multiplicity(), Multiplicity::Many);
        }

        #[test]
        fn rejects_type_mismatch() {
            let base = fixtures::bool_property("test");
            let ref_entry = RawPropertyRef {
                ref_path: "property_bank#/test".into(),
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
            };

            let result = Resolver::resolve_from_bank_ref(&base, &ref_entry);
            let _err = result.unwrap_err();
        }
    }

    // ── Child Override (Schema Inheritance) ─────────────────────────────────

    mod resolve_child_override {
        use super::*;

        #[test]
        fn child_completely_replaces_parent() {
            let parent = fixtures::bool_property("title");
            let child = fixtures::number_property(
                "title",
                Some(0.0f64),
                Some(100.0f64),
            );

            let result = Resolver::resolve_child_override(&parent, &child);

            // Child wins completely
            assert_eq!(result.name(), child.name());
            assert_eq!(result.id(), child.id());
            assert!(matches!(result.spec(), PropertySpec::Number(_)));
        }

        #[test]
        fn child_id_is_preserved() {
            let parent = fixtures::bool_property("test");
            let child = fixtures::bool_property("test");

            let result = Resolver::resolve_child_override(&parent, &child);

            // Child's ID is used
            assert_eq!(result.id(), child.id());
            assert_ne!(result.id(), parent.id());
        }
    }
}
