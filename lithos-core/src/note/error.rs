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
//! # Hierarchy
//!
//! ```text
//! NoteLoadError (Orchestration)
//!  ├── NoteIngestError (Ingestion Phase)
//!  │    ├── NoteFileError (I/O & Vault Boundaries)
//!  │    └── NoteParseError (Syntax & Extraction)
//!  ├── NoteError (Domain Umbrella)
//!  │    ├── TagError, TaskError, LinkError, etc.
//!  │    └── NoteFileError (Logical path validation)
//!  └── NoteRepositoryError (Persistence Phase)
//!       └── DbError (Storage Layer)
//! ```
//!
//! # Design Principles
//!
//! - **Context Preservation**: Every layer wraps the previous one using
//!   `#[error(transparent)]` to ensure the root cause is preserved in the
//!   `source()` chain.
//! - **Performance**: Dynamic error data uses `Box<str>` instead of `String` to
//!   minimize heap allocations in hot paths like indexing and LSP queries.
//! - **Phase Orientation**: Errors are categorized by where they occur in the
//!   pipeline, preventing the "everything is an ingestion error" anti-pattern.
//!
//! # Usage Guidelines
//!
//! | Error Type              | When to Use                                                            |
//! | :---------------------- | :--------------------------------------------------------------------- |
//! | [`NoteError`]           | Pure domain logic, entity constructors, and normalization methods.     |
//! | [`NoteIngestError`]     | Readers, lexical collectors, and parsers bridging raw bytes to structured facts. |
//! | [`NoteRepositoryError`] | Storage adapters, indexing logic, and identity stability checks.       |
//! | [`NoteLoadError`]       | Cross-cutting services coordinating the entire lifecycle.              |
//!
//! # Examples
//!
//! ## Handling a Load Failure
//!
//! ```ignore
//! match loader.load_content(path, content, None, None) {
//!     Err(NoteLoadError::Ingestion(e)) => handle_syntax_error(e),
//!     Err(NoteLoadError::Persistence(e)) => handle_database_error(e),
//!     Err(NoteLoadError::Validation(e)) => handle_domain_violation(e),
//!     Ok(id) => proceed(id),
//! }
//! ```

use std::path::PathBuf;

use super::{
    aggregate::NoteId,
    paths::NotePath,
    position::{SourceByteOffset, SourceByteRange},
};

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

    /// Internal structural error.
    #[error("internal error: {0}")]
    Internal(Box<str>),

    /// Unexpected end of input during scanning.
    #[error("unexpected end of input")]
    UnexpectedEof,
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

impl From<crate::fs::ReadError> for NoteIngestError {
    #[inline]
    fn from(err: crate::fs::ReadError) -> Self {
        let (path_str, message) = match &err {
            crate::fs::ReadError::Io {
                path,
                source,
            } => (path.to_string_lossy(), source.to_string()),
            crate::fs::ReadError::NotInBase {
                path,
                base,
            } => (
                path.to_string_lossy(),
                format!(
                    "Path {} is not within base {}",
                    path.display(),
                    base.display()
                ),
            ),
        };
        #[expect(
            clippy::expect_used,
            reason = "Static fallback 'vault.md' is always a valid NotePath"
        )]
        let note_path = NotePath::try_new(&path_str).unwrap_or_else(|_| {
            NotePath::try_new("vault.md").expect("static fallback valid")
        });
        NoteFileError::ReadFailed {
            path: note_path,
            message: message.into(),
        }
        .into()
    }
}

impl From<StructureError> for NoteIngestError {
    #[inline]
    fn from(err: StructureError) -> Self {
        NoteIngestError::Domain(err.into())
    }
}

/// Orchestration error returned by the Note processor pipeline.
///
/// Distinguishes between failures in the ingestion, domain validation,
/// and persistence phases of the note lifecycle.
#[derive(Debug, thiserror::Error)]
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

/// Errors surfaced during note processing.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum NoteProcessError {
    /// Markdown ingestion failed.
    #[error("ingestion failed: {0}")]
    Ingest(#[from] NoteIngestError),

    /// Repository access failed.
    #[error("repository failed: {0}")]
    Repository(#[from] NoteRepositoryError),

    /// Note validation failed.
    #[error("validation failed: {0}")]
    Validation(#[from] NoteError),
}

impl From<crate::db::DbError> for NoteLoadError {
    #[inline]
    fn from(err: crate::db::DbError) -> Self {
        NoteLoadError::Persistence(NoteRepositoryError::Storage(err))
    }
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

    /// Enabled parser extension is not yet represented by parser IR.
    #[error(
        "unsupported enabled extension '{extension}' (option: \
         {parser_option}, stage: {stage}, range: {range:?})"
    )]
    UnsupportedEnabledExtension {
        /// Human-readable extension name.
        extension: &'static str,
        /// Option flag that enabled this extension.
        parser_option: &'static str,
        /// Parser stage where support is missing.
        stage: &'static str,
        /// Optional source location for the unsupported construct.
        range: Option<SourceByteRange>,
    },

    /// Parser policy contract was violated while adapting or normalizing
    /// events.
    #[error(
        "policy violation '{policy}' (expected: {expected}, observed: \
         {observed}, range: {range:?})"
    )]
    PolicyViolation {
        /// The policy name or identifier.
        policy: &'static str,
        /// Expected behavior under policy.
        expected: &'static str,
        /// Observed behavior or condition.
        observed: &'static str,
        /// Optional source location where violation was detected.
        range: Option<SourceByteRange>,
    },

    /// Parser block stack underflow while processing a closing token.
    #[error(
        "event stack underflow (expected: {expected}, encountered: \
         {encountered}, depth: {depth}, range: {range:?})"
    )]
    EventStackUnderflow {
        /// Expected open element kind.
        expected: &'static str,
        /// Encountered closing element kind.
        encountered: &'static str,
        /// Stack depth at failure.
        depth: usize,
        /// Source range for the closing token.
        range: SourceByteRange,
    },

    /// Parser block stack close token does not match the active open token.
    #[error(
        "event stack mismatch (expected: {expected}, found: {found}, depth: \
         {depth}, start_range: {start_range:?}, end_range: {end_range:?})"
    )]
    EventStackMismatch {
        /// Expected closing kind for current stack top.
        expected: &'static str,
        /// Actual closing kind found in stream.
        found: &'static str,
        /// Stack depth at mismatch.
        depth: usize,
        /// Source range of the opening token if available.
        start_range: Option<SourceByteRange>,
        /// Source range of the mismatched closing token.
        end_range: SourceByteRange,
    },

    /// End-of-document reached with still-open block containers.
    #[error(
        "unclosed blocks at end of document (open_count: {open_count}, \
         top_kind: {top_kind:?}, at: {at:?})"
    )]
    UnclosedBlocks {
        /// Number of open blocks remaining on the stack.
        open_count: usize,
        /// Kind of top-most open block if known.
        top_kind: Option<&'static str>,
        /// Source offset where EOF was observed.
        at: SourceByteOffset,
    },

    /// Structural topology violation in parser state machine.
    #[error("invalid parser topology ({code}): {detail} (range: {range:?})")]
    InvalidTopology {
        /// Stable diagnostic code for this topology class.
        code: &'static str,
        /// Human-readable details.
        detail: Box<str>,
        /// Optional source location where the violation was observed.
        range: Option<SourceByteRange>,
    },
}

/// Errors surfaced by the persistence and repository layer.
#[derive(Debug, thiserror::Error)]
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

    /// The list item is not a checkbox (missing task status marker).
    #[error("item is not a checkbox task: {text}")]
    MissingStatus {
        /// The item's text content.
        text: Box<str>,
    },

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
    #[error("source offset {offset:?} overflow by {delta}")]
    OffsetOverflow {
        /// The base byte offset.
        offset: SourceByteOffset,
        /// The relative delta being applied.
        delta: usize,
    },

    /// A byte offset refers to a position outside the current source buffer.
    #[error(
        "source offset {offset:?} out of bounds (source length: \
         {source_len:?})"
    )]
    OutOfBounds {
        /// Problematic offset.
        offset: SourceByteOffset,
        /// Total buffer length.
        source_len: SourceByteOffset,
    },

    /// A source range is malformed (start position is after end position).
    #[error("invalid source range: start {start:?} > end {end:?}")]
    InvalidRange {
        /// Range start offset.
        start: SourceByteOffset,
        /// Range end offset.
        end: SourceByteOffset,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests error conversion chains and `#[from]` implementations.
    mod conversions {

        use super::*;
        use crate::db::DbError;

        #[test]
        fn db_error_to_load_error_chain() {
            let io_err = std::io::Error::other("test failure");
            let storage_err = redb::StorageError::from(io_err);
            let db_err = DbError::Table(redb::TableError::from(storage_err));
            let load_err: NoteLoadError = db_err.into();

            assert!(
                matches!(
                    &load_err,
                    NoteLoadError::Persistence(NoteRepositoryError::Storage(source))
                    if source.to_string().contains("test failure")
                ),
                "Expected Persistence(Storage) error chain, got: {load_err:?}"
            );
        }

        #[test]
        fn tag_error_to_domain_umbrella() {
            let tag_err = TagError::EmptyTag;
            let note_err: NoteError = tag_err.into();
            assert!(matches!(note_err, NoteError::Tag(TagError::EmptyTag)));
        }

        #[test]
        fn file_error_to_ingest_umbrella() {
            let file_err = NoteFileError::InvalidPath {
                path: "bad/path".into(),
                reason: "traversal",
            };
            let ingest_err: NoteIngestError = file_err.into();
            assert!(matches!(
                ingest_err,
                NoteIngestError::File(NoteFileError::InvalidPath { .. })
            ));
        }

        #[test]
        fn read_io_error_to_ingest_umbrella() {
            let io_err = std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            );
            let read_err = crate::fs::ReadError::Io {
                path: PathBuf::from("daily/note.md"),
                source: io_err,
            };
            let ingest_err: NoteIngestError = read_err.into();
            assert!(
                matches!(
                    &ingest_err,
                    NoteIngestError::File(NoteFileError::ReadFailed { path, message })
                    if path.as_str() == "daily/note.md" && message.as_ref().contains("not found")
                ),
                "Expected ReadFailed with path and message, got: \
                 {ingest_err:?}"
            );
        }

        #[test]
        fn read_not_in_base_to_ingest_umbrella() {
            let read_err = crate::fs::ReadError::NotInBase {
                path: PathBuf::from("../outside.md"),
                base: PathBuf::from("/vault"),
            };
            let ingest_err: NoteIngestError = read_err.into();
            assert!(
                matches!(
                    &ingest_err,
                    NoteIngestError::File(NoteFileError::ReadFailed { message, .. })
                    if message.as_ref().contains("outside")
                ),
                "Expected ReadFailed with boundary message, got: \
                 {ingest_err:?}"
            );
        }
    }

    /// Tests the `Display` implementation for various error variants.
    mod formatting {
        use std::path::PathBuf;

        use rstest::rstest;

        use super::*;
        use crate::note::aggregate::NoteId;

        #[test]
        fn note_file_error_formatting() {
            let escape_err = NoteFileError::VaultRootEscape {
                path: PathBuf::from("/etc/passwd"),
            };
            assert!(escape_err.to_string().contains("/etc/passwd"));
            assert!(escape_err.to_string().contains("outside the vault root"));

            let ext_err = NoteFileError::UnsupportedExtension {
                path: "note.txt".into(),
                found: "txt".into(),
            };
            assert!(ext_err.to_string().contains("note.txt"));
            assert!(ext_err.to_string().contains("expected .md, found 'txt'"));
        }

        #[test]
        fn note_parse_error_formatting() {
            let markdown_err = NoteParseError::Markdown {
                line: 10,
                column: 5,
                reason: "unbalanced bracket".into(),
            };
            let msg = markdown_err.to_string();
            assert!(msg.contains("line 10"));
            assert!(msg.contains("col 5"));
            assert!(msg.contains("unbalanced bracket"));

            let large_err = NoteParseError::SourceTooLarge {
                size: 5000,
                limit: 1000,
            };
            assert!(large_err.to_string().contains("5000 bytes"));
            assert!(large_err.to_string().contains("limit: 1000"));
        }

        #[test]
        fn repository_error_formatting() {
            let id = NoteId::new();
            let corruption_err = NoteRepositoryError::Corruption {
                id,
                reason: "invalid bytes".into(),
            };
            assert!(corruption_err.to_string().contains(&id.to_string()));
            assert!(corruption_err.to_string().contains("invalid bytes"));

            let limit_err = NoteRepositoryError::ResourceLimitExceeded {
                current: 100,
                limit: 50,
                context: "tasks",
            };
            assert!(limit_err.to_string().contains("tasks has 100 items"));
            assert!(limit_err.to_string().contains("limit: 50"));
        }

        #[rstest]
        #[case::tag(TagError::MissingHash, "must start with '#'")]
        #[case::task(TaskError::EmptyText, "text cannot be empty")]
        #[case::link(LinkError::EmptyTarget, "target cannot be empty")]
        #[case::heading(HeadingError::EmptyContent, "content cannot be empty")]
        #[case::list(ListError::MaxNestingExceeded { current: 10, limit: 5 }, "depth: 10, limit: 5")]
        fn sub_domain_error_formatting(
            #[case] err: impl std::fmt::Display,
            #[case] expected: &str,
        ) {
            assert!(err.to_string().contains(expected));
        }
    }

    /// Tests the `FrontmatterError` contextual helper.
    mod frontmatter_helpers {
        use super::*;

        #[test]
        fn with_key_adds_missing_context() {
            let err = FrontmatterError::KeyMissing {
                key: "".into(),
            };
            let err = err.with_key("author");
            assert!(
                matches!(
                    &err,
                    FrontmatterError::KeyMissing { key }
                    if key.as_ref() == "author"
                ),
                "Expected KeyMissing with 'author' context, got: {err:?}"
            );
        }

        #[test]
        fn with_key_does_not_overwrite_existing_context() {
            let err = FrontmatterError::TypeMismatch {
                key: "date".into(),
                expected: "string",
                actual: "number",
            };
            let err = err.with_key("original");
            assert!(
                matches!(
                    &err,
                    FrontmatterError::TypeMismatch { key, .. }
                    if key.as_ref() == "date"
                ),
                "Expected TypeMismatch to retain 'date' context, got: {err:?}"
            );
        }

        #[test]
        fn with_key_ignores_non_extraction_errors() {
            let err = FrontmatterError::InvalidAlias {
                value: "bad alias".into(),
                reason: "empty",
            };
            let err = err.with_key("ignored");
            assert!(
                matches!(
                    &err,
                    FrontmatterError::InvalidAlias { value, .. }
                    if value.as_ref() == "bad alias"
                ),
                "Expected InvalidAlias to retain its value, got: {err:?}"
            );
        }
    }

    /// Tests for equality logic and deep enum matching.
    mod logic {
        use super::*;

        #[test]
        fn error_equality_with_nested_variants() {
            let err1 = NoteError::Tag(TagError::InvalidSegment {
                segment: "work".into(),
                reason: "bad char",
            });
            let err2 = NoteError::Tag(TagError::InvalidSegment {
                segment: "work".into(),
                reason: "bad char",
            });
            let err3 = NoteError::Tag(TagError::InvalidSegment {
                segment: "life".into(),
                reason: "bad char",
            });

            assert_eq!(err1, err2);
            assert_ne!(err1, err3);
        }

        #[test]
        fn matches_named_fields_with_guards() {
            let err = NoteParseError::Markdown {
                line: 42,
                column: 1,
                reason: "EOF".into(),
            };

            assert!(matches!(
                err,
                NoteParseError::Markdown {
                    line,
                    column,
                    ..
                } if line == 42 && column == 1
            ));
        }
    }

    mod thread_safety {
        use super::*;

        fn is_send_sync<T: Send + Sync>() {}

        #[test]
        fn errors_are_send_and_sync() {
            is_send_sync::<NoteError>();
            is_send_sync::<NoteIngestError>();
            is_send_sync::<NoteLoadError>();
            is_send_sync::<NoteProcessError>();
            is_send_sync::<NoteRepositoryError>();
        }
    }
}
