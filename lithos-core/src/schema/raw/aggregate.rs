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
/// - `extends`: Optional parent schema for inheritance.
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
///     extends: None,
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

    /// Optional parent schema name for inheritance.
    /// Validated during deserialization via `SchemaName`'s custom Deserialize
    /// impl.
    pub extends: Option<SchemaName>,

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
