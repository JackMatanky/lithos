//! Linear internal discovery processor.

use std::{marker::PhantomData, path::PathBuf};

use traces_fs::DirPath;

use crate::{
    candidate::CandidatePath,
    discovery::{
        error::DiscoveryError,
        filter::{dedupe, filter_ignored},
        global::global_collect,
        input::DiscoveryInput,
        outcome::DiscoveryOutcome,
        probe::exact_probe,
        walk::AncestorEnumerator,
    },
    location::MARKERS,
    os_dirs::XDG_CONFIG_HOME,
};

pub(crate) struct Init;
pub(crate) struct LocalCollected;
pub(crate) struct GlobalCollected;

pub(crate) struct DiscoveryProcessor<State> {
    input: DiscoveryInput,
    local: Vec<CandidatePath>,
    global: Vec<CandidatePath>,
    _state: PhantomData<State>,
}

impl DiscoveryProcessor<Init> {
    pub(crate) fn new(input: DiscoveryInput) -> Self {
        Self {
            input,
            local: Vec::new(),
            global: Vec::new(),
            _state: PhantomData,
        }
    }

    pub(crate) fn collect_local(
        mut self,
    ) -> Result<DiscoveryProcessor<LocalCollected>, DiscoveryError> {
        if let Some(vault) = self.input.flag_vault() {
            self.local = exact_probe(vault, MARKERS);
        } else {
            self.local = AncestorEnumerator::new(
                self.input.anchor(),
                self.input.ceiling_dirs(),
            )
            .flat_map(|dir| exact_probe(&dir, MARKERS))
            .collect();

            if self.local.is_empty()
                && let Some(vault) = self.input.env_default_vault()?
            {
                self.local = exact_probe(&vault, MARKERS);
            }
        }

        Ok(DiscoveryProcessor {
            input: self.input,
            local: self.local,
            global: self.global,
            _state: PhantomData,
        })
    }
}

impl DiscoveryProcessor<LocalCollected> {
    pub(crate) fn collect_global(
        mut self,
    ) -> DiscoveryProcessor<GlobalCollected> {
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
            filter_ignored(dedupe(self.local), &[]).into_boxed_slice(),
            filter_ignored(dedupe(self.global), &[]).into_boxed_slice(),
            // ponytail: report detail wiring stays with old discovery until
            // diagnostics migrate; keep the public shape now.
            crate::DiscoveryReport::default(),
        )
    }
}

fn platform_global_dirs() -> Vec<DirPath> {
    let mut dirs = Vec::new();
    push_dir(&mut dirs, XDG_CONFIG_HOME.join("traces"));
    #[cfg(unix)]
    push_dir(&mut dirs, PathBuf::from("/etc/traces"));
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
    use crate::{DiscoveryOptions, SettingsEnvVars};

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

    mod state {
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
            assert_eq!(outcome.report(), &crate::DiscoveryReport::default());
        }
    }

    mod finish {
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
    }
}
