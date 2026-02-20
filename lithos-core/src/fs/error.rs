//! Error types for filesystem and parsing operations.
//!
//! These errors preserve contextual data (paths, line/column, or extensions)
//! so adapter layers can surface actionable diagnostics without leaking
//! low-level I/O details into domain logic.

/// Errors that can occur during config file parsing.
///
/// Each variant carries enough context (file path, line/column, parser
/// message) for the caller to produce a human-readable diagnostic without
/// re-reading the file.
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific error name is intentional; re-exported as \
              ParseError from the fs module root."
)]
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// I/O error reading the file at `path`.
    #[error("I/O error reading {path}: {source}")]
    Io {
        /// File path where the error occurred.
        path: std::path::PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// JSON parsing failed at the given location.
    #[error(
        "JSON parse error in {path}: {message} at line {line:?}, column \
         {column:?}"
    )]
    Json {
        /// File path where the error occurred.
        path: std::path::PathBuf,
        /// Parser error message.
        message: Box<str>,
        /// Line number of the error, if available.
        line: Option<usize>,
        /// Column number of the error, if available.
        column: Option<usize>,
    },

    /// TOML parsing failed at the given location.
    #[error(
        "TOML parse error in {path}: {message} at line {line:?}, column \
         {column:?}"
    )]
    Toml {
        /// File path where the error occurred.
        path: std::path::PathBuf,
        /// Parser error message.
        message: Box<str>,
        /// Line number of the error, if available.
        line: Option<usize>,
        /// Column number of the error, if available.
        column: Option<usize>,
    },

    /// The file format is not supported by the parser.
    ///
    /// `supported` lists the extensions that the parser accepts. The caller
    /// should surface this to the user with a suggestion to rename the file.
    #[error("Unsupported format for {path:?}: expected one of {supported:?}")]
    UnsupportedFormat {
        /// File path with the unsupported extension.
        path: std::path::PathBuf,
        /// Extensions this parser accepts (e.g. `&["json"]`).
        supported: &'static [&'static str],
    },

    /// YAML parsing failed at the given location.
    #[error(
        "YAML parse error in {path}: {message} at line {line:?}, column \
         {column:?}"
    )]
    Yaml {
        /// File path where the error occurred.
        path: std::path::PathBuf,
        /// Parser error message.
        message: Box<str>,
        /// Line number of the error, if available.
        line: Option<usize>,
        /// Column number of the error, if available.
        column: Option<usize>,
    },
}

/// Errors produced by path validation.
///
/// Each variant is designed for human-readable reporting while retaining
/// structured fields for programmatic error handling and testing.
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific error name is intentional; re-exported as \
              PathValidationError from the fs module root."
)]
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PathValidationError {
    /// The path string was empty.
    #[error("Path cannot be empty")]
    EmptyPath,

    /// The path is absolute when only relative paths are allowed.
    #[error("Absolute path not allowed: {0}")]
    AbsolutePathError(std::path::PathBuf),

    /// The path contains invalid encoding (non-UTF-8).
    #[error("Path contains invalid encoding: {0}")]
    InvalidPathEncoding(std::path::PathBuf),

    /// An I/O error occurred during symlink resolution.
    ///
    /// The inner [`std::io::Error`] describes the underlying OS error (e.g.
    /// a permission denial or a symlink loop).
    #[error("I/O error during symlink resolution: {0}")]
    IoError(#[source] std::io::Error),

    /// The path contains `..` components that attempt to escape the allowed
    /// directory.
    #[error("Path traversal detected: path contains '..' components")]
    PathTraversalError,

    /// The path accesses a restricted or hidden file (a component starting
    /// with `.`).
    #[error("Restricted path access denied: {0}")]
    RestrictedPathError(std::path::PathBuf),

    /// The path extension does not match the required extension.
    #[error("Invalid path extension for {path}: expected .{required}")]
    InvalidExtension {
        /// File path with the wrong extension.
        path: std::path::PathBuf,
        /// Required extension without a leading dot (e.g. `"md"`).
        required: Box<str>,
    },

    /// The symlink target escapes the configured root directory.
    #[error("Symlink escape detected: target is outside root boundary")]
    SymlinkEscapeError,

    /// The root path supplied to [`Validator::try_new_strict`] is not
    /// absolute.
    ///
    /// Strict-mode validation requires an absolute, canonicalized root so
    /// that symlink boundary checks are reliable. Callers must canonicalize
    /// the path with [`std::fs::canonicalize`] before constructing a strict
    /// validator.
    ///
    /// [`Validator::try_new_strict`]: super::validator::Validator::try_new_strict
    #[error("Validator root must be absolute, got: {0}")]
    RelativeRoot(std::path::PathBuf),
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test modules use conventional use-before-mod ordering"
)]
mod tests {
    use std::path::PathBuf;

    use rstest::rstest;

    use super::*;

    mod parse_error {
        use super::*;

        #[test]
        fn io_error_includes_path() {
            let error = ParseError::Io {
                path: PathBuf::from("config/missing.json"),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "not found",
                ),
            };
            let msg = format!("{error}");
            assert!(msg.contains("config/missing.json"));
        }

        #[rstest]
        #[case::json(ParseError::Json {
            path: PathBuf::from("data.json"),
            message: "unexpected token".into(),
            line: Some(10),
            column: Some(5),
        }, "data.json")]
        #[case::toml(ParseError::Toml {
            path: PathBuf::from("config.toml"),
            message: "invalid key".into(),
            line: None,
            column: None,
        }, "config.toml")]
        fn includes_path_in_message(
            #[case] error: ParseError,
            #[case] expected_path: &str,
        ) {
            let msg = format!("{error}");
            assert!(msg.contains(expected_path));
        }

        #[test]
        fn unsupported_format_lists_formats() {
            let error = ParseError::UnsupportedFormat {
                path: PathBuf::from("data.xml"),
                supported: &["json", "toml", "yaml"],
            };
            let msg = format!("{error}");
            assert!(
                msg.contains("json")
                    && msg.contains("toml")
                    && msg.contains("yaml")
            );
        }
    }

    mod path_validation_error {
        use super::*;

        #[rstest]
        #[case::empty(PathValidationError::EmptyPath, "empty")]
        #[case::traversal(PathValidationError::PathTraversalError, "traversal")]
        #[case::symlink(PathValidationError::SymlinkEscapeError, "symlink")]
        fn message_contains_keyword(
            #[case] error: PathValidationError,
            #[case] keyword: &str,
        ) {
            let msg = format!("{error}").to_lowercase();
            assert!(msg.contains(keyword), "Expected '{keyword}' in: {msg}");
        }

        #[test]
        fn restricted_path_includes_path() {
            let error = PathValidationError::RestrictedPathError(
                PathBuf::from(".git/config"),
            );
            assert!(format!("{error}").contains(".git/config"));
        }

        #[test]
        fn invalid_extension_includes_required() {
            let error = PathValidationError::InvalidExtension {
                path: PathBuf::from("note.txt"),
                required: "md".into(),
            };
            let msg = format!("{error}");
            assert!(msg.contains("note.txt") && msg.contains(".md"));
        }

        #[test]
        fn relative_root_includes_path() {
            let error = PathValidationError::RelativeRoot(PathBuf::from(
                "relative/path",
            ));
            let msg = format!("{error}");
            assert!(msg.contains("relative/path") && msg.contains("absolute"));
        }
    }
}
