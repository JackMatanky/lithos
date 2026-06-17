//! Service facade, configuration, and boundary data for the redesigned
//! discovery service.

use crate::{
    discovery::{
        context::DiscoveryContext,
        error::{DiscoveryError, ServiceConfigError},
        policy::{
            BOUNDARY_MARKER_PATTERNS, GLOBAL_MARKER_PATTERNS, MarkerPattern,
            VAULT_MARKER_PATTERNS,
        },
        port::DiscoveryPort,
        processor::{
            AscendingTraversal, DiscoveryProcessor, EnvOverride,
            ExplicitOverrideBranch, Finalized, FlagBranch, FlagOverride,
            GlobalResolution,
        },
        report::DiscoveryReport,
    },
    fs::{DirPath, FilePath},
};

/// A validated candidate config path and the base directory it was found from.
///
/// Both `base` and `path` are filesystem-validated at construction.
/// The `base` directory is the starting point used to resolve `path`
/// during a traversal or global probe pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidatePath {
    /// Base directory used to resolve the candidate.
    base: DirPath,
    /// Candidate config file path.
    path: FilePath,
}

impl CandidatePath {
    /// Creates a validated discovery candidate path.
    #[inline]
    #[must_use]
    pub fn new(base: DirPath, path: FilePath) -> Self {
        Self {
            base,
            path,
        }
    }

    /// Returns the base directory used to resolve this candidate.
    #[inline]
    #[must_use]
    pub fn base(&self) -> &DirPath {
        &self.base
    }

    /// Returns the candidate config file path.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &FilePath {
        &self.path
    }
}

/// Owned vault and global candidate slices returned by
/// [`DiscoveryResult::into_parts`].
type DiscoveryResultParts = (Box<[CandidatePath]>, Box<[CandidatePath]>);

/// Pure discovery output consumed by downstream configuration loading.
///
/// Candidates are stored as boxed slices — the list is frozen once
/// [`DiscoveryProcessor::finalize`] is called and never grows after that
/// point, so `Box<[T]>` communicates immutability and avoids carrying
/// unused `Vec` capacity into downstream callers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryResult {
    /// Ordered vault-local candidates.
    vault: Box<[CandidatePath]>,
    /// Ordered global candidates.
    global: Box<[CandidatePath]>,
}

impl DiscoveryResult {
    /// Creates discovery output from ordered vault and global candidates.
    #[inline]
    #[must_use]
    pub fn new<V, G>(vault: V, global: G) -> Self
    where
        V: Into<Box<[CandidatePath]>>,
        G: Into<Box<[CandidatePath]>>,
    {
        Self {
            vault: vault.into(),
            global: global.into(),
        }
    }

    /// Returns ordered vault-local candidates.
    #[inline]
    #[must_use]
    pub fn vault(&self) -> &[CandidatePath] {
        &self.vault
    }

    /// Returns ordered global candidates.
    #[inline]
    #[must_use]
    pub fn global(&self) -> &[CandidatePath] {
        &self.global
    }

    /// Consumes the result into owned boxed candidate slices.
    #[inline]
    #[must_use]
    pub fn into_parts(self) -> DiscoveryResultParts {
        (self.vault, self.global)
    }
}

/// Explicit stable configuration for the discovery service.
///
/// This configuration object holds the stable knobs that are set once per
/// application run: marker patterns, boundary markers, global directories,
/// and traversal policy. Per-invocation state like `suppress_global`,
/// explicit paths, and anchor directory belong in
/// [`DiscoveryContext`](crate::discovery::context::DiscoveryContext).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DiscoveryServiceConfig {
    /// Ordered marker patterns for vault/local config candidates.
    pub(crate) vault_marker_patterns: &'static [MarkerPattern],
    /// Ordered marker patterns for global config candidates.
    pub(crate) global_marker_patterns: &'static [MarkerPattern],
    /// Project boundary directory names that stop ascending traversal.
    pub(crate) boundary_markers: &'static [&'static str],
    /// Pre-resolved global namespace directories supplied by the app layer.
    pub(crate) global_directories: Vec<DirPath>,
    /// Whether a marker at the ceiling directory is valid.
    pub(crate) allow_marker_at_ceiling: bool,
}

impl Default for DiscoveryServiceConfig {
    fn default() -> Self {
        Self {
            vault_marker_patterns: VAULT_MARKER_PATTERNS,
            global_marker_patterns: GLOBAL_MARKER_PATTERNS,
            boundary_markers: BOUNDARY_MARKER_PATTERNS,
            global_directories: Vec::new(),
            allow_marker_at_ceiling: true,
        }
    }
}

impl DiscoveryServiceConfig {
    /// Validates internal consistency of the configuration.
    ///
    /// # Errors
    ///
    /// Returns [`DiscoveryError::Config`] if any marker pattern list is
    /// empty.
    pub(crate) fn validate(&self) -> Result<(), DiscoveryError> {
        if self.vault_marker_patterns.is_empty() {
            return Err(ServiceConfigError::VaultMarkerPatterns.into());
        }
        if self.global_marker_patterns.is_empty() {
            return Err(ServiceConfigError::GlobalMarkerPatterns.into());
        }
        if self.boundary_markers.is_empty() {
            return Err(ServiceConfigError::BoundaryMarkerPatterns.into());
        }
        Ok(())
    }
}

/// The concrete discovery service that will implement [`DiscoveryPort`].
///
/// Construction validates that the provided configuration is internally
/// consistent. No discovery execution happens at construction time.
///
/// This type is `pub` so that external crates can name the concrete
/// bootstrapper type returned by [`Bootstrapper::from_platform`].
/// Construction is still restricted to crate-internal code via the
/// `pub(crate)` [`DiscoveryService::new`] constructor.
///
/// [`DiscoveryPort`]: crate::discovery::port::DiscoveryPort
/// [`Bootstrapper::from_platform`]: crate::app::bootstrap::Bootstrapper::from_platform
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryService {
    /// Stable service configuration.
    config: DiscoveryServiceConfig,
}

impl DiscoveryService {
    /// Constructs a validated discovery service from the given config.
    ///
    /// # Errors
    ///
    /// Returns [`DiscoveryError::Config`] if the configuration is invalid.
    pub(crate) fn new(
        config: DiscoveryServiceConfig,
    ) -> Result<Self, DiscoveryError> {
        config.validate()?;
        Ok(Self {
            config,
        })
    }
}

impl DiscoveryPort for DiscoveryService {
    #[inline]
    fn discover(
        &self,
        context: &DiscoveryContext<'_>,
    ) -> Result<(DiscoveryResult, DiscoveryReport), DiscoveryError> {
        let init = DiscoveryProcessor::new(&self.config, context);
        let flag: DiscoveryProcessor<FlagOverride> = From::from(init);

        // Flags have highest precedence. If they fully determine the pipeline
        // path, skip env entirely.
        match flag.flag_branch() {
            FlagBranch::VaultAndConfigSet => {
                let final_s: DiscoveryProcessor<Finalized> = From::from(flag);
                return Ok(final_s.finalize());
            }
            FlagBranch::VaultOnlySet => {
                let global: DiscoveryProcessor<GlobalResolution> =
                    From::from(flag);
                let final_s: DiscoveryProcessor<Finalized> = From::from(global);
                return Ok(final_s.finalize());
            }
            FlagBranch::ConsultEnv => {}
        }

        // Flags were insufficient; consult environment variables.
        let env: DiscoveryProcessor<EnvOverride> = From::from(flag);

        match env.branch_strategy() {
            ExplicitOverrideBranch::VaultProbedSkipGlobal => {
                let final_s: DiscoveryProcessor<Finalized> = From::from(env);
                Ok(final_s.finalize())
            }
            ExplicitOverrideBranch::VaultProbedRunGlobal => {
                let global: DiscoveryProcessor<GlobalResolution> =
                    From::from(env);
                let final_s: DiscoveryProcessor<Finalized> = From::from(global);
                Ok(final_s.finalize())
            }
            ExplicitOverrideBranch::AscendSkipGlobal => {
                let ascend: DiscoveryProcessor<AscendingTraversal> =
                    TryFrom::try_from(env)?;
                let final_s: DiscoveryProcessor<Finalized> = From::from(ascend);
                Ok(final_s.finalize())
            }
            ExplicitOverrideBranch::AscendThenGlobal => {
                let ascend: DiscoveryProcessor<AscendingTraversal> =
                    TryFrom::try_from(env)?;
                let global: DiscoveryProcessor<GlobalResolution> =
                    From::from(ascend);
                let final_s: DiscoveryProcessor<Finalized> = From::from(global);
                Ok(final_s.finalize())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::FilePath;

    fn candidate(root: &tempfile::TempDir, name: &str) -> CandidatePath {
        let path = root.path().join(name);
        std::fs::write(&path, "").expect("write candidate file");

        CandidatePath::new(
            DirPath::try_new(root.path().to_path_buf())
                .expect("valid base dir"),
            FilePath::try_new(path).expect("valid candidate file"),
        )
    }

    fn make_dirpath(root: &tempfile::TempDir) -> DirPath {
        DirPath::try_new(root.path().to_path_buf())
            .expect("valid global directory")
    }

    mod discovery_result {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn keeps_vault_candidates_separate_from_global() {
            let vault_root = tempfile::tempdir().expect("vault root");
            let global_root = tempfile::tempdir().expect("global root");
            let vault = candidate(&vault_root, "lithos.toml");
            let global = candidate(&global_root, "lithos.toml");

            let result =
                DiscoveryResult::new(vec![vault.clone()], vec![global]);

            assert_eq!(result.vault(), [vault]);
        }

        #[test]
        fn keeps_global_candidates_separate_from_vault() {
            let vault_root = tempfile::tempdir().expect("vault root");
            let global_root = tempfile::tempdir().expect("global root");
            let vault = candidate(&vault_root, "lithos.toml");
            let global = candidate(&global_root, "lithos.toml");

            let result =
                DiscoveryResult::new(vec![vault], vec![global.clone()]);

            assert_eq!(result.global(), [global]);
        }

        #[test]
        fn into_parts_returns_vault_candidates() {
            let vault_root = tempfile::tempdir().expect("vault root");
            let global_root = tempfile::tempdir().expect("global root");
            let vault = candidate(&vault_root, "lithos.toml");
            let global = candidate(&global_root, "lithos.toml");
            let result =
                DiscoveryResult::new(vec![vault.clone()], vec![global]);

            let (vault_candidates, _) = result.into_parts();

            assert_eq!(*vault_candidates, [vault]);
        }

        #[test]
        fn into_parts_returns_global_candidates() {
            let vault_root = tempfile::tempdir().expect("vault root");
            let global_root = tempfile::tempdir().expect("global root");
            let vault = candidate(&vault_root, "lithos.toml");
            let global = candidate(&global_root, "lithos.toml");
            let result =
                DiscoveryResult::new(vec![vault], vec![global.clone()]);

            let (_, global_candidates) = result.into_parts();

            assert_eq!(*global_candidates, [global]);
        }
    }

    mod discovery_service_config {
        use super::*;
        use crate::discovery::policy::{
            BOUNDARY_MARKER_PATTERNS, GLOBAL_MARKER_PATTERNS,
            VAULT_MARKER_PATTERNS,
        };

        mod default {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn uses_vault_marker_patterns() {
                let config = DiscoveryServiceConfig::default();
                assert_eq!(config.vault_marker_patterns, VAULT_MARKER_PATTERNS);
            }

            #[test]
            fn uses_global_marker_patterns() {
                let config = DiscoveryServiceConfig::default();
                assert_eq!(
                    config.global_marker_patterns,
                    GLOBAL_MARKER_PATTERNS
                );
            }

            #[test]
            fn uses_default_boundary_markers() {
                let config = DiscoveryServiceConfig::default();
                assert_eq!(config.boundary_markers, BOUNDARY_MARKER_PATTERNS);
            }

            #[test]
            fn has_empty_global_directories() {
                let config = DiscoveryServiceConfig::default();
                assert!(config.global_directories.is_empty());
            }

            #[test]
            fn allows_marker_at_ceiling() {
                let config = DiscoveryServiceConfig::default();
                assert!(
                    config.allow_marker_at_ceiling,
                    "default should allow marker at ceiling"
                );
            }
        }

        mod constructor {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn accepts_explicit_global_directories() {
                let dir = tempfile::tempdir().expect("global dir");
                let global_dir = make_dirpath(&dir);

                let config = DiscoveryServiceConfig {
                    global_directories: vec![global_dir],
                    ..DiscoveryServiceConfig::default()
                };

                assert_eq!(config.global_directories.len(), 1);
            }

            #[test]
            fn disables_marker_at_ceiling() {
                let config = DiscoveryServiceConfig {
                    allow_marker_at_ceiling: false,
                    ..DiscoveryServiceConfig::default()
                };

                assert!(!config.allow_marker_at_ceiling);
            }
        }
    }

    mod discovery_service {
        use super::*;
        use crate::discovery::policy::{
            BOUNDARY_MARKER_PATTERNS, GLOBAL_MARKER_PATTERNS,
            VAULT_MARKER_PATTERNS,
        };

        mod constructor {
            use super::*;

            #[test]
            fn succeeds_with_default_config() {
                let service =
                    DiscoveryService::new(DiscoveryServiceConfig::default());
                assert!(
                    service.is_ok(),
                    "expected Ok, got: {:?}",
                    service.as_ref().err()
                );
            }

            #[test]
            fn succeeds_with_explicit_config() {
                let dir = tempfile::tempdir().expect("global dir");
                let config = DiscoveryServiceConfig {
                    vault_marker_patterns: VAULT_MARKER_PATTERNS,
                    global_marker_patterns: GLOBAL_MARKER_PATTERNS,
                    boundary_markers: BOUNDARY_MARKER_PATTERNS,
                    global_directories: vec![make_dirpath(&dir)],
                    allow_marker_at_ceiling: false,
                };

                let service = DiscoveryService::new(config);
                assert!(
                    service.is_ok(),
                    "expected Ok, got: {:?}",
                    service.as_ref().err()
                );
            }

            #[test]
            fn rejects_empty_vault_marker_patterns() {
                let config = DiscoveryServiceConfig {
                    vault_marker_patterns: &[],
                    ..DiscoveryServiceConfig::default()
                };

                let err = DiscoveryService::new(config)
                    .expect_err("should reject empty vault patterns");

                assert!(
                    matches!(
                        err,
                        DiscoveryError::Config(
                            ServiceConfigError::VaultMarkerPatterns,
                        )
                    ),
                    "expected VaultMarkerPatterns, got: {err:?}"
                );
            }

            #[test]
            fn rejects_empty_global_marker_patterns() {
                let config = DiscoveryServiceConfig {
                    global_marker_patterns: &[],
                    ..DiscoveryServiceConfig::default()
                };

                let err = DiscoveryService::new(config)
                    .expect_err("should reject empty global patterns");

                assert!(
                    matches!(
                        err,
                        DiscoveryError::Config(
                            ServiceConfigError::GlobalMarkerPatterns,
                        )
                    ),
                    "expected GlobalMarkerPatterns, got: {err:?}"
                );
            }

            #[test]
            fn rejects_empty_boundary_markers() {
                let config = DiscoveryServiceConfig {
                    boundary_markers: &[],
                    ..DiscoveryServiceConfig::default()
                };

                let err = DiscoveryService::new(config)
                    .expect_err("should reject empty boundary markers");

                assert!(
                    matches!(
                        err,
                        DiscoveryError::Config(
                            ServiceConfigError::BoundaryMarkerPatterns,
                        )
                    ),
                    "expected BoundaryMarkerPatterns, got: {err:?}"
                );
            }
        }
    }

    mod suppress_global_is_not_in_config {
        use super::*;

        #[test]
        fn belongs_to_discovery_flags_not_service_config() {
            let config = DiscoveryServiceConfig::default();
            let debug_repr = format!("{config:?}");
            assert!(
                !debug_repr.contains("suppress_global"),
                "suppress_global belongs in DiscoveryFlags, not \
                 DiscoveryServiceConfig: {debug_repr}"
            );
        }
    }
}
