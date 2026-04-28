//! Aggregate raw types for schema definitions.

use super::{
    property::{RawProperty, RawPropertyMap},
    version::RawSchemaVersion,
};
use crate::{
    fs::FileStats,
    schema::{
        error::SchemaIngestionError, identifier::SchemaName,
        property::PropertyName,
    },
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
/// Property definitions are either inline (`RawPropertyInline`) or references
/// (`RawPropertyRef`) via `RawProperty`.
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
    #[serde(skip, default = "default_file_stats")]
    file_stats: FileStats,
}

#[inline]
const fn default_file_stats() -> FileStats {
    FileStats::new(None, None, 0)
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

    /// Returns the file stats.
    #[inline]
    #[must_use]
    pub fn file_stats(&self) -> &FileStats {
        &self.file_stats
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

    /// Set file stats (called by Ingestor after deserialization).
    #[inline]
    #[must_use]
    pub fn with_file_stats(self, file_stats: FileStats) -> Self {
        Self {
            file_stats,
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
}
