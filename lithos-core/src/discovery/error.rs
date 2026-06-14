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
    /// Invalid discovery service configuration.
    #[error(transparent)]
    Config(#[from] ServiceConfigError),
}

/// Errors produced by [`DiscoveryServiceConfig`] validation.
///
/// These errors occur at service construction time, before any discovery
/// execution begins.
///
/// [`DiscoveryServiceConfig`]: crate::discovery::service::DiscoveryServiceConfig
#[allow(
    dead_code,
    reason = "Contract slice; wired in once orchestration lands"
)]
#[expect(
    clippy::enum_variant_names,
    reason = "Variants are intentionally explicit — the error type describes \
              WHAT is empty (vault patterns, global patterns, boundary \
              markers)"
)]
#[derive(Debug, thiserror::Error)]
pub(crate) enum ServiceConfigError {
    /// Vault marker pattern list is empty.
    #[error("vault_marker_patterns must not be empty")]
    VaultMarkerPatterns,
    /// Global marker pattern list is empty.
    #[error("global_marker_patterns must not be empty")]
    GlobalMarkerPatterns,
    /// Boundary marker list is empty.
    #[error("boundary_markers must not be empty")]
    BoundaryMarkerPatterns,
}

/// Fatal errors produced by explicit CLI flag override validation.
#[allow(
    dead_code,
    reason = "Contract slice; wired in once orchestration lands"
)]
#[derive(Debug, thiserror::Error)]
pub(crate) enum FlagOverrideError {
    /// Explicit config file override path does not exist on the filesystem.
    #[error("Explicit config file path not found: {path}")]
    GlobalConfigPathNotFound {
        /// The path that was not found.
        path: PathBuf,
    },
    /// Explicit config file override exists but does not refer to a file.
    #[error("Explicit config file is not a file: {path}")]
    GlobalConfigPathNotFile {
        /// The invalid config file path.
        path: PathBuf,
        /// The underlying filesystem path validation error.
        #[source]
        source: PathError,
    },
    /// Explicit vault directory override path does not exist on the filesystem.
    #[error("Explicit vault path not found: {path}")]
    VaultPathNotFound {
        /// The path that was not found.
        path: PathBuf,
    },
    /// Explicit vault directory override exists but does not refer to a
    /// directory.
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
    /// Config file path from environment does not exist on the filesystem.
    #[error("Environment config file path not found: {path}")]
    GlobalConfigPathNotFound {
        /// The path that was not found.
        path: PathBuf,
    },
    /// Config file path from environment exists but does not refer to a file.
    #[error("Environment config file is not a file: {path}")]
    GlobalConfigPathNotFile {
        /// The invalid environment config path.
        path: PathBuf,
        /// The underlying filesystem path validation error.
        #[source]
        source: PathError,
    },
    /// Vault path from environment does not exist on the filesystem.
    #[error("Environment vault path not found: {path}")]
    VaultPathNotFound {
        /// The path that was not found.
        path: PathBuf,
    },
    /// Vault path from environment exists but is not a directory.
    #[error("Environment vault path is not a directory: {path}")]
    VaultPathNotDirectory {
        /// The non-directory path.
        path: PathBuf,
        /// The underlying filesystem path validation error.
        #[source]
        source: PathError,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    mod discovery_error {
        use super::*;

        mod formatting {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn returns_flag_variant_message_transparently() {
                let err = DiscoveryError::from(
                    FlagOverrideError::GlobalConfigPathNotFound {
                        path: PathBuf::from("/missing/lithos.toml"),
                    },
                );
                assert_eq!(
                    err.to_string(),
                    "Explicit config file path not found: /missing/lithos.toml"
                );
            }

            #[test]
            fn returns_env_variant_message_transparently() {
                let err = DiscoveryError::from(
                    EnvironmentOverrideError::VaultPathNotFound {
                        path: PathBuf::from("/nonexistent"),
                    },
                );
                assert_eq!(
                    err.to_string(),
                    "Environment vault path not found: /nonexistent"
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

    mod flag_override {
        use super::*;

        mod formatting {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn returns_global_config_path_not_found_message() {
                let err = FlagOverrideError::GlobalConfigPathNotFound {
                    path: PathBuf::from("/missing/lithos.toml"),
                };
                assert_eq!(
                    err.to_string(),
                    "Explicit config file path not found: /missing/lithos.toml"
                );
            }

            #[test]
            fn returns_global_config_path_not_file_message() {
                let err = FlagOverrideError::GlobalConfigPathNotFile {
                    path: PathBuf::from("/some/dir"),
                    source: PathError::NotAFile(PathBuf::from("/some/dir")),
                };
                assert_eq!(
                    err.to_string(),
                    "Explicit config file is not a file: /some/dir"
                );
            }

            #[test]
            fn returns_vault_path_not_found_message() {
                let err = FlagOverrideError::VaultPathNotFound {
                    path: PathBuf::from("/missing/vault"),
                };
                assert_eq!(
                    err.to_string(),
                    "Explicit vault path not found: /missing/vault"
                );
            }

            #[test]
            fn returns_vault_path_not_directory_message() {
                let err = FlagOverrideError::VaultPathNotDirectory {
                    path: PathBuf::from("/some/file"),
                    source: PathError::NotADirectory(PathBuf::from(
                        "/some/file",
                    )),
                };
                assert_eq!(
                    err.to_string(),
                    "Explicit vault path is not a directory: /some/file"
                );
            }
        }
    }

    mod environment_override {
        use super::*;

        mod formatting {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn returns_global_config_path_not_found_message() {
                let err = EnvironmentOverrideError::GlobalConfigPathNotFound {
                    path: PathBuf::from("/missing/lithos.toml"),
                };
                assert_eq!(
                    err.to_string(),
                    "Environment config file path not found: \
                     /missing/lithos.toml"
                );
            }

            #[test]
            fn returns_global_config_path_not_file_message() {
                let err = EnvironmentOverrideError::GlobalConfigPathNotFile {
                    path: PathBuf::from("/some/dir"),
                    source: PathError::NotAFile(PathBuf::from("/some/dir")),
                };
                assert_eq!(
                    err.to_string(),
                    "Environment config file is not a file: /some/dir"
                );
            }

            #[test]
            fn returns_vault_path_not_found_message() {
                let err = EnvironmentOverrideError::VaultPathNotFound {
                    path: PathBuf::from("/nonexistent"),
                };
                assert_eq!(
                    err.to_string(),
                    "Environment vault path not found: /nonexistent"
                );
            }

            #[test]
            fn returns_vault_path_not_directory_message() {
                let err = EnvironmentOverrideError::VaultPathNotDirectory {
                    path: PathBuf::from("/some/file"),
                    source: PathError::NotADirectory(PathBuf::from(
                        "/some/file",
                    )),
                };
                assert_eq!(
                    err.to_string(),
                    "Environment vault path is not a directory: /some/file"
                );
            }
        }
    }

    mod service_config {
        use super::*;

        mod formatting {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn returns_empty_vault_marker_patterns_message() {
                let err = DiscoveryError::Config(
                    ServiceConfigError::VaultMarkerPatterns,
                );
                assert_eq!(
                    err.to_string(),
                    "vault_marker_patterns must not be empty"
                );
            }

            #[test]
            fn returns_empty_global_marker_patterns_message() {
                let err = DiscoveryError::Config(
                    ServiceConfigError::GlobalMarkerPatterns,
                );
                assert_eq!(
                    err.to_string(),
                    "global_marker_patterns must not be empty"
                );
            }

            #[test]
            fn returns_empty_boundary_marker_patterns_message() {
                let err = DiscoveryError::Config(
                    ServiceConfigError::BoundaryMarkerPatterns,
                );
                assert_eq!(
                    err.to_string(),
                    "boundary_markers must not be empty"
                );
            }
        }
    }
}
