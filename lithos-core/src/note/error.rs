//! Error types for the Note domain and its ingestion pipeline.
//!
//! This module provides a hierarchical, phase-oriented error taxonomy that
//! maps to the Note lifecycle:
//!
//! 1. **Ingestion**: Physical file access and vault boundary enforcement
//!    ([`NoteFileError`]).
//! 2. **Parsing**: Markdown and frontmatter syntax extraction
//!    ([`NoteParseError`]).
//! 3. **Validation**: Domain-level invariant enforcement for entities like
//!    tags, tasks, and links ([`NoteError`] umbrella).
//! 4. **Persistence**: Database integrity and identity management
//!    ([`NoteRepositoryError`]).
//! 5. **Orchestration**: The top-level coordination of the full pipeline
//!    ([`NoteLoadError`]).
//!
//! All errors use `thiserror` for descriptive formatting and support
//! zero-copy patterns by using `Box<str>` for dynamic message data where
//! applicable.

use std::path::PathBuf;

use super::{aggregate::NoteId, paths::NotePath};

// --- Umbrella Error Types ---

/// Unified domain error umbrella for the Note context.
///
/// This type wraps all errors related to business logic, entity validation,
/// and internal structural consistency. It is the primary error type
/// returned by domain methods.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteError {
    /// Validation or parsing failure for a hierarchical tag.
    #[error(transparent)]
    Tag(#[from] TagError),

    /// Validation or attribute parsing failure for a task checkbox.
    #[error(transparent)]
    Task(#[from] TaskError),

    /// Validation failure for a markdown or wiki link.
    #[error(transparent)]
    Link(#[from] LinkError),

    /// Structural or content validation failure for a note heading.
    #[error(transparent)]
    Heading(#[from] HeadingError),

    /// Nesting or structural violation in a markdown list.
    #[error(transparent)]
    List(#[from] ListError),

    /// Internal structural error related to source tracking and offsets.
    #[error(transparent)]
    Structure(#[from] StructureError),

    /// Syntax error encountered during the parsing of a note document.
    #[error(transparent)]
    Parse(#[from] NoteParseError),

    /// Logical error during frontmatter metadata access or type conversion.
    #[error(transparent)]
    Frontmatter(#[from] FrontmatterError),

    /// Global or vault-specific configuration violation.
    #[error(transparent)]
    Config(#[from] crate::config::error::ConfigError),

    /// Filesystem-level error or vault path boundary violation.
    #[error(transparent)]
    File(#[from] NoteFileError),
}

/// Errors occurring during the transition from physical file to raw facts.
///
/// Wraps filesystem I/O failures, encoding issues, and syntax errors
/// encountered during initial ingestion.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteIngestError {
    /// Low-level filesystem or vault-relative path error.
    #[error(transparent)]
    File(#[from] NoteFileError),

    /// Syntax error in markdown or frontmatter blocks.
    #[error(transparent)]
    Parse(#[from] NoteParseError),

    /// Logical domain violation detected immediately after parsing.
    #[error(transparent)]
    Domain(#[from] NoteError),
}

/// Orchestration error returned by the Note
/// [Loader][crate::note::loader::Loader].
///
/// Distinguishes between failures in the ingestion, domain validation,
/// and persistence phases of the note lifecycle.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteLoadError {
    /// The file could not be read or its basic syntax was invalid.
    #[error("ingestion failed: {0}")]
    Ingestion(#[from] NoteIngestError),

    /// The note facts violated domain-level business invariants.
    #[error("validation failed: {0}")]
    Validation(#[from] NoteError),

    /// The note could not be saved to or retrieved from the repository.
    #[error("persistence failed: {0}")]
    Persistence(#[from] NoteRepositoryError),
}

// --- Pipeline & Interface Errors ---

/// Filesystem and vault-boundary enforcement errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteFileError {
    /// Attempted to access a path that resolves outside the configured vault
    /// root.
    #[error("access denied: path '{path}' is outside the vault root")]
    VaultRootEscape {
        /// The absolute path that attempted the escape.
        path: PathBuf,
    },

    /// File extension is not supported by the Note context (only `.md`
    /// allowed).
    #[error(
        "unsupported extension for '{path}': expected .md, found '{found}'"
    )]
    UnsupportedExtension {
        /// The problematic file path.
        path: Box<str>,
        /// The actual extension found.
        found: Box<str>,
    },

    /// Logical path validation failure (e.g., reserved characters or
    /// traversal).
    #[error("invalid note path '{path}': {reason}")]
    InvalidPath {
        /// The raw path string.
        path: Box<str>,
        /// Human-readable reason for the failure.
        reason: &'static str,
    },

    /// Failure to read the raw content of a note file.
    #[error("failed to read note at '{path}': {message}")]
    ReadFailed {
        /// The vault-relative path of the note.
        path: NotePath,
        /// The underlying I/O error message.
        message: Box<str>,
    },

    /// Failure to retrieve filesystem metadata (e.g., mtime, ctime).
    #[error("failed to read metadata for '{path}': {message}")]
    MetadataFailed {
        /// The vault-relative path of the note.
        path: NotePath,
        /// The underlying I/O error message.
        message: Box<str>,
    },
}

/// Errors encountered during markdown and frontmatter parsing.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteParseError {
    /// Logical inconsistency or illegal state in the markdown event stream.
    #[error("markdown error at line {line}, col {column}: {reason}")]
    Markdown {
        /// 1-based line number.
        line: usize,
        /// 1-based column number.
        column: usize,
        /// Details of the extraction failure.
        reason: Box<str>,
    },

    /// Frontmatter syntax failure (YAML or TOML).
    #[error(
        "invalid {format} frontmatter (line {line:?}, col {column:?}): \
         {reason}"
    )]
    Frontmatter {
        /// The format name (e.g., "YAML", "TOML").
        format: &'static str,
        /// Optional 1-based line number.
        line: Option<usize>,
        /// Optional 1-based column number.
        column: Option<usize>,
        /// The raw parser error message.
        reason: Box<str>,
    },

    /// Note source text exceeds the internal processing limits.
    #[error("source too large: {size} bytes (limit: {limit})")]
    SourceTooLarge {
        /// Total size of the source text in bytes.
        size: usize,
        /// Maximum supported size in bytes.
        limit: usize,
    },

    /// Note content is not valid UTF-8.
    #[error("invalid UTF-8 encoding in note: {path}")]
    Encoding {
        /// The vault-relative path of the note.
        path: NotePath,
    },
}

/// Errors surfaced by the persistence and repository layer.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum NoteRepositoryError {
    /// Note requested by unique identifier was not found.
    #[error("note not found: {id}")]
    NotFound {
        /// The requested note ID.
        id: NoteId,
    },

    /// Note requested by vault path was not found.
    #[error("note not found at path: {path}")]
    NotFoundByPath {
        /// The requested note path.
        path: NotePath,
    },

    /// Persistence conflict where a note already exists at the target path.
    #[error("persistence conflict: note already exists at {path}")]
    AlreadyExists {
        /// The conflicting vault path.
        path: NotePath,
    },

    /// Violation of a storage constraint (e.g., size limits or unique indexes).
    #[error("storage constraint violation: {message}")]
    ConstraintViolation {
        /// Human-readable description of the violation.
        message: Box<str>,
    },

    /// Stored binary data failed domain validation upon retrieval.
    #[error("data corruption in note {id}: {reason}")]
    Corruption {
        /// The ID of the corrupt note.
        id: NoteId,
        /// Details of the validation failure.
        reason: Box<str>,
    },

    /// Note path is already bound to a different stable ID in the database.
    #[error("identity conflict: path '{path}' is already bound to ID {id}")]
    IdentityConflict {
        /// The existing note ID.
        id: NoteId,
        /// The vault path involved in the conflict.
        path: NotePath,
    },

    /// Note contains too many entities (tags, tasks, etc.) for efficient
    /// indexing.
    #[error(
        "resource limit exceeded: {context} has {current} items (limit: \
         {limit})"
    )]
    ResourceLimitExceeded {
        /// Current number of items observed.
        current: usize,
        /// Maximum allowed number of items.
        limit: usize,
        /// The specific domain context (e.g., "tasks").
        context: &'static str,
    },

    /// Low-level database infrastructure failure.
    #[error("storage error: {0}")]
    Storage(#[from] crate::db::DbError),
}

// --- Sub-Domain Logic Errors ---

/// Errors related to hierarchical tag construction and validation.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TagError {
    /// Tag string is missing the required '#' prefix.
    #[error("tag must start with '#'")]
    MissingHash,

    /// Tag string contains only the '#' prefix.
    #[error("tag cannot be empty")]
    EmptyTag,

    /// Tag path contains an empty segment (e.g., `#work//urgent`).
    #[error("tag contains an empty segment")]
    EmptySegment,

    /// A segment of the tag path violates character constraints.
    #[error("invalid tag segment '{segment}': {reason}")]
    InvalidSegment {
        /// The problematic segment text.
        segment: Box<str>,
        /// Human-readable reason for the failure.
        reason: &'static str,
    },
}

/// Errors related to task entity extraction and metadata validation.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum TaskError {
    /// Task description text is empty.
    #[error("task text cannot be empty")]
    EmptyText,

    /// The checkbox status symbol is not recognized by the current
    /// configuration.
    #[error("unrecognized status symbol: '{symbol}'")]
    UnrecognizedStatus {
        /// The raw character marker (e.g., '/').
        symbol: char,
    },

    /// Task priority value is not a finite number.
    #[error("invalid priority value {value}: must be finite")]
    InvalidPriority {
        /// The problematic numeric value.
        value: f64,
    },

    /// A metadata field key violates naming constraints.
    #[error("invalid metadata field '{key}': {reason}")]
    InvalidMetadataField {
        /// The problematic field key.
        key: Box<str>,
        /// Human-readable reason for the failure.
        reason: &'static str,
    },

    /// A temporal metadata field contains an unparseable date or time.
    #[error("invalid date for field '{keyword}': {reason}")]
    InvalidDate {
        /// The metadata keyword (e.g., "due").
        keyword: Box<str>,
        /// The raw value that failed to parse.
        raw: Box<str>,
        /// The specific parse error details.
        reason: &'static str,
    },
}

/// Errors related to link resolution and anchor consistency.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LinkError {
    /// Link target path is empty.
    #[error("link target cannot be empty")]
    EmptyTarget,

    /// Heading anchor component is empty.
    #[error("heading anchor cannot be empty")]
    EmptyHeadingAnchor,

    /// Block reference anchor component is empty.
    #[error("block reference anchor cannot be empty")]
    EmptyBlockRefAnchor,

    /// Link target format is logically invalid.
    #[error("invalid link target format: {target}")]
    InvalidTarget {
        /// The problematic target text.
        target: Box<str>,
    },

    /// Anchor targets are not permitted for external (URL) links.
    #[error("external links cannot contain anchors (found in '{target}')")]
    ExternalAnchorNotAllowed {
        /// The external target string.
        target: Box<str>,
    },

    /// Link alias text is empty.
    #[error("link alias cannot be empty")]
    EmptyAlias,

    /// Logical cycle detected in note-to-note references.
    #[error("circular reference detected: {target}")]
    CircularReference {
        /// The target participating in the cycle.
        target: Box<str>,
    },
}

/// Errors related to note heading validation.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HeadingError {
    /// Heading level is outside the standard markdown range (1-6).
    #[error("invalid heading level {level}: must be between 1 and 6")]
    InvalidLevel {
        /// The problematic level value.
        level: u32,
    },

    /// Heading text content is empty.
    #[error("heading content cannot be empty")]
    EmptyContent,
}

/// Errors related to markdown list structure and depth limits.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ListError {
    /// List nesting depth exceeds the internal safety limit (255).
    #[error(
        "maximum list nesting depth exceeded (depth: {current}, limit: \
         {limit})"
    )]
    MaxNestingExceeded {
        /// The observed depth.
        current: usize,
        /// The maximum allowed depth.
        limit: usize,
    },
}

/// Internal structural errors related to source buffer indexing and offsets.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StructureError {
    /// Mathematical overflow when calculating a source offset.
    #[error("source offset {offset} overflow by {delta}")]
    OffsetOverflow {
        /// The base byte offset.
        offset: usize,
        /// The relative delta being applied.
        delta: usize,
    },

    /// A byte offset refers to a position outside the current source buffer.
    #[error(
        "source offset {offset} out of bounds (source length: {source_len})"
    )]
    OutOfBounds {
        /// The problematic offset.
        offset: usize,
        /// Total length of the source buffer in bytes.
        source_len: usize,
    },

    /// A source range is malformed (start position is after end position).
    #[error("invalid source range: start {start} > end {end}")]
    InvalidRange {
        /// Range start offset.
        start: usize,
        /// Range end offset.
        end: usize,
    },

    /// Line number requested is zero or logically invalid.
    #[error("invalid line number: {line} (must be >= 1)")]
    InvalidLine {
        /// The problematic 1-based line number.
        line: u32,
    },

    /// Column number requested is zero or logically invalid.
    #[error("invalid column number: {column} (must be >= 1)")]
    InvalidColumn {
        /// The problematic 1-based column number.
        column: u32,
    },

    /// A block identifier (`^id`) violates character constraints.
    #[error(
        "invalid block identifier '{id}': must be alphanumeric and start with \
         ^"
    )]
    InvalidBlockId {
        /// The problematic identifier string.
        id: Box<str>,
    },

    /// Multiple blocks within the same note share the same identifier.
    #[error("duplicate block identifier '{id}' within the same note")]
    DuplicateBlockId {
        /// The conflicting block identifier.
        id: Box<str>,
    },
}

/// Errors related to frontmatter block processing and field access.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FrontmatterError {
    /// Validation failure for a note alias defined in frontmatter.
    #[error("invalid alias '{value}': {reason}")]
    InvalidAlias {
        /// The problematic alias string.
        value: Box<str>,
        /// Human-readable reason for the failure.
        reason: &'static str,
    },

    /// Validation failure for a file class defined in frontmatter.
    #[error("invalid file class '{value}': {reason}")]
    InvalidFileClass {
        /// The problematic file class name.
        value: Box<str>,
        /// Human-readable reason for the failure.
        reason: &'static str,
    },

    /// A required key was missing from the frontmatter map.
    #[error("missing required key: {key}")]
    KeyMissing {
        /// The name of the missing key.
        key: Box<str>,
    },

    /// A field value exists but cannot be converted to the requested type.
    #[error("type mismatch for key '{key}': expected {expected}, got {actual}")]
    TypeMismatch {
        /// The field key.
        key: Box<str>,
        /// Description of the expected type.
        expected: &'static str,
        /// Description of the actual observed type.
        actual: &'static str,
    },

    /// A date field contains a timestamp that is not representable.
    #[error("invalid date timestamp for key '{key}': {timestamp}")]
    InvalidDateTimestamp {
        /// The field key.
        key: Box<str>,
        /// The problematic Unix timestamp value.
        timestamp: i64,
    },
}

impl FrontmatterError {
    /// Attaches key context to an error if it doesn't already have one.
    ///
    /// This is useful for wrapping generic type mismatch or missing key errors
    /// during iterative extraction.
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
