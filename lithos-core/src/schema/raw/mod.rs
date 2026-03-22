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

use std::{collections::HashMap, time::SystemTime};

use super::error::SchemaIngestionError;

// ─────────────────────────────────────────────────────────────────────────────
// Schema Types
// ─────────────────────────────────────────────────────────────────────────────

/// Raw schema definition loaded from vault files.
///
/// Property names are validated during deserialization via `RawPropertyMap`,
/// ensuring all keys are valid `PropertyName` instances.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::RawSchema;
///
/// // Properties are validated during deserialization
/// let toml = r#"
/// [properties.my_property]
/// type = "bool"
/// required = true
/// "#;
/// let mut schema: RawSchema = toml::from_str(toml)?;
/// schema = schema.with_name("note".into());
/// // schema.properties() returns HashMap<PropertyName, RawProperty>
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawSchema {
    /// Schema format version (defaults to "1.0" if not specified).
    #[serde(rename = "$version", default)]
    version: RawSchemaVersion,

    /// Schema name (always derived from filename by Ingestor).
    ///
    /// This field is NOT read from the file - it is always set by the Ingestor
    /// based on the filename (without extension). The file format does not
    /// include a `name` field.
    #[serde(skip)]
    name: Box<str>,

    /// Optional parent schema name for inheritance.
    extends: Option<Box<str>>,

    /// Property names to exclude from parent schema.
    #[serde(default)]
    excludes: Vec<Box<str>>,

    /// Validated property map (keys are guaranteed valid `PropertyNames`).
    properties: property::RawPropertyMap<property::RawProperty>,

    /// File metadata for staleness detection.
    ///
    /// Populated during ingestion. Not serialized to TOML.
    #[serde(skip)]
    metadata: RawSchemaMetadata,
}

impl RawSchema {
    /// Returns the schema version.
    #[inline]
    #[must_use]
    pub fn version(&self) -> &RawSchemaVersion {
        &self.version
    }

    /// Returns the schema name.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the parent schema name (if present).
    #[inline]
    #[must_use]
    pub fn extends(&self) -> Option<&str> {
        self.extends.as_deref()
    }

    /// Returns the excluded property names.
    #[inline]
    #[must_use]
    pub fn excludes(&self) -> &[Box<str>] {
        &self.excludes
    }

    /// Returns the properties map.
    ///
    /// All keys are guaranteed to be valid `PropertyName` instances.
    #[inline]
    #[must_use]
    pub fn properties(
        &self,
    ) -> &HashMap<super::property::PropertyName, property::RawProperty> {
        self.properties.as_map()
    }

    /// Returns the metadata.
    #[inline]
    #[must_use]
    pub fn metadata(&self) -> &RawSchemaMetadata {
        &self.metadata
    }

    /// Set the schema name (called by Ingestor after deserialization).
    ///
    /// The name is derived from the filename, not the file content.
    #[inline]
    #[must_use]
    pub fn with_name(mut self, name: Box<str>) -> Self {
        self.name = name;
        self
    }

    /// Set metadata (called by Ingestor after deserialization).
    #[inline]
    #[must_use]
    pub fn with_metadata(mut self, metadata: RawSchemaMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Validate the schema version matches the expected version.
    ///
    /// This is separate from property key validation (which happens during
    /// deserialization) because version errors need path context for better
    /// error messages.
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

    /// Consuming constructor that validates the schema version and
    /// inheritance fields.
    ///
    /// Property name validation happens during deserialization via
    /// `RawPropertyMap`, so this only validates version, extends, and excludes.
    ///
    /// # Errors
    /// Returns `SchemaIngestionError` if validation fails.
    #[inline]
    pub fn validated(self, path: &str) -> Result<Self, SchemaIngestionError> {
        use super::{aggregate::SchemaName, property::PropertyName};

        self.validate_version(path)?;

        // Validate schema name syntax
        SchemaName::try_new(self.name.as_ref()).map_err(|error| {
            SchemaIngestionError::Schema {
                path: path.into(),
                source: error,
            }
        })?;

        // Validate parent schema name syntax (if present)
        if let Some(parent) = self.extends.as_ref() {
            SchemaName::try_new(parent.as_ref()).map_err(|error| {
                SchemaIngestionError::Schema {
                    path: path.into(),
                    source: error,
                }
            })?;
        }

        // Validate excludes syntax
        for excluded in &self.excludes {
            PropertyName::try_new_with_context(
                excluded.as_ref(),
                super::error::PropertyNameContext::Exclude,
            )
            .map_err(|error| SchemaIngestionError::Schema {
                path: path.into(),
                source: error,
            })?;
        }

        Ok(self)
    }
}

/// Raw property bank loaded from vault files.
///
/// Property names are validated during deserialization via `RawPropertyMap`,
/// ensuring all keys are valid `PropertyName` instances.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::RawPropertyBank;
///
/// // Properties are validated during deserialization
/// let toml = r#"
/// [properties.my_property]
/// type = "bool"
/// "#;
/// let bank: RawPropertyBank = toml::from_str(toml)?;
/// // bank.properties() returns HashMap<PropertyName, RawPropertyBankEntry>
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyBank {
    /// Property bank format version (defaults to "1.0" if not specified).
    #[serde(rename = "$version", default)]
    version: RawSchemaVersion,

    /// Validated property map (keys are guaranteed valid `PropertyNames`).
    properties: property::RawPropertyMap<property::RawPropertyBankEntry>,

    /// File metadata for staleness detection.
    ///
    /// Populated during ingestion. Not serialized to TOML.
    #[serde(skip)]
    metadata: RawSchemaMetadata,
}

impl RawPropertyBank {
    /// Returns the schema version.
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::raw::RawPropertyBank;
    /// # let bank: RawPropertyBank = unimplemented!();
    /// assert_eq!(bank.version().as_str(), "1.0");
    /// ```
    #[inline]
    #[must_use]
    pub fn version(&self) -> &RawSchemaVersion {
        &self.version
    }

    /// Returns the properties map.
    ///
    /// All keys are guaranteed to be valid `PropertyName` instances.
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::raw::RawPropertyBank;
    /// # let bank: RawPropertyBank = unimplemented!();
    /// for (name, entry) in bank.properties() {
    ///     // name is &PropertyName - already validated
    ///     println!("{}: {:?}", name.as_str(), entry);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn properties(
        &self,
    ) -> &HashMap<super::property::PropertyName, property::RawPropertyBankEntry>
    {
        self.properties.as_map()
    }

    /// Returns the metadata.
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::raw::RawPropertyBank;
    /// # let bank: RawPropertyBank = unimplemented!();
    /// let metadata = bank.metadata();
    /// println!("Created: {:?}", metadata.created_at);
    /// ```
    #[inline]
    #[must_use]
    pub fn metadata(&self) -> &RawSchemaMetadata {
        &self.metadata
    }

    /// Set metadata (called by Ingestor after deserialization).
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::raw::{RawPropertyBank, RawSchemaMetadata};
    /// # let mut bank: RawPropertyBank = unimplemented!();
    /// # let metadata: RawSchemaMetadata = unimplemented!();
    /// bank.set_metadata(metadata);
    /// ```
    #[inline]
    pub fn set_metadata(&mut self, metadata: RawSchemaMetadata) {
        self.metadata = metadata;
    }

    /// Validate the property bank version matches the expected version.
    ///
    /// This is separate from property key validation (which happens during
    /// deserialization) because version errors need path context for better
    /// error messages.
    ///
    /// # Errors
    /// Returns `SchemaIngestionError::UnsupportedVersion` if the version
    /// does not match.
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::raw::RawPropertyBank;
    /// # let bank: RawPropertyBank = unimplemented!();
    /// bank.validate_version("schemas/property_bank.toml")?;
    /// ```
    #[inline]
    pub fn validate_version(
        &self,
        path: &str,
    ) -> Result<(), SchemaIngestionError> {
        self.version.validate(path)
    }

    /// Consuming constructor that validates the property bank version.
    ///
    /// Property name validation happens during deserialization via
    /// `RawPropertyMap`, so this only validates the version field.
    ///
    /// # Errors
    /// Returns `SchemaIngestionError` if validation fails.
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::raw::RawPropertyBank;
    /// # let bank: RawPropertyBank = unimplemented!();
    /// let bank = bank.validated("schemas/property_bank.toml")?;
    /// ```
    #[inline]
    pub fn validated(self, path: &str) -> Result<Self, SchemaIngestionError> {
        self.validate_version(path)?;
        Ok(self)
    }
}

/// Raw file metadata for staleness detection.
///
/// Populated during ingestion from filesystem metadata.
/// Not part of the serialized TOML format.
///
/// Used for both schema files and property bank files.
///
/// ## Design Note
///
/// This type now only stores file timestamps. Content hashes and property
/// hashes have been moved to `HashMetadata` in the views layer to avoid
/// duplication and keep the Raw* types focused on parsing.
#[derive(Debug, Clone, PartialEq, Default)]
#[non_exhaustive]
pub struct RawSchemaMetadata {
    /// File creation timestamp (birthtime).
    ///
    /// None if the filesystem doesn't support birthtime.
    pub created_at: Option<SystemTime>,

    /// File modification timestamp (mtime).
    pub modified_at: Option<SystemTime>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Version Types
// ─────────────────────────────────────────────────────────────────────────────

/// Schema format version.
///
/// Represents the version string from schema and property bank files.
/// Validates against the supported version constant.
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
    /// Current supported schema version.
    pub const SUPPORTED: &'static str = "1.0";

    /// Validates the version against the expected version.
    ///
    /// # Errors
    /// Returns `SchemaIngestionError::UnsupportedVersion` if the version
    /// does not match the expected version.
    #[inline]
    pub fn validate(&self, path: &str) -> Result<(), SchemaIngestionError> {
        if self.0.as_ref() != Self::SUPPORTED {
            return Err(SchemaIngestionError::Version(
                super::error::SchemaVersionError::UnsupportedVersion {
                    path: path.into(),
                    found: self.0.clone(),
                    expected: Self::SUPPORTED.into(),
                },
            ));
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
        Self(Self::SUPPORTED.into())
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

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test helpers are grouped at the top for readability"
)]
mod tests {
    use super::*;

    // Helper to construct RawSchema using JSON deserialization
    #[expect(
        clippy::indexing_slicing,
        reason = "Test fixture indices into known-fixed JSON object"
    )]
    fn create_raw_schema(
        name: &str,
        extends: Option<&str>,
        excludes: &[&str],
    ) -> RawSchema {
        let mut json = serde_json::json!({
            "$version": "1.0",
            "properties": {}
        });
        if let Some(parent) = extends {
            json["extends"] = serde_json::json!(parent);
        }
        if !excludes.is_empty() {
            json["excludes"] = serde_json::json!(excludes);
        }
        serde_json::from_value::<RawSchema>(json)
            .expect("valid schema JSON")
            .with_name(name.into())
    }

    mod raw_schema {
        use super::*;

        #[test]
        fn defaults_to_empty_excludes() {
            let schema = create_raw_schema("note", None, &[]);

            assert!(
                schema.excludes().is_empty(),
                "RawSchema should have empty excludes by default"
            );
        }

        #[test]
        fn defaults_to_no_extends() {
            let schema = create_raw_schema("note", None, &[]);

            assert!(
                schema.extends().is_none(),
                "RawSchema should have no extends by default"
            );
        }

        #[test]
        fn validate_valid() {
            let schema = create_raw_schema("test_schema", None, &[]);
            schema.validated("test").unwrap();
        }

        #[test]
        fn validate_with_parent() {
            let schema =
                create_raw_schema("child_schema", Some("parent_schema"), &[]);
            schema.validated("test").unwrap();
        }

        #[test]
        fn validate_with_excludes() {
            let schema =
                create_raw_schema("test", Some("parent"), &["prop1", "prop2"]);
            schema.validated("test").unwrap();
        }

        #[test]
        fn validate_invalid_name() {
            let schema = create_raw_schema("Invalid Name", None, &[]); // Uppercase + space
            schema.validated("test").unwrap_err();
        }

        #[test]
        fn validate_invalid_parent_name() {
            let schema =
                create_raw_schema("child", Some("Parent!Invalid"), &[]); // Uppercase + special char
            schema.validated("test").unwrap_err();
        }

        #[test]
        fn validate_invalid_exclude_name() {
            let schema = create_raw_schema("test", Some("parent"), &[
                "Invalid Property",
            ]); // Space
            schema.validated("test").unwrap_err();
        }

        #[test]
        fn validate_with_valid_properties() {
            let json = serde_json::json!({
                "$version": "1.0",
                "properties": {
                    "valid_property": { "type": "bool" }
                }
            });
            let schema = serde_json::from_value::<RawSchema>(json)
                .unwrap()
                .with_name("test".into());
            schema.validated("test").unwrap();
        }

        #[test]
        fn validate_invalid_property_name() {
            // Property name validation now happens during deserialization
            // Invalid property names cannot be constructed via RawPropertyMap
            let json = serde_json::json!({
                "$version": "1.0",
                "name": "test",
                "properties": {
                    "Invalid Property!": { "type": "bool" }  // Space + special char
                }
            });
            // Deserialization should fail
            serde_json::from_value::<RawSchema>(json).unwrap_err();
        }
    }

    mod raw_property_bank {
        use super::*;

        #[test]
        fn validate_valid() {
            let json = serde_json::json!({
                "$version": "1.0",
                "properties": {
                    "title": {
                        "multi": false,
                        "type": "string"
                    }
                }
            });
            let bank: RawPropertyBank = serde_json::from_value(json).unwrap();
            bank.validated("test").unwrap();
        }

        #[test]
        fn validate_empty() {
            let json = serde_json::json!({
                "$version": "1.0",
                "properties": {}
            });
            let bank: RawPropertyBank = serde_json::from_value(json).unwrap();
            bank.validated("test").unwrap();
        }

        #[test]
        fn validate_invalid_property_name() {
            // Property name validation now happens during deserialization
            // Invalid property names cannot be constructed via RawPropertyMap
            let json = serde_json::json!({
                "$version": "1.0",
                "properties": {
                    "Invalid Name!": {  // Space + special char
                        "multi": false,
                        "type": "bool"
                    }
                }
            });
            // Deserialization should fail
            serde_json::from_value::<RawPropertyBank>(json).unwrap_err();
        }
    }
}
