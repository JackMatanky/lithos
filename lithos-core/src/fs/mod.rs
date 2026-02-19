//! Filesystem-related utilities and infrastructure.
//!
//! This module contains file system infrastructure for security validation,
//! deterministic discovery, read pipelines, and safe write orchestration.
//! It centralizes the file I/O policy surface so adapter layers can depend on
//! consistent, audited behavior instead of ad-hoc filesystem calls.
//!
//! ## Security-Critical Modules
//!
//! - **validator**: Path traversal protection and security validation.
//!   - Prevents path traversal attacks, symlink escapes, and arbitrary file
//!     access.
//!   - Re-exported as `PathValidator` for ergonomic imports.
//!
//! ## Data Processing Modules
//!
//! - **reader**: Root-scoped file access with validation and classification.
//!   - Read pipeline: validate → classify → read → parse.
//! - **types**: TOML/JSON/YAML parsing helpers with explicit format guards.
//! - **writer**: Root-scoped writes with atomic replace.

/// Filesystem error types.
pub mod error;
/// File system abstraction for testable file I/O.
pub mod reader;
/// Structured data parsers (TOML/JSON/YAML).
pub mod types;
/// Security-critical path validation utilities.
pub mod validator;
/// File system writer utilities.
pub mod writer;

// Ergonomic aliases with domain-clarifying names.
//
// These aliases keep call sites explicit about filesystem boundaries while
// avoiding long module paths in adapters.

/// Filesystem reader type alias.
#[expect(
    clippy::module_name_repetitions,
    reason = "Alias keeps explicit fs namespace in callers."
)]
pub type FsReader = reader::Reader;
/// Filesystem writer type alias.
#[expect(
    clippy::module_name_repetitions,
    reason = "Alias keeps explicit fs namespace in callers."
)]
pub type FsWriter = writer::Writer;
/// File metadata type alias.
pub type FileMetadata = reader::FileMetadata;
/// Filesystem error type alias.
#[expect(
    clippy::module_name_repetitions,
    reason = "Alias keeps explicit fs namespace in callers."
)]
pub type FsError = error::FsError;
/// Parse error type alias.
pub type ParseError = error::ParseError;
/// JSON parser type alias.
pub type Json = types::Json;
/// TOML parser type alias.
pub type Toml = types::Toml;
/// YAML parser type alias.
pub type Yaml = types::Yaml;
/// Markdown file type alias.
pub type Markdown = types::Markdown;
/// Path validator type alias.
pub type PathValidator = validator::Validator;
/// Path validation error type alias.
pub type PathValidationError = error::PathValidationError;

/// Checks if a path is a Windows-style absolute path or drive-relative path.
#[inline]
#[must_use]
pub fn is_windows_absolute_path(path: &str) -> bool {
    validator::is_windows_absolute_path(path)
}

/// Validates a vault-relative path.
///
/// Bundles common path constraints: non-empty, relative, no traversal,
/// optional extension. Use this helper to keep adapter call sites aligned on
/// the same security rules.
///
/// # Errors
///
/// Returns [`PathValidationError`] if the path is invalid or does not match
/// the required extension.
#[inline]
pub fn validate_vault_path(
    path: &str,
    require_extension: Option<&str>,
) -> Result<(), PathValidationError> {
    validator::validate_vault_path(path, require_extension)
}
