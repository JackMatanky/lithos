//! Bootstrap orchestration seams for runtime context acquisition.

use std::{env, path::PathBuf};

pub use crate::app::error::BootstrapError;
use crate::{
    config::{aggregate::Config, builder::Builder, repository::Repository},
    discovery::{
        context::{DiscoveryContext, DiscoveryEnv, DiscoveryFlags},
        error::DiscoveryError,
        port::DiscoveryPort,
        report::DiscoveryReport,
        service::{DiscoveryResult, DiscoveryService, DiscoveryServiceConfig},
    },
    fs::DirPath,
};

/// The outcome of a successful full bootstrap run.
///
/// Returned by [`Bootstrapper::run()`]. Contains the resolved [`Config`] and
/// the [`DiscoveryReport`] produced during the discovery phase.
#[derive(Debug)]
pub struct BootstrapResult {
    /// The fully resolved and merged configuration.
    pub config: Config,
    /// Non-fatal diagnostic information from the discovery phase.
    pub report: DiscoveryReport,
}

/// Application-owned bootstrap orchestration entry point.
///
/// `Bootstrapper` is generic over `D: DiscoveryPort` so that the discovery
/// implementation can be swapped out in tests without touching the
/// orchestration logic.
#[derive(Debug, Default)]
pub struct Bootstrapper<D: DiscoveryPort> {
    port: D,
}

impl<D: DiscoveryPort> Bootstrapper<D> {
    /// Creates a bootstrapper backed by the given discovery port.
    #[inline]
    pub fn new(port: D) -> Self {
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
    #[inline]
    pub fn build_context<'a>(
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
    #[inline]
    pub fn discover(
        &self,
        context: &DiscoveryContext<'_>,
    ) -> Result<(DiscoveryResult, DiscoveryReport), BootstrapError> {
        self.port.discover(context).map_err(Into::into)
    }

    /// Runs discovery only, without triggering config parsing or building.
    ///
    /// Builds the [`DiscoveryContext`] from the provided runtime inputs and
    /// runs discovery through the port, returning the raw discovery result and
    /// report.  The [`crate::config::builder::Builder`] is never invoked, so
    /// invalid TOML in candidate files does not cause an error.
    ///
    /// # Parameters
    ///
    /// - `flags`: Optional CLI flag overrides. Pass `None` when absent.
    /// - `env`: Optional environment variable overrides. Pass `None` when
    ///   absent.
    /// - `anchor`: The working directory used as the starting point for
    ///   ascending vault discovery. Must refer to an existing directory.
    ///
    /// # Errors
    ///
    /// - [`BootstrapError::Discovery`] if `anchor` does not exist, or if
    ///   discovery setup or execution fails.
    #[inline]
    pub fn run_discovery_only(
        &self,
        flags: Option<DiscoveryFlags>,
        env: Option<DiscoveryEnv<'_>>,
        anchor: &std::path::Path,
    ) -> Result<(DiscoveryResult, DiscoveryReport), BootstrapError> {
        let context = Self::build_context(flags, env, anchor)?;
        self.discover(&context)
    }

    /// Runs the full bootstrap pipeline: discovery → config build.
    ///
    /// Builds the [`DiscoveryContext`] from the provided runtime inputs, runs
    /// discovery through the port, then builds a [`Config`] from the
    /// discovered candidate paths using the given `repository`.
    ///
    /// # Parameters
    ///
    /// - `flags`: Optional CLI flag overrides (explicit vault/config paths,
    ///   suppress-global flag). Pass `None` when no user-supplied flags are
    ///   present.
    /// - `env`: Optional environment variable overrides (`LITHOS_VAULT`,
    ///   ceiling dirs). Pass `None` when no env overrides are present.
    /// - `anchor`: The working directory to use as the starting point for
    ///   ascending vault discovery. Must refer to an existing directory on the
    ///   filesystem — [`BootstrapError::Discovery`] is returned otherwise.
    /// - `repository`: The persistence repository for reading and writing
    ///   config views and built configs. Consumed by this call.
    ///
    /// # Errors
    ///
    /// - [`BootstrapError::Discovery`] if `anchor` does not exist, or if
    ///   discovery setup or execution fails.
    /// - [`BootstrapError::Config`] if configuration ingestion, validation, or
    ///   database operations fail.
    #[inline]
    pub fn run<R: Repository>(
        &self,
        flags: Option<DiscoveryFlags>,
        env: Option<DiscoveryEnv<'_>>,
        anchor: &std::path::Path,
        repository: R,
    ) -> Result<BootstrapResult, BootstrapError> {
        let context = Self::build_context(flags, env, anchor)?;
        let (discovery, report) = self.discover(&context)?;
        let config = Builder::from_discovery(discovery, repository).build()?;
        Ok(BootstrapResult {
            config,
            report,
        })
    }
}

impl Bootstrapper<DiscoveryService> {
    /// Creates the concrete discovery bootstrapper from platform config dirs.
    ///
    /// # Errors
    ///
    /// Returns [`BootstrapError`] if the concrete discovery service rejects
    /// its stable configuration.
    #[inline]
    pub fn from_platform() -> Result<Self, BootstrapError> {
        Self::with_global_directories(platform_global_directories())
    }

    /// Creates the concrete discovery bootstrapper with explicit global dirs.
    ///
    /// This keeps platform resolution outside tests while using the same
    /// concrete [`DiscoveryService`] construction path.
    ///
    /// # Errors
    ///
    /// Returns [`BootstrapError`] if the concrete discovery service rejects
    /// its stable configuration.
    #[inline]
    pub fn with_global_directories(
        global_directories: Vec<DirPath>,
    ) -> Result<Self, BootstrapError> {
        let config = DiscoveryServiceConfig {
            global_directories,
            ..DiscoveryServiceConfig::default()
        };
        let service = DiscoveryService::new(config)?;
        Ok(Self::new(service))
    }
}

fn platform_global_directories() -> Vec<DirPath> {
    platform_global_directory_candidates()
        .into_iter()
        .filter_map(|path| DirPath::try_new(path).ok())
        .collect()
}

fn platform_global_directory_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    match (env::var_os("XDG_CONFIG_HOME"), env::var_os("HOME")) {
        (Some(xdg_config_home), _) => {
            candidates.push(PathBuf::from(xdg_config_home).join("lithos"));
        }
        (None, Some(home)) => {
            candidates.push(PathBuf::from(home).join(".config/lithos"));
        }
        (None, None) => {}
    }

    #[cfg(windows)]
    if let Some(appdata) = env::var_os("APPDATA") {
        candidates.push(PathBuf::from(appdata).join("Lithos"));
    }

    #[cfg(unix)]
    candidates.push(PathBuf::from("/etc/lithos"));

    candidates
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use mockall::{mock, predicate::always};

    use super::*;
    use crate::{
        discovery::{
            context::DiscoveryContext,
            error::DiscoveryError,
            report::{
                DiscoveryReport, GlobalResolutionSkipReason,
                LocalTraversalStopReason, SkippedCeiling, SkippedCeilingReason,
            },
            service::{CandidatePath, DiscoveryResult, DiscoveryService},
        },
        fs::{DirPath, FilePath, PathError},
    };

    mock! {
        DiscoveryPort {}
        impl crate::discovery::port::DiscoveryPort for DiscoveryPort {
            fn discover<'ctx>(
                &self,
                context: &DiscoveryContext<'ctx>,
            ) -> Result<
                (DiscoveryResult, DiscoveryReport),
                DiscoveryError,
            >;
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
                    None,
                )
                .expect("valid env");
                Bootstrapper::<MockDiscoveryPort>::build_context(
                    Some(flags),
                    Some(env),
                    self.cwd.path(),
                )
            }
        }
    }

    fn placeholder_cache_root() -> crate::discovery::location::CacheRoot {
        use crate::discovery::location::{CacheLocation, GlobalCacheLocation};
        crate::discovery::location::CacheRoot {
            location: CacheLocation::Global(
                GlobalCacheLocation::PlatformUserCache,
            ),
            path: std::path::PathBuf::from("/tmp/placeholder-cache"),
        }
    }

    // --- build_context tests ---

    mod build_context {
        use super::*;

        mod constructor {
            use pretty_assertions::assert_eq;

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
                let context = Bootstrapper::<MockDiscoveryPort>::build_context(
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
                let context = Bootstrapper::<MockDiscoveryPort>::build_context(
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
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn returns_result_from_port_when_port_succeeds() {
            let expected =
                DiscoveryResult::new(vec![], vec![], placeholder_cache_root());
            let report = DiscoveryReport {
                skipped_ceilings: vec![],
                local_traversal_stop_reason:
                    LocalTraversalStopReason::FilesystemRoot,
                global_resolution_skip_reason: None,
            };
            let anchor = tempfile::tempdir().expect("anchor");
            let ctx =
                DiscoveryContext::new(anchor.path()).expect("valid context");

            let mut mock = MockDiscoveryPort::new();
            let ret = expected.clone();
            let rep = report.clone();
            mock.expect_discover()
                .with(always())
                .once()
                .returning(move |_| Ok((ret.clone(), rep.clone())));
            let bootstrapper = Bootstrapper::new(mock);

            let (result, _) =
                bootstrapper.discover(&ctx).expect("discover should succeed");

            assert_eq!(result, expected);
        }

        #[test]
        fn propagates_error_from_port_when_port_fails() {
            let anchor = tempfile::tempdir().expect("anchor");
            let ctx =
                DiscoveryContext::new(anchor.path()).expect("valid context");

            let mut mock = MockDiscoveryPort::new();
            mock.expect_discover().with(always()).once().returning(|_| {
                Err(DiscoveryError::InvalidAnchorDirectory {
                    path: std::path::PathBuf::from("/bad"),
                    source: PathError::NotADirectory(std::path::PathBuf::from(
                        "/bad",
                    )),
                })
            });
            let bootstrapper = Bootstrapper::new(mock);

            let err =
                bootstrapper.discover(&ctx).expect_err("discover should fail");

            assert!(
                matches!(
                    err,
                    BootstrapError::Discovery(
                        DiscoveryError::InvalidAnchorDirectory { .. }
                    )
                ),
                "expected InvalidAnchorDirectory, got: {err:?}"
            );
        }

        #[test]
        fn returns_report_from_port_when_port_succeeds() {
            let report = DiscoveryReport {
                skipped_ceilings: vec![SkippedCeiling {
                    segment: std::path::PathBuf::from("/missing"),
                    reason: SkippedCeilingReason::InvalidPath,
                }],
                local_traversal_stop_reason:
                    LocalTraversalStopReason::ExplicitConfigFile,
                global_resolution_skip_reason: Some(
                    GlobalResolutionSkipReason::SuppressedByFlag,
                ),
            };
            let anchor = tempfile::tempdir().expect("anchor");
            let ctx =
                DiscoveryContext::new(anchor.path()).expect("valid context");

            let mut mock = MockDiscoveryPort::new();
            let expected = report.clone();
            mock.expect_discover().with(always()).once().returning(move |_| {
                Ok((
                    DiscoveryResult::new(
                        vec![],
                        vec![],
                        placeholder_cache_root(),
                    ),
                    expected.clone(),
                ))
            });
            let bootstrapper = Bootstrapper::new(mock);

            let (_, result) =
                bootstrapper.discover(&ctx).expect("discover should succeed");

            assert_eq!(result, report);
        }
    }

    mod bootstrap_error {
        use super::*;
        use crate::config::error::ConfigError;

        #[test]
        fn includes_config_variant() {
            let e = BootstrapError::Config(ConfigError::Ingestion("x".into()));
            assert!(matches!(e, BootstrapError::Config(_)));
        }
    }

    mod run {
        use mockall::predicate::always;

        use super::*;
        use crate::{
            config::storage::testing::InMemoryRepository,
            discovery::service::{CandidatePath, DiscoveryResult},
            fs::{DirPath, FilePath},
        };

        #[test]
        fn builds_config_from_vault_only_discovery() {
            // IMPORTANT: the TOML must contain at least one non-default field
            // value so that `compute_field_hashes` returns a non-empty set.
            // An empty or all-default TOML would produce a `NoChanges` →
            // `UpdateViewOnly` → `UseCached` plan, and
            // `load_cached_config` would then fail on a fresh
            // `InMemoryRepository` ("No active config version found").
            // `[template]\ndirectory = "templates"` overrides the default and
            // drives the `Rebuild` path which saves the config before
            // returning it.
            let root = tempfile::tempdir().expect("vault root");
            let config_path = root.path().join("lithos.toml");
            std::fs::write(
                &config_path,
                "[template]\ndirectory = \"templates\"",
            )
            .expect("write config");
            let vault_candidate = CandidatePath::new(
                DirPath::try_new(root.path().to_path_buf())
                    .expect("valid base dir"),
                FilePath::try_new(config_path).expect("valid file path"),
            );
            let discovery = DiscoveryResult::new(
                vec![vault_candidate],
                vec![],
                placeholder_cache_root(),
            );
            let report = DiscoveryReport {
                skipped_ceilings: vec![],
                local_traversal_stop_reason:
                    LocalTraversalStopReason::FilesystemRoot,
                global_resolution_skip_reason: None,
            };
            let mut mock = MockDiscoveryPort::new();
            let disc = discovery.clone();
            let rep = report.clone();
            mock.expect_discover()
                .with(always())
                .once()
                .returning(move |_| Ok((disc.clone(), rep.clone())));
            let bootstrapper = Bootstrapper::new(mock);
            let anchor = tempfile::tempdir().expect("anchor");

            let result = bootstrapper.run::<InMemoryRepository>(
                None,
                None,
                anchor.path(),
                InMemoryRepository::new(),
            );

            assert!(result.is_ok(), "expected Ok, got: {result:?}");
        }

        #[test]
        fn propagates_discovery_error() {
            let mut mock = MockDiscoveryPort::new();
            mock.expect_discover().with(always()).once().returning(|_| {
                Err(DiscoveryError::InvalidAnchorDirectory {
                    path: std::path::PathBuf::from("/bad"),
                    source: crate::fs::PathError::NotADirectory(
                        std::path::PathBuf::from("/bad"),
                    ),
                })
            });
            let bootstrapper = Bootstrapper::new(mock);
            let anchor = tempfile::tempdir().expect("anchor");

            let err = bootstrapper
                .run::<InMemoryRepository>(
                    None,
                    None,
                    anchor.path(),
                    InMemoryRepository::new(),
                )
                .expect_err("expected error");

            assert!(
                matches!(err, BootstrapError::Discovery(_)),
                "expected Discovery error, got: {err:?}"
            );
        }

        #[test]
        fn propagates_config_error() {
            let root = tempfile::tempdir().expect("vault root");
            let config_path = root.path().join("lithos.toml");
            // "not = [toml" is an unclosed array literal — invalid TOML.
            // The parser returns a `TomlParse` error which is converted via
            // `ConfigIngestError → ConfigError::Ingestion` and then wrapped as
            // `BootstrapError::Config`.  Any TOML parse failure exercises this
            // path; invalid array syntax is the simplest trigger.
            std::fs::write(&config_path, "not = [toml").expect("write config");
            let vault_candidate = CandidatePath::new(
                DirPath::try_new(root.path().to_path_buf())
                    .expect("valid base dir"),
                FilePath::try_new(config_path).expect("valid file path"),
            );
            let discovery = DiscoveryResult::new(
                vec![vault_candidate],
                vec![],
                placeholder_cache_root(),
            );
            let report = DiscoveryReport {
                skipped_ceilings: vec![],
                local_traversal_stop_reason:
                    LocalTraversalStopReason::FilesystemRoot,
                global_resolution_skip_reason: None,
            };
            let mut mock = MockDiscoveryPort::new();
            let disc = discovery.clone();
            let rep = report.clone();
            mock.expect_discover()
                .with(always())
                .once()
                .returning(move |_| Ok((disc.clone(), rep.clone())));
            let bootstrapper = Bootstrapper::new(mock);
            let anchor = tempfile::tempdir().expect("anchor");

            let err = bootstrapper
                .run::<InMemoryRepository>(
                    None,
                    None,
                    anchor.path(),
                    InMemoryRepository::new(),
                )
                .expect_err("expected error");

            assert!(
                matches!(err, BootstrapError::Config(_)),
                "expected Config error, got: {err:?}"
            );
        }

        #[test]
        fn builds_config_from_vault_and_global_discovery() {
            // Verify run() handles a DiscoveryResult that contains both vault
            // AND global candidates (the most common production path).
            // IMPORTANT: same Rebuild-plan requirement applies — both TOML
            // files must have at least one non-default field value.
            let vault_root = tempfile::tempdir().expect("vault root");
            let global_root = tempfile::tempdir().expect("global root");
            let vault_path = vault_root.path().join("lithos.toml");
            let global_path = global_root.path().join("lithos.toml");
            std::fs::write(
                &vault_path,
                "[template]\ndirectory = \"vault-templates\"",
            )
            .expect("write vault config");
            std::fs::write(
                &global_path,
                "[template]\ndirectory = \"global-templates\"",
            )
            .expect("write global config");
            let vault_candidate = CandidatePath::new(
                DirPath::try_new(vault_root.path().to_path_buf())
                    .expect("valid vault base dir"),
                FilePath::try_new(vault_path).expect("valid vault path"),
            );
            let global_candidate = CandidatePath::new(
                DirPath::try_new(global_root.path().to_path_buf())
                    .expect("valid global base dir"),
                FilePath::try_new(global_path).expect("valid global path"),
            );
            let discovery = DiscoveryResult::new(
                vec![vault_candidate],
                vec![global_candidate],
                placeholder_cache_root(),
            );
            let report = DiscoveryReport {
                skipped_ceilings: vec![],
                local_traversal_stop_reason:
                    LocalTraversalStopReason::FilesystemRoot,
                global_resolution_skip_reason: None,
            };
            let mut mock = MockDiscoveryPort::new();
            let disc = discovery.clone();
            let rep = report.clone();
            mock.expect_discover()
                .with(always())
                .once()
                .returning(move |_| Ok((disc.clone(), rep.clone())));
            let bootstrapper = Bootstrapper::new(mock);
            let anchor = tempfile::tempdir().expect("anchor");

            let result = bootstrapper.run::<InMemoryRepository>(
                None,
                None,
                anchor.path(),
                InMemoryRepository::new(),
            );

            assert!(result.is_ok(), "expected Ok, got: {result:?}");
        }

        #[test]
        fn propagates_discovery_error_from_invalid_anchor() {
            // Verifies that run() returns BootstrapError::Discovery when the
            // anchor directory does not exist.  build_context() calls
            // DiscoveryContext::new() which validates the anchor, so the error
            // surfaces before the port is ever called.
            let non_existent = std::path::PathBuf::from(
                "/tmp/__lithos_test_nonexistent_anchor_dir__",
            );
            // Ensure the path really doesn't exist.
            let _ = std::fs::remove_dir_all(&non_existent);

            let mock = MockDiscoveryPort::new();
            let bootstrapper = Bootstrapper::new(mock);

            let err = bootstrapper
                .run::<InMemoryRepository>(
                    None,
                    None,
                    &non_existent,
                    InMemoryRepository::new(),
                )
                .expect_err("expected error for non-existent anchor");

            assert!(
                matches!(err, BootstrapError::Discovery(_)),
                "expected Discovery error, got: {err:?}"
            );
        }

        #[test]
        fn returns_report_alongside_config() {
            let root = tempfile::tempdir().expect("vault root");
            let config_path = root.path().join("lithos.toml");
            std::fs::write(
                &config_path,
                "[template]\ndirectory = \"templates\"",
            )
            .expect("write config");
            let vault_candidate = CandidatePath::new(
                DirPath::try_new(root.path().to_path_buf())
                    .expect("valid base dir"),
                FilePath::try_new(config_path).expect("valid file path"),
            );
            let discovery = DiscoveryResult::new(
                vec![vault_candidate],
                vec![],
                placeholder_cache_root(),
            );
            let expected_report = DiscoveryReport {
                skipped_ceilings: vec![],
                local_traversal_stop_reason:
                    LocalTraversalStopReason::ExplicitConfigFile,
                global_resolution_skip_reason: Some(
                    GlobalResolutionSkipReason::SuppressedByFlag,
                ),
            };
            let mut mock = MockDiscoveryPort::new();
            let disc = discovery.clone();
            let rep = expected_report.clone();
            mock.expect_discover()
                .with(always())
                .once()
                .returning(move |_| Ok((disc.clone(), rep.clone())));
            let bootstrapper = Bootstrapper::new(mock);
            let anchor = tempfile::tempdir().expect("anchor");

            let result = bootstrapper
                .run::<InMemoryRepository>(
                    None,
                    None,
                    anchor.path(),
                    InMemoryRepository::new(),
                )
                .expect("run should succeed");

            assert_eq!(
                result.report, expected_report,
                "returned report should match mock report"
            );
        }
    }

    mod run_discovery_only {
        use mockall::predicate::always;
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn returns_discovery_result_when_port_succeeds() {
            let expected =
                DiscoveryResult::new(vec![], vec![], placeholder_cache_root());
            let report = DiscoveryReport {
                skipped_ceilings: vec![],
                local_traversal_stop_reason:
                    LocalTraversalStopReason::FilesystemRoot,
                global_resolution_skip_reason: None,
            };
            let anchor = tempfile::tempdir().expect("anchor");
            let mut mock = MockDiscoveryPort::new();
            let ret = expected.clone();
            let rep = report.clone();
            mock.expect_discover()
                .with(always())
                .once()
                .returning(move |_| Ok((ret.clone(), rep.clone())));
            let bootstrapper = Bootstrapper::new(mock);

            let (result, _) = bootstrapper
                .run_discovery_only(None, None, anchor.path())
                .expect("run_discovery_only should succeed");

            assert_eq!(result, expected);
        }

        #[test]
        fn returns_report_when_port_succeeds() {
            let report = DiscoveryReport {
                skipped_ceilings: vec![SkippedCeiling {
                    segment: std::path::PathBuf::from("/missing"),
                    reason: SkippedCeilingReason::InvalidPath,
                }],
                local_traversal_stop_reason:
                    LocalTraversalStopReason::ExplicitConfigFile,
                global_resolution_skip_reason: Some(
                    GlobalResolutionSkipReason::SuppressedByFlag,
                ),
            };
            let anchor = tempfile::tempdir().expect("anchor");
            let mut mock = MockDiscoveryPort::new();
            let expected = report.clone();
            mock.expect_discover().with(always()).once().returning(move |_| {
                Ok((
                    DiscoveryResult::new(
                        vec![],
                        vec![],
                        placeholder_cache_root(),
                    ),
                    expected.clone(),
                ))
            });
            let bootstrapper = Bootstrapper::new(mock);

            let (_, result) = bootstrapper
                .run_discovery_only(None, None, anchor.path())
                .expect("run_discovery_only should succeed");

            assert_eq!(result, report);
        }

        #[test]
        fn propagates_discovery_error_when_port_fails() {
            let anchor = tempfile::tempdir().expect("anchor");
            let mut mock = MockDiscoveryPort::new();
            mock.expect_discover().with(always()).once().returning(|_| {
                Err(DiscoveryError::InvalidAnchorDirectory {
                    path: std::path::PathBuf::from("/bad"),
                    source: PathError::NotADirectory(std::path::PathBuf::from(
                        "/bad",
                    )),
                })
            });
            let bootstrapper = Bootstrapper::new(mock);

            let err = bootstrapper
                .run_discovery_only(None, None, anchor.path())
                .expect_err("run_discovery_only should fail");

            assert!(
                matches!(
                    err,
                    BootstrapError::Discovery(
                        DiscoveryError::InvalidAnchorDirectory { .. }
                    )
                ),
                "expected InvalidAnchorDirectory, got: {err:?}"
            );
        }

        #[test]
        fn propagates_discovery_error_from_invalid_anchor() {
            let non_existent = std::path::PathBuf::from(
                "/tmp/__lithos_test_nonexistent_anchor_dir_discovery_only__",
            );
            let _ = std::fs::remove_dir_all(&non_existent);

            let mock = MockDiscoveryPort::new();
            let bootstrapper = Bootstrapper::new(mock);

            let err = bootstrapper
                .run_discovery_only(None, None, &non_existent)
                .expect_err("expected error for non-existent anchor");

            assert!(
                matches!(err, BootstrapError::Discovery(_)),
                "expected Discovery error, got: {err:?}"
            );
        }

        #[test]
        fn does_not_return_config_error_when_discovery_result_contains_invalid_toml()
         {
            // run() would return BootstrapError::Config for invalid TOML
            // because Builder parses it.  run_discovery_only() must
            // NOT call Builder, so it must succeed even when the
            // discovered candidate path points to invalid TOML.
            let root = tempfile::tempdir().expect("vault root");
            let config_path = root.path().join("lithos.toml");
            // "not = [toml" is an unclosed array — invalid TOML.
            std::fs::write(&config_path, "not = [toml").expect("write config");
            let vault_candidate = CandidatePath::new(
                DirPath::try_new(root.path().to_path_buf())
                    .expect("valid base dir"),
                FilePath::try_new(config_path).expect("valid file path"),
            );
            let discovery = DiscoveryResult::new(
                vec![vault_candidate],
                vec![],
                placeholder_cache_root(),
            );
            let report = DiscoveryReport {
                skipped_ceilings: vec![],
                local_traversal_stop_reason:
                    LocalTraversalStopReason::FilesystemRoot,
                global_resolution_skip_reason: None,
            };
            let mut mock = MockDiscoveryPort::new();
            let disc = discovery.clone();
            let rep = report.clone();
            mock.expect_discover()
                .with(always())
                .once()
                .returning(move |_| Ok((disc.clone(), rep.clone())));
            let bootstrapper = Bootstrapper::new(mock);
            let anchor = tempfile::tempdir().expect("anchor");

            let result =
                bootstrapper.run_discovery_only(None, None, anchor.path());

            assert!(
                result.is_ok(),
                "run_discovery_only must not trigger config parsing; got: \
                 {result:?}"
            );
        }
    }

    mod concrete_service {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn returns_app_result_from_concrete_discovery_service() {
            let root = tempfile::tempdir().expect("vault root");
            let config_path = root.path().join("lithos.toml");
            std::fs::write(&config_path, "").expect("write config");
            let flags = DiscoveryFlags::new(
                Some(config_path.as_path()),
                Some(root.path()),
                true,
            )
            .expect("valid flags");
            let context = Bootstrapper::<DiscoveryService>::build_context(
                Some(flags),
                None,
                root.path(),
            )
            .expect("valid context");
            let expected = CandidatePath::new(
                DirPath::try_new(root.path().to_path_buf())
                    .expect("valid base dir"),
                FilePath::try_new(config_path).expect("valid config file"),
            );
            let bootstrapper = Bootstrapper::with_global_directories(vec![])
                .expect("valid bootstrapper");

            let (result, _) = bootstrapper
                .discover(&context)
                .expect("discovery should succeed");

            assert_eq!(result.vault(), [expected]);
        }

        #[test]
        fn constructs_concrete_service_from_platform_directories() {
            let result = Bootstrapper::from_platform();

            assert!(result.is_ok());
        }

        #[test]
        fn run_builds_config_from_vault_with_platform_bootstrapper() {
            use crate::config::storage::testing::InMemoryRepository;

            let root = tempfile::tempdir().expect("vault root");
            let config_path = root.path().join("lithos.toml");
            std::fs::write(
                &config_path,
                "[template]\ndirectory = \"templates\"",
            )
            .expect("write config");
            let flags = DiscoveryFlags::new(
                Some(config_path.as_path()),
                Some(root.path()),
                true,
            )
            .expect("valid flags");
            let bootstrapper = Bootstrapper::with_global_directories(vec![])
                .expect("valid bootstrapper");

            let result = bootstrapper.run(
                Some(flags),
                None,
                root.path(),
                InMemoryRepository::new(),
            );

            assert!(result.is_ok(), "expected Ok, got: {result:?}");
        }
    }
}
