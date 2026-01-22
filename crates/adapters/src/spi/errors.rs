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

use std::path::PathBuf;

/// Errors that can occur during config file parsing.
///
/// These errors provide rich context including file paths, line numbers,
/// and format-specific error details to aid debugging.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// JSON parsing failed.
    #[error(
        "JSON parse error in {path}: {message} at line {line:?}, column \
         {column:?}"
    )]
    Json {
        /// File path where error occurred.
        path: PathBuf,
        /// Error message from parser.
        message: String,
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
        path: PathBuf,
        /// Error message from parser.
        message: String,
        /// Line number where error occurred.
        line: Option<usize>,
        /// Column number where error occurred.
        column: Option<usize>,
    },

    /// Unsupported file format.
    #[error("Unsupported format for {path:?}: expected one of {supported:?}")]
    UnsupportedFormat {
        /// File path with unsupported extension.
        path: PathBuf,
        /// List of supported extensions.
        supported: Vec<&'static str>,
    },

    /// YAML parsing failed.
    #[error(
        "YAML parse error in {path}: {message} at line {line:?}, column \
         {column:?}"
    )]
    Yaml {
        /// File path where error occurred.
        path: PathBuf,
        /// Error message from parser.
        message: String,
        /// Line number where error occurred.
        line: Option<usize>,
        /// Column number where error occurred.
        column: Option<usize>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    // [4.1-U-10] Thread Safety
    #[test]
    fn should_be_send_and_sync() {
        // Given a generic function that requires Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}

        // Then ParseError should satisfy these bounds (compilation check)
        assert_send_sync::<ParseError>();
    }

    // [4.1-U-11] TOML Error Display
    #[test]
    fn should_include_context_in_toml_error() {
        // Given a TOML error with specific line and column info
        let error = ParseError::Toml {
            path: PathBuf::from("config.toml"),
            message: "unexpected token".to_owned(),
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
            path: PathBuf::from("data.json"),
            message: "trailing comma".to_owned(),
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
            path: PathBuf::from("config.xml"),
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
}
