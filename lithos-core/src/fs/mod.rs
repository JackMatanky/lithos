//! Filesystem infrastructure for the Lithos core library.
//!
//! This module centralises all file I/O policy so adapter layers depend on
//! consistent, audited behaviour instead of ad-hoc `std::fs` calls. It
//! provides:
//!
//! - **Security validation** — path traversal protection, symlink escape
//!   detection, and hidden-file blocking via [`PathValidator`].
//! - **Root-scoped file access** — deterministic discovery, read pipelines, and
//!   metadata access via [`FsReader`].
//! - **Safe write orchestration** — atomic replace semantics via [`FsWriter`].
//! - **Structured data parsing** — JSON/TOML/YAML parsing helpers with explicit
//!   format guards (module-internal).
//!
//! # Access points
//!
//! | What you need                    | How to get it                                    |
//! |----------------------------------|--------------------------------------------------|
//! | Validate a vault path string     | [`PathValidator::validate_vault_path`]           |
//! | Validate an arbitrary path       | [`PathValidator::new_flexible`] + `.validate()`  |
//! | Read files from a vault root     | [`FsReader::new`]                                |
//! | Write files to a vault root      | [`FsWriter::new`]                                |
//!
//! # Security note
//!
//! All path validation for vault operations is centralised in
//! [`PathValidator`]. Bypassing it in adapter code creates path-traversal and
//! symlink-escape vulnerabilities.

/// Filesystem error types.
pub mod error;
/// Root-scoped file reader with validation and format-classification pipeline.
pub mod reader;
/// Structured data parsers (TOML/JSON/YAML) — module-internal.
pub(crate) mod types;
/// Security-critical path validation utilities.
pub mod validator;
/// Root-scoped filesystem writer with atomic-replace semantics.
pub mod writer;

// ─── Public type aliases ────────────────────────────────────────────────────
//
// These aliases surface the most commonly used types at the `fs::` path so
// adapter call sites stay readable without long module chains.

/// Root-scoped filesystem reader.
///
/// See [`reader::Reader`] for the full API.
#[expect(
    clippy::module_name_repetitions,
    reason = "The `Fs` prefix is intentional: it makes the filesystem \
              boundary explicit at call sites (`FsReader` vs a bare \
              `Reader`). Removing the prefix would conflict with domain \
              reader types."
)]
pub type FsReader = reader::Reader;

/// Root-scoped filesystem writer.
///
/// See [`writer::Writer`] for the full API.
///
/// Currently `pub(crate)` — will be promoted to `pub` when the template
/// module provides its first caller.
#[expect(
    dead_code,
    reason = "FsWriter has no external callers yet; it will be promoted to \
              `pub` when the template module adapter is implemented."
)]
pub(crate) type FsWriter = writer::Writer;

/// Parse error type alias.
///
/// See [`error::ParseError`] for all variants.
pub type ParseError = error::ParseError;

/// Path validator type alias.
///
/// The primary entry point for all path validation. Use
/// [`PathValidator::validate_vault_path`] for string-based vault path
/// validation, or construct a validator with [`PathValidator::new_flexible`] /
/// [`PathValidator::try_new_strict`] for finer control.
pub type PathValidator = validator::Validator;

/// Path validation error type alias.
///
/// See [`error::PathValidationError`] for all variants.
pub type PathValidationError = error::PathValidationError;

/// File system timestamp type alias.
///
/// Represents file creation/modification timestamps as seconds since Unix
/// epoch. This is an infrastructure primitive used only in the adapter layer.
///
/// See [`reader::FileTimestamp`] for the full API.
pub type FileTimestamp = reader::FileTimestamp;
