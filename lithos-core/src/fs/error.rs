//! Error types for filesystem and parsing operations.
//!
//! These errors preserve contextual data (paths, line/column, or extensions)
//! so adapter layers can surface actionable diagnostics without leaking
//! low-level I/O details into domain logic.

/// Errors that can occur during config file parsing.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "Enum name reflects specific parsing context within the error \
              module"
)]
pub enum ParseError {
    /// An I/O error occurred while reading the file.
    #[error("Failed to read {path}: {source}")]
    Io {
        /// The path to the file that failed to be read.
        path: std::path::PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// A JSON parsing error occurred.
    #[error(
        "JSON error in {path} at line {line:?}, column {column:?}: {message}"
    )]
    Json {
        /// The path to the malformed JSON file.
        path: std::path::PathBuf,
        /// The error message from the JSON parser.
        message: Box<str>,
        /// The line number where the error occurred.
        line: Option<usize>,
        /// The column number where the error occurred.
        column: Option<usize>,
    },

    /// A TOML parsing error occurred.
    #[error(
        "TOML error in {path} at line {line:?}, column {column:?}: {message}"
    )]
    Toml {
        /// The path to the malformed TOML file.
        path: std::path::PathBuf,
        /// The error message from the TOML parser.
        message: Box<str>,
        /// The line number where the error occurred.
        line: Option<usize>,
        /// The column number where the error occurred.
        column: Option<usize>,
    },

    /// A YAML parsing error occurred.
    #[error(
        "YAML error in {path} at line {line:?}, column {column:?}: {message}"
    )]
    Yaml {
        /// The path to the malformed YAML file.
        path: std::path::PathBuf,
        /// The error message from the YAML parser.
        message: Box<str>,
        /// The line number where the error occurred.
        line: Option<usize>,
        /// The column number where the error occurred.
        column: Option<usize>,
    },

    /// The file format is not supported.
    #[error("Unsupported format for {path}. Supported: {supported:?}")]
    UnsupportedFormat {
        /// The path to the file with an unsupported format.
        path: std::path::PathBuf,
        /// A list of supported formats.
        supported: &'static [&'static str],
    },

    /// Path was not within the expected base directory.
    #[error("Path {path} is not within base directory {base}")]
    NotInBasePath {
        /// The path that was outside the base.
        path: std::path::PathBuf,
        /// The expected base directory.
        base: std::path::PathBuf,
    },
}

/// Errors related to path validation and vault safety.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "Enum name reflects specific validation context within the error \
              module"
)]
pub enum PathValidationError {
    /// An I/O error occurred during path validation.
    #[error("I/O error during validation: {0}")]
    IoError(#[from] std::io::Error),

    /// The path is empty.
    #[error("Path is empty")]
    EmptyPath,

    /// The path is absolute, which is not allowed for vault paths.
    #[error("Vault path must be relative, got: {0}")]
    AbsolutePathError(std::path::PathBuf),

    /// The path contains traversal components (e.g., `..`).
    #[error("Path traversal detected in vault path")]
    PathTraversalError,

    /// A symlink escapes the vault root.
    #[error("Symlink escapes the vault root")]
    SymlinkEscapeError,

    /// The path is restricted (e.g., hidden files like `.git`).
    #[error("Path is restricted: {0}")]
    RestrictedPathError(std::path::PathBuf),

    /// The path does not have the required extension.
    #[error("Path {path} must have extension .{required}")]
    InvalidExtension {
        /// The path with the invalid extension.
        path: std::path::PathBuf,
        /// The required extension.
        required: String,
    },

    /// The path contains invalid UTF-8.
    #[error("Path contains invalid UTF-8: {0:?}")]
    InvalidPathEncoding(std::path::PathBuf),

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

/// Errors that can occur during [`DirEntry`](std::fs::DirEntry) conversions.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "DirEntry prefix clarifies this is for std::fs::DirEntry \
              conversions"
)]
pub enum DirEntryError {
    /// Invalid UTF-8 in path.
    #[error("Invalid UTF-8 in path: {0}")]
    InvalidUtf8(String),

    /// I/O error during conversion.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
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
        fn formats_io_error_with_path() {
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
        fn formats_unsupported_format_error_with_list() {
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
        fn formats_message_containing_expected_keyword(
            #[case] error: PathValidationError,
            #[case] keyword: &str,
        ) {
            let msg = format!("{error}").to_lowercase();
            assert!(msg.contains(keyword), "Expected '{keyword}' in: {msg}");
        }

        #[test]
        fn formats_restricted_path_error_with_path() {
            let error = PathValidationError::RestrictedPathError(
                PathBuf::from(".git/config"),
            );
            assert!(format!("{error}").contains(".git/config"));
        }

        #[test]
        fn formats_invalid_extension_error_with_required() {
            let error = PathValidationError::InvalidExtension {
                path: PathBuf::from("note.txt"),
                required: "md".into(),
            };
            let msg = format!("{error}");
            assert!(msg.contains("note.txt") && msg.contains(".md"));
        }

        #[test]
        fn formats_relative_root_error_with_path() {
            let error = PathValidationError::RelativeRoot(PathBuf::from(
                "relative/path",
            ));
            let msg = format!("{error}");
            assert!(msg.contains("relative/path") && msg.contains("absolute"));
        }
    }
}
