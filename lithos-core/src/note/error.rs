//! Error types for the note domain and ingestion pipeline.
//!
//! This module defines a phase-oriented error taxonomy that matches the
//! note lifecycle: Ingestion → Parsing → Domain Validation → Persistence →
//! Orchestration.

use std::path::PathBuf;

use super::{aggregate::NoteId, paths::NotePath};

/// Top-level umbrella for all domain-related errors in the note module.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteError {
    /// Errors surfaced when validating or parsing tags.
    #[error(transparent)]
    Tag(#[from] TagError),

    /// Errors surfaced when validating or parsing tasks.
    #[error(transparent)]
    Task(#[from] TaskError),

    /// Errors surfaced when validating or parsing links.
    #[error(transparent)]
    Link(#[from] LinkError),

    /// Errors surfaced when validating note headings.
    #[error(transparent)]
    Heading(#[from] HeadingError),

    /// Errors surfaced when validating list nesting.
    #[error(transparent)]
    List(#[from] ListError),

    /// Internal structural errors related to source positions.
    #[error(transparent)]
    Structure(#[from] StructureError),

    /// Errors occurring during the parsing phase.
    #[error(transparent)]
    Parse(#[from] NoteParseError),

    /// Errors related to frontmatter access or validation.
    #[error(transparent)]
    Frontmatter(#[from] FrontmatterError),

    /// Configuration-related errors.
    #[error(transparent)]
    Config(#[from] crate::config::error::ConfigError),

    /// Filesystem and path-related errors.
    #[error(transparent)]
    File(#[from] NoteFileError),
}

/// Errors occurring when bridging the physical vault to raw markdown facts.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteIngestError {
    /// Filesystem-level errors.
    #[error(transparent)]
    File(#[from] NoteFileError),

    /// Markdown or frontmatter syntax errors.
    #[error(transparent)]
    Parse(#[from] NoteParseError),

    /// Domain validation failure immediately following ingestion.
    #[error(transparent)]
    Domain(#[from] NoteError),
}

/// Filesystem and vault-boundary errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteFileError {
    /// Attempted to access a file outside the vault root.
    #[error("access denied: path '{path}' is outside the vault root")]
    VaultRootEscape {
        /// The problematic path.
        path: PathBuf,
    },

    /// File does not have a supported extension (only .md allowed).
    #[error(
        "unsupported extension for '{path}': expected .md, found '{found}'"
    )]
    UnsupportedExtension {
        /// The problematic path.
        path: Box<str>,
        /// The extension found.
        found: Box<str>,
    },

    /// Logical path validation failure.
    #[error("invalid note path '{path}': {reason}")]
    InvalidPath {
        /// The raw path string.
        path: Box<str>,
        /// The reason for failure.
        reason: &'static str,
    },

    /// Failure to read file content.
    #[error("failed to read note at '{path}': {message}")]
    ReadFailed {
        /// The note path.
        path: NotePath,
        /// The error message.
        message: Box<str>,
    },

    /// Failure to read filesystem metadata (timestamps).
    #[error("failed to read metadata for '{path}': {message}")]
    MetadataFailed {
        /// The note path.
        path: NotePath,
        /// The error message.
        message: Box<str>,
    },
}

/// Markdown and frontmatter syntax errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteParseError {
    /// Markdown extraction logic failure.
    #[error("markdown error at line {line}, col {column}: {reason}")]
    Markdown {
        /// Line number (1-based).
        line: usize,
        /// Column number (1-based).
        column: usize,
        /// The reason for failure.
        reason: Box<str>,
    },

    /// Frontmatter syntax failure (YAML/TOML).
    #[error(
        "invalid {format} frontmatter (line {line:?}, col {column:?}): \
         {reason}"
    )]
    Frontmatter {
        /// The format (YAML or TOML).
        format: &'static str,
        /// Optional line number.
        line: Option<usize>,
        /// Optional column number.
        column: Option<usize>,
        /// The error reason.
        reason: Box<str>,
    },

    /// Note content exceeds maximum supported size.
    #[error("source too large: {size} bytes (limit: {limit})")]
    SourceTooLarge {
        /// Actual size in bytes.
        size: usize,
        /// Maximum supported size.
        limit: usize,
    },

    /// Content is not valid UTF-8.
    #[error("invalid UTF-8 encoding in note: {path}")]
    Encoding {
        /// The note path.
        path: NotePath,
    },
}

/// Persistence and repository interface errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteRepositoryError {
    /// Note not found by ID.
    #[error("note not found: {id}")]
    NotFound {
        /// The requested note ID.
        id: NoteId,
    },

    /// Note not found by path.
    #[error("note not found at path: {path}")]
    NotFoundByPath {
        /// The requested note path.
        path: NotePath,
    },

    /// Uniqueness conflict (e.g. path already exists).
    #[error("persistence conflict: note already exists at {path}")]
    AlreadyExists {
        /// The conflicting path.
        path: NotePath,
    },

    /// Database constraint or logic violation.
    #[error("storage constraint violation: {message}")]
    ConstraintViolation {
        /// The error message.
        message: Box<str>,
    },

    /// Stored data is corrupt or fails domain validation.
    #[error("data corruption in note {id}: {reason}")]
    Corruption {
        /// The note ID.
        id: NoteId,
        /// The corruption reason.
        reason: Box<str>,
    },

    /// Path exists but maps to a different stable ID.
    #[error("identity conflict: path '{path}' is already bound to ID {id}")]
    IdentityConflict {
        /// The existing note ID.
        id: NoteId,
        /// The conflicting path.
        path: NotePath,
    },

    /// Projection bloat prevention.
    #[error(
        "resource limit exceeded: {context} has {current} items (limit: \
         {limit})"
    )]
    ResourceLimitExceeded {
        /// Current item count.
        current: usize,
        /// Limit enforced.
        limit: usize,
        /// Boundary context.
        context: &'static str,
    },

    /// Low-level database failure.
    #[error("storage error: {0}")]
    Storage(#[from] crate::db::DbError),
}

/// Orchestration errors coordination the full pipeline.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteLoadError {
    /// Ingestion failed.
    #[error("ingestion failed: {0}")]
    Ingestion(#[from] NoteIngestError),

    /// Domain validation failed.
    #[error("validation failed: {0}")]
    Validation(#[from] NoteError),

    /// Persistence failed.
    #[error("persistence failed: {0}")]
    Persistence(#[from] NoteRepositoryError),
}

// --- Sub-Domain Logic Errors ---

/// Errors related to hierarchical tag validation.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TagError {
    /// Tag must start with '#'.
    #[error("tag must start with '#'")]
    MissingHash,

    /// Tag is empty after the hash.
    #[error("tag cannot be empty")]
    EmptyTag,

    /// Tag contains an empty segment (e.g. `##`).
    #[error("tag contains an empty segment")]
    EmptySegment,

    /// Tag segment contains invalid characters.
    #[error("invalid tag segment '{segment}': {reason}")]
    InvalidSegment {
        /// The invalid segment text.
        segment: Box<str>,
        /// The validation reason.
        reason: &'static str,
    },
}

/// Errors related to task extraction and validation.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum TaskError {
    /// Task text is empty after trimming.
    #[error("task text cannot be empty")]
    EmptyText,

    /// Checkbox status symbol is unrecognized.
    #[error("unrecognized status symbol: '{symbol}'")]
    UnrecognizedStatus {
        /// The raw status symbol.
        symbol: char,
    },

    /// Task priority is non-finite.
    #[error("invalid priority value {value}: must be finite")]
    InvalidPriority {
        /// The invalid numeric value.
        value: f64,
    },

    /// Task metadata field failed validation.
    #[error("invalid metadata field '{key}': {reason}")]
    InvalidMetadataField {
        /// The metadata key.
        key: Box<str>,
        /// The failure reason.
        reason: &'static str,
    },

    /// Task date field failed parsing.
    #[error("invalid date for field '{keyword}': {reason}")]
    InvalidDate {
        /// The field keyword.
        keyword: Box<str>,
        /// The raw date string.
        raw: Box<str>,
        /// The failure reason.
        reason: &'static str,
    },
}

/// Errors related to link and anchor validation.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LinkError {
    /// Link target is empty.
    #[error("link target cannot be empty")]
    EmptyTarget,

    /// Heading anchor text is empty.
    #[error("heading anchor cannot be empty")]
    EmptyHeadingAnchor,

    /// Block reference anchor text is empty.
    #[error("block reference anchor cannot be empty")]
    EmptyBlockRefAnchor,

    /// Link target has invalid format.
    #[error("invalid link target format: {target}")]
    InvalidTarget {
        /// The problematic target.
        target: Box<str>,
    },

    /// External links cannot contain anchors.
    #[error("external links cannot contain anchors (found in '{target}')")]
    ExternalAnchorNotAllowed {
        /// The problematic target.
        target: Box<str>,
    },

    /// Link alias text is empty.
    #[error("link alias cannot be empty")]
    EmptyAlias,

    /// Circular link reference detected.
    #[error("circular reference detected: {target}")]
    CircularReference {
        /// The target involved in the cycle.
        target: Box<str>,
    },
}

/// Errors related to heading validation.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HeadingError {
    /// Heading level is outside the 1-6 range.
    #[error("invalid heading level {level}: must be between 1 and 6")]
    InvalidLevel {
        /// The invalid level.
        level: u32,
    },

    /// Heading text content is empty.
    #[error("heading content cannot be empty")]
    EmptyContent,
}

/// Errors related to list structure limits.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ListError {
    /// List nesting exceeds internal limits.
    #[error(
        "maximum list nesting depth exceeded (depth: {current}, limit: \
         {limit})"
    )]
    MaxNestingExceeded {
        /// Observed depth.
        current: usize,
        /// Limit enforced.
        limit: usize,
    },
}

/// Errors related to source offsets and block identifiers.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StructureError {
    /// Resulting offset exceeds u32 range.
    #[error("source offset {offset} overflow by {delta}")]
    OffsetOverflow {
        /// Base offset.
        offset: usize,
        /// Increment delta.
        delta: usize,
    },

    /// Offset exceeds source buffer bounds.
    #[error(
        "source offset {offset} out of bounds (source length: {source_len})"
    )]
    OutOfBounds {
        /// Problematic offset.
        offset: usize,
        /// Total buffer length.
        source_len: usize,
    },

    /// Range has start > end.
    #[error("invalid source range: start {start} > end {end}")]
    InvalidRange {
        /// Range start.
        start: usize,
        /// Range end.
        end: usize,
    },

    /// Line number is zero or invalid.
    #[error("invalid line number: {line} (must be >= 1)")]
    InvalidLine {
        /// The problematic line number.
        line: u32,
    },

    /// Column number is zero or invalid.
    #[error("invalid column number: {column} (must be >= 1)")]
    InvalidColumn {
        /// The problematic column number.
        column: u32,
    },

    /// Block identifier format is invalid.
    #[error(
        "invalid block identifier '{id}': must be alphanumeric and start with \
         ^"
    )]
    InvalidBlockId {
        /// The problematic identifier.
        id: Box<str>,
    },

    /// Block identifier is not unique within the note.
    #[error("duplicate block identifier '{id}' within the same note")]
    DuplicateBlockId {
        /// The duplicate identifier.
        id: Box<str>,
    },
}

/// Errors related to frontmatter validation and access.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FrontmatterError {
    /// Alias validation failure.
    #[error("invalid alias '{value}': {reason}")]
    InvalidAlias {
        /// The alias value.
        value: Box<str>,
        /// The failure reason.
        reason: &'static str,
    },

    /// File class validation failure.
    #[error("invalid file class '{value}': {reason}")]
    InvalidFileClass {
        /// The class value.
        value: Box<str>,
        /// The failure reason.
        reason: &'static str,
    },

    /// A required key was missing.
    #[error("missing required key: {key}")]
    KeyMissing {
        /// The missing key.
        key: Box<str>,
    },

    /// Value type does not match expectation.
    #[error("type mismatch for key '{key}': expected {expected}, got {actual}")]
    TypeMismatch {
        /// The field key.
        key: Box<str>,
        /// Expected type name.
        expected: &'static str,
        /// Actual type name.
        actual: &'static str,
    },

    /// Date field contains an unrepresentable timestamp.
    #[error("invalid date timestamp for key '{key}': {timestamp}")]
    InvalidDateTimestamp {
        /// The field key.
        key: Box<str>,
        /// The raw timestamp.
        timestamp: i64,
    },
}

impl FrontmatterError {
    /// Attaches key context to an error if it doesn't already have one.
    #[inline]
    #[must_use]
    pub fn with_key(mut self, field_key: &str) -> Self {
        match self {
            Self::KeyMissing {
                ref mut key,
            }
            | Self::TypeMismatch {
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
            Self::InvalidAlias {
                ..
            }
            | Self::InvalidFileClass {
                ..
            } => {}
        }
        self
    }
}

// --- Conversions ---

impl From<crate::db::DbError> for NoteLoadError {
    #[inline]
    fn from(err: crate::db::DbError) -> Self {
        NoteLoadError::Persistence(NoteRepositoryError::Storage(err))
    }
}

impl From<crate::fs::error::ParseError> for NoteIngestError {
    #[inline]
    fn from(err: crate::fs::error::ParseError) -> Self {
        #[expect(clippy::unwrap_used, reason = "Static dummy path is valid")]
        let dummy_path = NotePath::try_new("vault.md").unwrap();
        NoteFileError::ReadFailed {
            path: dummy_path,
            message: err.to_string().into(),
        }
        .into()
    }
}
