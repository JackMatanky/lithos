use std::path::PathBuf;

use super::{
    context::DiscoveryContext,
    error::DiscoveryError,
    probe::FolderProbe,
    report::{
        DiscoveryReport, GlobalResolutionSkipReason, LocalTraversalStopReason,
    },
    service::{CandidatePath, DiscoveryResult, DiscoveryServiceConfig},
    walk::BoundedAscent,
};

pub(crate) struct Init;

pub(crate) struct FlagOverride;

pub(crate) struct EnvOverride;

pub(crate) struct AscendingTraversal;

pub(crate) struct GlobalResolution;

pub(crate) struct Finalized;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    clippy::enum_variant_names,
    reason = "variants describe branch strategy"
)]
pub(crate) enum ExplicitOverrideBranch {
    VaultProbedSkipGlobal,
    VaultProbedRunGlobal,
    AscendSkipGlobal,
    AscendThenGlobal,
}

pub(crate) struct DiscoveryProcessor<'ctx, P> {
    config: &'ctx DiscoveryServiceConfig,
    ctx: &'ctx DiscoveryContext<'ctx>,
    vault: Vec<CandidatePath>,
    global: Vec<CandidatePath>,
    report: DiscoveryReport,
    #[expect(dead_code, reason = "typestate marker")]
    phase: P,
}

impl<'ctx> DiscoveryProcessor<'ctx, Init> {
    pub(crate) fn new(
        config: &'ctx DiscoveryServiceConfig,
        ctx: &'ctx DiscoveryContext<'ctx>,
    ) -> Self {
        Self {
            config,
            ctx,
            vault: Vec::new(),
            global: Vec::new(),
            report: DiscoveryReport {
                skipped_ceilings: Vec::new(),
                local_traversal_stop_reason:
                    LocalTraversalStopReason::FilesystemRoot,
                global_resolution_skip_reason: None,
            },
            phase: Init,
        }
    }
}

impl<'ctx> From<DiscoveryProcessor<'ctx, Init>>
    for DiscoveryProcessor<'ctx, FlagOverride>
{
    fn from(val: DiscoveryProcessor<'ctx, Init>) -> Self {
        let flags = val.ctx.flags();

        let vault = if let Some(vault_dir) = flags.vault_dir() {
            let probe = FolderProbe {
                patterns: val.config.vault_marker_patterns,
            };
            probe.probe(vault_dir)
        } else {
            Vec::new()
        };

        Self {
            config: val.config,
            ctx: val.ctx,
            vault,
            global: val.global,
            report: val.report,
            phase: FlagOverride,
        }
    }
}

impl<'ctx> From<DiscoveryProcessor<'ctx, FlagOverride>>
    for DiscoveryProcessor<'ctx, EnvOverride>
{
    fn from(val: DiscoveryProcessor<'ctx, FlagOverride>) -> Self {
        let has_config = val.ctx.flags().config_file().is_some()
            || val.ctx.env().config_file().is_some();

        let mut report = val.report;
        if has_config {
            report.local_traversal_stop_reason =
                LocalTraversalStopReason::ExplicitConfigFile;
        }

        Self {
            config: val.config,
            ctx: val.ctx,
            vault: val.vault,
            global: val.global,
            report,
            phase: EnvOverride,
        }
    }
}

impl DiscoveryProcessor<'_, EnvOverride> {
    pub(crate) fn branch_strategy(&self) -> ExplicitOverrideBranch {
        let has_vault_override = self.ctx.flags().vault_dir().is_some()
            || self.ctx.env().vault_dir().is_some();
        let has_config_override = self.ctx.flags().config_file().is_some()
            || self.ctx.env().config_file().is_some();

        match (has_vault_override, has_config_override) {
            (true, true) => ExplicitOverrideBranch::VaultProbedSkipGlobal,
            (true, false) => ExplicitOverrideBranch::VaultProbedRunGlobal,
            (false, true) => ExplicitOverrideBranch::AscendSkipGlobal,
            (false, false) => ExplicitOverrideBranch::AscendThenGlobal,
        }
    }
}

impl<'ctx> TryFrom<DiscoveryProcessor<'ctx, EnvOverride>>
    for DiscoveryProcessor<'ctx, AscendingTraversal>
{
    type Error = DiscoveryError;

    fn try_from(
        val: DiscoveryProcessor<'ctx, EnvOverride>,
    ) -> Result<Self, Self::Error> {
        let anchor_dir = val
            .ctx
            .flags()
            .vault_dir()
            .or_else(|| val.ctx.env().vault_dir())
            .unwrap_or_else(|| val.ctx.anchor());
        let anchor_path = anchor_dir.as_path().to_path_buf();
        let canonical = anchor_path.canonicalize().map_err(|source| {
            DiscoveryError::CanonicalizePath {
                path: anchor_path,
                source,
            }
        })?;

        let probe = FolderProbe {
            patterns: val.config.vault_marker_patterns,
        };

        let (valid_ceilings, skipped) = val.ctx.env().resolve_ceiling_dirs();
        let ceilings: std::collections::HashSet<PathBuf> =
            valid_ceilings.iter().map(|d| d.as_path().to_path_buf()).collect();
        let mut report = val.report;
        report.skipped_ceilings = skipped;

        let walker = BoundedAscent::new(
            &canonical,
            &ceilings,
            val.config.allow_marker_at_ceiling,
        );

        let mut vault = val.vault;
        let mut found = false;

        for current in walker {
            if ceilings.contains(current) {
                report.local_traversal_stop_reason =
                    LocalTraversalStopReason::CeilingEnforced {
                        ceiling: current.to_path_buf(),
                    };
                break;
            }

            let is_boundary =
                val.config.boundary_markers.iter().any(|marker| {
                    current.file_name().is_some_and(|name| name == *marker)
                });

            if is_boundary {
                report.local_traversal_stop_reason =
                    LocalTraversalStopReason::ProjectBoundaryMarker {
                        marker: current.to_path_buf(),
                    };
                break;
            }

            let candidates = probe.probe_dir(current);
            if !candidates.is_empty() {
                vault = candidates;
                found = true;
                break;
            }
        }

        if !found
            && report.local_traversal_stop_reason
                == LocalTraversalStopReason::FilesystemRoot
        {
            report.local_traversal_stop_reason =
                LocalTraversalStopReason::FilesystemRoot;
        }

        Ok(Self {
            config: val.config,
            ctx: val.ctx,
            vault,
            global: val.global,
            report,
            phase: AscendingTraversal,
        })
    }
}

impl<'ctx> From<DiscoveryProcessor<'ctx, EnvOverride>>
    for DiscoveryProcessor<'ctx, GlobalResolution>
{
    fn from(val: DiscoveryProcessor<'ctx, EnvOverride>) -> Self {
        let mut report = val.report;
        if val.ctx.flags().suppress_global() {
            report.global_resolution_skip_reason =
                Some(GlobalResolutionSkipReason::SuppressedByFlag);
        }

        Self {
            config: val.config,
            ctx: val.ctx,
            vault: val.vault,
            global: val.global,
            report,
            phase: GlobalResolution,
        }
    }
}

impl<'ctx> From<DiscoveryProcessor<'ctx, EnvOverride>>
    for DiscoveryProcessor<'ctx, Finalized>
{
    fn from(val: DiscoveryProcessor<'ctx, EnvOverride>) -> Self {
        let mut report = val.report;
        if val.ctx.flags().suppress_global() {
            report.global_resolution_skip_reason =
                Some(GlobalResolutionSkipReason::SuppressedByFlag);
        }

        Self {
            config: val.config,
            ctx: val.ctx,
            vault: val.vault,
            global: val.global,
            report,
            phase: Finalized,
        }
    }
}

impl<'ctx> TryFrom<DiscoveryProcessor<'ctx, AscendingTraversal>>
    for DiscoveryProcessor<'ctx, GlobalResolution>
{
    type Error = DiscoveryError;

    fn try_from(
        val: DiscoveryProcessor<'ctx, AscendingTraversal>,
    ) -> Result<Self, Self::Error> {
        let probe = FolderProbe {
            patterns: val.config.global_marker_patterns,
        };

        let mut global = Vec::new();
        for global_dir in &val.config.global_directories {
            let candidates = probe.probe(global_dir);
            if !candidates.is_empty() {
                global = candidates;
                break;
            }
        }

        Ok(Self {
            config: val.config,
            ctx: val.ctx,
            vault: val.vault,
            global,
            report: val.report,
            phase: GlobalResolution,
        })
    }
}

impl<'ctx> TryFrom<DiscoveryProcessor<'ctx, AscendingTraversal>>
    for DiscoveryProcessor<'ctx, Finalized>
{
    type Error = DiscoveryError;

    fn try_from(
        val: DiscoveryProcessor<'ctx, AscendingTraversal>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            config: val.config,
            ctx: val.ctx,
            vault: val.vault,
            global: val.global,
            report: val.report,
            phase: Finalized,
        })
    }
}

impl<'ctx> From<DiscoveryProcessor<'ctx, GlobalResolution>>
    for DiscoveryProcessor<'ctx, Finalized>
{
    fn from(val: DiscoveryProcessor<'ctx, GlobalResolution>) -> Self {
        Self {
            config: val.config,
            ctx: val.ctx,
            vault: val.vault,
            global: val.global,
            report: val.report,
            phase: Finalized,
        }
    }
}

impl DiscoveryProcessor<'_, Finalized> {
    pub(crate) fn finalize(self) -> (DiscoveryResult, DiscoveryReport) {
        let result = DiscoveryResult::new(self.vault, self.global);
        (result, self.report)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::discovery::{
        context::{DiscoveryContext, DiscoveryEnv, DiscoveryFlags},
        policy::VAULT_MARKER_PATTERNS,
        service::DiscoveryServiceConfig,
    };

    fn default_config() -> DiscoveryServiceConfig {
        DiscoveryServiceConfig::default()
    }

    fn make_context(
        anchor: &Path,
    ) -> Result<DiscoveryContext<'_>, DiscoveryError> {
        DiscoveryContext::new(anchor)
    }

    fn write_marker(root: &Path, relative: &str) -> PathBuf {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent");
        }
        std::fs::write(&path, "").expect("write marker");
        path
    }

    mod init {
        use super::*;

        #[test]
        fn stores_config_and_context_refs() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");
            let p = DiscoveryProcessor::new(&config, &ctx);

            assert_eq!(p.config.vault_marker_patterns, VAULT_MARKER_PATTERNS);
            assert_eq!(p.ctx.anchor().as_path(), root.path());
            assert!(p.vault.is_empty());
            assert!(p.global.is_empty());
            assert!(p.report.skipped_ceilings.is_empty());
            assert_eq!(
                p.report.local_traversal_stop_reason,
                LocalTraversalStopReason::FilesystemRoot
            );
            assert!(p.report.global_resolution_skip_reason.is_none());
        }
    }

    mod init_to_flag_override {
        use super::*;

        #[test]
        fn probes_flag_vault_dir_when_present() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");
            write_marker(vault_dir.path(), "lithos.toml");

            let config = default_config();
            let flags =
                DiscoveryFlags::new(None, Some(vault_dir.path()), false)
                    .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();

            assert!(!flag.vault.is_empty());
        }

        #[test]
        fn leaves_vault_empty_when_no_flag_vault_dir() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();

            assert!(flag.vault.is_empty());
        }
    }

    mod flag_to_env_override {
        use super::*;

        #[test]
        fn sets_explicit_config_stop_reason_when_config_present() {
            let root = tempfile::tempdir().expect("root");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let flags =
                DiscoveryFlags::new(Some(config_file.path()), None, false)
                    .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();

            assert_eq!(
                env.report.local_traversal_stop_reason,
                LocalTraversalStopReason::ExplicitConfigFile
            );
        }
    }

    mod branch_strategy {
        use super::*;

        #[test]
        fn vault_and_config_skip_both() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let flags = DiscoveryFlags::new(
                Some(config_file.path()),
                Some(vault_dir.path()),
                false,
            )
            .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();

            assert_eq!(
                env.branch_strategy(),
                ExplicitOverrideBranch::VaultProbedSkipGlobal
            );
        }

        #[test]
        fn vault_only_runs_global() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");

            let config = default_config();
            let flags =
                DiscoveryFlags::new(None, Some(vault_dir.path()), false)
                    .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();

            assert_eq!(
                env.branch_strategy(),
                ExplicitOverrideBranch::VaultProbedRunGlobal
            );
        }

        #[test]
        fn config_only_ascend_skip_global() {
            let root = tempfile::tempdir().expect("root");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let flags =
                DiscoveryFlags::new(Some(config_file.path()), None, false)
                    .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();

            assert_eq!(
                env.branch_strategy(),
                ExplicitOverrideBranch::AscendSkipGlobal
            );
        }

        #[test]
        fn no_overrides_ascend_then_global() {
            let root = tempfile::tempdir().expect("root");

            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();

            assert_eq!(
                env.branch_strategy(),
                ExplicitOverrideBranch::AscendThenGlobal
            );
        }

        #[test]
        fn vault_from_env_triggers_vault_probed() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");

            let config = default_config();
            let env_override =
                DiscoveryEnv::new(None, Some(vault_dir.path()), None)
                    .expect("env");
            let ctx =
                make_context(root.path()).expect("ctx").with_env(env_override);

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();

            assert_eq!(
                env.branch_strategy(),
                ExplicitOverrideBranch::VaultProbedRunGlobal
            );
        }
    }

    mod ascending_traversal {
        use super::*;

        #[test]
        fn finds_marker_in_anchor_directory() {
            let root = tempfile::tempdir().expect("root");
            write_marker(root.path(), "lithos.toml");

            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();
            let ascend: DiscoveryProcessor<AscendingTraversal> =
                env.try_into().expect("ascend");

            assert!(!ascend.vault.is_empty());
        }

        #[test]
        fn stops_at_boundary_marker() {
            let root = tempfile::tempdir().expect("root");
            let boundary = root.path().join(".git");
            std::fs::create_dir(&boundary).expect("boundary");
            let nested = boundary.join("deep");
            std::fs::create_dir_all(&nested).expect("nested");

            let config = default_config();
            let ctx = make_context(&nested).expect("ctx");

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();
            let ascend: DiscoveryProcessor<AscendingTraversal> =
                env.try_into().expect("ascend");

            assert!(ascend.vault.is_empty());
            assert_eq!(
                ascend.report.local_traversal_stop_reason,
                LocalTraversalStopReason::ProjectBoundaryMarker {
                    marker: boundary.canonicalize().expect("canonical")
                }
            );
        }

        #[test]
        fn records_skipped_ceilings_from_env() {
            let root = tempfile::tempdir().expect("root");
            let config = default_config();
            let env =
                DiscoveryEnv::new(None, None, Some(std::ffi::OsStr::new("")))
                    .expect("env");
            let ctx = make_context(root.path()).expect("ctx").with_env(env);

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env_phase: DiscoveryProcessor<EnvOverride> = flag.into();
            let ascend: DiscoveryProcessor<AscendingTraversal> =
                env_phase.try_into().expect("ascend");

            assert!(!ascend.report.skipped_ceilings.is_empty());
        }
    }

    mod finalize {
        use super::*;

        #[test]
        fn returns_result_and_report() {
            let root = tempfile::tempdir().expect("root");

            let config = default_config();
            let ctx = make_context(root.path()).expect("ctx");

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();
            let final_: DiscoveryProcessor<Finalized> = env.into();
            let (result, report) = final_.finalize();

            assert!(result.vault().is_empty());
            assert!(result.global().is_empty());
            assert_eq!(
                report.local_traversal_stop_reason,
                LocalTraversalStopReason::FilesystemRoot
            );
        }

        #[test]
        fn vault_probed_skip_global_pipeline() {
            let root = tempfile::tempdir().expect("root");
            let vault_dir = tempfile::tempdir().expect("vault dir");
            write_marker(vault_dir.path(), "lithos.toml");
            let config_file = tempfile::NamedTempFile::new().expect("config");

            let config = default_config();
            let flags = DiscoveryFlags::new(
                Some(config_file.path()),
                Some(vault_dir.path()),
                false,
            )
            .expect("flags");
            let ctx = make_context(root.path()).expect("ctx").with_flags(flags);

            let init = DiscoveryProcessor::new(&config, &ctx);
            let flag: DiscoveryProcessor<FlagOverride> = init.into();
            let env: DiscoveryProcessor<EnvOverride> = flag.into();

            assert_eq!(
                env.branch_strategy(),
                ExplicitOverrideBranch::VaultProbedSkipGlobal
            );

            let final_: DiscoveryProcessor<Finalized> = env.into();
            let (result, _) = final_.finalize();

            assert!(!result.vault().is_empty());
            assert!(result.global().is_empty());
        }
    }
}
