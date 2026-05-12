//! Filesystem infrastructure for the Lithos core library.
//!
//! This module centralises all file I/O policy so adapter layers depend on
//! consistent, audited behaviour instead of ad-hoc `std::fs` calls. It
//! provides:
//!
//! - **Security validation** — path traversal protection, symlink escape
//!   detection, and hidden-file blocking via [`PathValidator`].
//! - **Directory scanning** — configurable traversal with glob patterns,
//!   extension filters, and depth control via [`DirScanner`].
//! - **Root-scoped file access** — deterministic discovery, read pipelines, and
//!   metadata access via [`FsReader`].
//! - **Safe write orchestration** — atomic replace semantics via [`FsWriter`].
//! - **Structured data parsing** — JSON/TOML/YAML parsing with explicit format
//!   detection and validation.
//!
//! # Access points
//!
//! | What you need                    | How to get it                                    |
//! |----------------------------------|--------------------------------------------------|
//! | Validate a vault path string     | [`PathValidator::validate_vault_path`]           |
//! | Validate an arbitrary path       | [`PathValidator::new_flexible`] + `.validate()`  |
//! | Scan directory for files         | [`DirScanner::new`] + `.paths()` or `.entries()` |
//! | Read files from a vault root     | [`FsReader::new`]                                |
//! | Write files to a vault root      | [`FsWriter::new`]                                |
//!
//! # Security note
//!
//! All path validation for vault operations is centralised in
//! [`PathValidator`]. Bypassing it in adapter code creates path-traversal and
//! symlink-escape vulnerabilities.

#![allow(
    clippy::pub_use,
    reason = "Intentional re-exports for flat, ergonomic public API"
)]

/// Filesystem entry types for files and directories.
pub mod entry;
/// Filesystem error types.
pub mod error;
/// Ergonomic conversions for filesystem entries.
pub mod file;
/// File format detection and classification.
pub mod format;
/// Filesystem metadata types for files and directories.
pub mod metadata;
/// Owned and borrowed name components.
pub mod name;
/// Path types for filesystem primitives.
pub mod path;
/// Root-scoped file reader with validation and format-classification pipeline.
pub mod reader;
/// Directory scanning utilities for finding files matching criteria.
pub mod scanner;
/// Structured data parsers (TOML/JSON/YAML) — module-internal.
pub(crate) mod types;
/// Security-critical path validation utilities.
pub mod validator;
/// Root-scoped filesystem writer with atomic-replace semantics.
pub mod writer;

// ─── Public Re-exports ──────────────────────────────────────────────────────
//
// Re-export commonly used types at the `fs::` path so adapter call sites stay
// readable without long module chains.

pub use entry::{FsDir, FsEntry, FsFile};
pub use error::{ParseError, PathValidationError};
pub use file::{FileEntry, FileInfo, FileName};
pub use format::{FileExtensionRef, FileFormat};
pub use metadata::{DirMetadata, FileMetadata, FsMetadata, FsTimes};
pub use name::{BaseName, BaseNameRef, DirName, DirNameRef, FileNameRef};
pub use path::{AbsolutePath, DirPath, FilePath, FsPath, RelativePath};
#[expect(
    clippy::module_name_repetitions,
    reason = "FsReader alias clarifies this is the filesystem reader in \
              re-exports"
)]
pub use reader::{FileTimestamp, Reader as FsReader};
pub use scanner::{DirScanInput, DirScanner};
pub use validator::Validator as PathValidator;
