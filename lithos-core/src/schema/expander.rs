//! `RefExpander` — expands raw property bank `$ref` pointers into
//! validated [`Property`] values.
//!
//! # Pipeline position
//!
//! ```text
//! Ingestor → RawPropertyMap<RawProperty>
//! RefExpander  ← here
//! → PropertyMap
//! Merger
//! ```
//!
//! After `RefExpander` completes, the [`PropertyBank`] is no longer
//! referenced anywhere in the pipeline.
//!
//! # Design
//!
//! - **Input**: `HashMap<PropertyName, RawPropertyRef>` + `&PropertyBank`
//! - **Output**: `PropertyMap`
//! - Expander only handles property bank references; inline properties are
//!   validated elsewhere via `TryFrom` on `Property`.

use std::collections::HashMap;

use super::{
    bank::PropertyBank,
    error::SchemaError,
    property::{
        Multiplicity, Optionality, Property, PropertyMap, PropertyName,
    },
    property_spec::PropertySpec,
    raw::{
        date::RawDateSpec,
        file::RawFileSpec,
        number::RawNumberSpec,
        property::RawPropertyRef,
        string::{RawOptions, RawStringSpec},
    },
};
type RefPropertyMap = HashMap<PropertyName, RawPropertyRef>;
type ExpandedPropertyMap = PropertyMap;

// ─────────────────────────────────────────────────────────────────────────────
//  RefExpander
// ─────────────────────────────────────────────────────────────────────────────

/// Resolves raw `$ref` property pointers against the [`PropertyBank`].
///
/// Holds a shared reference to the bank so callers can pre-build the bank
/// once and reuse the expander across multiple schemas.
///
/// **Internal API**: This type is public solely for benchmarking purposes.
/// Do not depend on it in production code - use `Builder` instead.
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

    /// Expand references for a single schema property map.
    #[inline]
    pub fn expand_properties(
        &self,
        properties: &RefPropertyMap,
    ) -> Result<ExpandedPropertyMap, SchemaError> {
        let mut expanded = PropertyMap::new();

        #[expect(
            clippy::iter_over_hash_type,
            reason = "Ordering is not required for property expansion"
        )]
        for (name, entry) in properties {
            let prop = self.expand_property(name, entry)?;
            expanded.insert(name.clone(), prop);
        }

        Ok(expanded)
    }

    /// Resolve a single raw property reference into a validated [`Property`].
    fn expand_property(
        &self,
        name: &PropertyName,
        entry: &RawPropertyRef,
    ) -> Result<Property, SchemaError> {
        let bank_name = entry.ref_path.target_name();
        let base = self.bank.get(bank_name).ok_or_else(|| {
            SchemaError::PropertyRef(super::error::PropertyRefError::NotFound {
                reference: entry.ref_path.as_str().into(),
            })
        })?;

        let optionality = Self::optionality(base.optionality(), entry.required);
        let multiplicity = Self::multiplicity(base.multiplicity(), entry.multi);
        let spec = Self::spec(base.spec(), name, entry)?;

        Ok(Property::new(base.id(), optionality, multiplicity, spec))
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on &PropertySpec with value patterns is idiomatic"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "Spec resolution keeps all override logic in one place"
    )]
    fn spec(
        base: &PropertySpec,
        name: &PropertyName,
        ref_entry: &RawPropertyRef,
    ) -> Result<PropertySpec, SchemaError> {
        let has_number = ref_entry.min.is_some()
            || ref_entry.max.is_some()
            || ref_entry.step.is_some();
        let has_string =
            ref_entry.options.is_some() || ref_entry.pattern.is_some();
        let has_date = ref_entry.format.is_some();
        let has_file =
            ref_entry.directory.is_some() || ref_entry.file_class.is_some();

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
                let overrides = RawNumberSpec {
                    min: ref_entry.min,
                    max: ref_entry.max,
                    step: ref_entry.step,
                };
                Ok(PropertySpec::Number(
                    spec.clone().apply_overrides(&overrides)?,
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
                let options = warn_and_normalize_options(
                    name,
                    "ref_override",
                    ref_entry.options.as_ref(),
                );
                let overrides = RawStringSpec {
                    options,
                    pattern: ref_entry.pattern.clone(),
                };
                Ok(PropertySpec::String(
                    spec.clone().apply_overrides(&overrides)?,
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
                let overrides = RawDateSpec {
                    format: ref_entry.format.clone(),
                };
                Ok(PropertySpec::Date(
                    spec.clone().apply_overrides(&overrides)?,
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
                let overrides = RawFileSpec {
                    directory: ref_entry.directory.clone(),
                    file_class: ref_entry.file_class.clone(),
                };
                Ok(PropertySpec::File(
                    spec.clone().apply_overrides(&overrides)?,
                ))
            }
        }
    }

    #[inline]
    #[must_use]
    fn optionality(
        base: Optionality,
        override_val: Option<bool>,
    ) -> Optionality {
        override_val.map_or(base, Optionality::from)
    }

    #[inline]
    #[must_use]
    fn multiplicity(
        base: Multiplicity,
        override_val: Option<bool>,
    ) -> Multiplicity {
        override_val.map_or(base, Multiplicity::from)
    }
}

#[expect(
    clippy::pattern_type_mismatch,
    reason = "matching on &RawOptions with value patterns is idiomatic"
)]
fn warn_and_normalize_options(
    name: &PropertyName,
    context: &'static str,
    options: Option<&RawOptions>,
) -> Option<RawOptions> {
    let options = options?;

    let empty = match options {
        RawOptions::Plain(items) => items.is_empty(),
        RawOptions::Ordered(entries) | RawOptions::Labeled(entries) => {
            entries.is_empty()
        }
    };

    if empty {
        tracing::warn!(
            property = name.as_str(),
            context,
            "options list is empty; treating as unspecified"
        );
        return None;
    }

    if has_duplicate_option_values(options) {
        tracing::warn!(
            property = name.as_str(),
            context,
            "options list contains duplicate values"
        );
    }

    Some(options.clone())
}

#[expect(
    clippy::pattern_type_mismatch,
    reason = "matching on &RawOptions with value patterns is idiomatic"
)]
fn has_duplicate_option_values(options: &RawOptions) -> bool {
    let mut seen: std::collections::HashSet<&str> =
        std::collections::HashSet::new();
    match options {
        RawOptions::Plain(items) => {
            items.iter().map(AsRef::as_ref).any(|value| !seen.insert(value))
        }
        RawOptions::Ordered(entries) | RawOptions::Labeled(entries) => entries
            .iter()
            .map(|entry| entry.value.as_ref())
            .any(|value| !seen.insert(value)),
    }
}

#[inline]
fn type_mismatch(expected: &str, actual: &str) -> SchemaError {
    SchemaError::PropertyRef(super::error::PropertyRefError::TypeMismatch {
        expected: expected.into(),
        actual: actual.into(),
    })
}

// ─────────────────────────────────────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures before test sub-modules"
)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::schema::{
        bank::PropertyBank,
        property::{PropertyId, PropertyMap, PropertyName},
        property_spec::{BoolSpec, PropertySpec},
        raw::property::RawPropertyRef,
    };

    mod fixtures {
        use super::*;

        pub fn bool_property(
            name: &str,
        ) -> Result<(PropertyName, Property), SchemaError> {
            let property = Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID),
                Optionality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            );
            Ok((PropertyName::try_new(name)?, property))
        }

        pub fn bank_with(entry: (PropertyName, Property)) -> PropertyBank {
            let mut properties = PropertyMap::new();
            properties.insert(entry.0, entry.1);
            PropertyBank::from(properties)
        }

        pub fn ref_entry(ref_path: &str) -> RawPropertyRef {
            let json = format!(r#"{{"$ref": "{ref_path}"}}"#);
            serde_json::from_str(&json)
                .expect("Test fixture should create valid RawPropertyRef")
        }

        pub fn ref_with_overrides(
            ref_path: &str,
            required: Option<bool>,
            multi: Option<bool>,
        ) -> RawPropertyRef {
            use std::fmt::Write as _;

            let mut json = format!(r#"{{"$ref": "{ref_path}""#);
            if let Some(req) = required {
                write!(json, r#", "required": {req}"#)
                    .expect("write to string fails only on OOM");
            }
            if let Some(m) = multi {
                write!(json, r#", "multi": {m}"#)
                    .expect("write to string fails only on OOM");
            }
            json.push('}');

            serde_json::from_str(&json)
                .expect("Test fixture should create valid RawPropertyRef")
        }
    }

    const TEST_PROPERTY_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0F01);

    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test functions use assert! macros"
    )]
    mod expand_property {
        use super::*;

        #[test]
        fn ref_resolves_from_bank() -> Result<(), SchemaError> {
            let base = fixtures::bool_property("status")?;
            let bank = fixtures::bank_with(base);
            let expander = RefExpander::new(&bank);
            let name = PropertyName::try_new("alias")?;
            let entry = fixtures::ref_entry("#property_bank/status");
            let mut refs = HashMap::new();
            refs.insert(name.clone(), entry);
            let expanded_props = expander.expand_properties(&refs)?;
            assert!(expanded_props.has(&name));
            Ok(())
        }

        #[test]
        fn ref_overrides_optionality_and_multiplicity()
        -> Result<(), SchemaError> {
            let base = fixtures::bool_property("status")?;
            let bank = fixtures::bank_with(base);
            let expander = RefExpander::new(&bank);
            let entry = fixtures::ref_with_overrides(
                "#property_bank/status",
                Some(false),
                Some(true),
            );
            let name = PropertyName::try_new("status")?;
            let mut refs = HashMap::new();
            refs.insert(name.clone(), entry);
            let expanded_props = expander.expand_properties(&refs)?;
            let prop =
                expanded_props.get(&name).expect("expanded property exists");
            assert_eq!(prop.optionality(), Optionality::Optional);
            assert_eq!(prop.multiplicity(), Multiplicity::Many);
            Ok(())
        }

        #[test]
        fn ref_type_mismatch_returns_error() {
            let base =
                fixtures::bool_property("status").expect("valid property");
            let bank = fixtures::bank_with(base);
            let expander = RefExpander::new(&bank);
            let json = r##"{
                "$ref": "#property_bank/status",
                "min": 0.0
            }"##;
            let entry: RawPropertyRef = serde_json::from_str(json)
                .expect("Valid ref with number override should deserialize");
            let name = PropertyName::try_new("status").expect("valid name");
            let mut refs = HashMap::new();
            refs.insert(name, entry);
            let result = expander.expand_properties(&refs);
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
            let entry = fixtures::ref_entry("#property_bank/missing");
            let name = PropertyName::try_new("missing").expect("valid name");
            let mut refs = HashMap::new();
            refs.insert(name, entry);
            let result = expander.expand_properties(&refs);
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
}
