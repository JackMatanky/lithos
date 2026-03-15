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
//! - Property ref existence (validated by [`crate::schema::expander`])
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

pub mod property;
pub mod property_spec;

use std::{
    collections::{BTreeMap, HashMap},
    time::SystemTime,
};

use super::error::{SchemaError, SchemaIngestionError};

/// Current supported schema version.
pub const SCHEMA_VERSION: &str = "1.0";

// ─────────────────────────────────────────────────────────────────────────────
// Version Types
// ─────────────────────────────────────────────────────────────────────────────

/// Schema format version.
///
/// Represents the version string from schema and property bank files.
/// Validates against `SCHEMA_VERSION` constant.
///
/// # Design Note
///
/// This type uses the `validated()` pattern (validate after deserialization)
/// rather than "parse, don't validate" because Raw types are meant to be
/// direct representations of file contents, not domain types. A more idiomatic
/// solution would use custom serde deserializers to validate during parsing,
/// but that trades off error reporting quality and pipeline flexibility.
///
/// TODO: Revisit this design to find a more idiomatic Rust solution that
/// maintains the benefits of separate parsing and validation phases.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct RawSchemaVersion(Box<str>);

impl RawSchemaVersion {
    /// Validates the version against the expected version.
    ///
    /// # Errors
    /// Returns `SchemaIngestionError::UnsupportedVersion` if the version
    /// does not match the expected version.
    #[inline]
    pub fn validate(&self, path: &str) -> Result<(), SchemaIngestionError> {
        if self.0.as_ref() != SCHEMA_VERSION {
            return Err(SchemaIngestionError::UnsupportedVersion {
                path: path.into(),
                found: self.0.clone(),
                expected: SCHEMA_VERSION.into(),
            });
        }
        Ok(())
    }

    /// Returns the version string.
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for RawSchemaVersion {
    #[inline]
    fn default() -> Self {
        Self(SCHEMA_VERSION.into())
    }
}

impl From<&str> for RawSchemaVersion {
    #[inline]
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl From<Box<str>> for RawSchemaVersion {
    #[inline]
    fn from(s: Box<str>) -> Self {
        Self(s)
    }
}

impl From<String> for RawSchemaVersion {
    #[inline]
    fn from(s: String) -> Self {
        Self(s.into())
    }
}

impl AsRef<str> for RawSchemaVersion {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Schema Types
// ─────────────────────────────────────────────────────────────────────────────

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
    #[serde(rename = "$version", default)]
    pub version: RawSchemaVersion,
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
    pub properties: HashMap<Box<str>, property::RawProperty>,
    /// File metadata for staleness detection.
    ///
    /// Populated during ingestion. Not serialized to TOML.
    #[serde(skip)]
    pub metadata: RawSchemaMetadata,
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
        self.version.validate(path)
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
        use super::aggregate::SchemaName;
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

    /// Consuming constructor that validates the schema.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn validated(self, path: &str) -> Result<Self, SchemaIngestionError> {
        self.validate_version(path)?;
        self.validate().map_err(SchemaIngestionError::from)?;
        Ok(self)
    }
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
    #[serde(rename = "$version", default)]
    pub version: RawSchemaVersion,
    /// Map of property name to property definition.
    pub properties: HashMap<Box<str>, property::RawPropertyBankEntry>,
    /// File metadata for staleness detection.
    ///
    /// Populated during ingestion. Not serialized to TOML.
    #[serde(skip)]
    pub metadata: RawSchemaMetadata,
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
        self.version.validate(path)
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

    /// Consuming constructor that validates the property bank.
    ///
    /// # Errors
    /// Returns `SchemaIngestionError` if validation fails.
    #[inline]
    pub fn validated(self, path: &str) -> Result<Self, SchemaIngestionError> {
        self.validate_version(path)?;
        self.validate().map_err(SchemaIngestionError::from)?;
        Ok(self)
    }
}

/// Raw file metadata for staleness detection.
///
/// Populated during ingestion from filesystem metadata and content hashing.
/// Not part of the serialized TOML format.
///
/// Used for both schema files and property bank files.
#[derive(Debug, Clone, PartialEq, Default)]
#[non_exhaustive]
pub struct RawSchemaMetadata {
    /// File creation timestamp (birthtime).
    ///
    /// None if the filesystem doesn't support birthtime.
    pub created_at: Option<SystemTime>,

    /// File modification timestamp (mtime).
    pub modified_at: Option<SystemTime>,

    /// BLAKE3 hash of raw file content (before parsing).
    ///
    /// Computed from raw file bytes during ingestion.
    pub content_hash: Option<[u8; 32]>,

    /// Per-property BLAKE3 hashes for incremental resolution.
    ///
    /// Maps property name to its content hash. Enables detecting which
    /// specific properties changed without re-parsing the entire file.
    ///
    /// Populated for both schema files and property bank files.
    pub property_hashes: BTreeMap<Box<str>, [u8; 32]>,
}

impl RawSchemaMetadata {
    /// Compute per-property hashes from a property map.
    ///
    /// Hashes each property definition to enable incremental change detection.
    /// Uses JSON serialization for stable, canonical hashing (TOML/YAML have
    /// unstable ordering).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::collections::HashMap;
    /// use lithos_core::schema::raw::{RawSchemaMetadata, RawProperty};
    ///
    /// let mut properties = HashMap::new();
    /// properties.insert("title".into(), RawProperty::Ref("text".into()));
    ///
    /// let hashes = RawSchemaMetadata::compute_property_hashes(&properties);
    /// assert_eq!(hashes.len(), 1);
    /// ```
    #[must_use]
    #[inline]
    #[expect(
        clippy::iter_over_hash_type,
        reason = "Iteration order doesn't matter - we're collecting into \
                  BTreeMap which provides deterministic ordering"
    )]
    pub fn compute_property_hashes(
        properties: &HashMap<Box<str>, property::RawProperty>,
    ) -> BTreeMap<Box<str>, [u8; 32]> {
        let mut hashes = BTreeMap::new();

        for (name, prop) in properties {
            // Serialize property to JSON for stable hashing
            if let Ok(json) = serde_json::to_string(prop) {
                let hash = blake3::hash(json.as_bytes());
                hashes.insert(name.clone(), *hash.as_bytes());
            }
        }

        hashes
    }
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{property, property_spec, *};

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
            metadata: RawSchemaMetadata::default(),
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
            metadata: RawSchemaMetadata::default(),
        };

        assert!(
            schema.extends.is_none(),
            "RawSchema should have no extends by default"
        );
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
            metadata: RawSchemaMetadata::default(),
        };

        schema.validated("test").unwrap();
    }

    #[test]
    fn raw_schema_validate_with_parent() {
        let schema = RawSchema {
            version: SCHEMA_VERSION.into(),
            name: "child_schema".into(),
            extends: Some("parent_schema".into()),
            excludes: Vec::new(),
            properties: HashMap::new(),
            metadata: RawSchemaMetadata::default(),
        };

        schema.validated("test").unwrap();
    }

    #[test]
    fn raw_schema_validate_with_excludes() {
        let schema = RawSchema {
            version: SCHEMA_VERSION.into(),
            name: "test".into(),
            extends: Some("parent".into()),
            excludes: vec!["prop1".into(), "prop2".into()],
            properties: HashMap::new(),
            metadata: RawSchemaMetadata::default(),
        };

        schema.validated("test").unwrap();
    }

    #[test]
    fn raw_schema_validate_invalid_name() {
        let schema = RawSchema {
            version: SCHEMA_VERSION.into(),
            name: "Invalid Name".into(), // Uppercase + space
            extends: None,
            excludes: Vec::new(),
            properties: HashMap::new(),
            metadata: RawSchemaMetadata::default(),
        };

        schema.validated("test").unwrap_err();
    }

    #[test]
    fn raw_schema_validate_invalid_parent_name() {
        let schema = RawSchema {
            version: SCHEMA_VERSION.into(),
            name: "child".into(),
            extends: Some("Parent!Invalid".into()), // Uppercase + special char
            excludes: Vec::new(),
            properties: HashMap::new(),
            metadata: RawSchemaMetadata::default(),
        };

        schema.validated("test").unwrap_err();
    }

    #[test]
    fn raw_schema_validate_invalid_exclude_name() {
        let schema = RawSchema {
            version: SCHEMA_VERSION.into(),
            name: "test".into(),
            extends: Some("parent".into()),
            excludes: vec!["Invalid Property".into()], // Space
            properties: HashMap::new(),
            metadata: RawSchemaMetadata::default(),
        };

        schema.validated("test").unwrap_err();
    }

    #[test]
    fn raw_schema_validate_with_valid_properties() {
        let mut properties = HashMap::new();
        properties.insert(
            "valid_property".into(),
            property::RawProperty::Inline(property::RawPropertyInline {
                required: false,
                multi: false,
                spec: property_spec::RawPropertySpec::Bool(
                    property_spec::RawBoolSpec,
                ),
            }),
        );

        let schema = RawSchema {
            version: SCHEMA_VERSION.into(),
            name: "test".into(),
            extends: None,
            excludes: Vec::new(),
            properties,
            metadata: RawSchemaMetadata::default(),
        };

        schema.validated("test").unwrap();
    }

    #[test]
    fn raw_schema_validate_invalid_property_name() {
        let mut properties = HashMap::new();
        properties.insert(
            "Invalid Property!".into(), // Space + special char
            property::RawProperty::Inline(property::RawPropertyInline {
                required: false,
                multi: false,
                spec: property_spec::RawPropertySpec::Bool(
                    property_spec::RawBoolSpec,
                ),
            }),
        );

        let schema = RawSchema {
            version: SCHEMA_VERSION.into(),
            name: "test".into(),
            extends: None,
            excludes: Vec::new(),
            properties,
            metadata: RawSchemaMetadata::default(),
        };

        schema.validated("test").unwrap_err();
    }

    #[test]
    fn raw_property_bank_validate_valid() {
        let mut properties = HashMap::new();
        properties.insert("title".into(), property::RawPropertyBankEntry {
            multi: false,
            spec: property_spec::RawPropertySpec::String(
                property_spec::RawStringSpec::default(),
            ),
        });

        let bank = RawPropertyBank {
            version: SCHEMA_VERSION.into(),
            properties,
            metadata: RawSchemaMetadata::default(),
        };

        bank.validated("test").unwrap();
    }

    #[test]
    fn raw_property_bank_validate_empty() {
        let bank = RawPropertyBank {
            version: SCHEMA_VERSION.into(),
            properties: HashMap::new(),
            metadata: RawSchemaMetadata::default(),
        };

        bank.validated("test").unwrap();
    }

    #[test]
    fn raw_property_bank_validate_invalid_property_name() {
        let mut properties = HashMap::new();
        properties.insert(
            "Invalid Name!".into(), // Space + special char
            property::RawPropertyBankEntry {
                multi: false,
                spec: property_spec::RawPropertySpec::Bool(
                    property_spec::RawBoolSpec,
                ),
            },
        );

        let bank = RawPropertyBank {
            version: SCHEMA_VERSION.into(),
            properties,
            metadata: RawSchemaMetadata::default(),
        };

        bank.validated("test").unwrap_err();
    }
}
