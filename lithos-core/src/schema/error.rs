//! Error types for the Schema domain and its ingestion pipeline.
//!
//! This module provides a hierarchical, phase-oriented error taxonomy that
//! maps to the Schema lifecycle:
//!
//! 1. **File Access**: Physical file access, filename validation, and format
//!    filtering ([`SchemaFileError`]).
//! 2. **Parsing**: Structured decoding of JSON/TOML/YAML and cached views
//!    ([`SchemaParseError`]).
//! 3. **Versioning**: Schema version compatibility checks
//!    ([`SchemaVersionError`]).
//! 4. **Syntax**: Name and shape validation for raw schema inputs
//!    ([`SchemaSyntaxError`]).
//! 5. **Validation/Resolution**: Domain-level validation, property resolution,
//!    and inheritance checks ([`SchemaError`] umbrella).
//! 6. **Persistence**: Storage integrity and lookup behavior
//!    ([`SchemaRepositoryError`]).
//! 7. **Orchestration**: Top-level coordination of the full pipeline
//!    ([`SchemaLoaderError`]).
//!
//! # Hierarchy
//!
//! ```text
//! SchemaLoaderError (Orchestration)
//!  ├── SchemaIngestionError (Ingestion Phase)
//!  │    ├── SchemaFileError (I/O & Naming)
//!  │    ├── SchemaParseError (Syntax & Extraction)
//!  │    ├── SchemaVersionError (Version Gate)
//!  │    ├── SchemaSyntaxError (Raw Validation)
//!  │    └── SchemaError (Domain Validation)
//!  ├── SchemaError (Domain Umbrella)
//!  │    ├── PropertySpecError, PropertyValueError
//!  │    ├── PropertyRefError, PropertyBankError
//!  │    └── SchemaInheritanceError, SchemaResolutionError
//!  └── SchemaRepositoryError (Persistence Phase)
//!       └── DbError
//! ```
//!
//! # Design Principles
//!
//! - **Context Preservation**: Each layer wraps the previous one using
//!   `#[error(transparent)]` to preserve the `source()` chain.
//! - **Performance**: Dynamic error data uses `Box<str>` instead of `String` to
//!   reduce heap allocations.
//! - **Phase Orientation**: Errors are categorized by pipeline stage, avoiding
//!   the “everything is ingestion” anti-pattern.
//!
//! # Usage Guidelines
//!
//! | Error Type                | When to Use                                                         |
//! | :------------------------ | :------------------------------------------------------------------ |
//! | [`SchemaError`]           | Domain validation, property resolution, and inheritance handling.   |
//! | [`SchemaIngestionError`]  | Readers/parsers bridging raw bytes to structured schema data.       |
//! | [`SchemaRepositoryError`] | Storage adapters and persistence integrity checks.                  |
//! | [`SchemaLoaderError`]     | Cross-cutting orchestration across the entire lifecycle.            |
//!
//! # Examples
//!
//! ## Handling a Load Failure
//!
//! ```ignore
//! match loader.load_all() {
//!     Err(SchemaLoaderError::Ingestion(e)) => handle_ingestion_error(e),
//!     Err(SchemaLoaderError::Repository(e)) => handle_storage_error(e),
//!     Err(SchemaLoaderError::Resolution(e)) => handle_schema_error(e),
//!     Ok(count) => println!("Loaded {count} schemas"),
//! }
//! ```

use std::path::PathBuf;

use crate::db::DbError;

/// Domain-level errors for in-memory schema validation and resolution.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum SchemaError {
    /// Returned when raw syntax validation fails.
    #[error(transparent)]
    Syntax(#[from] SchemaSyntaxError),

    /// Returned when property specification configuration is invalid.
    #[error(transparent)]
    PropertySpec(#[from] PropertySpecError),

    /// Returned when a value fails validation against a spec.
    #[error(transparent)]
    PropertyValue(#[from] PropertyValueError),

    /// Returned when a property reference is invalid or unresolved.
    #[error(transparent)]
    PropertyRef(#[from] PropertyRefError),

    /// Returned when property bank constraints are violated.
    #[error(transparent)]
    PropertyBank(#[from] PropertyBankError),

    /// Returned when inheritance resolution fails.
    #[error(transparent)]
    Inheritance(#[from] SchemaInheritanceError),

    /// Returned when schema resolution fails outside inheritance concerns.
    #[error(transparent)]
    Resolution(#[from] SchemaResolutionError),
}

/// Schema ingestion error with file context.
///
/// This error type preserves context throughout the ingestion pipeline:
/// - **Parse errors**: Contain line/column from serde
/// - **Validation errors**: Contain file path from ingestor
/// - **`FileName` errors**: Contain full path for user feedback
///
/// # Error Chain Example
///
/// ```text
/// SchemaIngestionError::Parse {
///     path: "schemas/note.toml",
///     source: "invalid schema name in extends field at line 5, column 10"
/// }
/// ```
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaIngestionError {
    /// Returned when filesystem access fails.
    #[error(transparent)]
    File(#[from] SchemaFileError),

    /// Returned when structured parsing fails.
    #[error(transparent)]
    Parse(#[from] SchemaParseError),

    /// Returned when schema version validation fails.
    #[error(transparent)]
    Version(#[from] SchemaVersionError),

    /// Returned when syntax validation fails.
    #[error(transparent)]
    Syntax(#[from] SchemaSyntaxError),

    /// Returned when repository access fails during ingestion.
    #[error(transparent)]
    Repository(#[from] SchemaRepositoryError),

    /// Returned when schema validation fails with file context.
    #[error("schema validation failed in {path}: {source}")]
    Schema {
        /// Path to the schema file that failed validation.
        path: PathBuf,
        /// Underlying schema error.
        #[source]
        source: SchemaError,
    },
}

/// Errors returned by schema repository implementations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaRepositoryError {
    /// Returned when the underlying storage layer fails.
    #[error(transparent)]
    Storage(#[from] DbError),

    /// Returned when domain validation fails while saving or loading.
    #[error(transparent)]
    Domain(#[from] SchemaError),

    /// Returned when an expected entity is missing by ID.
    #[error("schema not found: {0}")]
    NotFoundById(crate::schema::identifier::SchemaId),

    /// Returned when an expected entity is missing by name.
    #[error("schema name not found: {0}")]
    NotFoundByName(crate::schema::identifier::SchemaName),

    /// Returned when an expected entity is missing by path.
    #[error("schema path not found: {0}")]
    NotFoundByPath(crate::fs::RelativePath),

    /// Returned when the property bank has not been initialized.
    #[error(
        "PropertyBank not found in database - initialize by loading schema \
         files or creating properties"
    )]
    PropertyBankNotFound,

    /// Returned when the version history is missing or empty for a view.
    #[error(
        "version history missing for {0} - cached view is corrupt or empty"
    )]
    EmptyVersionHistory(crate::fs::RelativePath),
}

/// High-level errors returned by schema loading operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaLoaderError {
    /// Returned when ingestion (file I/O or parsing) fails.
    #[error(transparent)]
    Ingestion(#[from] SchemaIngestionError),

    /// Returned when repository access fails.
    #[error(transparent)]
    Repository(#[from] SchemaRepositoryError),

    /// Returned when schema resolution fails.
    #[error(transparent)]
    Resolution(#[from] SchemaError),
}

impl From<SchemaInheritanceError> for SchemaLoaderError {
    #[inline]
    fn from(value: SchemaInheritanceError) -> Self {
        Self::Resolution(SchemaError::Inheritance(value))
    }
}

/// File-system and filename errors during ingestion.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaFileError {
    /// Returned when a schema file cannot be read.
    #[error("failed to read {path}: {source}")]
    Io {
        /// Path to the file that failed to read.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Returned when a filename or basename is invalid.
    #[error("invalid filename: {path} ({reason})")]
    InvalidFileName {
        /// Path to the file with an invalid name.
        path: PathBuf,
        /// Reason the filename was rejected.
        reason: Box<str>,
    },

    /// Returned when the file format is not supported.
    #[error("unsupported format for {path}: expected one of {supported:?}")]
    UnsupportedFormat {
        /// Path to the file with unsupported format.
        path: PathBuf,
        /// Supported formats.
        supported: Vec<Box<str>>,
    },

    /// Returned for filesystem errors not tied to a specific file.
    #[error("filesystem error: {reason}")]
    FileSystem {
        /// Error details from the filesystem layer.
        reason: Box<str>,
    },

    /// Path was not within the expected base directory.
    #[error("path {path} is not within base directory {base}")]
    NotInBasePath {
        /// The path that was outside the base.
        path: PathBuf,
        /// The expected base directory.
        base: PathBuf,
    },
}

/// Structured parse errors with file location context.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum SchemaParseError {
    /// Returned when JSON parsing fails.
    #[error(
        "JSON parse error in {path} at line {line:?}, column {column:?}: \
         {message}"
    )]
    Json {
        /// Path to the file being parsed.
        path: PathBuf,
        /// Parser error message.
        message: Box<str>,
        /// Line number (if available).
        line: Option<usize>,
        /// Column number (if available).
        column: Option<usize>,
    },

    /// Returned when TOML parsing fails.
    #[error(
        "TOML parse error in {path} at line {line:?}, column {column:?}: \
         {message}"
    )]
    Toml {
        /// Path to the file being parsed.
        path: PathBuf,
        /// Parser error message.
        message: Box<str>,
        /// Line number (if available).
        line: Option<usize>,
        /// Column number (if available).
        column: Option<usize>,
    },

    /// Returned when YAML parsing fails.
    #[error(
        "YAML parse error in {path} at line {line:?}, column {column:?}: \
         {message}"
    )]
    Yaml {
        /// Path to the file being parsed.
        path: PathBuf,
        /// Parser error message.
        message: Box<str>,
        /// Line number (if available).
        line: Option<usize>,
        /// Column number (if available).
        column: Option<usize>,
    },

    /// Returned when cached view deserialization fails.
    #[error("cached view parse error in {path}: {reason}")]
    CachedView {
        /// Path to the cached view file.
        path: PathBuf,
        /// Deserialization error details.
        reason: Box<str>,
    },

    /// Returned when serialization of a cached view fails.
    #[error("serialization error in {path}: {reason}")]
    Serialization {
        /// Path to the file being serialized.
        path: PathBuf,
        /// Serialization error details.
        reason: Box<str>,
    },
}

/// Schema version validation errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaVersionError {
    /// Returned when the schema version is unsupported.
    #[error(
        "unsupported schema version in {path}: got '{found}', expected \
         '{expected}'"
    )]
    UnsupportedVersion {
        /// Path to the file with an unsupported version.
        path: PathBuf,
        /// Version found in the file.
        found: Box<str>,
        /// Expected version value.
        expected: Box<str>,
    },
}

/// Syntax validation failures in raw schema inputs.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaSyntaxError {
    /// Returned when the schema name is invalid.
    #[error(transparent)]
    SchemaName(#[from] SchemaNameError),

    /// Returned when a property name is invalid.
    #[error(transparent)]
    PropertyName(#[from] PropertyNameError),
}

/// Schema name validation failures.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaNameError {
    /// Returned when the schema name is empty.
    #[error("schema name cannot be empty")]
    Empty,

    /// Returned when the schema name exceeds the maximum length.
    #[error("schema name too long: {len} (max {max})")]
    TooLong {
        /// Length of the provided name.
        len: usize,
        /// Maximum allowed length.
        max: usize,
    },

    /// Returned when the schema name does not match the expected format.
    #[error("invalid schema name: {name}")]
    InvalidFormat {
        /// The invalid name.
        name: Box<str>,
    },

    /// Returned when the schema name regex fails to compile.
    #[error("invalid schema name regex: {reason}")]
    InvalidRegex {
        /// Regex error details.
        reason: Box<str>,
    },
}

/// Identifies where a property name was used for better diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PropertyNameContext {
    /// Property name defined in a schema file.
    SchemaProperty,
    /// Property name defined in a property bank file.
    PropertyBank,
    /// Property name used in an excludes list.
    Exclude,
}

impl std::fmt::Display for PropertyNameContext {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match *self {
            Self::SchemaProperty => "schema property",
            Self::PropertyBank => "property bank",
            Self::Exclude => "exclude list",
        };
        f.write_str(label)
    }
}

/// Property name validation failures.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PropertyNameError {
    /// Returned when the property name is empty.
    #[error("property name cannot be empty ({context})")]
    Empty {
        /// Context of the property name.
        context: PropertyNameContext,
    },

    /// Returned when the property name exceeds the maximum length.
    #[error("property name too long: {len} (max {max}) ({context})")]
    TooLong {
        /// Length of the provided name.
        len: usize,
        /// Maximum allowed length.
        max: usize,
        /// Context of the property name.
        context: PropertyNameContext,
    },

    /// Returned when the property name does not match the expected format.
    #[error("invalid property name: {name} ({context})")]
    InvalidFormat {
        /// The invalid name.
        name: Box<str>,
        /// Context of the property name.
        context: PropertyNameContext,
    },

    /// Returned when the property name regex fails to compile.
    #[error("invalid property name regex: {reason}")]
    InvalidRegex {
        /// Regex error details.
        reason: Box<str>,
    },
}

/// Property spec configuration failures.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum PropertySpecError {
    /// Returned when a date spec omits the required format.
    #[error("date format is required")]
    DateFormatRequired,

    /// Returned when a date format is invalid.
    #[error("invalid date format: {format}")]
    InvalidDateFormat {
        /// The invalid format string.
        format: Box<str>,
    },

    /// Returned when a regex pattern fails to compile.
    #[error("invalid regex pattern: {pattern} ({reason})")]
    InvalidRegex {
        /// Regex pattern string.
        pattern: Box<str>,
        /// Regex error details.
        reason: Box<str>,
    },

    /// Returned when an options list is empty.
    #[error("options list cannot be empty")]
    OptionsEmpty,

    /// Returned when an option value does not match a pattern constraint.
    #[error("option value '{value}' does not match pattern {pattern}")]
    OptionPatternMismatch {
        /// Option value.
        value: Box<str>,
        /// Pattern string.
        pattern: Box<str>,
    },

    /// Returned when an option value is empty or whitespace.
    #[error("option value cannot be empty")]
    OptionValueEmpty,

    /// Returned when an option order key is not a valid integer.
    #[error("option order key must be an integer: {key}")]
    InvalidOptionsEntryOrderType {
        /// The invalid key.
        key: Box<str>,
    },

    /// Returned when an option order key is less than 1.
    #[error("option order key must be >= 1: {order}")]
    InvalidOptionsEntryOrderValue {
        /// The invalid order value.
        order: u32,
    },

    /// Returned when a directory path constraint is invalid.
    #[error("invalid directory path: {path}")]
    InvalidDirectoryPath {
        /// The invalid directory path.
        path: Box<str>,
    },

    /// Returned when a numeric range is invalid.
    #[error("invalid range: min {min} cannot be greater than max {max}")]
    InvalidRange {
        /// Minimum value.
        min: f64,
        /// Maximum value.
        max: f64,
    },

    /// Returned when a numeric constraint is non-finite.
    #[error("{context} must be finite: {value}")]
    NonFinite {
        /// Provided value.
        value: f64,
        /// Context label.
        context: Box<str>,
    },

    /// Returned when a file class constraint is invalid.
    #[error("invalid file class: {class}")]
    InvalidFileClass {
        /// The invalid file class.
        class: Box<str>,
    },

    /// Returned when an archived spec fails to deserialize.
    #[error("failed to deserialize {spec}: {reason}")]
    Deserialization {
        /// Spec name.
        spec: &'static str,
        /// Deserialization error details.
        reason: Box<str>,
    },
}

/// Property value validation failures.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum PropertyValueError {
    /// Returned when a value has the wrong type.
    #[error("invalid type: {value} (expected: {expected})")]
    InvalidType {
        /// Provided value representation.
        value: Box<str>,
        /// Expected type representation.
        expected: Box<str>,
    },

    /// Returned when a value is not in the allowed options list.
    #[error("invalid enum value: {value} (allowed: {allowed:?})")]
    InvalidEnumValue {
        /// Provided value.
        value: Box<str>,
        /// Allowed values.
        allowed: Vec<Box<str>>,
    },

    /// Returned when a value does not match a pattern constraint.
    #[error("value {value} does not match pattern {pattern}")]
    PatternMismatch {
        /// Provided value.
        value: Box<str>,
        /// Pattern string.
        pattern: Box<str>,
    },

    /// Returned when a value does not match a date format.
    #[error("value {value} does not match format {format}")]
    DateFormatMismatch {
        /// Provided value.
        value: Box<str>,
        /// Expected format.
        format: Box<str>,
    },

    /// Returned when a numeric value is out of range.
    #[error("number out of range: {value} (min: {min:?}, max: {max:?})")]
    NumberOutOfRange {
        /// Provided value.
        value: f64,
        /// Minimum allowed value.
        min: Option<f64>,
        /// Maximum allowed value.
        max: Option<f64>,
    },

    /// Returned when a numeric value does not align with the step constraint.
    #[error("invalid step value: {value} (step: {step})")]
    InvalidStepValue {
        /// Provided value.
        value: f64,
        /// Step constraint.
        step: f64,
    },

    /// Returned when a numeric value is non-finite.
    #[error("{context} must be finite: {value}")]
    NonFinite {
        /// Provided value.
        value: f64,
        /// Context label.
        context: Box<str>,
    },
}

/// Errors for `$ref` property references.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PropertyRefError {
    /// Returned when a `$ref` string is not in the expected format.
    #[error(
        "invalid property reference: '{reference}' (expected format: \
         #property_bank/<name>)"
    )]
    InvalidFormat {
        /// The invalid reference string.
        reference: Box<str>,
    },

    /// Returned when the referenced property does not exist.
    #[error("property reference not found: {reference}")]
    NotFound {
        /// The reference string.
        reference: Box<str>,
    },

    /// Returned when a `$ref` override changes the property type.
    #[error(
        "cannot change property type via $ref override: expected {expected}, \
         got {actual}"
    )]
    TypeMismatch {
        /// Expected type.
        expected: Box<str>,
        /// Actual override type.
        actual: Box<str>,
    },
}

/// Errors raised by property bank operations.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PropertyBankError {
    /// Returned when a property name is defined more than once.
    #[error("duplicate property name: {name}")]
    DuplicatePropertyName {
        /// Property name.
        name: Box<str>,
    },

    /// Returned when a property ID is reused for a different name.
    #[error("duplicate property id: {id}")]
    DuplicatePropertyId {
        /// Property id.
        id: Box<str>,
    },
}

/// Errors related to schema inheritance resolution.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaInheritanceError {
    /// Returned when a schema node is missing from the graph.
    #[error("missing schema node for id {id}")]
    MissingNode {
        /// Schema identifier.
        id: crate::schema::identifier::SchemaId,
    },

    /// Returned when a cycle is detected in the inheritance graph.
    #[error("cycle detected in schema inheritance graph")]
    CycleDetected {
        /// Nodes involved in the cycle.
        nodes: Vec<crate::schema::identifier::SchemaId>,
    },

    /// Returned when the inheritance graph is not directed.
    #[error("inheritance graph is not directed")]
    NotDirected,

    /// Returned when a declared parent schema is missing.
    #[error("parent not found: {name}")]
    ParentNotFound {
        /// Parent schema name.
        name: Box<str>,
    },

    /// Returned when inheritance cycles are detected.
    #[error("circular schema inheritance detected: {name}")]
    CircularInheritance {
        /// Schema name involved in the cycle.
        name: Box<str>,
    },

    /// Returned when inheritance depth exceeds the maximum.
    #[error("inheritance depth exceeded: {depth} (max: {max})")]
    DepthExceeded {
        /// Actual depth.
        depth: usize,
        /// Maximum allowed depth.
        max: usize,
    },
}

impl TryFrom<crate::graph::GraphError<crate::schema::identifier::SchemaId>>
    for SchemaError
{
    type Error = SchemaError;

    #[inline]
    fn try_from(
        err: crate::graph::GraphError<crate::schema::identifier::SchemaId>,
    ) -> Result<Self, Self::Error> {
        let inheritance = match err {
            crate::graph::GraphError::CycleDetected {
                nodes,
            } => SchemaInheritanceError::CycleDetected {
                nodes,
            },
            crate::graph::GraphError::NotDirected => {
                SchemaInheritanceError::NotDirected
            }
            crate::graph::GraphError::MissingNode {
                id,
            } => SchemaInheritanceError::MissingNode {
                id,
            },
        };

        Ok(SchemaError::Inheritance(inheritance))
    }
}

/// Resolution errors not covered by other categories.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SchemaResolutionError {
    /// Returned when duplicate schema names are detected.
    #[error("duplicate schema name: {name}")]
    DuplicateSchemaName {
        /// Schema name.
        name: Box<str>,
    },

    /// Returned when a schema node is missing during merge.
    #[error("schema node missing for id {id}")]
    MissingNode {
        /// Schema ID.
        id: crate::schema::identifier::SchemaId,
    },

    /// Returned when a parent schema is not found.
    #[error("parent schema '{parent}' not found for schema '{child}'")]
    ParentNotFound {
        /// Child schema name.
        child: crate::schema::identifier::SchemaName,
        /// Parent schema name.
        parent: crate::schema::identifier::SchemaName,
    },

    /// Returned when a cycle is detected in the inheritance graph.
    #[error("cycle detected in schema inheritance: {}", schemas.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", "))]
    CycleDetected {
        /// Schemas involved in the cycle.
        schemas: Vec<crate::schema::identifier::SchemaName>,
    },

    /// Returned when the graph is not directed.
    #[error("graph is not directed")]
    NotDirected,
}

impl From<SchemaNameError> for SchemaError {
    #[inline]
    fn from(err: SchemaNameError) -> Self {
        Self::Syntax(err.into())
    }
}

impl From<PropertyNameError> for SchemaError {
    #[inline]
    fn from(err: PropertyNameError) -> Self {
        Self::Syntax(err.into())
    }
}

impl From<crate::fs::error::ParseError> for SchemaIngestionError {
    #[inline]
    fn from(err: crate::fs::error::ParseError) -> Self {
        use crate::fs::error::ParseError;
        match err {
            ParseError::Json {
                path,
                message,
                line,
                column,
            } => Self::Parse(SchemaParseError::Json {
                path,
                message,
                line,
                column,
            }),
            ParseError::Toml {
                path,
                message,
                line,
                column,
            } => Self::Parse(SchemaParseError::Toml {
                path,
                message,
                line,
                column,
            }),
            ParseError::Yaml {
                path,
                message,
                line,
                column,
            } => Self::Parse(SchemaParseError::Yaml {
                path,
                message,
                line,
                column,
            }),
            ParseError::UnsupportedFormat {
                path,
                supported,
            } => Self::File(SchemaFileError::UnsupportedFormat {
                path,
                supported: supported.iter().map(|&s| s.into()).collect(),
            }),
        }
    }
}

impl From<crate::fs::error::ReadError> for SchemaIngestionError {
    #[inline]
    fn from(err: crate::fs::error::ReadError) -> Self {
        use crate::fs::error::ReadError;
        match err {
            ReadError::Io {
                path,
                source,
            } => Self::File(SchemaFileError::Io {
                path,
                source,
            }),
            ReadError::RootScope(
                crate::fs::error::RootScopeError::PathOutsideVaultRootBoundary {
                    path,
                    root,
                },
            ) => Self::File(SchemaFileError::NotInBasePath { path, base: root }),
        }
    }
}

impl From<crate::fs::error::FsError> for SchemaIngestionError {
    #[inline]
    fn from(err: crate::fs::error::FsError) -> Self {
        use crate::fs::error::FsError;
        match err {
            FsError::Read(e) => Self::from(e),
            FsError::Scan(e) => Self::from(e),
            FsError::Parse(e) => Self::from(e),
            FsError::Path(e) => Self::from(e),
            FsError::Validation(e) => Self::File(SchemaFileError::Io {
                path: std::path::PathBuf::from("unknown"),
                source: std::io::Error::other(e.to_string()),
            }),
            FsError::RootScope(
                crate::fs::error::RootScopeError::PathOutsideVaultRootBoundary {
                    path,
                    root,
                },
            ) => Self::File(SchemaFileError::NotInBasePath { path, base: root }),
        }
    }
}

impl From<crate::fs::error::ScanError> for SchemaIngestionError {
    #[inline]
    fn from(err: crate::fs::error::ScanError) -> Self {
        use crate::fs::error::ScanError;
        match err {
            ScanError::Traversal {
                path,
                source,
            } => Self::File(SchemaFileError::Io {
                path,
                source,
            }),
            ScanError::InvalidPattern {
                pattern,
                message,
            } => Self::File(SchemaFileError::Io {
                path: std::path::PathBuf::from(pattern.as_ref()),
                source: std::io::Error::other(message.as_ref()),
            }),
            ScanError::UnsupportedEntryType(path) => {
                Self::File(SchemaFileError::Io {
                    path,
                    source: std::io::Error::other("Unsupported entry type"),
                })
            }
            ScanError::Path(e) => Self::from(e),
        }
    }
}

impl From<crate::fs::error::PathError> for SchemaIngestionError {
    #[inline]
    fn from(err: crate::fs::error::PathError) -> Self {
        Self::File(SchemaFileError::Io {
            path: std::path::PathBuf::from("unknown"),
            source: std::io::Error::other(err.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod thread_safety {
        use super::*;

        #[test]
        fn errors_are_send_and_sync() {
            fn assert_send_sync<T: Send + Sync>() {}

            assert_send_sync::<SchemaError>();
            assert_send_sync::<SchemaIngestionError>();
            assert_send_sync::<SchemaRepositoryError>();
            assert_send_sync::<SchemaLoaderError>();
            assert_send_sync::<SchemaFileError>();
            assert_send_sync::<SchemaParseError>();
            assert_send_sync::<SchemaVersionError>();
            assert_send_sync::<SchemaSyntaxError>();
            assert_send_sync::<SchemaNameError>();
            assert_send_sync::<PropertyNameContext>();
            assert_send_sync::<PropertyNameError>();
            assert_send_sync::<PropertySpecError>();
            assert_send_sync::<PropertyValueError>();
            assert_send_sync::<PropertyRefError>();
            assert_send_sync::<PropertyBankError>();
            assert_send_sync::<SchemaInheritanceError>();
            assert_send_sync::<SchemaResolutionError>();
        }
    }

    mod formatting {
        use rstest::rstest;

        use super::*;

        #[rstest]
        #[case::schema_property(
            PropertyNameContext::SchemaProperty,
            "schema property"
        )]
        #[case::property_bank(
            PropertyNameContext::PropertyBank,
            "property bank"
        )]
        #[case::exclude(PropertyNameContext::Exclude, "exclude list")]
        fn property_name_context_formats(
            #[case] context: PropertyNameContext,
            #[case] expected: &str,
        ) {
            assert_eq!(
                context.to_string(),
                expected,
                "Expected context label '{expected}', got '{context}'"
            );
        }

        #[rstest]
        #[case::schema_name_empty(
            SchemaError::Syntax(SchemaSyntaxError::SchemaName(
                SchemaNameError::Empty
            )),
            "schema name cannot be empty"
        )]
        #[case::property_name_empty(
            SchemaError::Syntax(SchemaSyntaxError::PropertyName(
                PropertyNameError::Empty {
                    context: PropertyNameContext::SchemaProperty,
                }
            )),
            "property name cannot be empty"
        )]
        #[case::property_ref_invalid(
            SchemaError::PropertyRef(PropertyRefError::InvalidFormat {
                reference: "#property_bank/title".into(),
            }),
            "invalid property reference"
        )]
        #[case::inheritance_parent_missing(
            SchemaError::Inheritance(SchemaInheritanceError::ParentNotFound {
                name: "parent".into(),
            }),
            "parent not found"
        )]
        fn schema_error_display_contains_message(
            #[case] error: SchemaError,
            #[case] expected_fragment: &str,
        ) {
            let rendered = error.to_string();
            assert!(
                rendered.contains(expected_fragment),
                "Expected display to contain '{expected_fragment}', got: \
                 {rendered}"
            );
        }

        #[rstest]
        #[case::not_found_by_id(
            SchemaRepositoryError::NotFoundById(
                crate::schema::identifier::SchemaId::new()
            ),
            "schema not found:"
        )]
        #[case::not_found_by_name(
            SchemaRepositoryError::NotFoundByName(crate::schema::identifier::SchemaName::try_new("test").unwrap()),
            "schema name not found: test"
        )]
        #[case::not_found_by_path(
            SchemaRepositoryError::NotFoundByPath(crate::fs::RelativePath::try_from("test.json").unwrap()),
            "schema path not found: test.json"
        )]
        #[case::empty_version_history(
            SchemaRepositoryError::EmptyVersionHistory(crate::fs::RelativePath::try_from("test.json").unwrap()),
            "version history missing for test.json"
        )]
        fn repository_error_display_contains_message(
            #[case] error: SchemaRepositoryError,
            #[case] expected_fragment: &str,
        ) {
            let rendered = error.to_string();
            assert!(
                rendered.contains(expected_fragment),
                "Expected display to contain '{expected_fragment}', got: \
                 {rendered}"
            );
        }
    }

    mod sources {
        use std::error::Error as StdError;

        use super::*;

        #[test]
        fn schema_file_io_exposes_source() {
            let error = SchemaFileError::Io {
                path: PathBuf::from("schemas/bad.json"),
                source: std::io::Error::other("disk failed"),
            };

            let source = StdError::source(&error);
            assert!(
                source.is_some(),
                "Expected source for SchemaFileError::Io"
            );

            if let Some(source) = source {
                assert!(
                    source.to_string().contains("disk failed"),
                    "Expected source message to include 'disk failed', got: \
                     {source}"
                );
            }
        }

        #[test]
        fn ingestion_schema_exposes_source() {
            let error = SchemaIngestionError::Schema {
                path: PathBuf::from("schemas/bad.json"),
                source: SchemaError::PropertyRef(PropertyRefError::NotFound {
                    reference: "#property_bank/title".into(),
                }),
            };

            let source = StdError::source(&error);
            assert!(
                source.is_some(),
                "Expected source for SchemaIngestionError::Schema"
            );

            if let Some(source) = source {
                assert!(
                    source.to_string().contains("property reference not found"),
                    "Expected source message to mention missing reference, \
                     got: {source}"
                );
            }
        }

        #[test]
        fn schema_file_filesystem_has_no_source() {
            let error = SchemaFileError::FileSystem {
                reason: "vault unavailable".into(),
            };

            assert!(
                StdError::source(&error).is_none(),
                "Expected no source for SchemaFileError::FileSystem"
            );
        }
    }

    mod equality {
        use super::*;

        #[test]
        fn schema_error_equality_compares_variants() {
            let left = SchemaError::PropertyRef(PropertyRefError::NotFound {
                reference: "#property_bank/title".into(),
            });
            let right = SchemaError::PropertyRef(PropertyRefError::NotFound {
                reference: "#property_bank/title".into(),
            });

            assert_eq!(
                left, right,
                "Expected identical property reference errors to be equal"
            );
        }

        #[test]
        fn schema_error_equality_distinguishes_variants() {
            let left = SchemaError::PropertyRef(PropertyRefError::NotFound {
                reference: "#property_bank/title".into(),
            });
            let right = SchemaError::PropertyRef(PropertyRefError::NotFound {
                reference: "#property_bank/status".into(),
            });

            assert!(
                left != right,
                "Expected different references to be unequal: {left:?} vs \
                 {right:?}"
            );
        }
    }

    mod conversions {
        use super::*;

        #[test]
        fn schema_name_error_converts_into_schema_error() {
            let error: SchemaError = SchemaNameError::Empty.into();

            assert!(
                matches!(
                    error,
                    SchemaError::Syntax(SchemaSyntaxError::SchemaName(
                        SchemaNameError::Empty
                    ))
                ),
                "Expected SchemaNameError::Empty to convert into \
                 SchemaError::Syntax"
            );
        }

        #[test]
        fn property_name_error_converts_into_schema_error() {
            let error: SchemaError = PropertyNameError::Empty {
                context: PropertyNameContext::SchemaProperty,
            }
            .into();

            assert!(
                matches!(
                    error,
                    SchemaError::Syntax(SchemaSyntaxError::PropertyName(
                        PropertyNameError::Empty {
                            context: PropertyNameContext::SchemaProperty,
                        }
                    ))
                ),
                "Expected PropertyNameError::Empty to convert into \
                 SchemaError::Syntax"
            );
        }

        #[test]
        fn parse_error_maps_to_schema_ingestion_error() {
            let error: SchemaIngestionError =
                crate::fs::error::ParseError::UnsupportedFormat {
                    path: PathBuf::from("schemas/schema.xml"),
                    supported: &["json", "yaml"],
                }
                .into();

            match error {
                SchemaIngestionError::File(
                    SchemaFileError::UnsupportedFormat {
                        path,
                        supported,
                    },
                ) => {
                    assert_eq!(
                        path,
                        PathBuf::from("schemas/schema.xml"),
                        "Expected unsupported format to preserve the path"
                    );
                    assert_eq!(
                        supported,
                        vec!["json".into(), "yaml".into()],
                        "Expected supported formats to be copied"
                    );
                }
                other @ (SchemaIngestionError::File(_)
                | SchemaIngestionError::Parse(_)
                | SchemaIngestionError::Version(_)
                | SchemaIngestionError::Syntax(_)
                | SchemaIngestionError::Repository(_)
                | SchemaIngestionError::Schema {
                    ..
                }) => {
                    let is_expected = matches!(
                        other,
                        SchemaIngestionError::File(
                            SchemaFileError::UnsupportedFormat { .. }
                        )
                    );
                    assert!(
                        is_expected,
                        "Expected SchemaIngestionError::File::UnsupportedFormat, got: {other:?}"
                    );
                }
            }
        }

        #[test]
        fn db_error_converts_into_schema_repository_error() {
            let db_error = DbError::Serialization("test".into());
            let repo_error: SchemaRepositoryError = db_error.into();

            assert!(
                matches!(
                    repo_error,
                    SchemaRepositoryError::Storage(DbError::Serialization(_))
                ),
                "Expected DbError::Serialization to convert into \
                 SchemaRepositoryError::Storage(_)"
            );
        }
    }
}
