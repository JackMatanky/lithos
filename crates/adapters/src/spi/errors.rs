//! Adapter error types for Lithos infrastructure.
//!
//! This module defines adapter-level errors for infrastructure operations
//! (file parsing, I/O, etc.) following hexagonal architecture principles.
//!
//! # Error Handling Strategy
//! - Use `thiserror::Error` for all adapter errors
//! - Each error variant includes descriptive context
//! - Errors are `Send + Sync` for use across async boundaries
//! - Errors include file paths, line numbers, and actionable messages

/// Errors that can occur during config file parsing.
///
/// These errors provide rich context including file paths, line numbers,
/// and format-specific error details to aid debugging.
///
/// # Memory Layout
///
/// Error context is boxed to keep the enum small (~24 bytes on 64-bit).
/// This prevents `Result<T, ParseError>` from bloating function signatures
/// with large error payloads (which is critical for hot parsing paths).
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// JSON parsing failed.
    #[error(
        "JSON parse error in {path}: {message} at line {line:?}, column \
         {column:?}"
    )]
    Json {
        /// File path where error occurred (boxed to reduce enum size).
        path: Box<std::path::Path>,
        /// Error message from parser (boxed to reduce enum size).
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
        /// File path where error occurred (boxed to reduce enum size).
        path: Box<std::path::Path>,
        /// Error message from parser (boxed to reduce enum size).
        message: Box<str>,
        /// Line number where error occurred.
        line: Option<usize>,
        /// Column number where error occurred.
        column: Option<usize>,
    },

    /// Unsupported file format.
    #[error("Unsupported format for {path:?}: expected one of {supported:?}")]
    UnsupportedFormat {
        /// File path with unsupported extension (boxed to reduce enum size).
        path: Box<std::path::Path>,
        /// List of supported extensions.
        supported: Vec<&'static str>,
    },

    /// YAML parsing failed.
    #[error(
        "YAML parse error in {path}: {message} at line {line:?}, column \
         {column:?}"
    )]
    Yaml {
        /// File path where error occurred (boxed to reduce enum size).
        path: Box<std::path::Path>,
        /// Error message from parser (boxed to reduce enum size).
        message: Box<str>,
        /// Line number where error occurred.
        line: Option<usize>,
        /// Column number where error occurred.
        column: Option<usize>,
    },
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    // [4.1-U-10] Thread Safety
    #[test]
    fn should_be_send_and_sync() {
        // Given a generic function that requires Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}

        // Then ParseError should satisfy these bounds (compilation check)
        assert_send_sync::<ParseError>();
    }

    // [4.3-R-01] Memory Layout - ParseError must stay compact
    #[test]
    fn parse_error_must_respect_large_error_threshold() {
        // Given the large-error-threshold from clippy.toml (128 bytes)
        const THRESHOLD: usize = 128;
        let size = std::mem::size_of::<ParseError>();

        // Then ParseError should not exceed this threshold
        assert!(
            size <= THRESHOLD,
            "ParseError is {size} bytes, exceeds clippy.toml \
             large-error-threshold of {THRESHOLD} bytes. Consider boxing \
             large variants."
        );
    }

    // [4.1-U-11] TOML Error Display
    #[test]
    fn should_include_context_in_toml_error() {
        // Given a TOML error with specific line and column info
        let error = ParseError::Toml {
            path: PathBuf::from("config.toml").into_boxed_path(),
            message: "unexpected token".into(),
            line: Some(10),
            column: Some(5),
        };

        // When displaying the error as a string
        let display = error.to_string();

        // Then the output should contain the filename, message, and line number
        assert!(display.contains("config.toml"), "Should contain filename");
        assert!(display.contains("unexpected token"), "Should contain message");
        assert!(display.contains("10"), "Should contain line number");
    }

    // [4.1-U-12] JSON Error Display
    #[test]
    fn should_include_context_in_json_error() {
        // Given a JSON error with specific line and column info
        let error = ParseError::Json {
            path: PathBuf::from("data.json").into_boxed_path(),
            message: "trailing comma".into(),
            line: Some(42),
            column: Some(8),
        };

        // When displaying the error as a string
        let display = error.to_string();

        // Then the output should contain the filename, message, and line number
        assert!(display.contains("data.json"), "Should contain filename");
        assert!(display.contains("trailing comma"), "Should contain message");
        assert!(display.contains("42"), "Should contain line number");
    }

    // [4.1-U-13] Unsupported Format Error Display
    #[test]
    fn should_list_supported_extensions_in_error() {
        // Given an unsupported format error
        let error = ParseError::UnsupportedFormat {
            path: PathBuf::from("config.xml").into_boxed_path(),
            supported: vec!["toml", "json", "yaml"],
        };

        // When displaying the error as a string
        let display = error.to_string();

        // Then the output should contain the filename and list of supported
        // extensions
        assert!(display.contains("config.xml"), "Should contain filename");
        assert!(display.contains("toml"), "Should list toml");
        assert!(display.contains("json"), "Should list json");
        assert!(display.contains("yaml"), "Should list yaml");
    }

    // [4.2-U-10] Path Validation Thread Safety
    #[test]
    fn should_path_validation_be_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PathValidationError>();
    }

    // [4.3-R-02] Memory Layout - PathValidationError must stay compact
    #[test]
    fn path_validation_error_must_respect_large_error_threshold() {
        // Given the large-error-threshold from clippy.toml (128 bytes)
        const THRESHOLD: usize = 128;
        let size = std::mem::size_of::<PathValidationError>();

        // Then PathValidationError should not exceed this threshold
        assert!(
            size <= THRESHOLD,
            "PathValidationError is {size} bytes, exceeds clippy.toml \
             large-error-threshold of {THRESHOLD} bytes. Consider boxing \
             large variants."
        );
    }

    // [5.1-U-01] CacheError Type Existence
    #[test]
    fn should_find_cache_error_type() {
        // This test will fail to compile if CacheError is not defined
        fn assert_type_exists<T>() {}
        assert_type_exists::<CacheError>();
    }

    // [5.1-U-04] CacheError::IoError From std::io::Error
    #[test]
    fn should_create_io_error_from_std_io_error() {
        let io_error =
            std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let error: CacheError = io_error.into();
        assert!(
            matches!(error, CacheError::IoError(_)),
            "Expected IoError, but got {error:?}"
        );
    }

    // [5.1-U-05] CacheError::SerializationError
    #[test]
    fn should_create_serialization_error() {
        let error = CacheError::SerializationError {
            type_name: "test_type",
            message: "failed to serialize".into(),
        };
        let display = error.to_string();
        assert!(display.contains("failed to serialize"));
        assert!(display.contains("test_type"));
    }

    // [5.1-U-06] CacheError::BackendError
    #[test]
    fn should_create_backend_error() {
        let error = CacheError::BackendError {
            backend: "moka",
            message: "eviction".into(),
        };
        let display = error.to_string();
        assert!(display.contains("eviction"));
        assert!(display.contains("moka"));
    }

    // [5.1-U-08] CacheError::PartialWrite
    #[test]
    fn should_create_partial_write_error() {
        let error = CacheError::PartialWrite {
            backend: "coordinator",
            message: "disk committed, memory failed".into(),
        };
        let display = error.to_string();
        assert!(display.contains("partial write"));
        assert!(display.contains("coordinator"));
    }

    // [5.1-U-06] CacheError::RuntimeError
    #[test]
    fn should_create_runtime_error() {
        let error = CacheError::RuntimeError {
            runtime: "tokio",
            message: "missing runtime".into(),
        };
        let display = error.to_string();
        assert!(display.contains("missing runtime"));
        assert!(display.contains("tokio"));
    }

    // [5.1-U-07] CacheError Thread Safety
    #[test]
    fn cache_error_should_be_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CacheError>();
    }
}

/// Path validation error types.
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

/// Errors that can occur during cache operations.
///
/// # Example
///
/// ```rust
/// use lithos_adapters::spi::errors::CacheError;
///
/// let error = CacheError::BackendError {
///     backend: "memory",
///     message: "connection refused".into(),
/// };
/// assert!(error.to_string().contains("memory"));
/// ```
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CacheError {
    /// Backend-specific error.
    #[error("Cache backend ({backend}) error: {message}")]
    BackendError {
        /// The name of the backend (e.g., "moka", "redb").
        backend: &'static str,
        /// Descriptive error message.
        message: Box<str>,
    },

    /// I/O error during cache access.
    #[error("Cache I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Partial write succeeded in one layer but failed in another.
    #[error("Cache partial write ({backend}): {message}")]
    PartialWrite {
        /// The name of the backend (e.g., "coordinator").
        backend: &'static str,
        /// Descriptive error message.
        message: Box<str>,
    },

    /// Runtime configuration error.
    #[error("Cache runtime ({runtime}) error: {message}")]
    RuntimeError {
        /// The runtime name (e.g., "tokio").
        runtime: &'static str,
        /// Descriptive error message.
        message: Box<str>,
    },

    /// Serialization/deserialization failed.
    #[error("Cache serialization error for type {type_name}: {message}")]
    SerializationError {
        /// The name of the type being serialized.
        type_name: &'static str,
        /// Descriptive error message.
        message: Box<str>,
    },
}
