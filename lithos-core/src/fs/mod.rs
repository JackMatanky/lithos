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
//!   metadata access via [`FileReader`].
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
//! | Read files from a vault root     | [`FileReader::new`]                                |
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
/// Security-critical path validation utilities.
pub mod validator;
/// Root-scoped filesystem writer with atomic-replace semantics.
pub mod writer;

// ─── Public Re-exports ──────────────────────────────────────────────────────
//
// Re-export commonly used types at the `fs::` path so adapter call sites stay
// readable without long module chains.

pub use entry::{FsDir, FsEntry, FsFile};
pub use error::{
    FsError, ParseError, PathError, PathValidationError, ReadError, ScanError,
};
pub use format::{FileExtensionRef, FileFormat, StructuredFileFormat};
pub use metadata::{DirMetadata, FileMetadata, FsMetadata, FsTimes};
pub use name::{
    BaseName, BaseNameRef, DirName, DirNameRef, FileName, FileNameRef,
};
pub use path::{
    AbsolutePath, DirPath, FilePath, FsPath, FsPathRef, PathKey, RelativePath,
};
pub use reader::FileReader;
pub use scanner::{DirScanInput, DirScanner};
pub use validator::Validator as PathValidator;
