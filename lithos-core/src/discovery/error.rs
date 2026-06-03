use std::{io, path::PathBuf};

#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum VaultDiscoveryWarning {
    EmptyCeilingSegment,
    InvalidCeilingSegment {
        segment: PathBuf,
    },
}

#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, thiserror::Error)]
pub(crate) enum DiscoveryError {
    #[error("Explicit vault path does not exist: {path}")]
    ExplicitPathMissing {
        path: PathBuf,
    },
    #[error("Explicit vault path is not a directory: {path}")]
    ExplicitPathNotDirectory {
        path: PathBuf,
    },
    #[error("Environment vault path does not exist: {path}")]
    EnvironmentPathMissing {
        path: PathBuf,
    },
    #[error("Environment vault path is not a directory: {path}")]
    EnvironmentPathNotDirectory {
        path: PathBuf,
    },
    #[error("Failed to canonicalize current directory {path}: {source}")]
    CurrentDirectoryCanonicalize {
        path: PathBuf,
        source: io::Error,
    },
    #[error("Failed to canonicalize path {path}: {source}")]
    CanonicalizePath {
        path: PathBuf,
        source: io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    mod vault_discovery_warning {
        use super::*;

        #[test]
        fn empty_segment_is_constructible() {
            let w = VaultDiscoveryWarning::EmptyCeilingSegment;
            assert_eq!(w, VaultDiscoveryWarning::EmptyCeilingSegment);
        }

        #[test]
        fn invalid_segment_holds_path() {
            let w = VaultDiscoveryWarning::InvalidCeilingSegment {
                segment: PathBuf::from("/invalid"),
            };
            assert!(matches!(
                w,
                VaultDiscoveryWarning::InvalidCeilingSegment { .. }
            ));
        }
    }

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
    }
}
