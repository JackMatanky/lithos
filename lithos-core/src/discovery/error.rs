//! Error types for the discovery process.

use std::{io, path::PathBuf};

/// Fatal errors encountered during vault or global configuration discovery.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, thiserror::Error)]
pub(crate) enum DiscoveryError {
    /// A path provided via CLI flag does not exist on disk.
    #[error("Explicit vault path does not exist: {path}")]
    ExplicitPathMissing {
        /// The missing path.
        path: PathBuf,
    },
    /// A path provided via CLI flag exists but is not a directory.
    #[error("Explicit vault path is not a directory: {path}")]
    ExplicitPathNotDirectory {
        /// The non-directory path.
        path: PathBuf,
    },
    /// A path provided via environment variable does not exist on disk.
    #[error("Environment vault path does not exist: {path}")]
    EnvironmentPathMissing {
        /// The missing path.
        path: PathBuf,
    },
    /// A path provided via environment variable exists but is not a directory.
    #[error("Environment vault path is not a directory: {path}")]
    EnvironmentPathNotDirectory {
        /// The non-directory path.
        path: PathBuf,
    },
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
}

#[cfg(test)]
mod tests {
    use super::*;

    mod discovery_error {
        use super::*;

        #[test]
        fn returns_missing_explicit_path_error_message() {
            let err = DiscoveryError::ExplicitPathMissing {
                path: PathBuf::from("/nonexistent"),
            };
            assert_eq!(
                err.to_string(),
                "Explicit vault path does not exist: /nonexistent"
            );
        }

        #[test]
        fn returns_not_directory_explicit_path_error_message() {
            let err = DiscoveryError::ExplicitPathNotDirectory {
                path: PathBuf::from("/some/file"),
            };
            assert_eq!(
                err.to_string(),
                "Explicit vault path is not a directory: /some/file"
            );
        }

        #[test]
        fn returns_missing_environment_path_error_message() {
            let err = DiscoveryError::EnvironmentPathMissing {
                path: PathBuf::from("/nonexistent"),
            };
            assert_eq!(
                err.to_string(),
                "Environment vault path does not exist: /nonexistent"
            );
        }

        #[test]
        fn returns_not_directory_environment_path_error_message() {
            let err = DiscoveryError::EnvironmentPathNotDirectory {
                path: PathBuf::from("/some/file"),
            };
            assert_eq!(
                err.to_string(),
                "Environment vault path is not a directory: /some/file"
            );
        }

        #[test]
        fn returns_canonicalize_current_directory_error_message() {
            let err = DiscoveryError::CurrentDirectoryCanonicalize {
                path: PathBuf::from("/cwd"),
                source: io::Error::new(io::ErrorKind::NotFound, "not found"),
            };
            let msg = err.to_string();
            assert!(
                msg.starts_with(
                    "Failed to canonicalize current directory /cwd"
                )
            );
            assert!(msg.contains("not found"));
        }

        #[test]
        fn returns_canonicalize_path_error_message() {
            let err = DiscoveryError::CanonicalizePath {
                path: PathBuf::from("/some/path"),
                source: io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "denied",
                ),
            };
            let msg = err.to_string();
            assert!(msg.starts_with("Failed to canonicalize path /some/path"));
            assert!(msg.contains("denied"));
        }

        #[test]
        fn returns_read_directory_error_message() {
            let err = DiscoveryError::ReadDirectory {
                path: PathBuf::from("/some/dir"),
                source: io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "denied",
                ),
            };
            let msg = err.to_string();
            assert!(msg.starts_with("Failed to read directory /some/dir"));
            assert!(msg.contains("denied"));
        }
    }
}
