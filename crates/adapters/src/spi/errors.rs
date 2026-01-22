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

    #[test]
    fn parse_error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ParseError>();
    }

    #[test]
    fn toml_error_display_includes_context() {
        let error = ParseError::Toml {
            path: PathBuf::from("config.toml"),
            message: "unexpected token".to_owned(),
            line: Some(10),
            column: Some(5),
        };
        let display = error.to_string();
        assert!(display.contains("config.toml"));
        assert!(display.contains("unexpected token"));
        assert!(display.contains("10"));
    }

    #[test]
    fn json_error_display_includes_context() {
        let error = ParseError::Json {
            path: PathBuf::from("data.json"),
            message: "trailing comma".to_owned(),
            line: Some(42),
            column: Some(8),
        };
        let display = error.to_string();
        assert!(display.contains("data.json"));
        assert!(display.contains("trailing comma"));
        assert!(display.contains("42"));
    }

    #[test]
    fn unsupported_format_lists_supported_extensions() {
        let error = ParseError::UnsupportedFormat {
            path: PathBuf::from("config.xml"),
            supported: vec!["toml", "json", "yaml"],
        };
        let display = error.to_string();
        assert!(display.contains("config.xml"));
        assert!(display.contains("toml"));
        assert!(display.contains("json"));
        assert!(display.contains("yaml"));
    }
}
