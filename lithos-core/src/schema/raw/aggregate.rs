//! Root aggregate for raw schema parsing.
//!
//! Provides the [`RawSchema`] type which represents a schema file before
//! inheritance resolution.

use crate::{
    fs::metadata::{FileMetadata, FsTimes},
    schema::{
        error::SchemaIngestionError,
        identifier::SchemaName,
        property::PropertyName,
        raw::{RawProperty, RawPropertyMap, version::RawSchemaVersion},
    },
};

/// Represents a raw schema as parsed from a file.
///
/// This structure captures the serialized form of a schema, including its
/// version, inheritance settings, and property definitions. It is the first
/// stage of the schema ingestion pipeline.
///
/// # Field Policy
///
/// - `version`: Schema format version (defaults to "1.0").
/// - `name`: Derived from filename, not file content.
/// - `extends`: Parent schema list for inheritance.
/// - `excludes`: Properties to exclude from parent.
/// - `properties`: Map of property definitions.
/// - `metadata`: File metadata for staleness detection.
///
/// # Example
///
/// ```ignore
/// # use lithos_core::schema::raw::{RawSchema, RawPropertyMap, RawProperty};
/// # use lithos_core::fs::metadata::{FileMetadata, FsTimes};
/// #
/// # fn example() {
/// // RawSchema is typically parsed from files, not constructed directly
/// // Example shown for illustration purposes only
/// let raw = RawSchema {
///     version: "1.0".into(),
///     name: "User".into(),
///     extends: vec![],
///     excludes: vec![],
///     properties: RawPropertyMap::new(),
///     metadata: FileMetadata::new(FsTimes::new(None, None), 0, false),
/// };
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
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

    /// Parent schema names for inheritance.
    ///
    /// Accepts both legacy single-string input and list input during
    /// deserialization.
    #[serde(default, deserialize_with = "deserialize_extends")]
    pub extends: Vec<SchemaName>,

    /// Property names to exclude from parent schema.
    /// Validated during deserialization via `PropertyName`'s custom
    /// Deserialize impl.
    #[serde(default)]
    pub excludes: Vec<PropertyName>,

    /// Validated property map (keys are guaranteed valid `PropertyNames`).
    pub properties: RawPropertyMap<RawProperty>,

    /// File metadata for staleness detection.
    ///
    /// Populated during ingestion. Not serialized to TOML.
    #[serde(skip, default = "default_metadata")]
    pub metadata: FileMetadata,
}

#[inline]
const fn default_metadata() -> FileMetadata {
    FileMetadata::new(FsTimes::new(None, None), 0, false)
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

    /// Returns parent schema names.
    #[inline]
    #[must_use]
    pub fn extends(&self) -> &[SchemaName] {
        &self.extends
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

    /// Returns the file metadata.
    #[inline]
    #[must_use]
    pub fn metadata(&self) -> &FileMetadata {
        &self.metadata
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

    /// Set file metadata (called by Ingestor after deserialization).
    #[inline]
    #[must_use]
    pub fn with_metadata(self, metadata: FileMetadata) -> Self {
        Self {
            metadata,
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

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum ExtendsInput {
    Single(SchemaName),
    Multiple(Vec<SchemaName>),
}

fn deserialize_extends<'de, D>(
    deserializer: D,
) -> Result<Vec<SchemaName>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = <Option<ExtendsInput> as serde::Deserialize>::deserialize(
        deserializer,
    )?;
    Ok(match value {
        None => Vec::new(),
        Some(ExtendsInput::Single(parent)) => vec![parent],
        Some(ExtendsInput::Multiple(parents)) => parents,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    mod deserialization {
        use super::*;

        #[test]
        fn returns_empty_extends_when_field_missing() {
            let raw: RawSchema =
                serde_json::from_str(r#"{"properties":{},"excludes":[]}"#)
                    .unwrap();

            assert!(raw.extends().is_empty());
        }

        #[test]
        fn returns_single_parent_when_extends_is_string() {
            let raw: RawSchema = serde_json::from_str(
                r#"{"extends":"base","properties":{},"excludes":[]}"#,
            )
            .unwrap();

            assert_eq!(raw.extends().len(), 1);
            assert_eq!(
                raw.extends().first().map(SchemaName::as_str),
                Some("base")
            );
        }

        #[test]
        fn returns_all_parents_when_extends_is_list() {
            let raw: RawSchema = serde_json::from_str(
                r#"{"extends":["base","shared"],"properties":{},"excludes":[]}"#,
            )
            .unwrap();

            assert_eq!(raw.extends().len(), 2);
            assert_eq!(
                raw.extends().first().map(SchemaName::as_str),
                Some("base")
            );
            assert_eq!(
                raw.extends().get(1).map(SchemaName::as_str),
                Some("shared")
            );
        }
    }
}
