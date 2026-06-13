//! Error types for the discovery process.
//!
//! This module defines the [`DiscoveryError`] enum, which consolidates all
//! fatal failure conditions that can occur during vault or global configuration
//! discovery.
//!
//! # Error Classification
//!
//! - **Flag Errors**: Missing or invalid paths provided via explicit CLI flags.
//! - **Environment Errors**: Missing or invalid paths provided via environment
//!   variables.
//! - **Filesystem Errors**: Issues canonicalizing paths, reading directories,
//!   or permission failures.
//! - **Current Directory Errors**: Failures when establishing the starting
//!   point for ascending discovery.

use std::{io, path::PathBuf};

use crate::fs::PathError;

/// Fatal errors encountered during vault or global configuration discovery.
///
/// These errors typically indicate a configuration error (e.g., pointing to a
/// missing directory) or a system-level issue (e.g., permission denied).
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, thiserror::Error)]
pub(crate) enum DiscoveryError {
    /// Fatal error while validating explicit CLI flag overrides.
    #[error(transparent)]
    Flag(#[from] FlagOverrideError),
    /// Fatal error while validating environment variable overrides.
    #[error(transparent)]
    Env(#[from] EnvironmentOverrideError),
    /// Failed to canonicalize the current working directory.
    #[error("Failed to canonicalize current directory {path}: {source}")]
    CurrentDirectoryCanonicalize {
        /// The path that failed canonicalization.
        path: PathBuf,
        /// The underlying I/O error.
        source: io::Error,
    },
    /// Failed to canonicalize a specific path during discovery.
    #[error("Failed to canonicalize path {path}: {source}")]
    CanonicalizePath {
        /// The path that failed canonicalization.
        path: PathBuf,
        /// The underlying I/O error.
        source: io::Error,
    },
    /// Failed to read a directory during discovery.
    #[error("Failed to read directory {path}: {source}")]
    ReadDirectory {
        /// The directory that could not be read.
        path: PathBuf,
        /// The underlying I/O error.
        source: io::Error,
    },
    /// Active invocation anchor is invalid and cannot seed discovery.
    #[error("Invalid active anchor directory: {path}")]
    InvalidAnchorDirectory {
        /// The invalid anchor path.
        path: PathBuf,
        /// The underlying filesystem path validation error.
        #[source]
        source: PathError,
    },
}

/// Fatal errors produced by explicit CLI flag override validation.
#[allow(
    dead_code,
    reason = "Contract slice; wired in once orchestration lands"
)]
#[derive(Debug, thiserror::Error)]
pub(crate) enum FlagOverrideError {
    /// Explicit config file override does not refer to a file.
    ///
    /// Covers both non-existent paths and paths that exist but are directories.
    #[error("Explicit config file is not a file: {path}")]
    GlobalConfigPathNotFile {
        /// The invalid config file path.
        path: PathBuf,
        /// The underlying filesystem path validation error.
        #[source]
        source: PathError,
    },
    /// Explicit vault directory override does not refer to a directory.
    ///
    /// Covers both non-existent paths and paths that exist but are files.
    #[error("Explicit vault path is not a directory: {path}")]
    VaultPathNotDirectory {
        /// The invalid vault directory path.
        path: PathBuf,
        /// The underlying filesystem path validation error.
        #[source]
        source: PathError,
    },
}

/// Fatal errors produced by environment variable override validation.
#[allow(
    dead_code,
    reason = "Contract slice; wired in once orchestration lands"
)]
#[derive(Debug, thiserror::Error)]
pub(crate) enum EnvironmentOverrideError {
    /// Config file path from environment does not refer to a file.
    #[error("Environment config file is not a file: {path}")]
    GlobalConfigPathNotFile {
        /// The invalid environment config path.
        path: PathBuf,
        /// The underlying filesystem path validation error.
        #[source]
        source: PathError,
    },
    /// Vault path from environment does not exist.
    #[error("Environment vault path does not exist: {path}")]
    VaultPathMissing {
        /// The missing path.
        path: PathBuf,
    },
    /// Vault path from environment exists but is not a directory.
    #[error("Environment vault path is not a directory: {path}")]
    VaultPathNotDirectory {
        /// The non-directory path.
        path: PathBuf,
    },
    /// Vault path from environment failed path validation for another reason.
    #[error("Environment vault path is invalid: {path}")]
    VaultPathInvalid {
        /// The invalid path.
        path: PathBuf,
        /// The underlying filesystem path validation error.
        #[source]
        source: PathError,
    },
}

impl EnvironmentOverrideError {
    /// Constructs the appropriate environment vault error from a [`PathError`].
    pub(crate) fn from_vault_path_error(
        path: PathBuf,
        source: PathError,
    ) -> Self {
        match source {
            PathError::Empty => Self::VaultPathMissing {
                path,
            },
            PathError::NotADirectory(_) => Self::VaultPathNotDirectory {
                path,
            },
            source => Self::VaultPathInvalid {
                path,
                source,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod flag_override {
        use super::*;

        mod formatting {
            use super::*;

            #[test]
            fn returns_global_config_path_not_file_message() {
                let err = FlagOverrideError::GlobalConfigPathNotFile {
                    path: PathBuf::from("/missing/lithos.toml"),
                    source: PathError::NotAFile(PathBuf::from(
                        "/missing/lithos.toml",
                    )),
                };
                assert_eq!(
                    err.to_string(),
                    "Explicit config file is not a file: /missing/lithos.toml"
                );
            }

            #[test]
            fn returns_vault_path_not_directory_message() {
                let err = FlagOverrideError::VaultPathNotDirectory {
                    path: PathBuf::from("/missing/vault"),
                    source: PathError::NotADirectory(PathBuf::from(
                        "/missing/vault",
                    )),
                };
                assert_eq!(
                    err.to_string(),
                    "Explicit vault path is not a directory: /missing/vault"
                );
            }
        }
    }

    mod environment_override {
        use super::*;

        mod from_vault_path_error {
            use super::*;

            #[test]
            fn returns_vault_path_missing_when_source_is_empty() {
                let err = EnvironmentOverrideError::from_vault_path_error(
                    PathBuf::from("/gone"),
                    PathError::Empty,
                );
                assert!(
                    matches!(
                        err,
                        EnvironmentOverrideError::VaultPathMissing { .. }
                    ),
                    "expected VaultPathMissing, got: {err:?}"
                );
            }

            #[test]
            fn returns_vault_path_not_directory_when_source_is_not_a_directory()
            {
                let err = EnvironmentOverrideError::from_vault_path_error(
                    PathBuf::from("/file"),
                    PathError::NotADirectory(PathBuf::from("/file")),
                );
                assert!(
                    matches!(
                        err,
                        EnvironmentOverrideError::VaultPathNotDirectory { .. }
                    ),
                    "expected VaultPathNotDirectory, got: {err:?}"
                );
            }

            #[test]
            fn returns_vault_path_invalid_for_other_path_errors() {
                let err = EnvironmentOverrideError::from_vault_path_error(
                    PathBuf::from("/bad"),
                    PathError::NotRelative(PathBuf::from("/bad")),
                );
                assert!(
                    matches!(
                        err,
                        EnvironmentOverrideError::VaultPathInvalid { .. }
                    ),
                    "expected VaultPathInvalid, got: {err:?}"
                );
            }
        }

        mod formatting {
            use super::*;

            #[test]
            fn returns_vault_path_missing_message() {
                let err = EnvironmentOverrideError::VaultPathMissing {
                    path: PathBuf::from("/nonexistent"),
                };
                assert_eq!(
                    err.to_string(),
                    "Environment vault path does not exist: /nonexistent"
                );
            }

            #[test]
            fn returns_vault_path_not_directory_message() {
                let err = EnvironmentOverrideError::VaultPathNotDirectory {
                    path: PathBuf::from("/some/file"),
                };
                assert_eq!(
                    err.to_string(),
                    "Environment vault path is not a directory: /some/file"
                );
            }
        }
    }

    mod discovery_error {
        use super::*;

        mod formatting {
            use super::*;

            #[test]
            fn returns_flag_variant_message_transparently() {
                let err = DiscoveryError::from(
                    FlagOverrideError::GlobalConfigPathNotFile {
                        path: PathBuf::from("/missing/lithos.toml"),
                        source: PathError::NotAFile(PathBuf::from(
                            "/missing/lithos.toml",
                        )),
                    },
                );
                assert_eq!(
                    err.to_string(),
                    "Explicit config file is not a file: /missing/lithos.toml"
                );
            }

            #[test]
            fn returns_env_variant_message_transparently() {
                let err = DiscoveryError::from(
                    EnvironmentOverrideError::VaultPathMissing {
                        path: PathBuf::from("/nonexistent"),
                    },
                );
                assert_eq!(
                    err.to_string(),
                    "Environment vault path does not exist: /nonexistent"
                );
            }

            #[test]
            fn returns_canonicalize_current_directory_message_with_path() {
                let err = DiscoveryError::CurrentDirectoryCanonicalize {
                    path: PathBuf::from("/cwd"),
                    source: io::Error::new(
                        io::ErrorKind::NotFound,
                        "not found",
                    ),
                };
                assert!(
                    err.to_string().starts_with(
                        "Failed to canonicalize current directory /cwd"
                    ),
                    "unexpected message: {err}"
                );
            }

            #[test]
            fn returns_canonicalize_current_directory_message_with_cause() {
                let err = DiscoveryError::CurrentDirectoryCanonicalize {
                    path: PathBuf::from("/cwd"),
                    source: io::Error::new(
                        io::ErrorKind::NotFound,
                        "not found",
                    ),
                };
                assert!(
                    err.to_string().contains("not found"),
                    "missing cause in: {err}"
                );
            }

            #[test]
            fn returns_canonicalize_path_message_with_path() {
                let err = DiscoveryError::CanonicalizePath {
                    path: PathBuf::from("/some/path"),
                    source: io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "denied",
                    ),
                };
                assert!(
                    err.to_string()
                        .starts_with("Failed to canonicalize path /some/path"),
                    "unexpected message: {err}"
                );
            }

            #[test]
            fn returns_canonicalize_path_message_with_cause() {
                let err = DiscoveryError::CanonicalizePath {
                    path: PathBuf::from("/some/path"),
                    source: io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "denied",
                    ),
                };
                assert!(
                    err.to_string().contains("denied"),
                    "missing cause in: {err}"
                );
            }

            #[test]
            fn returns_read_directory_message_with_path() {
                let err = DiscoveryError::ReadDirectory {
                    path: PathBuf::from("/some/dir"),
                    source: io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "denied",
                    ),
                };
                assert!(
                    err.to_string()
                        .starts_with("Failed to read directory /some/dir"),
                    "unexpected message: {err}"
                );
            }

            #[test]
            fn returns_read_directory_message_with_cause() {
                let err = DiscoveryError::ReadDirectory {
                    path: PathBuf::from("/some/dir"),
                    source: io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "denied",
                    ),
                };
                assert!(
                    err.to_string().contains("denied"),
                    "missing cause in: {err}"
                );
            }
        }
    }
}
