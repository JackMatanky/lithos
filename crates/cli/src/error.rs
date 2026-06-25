// miette's `#[diagnostic(help(...))]` derive generates field bindings
// that `unused_assignments` flags as never read. Every variant with
// `#[diagnostic(help)]` is affected; this is a known miette behavior.
#![allow(
    unused_assignments,
    reason = "miette derive generates field bindings without reads"
)]
//! CLI error types with exit code derivation.
//!
//! This module defines [`CliError`], the top-level error type for the Traces
//! CLI binary. It wraps [`AppError`] and owns the mapping from error
//! variants to POSIX exit codes.
//!
//! Exit code conventions:
//! - `1` — vault not found (no anchor directory)
//! - `2` — invalid explicit path or configuration error (user error)
//! - `3` — filesystem permission denied or unreadable directory (I/O error)

use std::path::PathBuf;

use trace_app::error::AppError;
use trace_fs::{PathError, error::RootScopeError};
use trace_indexer::{IndexerError, IndexerRepositoryError, ScannerError};
use trace_settings::DiscoveryError;

/// Top-level CLI error that wraps the bootstrap pipeline error.
///
/// `CliError` is the outermost error type returned from CLI command handlers.
/// It derives [`miette::Diagnostic`] so that `miette` can render rich
/// diagnostics when `main()` returns `Err(e.into())`.
///
/// Exit codes are derived via [`CliError::exit_code`] and applied in `main()`
/// (Slice 8). `std::process::exit` is forbidden by the workspace lint
/// (`clippy::exit = "deny"`); the runner in Slice 8 handles the actual exit.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(crate) enum CliError {
    /// Application pipeline failed (discovery, config, or indexing error).
    #[error(transparent)]
    Bootstrap(#[from] AppError),

    /// Writing to stdout or stderr failed.
    #[error("failed to write to {stream}")]
    Write {
        /// The stream that failed (`"stdout"` or `"stderr"`).
        stream: &'static str,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// An invalid explicit path was provided.
    #[error("invalid path: {0}")]
    InvalidPath(String),

    /// Error during the index operation.
    #[error(transparent)]
    Index(#[from] IndexCommandError),
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(crate) enum IndexCommandError {
    /// The scan path or file does not exist on disk.
    ///
    /// Triggered when the scanner encounters a `NotFound` I/O error or when a
    /// `PathError` indicates the path is invalid. The user should provide a
    /// valid path or omit `--path` to scan the entire vault.
    #[error("{} does not exist", path.display())]
    #[diagnostic(help(
        "Provide a valid path, or omit --path to index the entire vault"
    ))]
    ScanPathNotFound {
        path: PathBuf,
    },

    /// The scan path exists but is not readable.
    ///
    /// Triggered when the scanner encounters a `PermissionDenied` I/O error.
    /// The user should grant read permission to the path.
    #[error("cannot read {}: permission denied", path.display())]
    #[diagnostic(help("Grant read permission: chmod +r {}", path.display()))]
    ScanAccessDenied {
        path: PathBuf,
    },

    /// The index database encountered a storage-level failure.
    ///
    /// Triggered by [`IndexerRepositoryError::Storage`] or
    /// [`IndexerRepositoryError::DuplicatePath`]. The user should rebuild
    /// the database with `--rebuild`.
    #[error("index database error: {detail}")]
    #[diagnostic(help(
        "Run `traces index --rebuild` to recreate the database"
    ))]
    StorageFailure {
        detail: String,
    },

    /// An I/O error occurred while scanning a filesystem path.
    ///
    /// Triggered by scanner traversal errors other than `NotFound` or
    /// `PermissionDenied` (e.g. disk errors, connection resets). The user
    /// should check disk health and retry.
    #[error("I/O error reading {}: {detail}", path.display())]
    #[diagnostic(help("Check disk space and filesystem health, then retry"))]
    ScanIoError {
        path: PathBuf,
        detail: String,
    },
}

impl From<IndexerError> for IndexCommandError {
    fn from(err: IndexerError) -> Self {
        match err {
            IndexerError::Path(e) => {
                let path = match e {
                    // ponytail: PathError::Empty intentionally falls through
                    // to the wildcard arm
                    PathError::NotAFile(p)
                    | PathError::NotADirectory(p)
                    | PathError::NotRelative(p)
                    | PathError::NotAbsolute(p)
                    | PathError::ParentTraversal(p)
                    | PathError::CurrentDirComponent(p)
                    | PathError::PlatformPrefix(p)
                    | PathError::InvalidUtf8(p)
                    | PathError::NoFileName(p)
                    | PathError::NoStem(p) => p,
                    PathError::RootScope(
                        RootScopeError::PathOutsideVaultRootBoundary {
                            path,
                            ..
                        },
                    ) => path,
                    // ponytail: PathError is #[non_exhaustive]; unknown
                    // future variants default to empty path
                    _ => PathBuf::new(),
                };
                IndexCommandError::ScanPathNotFound {
                    path,
                }
            }
            IndexerError::Scanner(ScannerError::Traversal {
                path,
                source,
            }) => match source.kind() {
                std::io::ErrorKind::NotFound => {
                    IndexCommandError::ScanPathNotFound {
                        path,
                    }
                }
                std::io::ErrorKind::PermissionDenied => {
                    IndexCommandError::ScanAccessDenied {
                        path,
                    }
                }
                _ => IndexCommandError::ScanIoError {
                    path,
                    detail: source.to_string(),
                },
            },
            IndexerError::Repository(IndexerRepositoryError::Storage(e)) => {
                IndexCommandError::StorageFailure {
                    detail: e.to_string(),
                }
            }
            IndexerError::Repository(
                IndexerRepositoryError::DuplicatePath(p),
            ) => IndexCommandError::StorageFailure {
                detail: format!("duplicate path: {}", p.as_str()),
            },
            IndexerError::Repository(other) => {
                IndexCommandError::StorageFailure {
                    detail: other.to_string(),
                }
            }
            _ => IndexCommandError::StorageFailure {
                detail: err.to_string(),
            },
        }
    }
}

impl CliError {
    /// Returns the POSIX exit code appropriate for this error.
    ///
    /// | Code | Meaning                                             |
    /// |------|-----------------------------------------------------|
    /// | `1`  | Vault not found — no valid anchor directory         |
    /// | `2`  | Invalid explicit path or configuration error        |
    /// | `3`  | Filesystem permission denied or directory unreadable|
    ///
    /// # Exit code mapping
    ///
    /// - [`AppError::Discovery`] with
    ///   [`DiscoveryError::InvalidAnchorDirectory`] → 1
    /// - [`AppError::Discovery`] with [`DiscoveryError::Flag`] or
    ///   [`DiscoveryError::Env`] → 2
    /// - [`AppError::Discovery`] with [`DiscoveryError::CanonicalizePath`]
    ///   where `source.kind() == PermissionDenied` → 3
    /// - [`AppError::Config`] → 2
    /// - [`CliError::Write`] → 3 (I/O failure writing to stdout or stderr)
    /// - All other variants → 2
    pub(crate) fn exit_code(&self) -> i32 {
        match self {
            Self::Bootstrap(AppError::Discovery(discovery_err)) => {
                exit_code_for_discovery(discovery_err)
            }
            Self::Bootstrap(AppError::Config(_)) | Self::InvalidPath(_) => 2,
            Self::Bootstrap(AppError::Indexer(_))
            | Self::Write {
                ..
            } => 3,
            Self::Index(err) => match err {
                IndexCommandError::ScanPathNotFound {
                    ..
                }
                | IndexCommandError::StorageFailure {
                    ..
                } => 2,
                IndexCommandError::ScanAccessDenied {
                    ..
                }
                | IndexCommandError::ScanIoError {
                    ..
                } => 3,
            },
        }
    }
}

/// Derives an exit code from a [`DiscoveryError`].
#[expect(
    clippy::match_same_arms,
    reason = "Flag and Env arms are explicit per exit-code specification; the \
              wildcard catch-all also returning 2 is intentional"
)]
fn exit_code_for_discovery(err: &DiscoveryError) -> i32 {
    match err {
        DiscoveryError::InvalidAnchorDirectory {
            ..
        } => 1,
        DiscoveryError::Flag(_) | DiscoveryError::Env(_) => 2,
        DiscoveryError::CanonicalizePath {
            source,
            ..
        } if source.kind() == std::io::ErrorKind::PermissionDenied => 3,
        _ => 2,
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use trace_app::error::AppError;
    use trace_fs::PathError;
    use trace_settings::{
        DiscoveryError,
        discovery::error::{
            EnvironmentOverrideError, FlagOverrideError, ServiceConfigError,
        },
        error::ConfigError,
    };

    use super::{CliError, IndexCommandError};

    mod conversions {
        use super::*;

        #[test]
        fn wraps_discovery_error_in_bootstrap_variant() {
            let bootstrap_err =
                AppError::Discovery(DiscoveryError::InvalidAnchorDirectory {
                    path: PathBuf::from("/bad"),
                    source: PathError::NotADirectory(PathBuf::from("/bad")),
                });
            let cli_err = CliError::from(bootstrap_err);
            assert!(
                matches!(cli_err, CliError::Bootstrap(AppError::Discovery(_))),
                "expected CliError::Bootstrap(Discovery(..)), got: {cli_err:?}"
            );
        }
    }

    mod index_command_error {
        use trace_db::DbError;
        use trace_fs::error::RootScopeError;
        use trace_indexer::{
            IndexerError, IndexerRepositoryError, ScannerError,
        };

        use super::*;

        fn assert_path_maps_to_scan_path_not_found(
            path_error: PathError,
            expected_path: &str,
        ) {
            let indexer_err = IndexerError::Path(path_error);
            let cmd_err: IndexCommandError = indexer_err.into();
            let expected = PathBuf::from(expected_path);
            assert!(
                matches!(&cmd_err, IndexCommandError::ScanPathNotFound { path } if *path == expected),
                "expected ScanPathNotFound with path={expected_path:?}, got \
                 {cmd_err:?}"
            );
        }

        #[test]
        fn extracts_actual_path_from_not_a_directory() {
            assert_path_maps_to_scan_path_not_found(
                PathError::NotADirectory(PathBuf::from("/actual/path")),
                "/actual/path",
            );
        }

        #[test]
        fn extracts_actual_path_from_not_a_file() {
            assert_path_maps_to_scan_path_not_found(
                PathError::NotAFile(PathBuf::from("/a/file")),
                "/a/file",
            );
        }

        #[test]
        fn extracts_actual_path_from_not_relative() {
            assert_path_maps_to_scan_path_not_found(
                PathError::NotRelative(PathBuf::from("/abs/path")),
                "/abs/path",
            );
        }

        #[test]
        fn extracts_actual_path_from_root_scope_error() {
            let root_err = RootScopeError::PathOutsideVaultRootBoundary {
                root: PathBuf::from("/vault"),
                path: PathBuf::from("/outside/file"),
            };
            let path_error = PathError::RootScope(root_err);
            let indexer_err = IndexerError::Path(path_error);
            let cmd_err: IndexCommandError = indexer_err.into();
            let expected = PathBuf::from("/outside/file");
            assert!(
                matches!(&cmd_err, IndexCommandError::ScanPathNotFound { path } if *path == expected),
                "expected ScanPathNotFound with path=/outside/file, got \
                 {cmd_err:?}"
            );
        }

        #[test]
        fn maps_not_found_traversal_to_scan_path_not_found() {
            let io_err = std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            );
            let scanner_err = ScannerError::Traversal {
                path: PathBuf::from("/missing"),
                source: io_err,
            };
            let indexer_err: IndexerError = scanner_err.into();
            let cmd_err: IndexCommandError = indexer_err.into();
            let expected = PathBuf::from("/missing");
            assert!(
                matches!(&cmd_err, IndexCommandError::ScanPathNotFound { path } if *path == expected),
                "expected ScanPathNotFound with path=/missing, got {cmd_err:?}"
            );
        }

        #[test]
        fn maps_permission_denied_traversal_to_scan_access_denied() {
            let io_err = std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "access denied",
            );
            let scanner_err = ScannerError::Traversal {
                path: PathBuf::from("/restricted"),
                source: io_err,
            };
            let indexer_err: IndexerError = scanner_err.into();
            let cmd_err: IndexCommandError = indexer_err.into();
            let expected = PathBuf::from("/restricted");
            assert!(
                matches!(&cmd_err, IndexCommandError::ScanAccessDenied { path } if *path == expected),
                "expected ScanAccessDenied with path=/restricted, got \
                 {cmd_err:?}"
            );
        }

        #[test]
        fn maps_other_traversal_to_scan_io_error() {
            let io_err = std::io::Error::new(
                std::io::ErrorKind::ConnectionReset,
                "connection reset",
            );
            let scanner_err = ScannerError::Traversal {
                path: PathBuf::from("/broken"),
                source: io_err,
            };
            let indexer_err: IndexerError = scanner_err.into();
            let cmd_err: IndexCommandError = indexer_err.into();
            let expected = PathBuf::from("/broken");
            assert!(
                matches!(&cmd_err, IndexCommandError::ScanIoError { path, .. } if *path == expected),
                "expected ScanIoError with path=/broken, got {cmd_err:?}"
            );
        }

        #[test]
        fn maps_storage_error_from_repository() {
            let db_err = DbError::Serialization("corrupt data".into());
            let repo_err: IndexerRepositoryError = db_err.into();
            let indexer_err: IndexerError = repo_err.into();
            let cmd_err: IndexCommandError = indexer_err.into();
            assert!(
                matches!(&cmd_err, IndexCommandError::StorageFailure { detail } if detail.contains("corrupt")),
                "expected StorageFailure with detail containing 'corrupt', \
                 got {cmd_err:?}"
            );
        }

        #[test]
        fn maps_duplicate_path_from_repository() {
            let path_key = trace_fs::PathKey::try_new("dup").unwrap();
            let repo_err = IndexerRepositoryError::DuplicatePath(path_key);
            let indexer_err: IndexerError = repo_err.into();
            let cmd_err: IndexCommandError = indexer_err.into();
            assert!(
                matches!(&cmd_err, IndexCommandError::StorageFailure { detail } if detail.contains("duplicate")),
                "expected StorageFailure with 'duplicate', got {cmd_err:?}"
            );
        }
    }

    mod display_and_diagnostic {
        use miette::Diagnostic;

        use super::*;

        fn help_text(err: &IndexCommandError) -> String {
            err.help().map(|h| h.to_string()).unwrap_or_default()
        }

        #[test]
        fn scan_path_not_found_display_and_help() {
            let err = IndexCommandError::ScanPathNotFound {
                path: PathBuf::from("/a/b"),
            };
            assert_eq!(err.to_string(), "/a/b does not exist");
            assert_eq!(
                help_text(&err),
                "Provide a valid path, or omit --path to index the entire \
                 vault"
            );
        }

        #[test]
        fn scan_access_denied_display_and_help() {
            let err = IndexCommandError::ScanAccessDenied {
                path: PathBuf::from("/x"),
            };
            assert_eq!(err.to_string(), "cannot read /x: permission denied");
            assert!(
                help_text(&err).starts_with("Grant read permission"),
                "help text: {}",
                help_text(&err)
            );
        }

        #[test]
        fn storage_failure_display_and_help() {
            let err = IndexCommandError::StorageFailure {
                detail: "corrupt data".to_owned(),
            };
            assert_eq!(err.to_string(), "index database error: corrupt data");
            assert_eq!(
                help_text(&err),
                "Run `traces index --rebuild` to recreate the database"
            );
        }

        #[test]
        fn scan_io_error_display_and_help() {
            let err = IndexCommandError::ScanIoError {
                path: PathBuf::from("/y"),
                detail: "disk error".to_owned(),
            };
            assert_eq!(err.to_string(), "I/O error reading /y: disk error");
            assert_eq!(
                help_text(&err),
                "Check disk space and filesystem health, then retry"
            );
        }
    }

    mod exit_code {
        use super::*;

        #[test]
        fn returns_1_when_vault_not_found() {
            let err = CliError::Bootstrap(AppError::Discovery(
                DiscoveryError::InvalidAnchorDirectory {
                    path: PathBuf::from("/no/vault"),
                    source: PathError::NotADirectory(PathBuf::from(
                        "/no/vault",
                    )),
                },
            ));
            assert_eq!(
                err.exit_code(),
                1,
                "InvalidAnchorDirectory must map to exit code 1"
            );
        }

        #[test]
        fn returns_2_when_explicit_path_is_invalid() {
            let err = CliError::Bootstrap(AppError::Discovery(
                DiscoveryError::Flag(FlagOverrideError::VaultPathNotFound {
                    path: PathBuf::from("/explicit/missing"),
                }),
            ));
            assert_eq!(
                err.exit_code(),
                2,
                "Flag error must map to exit code 2"
            );
        }

        #[test]
        fn returns_2_when_env_override_is_invalid() {
            let err =
                CliError::Bootstrap(AppError::Discovery(DiscoveryError::Env(
                    EnvironmentOverrideError::VaultPathNotFound {
                        path: PathBuf::from("/env/missing"),
                    },
                )));
            assert_eq!(err.exit_code(), 2, "Env error must map to exit code 2");
        }

        #[test]
        fn returns_2_when_config_error() {
            let err = CliError::Bootstrap(AppError::Config(
                ConfigError::Ingestion("bad toml".into()),
            ));
            assert_eq!(
                err.exit_code(),
                2,
                "Config error must map to exit code 2"
            );
        }

        #[test]
        fn returns_2_when_service_config_error() {
            let err = CliError::Bootstrap(AppError::Discovery(
                DiscoveryError::Config(ServiceConfigError::VaultMarkerPatterns),
            ));
            assert_eq!(
                err.exit_code(),
                2,
                "ServiceConfigError (catch-all) must map to exit code 2"
            );
        }

        #[test]
        fn returns_3_when_write_fails() {
            let source = std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "broken pipe",
            );
            let err = CliError::Write {
                stream: "stdout",
                source,
            };
            assert_eq!(
                err.exit_code(),
                3,
                "Write error must map to exit code 3"
            );
        }

        #[test]
        fn returns_2_when_canonicalize_path_is_not_found() {
            // CanonicalizePath with NotFound (not PermissionDenied) falls
            // through to catch-all → 2
            let source =
                std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
            let err = CliError::Bootstrap(AppError::Discovery(
                DiscoveryError::CanonicalizePath {
                    path: PathBuf::from("/missing/path"),
                    source,
                },
            ));
            assert_eq!(
                err.exit_code(),
                2,
                "CanonicalizePath with non-PermissionDenied must map to exit \
                 code 2"
            );
        }
    }
}
