//! Filesystem-related utilities and infrastructure.
//!
//! This module contains file system infrastructure for security validation,
//! path manipulation, and structured data parsing.
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
//! - **parsers**: TOML/JSON/YAML parsing strategies with auto-detection.
//!   - Strategy pattern implementation for structured data formats.
//!   - Re-exported as `FormatDispatcher` for clarity in calling code.
//! - **markdown**: Offset-aware markdown parsing utilities.
//!   - Wraps pulldown-cmark for adapter layers.

#![expect(
    clippy::module_name_repetitions,
    reason = "Namespaced types improve clarity in calling code"
)]
/// Filesystem error types.
pub mod error;
/// Markdown parsing utilities.
pub mod markdown;
/// Structured data parsers (TOML/JSON/YAML).
pub mod parsers;
/// File system abstraction for testable file I/O.
pub mod source;
/// Security-critical path validation utilities.
pub mod validator;

// Ergonomic aliases with domain-clarifying names (avoid `pub use` re-exports).

/// Format dispatcher type alias.
pub type FormatDispatcher = parsers::Dispatcher;
/// Markdown parser type alias.
pub type MarkdownParser = markdown::MarkdownParser;
/// Markdown offset iterator type alias.
pub type MarkdownOffsetIter<'markdown> = pulldown_cmark::OffsetIter<
    'markdown,
    pulldown_cmark::DefaultBrokenLinkCallback,
>;
/// Filesystem error type alias.
pub type FsError = error::FsError;
/// Parse error type alias.
pub type ParseError = error::ParseError;
/// Path validator type alias.
pub type PathValidator = validator::Validator;
/// Path validation error type alias.
pub type PathValidationError = error::PathValidationError;
/// Filesystem file source type alias.
pub type FsFileSource = source::FsFileSource;
/// In-memory file source type alias.
pub type InMemoryFileSource = source::InMemoryFileSource;

/// Checks if a path is a Windows-style absolute path or drive-relative path.
#[inline]
#[must_use]
pub fn is_windows_absolute_path(path: &str) -> bool {
    validator::is_windows_absolute_path(path)
}

/// Validates a vault-relative path.
///
/// Bundles common path constraints: non-empty, relative, no traversal,
/// optional extension.
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
