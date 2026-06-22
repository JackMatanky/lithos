//! CLI error types with exit code derivation.
//!
//! This module defines [`CliError`], the top-level error type for the Lithos
//! CLI binary. It wraps [`BootstrapError`] and owns the mapping from error
//! variants to POSIX exit codes.
//!
//! Exit code conventions:
//! - `1` — vault not found (no anchor directory)
//! - `2` — invalid explicit path or configuration error (user error)
//! - `3` — filesystem permission denied or unreadable directory (I/O error)

use trace_app::bootstrap::BootstrapError;
use trace_discovery::error::DiscoveryError;

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
    /// Bootstrap pipeline failed (discovery or config error).
    #[error(transparent)]
    Bootstrap(#[from] BootstrapError),

    /// Writing to stdout or stderr failed.
    #[error("failed to write to {stream}")]
    Write {
        /// The stream that failed (`"stdout"` or `"stderr"`).
        stream: &'static str,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
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
    /// - [`BootstrapError::Discovery`] with
    ///   [`DiscoveryError::InvalidAnchorDirectory`] → 1
    /// - [`BootstrapError::Discovery`] with [`DiscoveryError::Flag`] or
    ///   [`DiscoveryError::Env`] → 2
    /// - [`BootstrapError::Discovery`] with
    ///   [`DiscoveryError::CanonicalizePath`] where `source.kind() ==
    ///   PermissionDenied` → 3
    /// - [`BootstrapError::Config`] → 2
    /// - [`CliError::Write`] → 3 (I/O failure writing to stdout or stderr)
    /// - All other variants → 2
    pub(crate) fn exit_code(&self) -> i32 {
        match self {
            Self::Bootstrap(BootstrapError::Discovery(discovery_err)) => {
                exit_code_for_discovery(discovery_err)
            }
            Self::Bootstrap(BootstrapError::Config(_)) => 2,
            Self::Write {
                ..
            } => 3,
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use trace_app::bootstrap::BootstrapError;
    use trace_config::error::ConfigError;
    use trace_discovery::error::{
        DiscoveryError, EnvironmentOverrideError, FlagOverrideError,
        ServiceConfigError,
    };
    use trace_fs::PathError;

    use super::CliError;

    mod conversions {
        use super::*;

        #[test]
        fn converts_bootstrap_error_to_cli_error() {
            let bootstrap_err = BootstrapError::Discovery(
                DiscoveryError::InvalidAnchorDirectory {
                    path: PathBuf::from("/bad"),
                    source: PathError::NotADirectory(PathBuf::from("/bad")),
                },
            );
            let cli_err = CliError::from(bootstrap_err);
            assert!(
                matches!(
                    cli_err,
                    CliError::Bootstrap(BootstrapError::Discovery(_))
                ),
                "expected CliError::Bootstrap(Discovery(..)), got: {cli_err:?}"
            );
        }
    }

    mod exit_code {
        use super::*;

        #[test]
        fn returns_1_when_vault_not_found() {
            let err = CliError::Bootstrap(BootstrapError::Discovery(
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
            let err = CliError::Bootstrap(BootstrapError::Discovery(
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
            let err = CliError::Bootstrap(BootstrapError::Discovery(
                DiscoveryError::Env(
                    EnvironmentOverrideError::VaultPathNotFound {
                        path: PathBuf::from("/env/missing"),
                    },
                ),
            ));
            assert_eq!(err.exit_code(), 2, "Env error must map to exit code 2");
        }

        #[test]
        fn returns_2_when_config_error() {
            let err = CliError::Bootstrap(BootstrapError::Config(
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
            let err = CliError::Bootstrap(BootstrapError::Discovery(
                DiscoveryError::Config(ServiceConfigError::VaultMarkerPatterns),
            ));
            assert_eq!(
                err.exit_code(),
                2,
                "ServiceConfigError (catch-all) must map to exit code 2"
            );
        }

        #[test]
        fn returns_3_when_permission_denied() {
            let source = std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "permission denied",
            );
            let err = CliError::Bootstrap(BootstrapError::Discovery(
                DiscoveryError::CanonicalizePath {
                    path: PathBuf::from("/restricted/path"),
                    source,
                },
            ));
            assert_eq!(
                err.exit_code(),
                3,
                "CanonicalizePath with PermissionDenied must map to exit code \
                 3"
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
            let err = CliError::Bootstrap(BootstrapError::Discovery(
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
