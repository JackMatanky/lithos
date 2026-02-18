//! Error types for filesystem and parsing operations.

/// File system error types.
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific error name is intentional"
)]
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum FsError {
    /// IO operation failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors that can occur during config file parsing.
///
/// These errors provide rich context including file paths, line numbers,
/// and format-specific error details to aid debugging.
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific error name is intentional"
)]
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// I/O error reading file.
    #[error("I/O error reading {path}: {source}")]
    Io {
        /// File path where error occurred.
        path: std::path::PathBuf,
        /// Source I/O error.
        #[source]
        source: std::io::Error,
    },

    /// JSON parsing failed.
    #[error(
        "JSON parse error in {path}: {message} at line {line:?}, column \
         {column:?}"
    )]
    Json {
        /// File path where error occurred.
        path: std::path::PathBuf,
        /// Error message from parser.
        message: Box<str>,
        /// Line number where error occurred.
        line: Option<usize>,
        /// Column number where error occurred.
        column: Option<usize>,
    },

    /// TOML parsing failed.
    #[error(
        "TOML parse error in {path}: {message} at line {line:?}, column \
         {column:?}"
    )]
    Toml {
        /// File path where error occurred.
        path: std::path::PathBuf,
        /// Error message from parser.
        message: Box<str>,
        /// Line number where error occurred.
        line: Option<usize>,
        /// Column number where error occurred.
        column: Option<usize>,
    },

    /// Unsupported file format.
    #[error("Unsupported format for {path:?}: expected one of {supported:?}")]
    UnsupportedFormat {
        /// File path with unsupported extension.
        path: std::path::PathBuf,
        /// List of supported extensions.
        supported: &'static [&'static str],
    },

    /// YAML parsing failed.
    #[error(
        "YAML parse error in {path}: {message} at line {line:?}, column \
         {column:?}"
    )]
    Yaml {
        /// File path where error occurred.
        path: std::path::PathBuf,
        /// Error message from parser.
        message: Box<str>,
        /// Line number where error occurred.
        line: Option<usize>,
        /// Column number where error occurred.
        column: Option<usize>,
    },
}

/// Path validation error types.
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific error name is intentional"
)]
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PathValidationError {
    /// Path is absolute when only relative paths are allowed.
    #[error("Absolute path not allowed: {0}")]
    AbsolutePathError(String),

    /// Path contains invalid encoding (non-UTF8).
    #[error("Path contains invalid encoding: {0}")]
    InvalidPathEncoding(String),

    /// I/O error during symlink resolution.
    ///
    /// # Note
    ///
    /// This stores a string because `PathValidationError` requires `Clone +
    /// Eq`, which `std::io::Error` does not implement.
    #[error("I/O error during symlink resolution: {0}")]
    IoError(String),

    /// Path contains `..` components attempting traversal outside allowed
    /// directory.
    #[error("Path traversal detected: path contains '..' components")]
    PathTraversalError,

    /// Path accesses restricted or hidden files.
    #[error("Restricted path access denied: {0}")]
    RestrictedPathError(String),

    /// Symlink target escapes the configured root directory.
    #[error("Symlink escape detected: target is outside root boundary")]
    SymlinkEscapeError,
}
