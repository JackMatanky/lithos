//! Raw schema and property input definitions (syntax validation layer).
//!
//! ## Validation Boundaries
//!
//! This module implements **syntax-only validation** using the type system
//! and regex patterns. It does NOT validate semantics.
//!
//! ### What Raw Validates (Syntax)
//!
//! - File name format (alphanumeric + dash/underscore, lowercase)
//! - Property name syntax (via regex)
//! - Unique property names within a schema
//! - Security violations (path traversal attempts)
//!
//! ### What Raw Does NOT Validate (Semantics)
//!
//! - Property ref existence (validated by [`crate::schema::dereferencer`])
//! - Schema ref existence (validated by [`crate::schema::extender`])
//! - Circular inheritance (validated by [`crate::schema::extender`])
//! - Depth limits (validated by [`crate::schema::resolver`])
//!
//! **Key Principle**: Validate as late as possible (only when you have the
//! data needed to validate).

#![expect(
    clippy::module_name_repetitions,
    reason = "Raw* types follow naming conventions for input layer types"
)]

use std::collections::BTreeMap;

use super::{
    error::{SchemaError, SchemaIngestionError},
    property_spec::{
        BoolSpec, DateSpec, FileSpec, NumberSpec, OptionEntry, PropertySpec,
        StringFormat, StringSpec,
    },
};

/// Current supported schema version.
pub const SCHEMA_VERSION: &str = "1.0";

/// Raw schema definition loaded from vault files.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{RawSchema, RawProperty, RawPropertyInline};
/// use lithos_core::schema::raw::{RawPropertySpec, RawBoolSpec};
/// use std::collections::HashMap;
/// # fn run() -> Result<(), Box<dyn std::error::Error>> {
///
/// let mut properties = HashMap::new();
/// properties.insert(
///     "archived".into(),
///     RawProperty::Inline(RawPropertyInline {
///         required: false,
///         multi: false,
///         spec: RawPropertySpec::Bool(RawBoolSpec),
///     }),
/// );
/// let schema = RawSchema {
///     name: "note".into(),
///     extends: None,
///     excludes: Vec::new(),
///     properties,
/// };
/// assert_eq!(schema.properties.len(), 1, "Schema should contain one property");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawSchema {
    /// Schema format version (defaults to "1.0" if not specified).
    #[serde(rename = "$version", default = "default_schema_version")]
    pub version: Box<str>,
    /// Schema name (always derived from filename by Ingestor).
    ///
    /// This field is NOT read from the file - it is always set by the Ingestor
    /// based on the filename (without extension). The file format does not
    /// include a `name` field.
    #[serde(skip)]
    pub name: Box<str>,
    /// Optional parent schema name for inheritance.
    pub extends: Option<Box<str>>,
    /// Property names to exclude from parent schema.
    #[serde(default)]
    pub excludes: Vec<Box<str>>,
    /// Map of property name to property definition.
    pub properties: std::collections::HashMap<Box<str>, RawProperty>,
}

/// Default function for schema version field.
fn default_schema_version() -> Box<str> {
    SCHEMA_VERSION.into()
}

impl RawSchema {
    /// Validate the schema version matches the expected version.
    ///
    /// # Errors
    /// Returns `SchemaIngestionError::UnsupportedVersion` if the version
    /// does not match.
    #[inline]
    pub fn validate_version(
        &self,
        path: &str,
    ) -> Result<(), SchemaIngestionError> {
        if self.version.as_ref() != SCHEMA_VERSION {
            return Err(SchemaIngestionError::UnsupportedVersion {
                path: path.into(),
                found: self.version.clone(),
                expected: SCHEMA_VERSION.into(),
            });
        }
        Ok(())
    }

    /// Validate raw schema syntax and structure.
    ///
    /// This performs syntactic validation only:
    /// - Schema name syntax (via `SchemaName`)
    /// - Unique property names
    /// - Parent schema name syntax (if present)
    /// - Exclude property name syntax
    ///
    /// Semantic validation (property refs exist, circular inheritance, etc.)
    /// happens during resolution.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn validate(&self) -> Result<(), SchemaError> {
        // Validate schema name syntax
        use super::id::SchemaName;
        SchemaName::try_new(self.name.as_ref())?;

        // Validate parent schema name syntax (if present)
        if let Some(parent) = self.extends.as_ref() {
            SchemaName::try_new(parent.as_ref())?;
        }

        // Validate excludes syntax
        for excluded in &self.excludes {
            use super::property::PropertyName;
            PropertyName::try_new(excluded.as_ref())?;
        }

        // Check for duplicate property names
        // (HashMap ensures uniqueness, but check for clarity)
        if self.properties.is_empty() {
            // Empty properties is valid (may inherit all from parent)
            return Ok(());
        }

        // Validate property name syntax
        #[expect(
            clippy::iter_over_hash_type,
            reason = "Validation does not depend on iteration order"
        )]
        for prop_name in self.properties.keys() {
            use super::property::PropertyName;
            PropertyName::try_new(prop_name.as_ref())?;
        }

        Ok(())
    }
}

/// Raw property for schema properties map.
///
/// Used in `RawSchema.properties` where the name is the map key.
/// Discriminated by presence of `$ref` field. Ref is tried first because
/// it has a required `$ref` field that Inline never has.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{
///     RawBoolSpec, RawProperty, RawPropertyInline, RawPropertySpec,
/// };
///
/// let property = RawProperty::Inline(RawPropertyInline {
///     required: false,
///     multi: false,
///     spec: RawPropertySpec::Bool(RawBoolSpec),
/// });
/// match property {
///     RawProperty::Inline(_) => {}
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum RawProperty {
    /// A reference to a property in the property bank with optional overrides.
    Ref(RawPropertyRef),
    /// An inline property definition.
    Inline(RawPropertyInline),
}

/// Inline variant of a raw property.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{
///     RawBoolSpec, RawPropertyInline, RawPropertySpec,
/// };
///
/// let inline = RawPropertyInline {
///     required: false,
///     multi: false,
///     spec: RawPropertySpec::Bool(RawBoolSpec),
/// };
/// let _ = inline;
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyInline {
    /// Whether property is required.
    #[serde(default)]
    pub required: bool,
    /// Whether property accepts multiple values.
    #[serde(default)]
    pub multi: bool,
    /// Type-specific validation constraints.
    #[serde(flatten)]
    pub spec: RawPropertySpec,
}

/// Reference variant of a raw property with optional overrides.
///
/// Override fields are grouped by type via flattened `Raw*Spec` structs.
/// All override fields are `Option<T>` — `None` means "don't override".
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{
///     RawDateSpec, RawFileSpec, RawNumberSpec, RawPropertyRef, RawStringSpec,
/// };
///
/// let reference = RawPropertyRef {
///     ref_path: "property_bank#/flag".into(),
///     required: None,
///     multi: None,
///     number: RawNumberSpec::default(),
///     string: RawStringSpec::default(),
///     date: RawDateSpec::default(),
///     file: RawFileSpec::default(),
/// };
/// let _ = reference;
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyRef {
    /// The reference path (e.g., `property_bank#/date_iso_8601`).
    #[serde(rename = "$ref")]
    pub ref_path: Box<str>,
    /// Override whether property is required.
    pub required: Option<bool>,
    /// Override whether property accepts multiple values.
    pub multi: Option<bool>,
    /// Number-type overrides (min, max, step).
    #[serde(flatten)]
    pub number: RawNumberSpec,
    /// String-type overrides (options, pattern).
    #[serde(flatten)]
    pub string: RawStringSpec,
    /// Date-type overrides (format).
    #[serde(flatten)]
    pub date: RawDateSpec,
    /// File-type overrides (directory, `file_class`).
    #[serde(flatten)]
    pub file: RawFileSpec,
}

/// Raw property specification (serde-facing input type).
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::{RawBoolSpec, RawPropertySpec};
///
/// let spec = RawPropertySpec::Bool(RawBoolSpec);
/// match spec {
///     RawPropertySpec::Bool(_) => {}
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[non_exhaustive]
pub enum RawPropertySpec {
    /// Boolean property definition (marker type).
    Bool(RawBoolSpec),
    /// Date property definition.
    Date(RawDateSpec),
    /// File property definition.
    File(RawFileSpec),
    /// Number property definition.
    Number(RawNumberSpec),
    /// String property definition.
    String(RawStringSpec),
}

impl RawPropertySpec {
    /// Validate and compile a persisted definition into a validated spec.
    ///
    /// # Errors
    /// Returns `SchemaError` if the definition is invalid.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::raw::{RawBoolSpec, RawPropertySpec};
    ///
    /// let spec = RawPropertySpec::Bool(RawBoolSpec);
    /// let _validated = spec.try_into_validated()?;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    #[inline]
    pub fn try_into_validated(self) -> Result<PropertySpec, SchemaError> {
        match self {
            Self::Bool(_) => Ok(PropertySpec::Bool(BoolSpec::default())),
            Self::Date(def) => {
                let format = def.format.ok_or_else(|| {
                    SchemaError::ValidationFailed(
                        "date format is required".into(),
                    )
                })?;
                Ok(PropertySpec::Date(DateSpec::try_new(&format)?))
            }
            Self::File(def) => Ok(PropertySpec::File(FileSpec::try_new(
                def.directory.as_deref(),
                def.file_class.as_deref(),
            )?)),
            Self::Number(def) => Ok(PropertySpec::Number(NumberSpec::try_new(
                def.min, def.max, def.step,
            )?)),
            Self::String(def) => Ok(PropertySpec::String(StringSpec::try_new(
                def.pattern,
                def.format,
                def.options.map(RawOptions::into_entries),
            )?)),
        }
    }
}

/// Boolean property definition (marker type).
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::RawBoolSpec;
///
/// let _spec = RawBoolSpec;
/// ```
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
#[expect(
    clippy::exhaustive_structs,
    reason = "Marker type with no fields; non_exhaustive prevents construction"
)]
pub struct RawBoolSpec;

/// Date property definition.
///
/// All fields are `Option<T>` to support both inline definitions
/// (where `format` is required) and override contexts (where `None`
/// means "don't override").
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::RawDateSpec;
///
/// let _spec = RawDateSpec::default();
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct RawDateSpec {
    /// Date format string (using chrono format tokens).
    pub format: Option<Box<str>>,
}

/// File property definition.
///
/// All fields are `Option<T>` to support both inline definitions
/// and override contexts.
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::RawFileSpec;
///
/// let _spec = RawFileSpec::default();
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct RawFileSpec {
    /// Optional directory restriction (vault-relative path).
    pub directory: Option<Box<str>>,
    /// Optional file class restriction (schema name).
    pub file_class: Option<Box<str>>,
}

/// Number property definition.
///
/// All fields are `Option<T>` to support both inline definitions
/// and override contexts.
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::RawNumberSpec;
///
/// let _spec = RawNumberSpec::default();
/// ```
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct RawNumberSpec {
    /// Optional maximum value.
    pub max: Option<f64>,
    /// Optional minimum value.
    pub min: Option<f64>,
    /// Optional step increment.
    pub step: Option<f64>,
}

/// String property definition.
///
/// Supports `options`, `pattern`, and `format` per the meta-schema.
/// All fields are `Option<T>` to support both inline definitions
/// and override contexts.
///
/// # Invariants
/// - `format` and `pattern` are mutually exclusive (validated during
///   conversion).
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::RawStringSpec;
///
/// let _spec = RawStringSpec::default();
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct RawStringSpec {
    /// Optional allowed values in one of three formats.
    pub options: Option<RawOptions>,
    /// Optional regex pattern (mutually exclusive with `format`).
    pub pattern: Option<Box<str>>,
    /// Optional named format (mutually exclusive with `pattern`).
    pub format: Option<StringFormat>,
}

/// Raw property bank loaded from vault files.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::RawPropertyBank;
///
/// let bank = RawPropertyBank {
///     properties: std::collections::HashMap::new(),
/// };
/// let _ = bank;
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyBank {
    /// Property bank format version (defaults to "1.0" if not specified).
    #[serde(rename = "$version", default = "default_schema_version")]
    pub version: Box<str>,
    /// Map of property name to property definition.
    pub properties: std::collections::HashMap<Box<str>, RawPropertyBankEntry>,
}

impl RawPropertyBank {
    /// Validate the property bank version matches the expected version.
    ///
    /// # Errors
    /// Returns `SchemaIngestionError::UnsupportedVersion` if the version
    /// does not match.
    #[inline]
    pub fn validate_version(
        &self,
        path: &str,
    ) -> Result<(), SchemaIngestionError> {
        if self.version.as_ref() != SCHEMA_VERSION {
            return Err(SchemaIngestionError::UnsupportedVersion {
                path: path.into(),
                found: self.version.clone(),
                expected: SCHEMA_VERSION.into(),
            });
        }
        Ok(())
    }

    /// Validate raw property bank syntax and structure.
    ///
    /// This performs syntactic validation only:
    /// - Property names are unique (enforced by `HashMap` structure)
    /// - Property name syntax (via `PropertyName`)
    /// - Property specs are valid (enforced by serde deserialization)
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn validate(&self) -> Result<(), SchemaError> {
        // Validate property name syntax
        #[expect(
            clippy::iter_over_hash_type,
            reason = "Validation does not depend on iteration order"
        )]
        for prop_name in self.properties.keys() {
            use super::property::PropertyName;
            PropertyName::try_new(prop_name.as_ref())?;
        }

        Ok(())
    }
}

/// Entry in the raw property bank.
///
/// The property name is the map key, not a field here.
/// `required` is not present because the bank is schema-agnostic.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{RawBoolSpec, RawPropertyBankEntry, RawPropertySpec};
///
/// let entry = RawPropertyBankEntry {
///     multi: false,
///     spec: RawPropertySpec::Bool(RawBoolSpec),
/// };
/// let _ = entry;
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyBankEntry {
    /// Whether property accepts multiple values.
    #[serde(default)]
    pub multi: bool,
    /// Type-specific validation constraints.
    #[serde(flatten)]
    pub spec: RawPropertySpec,
}

/// Raw options definition supporting three formats.
///
/// # Modes
///
/// - **Mode 1 (List)**: `["a", "b"]` — plain array of string values
/// - **Mode 2 (Map)**: `{"1": "to_do", "2": "done"}` — ordered integer-keyed
///   object
/// - **Mode 3 (Rich)**: `[{"value": "a", "label": "A", "order": 1}]` — rich
///   entries with labels
///
/// # Deserialization Strategy
///
/// Uses custom deserializer with explicit type checking:
/// 1. If sequence: try as List (strings), then Rich (objects)
/// 2. If map: deserialize as Map
/// 3. Fail with clear error for other types
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::RawOptions;
///
/// let options = RawOptions::List(vec!["open".into(), "closed".into()]);
/// match options {
///     RawOptions::List(_) => {}
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[non_exhaustive]
pub enum RawOptions {
    /// Mode 1: Plain array of string values.
    List(Vec<Box<str>>),
    /// Mode 2: Integer-keyed ordered object.
    Map(BTreeMap<Box<str>, Box<str>>),
    /// Mode 3: Rich entries with optional label and order.
    Rich(Vec<RawOptionEntry>),
}

/// Rich option entry with optional label and display order.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::RawOptionEntry;
///
/// let entry = RawOptionEntry {
///     value: "open".into(),
///     label: Some("Open".into()),
///     order: Some(1),
/// };
/// let _ = entry;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawOptionEntry {
    /// The option value.
    pub value: Box<str>,
    /// Optional display label.
    pub label: Option<Box<str>>,
    /// Optional display order (lower = earlier).
    pub order: Option<u32>,
}

// Custom deserializer for RawOptions to avoid relying on untagged variant order
#[expect(
    clippy::missing_trait_methods,
    clippy::missing_inline_in_public_items,
    clippy::excessive_nesting,
    reason = "Custom serde Visitor requires specific method impls; excessive \
              nesting is inherent to discriminating union variants by peeking"
)]
impl<'de> serde::Deserialize<'de> for RawOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::fmt;

        use serde::de::{Error, MapAccess, SeqAccess, Visitor};

        struct RawOptionsVisitor;

        impl<'de> Visitor<'de> for RawOptionsVisitor {
            type Value = RawOptions;

            fn expecting(
                &self,
                formatter: &mut fmt::Formatter<'_>,
            ) -> fmt::Result {
                formatter.write_str(
                    "a sequence of strings, a sequence of objects with \
                     'value' field, or a map with string keys and string \
                     values",
                )
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                // Peek at first element to determine if List or Rich
                let first = seq.next_element::<serde_json::Value>()?;
                if let Some(value) = first {
                    if value.is_string() {
                        // List mode: array of strings
                        let mut items = vec![
                            value
                                .as_str()
                                .ok_or_else(|| {
                                    Error::custom("expected string")
                                })?
                                .into(),
                        ];
                        while let Some(elem) =
                            seq.next_element::<serde_json::Value>()?
                        {
                            if let Some(s) = elem.as_str() {
                                items.push(s.into());
                            } else {
                                return Err(Error::custom(
                                    "expected all array elements to be strings",
                                ));
                            }
                        }
                        Ok(RawOptions::List(items))
                    } else if value.is_object() {
                        // Rich mode: array of objects
                        let first_entry: RawOptionEntry =
                            serde_json::from_value(value)
                                .map_err(Error::custom)?;
                        let mut entries = vec![first_entry];
                        while let Some(entry) =
                            seq.next_element::<RawOptionEntry>()?
                        {
                            entries.push(entry);
                        }
                        Ok(RawOptions::Rich(entries))
                    } else {
                        Err(Error::custom(
                            "expected array elements to be strings or objects",
                        ))
                    }
                } else {
                    // Empty array defaults to List
                    Ok(RawOptions::List(vec![]))
                }
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                // Map mode: object with string keys and string values
                let mut result = BTreeMap::new();
                while let Some((key, value)) =
                    map.next_entry::<Box<str>, Box<str>>()?
                {
                    result.insert(key, value);
                }
                Ok(RawOptions::Map(result))
            }
        }

        deserializer.deserialize_any(RawOptionsVisitor)
    }
}

impl RawOptions {
    /// Convert raw options to a normalized vector of `OptionEntry`.
    ///
    /// # Modes
    ///
    /// - **List**: entries have `value = item`, `label = None`, order
    ///   preserved.
    /// - **Map**: keys parsed as integers, sorted by key, entries have `value =
    ///   map_value`, `label = None`.
    /// - **Rich**: sorted by `order` field (then array position), entries have
    ///   `value` and `label`.
    ///
    /// # Panics
    /// Panics if Rich mode option list has more than `u32::MAX` entries (>4
    /// billion). This is unrealistic in practice and indicates a malformed
    /// input.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::raw::RawOptions;
    ///
    /// let entries = RawOptions::List(vec!["open".into()]).into_entries();
    /// assert_eq!(entries.len(), 1);
    /// assert_eq!(entries[0].value.as_ref(), "open");
    /// ```
    #[inline]
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "Fail-fast on unrealistic >4 billion options prevents silent \
                  data corruption"
    )]
    pub fn into_entries(self) -> Vec<OptionEntry> {
        match self {
            Self::List(items) => items
                .into_iter()
                .map(|value| OptionEntry {
                    value,
                    label: None,
                })
                .collect(),
            Self::Map(map) => {
                let mut entries: Vec<_> = map
                    .into_iter()
                    .filter_map(|(key, value)| {
                        key.parse::<u32>()
                            .inspect_err(|e| {
                                tracing::debug!(
                                    key = %key,
                                    error = %e,
                                    "Option map key is not a valid u32, entry will be skipped"
                                );
                            })
                            .ok()
                            .map(|order| (order, value))
                    })
                    .collect();
                entries.sort_by_key(|&(order, _)| order);
                entries
                    .into_iter()
                    .map(|(_, value)| OptionEntry {
                        value,
                        label: None,
                    })
                    .collect()
            }
            Self::Rich(entries) => {
                let mut entries: Vec<_> = entries
                    .into_iter()
                    .enumerate()
                    .map(|(idx, entry)| {
                        let order = entry.order.unwrap_or_else(|| {
                            u32::try_from(idx).expect(
                                "Option list index exceeds u32::MAX (>4 \
                                 billion entries)",
                            )
                        });
                        (order, entry)
                    })
                    .collect();
                entries.sort_by_key(|&(order, _)| order);
                entries
                    .into_iter()
                    .map(|(_, entry)| OptionEntry {
                        value: entry.value,
                        label: entry.label,
                    })
                    .collect()
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module: raw_options_tests intentionally grouped for \
              logical organization"
)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn schema_name() -> Box<str> {
        "note".into()
    }

    #[test]
    fn raw_schema_defaults_to_empty_excludes() {
        let schema = RawSchema {
            version: SCHEMA_VERSION.into(),
            name: schema_name(),
            extends: None,
            excludes: Vec::new(),
            properties: HashMap::new(),
        };

        assert!(
            schema.excludes.is_empty(),
            "RawSchema should have empty excludes by default"
        );
    }

    #[test]
    fn raw_schema_defaults_to_no_extends() {
        let schema = RawSchema {
            version: SCHEMA_VERSION.into(),
            name: schema_name(),
            extends: None,
            excludes: Vec::new(),
            properties: HashMap::new(),
        };

        assert!(
            schema.extends.is_none(),
            "RawSchema should have no extends by default"
        );
    }

    #[test]
    fn raw_property_inline_variant_constructs() {
        let inline = RawPropertyInline {
            required: false,
            multi: false,
            spec: RawPropertySpec::Bool(RawBoolSpec),
        };
        let inline_variant = RawProperty::Inline(inline);

        assert!(
            matches!(inline_variant, RawProperty::Inline(_)),
            "RawProperty should be Inline variant"
        );
    }

    #[test]
    fn raw_property_ref_variant_constructs() {
        let reference = RawPropertyRef {
            ref_path: "property_bank#/status".into(),
            required: None,
            multi: None,
            number: RawNumberSpec::default(),
            string: RawStringSpec::default(),
            date: RawDateSpec::default(),
            file: RawFileSpec::default(),
        };
        let reference_variant = RawProperty::Ref(reference);

        assert!(
            matches!(reference_variant, RawProperty::Ref(_)),
            "RawProperty should be Ref variant"
        );
    }

    // --- RawOptions Deserialization Tests (E-04) ---

    #[expect(
        clippy::indexing_slicing,
        clippy::panic,
        clippy::wildcard_enum_match_arm,
        clippy::items_after_statements,
        reason = "Test code: indexing is safe after len check, panic shows \
                  test failure clearly, wildcard for unknown future variants \
                  is fine"
    )]
    mod raw_options_tests {
        use super::*;

        #[test]
        fn raw_options_deserializes_list_from_json() {
            let json = r#"["open", "closed", "archived"]"#;
            let options: RawOptions = serde_json::from_str(json).unwrap();
            match options {
                RawOptions::List(items) => {
                    assert_eq!(items.len(), 3);
                    assert_eq!(items[0].as_ref(), "open");
                    assert_eq!(items[1].as_ref(), "closed");
                    assert_eq!(items[2].as_ref(), "archived");
                }
                _ => panic!("Expected List variant"),
            }
        }

        #[test]
        fn raw_options_deserializes_empty_list() {
            let json = "[]";
            let options: RawOptions = serde_json::from_str(json).unwrap();
            match options {
                RawOptions::List(items) => {
                    assert!(items.is_empty());
                }
                _ => panic!("Expected List variant"),
            }
        }

        #[test]
        fn raw_options_deserializes_map_from_json() {
            let json = r#"{"1": "todo", "2": "done", "3": "archived"}"#;
            let options: RawOptions = serde_json::from_str(json).unwrap();
            match options {
                RawOptions::Map(map) => {
                    assert_eq!(map.len(), 3);
                    assert_eq!(
                        map.get("1").map(std::convert::AsRef::as_ref),
                        Some("todo")
                    );
                    assert_eq!(
                        map.get("2").map(std::convert::AsRef::as_ref),
                        Some("done")
                    );
                    assert_eq!(
                        map.get("3").map(std::convert::AsRef::as_ref),
                        Some("archived")
                    );
                }
                _ => panic!("Expected Map variant"),
            }
        }

        #[test]
        fn raw_options_deserializes_rich_from_json() {
            let json = r#"[
            {"value": "open", "label": "Open", "order": 1},
            {"value": "closed", "label": "Closed", "order": 2}
        ]"#;
            let options: RawOptions = serde_json::from_str(json).unwrap();
            match options {
                RawOptions::Rich(entries) => {
                    assert_eq!(entries.len(), 2);
                    assert_eq!(entries[0].value.as_ref(), "open");
                    assert_eq!(entries[0].label.as_deref(), Some("Open"));
                    assert_eq!(entries[0].order, Some(1));
                    assert_eq!(entries[1].value.as_ref(), "closed");
                    assert_eq!(entries[1].label.as_deref(), Some("Closed"));
                    assert_eq!(entries[1].order, Some(2));
                }
                _ => panic!("Expected Rich variant"),
            }
        }

        #[test]
        fn raw_options_deserializes_rich_with_minimal_fields() {
            let json = r#"[{"value": "open"}]"#;
            let options: RawOptions = serde_json::from_str(json).unwrap();
            match options {
                RawOptions::Rich(entries) => {
                    assert_eq!(entries.len(), 1);
                    assert_eq!(entries[0].value.as_ref(), "open");
                    assert_eq!(entries[0].label, None);
                    assert_eq!(entries[0].order, None);
                }
                _ => panic!("Expected Rich variant"),
            }
        }

        #[test]
        fn raw_options_deserializes_from_yaml_list() {
            let yaml = "
- open
- closed
- archived
";
            let options: RawOptions = serde_yaml::from_str(yaml).unwrap();
            match options {
                RawOptions::List(items) => {
                    assert_eq!(items.len(), 3);
                    assert_eq!(items[0].as_ref(), "open");
                }
                _ => panic!("Expected List variant"),
            }
        }

        #[test]
        fn raw_options_deserializes_from_toml_inline_array() {
            // TOML requires inline arrays to be in a table context
            let toml_str = r#"options = ["open", "closed"]"#;
            #[derive(serde::Deserialize)]
            struct Wrapper {
                options: RawOptions,
            }
            let wrapper: Wrapper = toml::from_str(toml_str).unwrap();
            match wrapper.options {
                RawOptions::List(items) => {
                    assert_eq!(items.len(), 2);
                    assert_eq!(items[0].as_ref(), "open");
                    assert_eq!(items[1].as_ref(), "closed");
                }
                _ => panic!("Expected List variant"),
            }
        }

        #[test]
        fn raw_options_rejects_mixed_array_types() {
            let json = r#"["string", 123]"#;
            let result: Result<RawOptions, _> = serde_json::from_str(json);
            assert!(result.is_err(), "Should reject mixed types in array");
        }

        #[test]
        fn raw_options_rejects_invalid_rich_structure() {
            let json = r#"[{"label": "Missing value field"}]"#;
            let result: Result<RawOptions, _> = serde_json::from_str(json);
            assert!(
                result.is_err(),
                "Should reject rich entries missing 'value' field"
            );
        }

        #[test]
        fn raw_options_into_entries_list_preserves_order() {
            let options =
                RawOptions::List(vec!["a".into(), "b".into(), "c".into()]);
            let entries = options.into_entries();
            assert_eq!(entries.len(), 3);
            assert_eq!(entries[0].value.as_ref(), "a");
            assert_eq!(entries[1].value.as_ref(), "b");
            assert_eq!(entries[2].value.as_ref(), "c");
            assert!(entries.iter().all(|e| e.label.is_none()));
        }

        #[test]
        fn raw_options_into_entries_map_sorts_by_key() {
            let mut map = BTreeMap::new();
            map.insert("3".into(), "third".into());
            map.insert("1".into(), "first".into());
            map.insert("2".into(), "second".into());
            let options = RawOptions::Map(map);
            let entries = options.into_entries();
            assert_eq!(entries.len(), 3);
            assert_eq!(entries[0].value.as_ref(), "first");
            assert_eq!(entries[1].value.as_ref(), "second");
            assert_eq!(entries[2].value.as_ref(), "third");
        }

        #[test]
        fn raw_options_into_entries_rich_sorts_by_order() {
            let options = RawOptions::Rich(vec![
                RawOptionEntry {
                    value: "c".into(),
                    label: Some("Third".into()),
                    order: Some(3),
                },
                RawOptionEntry {
                    value: "a".into(),
                    label: Some("First".into()),
                    order: Some(1),
                },
                RawOptionEntry {
                    value: "b".into(),
                    label: Some("Second".into()),
                    order: Some(2),
                },
            ]);
            let entries = options.into_entries();
            assert_eq!(entries.len(), 3);
            assert_eq!(entries[0].value.as_ref(), "a");
            assert_eq!(entries[0].label.as_deref(), Some("First"));
            assert_eq!(entries[1].value.as_ref(), "b");
            assert_eq!(entries[2].value.as_ref(), "c");
        }

        #[test]
        fn raw_options_into_entries_rich_uses_array_position_when_no_order() {
            let options = RawOptions::Rich(vec![
                RawOptionEntry {
                    value: "first".into(),
                    label: None,
                    order: None,
                },
                RawOptionEntry {
                    value: "second".into(),
                    label: None,
                    order: None,
                },
            ]);
            let entries = options.into_entries();
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].value.as_ref(), "first");
            assert_eq!(entries[1].value.as_ref(), "second");
        }

        // --- Edge Case Tests for E-04 (Deserialization Disambiguation) ---

        #[test]
        fn raw_options_disambiguates_empty_object_array_as_rich() {
            // Array of objects with only `value` field should deserialize as
            // Rich
            let json = r#"[{"value": "a"}, {"value": "b"}]"#;
            let options: RawOptions = serde_json::from_str(json).unwrap();
            match options {
                RawOptions::Rich(entries) => {
                    assert_eq!(entries.len(), 2);
                    assert_eq!(entries[0].value.as_ref(), "a");
                    assert_eq!(entries[0].label, None);
                }
                _ => panic!("Expected Rich variant for array of objects"),
            }
        }

        #[test]
        fn raw_options_disambiguates_strings_as_list() {
            // Array of strings should always deserialize as List
            let json = r#"["value", "label", "order"]"#;
            let options: RawOptions = serde_json::from_str(json).unwrap();
            match options {
                RawOptions::List(items) => {
                    assert_eq!(items.len(), 3);
                    // These are literal strings, not field names
                    assert_eq!(items[0].as_ref(), "value");
                    assert_eq!(items[1].as_ref(), "label");
                    assert_eq!(items[2].as_ref(), "order");
                }
                _ => panic!("Expected List variant for string array"),
            }
        }

        #[test]
        fn raw_options_rejects_array_of_numbers() {
            // Numbers are not valid option values
            let json = "[1, 2, 3]";
            let result: Result<RawOptions, _> = serde_json::from_str(json);
            assert!(
                result.is_err(),
                "Should reject array of numbers (not strings or objects)"
            );
        }

        #[test]
        fn raw_options_rejects_array_of_bools() {
            // Booleans are not valid option values
            let json = "[true, false]";
            let result: Result<RawOptions, _> = serde_json::from_str(json);
            assert!(
                result.is_err(),
                "Should reject array of booleans (not strings or objects)"
            );
        }

        #[test]
        fn raw_options_rejects_array_of_nulls() {
            // Nulls are not valid option values
            let json = "[null, null]";
            let result: Result<RawOptions, _> = serde_json::from_str(json);
            assert!(
                result.is_err(),
                "Should reject array of nulls (not strings or objects)"
            );
        }

        #[test]
        fn raw_options_rejects_nested_arrays() {
            // Nested arrays are not a valid mode
            let json = r#"[["a", "b"], ["c", "d"]]"#;
            let result: Result<RawOptions, _> = serde_json::from_str(json);
            assert!(
                result.is_err(),
                "Should reject nested arrays (not a valid mode)"
            );
        }

        #[test]
        fn raw_options_rejects_string_primitive() {
            // Single string is not an array or map
            let json = r#""single_value""#;
            let result: Result<RawOptions, _> = serde_json::from_str(json);
            assert!(
                result.is_err(),
                "Should reject string primitive (must be array or map)"
            );
        }

        #[test]
        fn raw_options_rejects_number_primitive() {
            // Single number is not an array or map
            let json = "42";
            let result: Result<RawOptions, _> = serde_json::from_str(json);
            assert!(
                result.is_err(),
                "Should reject number primitive (must be array or map)"
            );
        }

        #[test]
        fn raw_options_map_with_non_numeric_keys() {
            // Map keys can be any string, not just numeric
            let json = r#"{"open": "Open", "closed": "Closed"}"#;
            let options: RawOptions = serde_json::from_str(json).unwrap();
            match options {
                RawOptions::Map(map) => {
                    assert_eq!(map.len(), 2);
                    assert_eq!(
                        map.get("open").map(std::convert::AsRef::as_ref),
                        Some("Open")
                    );
                }
                _ => panic!("Expected Map variant"),
            }
        }

        #[test]
        fn raw_options_rich_with_extra_fields_ignored() {
            // Extra fields in Rich entries should be ignored (forward compat)
            let json = r#"[{"value": "a", "label": "A", "extra": "ignored"}]"#;
            let options: RawOptions = serde_json::from_str(json).unwrap();
            match options {
                RawOptions::Rich(entries) => {
                    assert_eq!(entries.len(), 1);
                    assert_eq!(entries[0].value.as_ref(), "a");
                    assert_eq!(entries[0].label.as_deref(), Some("A"));
                }
                _ => panic!("Expected Rich variant"),
            }
        }

        #[test]
        fn raw_options_empty_map() {
            // Empty map should deserialize successfully
            let json = "{}";
            let options: RawOptions = serde_json::from_str(json).unwrap();
            match options {
                RawOptions::Map(map) => {
                    assert!(map.is_empty());
                }
                _ => panic!("Expected Map variant for empty object"),
            }
        }
    }

    // ───────────────────────────────────────────────────────────────────────
    //  Raw Validation Tests
    // ───────────────────────────────────────────────────────────────────────

    #[test]
    fn raw_schema_validate_valid() {
        let schema = RawSchema {
            version: SCHEMA_VERSION.into(),
            name: "test_schema".into(),
            extends: None,
            excludes: Vec::new(),
            properties: HashMap::new(),
        };

        schema.validate().unwrap();
    }

    #[test]
    fn raw_schema_validate_with_parent() {
        let schema = RawSchema {
            version: SCHEMA_VERSION.into(),
            name: "child_schema".into(),
            extends: Some("parent_schema".into()),
            excludes: Vec::new(),
            properties: HashMap::new(),
        };

        schema.validate().unwrap();
    }

    #[test]
    fn raw_schema_validate_with_excludes() {
        let schema = RawSchema {
            version: SCHEMA_VERSION.into(),
            name: "test".into(),
            extends: Some("parent".into()),
            excludes: vec!["prop1".into(), "prop2".into()],
            properties: HashMap::new(),
        };

        schema.validate().unwrap();
    }

    #[test]
    fn raw_schema_validate_invalid_name() {
        let schema = RawSchema {
            version: SCHEMA_VERSION.into(),
            name: "Invalid Name".into(), // Uppercase + space
            extends: None,
            excludes: Vec::new(),
            properties: HashMap::new(),
        };

        schema.validate().unwrap_err();
    }

    #[test]
    fn raw_schema_validate_invalid_parent_name() {
        let schema = RawSchema {
            version: SCHEMA_VERSION.into(),
            name: "child".into(),
            extends: Some("Parent!Invalid".into()), // Uppercase + special char
            excludes: Vec::new(),
            properties: HashMap::new(),
        };

        schema.validate().unwrap_err();
    }

    #[test]
    fn raw_schema_validate_invalid_exclude_name() {
        let schema = RawSchema {
            version: SCHEMA_VERSION.into(),
            name: "test".into(),
            extends: Some("parent".into()),
            excludes: vec!["Invalid Property".into()], // Space
            properties: HashMap::new(),
        };

        schema.validate().unwrap_err();
    }

    #[test]
    fn raw_schema_validate_with_valid_properties() {
        let mut properties = HashMap::new();
        properties.insert(
            "valid_property".into(),
            RawProperty::Inline(RawPropertyInline {
                required: false,
                multi: false,
                spec: RawPropertySpec::Bool(RawBoolSpec),
            }),
        );

        let schema = RawSchema {
            version: SCHEMA_VERSION.into(),
            name: "test".into(),
            extends: None,
            excludes: Vec::new(),
            properties,
        };

        schema.validate().unwrap();
    }

    #[test]
    fn raw_schema_validate_invalid_property_name() {
        let mut properties = HashMap::new();
        properties.insert(
            "Invalid Property!".into(), // Space + special char
            RawProperty::Inline(RawPropertyInline {
                required: false,
                multi: false,
                spec: RawPropertySpec::Bool(RawBoolSpec),
            }),
        );

        let schema = RawSchema {
            version: SCHEMA_VERSION.into(),
            name: "test".into(),
            extends: None,
            excludes: Vec::new(),
            properties,
        };

        schema.validate().unwrap_err();
    }

    #[test]
    fn raw_property_bank_validate_valid() {
        let mut properties = HashMap::new();
        properties.insert("title".into(), RawPropertyBankEntry {
            multi: false,
            spec: RawPropertySpec::String(RawStringSpec::default()),
        });

        let bank = RawPropertyBank {
            version: SCHEMA_VERSION.into(),
            properties,
        };

        bank.validate().unwrap();
    }

    #[test]
    fn raw_property_bank_validate_empty() {
        let bank = RawPropertyBank {
            version: SCHEMA_VERSION.into(),
            properties: HashMap::new(),
        };

        bank.validate().unwrap();
    }

    #[test]
    fn raw_property_bank_validate_invalid_property_name() {
        let mut properties = HashMap::new();
        properties.insert(
            "Invalid Name!".into(), // Space + special char
            RawPropertyBankEntry {
                multi: false,
                spec: RawPropertySpec::Bool(RawBoolSpec),
            },
        );

        let bank = RawPropertyBank {
            version: SCHEMA_VERSION.into(),
            properties,
        };

        bank.validate().unwrap_err();
    }
}
