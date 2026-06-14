//! Bootstrap orchestration seams for runtime context acquisition.

use crate::discovery::{
    context::{DiscoveryContext, DiscoveryEnv, DiscoveryFlags},
    error::DiscoveryError,
    port::DiscoveryPort,
    service::DiscoveryResult,
};

/// Application-owned bootstrap orchestration entry point.
///
/// `Bootstrapper` is generic over `D: DiscoveryPort` so that the discovery
/// implementation can be swapped out in tests without touching the
/// orchestration logic.
#[derive(Debug, Default)]
#[allow(dead_code, reason = "Contract slice; full orchestration lands later")]
pub(crate) struct Bootstrapper<D: DiscoveryPort> {
    port: D,
}

#[allow(dead_code, reason = "Contract slice; full orchestration lands later")]
impl<D: DiscoveryPort> Bootstrapper<D> {
    /// Creates a bootstrapper backed by the given discovery port.
    pub(crate) fn new(port: D) -> Self {
        Self {
            port,
        }
    }

    /// Builds Discovery's input contract from app-owned runtime sources.
    ///
    /// `anchor` is the working directory and is always required. `flags` and
    /// `env` are optional overrides from the CLI and environment respectively;
    /// pass `None` for either when no user-supplied override is present.
    ///
    /// # Errors
    ///
    /// Returns [`DiscoveryError::InvalidAnchorDirectory`] if `anchor` does not
    /// resolve to an existing directory, or a flag/env validation error if the
    /// provided paths are invalid.
    pub(crate) fn build_context<'a>(
        flags: Option<DiscoveryFlags>,
        env: Option<DiscoveryEnv<'a>>,
        anchor: &std::path::Path,
    ) -> Result<DiscoveryContext<'a>, DiscoveryError> {
        let mut ctx = DiscoveryContext::new(anchor)?;
        if let Some(f) = flags {
            ctx = ctx.with_flags(f);
        }
        if let Some(e) = env {
            ctx = ctx.with_env(e);
        }
        Ok(ctx)
    }

    /// Runs discovery using the bootstrapper's port and the given context.
    ///
    /// # Errors
    ///
    /// Propagates any [`DiscoveryError`] returned by the port implementation.
    pub(crate) fn discover(
        &self,
        context: &DiscoveryContext<'_>,
    ) -> Result<DiscoveryResult, DiscoveryError> {
        self.port.discover(context)
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;
    use crate::{
        discovery::{
            context::DiscoveryContext, error::DiscoveryError,
            service::DiscoveryResult,
        },
        fs::{DirPath, FilePath, PathError},
    };

    // --- Mock port ---

    struct MockPort {
        result: Result<DiscoveryResult, DiscoveryError>,
    }

    impl MockPort {
        fn succeeding(result: DiscoveryResult) -> Self {
            Self {
                result: Ok(result),
            }
        }

        fn failing() -> Self {
            Self {
                result: Err(DiscoveryError::InvalidAnchorDirectory {
                    path: std::path::PathBuf::from("/bad"),
                    source: PathError::NotADirectory(std::path::PathBuf::from(
                        "/bad",
                    )),
                }),
            }
        }
    }

    impl DiscoveryPort for MockPort {
        fn discover(
            &self,
            _context: &DiscoveryContext<'_>,
        ) -> Result<DiscoveryResult, DiscoveryError> {
            match &self.result {
                Ok(r) => Ok(r.clone()),
                Err(_) => Err(DiscoveryError::InvalidAnchorDirectory {
                    path: std::path::PathBuf::from("/bad"),
                    source: PathError::NotADirectory(std::path::PathBuf::from(
                        "/bad",
                    )),
                }),
            }
        }
    }

    // --- Fixtures ---

    mod fixtures {
        use super::*;

        pub(super) struct BootstrapInputs {
            pub(super) cwd: tempfile::TempDir,
            pub(super) cli_vault: tempfile::TempDir,
            pub(super) env_vault: tempfile::TempDir,
            pub(super) cli_config: std::path::PathBuf,
            pub(super) env_config: std::path::PathBuf,
            pub(super) ceilings: &'static std::ffi::OsStr,
        }

        impl BootstrapInputs {
            pub(super) fn new() -> Self {
                let cwd = tempfile::tempdir().expect("cwd dir");
                let cli_vault = tempfile::tempdir().expect("cli vault dir");
                let env_vault = tempfile::tempdir().expect("env vault dir");
                let cli_config = cli_vault.path().join("lithos.toml");
                let env_config = env_vault.path().join("lithos.toml");
                std::fs::write(&cli_config, "").expect("write cli config");
                std::fs::write(&env_config, "").expect("write env config");
                let ceilings = OsStr::new("/work:/home");
                Self {
                    cwd,
                    cli_vault,
                    env_vault,
                    cli_config,
                    env_config,
                    ceilings,
                }
            }

            pub(super) fn build_context(
                &self,
            ) -> Result<DiscoveryContext<'_>, DiscoveryError> {
                let flags = DiscoveryFlags::new(
                    Some(self.cli_config.as_path()),
                    Some(self.cli_vault.path()),
                    true,
                )
                .expect("valid flags");
                let env = DiscoveryEnv::new(
                    Some(self.env_config.as_path()),
                    Some(self.env_vault.path()),
                    Some(self.ceilings),
                )
                .expect("valid env");
                Bootstrapper::<MockPort>::build_context(
                    Some(flags),
                    Some(env),
                    self.cwd.path(),
                )
            }
        }
    }

    // --- build_context tests ---

    mod build_context {
        use super::*;

        mod constructor {
            use super::*;

            #[test]
            fn returns_context_with_anchor_matching_cwd() {
                let inputs = fixtures::BootstrapInputs::new();
                let context = inputs.build_context().expect("valid context");
                assert_eq!(
                    context.anchor().as_path(),
                    inputs.cwd.path(),
                    "anchor should match the injected cwd"
                );
            }

            #[test]
            fn returns_context_with_flag_config_file() {
                let inputs = fixtures::BootstrapInputs::new();
                let context = inputs.build_context().expect("valid context");
                assert_eq!(
                    context.flags().config_file().map(FilePath::as_path),
                    Some(inputs.cli_config.as_path()),
                    "flag config_file should match the injected cli config \
                     path"
                );
            }

            #[test]
            fn returns_context_with_flag_vault_dir() {
                let inputs = fixtures::BootstrapInputs::new();
                let context = inputs.build_context().expect("valid context");
                assert_eq!(
                    context.flags().vault_dir().map(DirPath::as_path),
                    Some(inputs.cli_vault.path()),
                    "flag vault_dir should match the injected cli vault path"
                );
            }

            #[test]
            fn returns_context_with_suppress_global_set() {
                let inputs = fixtures::BootstrapInputs::new();
                let context = inputs.build_context().expect("valid context");
                assert!(
                    context.flags().suppress_global(),
                    "suppress_global should be set from the injected flags"
                );
            }

            #[test]
            fn returns_context_with_env_config_file() {
                let inputs = fixtures::BootstrapInputs::new();
                let context = inputs.build_context().expect("valid context");
                assert_eq!(
                    context.env().config_file().map(FilePath::as_path),
                    Some(inputs.env_config.as_path()),
                    "env config_file should match the injected env config path"
                );
            }

            #[test]
            fn returns_context_with_env_vault_dir() {
                let inputs = fixtures::BootstrapInputs::new();
                let context = inputs.build_context().expect("valid context");
                assert_eq!(
                    context.env().vault_dir().map(DirPath::as_path),
                    Some(inputs.env_vault.path()),
                    "env vault_dir should match the injected env vault path"
                );
            }

            #[test]
            fn returns_context_with_ceiling_dirs_raw() {
                let inputs = fixtures::BootstrapInputs::new();
                let context = inputs.build_context().expect("valid context");
                assert_eq!(
                    context.env().ceiling_dirs_raw(),
                    Some(inputs.ceilings),
                    "ceiling_dirs_raw should match the injected ceiling dirs"
                );
            }

            #[test]
            fn returns_context_with_default_flags_when_none_given() {
                let cwd = tempfile::tempdir().expect("cwd dir");
                let context = Bootstrapper::<MockPort>::build_context(
                    None,
                    None,
                    cwd.path(),
                )
                .expect("valid context");
                assert_eq!(
                    context.flags(),
                    &DiscoveryFlags::default(),
                    "flags should be default when None given"
                );
            }

            #[test]
            fn returns_context_with_default_env_when_none_given() {
                let cwd = tempfile::tempdir().expect("cwd dir");
                let context = Bootstrapper::<MockPort>::build_context(
                    None,
                    None,
                    cwd.path(),
                )
                .expect("valid context");
                assert_eq!(
                    context.env(),
                    &DiscoveryEnv::default(),
                    "env should be default when None given"
                );
            }
        }
    }

    // --- discover tests ---

    mod discover {
        use super::*;

        #[test]
        fn returns_result_from_port_when_port_succeeds() {
            let expected = DiscoveryResult::new(vec![], vec![]);
            let bootstrapper =
                Bootstrapper::new(MockPort::succeeding(expected.clone()));
            let anchor = tempfile::tempdir().expect("anchor");
            let ctx =
                DiscoveryContext::new(anchor.path()).expect("valid context");

            let result =
                bootstrapper.discover(&ctx).expect("discover should succeed");

            assert_eq!(
                result, expected,
                "bootstrapper should return the result from the port"
            );
        }

        #[test]
        fn propagates_error_from_port_when_port_fails() {
            let bootstrapper = Bootstrapper::new(MockPort::failing());
            let anchor = tempfile::tempdir().expect("anchor");
            let ctx =
                DiscoveryContext::new(anchor.path()).expect("valid context");

            let err =
                bootstrapper.discover(&ctx).expect_err("discover should fail");

            assert!(
                matches!(err, DiscoveryError::InvalidAnchorDirectory { .. }),
                "expected InvalidAnchorDirectory, got: {err:?}"
            );
        }
    }
}
