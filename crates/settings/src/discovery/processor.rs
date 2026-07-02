//! Linear internal discovery processor.

use std::{marker::PhantomData, path::PathBuf};

use traces_fs::DirPath;

use crate::{
    DiscoveryOutcome, DiscoveryReport,
    candidate::CandidatePath,
    discovery::{
        error::DiscoveryError,
        filter::{dedupe, dedupe_keep_last, filter_ignored},
        global::global_collect,
        input::DiscoveryInput,
        probe::exact_probe,
        targets::VAULT_CONFIG_TARGETS,
        walk::AncestorEnumerator,
    },
};

pub(crate) struct Init;
pub(crate) struct LocalCollected;
pub(crate) struct GlobalCollected;

pub(crate) struct DiscoveryProcessor<State> {
    input: DiscoveryInput,
    local: Vec<CandidatePath>,
    global: Vec<CandidatePath>,
    report: DiscoveryReport,
    _state: PhantomData<State>,
}

impl DiscoveryProcessor<Init> {
    pub(crate) fn new(input: DiscoveryInput) -> Self {
        Self {
            input,
            local: Vec::new(),
            global: Vec::new(),
            report: DiscoveryReport::default(),
            _state: PhantomData,
        }
    }

    pub(crate) fn collect_local(
        mut self,
    ) -> Result<DiscoveryProcessor<LocalCollected>, DiscoveryError> {
        // Ceiling segments dropped during input normalization are surfaced as
        // report diagnostics regardless of which local branch runs.
        self.report.skipped_ceilings = self.input.skipped_ceilings().to_vec();

        // An explicit `--vault` flag overrides the start directory for the
        // ascending walk. Both paths fall back to the default vault when
        // they find no local marker.
        let start =
            self.input.flag_vault().unwrap_or_else(|| self.input.anchor());
        let mut walk =
            AncestorEnumerator::new(start, self.input.ceiling_dirs());
        self.local = walk
            .by_ref()
            .flat_map(|dir| exact_probe(&dir, VAULT_CONFIG_TARGETS))
            .collect();
        self.report.local_traversal_stop_reason = walk.stop_reason().clone();

        if self.local.is_empty()
            && let Some(vault) = self.input.env_default_vault()?
        {
            self.local = exact_probe(&vault, VAULT_CONFIG_TARGETS);
        }

        Ok(DiscoveryProcessor {
            input: self.input,
            local: self.local,
            global: self.global,
            report: self.report,
            _state: PhantomData,
        })
    }
}

impl DiscoveryProcessor<LocalCollected> {
    pub(crate) fn collect_global(
        mut self,
    ) -> DiscoveryProcessor<GlobalCollected> {
        if self.input.suppress_global() {
            self.report.global_resolution_skip_reason = Some(
                crate::report::GlobalResolutionSkipReason::SuppressedByFlag,
            );
        }

        let platform_dirs = platform_global_dirs();
        self.global = global_collect(
            self.input.suppress_global(),
            self.input.flag_global(),
            self.input.env_global(),
            &platform_dirs,
        );

        DiscoveryProcessor {
            input: self.input,
            local: self.local,
            global: self.global,
            report: self.report,
            _state: PhantomData,
        }
    }
}

impl DiscoveryProcessor<GlobalCollected> {
    pub(crate) fn finish(self) -> DiscoveryOutcome {
        // ponytail: ignored trust-store paths are not wired into this slice
        // yet; pass tracked ignored paths here when trust is
        // integrated.
        DiscoveryOutcome::new(
            filter_ignored(dedupe_keep_last(self.local), &[])
                .into_boxed_slice(),
            filter_ignored(dedupe(self.global), &[]).into_boxed_slice(),
            self.report,
        )
    }
}

fn platform_global_dirs() -> Vec<DirPath> {
    let mut dirs = Vec::new();
    push_dir(&mut dirs, crate::dirs::CONFIG.clone());
    #[cfg(unix)]
    push_dir(&mut dirs, crate::dirs::SYSTEM_CONFIG.clone());
    dirs
}

fn push_dir(dirs: &mut Vec<DirPath>, path: PathBuf) {
    if let Ok(dir) = DirPath::try_new(path) {
        dirs.push(dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DiscoveryOptions, env_var::SettingsEnvVars};

    fn input(
        anchor: PathBuf,
        env_default_vault: Option<PathBuf>,
    ) -> DiscoveryInput {
        DiscoveryInput::from_options(
            &DiscoveryOptions::new(anchor, None, None, false),
            &SettingsEnvVars::new(env_default_vault, None, None, None, false),
        )
        .unwrap()
    }

    fn input_with_flag_vault(
        anchor: PathBuf,
        flag_vault: PathBuf,
        env_default_vault: Option<PathBuf>,
    ) -> DiscoveryInput {
        DiscoveryInput::from_options(
            &DiscoveryOptions::new(anchor, None, Some(flag_vault), false),
            &SettingsEnvVars::new(env_default_vault, None, None, None, false),
        )
        .expect("valid discovery input")
    }

    fn input_with_suppressed_global(anchor: PathBuf) -> DiscoveryInput {
        DiscoveryInput::from_options(
            &DiscoveryOptions::new(anchor, None, None, true),
            &SettingsEnvVars::new(None, None, None, None, false),
        )
        .expect("valid discovery input")
    }

    fn input_with_ceilings(
        anchor: PathBuf,
        ceilings: Vec<PathBuf>,
    ) -> DiscoveryInput {
        DiscoveryInput::from_options(
            &DiscoveryOptions::new(anchor, None, None, false),
            &SettingsEnvVars::new(None, None, None, Some(ceilings), false),
        )
        .expect("valid discovery input")
    }

    mod report {
        use pretty_assertions::assert_eq;

        use super::*;
        use crate::report::{
            GlobalResolutionSkipReason, LocalTraversalStopReason,
            SkippedCeiling, SkippedCeilingReason,
        };

        #[test]
        fn report_records_suppressed_global_resolution() {
            let root = tempfile::tempdir().expect("root");

            let outcome = DiscoveryProcessor::new(
                input_with_suppressed_global(root.path().to_path_buf()),
            )
            .collect_local()
            .expect("local collection")
            .collect_global()
            .finish();

            assert_eq!(
                outcome.report().global_resolution_skip_reason,
                Some(GlobalResolutionSkipReason::SuppressedByFlag)
            );
        }

        #[test]
        fn records_ceiling_enforced_stop_reason() {
            let root = tempfile::tempdir().expect("root");
            let ceiling = root.path().canonicalize().expect("canonical root");
            let anchor = ceiling.join("a").join("b");
            std::fs::create_dir_all(&anchor).expect("anchor");

            let outcome =
                DiscoveryProcessor::new(input_with_ceilings(anchor, vec![
                    ceiling.clone(),
                ]))
                .collect_local()
                .expect("local collection")
                .collect_global()
                .finish();

            assert_eq!(
                outcome.report().local_traversal_stop_reason,
                LocalTraversalStopReason::CeilingEnforced {
                    ceiling
                }
            );
        }

        #[test]
        fn records_project_boundary_marker_stop_reason() {
            let root = tempfile::tempdir().expect("root");
            let repo = root.path().join("repo");
            let anchor = repo.join("a").join("b");
            std::fs::create_dir_all(&anchor).expect("anchor");
            std::fs::create_dir(repo.join(".git")).expect("git marker");

            let outcome = DiscoveryProcessor::new(input(anchor, None))
                .collect_local()
                .expect("local collection")
                .collect_global()
                .finish();

            assert_eq!(
                outcome.report().local_traversal_stop_reason,
                LocalTraversalStopReason::ProjectBoundaryMarker {
                    marker: repo.join(".git")
                }
            );
        }

        #[test]
        fn records_filesystem_root_stop_reason_by_default() {
            let root = tempfile::tempdir().expect("root");

            let outcome =
                DiscoveryProcessor::new(input(root.path().into(), None))
                    .collect_local()
                    .expect("local collection")
                    .collect_global()
                    .finish();

            assert_eq!(
                outcome.report().local_traversal_stop_reason,
                LocalTraversalStopReason::FilesystemRoot
            );
        }

        #[test]
        fn propagates_skipped_ceilings_from_input() {
            let root = tempfile::tempdir().expect("root");
            let missing = PathBuf::from("/definitely/not/a/real/ceiling");

            let outcome = DiscoveryProcessor::new(input_with_ceilings(
                root.path().to_path_buf(),
                vec![missing.clone()],
            ))
            .collect_local()
            .expect("local collection")
            .collect_global()
            .finish();

            assert_eq!(outcome.report().skipped_ceilings, [SkippedCeiling {
                segment: missing,
                reason: SkippedCeilingReason::InvalidPath,
            }]);
        }
    }

    mod state {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn transitions_are_explicit_and_linear() {
            let root = tempfile::tempdir().expect("root");
            let init = DiscoveryProcessor::new(input(root.path().into(), None));

            let local = init.collect_local().unwrap();
            let global = local.collect_global();
            let outcome = global.finish();

            assert!(outcome.vault().is_empty());
            assert!(outcome.global().is_empty());
            assert_eq!(
                outcome.report(),
                &crate::discovery::report::DiscoveryReport::default()
            );
        }
    }

    mod finish {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn returns_outcome_with_local_outer_to_nearest_ordering() {
            let root = tempfile::tempdir().expect("root");
            let outer = root.path().join("outer");
            let inner = outer.join("inner");
            std::fs::create_dir_all(&inner).expect("inner");
            std::fs::write(outer.join("traces.toml"), "")
                .expect("outer marker");
            std::fs::write(inner.join("traces.toml"), "")
                .expect("inner marker");

            let outcome = DiscoveryProcessor::new(input(inner, None))
                .collect_local()
                .unwrap()
                .collect_global()
                .finish();

            let paths: Vec<_> = outcome
                .vault()
                .iter()
                .map(|candidate| candidate.path().as_path().to_path_buf())
                .collect();
            assert_eq!(paths, vec![
                outer.join("traces.toml"),
                root.path().join("outer/inner/traces.toml"),
            ]);
        }

        #[test]
        fn ignores_invalid_default_vault_when_local_marker_exists() {
            let root = tempfile::tempdir().expect("root");
            let config = root.path().join("traces.toml");
            std::fs::write(&config, "").expect("config");

            let outcome = DiscoveryProcessor::new(input(
                root.path().into(),
                Some(root.path().join("missing-vault")),
            ))
            .collect_local()
            .unwrap()
            .collect_global()
            .finish();

            assert_eq!(
                outcome
                    .vault()
                    .first()
                    .map(|candidate| candidate.path().as_path()),
                Some(config.as_path())
            );
        }

        #[test]
        fn uses_default_vault_only_when_local_collection_is_empty() {
            let root = tempfile::tempdir().expect("root");
            let anchor = root.path().join("anchor");
            let fallback = root.path().join("fallback");
            std::fs::create_dir_all(&anchor).expect("anchor");
            std::fs::create_dir_all(&fallback).expect("fallback");
            std::fs::write(fallback.join("traces.toml"), "")
                .expect("fallback marker");

            let outcome =
                DiscoveryProcessor::new(input(anchor, Some(fallback.clone())))
                    .collect_local()
                    .unwrap()
                    .collect_global()
                    .finish();

            assert_eq!(
                outcome
                    .vault()
                    .first()
                    .map(|candidate| candidate.base().as_path()),
                Some(fallback.as_path())
            );
        }

        #[test]
        fn flag_vault_starts_outer_to_nearest_ancestor_collection() {
            let root = tempfile::tempdir().expect("root");
            let outer = root.path().join("outer");
            let flag_vault = outer.join("inner");
            let anchor = root.path().join("ignored-anchor");
            std::fs::create_dir_all(&flag_vault).expect("flag vault");
            std::fs::create_dir_all(&anchor).expect("anchor");
            std::fs::write(outer.join("traces.toml"), "")
                .expect("outer marker");
            std::fs::write(flag_vault.join("traces.toml"), "")
                .expect("inner marker");

            let outcome = DiscoveryProcessor::new(input_with_flag_vault(
                anchor,
                flag_vault.clone(),
                None,
            ))
            .collect_local()
            .expect("local collection")
            .collect_global()
            .finish();

            let paths: Vec<_> = outcome
                .vault()
                .iter()
                .map(|candidate| candidate.path().as_path().to_path_buf())
                .collect();

            assert_eq!(paths, vec![
                outer.join("traces.toml"),
                flag_vault.join("traces.toml")
            ]);
        }

        #[cfg(unix)]
        #[test]
        fn nearest_candidate_wins_when_config_reachable_from_two_ancestors() {
            // Layout:
            //   root/real/traces.toml         (the physical config)
            //   root/real/child               (anchor's parent)
            //   root/link -> root/real        (symlinked ancestor)
            //   root/link/child               (== root/real/child via symlink)
            // Walking up from root/link/child yields link/child, link, root.
            // `link/traces.toml` and `real/traces.toml` share a canonical key;
            // the nearest occurrence (deepest ancestor) must survive.
            let root = tempfile::tempdir().expect("root");
            let real = root.path().join("real");
            let child = real.join("child");
            std::fs::create_dir_all(&child).expect("child");
            std::fs::write(real.join("traces.toml"), "").expect("marker");
            let link = root.path().join("link");
            std::os::unix::fs::symlink(&real, &link).expect("dir symlink");

            let anchor = link.join("child");
            let outcome = DiscoveryProcessor::new(input(anchor, None))
                .collect_local()
                .expect("local collection")
                .collect_global()
                .finish();

            // Exactly one candidate survives dedupe (same canonical target).
            assert_eq!(outcome.vault().len(), 1);
            // The surviving candidate is the nearest ancestor's view (link),
            // not the outer real path.
            assert_eq!(
                outcome.vault().first().map(|c| c.path().as_path()),
                Some(link.join("traces.toml").as_path())
            );
        }

        #[test]
        fn flag_vault_uses_default_vault_when_collection_is_empty() {
            let root = tempfile::tempdir().expect("root");
            let anchor = root.path().join("anchor");
            let flag_vault = root.path().join("flag-vault");
            let fallback = root.path().join("fallback");
            std::fs::create_dir_all(&anchor).expect("anchor");
            std::fs::create_dir_all(&flag_vault).expect("flag vault");
            std::fs::create_dir_all(&fallback).expect("fallback");
            std::fs::write(fallback.join("traces.toml"), "")
                .expect("fallback marker");

            let outcome = DiscoveryProcessor::new(input_with_flag_vault(
                anchor,
                flag_vault,
                Some(fallback.clone()),
            ))
            .collect_local()
            .expect("local collection")
            .collect_global()
            .finish();

            assert_eq!(
                outcome
                    .vault()
                    .first()
                    .map(|candidate| candidate.base().as_path()),
                Some(fallback.as_path())
            );
        }
    }
}
