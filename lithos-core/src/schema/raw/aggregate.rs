//! Aggregate raw types for schema and property bank definitions.
//!
//! This module contains the top-level structures used for deserializing
//! schema files and property banks from the vault. These types represent
//! the "aggregate" inputs that combine multiple properties and metadata.

use std::time::SystemTime;

use super::property::{RawProperty, RawPropertyBankEntry, RawPropertyMap};
use crate::schema::{
    aggregate::SchemaName, error::SchemaIngestionError, property::PropertyName,
};

// ─────────────────────────────────────────────────────────────────────────────
// Schema Types
// ─────────────────────────────────────────────────────────────────────────────

/// Raw schema definition loaded from vault files.
///
/// Property names in the `properties` map are validated during deserialization
/// via `RawPropertyMap`. The `extends` and `excludes` fields are validated
/// during deserialization via custom `Deserialize` impls for `SchemaName` and
/// `PropertyName`.
///
/// The `name` field is special - it's derived from the filename (not the file
/// content) and validated in `validated()` with file path context for better
/// error messages.
///
/// # Examples
/// ```
/// use lithos_core::schema::raw::RawSchema;
///
/// // All name fields validated during deserialization
/// let toml = r#"
/// "$version" = "1.0"
/// extends = "base_schema"
/// excludes = ["inherited_prop"]
///
/// [properties.my_property]
/// type = "bool"
/// required = true
/// "#;
///
/// let schema: RawSchema = toml::from_str(toml).unwrap();
/// // If we reach this point, all syntax is valid!
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
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
    /// Validated during deserialization via `SchemaName`'s custom Deserialize
    /// impl.
    extends: Option<SchemaName>,

    /// Property names to exclude from parent schema.
    /// Validated during deserialization via `PropertyName`'s custom
    /// Deserialize impl.
    #[serde(default)]
    excludes: Vec<PropertyName>,

    /// Validated property map (keys are guaranteed valid `PropertyNames`).
    properties: RawPropertyMap<RawProperty>,

    /// File metadata for staleness detection.
    ///
    /// Populated during ingestion. Not serialized to TOML.
    #[serde(skip)]
    file_times: RawFileTimes,
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
    pub fn extends(&self) -> Option<&SchemaName> {
        self.extends.as_ref()
    }

    /// Returns the excluded property names.
    #[inline]
    #[must_use]
    pub fn excludes(&self) -> &[PropertyName] {
        &self.excludes
    }

    /// Returns the properties map.
    ///
    /// All keys are guaranteed to be valid `PropertyName` instances.
    #[inline]
    #[must_use]
    pub fn properties(&self) -> &RawPropertyMap<RawProperty> {
        &self.properties
    }

    /// Returns the file timestamps.
    #[inline]
    #[must_use]
    pub fn file_times(&self) -> &RawFileTimes {
        &self.file_times
    }

    /// Set the schema name (called by Ingestor after deserialization).
    ///
    /// The name is derived from the filename, not the file content.
    #[inline]
    #[must_use]
    pub fn with_name(self, name: Box<str>) -> Self {
        Self {
            name,
            ..self
        }
    }

    /// Set file timestamps (called by Ingestor after deserialization).
    #[inline]
    #[must_use]
    pub fn with_file_times(self, file_times: RawFileTimes) -> Self {
        Self {
            file_times,
            ..self
        }
    }

    /// Consuming constructor that validates the schema name.
    ///
    /// Property names in `properties`, `extends`, and `excludes` are already
    /// validated during deserialization via custom `Deserialize`
    /// implementations. Version is validated during deserialization. This
    /// method only validates:
    /// - Schema name (derived from filename, needs file path context)
    ///
    /// # Errors
    /// Returns `SchemaIngestionError` if validation fails.
    #[inline]
    pub fn validated(self, path: &str) -> Result<Self, SchemaIngestionError> {
        // Validate schema name syntax (filename → SchemaName conversion)
        SchemaName::try_new(self.name.as_ref()).map_err(|error| {
            SchemaIngestionError::Schema {
                path: path.into(),
                source: error,
            }
        })?;

        // extends and excludes are already validated by serde
        // (custom Deserialize impls ensure type safety)

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
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct RawPropertyBank {
    /// Property bank format version (defaults to "1.0" if not specified).
    /// Validated during deserialization via `RawSchemaVersion`'s custom
    /// Deserialize impl.
    #[serde(rename = "$version", default)]
    version: RawSchemaVersion,

    /// Validated property map (keys are guaranteed valid `PropertyNames`).
    properties: RawPropertyMap<RawPropertyBankEntry>,

    /// File metadata for staleness detection.
    ///
    /// Populated during ingestion. Not serialized to TOML.
    #[serde(skip)]
    file_times: RawFileTimes,
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
    /// for (name, entry) in bank.properties().iter() {
    ///     // name is &PropertyName - already validated
    ///     println!("{}: {:?}", name.as_str(), entry);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn properties(&self) -> &RawPropertyMap<RawPropertyBankEntry> {
        &self.properties
    }

    /// Returns the file timestamps.
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::raw::RawPropertyBank;
    /// # let bank: RawPropertyBank = unimplemented!();
    /// let file_times = bank.file_times();
    /// println!("Created: {:?}", file_times.created_at);
    /// ```
    #[inline]
    #[must_use]
    pub fn file_times(&self) -> &RawFileTimes {
        &self.file_times
    }

    /// Set file timestamps (called by Ingestor after deserialization).
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::raw::{RawPropertyBank, RawFileTimes};
    /// # let bank: RawPropertyBank = unimplemented!();
    /// # let file_times: RawFileTimes = unimplemented!();
    /// let bank = bank.with_file_times(file_times);
    /// ```
    #[inline]
    #[must_use]
    pub fn with_file_times(self, file_times: RawFileTimes) -> Self {
        Self {
            file_times,
            ..self
        }
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
pub struct RawFileTimes {
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
/// Validates against the supported version constant during deserialization.
///
/// # Design Note
///
/// This type implements custom `serde::Deserialize` to validate the version
/// string during parsing, following the "parse, don't validate" principle.
/// Invalid versions are rejected immediately with clear error messages.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RawSchemaVersion(Box<str>);

impl RawSchemaVersion {
    /// Current supported schema version.
    pub const SUPPORTED: &'static str = "1.0";

    /// Returns the version string.
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[expect(
    clippy::missing_trait_methods,
    reason = "deserialize_in_place is not applicable for this wrapper"
)]
impl<'de> serde::Deserialize<'de> for RawSchemaVersion {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = Box::<str>::deserialize(deserializer)?;

        if s.as_ref() != Self::SUPPORTED {
            return Err(serde::de::Error::custom(format!(
                "unsupported schema version '{}', expected '{}'",
                s,
                Self::SUPPORTED
            )));
        }

        Ok(Self(s))
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
            // Parent name validation now happens during deserialization
            // Invalid parent names cannot be constructed via SchemaName
            let json = serde_json::json!({
                "$version": "1.0",
                "extends": "Parent!Invalid",  // Uppercase + special char
                "properties": {}
            });
            // Deserialization should fail
            serde_json::from_value::<RawSchema>(json).unwrap_err();
        }

        #[test]
        fn validate_invalid_exclude_name() {
            // Exclude name validation now happens during deserialization
            // Invalid property names cannot be constructed via PropertyName
            let json = serde_json::json!({
                "$version": "1.0",
                "extends": "parent",
                "excludes": ["Invalid Property"],  // Space
                "properties": {}
            });
            // Deserialization should fail
            serde_json::from_value::<RawSchema>(json).unwrap_err();
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
        fn deserializes_valid() {
            let json = serde_json::json!({
                "$version": "1.0",
                "properties": {
                    "title": {
                        "multi": false,
                        "type": "string"
                    }
                }
            });
            let _: RawPropertyBank = serde_json::from_value(json).unwrap();
        }

        #[test]
        fn deserializes_empty() {
            let json = serde_json::json!({
                "$version": "1.0",
                "properties": {}
            });
            let _: RawPropertyBank = serde_json::from_value(json).unwrap();
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
