//! Error types for filesystem and parsing operations.
//!
//! These errors preserve contextual data (paths, line/column, or extensions)
//! so adapter layers can surface actionable diagnostics without leaking
//! low-level I/O details into domain logic.

use std::path::PathBuf;

/// Module-level public error that composes all filesystem operation errors.
///
/// This is the primary error type returned by `FsReader` methods. It acts as a
/// pure compositor with no direct variants - all errors flow through child
/// types.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FsError {
    /// File read or metadata access error.
    #[error(transparent)]
    Read(#[from] ReadError),

    /// Directory traversal error.
    #[error(transparent)]
    Scan(#[from] ScanError),

    /// Structured file deserialization error.
    #[error(transparent)]
    Parse(#[from] ParseError),

    /// Path construction error.
    #[error(transparent)]
    Path(#[from] PathError),

    /// Path validation error (security checks).
    #[error(transparent)]
    Validation(#[from] PathValidationError),
}

/// Errors related to path construction and name extraction.
///
/// These errors occur when validating or constructing path types (`FilePath`,
/// `DirPath`, `FileName`, `BaseName`) or converting from raw path strings.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "PathError clarifies this is path-specific within the error \
              module"
)]
pub enum PathError {
    /// The path string is empty.
    #[error("Path is empty")]
    Empty,

    /// Expected a file, but path does not refer to one.
    #[error("Path does not refer to a file: {0}")]
    NotAFile(PathBuf),

    /// Expected a directory, but path does not refer to one.
    #[error("Path does not refer to a directory: {0}")]
    NotADirectory(PathBuf),

    /// Expected a relative path, but got an absolute path.
    #[error("Expected relative path, got absolute: {0}")]
    NotRelative(PathBuf),

    /// Expected an absolute path, but got a relative path.
    #[error("Expected absolute path, got relative: {0}")]
    NotAbsolute(PathBuf),

    /// Path contains parent traversal component (`..`).
    #[error("Path contains parent traversal (..): {0}")]
    ParentTraversal(PathBuf),

    /// Path contains current directory component (`.`).
    #[error("Path contains current directory component (.): {0}")]
    CurrentDirComponent(PathBuf),

    /// Path contains platform-specific prefix (e.g., `C:` on Windows).
    #[error("Path contains platform prefix: {0}")]
    PlatformPrefix(PathBuf),

    /// Path is not valid UTF-8.
    #[error("Path contains invalid UTF-8: {0:?}")]
    InvalidUtf8(PathBuf),

    /// Path has no filename component.
    #[error("Path has no filename component: {0}")]
    NoFileName(PathBuf),

    /// Path has no stem (basename without extension).
    #[error("Path has no stem: {0}")]
    NoStem(PathBuf),
}

/// Errors related to file input access operations.
///
/// These errors occur when reading files, accessing metadata, or when paths
/// violate vault root boundaries.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "ReadError clarifies this is read-specific within the error \
              module"
)]
pub enum ReadError {
    /// An I/O error occurred while reading a file or accessing metadata.
    #[error("Failed to read {path}: {source}")]
    Io {
        /// The path that failed to be read.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// Path is not within the expected base directory (vault root boundary
    /// violation).
    #[error("Path {path} is not within base directory {base}")]
    NotInBase {
        /// The path that was outside the base.
        path: PathBuf,
        /// The expected base directory.
        base: PathBuf,
    },
}

/// Errors related to directory traversal operations.
///
/// These errors occur during walkdir traversal, glob pattern matching, or when
/// encountering unsupported filesystem entry types (sockets, fifos, etc.).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "ScanError clarifies this is scan-specific within the error \
              module"
)]
pub enum ScanError {
    /// A walkdir entry or metadata read failed during traversal.
    #[error("Traversal failed for {path}: {source}")]
    Traversal {
        /// The path where traversal failed.
        path: PathBuf,
        /// The underlying I/O error (preserved, not erased).
        source: std::io::Error,
    },

    /// A glob pattern is syntactically invalid.
    #[error("Invalid glob pattern '{pattern}': {message}")]
    InvalidPattern {
        /// The invalid glob pattern.
        pattern: Box<str>,
        /// The error message from the glob parser.
        message: Box<str>,
    },

    /// Filesystem entry is neither file nor directory (socket, fifo, etc.).
    #[error("Unsupported entry type: {0}")]
    UnsupportedEntryType(PathBuf),

    /// Path construction failed during scan.
    #[error(transparent)]
    Path(#[from] PathError),
}

/// Errors that can occur during structured file deserialization.
///
/// This error type is now narrowed to only deserialization concerns (JSON,
/// TOML, YAML parsing). File I/O errors are handled by `ReadError` and path
/// errors by `PathError`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "Enum name reflects specific parsing context within the error \
              module"
)]
pub enum ParseError {
    /// A JSON parsing error occurred.
    #[error(
        "JSON error in {path} at line {line:?}, column {column:?}: {message}"
    )]
    Json {
        /// The path to the malformed JSON file.
        path: PathBuf,
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
        path: PathBuf,
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
        path: PathBuf,
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
        path: PathBuf,
        /// A list of supported formats.
        supported: &'static [&'static str],
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
    AbsolutePathError(PathBuf),

    /// The path contains traversal components (e.g., `..`).
    #[error("Path traversal detected in vault path")]
    PathTraversalError,

    /// A symlink escapes the vault root.
    #[error("Symlink escapes the vault root")]
    SymlinkEscapeError,

    /// The path is restricted (e.g., hidden files like `.git`).
    #[error("Path is restricted: {0}")]
    RestrictedPathError(PathBuf),

    /// The path does not have the required extension.
    #[error("Path {path} must have extension .{required}")]
    InvalidExtension {
        /// The path with the invalid extension.
        path: PathBuf,
        /// The required extension.
        required: String,
    },

    /// The path contains invalid UTF-8.
    #[error("Path contains invalid UTF-8: {0:?}")]
    InvalidPathEncoding(PathBuf),

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
    RelativeRoot(PathBuf),
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
        #[case::yaml(ParseError::Yaml {
            path: PathBuf::from("data.yaml"),
            message: "parse error".into(),
            line: Some(5),
            column: Some(10),
        }, "data.yaml")]
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

    mod path_error {
        use super::*;

        mod display_messages {
            use super::*;

            #[test]
            fn formats_not_a_file_with_path() {
                let error = PathError::NotAFile(PathBuf::from("dir/subdir"));
                let msg = format!("{error}");
                assert!(msg.contains("dir/subdir"), "Expected path in: {msg}");
                assert!(
                    msg.to_lowercase().contains("file"),
                    "Expected 'file' in: {msg}"
                );
            }

            #[test]
            fn formats_not_a_directory_with_path() {
                let error = PathError::NotADirectory(PathBuf::from("file.txt"));
                let msg = format!("{error}");
                assert!(msg.contains("file.txt"), "Expected path in: {msg}");
                assert!(
                    msg.to_lowercase().contains("directory"),
                    "Expected 'directory' in: {msg}"
                );
            }

            #[test]
            fn formats_not_relative_with_path() {
                let error =
                    PathError::NotRelative(PathBuf::from("/absolute/path"));
                let msg = format!("{error}");
                assert!(
                    msg.contains("/absolute/path"),
                    "Expected path in: {msg}"
                );
                assert!(
                    msg.to_lowercase().contains("relative"),
                    "Expected 'relative' in: {msg}"
                );
            }

            #[test]
            fn formats_not_absolute_with_path() {
                let error =
                    PathError::NotAbsolute(PathBuf::from("relative/path"));
                let msg = format!("{error}");
                assert!(
                    msg.contains("relative/path"),
                    "Expected path in: {msg}"
                );
                assert!(
                    msg.to_lowercase().contains("absolute"),
                    "Expected 'absolute' in: {msg}"
                );
            }

            #[test]
            fn formats_parent_traversal_with_path() {
                let error =
                    PathError::ParentTraversal(PathBuf::from("../parent"));
                let msg = format!("{error}");
                assert!(msg.contains("../parent"), "Expected path in: {msg}");
                assert!(msg.contains(".."), "Expected '..' in: {msg}");
            }

            #[test]
            fn formats_current_dir_component_with_path() {
                let error =
                    PathError::CurrentDirComponent(PathBuf::from("./current"));
                let msg = format!("{error}");
                assert!(msg.contains("./current"), "Expected path in: {msg}");
                assert!(msg.contains('.'), "Expected '.' reference in: {msg}");
            }

            #[test]
            fn formats_platform_prefix_with_path() {
                let error =
                    PathError::PlatformPrefix(PathBuf::from("C:/Windows"));
                let msg = format!("{error}");
                assert!(msg.contains("C:/Windows"), "Expected path in: {msg}");
                assert!(
                    msg.to_lowercase().contains("prefix"),
                    "Expected 'prefix' in: {msg}"
                );
            }

            #[test]
            fn formats_invalid_utf8_with_path() {
                let error = PathError::InvalidUtf8(PathBuf::from("path"));
                let msg = format!("{error}");
                assert!(msg.contains("path"), "Expected path in: {msg}");
                assert!(
                    msg.to_lowercase().contains("utf"),
                    "Expected 'utf' in: {msg}"
                );
            }

            #[test]
            fn formats_no_filename_with_path() {
                let error = PathError::NoFileName(PathBuf::from("/dir/"));
                let msg = format!("{error}");
                assert!(msg.contains("/dir/"), "Expected path in: {msg}");
                assert!(
                    msg.to_lowercase().contains("filename"),
                    "Expected 'filename' in: {msg}"
                );
            }

            #[test]
            fn formats_no_stem_with_path() {
                let error = PathError::NoStem(PathBuf::from(".hidden"));
                let msg = format!("{error}");
                assert!(msg.contains(".hidden"), "Expected path in: {msg}");
                assert!(
                    msg.to_lowercase().contains("stem"),
                    "Expected 'stem' in: {msg}"
                );
            }
        }
    }

    mod read_error {
        use super::*;

        mod display_messages {
            use super::*;

            #[test]
            fn formats_io_error_with_path_context() {
                let error = ReadError::Io {
                    path: PathBuf::from("vault/note.md"),
                    source: std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "not found",
                    ),
                };
                let msg = format!("{error}");
                assert!(
                    msg.contains("vault/note.md"),
                    "Expected path in: {msg}"
                );
                assert!(
                    msg.contains("not found"),
                    "Expected source error in: {msg}"
                );
            }

            #[test]
            fn formats_not_in_base_with_paths() {
                let error = ReadError::NotInBase {
                    path: PathBuf::from("/outside/vault"),
                    base: PathBuf::from("/vault/root"),
                };
                let msg = format!("{error}");
                assert!(
                    msg.contains("/outside/vault"),
                    "Expected path in: {msg}"
                );
                assert!(msg.contains("/vault/root"), "Expected base in: {msg}");
            }
        }
    }

    mod scan_error {
        use super::*;

        mod display_messages {
            use super::*;

            #[test]
            fn formats_traversal_with_path() {
                let error = ScanError::Traversal {
                    path: PathBuf::from("scan/dir"),
                    source: std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "permission denied",
                    ),
                };
                let msg = format!("{error}");
                assert!(msg.contains("scan/dir"), "Expected path in: {msg}");
                assert!(
                    msg.contains("permission denied"),
                    "Expected source in: {msg}"
                );
            }

            #[test]
            fn formats_invalid_pattern_with_pattern() {
                let error = ScanError::InvalidPattern {
                    pattern: "**[invalid".into(),
                    message: "unclosed bracket".into(),
                };
                let msg = format!("{error}");
                assert!(
                    msg.contains("**[invalid"),
                    "Expected pattern in: {msg}"
                );
                assert!(
                    msg.contains("unclosed bracket"),
                    "Expected message in: {msg}"
                );
            }

            #[test]
            fn formats_unsupported_entry_type_with_path() {
                let error =
                    ScanError::UnsupportedEntryType(PathBuf::from("/dev/null"));
                let msg = format!("{error}");
                assert!(msg.contains("/dev/null"), "Expected path in: {msg}");
            }

            #[test]
            fn formats_composed_path_error() {
                let path_error = PathError::Empty;
                let error = ScanError::Path(path_error);
                let msg = format!("{error}");
                assert!(
                    msg.to_lowercase().contains("empty"),
                    "Expected path error message in: {msg}"
                );
            }
        }

        mod conversions {
            use super::*;

            #[test]
            fn converts_from_path_error_automatically() {
                let path_error = PathError::NotAFile(PathBuf::from("file"));
                let _scan_error: ScanError = path_error.into();
                // Compilation success proves the #[from] conversion works
            }
        }

        mod source_preservation {
            use super::*;

            #[test]
            fn preserves_original_io_error_in_traversal() {
                let io_error = std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "original error",
                );
                let error = ScanError::Traversal {
                    path: PathBuf::from("test"),
                    source: io_error,
                };
                // Verify source is accessible through the public Error trait
                let source = std::error::Error::source(&error)
                    .expect("Traversal should expose original io::Error");
                let io_err = source
                    .downcast_ref::<std::io::Error>()
                    .expect("source should downcast to std::io::Error");
                assert_eq!(io_err.kind(), std::io::ErrorKind::NotFound);
                assert!(io_err.to_string().contains("original error"));
            }
        }
    }

    mod fs_error {
        use super::*;

        mod conversions {
            use super::*;

            #[test]
            fn converts_from_read_error_automatically() {
                let read_error = ReadError::Io {
                    path: PathBuf::from("file.txt"),
                    source: std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "not found",
                    ),
                };
                let _fs_error: FsError = read_error.into();
            }

            #[test]
            fn converts_from_scan_error_automatically() {
                let scan_error =
                    ScanError::UnsupportedEntryType(PathBuf::from("/dev/null"));
                let _fs_error: FsError = scan_error.into();
            }

            #[test]
            fn converts_from_parse_error_automatically() {
                let parse_error = ParseError::Json {
                    path: PathBuf::from("data.json"),
                    message: "error".into(),
                    line: None,
                    column: None,
                };
                let _fs_error: FsError = parse_error.into();
            }

            #[test]
            fn converts_from_path_error_automatically() {
                let path_error = PathError::Empty;
                let _fs_error: FsError = path_error.into();
            }

            #[test]
            fn converts_from_validation_error_automatically() {
                let validation_error = PathValidationError::EmptyPath;
                let _fs_error: FsError = validation_error.into();
            }
        }
    }
}
