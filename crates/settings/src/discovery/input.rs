//! Normalized inputs for the internal discovery pipeline.

use std::path::{Path, PathBuf};

use traces_fs::{DirPath, FilePath};

use crate::{
    DiscoveryOptions,
    discovery::{
        error::{DiscoveryError, EnvironmentOverrideError, FlagOverrideError},
        report::{SkippedCeiling, SkippedCeilingReason},
    },
    env_var::SettingsEnvVars,
    os_dirs::HOME,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DiscoveryInput {
    anchor: DirPath,
    flag_global: Option<FilePath>,
    flag_vault: Option<DirPath>,
    env_global: Option<FilePath>,
    env_default_vault: Option<PathBuf>,
    ceiling_dirs: Box<[PathBuf]>,
    skipped_ceilings: Vec<SkippedCeiling>,
    suppress_global: bool,
}

impl DiscoveryInput {
    pub(crate) fn from_options(
        options: &DiscoveryOptions,
        env: &SettingsEnvVars,
    ) -> Result<Self, DiscoveryError> {
        let anchor = normalize_anchor(options.anchor())?;
        let flag_vault =
            options.vault_dir().map(validate_flag_vault).transpose()?;
        let suppress_global =
            options.suppress_global() || env.suppress_global();
        let flag_global = if suppress_global {
            None
        } else {
            options.config_file().map(validate_flag_global).transpose()?
        };
        let env_global = if suppress_global {
            None
        } else {
            env.global_config().map(validate_env_global).transpose()?
        };
        let env_default_vault = env.default_vault_dir().map(Path::to_path_buf);
        let (ceiling_dirs, skipped_ceilings) =
            normalize_ceilings(env.ceiling_dirs().unwrap_or(&[]));

        Ok(Self {
            anchor,
            flag_global,
            flag_vault,
            env_global,
            env_default_vault,
            ceiling_dirs,
            skipped_ceilings,
            suppress_global,
        })
    }

    pub(crate) fn anchor(&self) -> &DirPath {
        &self.anchor
    }

    pub(crate) fn flag_global(&self) -> Option<&FilePath> {
        self.flag_global.as_ref()
    }

    pub(crate) fn flag_vault(&self) -> Option<&DirPath> {
        self.flag_vault.as_ref()
    }

    pub(crate) fn env_global(&self) -> Option<&FilePath> {
        self.env_global.as_ref()
    }

    pub(crate) fn env_default_vault(
        &self,
    ) -> Result<Option<DirPath>, EnvironmentOverrideError> {
        self.env_default_vault
            .as_ref()
            .map(|path| validate_env_default_vault(path))
            .transpose()
    }

    pub(crate) fn ceiling_dirs(&self) -> &[PathBuf] {
        &self.ceiling_dirs
    }

    pub(crate) fn skipped_ceilings(&self) -> &[SkippedCeiling] {
        &self.skipped_ceilings
    }

    pub(crate) fn suppress_global(&self) -> bool {
        self.suppress_global
    }
}

/// Normalize raw ceiling segments for lexical ancestor matching.
///
/// Applies `~/` → `$HOME` expansion (mirroring how `mise` normalizes ceiling
/// paths in `file::all_dirs`) and drops empty and non-existent segments. Paths
/// are intentionally **not** canonicalized: ancestor comparison in
/// [`AncestorEnumerator`](super::walk::AncestorEnumerator) is lexical, matching
/// `mise`'s behavior, so canonicalizing here would break matches against a
/// lexically-walked ancestor. Dropped segments are reported as
/// [`SkippedCeiling`] entries for diagnostics.
fn normalize_ceilings(
    raw: &[PathBuf],
) -> (Box<[PathBuf]>, Vec<SkippedCeiling>) {
    let mut kept = Vec::new();
    let mut skipped = Vec::new();

    for segment in raw {
        if segment.as_os_str().is_empty() {
            skipped.push(SkippedCeiling {
                segment: segment.clone(),
                reason: SkippedCeilingReason::EmptySegment,
            });
            continue;
        }

        let expanded = expand_home(segment);
        if expanded.is_dir() {
            kept.push(expanded);
        } else {
            skipped.push(SkippedCeiling {
                segment: segment.clone(),
                reason: SkippedCeilingReason::InvalidPath,
            });
        }
    }

    (kept.into_boxed_slice(), skipped)
}

/// Expand a leading `~/` to the user's home directory.
fn expand_home(path: &Path) -> PathBuf {
    match path.strip_prefix("~") {
        Ok(rest) => HOME.join(rest),
        Err(_) => path.to_path_buf(),
    }
}

fn normalize_anchor(path: &Path) -> Result<DirPath, DiscoveryError> {
    let dir = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    DirPath::try_new(dir.to_path_buf()).map_err(|source| {
        DiscoveryError::InvalidAnchorDirectory {
            path: path.to_path_buf(),
            source,
        }
    })
}

fn validate_flag_global(path: &Path) -> Result<FilePath, FlagOverrideError> {
    if !path.exists() {
        return Err(FlagOverrideError::GlobalConfigPathNotFound {
            path: path.to_path_buf(),
        });
    }
    FilePath::try_new(path.to_path_buf()).map_err(|source| {
        FlagOverrideError::GlobalConfigPathNotFile {
            path: path.to_path_buf(),
            source,
        }
    })
}

fn validate_flag_vault(path: &Path) -> Result<DirPath, FlagOverrideError> {
    if !path.exists() {
        return Err(FlagOverrideError::VaultPathNotFound {
            path: path.to_path_buf(),
        });
    }
    DirPath::try_new(path.to_path_buf()).map_err(|source| {
        FlagOverrideError::VaultPathNotDirectory {
            path: path.to_path_buf(),
            source,
        }
    })
}

fn validate_env_global(
    path: &Path,
) -> Result<FilePath, EnvironmentOverrideError> {
    if !path.exists() {
        return Err(EnvironmentOverrideError::GlobalConfigPathNotFound {
            path: path.to_path_buf(),
        });
    }
    FilePath::try_new(path.to_path_buf()).map_err(|source| {
        EnvironmentOverrideError::GlobalConfigPathNotFile {
            path: path.to_path_buf(),
            source,
        }
    })
}

fn validate_env_default_vault(
    path: &Path,
) -> Result<DirPath, EnvironmentOverrideError> {
    if !path.exists() {
        return Err(EnvironmentOverrideError::VaultPathNotFound {
            path: path.to_path_buf(),
        });
    }
    DirPath::try_new(path.to_path_buf()).map_err(|source| {
        EnvironmentOverrideError::VaultPathNotDirectory {
            path: path.to_path_buf(),
            source,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    mod constructor {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn from_options_merges_flag_and_env() {
            let root = tempfile::tempdir().expect("temp dir");
            let anchor = root.path().join("anchor");
            let flag_vault = root.path().join("flag-vault");
            let env_vault = root.path().join("env-vault");
            std::fs::create_dir_all(&anchor).expect("anchor");
            std::fs::create_dir_all(&flag_vault).expect("flag vault");
            std::fs::create_dir_all(&env_vault).expect("env vault");
            let flag_global = root.path().join("flag.toml");
            let env_global = root.path().join("env.toml");
            std::fs::write(&flag_global, "").expect("flag global");
            std::fs::write(&env_global, "").expect("env global");
            let ceiling = root.path().join("ceiling");
            std::fs::create_dir_all(&ceiling).expect("ceiling");

            let options = DiscoveryOptions::new(
                anchor.clone(),
                Some(flag_global.clone()),
                Some(flag_vault.clone()),
                false,
            );
            let env = SettingsEnvVars::new(
                Some(env_vault.clone()),
                Some(env_global.clone()),
                None,
                Some(vec![ceiling.clone()]),
                false,
            );

            let input = DiscoveryInput::from_options(&options, &env).unwrap();

            assert_eq!(input.anchor().as_path(), anchor.as_path());
            assert_eq!(
                input.flag_global().map(FilePath::as_path),
                Some(flag_global.as_path())
            );
            assert_eq!(
                input.flag_vault().map(DirPath::as_path),
                Some(flag_vault.as_path())
            );
            assert_eq!(
                input.env_global().map(FilePath::as_path),
                Some(env_global.as_path())
            );
            assert_eq!(
                input
                    .env_default_vault()
                    .unwrap()
                    .as_ref()
                    .map(DirPath::as_path),
                Some(env_vault.as_path())
            );
            assert_eq!(input.ceiling_dirs(), [ceiling].as_ref());
            assert!(!input.suppress_global());
        }
    }

    mod validation_precedence {
        use super::*;

        #[test]
        fn suppress_global_skips_invalid_env_global() {
            let root = tempfile::tempdir().expect("root");
            let options = DiscoveryOptions::new(
                root.path().to_path_buf(),
                None,
                None,
                true,
            );
            let env = SettingsEnvVars::new(
                None,
                Some(root.path().join("missing-global.toml")),
                None,
                None,
                false,
            );

            let input = DiscoveryInput::from_options(&options, &env).unwrap();

            assert!(input.env_global().is_none());
            assert!(input.suppress_global());
        }

        #[test]
        fn suppress_global_skips_invalid_flag_global() {
            let root = tempfile::tempdir().expect("root");
            let options = DiscoveryOptions::new(
                root.path().to_path_buf(),
                Some(root.path().join("missing-flag.toml")),
                None,
                true,
            );
            let env = SettingsEnvVars::new(None, None, None, None, false);

            let input = DiscoveryInput::from_options(&options, &env).unwrap();

            assert!(input.flag_global().is_none());
            assert!(input.suppress_global());
        }

        #[test]
        fn env_suppress_global_skips_invalid_flag_and_env_global() {
            let root = tempfile::tempdir().expect("root");
            let options = DiscoveryOptions::new(
                root.path().to_path_buf(),
                Some(root.path().join("missing-flag.toml")),
                None,
                false,
            );
            let env = SettingsEnvVars::new(
                None,
                Some(root.path().join("missing-env.toml")),
                None,
                None,
                true,
            );

            let input = DiscoveryInput::from_options(&options, &env).unwrap();

            assert!(input.flag_global().is_none());
            assert!(input.env_global().is_none());
            assert!(input.suppress_global());
        }

        #[test]
        fn invalid_env_default_vault_is_lazy() {
            let root = tempfile::tempdir().expect("root");
            let options = DiscoveryOptions::new(
                root.path().to_path_buf(),
                None,
                None,
                true,
            );
            let env = SettingsEnvVars::new(
                Some(root.path().join("missing-vault")),
                None,
                None,
                None,
                false,
            );

            let input = DiscoveryInput::from_options(&options, &env).unwrap();

            assert!(input.env_default_vault().is_err());
        }
    }

    mod accessors {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn anchor_file_normalizes_to_parent_directory() {
            let root = tempfile::tempdir().expect("temp dir");
            let anchor_file = root.path().join("note.md");
            std::fs::write(&anchor_file, "").expect("anchor file");
            let options = DiscoveryOptions::new(anchor_file, None, None, false);
            let env = SettingsEnvVars::new(None, None, None, None, false);

            let input = DiscoveryInput::from_options(&options, &env).unwrap();

            assert_eq!(input.anchor().as_path(), root.path());
        }
    }

    mod ceilings {
        use pretty_assertions::assert_eq;

        use super::*;

        fn input_with_ceilings(raw: Vec<PathBuf>) -> DiscoveryInput {
            let root = tempfile::tempdir().expect("root");
            let options = DiscoveryOptions::new(
                root.path().to_path_buf(),
                None,
                None,
                false,
            );
            let env = SettingsEnvVars::new(None, None, None, Some(raw), false);
            DiscoveryInput::from_options(&options, &env).expect("input")
        }

        #[test]
        fn keeps_existing_directory_segments() {
            let dir = tempfile::tempdir().expect("dir");
            let input = input_with_ceilings(vec![dir.path().to_path_buf()]);

            assert_eq!(input.ceiling_dirs(), [dir.path().to_path_buf()]);
            assert!(input.skipped_ceilings().is_empty());
        }

        #[test]
        fn records_empty_segment_as_skipped() {
            let input = input_with_ceilings(vec![PathBuf::new()]);

            assert!(input.ceiling_dirs().is_empty());
            assert_eq!(input.skipped_ceilings(), [SkippedCeiling {
                segment: PathBuf::new(),
                reason: SkippedCeilingReason::EmptySegment,
            }]);
        }

        #[test]
        fn records_nonexistent_segment_as_invalid() {
            let missing = PathBuf::from("/definitely/not/a/real/ceiling");
            let input = input_with_ceilings(vec![missing.clone()]);

            assert!(input.ceiling_dirs().is_empty());
            assert_eq!(input.skipped_ceilings(), [SkippedCeiling {
                segment: missing,
                reason: SkippedCeilingReason::InvalidPath,
            }]);
        }

        #[test]
        fn expands_tilde_prefix_to_home() {
            // Under test builds HOME is redirected to tests/fixtures; a
            // `~/<random>` path expands there and (not existing) is reported
            // as invalid, proving expansion occurred against HOME rather than
            // being treated as a literal "~" directory.
            let input = input_with_ceilings(vec![PathBuf::from(
                "~/traces-nonexistent-ceiling",
            )]);

            let expected = HOME.join("traces-nonexistent-ceiling");
            assert_eq!(
                input.skipped_ceilings(),
                [SkippedCeiling {
                    segment: PathBuf::from("~/traces-nonexistent-ceiling"),
                    reason: SkippedCeilingReason::InvalidPath,
                }],
                "tilde ceiling should expand to {} and be reported invalid",
                expected.display()
            );
        }

        #[test]
        fn expand_home_joins_home_for_tilde_paths() {
            assert_eq!(expand_home(Path::new("~/config")), HOME.join("config"));
            assert_eq!(
                expand_home(Path::new("/abs/path")),
                PathBuf::from("/abs/path")
            );
        }
    }
}
