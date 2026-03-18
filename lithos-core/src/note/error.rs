//! Error types for note domain and persistence operations.

use super::{aggregate::NoteId, paths::NotePath, value::FieldValueType};

/// Note-related errors.
///
/// This enum covers domain-level errors related to parsing, validation,
/// and consistency of note ingest artifacts and projections.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteError {
    /// Note already exists.
    #[error("note already exists: {0}")]
    AlreadyExists(NotePath),

    /// Frontmatter parsing error.
    #[error("frontmatter error: {0}")]
    Frontmatter(#[from] FrontmatterParseError),

    /// Frontmatter access/extraction error.
    #[error(transparent)]
    FrontmatterAccess(#[from] FrontmatterError),

    /// Note path is invalid.
    #[error("invalid note path: {0}")]
    InvalidPath(Box<str>),

    /// Link parsing error.
    #[error("link error: {0}")]
    Link(#[from] LinkError),

    /// Note metadata validation error.
    #[error("note metadata error: {0}")]
    Metadata(#[from] NoteMetadataError),

    /// Configuration validation error.
    #[error("config error: {0}")]
    Config(#[from] crate::config::error::ConfigError),

    /// Note not found.
    #[error("note not found: {0}")]
    NotFound(NoteId),

    /// Storage error.
    #[error("storage error: {0}")]
    Storage(Box<str>),

    /// Tag error.
    #[error("tag error: {0}")]
    Tag(#[from] TagError),

    /// Task error.
    #[error("task error: {0}")]
    Task(#[from] TaskError),

    /// List nesting depth is out of range.
    #[error("list depth out of range: {depth}")]
    ListDepthOutOfRange {
        /// The observed list depth.
        depth: usize,
        /// Conversion error details.
        reason: &'static str,
    },

    /// Structural error within a note.
    #[error("note structure error: {0}")]
    Structure(&'static str),
}

/// Errors surfaced during note ingestion (file + parse + validation).
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteIngestError {
    /// Source I/O or parsing error.
    #[error("source error: {0}")]
    Source(Box<str>),

    /// Domain validation error.
    #[error(transparent)]
    Domain(#[from] NoteError),
}

impl From<NoteIngestError> for NoteError {
    #[inline]
    fn from(error: NoteIngestError) -> Self {
        match error {
            NoteIngestError::Source(message) => NoteError::Storage(message),
            NoteIngestError::Domain(error) => error,
        }
    }
}

impl From<crate::fs::error::ParseError> for NoteIngestError {
    #[inline]
    fn from(error: crate::fs::error::ParseError) -> Self {
        NoteIngestError::Source(error.to_string().into())
    }
}

/// Errors surfaced when validating or parsing tags.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TagError {
    /// Tag does not start with '#'.
    #[error("tag must start with #")]
    MissingHash,
    /// Tag is empty after the hash.
    #[error("tag cannot be empty")]
    EmptyTag,
    /// Tag contains an empty path segment.
    #[error("empty tag segment")]
    EmptySegment,
    /// Tag segment contains invalid characters.
    #[error(
        "invalid tag segment '{segment}': only alphanumeric, underscore, and \
         hyphen allowed"
    )]
    InvalidSegment {
        /// The invalid segment text.
        segment: Box<str>,
    },
}

/// Errors surfaced when validating or parsing tasks.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TaskError {
    /// Task text is empty after trimming.
    #[error("task text cannot be empty")]
    EmptyText,
    /// Task field key is empty after trimming.
    #[error("task field key cannot be empty")]
    FieldKeyEmpty,
    /// Task field key contains invalid characters.
    #[error("task field key must be ASCII alphanumeric, '_' or '-'")]
    FieldKeyInvalidChars,
    /// Checkbox status symbol is not recognized in the config.
    #[error("unrecognized status symbol: '{symbol}'")]
    UnrecognizedStatusSymbol {
        /// The raw status symbol.
        symbol: char,
    },
    /// Checkbox status symbol is invalid.
    #[error("invalid status symbol '{symbol}': {reason}")]
    InvalidStatusSymbol {
        /// The raw status symbol.
        symbol: char,
        /// Validation failure details.
        reason: &'static str,
    },
    /// Task date field is not parseable.
    #[error("invalid date for field '{keyword}': {reason}")]
    InvalidDate {
        /// The field keyword.
        keyword: Box<str>,
        /// Parse error details.
        reason: &'static str,
    },
    /// Task date field contains an invalid time.
    #[error("invalid time for date in field '{keyword}'")]
    InvalidDateTime {
        /// The field keyword.
        keyword: Box<str>,
    },
    /// Task metadata field failed validation.
    #[error("invalid metadata field '{keyword}': {reason}")]
    InvalidMetadataField {
        /// The field keyword.
        keyword: Box<str>,
        /// Validation failure details.
        reason: &'static str,
    },
    /// Task metadata integer value is invalid.
    #[error("invalid integer value '{raw}': {reason}")]
    InvalidInteger {
        /// The raw value string.
        raw: Box<str>,
        /// Parse error details.
        reason: &'static str,
    },
    /// Task metadata float value is invalid.
    #[error("invalid float value '{raw}': {reason}")]
    InvalidFloat {
        /// The raw value string.
        raw: Box<str>,
        /// Parse error details.
        reason: &'static str,
    },
    /// Task priority value is invalid.
    #[error("invalid task priority: {reason}")]
    InvalidPriority {
        /// Validation failure details.
        reason: &'static str,
    },
}

/// Errors surfaced when validating or parsing links.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LinkError {
    /// Link target is empty.
    #[error("link target cannot be empty")]
    EmptyTarget,
    /// External links cannot contain anchors.
    #[error("external links cannot have anchors")]
    ExternalAnchor,
    /// Heading anchor text is empty.
    #[error("heading anchor cannot be empty")]
    EmptyHeadingAnchor,
    /// Block reference anchor text is empty.
    #[error("block reference anchor cannot be empty")]
    EmptyBlockRefAnchor,
    /// Link alias text is empty.
    #[error("link alias cannot be empty")]
    EmptyAlias,
}

/// Errors surfaced when validating note metadata values.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NoteMetadataError {
    /// Alias name is empty.
    #[error("alias name cannot be empty")]
    AliasEmpty,
    /// File class name is empty.
    #[error("file class cannot be empty")]
    FileClassEmpty,
    /// Folder path is empty.
    #[error("folder path cannot be empty")]
    FolderEmpty,
    /// Heading text is empty.
    #[error("heading text cannot be empty")]
    HeadingTextEmpty,
}

/// Errors surfaced by Note command operations.
///
/// Combines domain errors with low-level storage errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteCommandError {
    /// Domain validation error.
    #[error(transparent)]
    Domain(#[from] NoteError),

    /// Storage operation error.
    #[error(transparent)]
    Storage(#[from] crate::db::DbError),
}

/// Errors surfaced by Note query operations.
///
/// Combines domain errors with low-level storage errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteQueryError {
    /// Domain validation error.
    #[error(transparent)]
    Domain(#[from] NoteError),

    /// Storage operation error.
    #[error(transparent)]
    Storage(#[from] crate::db::DbError),
}

/// Errors surfaced by strict metadata accessors (frontmatter and tasks).
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FrontmatterError {
    /// A required key was missing from the map.
    #[error("missing frontmatter key: {key}")]
    Missing {
        /// The missing key.
        key: Box<str>,
    },

    /// A key exists, but the value has an unexpected runtime type.
    #[error(
        "frontmatter key '{key}' has wrong type (expected {expected}, got \
         {actual})"
    )]
    TypeMismatch {
        /// The key that was requested.
        key: Box<str>,
        /// The expected type description.
        expected: FieldValueType,
        /// The actual runtime type.
        actual: FieldValueType,
    },

    /// A key exists and is an array, but at least one element has the wrong
    /// type.
    #[error(
        "frontmatter key '{key}' has wrong array element type at index \
         {index} (expected {expected}, got {actual})"
    )]
    ArrayElementTypeMismatch {
        /// The key that was requested.
        key: Box<str>,
        /// The index of the first mismatched array element.
        index: usize,
        /// The expected element type.
        expected: FieldValueType,
        /// The actual element type.
        actual: FieldValueType,
    },

    /// A key exists and is a date timestamp, but the timestamp is not
    /// representable as a UTC datetime.
    #[error("frontmatter key '{key}' has invalid date timestamp: {timestamp}")]
    InvalidDateTimestamp {
        /// The key that was requested.
        key: Box<str>,
        /// The invalid timestamp.
        timestamp: i64,
    },
}

impl FrontmatterError {
    /// Attaches key context to an error if it doesn't already have one.
    #[inline]
    #[must_use]
    pub fn with_key(mut self, field_key: &str) -> Self {
        match self {
            Self::Missing {
                ref mut key,
            }
            | Self::TypeMismatch {
                ref mut key,
                ..
            }
            | Self::ArrayElementTypeMismatch {
                ref mut key,
                ..
            }
            | Self::InvalidDateTimestamp {
                ref mut key,
                ..
            } => {
                if key.is_empty() {
                    *key = field_key.into();
                }
            }
        }
        self
    }
}

/// Unified error type for [`super::value::FieldValue`] operations.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FieldValueError {
    /// Type mismatch between expected and actual value.
    #[error("type mismatch: expected {expected}, found {actual}")]
    TypeMismatch {
        /// Expected value type.
        expected: FieldValueType,
        /// Actual value type.
        actual: FieldValueType,
    },
    /// Invalid date timestamp.
    #[error("invalid date timestamp: {timestamp}")]
    InvalidDateTimestamp {
        /// The problematic timestamp.
        timestamp: i64,
    },
    /// Array element type mismatch.
    #[error(
        "array element type mismatch at index {index}: expected {expected}, \
         found {actual}"
    )]
    ArrayElementTypeMismatch {
        /// Index of the problematic element.
        index: usize,
        /// Expected element type.
        expected: FieldValueType,
        /// Actual element type.
        actual: FieldValueType,
    },
}

impl From<FieldValueError> for FrontmatterError {
    #[inline]
    fn from(error: FieldValueError) -> Self {
        match error {
            FieldValueError::TypeMismatch {
                expected,
                actual,
            } => Self::TypeMismatch {
                key: "".into(),
                expected,
                actual,
            },
            FieldValueError::InvalidDateTimestamp {
                timestamp,
            } => Self::InvalidDateTimestamp {
                key: "".into(),
                timestamp,
            },
            FieldValueError::ArrayElementTypeMismatch {
                index,
                expected,
                actual,
            } => Self::ArrayElementTypeMismatch {
                key: "".into(),
                index,
                expected,
                actual,
            },
        }
    }
}

/// Errors surfaced when parsing frontmatter blocks.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FrontmatterParseError {
    /// YAML frontmatter could not be parsed.
    #[error("invalid YAML: {reason}")]
    InvalidYaml {
        /// Parse error details.
        reason: &'static str,
    },
    /// TOML frontmatter could not be parsed.
    #[error("invalid TOML: {reason}")]
    InvalidToml {
        /// Parse error details.
        reason: &'static str,
    },
    /// Frontmatter must be a YAML mapping.
    #[error("frontmatter must be a YAML mapping")]
    NotYamlMapping,
    /// Frontmatter must be a TOML table.
    #[error("frontmatter must be a TOML table")]
    NotTomlTable,
    /// YAML map contained a non-string key.
    #[error("non-string key")]
    NonStringKey,
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn note_error_is_send_and_sync() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<NoteError>();
    }

    #[rstest]
    #[case(NoteError::InvalidPath("test.md".into()))]
    #[case(NoteError::NotFound(NoteId::new()))]
    #[case(NoteError::AlreadyExists(
        NotePath::try_new("test.md").expect("valid path")
    ))]
    #[case(NoteError::Metadata(NoteMetadataError::HeadingTextEmpty))]
    #[case(NoteError::Config(
        crate::config::error::ConfigError::ValidationFailed {
            field: "frontmatter_key".into(),
            message: "empty".into(),
        }
    ))]
    #[case(NoteError::Frontmatter(FrontmatterParseError::NotYamlMapping))]
    #[case(NoteError::FrontmatterAccess(FrontmatterError::Missing {
        key: "title".into(),
    }))]
    #[case(NoteError::Link(LinkError::EmptyTarget))]
    #[case(NoteError::Tag(TagError::MissingHash))]
    #[case(NoteError::Task(TaskError::EmptyText))]
    #[case(NoteError::Task(TaskError::InvalidPriority {
        reason: "not finite",
    }))]
    #[case(NoteError::ListDepthOutOfRange {
        depth: 300,
        reason: "out of range",
    })]
    #[case(NoteError::Storage("io error".into()))]
    fn note_error_display_is_comprehensive(#[case] error: NoteError) {
        assert!(
            !error.to_string().is_empty(),
            "Error {error:?} should have non-empty display message"
        );
    }
}
