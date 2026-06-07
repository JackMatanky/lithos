//! Orchestration engine for vault and global configuration discovery.
//!
//! This module provides the high-level API for locating configuration roots
//! based on policy-driven precedence (CLI flags, environment variables, and
//! filesystem convention).

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use super::{
    diagnostics::{DiscoveryWarning, VaultDiscoveryWarning},
    error::DiscoveryError,
    policy::{DiscoveryPolicy, GlobalSourceType, VaultSourceType},
    probe::{DiscoveryProbe, GlobalConfigProbe, VaultRootProbe},
    selector::select_candidate,
    walk::{BoundedAscent, DiscoveryBoundaries},
};
use crate::fs::format::StructuredFileFormat;

/// A root marker file found during vault root resolution.
///
/// Carries the canonicalized path to the marker file (e.g. `lithos.toml`) and
/// the base directory it was found in. Does not include Config location
/// taxonomy; that classification is Config-owned.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FoundRootMarker {
    /// Base directory the marker was found in (the vault root candidate).
    pub(crate) base: PathBuf,
    /// Absolute canonicalized path to the marker file.
    pub(crate) path: PathBuf,
    /// The detected structured format of the marker file.
    pub(crate) format: StructuredFileFormat,
}

/// Orchestrates the discovery of vault and global configuration markers.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) struct DiscoveryEngine {
    policy: DiscoveryPolicy,
}

#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
impl DiscoveryEngine {
    /// Create a new discovery engine with the given policy.
    pub(crate) fn new(policy: DiscoveryPolicy) -> Self {
        Self {
            policy,
        }
    }

    /// Find the vault root and marker based on policy precedence.
    ///
    /// Iterates through allowed sources (Flag -> Env -> Walk) until a root is
    /// successfully resolved or an error occurs.
    pub(crate) fn find_vault(
        &self,
        input: &DiscoveryInput<'_>,
    ) -> Result<VaultDiscoveryResult, DiscoveryError> {
        for source_type in &self.policy.precedence {
            match source_type {
                VaultSourceType::ExplicitFlag => {
                    if let Some(path) = input.flag_path {
                        let root = Self::validate_override(path, *source_type)?;
                        return Ok(VaultDiscoveryResult {
                            root: Some(root),
                            marker: None,
                            alternatives: vec![],
                            source: Some(VaultSourceType::ExplicitFlag),
                            warnings: vec![],
                        });
                    }
                }
                VaultSourceType::EnvVar => {
                    if let Some(path) = input.env_path {
                        let root = Self::validate_override(path, *source_type)?;
                        return Ok(VaultDiscoveryResult {
                            root: Some(root),
                            marker: None,
                            alternatives: vec![],
                            source: Some(VaultSourceType::EnvVar),
                            warnings: vec![],
                        });
                    }
                }
                VaultSourceType::AscendingWalk => {
                    return self.resolve_ascending(input);
                }
            }
        }

        Ok(VaultDiscoveryResult {
            root: None,
            marker: None,
            alternatives: vec![],
            source: None,
            warnings: vec![],
        })
    }

    /// Find the global configuration marker.
    #[allow(
        clippy::unused_self,
        reason = "global discovery uses fixed global source precedence"
    )]
    pub(crate) fn find_global(
        &self,
        input: &GlobalDiscoveryInput<'_>,
    ) -> Result<GlobalDiscoveryResult, DiscoveryError> {
        if input.suppress_global {
            return Ok(GlobalDiscoveryResult::default());
        }

        if let Some(path) = input.env_config_file
            && path.is_file()
        {
            let format = StructuredFileFormat::from_path(path);
            let Some(format) = format else {
                return Ok(GlobalDiscoveryResult::default());
            };
            let canonical = path.canonicalize().map_err(|source| {
                DiscoveryError::CanonicalizePath {
                    path: path.to_path_buf(),
                    source,
                }
            })?;
            // `path` is a file, so `.parent()` should always return a valid
            // directory. A root-component path (e.g. `/`) has no parent and
            // cannot be a regular file, so `None` here indicates a broken
            // invariant; return a canonicalization error rather than silently
            // falling back to an empty base which would corrupt later path
            // resolution.
            let base = path.parent().ok_or_else(|| {
                DiscoveryError::CanonicalizePath {
                    path: path.to_path_buf(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "LITHOS_CONFIG_FILE path has no parent directory",
                    ),
                }
            })?;

            return Ok(GlobalDiscoveryResult {
                marker: Some(FoundRootMarker {
                    base: base.to_path_buf(),
                    path: canonical,
                    format,
                }),
                alternatives: vec![],
                source: Some(GlobalSourceType::EnvVar),
                warnings: vec![],
            });
        }

        for source in GlobalSourceType::PRECEDENCE {
            let base = match source {
                GlobalSourceType::EnvVar => continue,
                GlobalSourceType::XdgConfig => input.xdg_config_base,
                GlobalSourceType::UserConfig => input.user_config_base,
                GlobalSourceType::SystemConfig => input.system_config_base,
            };
            let Some(base) = base else {
                continue;
            };
            let mut warnings = Vec::new();
            let Some(markers) =
                GlobalConfigProbe::probe_with_warnings(base, &mut warnings)?
            else {
                continue;
            };
            let Some(selected) = select_candidate(&markers).cloned() else {
                continue;
            };
            let selected_path = selected.path.clone();
            let alternatives = markers
                .into_iter()
                .filter(|marker| marker.path != selected_path)
                .collect();

            return Ok(GlobalDiscoveryResult {
                marker: Some(selected),
                alternatives,
                source: Some(source),
                warnings,
            });
        }

        Ok(GlobalDiscoveryResult::default())
    }

    /// Performs an ascending walk from the current working directory to find a
    /// marker.
    fn resolve_ascending(
        &self,
        input: &DiscoveryInput<'_>,
    ) -> Result<VaultDiscoveryResult, DiscoveryError> {
        let start = input.cwd.canonicalize().map_err(|source| {
            DiscoveryError::CurrentDirectoryCanonicalize {
                path: input.cwd.to_path_buf(),
                source,
            }
        })?;

        let mut warnings: Vec<VaultDiscoveryWarning> = Vec::new();
        let ceilings = DiscoveryBoundaries::parse_ceilings(
            input.ceiling_dirs_raw,
            &mut warnings,
        );

        let walker = BoundedAscent::new(
            &start,
            &ceilings,
            self.policy.allow_marker_at_ceiling,
        );
        let probe = VaultRootProbe;
        let mut all_markers: Vec<FoundRootMarker> = Vec::new();
        let mut found_root: Option<PathBuf> = None;

        for current in walker {
            if let Some(mut markers) = probe.probe(current)? {
                found_root = Some(current.to_path_buf());
                all_markers.append(&mut markers);
                break;
            }
        }

        let selected = select_candidate(&all_markers).cloned();
        let (marker, alternatives) = match selected {
            Some(ref selected) => {
                let selected_path = selected.path.clone();
                let alts: Vec<FoundRootMarker> = all_markers
                    .into_iter()
                    .filter(|m| m.path != selected_path)
                    .collect();
                (Some(selected.clone()), alts)
            }
            None => (None, vec![]),
        };

        let source =
            found_root.as_ref().map(|_| VaultSourceType::AscendingWalk);

        Ok(VaultDiscoveryResult {
            root: found_root,
            marker,
            alternatives,
            source,
            warnings,
        })
    }

    /// Validates that a path provided via flag or environment variable is a
    /// directory.
    fn validate_override(
        path: &Path,
        source: VaultSourceType,
    ) -> Result<PathBuf, DiscoveryError> {
        if !path.exists() {
            return Err(match source {
                VaultSourceType::ExplicitFlag => {
                    DiscoveryError::ExplicitPathMissing {
                        path: path.to_path_buf(),
                    }
                }
                VaultSourceType::EnvVar => {
                    DiscoveryError::EnvironmentPathMissing {
                        path: path.to_path_buf(),
                    }
                }
                VaultSourceType::AscendingWalk => {
                    DiscoveryError::CanonicalizePath {
                        path: path.to_path_buf(),
                        source: std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "ascending discovery path is missing",
                        ),
                    }
                }
            });
        }

        if !path.is_dir() {
            return Err(match source {
                VaultSourceType::ExplicitFlag => {
                    DiscoveryError::ExplicitPathNotDirectory {
                        path: path.to_path_buf(),
                    }
                }
                VaultSourceType::EnvVar => {
                    DiscoveryError::EnvironmentPathNotDirectory {
                        path: path.to_path_buf(),
                    }
                }
                VaultSourceType::AscendingWalk => {
                    DiscoveryError::CanonicalizePath {
                        path: path.to_path_buf(),
                        source: std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "ascending discovery path is not a directory",
                        ),
                    }
                }
            });
        }

        path.canonicalize().map_err(|error| DiscoveryError::CanonicalizePath {
            path: path.to_path_buf(),
            source: error,
        })
    }
}

/// Collection of inputs required for the discovery process.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) struct DiscoveryInput<'a> {
    /// Path provided via `--vault` CLI flag.
    pub(crate) flag_path: Option<&'a Path>,
    /// Path provided via `LITHOS_VAULT` environment variable.
    pub(crate) env_path: Option<&'a Path>,
    /// Current working directory to start ascending discovery from.
    pub(crate) cwd: &'a Path,
    /// Raw ceiling directory string (platform-specific separator).
    pub(crate) ceiling_dirs_raw: Option<&'a OsStr>,
}

/// Inputs for global configuration discovery.
///
/// `env_config_file` is a direct **file path**; the `*_base` fields are
/// **directories** probed for `lithos.{toml,json,yaml,yml}`.
#[allow(dead_code, reason = "Phase-2 seam; wired in once orchestration lands")]
pub(crate) struct GlobalDiscoveryInput<'a> {
    /// Full file path from `LITHOS_CONFIG_FILE` (not a directory).
    pub(crate) env_config_file: Option<&'a Path>,
    /// XDG base directory, e.g. `$XDG_CONFIG_HOME/lithos`.
    pub(crate) xdg_config_base: Option<&'a Path>,
    /// User config base directory, e.g. `~/.config/lithos`.
    pub(crate) user_config_base: Option<&'a Path>,
    /// System config base directory, e.g. `/etc/lithos`.
    pub(crate) system_config_base: Option<&'a Path>,
    /// Skip all global lookup and return empty; corresponds to
    /// `--no-global-config`.
    pub(crate) suppress_global: bool,
}

/// The result of a successful or partially-successful vault discovery.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug)]
pub(crate) struct VaultDiscoveryResult {
    /// The resolved vault root directory.
    pub(crate) root: Option<PathBuf>,
    /// The highest-precedence marker file found.
    pub(crate) marker: Option<FoundRootMarker>,
    /// Other marker files found in the same root (e.g. different formats).
    pub(crate) alternatives: Vec<FoundRootMarker>,
    /// Which source established the root.
    pub(crate) source: Option<VaultSourceType>,
    /// Non-fatal warnings encountered during discovery.
    pub(crate) warnings: Vec<VaultDiscoveryWarning>,
}

/// The result of a global configuration discovery.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct GlobalDiscoveryResult {
    /// The discovered global marker file.
    pub(crate) marker: Option<FoundRootMarker>,
    /// Alternative formats of the global marker.
    pub(crate) alternatives: Vec<FoundRootMarker>,
    /// Which source established the global config.
    pub(crate) source: Option<GlobalSourceType>,
    /// Non-fatal warnings encountered during discovery.
    pub(crate) warnings: Vec<DiscoveryWarning>,
}

#[cfg(test)]
mod tests {
    use std::{env, ffi::OsString};

    use tempfile::tempdir;

    use super::*;

    fn engine() -> DiscoveryEngine {
        DiscoveryEngine::new(DiscoveryPolicy::default())
    }

    fn engine_no_ceiling_markers() -> DiscoveryEngine {
        DiscoveryEngine::new(DiscoveryPolicy {
            allow_marker_at_ceiling: false,
            ..DiscoveryPolicy::default()
        })
    }

    fn input_from_cwd(cwd: &Path) -> DiscoveryInput<'_> {
        DiscoveryInput {
            flag_path: None,
            env_path: None,
            cwd,
            ceiling_dirs_raw: None,
        }
    }

    fn input_with_ceilings<'a>(
        cwd: &'a Path,
        ceilings: &'a OsString,
    ) -> DiscoveryInput<'a> {
        DiscoveryInput {
            flag_path: None,
            env_path: None,
            cwd,
            ceiling_dirs_raw: Some(ceilings.as_os_str()),
        }
    }

    fn write_marker(root: &Path, relative: &str) -> PathBuf {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create marker parent dir");
        }
        std::fs::write(&path, "").expect("write marker file");
        path
    }

    fn path_list(paths: &[&Path]) -> OsString {
        env::join_paths(paths.iter().copied()).expect("join paths")
    }

    mod find_vault {
        use super::*;

        #[test]
        fn returns_explicit_source_when_explicit_path_exists() {
            let explicit = tempdir().expect("explicit dir");
            let env_dir = tempdir().expect("env dir");
            let cwd = tempdir().expect("cwd");

            let result = engine()
                .find_vault(&DiscoveryInput {
                    flag_path: Some(explicit.path()),
                    env_path: Some(env_dir.path()),
                    cwd: cwd.path(),
                    ceiling_dirs_raw: None,
                })
                .expect("resolution succeeds");

            assert_eq!(result.source, Some(VaultSourceType::ExplicitFlag));
        }

        #[test]
        fn returns_explicit_root_when_explicit_path_exists() {
            let explicit = tempdir().expect("explicit dir");
            let env_dir = tempdir().expect("env dir");
            let cwd = tempdir().expect("cwd");

            let result = engine()
                .find_vault(&DiscoveryInput {
                    flag_path: Some(explicit.path()),
                    env_path: Some(env_dir.path()),
                    cwd: cwd.path(),
                    ceiling_dirs_raw: None,
                })
                .expect("resolution succeeds");

            assert_eq!(
                result.root,
                Some(
                    explicit.path().canonicalize().expect("canonical explicit")
                )
            );
        }

        #[test]
        fn returns_environment_source_when_explicit_path_is_absent() {
            let env_dir = tempdir().expect("env dir");
            let cwd = tempdir().expect("cwd");

            let result = engine()
                .find_vault(&DiscoveryInput {
                    flag_path: None,
                    env_path: Some(env_dir.path()),
                    cwd: cwd.path(),
                    ceiling_dirs_raw: None,
                })
                .expect("resolution succeeds");

            assert_eq!(result.source, Some(VaultSourceType::EnvVar));
        }

        #[test]
        fn returns_environment_root_when_environment_path_exists() {
            let env_dir = tempdir().expect("env dir");
            let cwd = tempdir().expect("cwd");

            let result = engine()
                .find_vault(&DiscoveryInput {
                    flag_path: None,
                    env_path: Some(env_dir.path()),
                    cwd: cwd.path(),
                    ceiling_dirs_raw: None,
                })
                .expect("resolution succeeds");

            assert_eq!(
                result.root,
                Some(env_dir.path().canonicalize().expect("canonical env"))
            );
        }

        #[test]
        fn prefers_explicit_path_over_environment_path() {
            let explicit = tempdir().expect("explicit dir");
            let env_dir = tempdir().expect("env dir");
            let cwd = tempdir().expect("cwd");

            let result = engine()
                .find_vault(&DiscoveryInput {
                    flag_path: Some(explicit.path()),
                    env_path: Some(env_dir.path()),
                    cwd: cwd.path(),
                    ceiling_dirs_raw: None,
                })
                .expect("resolution succeeds");

            assert_eq!(result.source, Some(VaultSourceType::ExplicitFlag));
        }

        #[test]
        fn returns_ascending_source_when_marker_is_found() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.toml");

            let result = engine()
                .find_vault(&input_from_cwd(root.path()))
                .expect("resolution succeeds");

            assert_eq!(result.source, Some(VaultSourceType::AscendingWalk));
        }

        #[test]
        fn returns_root_when_marker_exists_in_cwd() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.toml");

            let result = engine()
                .find_vault(&input_from_cwd(root.path()))
                .expect("resolution succeeds");

            assert_eq!(
                result.root,
                Some(root.path().canonicalize().expect("canonical root"))
            );
        }

        #[test]
        fn returns_root_when_marker_exists_in_ancestor() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.toml");

            let child = root.path().join("a").join("b");
            std::fs::create_dir_all(&child).expect("child");

            let result = engine()
                .find_vault(&DiscoveryInput {
                    flag_path: None,
                    env_path: None,
                    cwd: &child,
                    ceiling_dirs_raw: None,
                })
                .expect("resolution succeeds");

            assert_eq!(
                result.root,
                Some(root.path().canonicalize().expect("canonical root"))
            );
        }

        #[test]
        fn returns_marker_when_marker_exists_in_ancestor() {
            let root = tempdir().expect("root");
            let marker_path = write_marker(root.path(), "lithos.toml");

            let child = root.path().join("a").join("b");
            std::fs::create_dir_all(&child).expect("child");

            let result = engine()
                .find_vault(&input_from_cwd(&child))
                .expect("resolution succeeds");
            let marker = result.marker.expect("marker is discovered");

            assert_eq!(
                marker.path,
                marker_path.canonicalize().expect("canonical marker")
            );
        }

        #[test]
        fn returns_none_when_no_marker_exists() {
            let root = tempdir().expect("root");

            let result = engine()
                .find_vault(&input_from_cwd(root.path()))
                .expect("resolution succeeds");

            assert_eq!(result.root, None);
        }

        #[test]
        fn returns_none_when_ceiling_stops_before_marker() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.toml");

            let stop = root.path().join("level1");
            let cwd = stop.join("level2");
            std::fs::create_dir_all(&cwd).expect("cwd");

            let ceilings = path_list(&[&stop]);
            let result = engine()
                .find_vault(&input_with_ceilings(&cwd, &ceilings))
                .expect("resolution succeeds");

            assert_eq!(result.root, None);
        }

        #[test]
        fn returns_marker_at_ceiling_when_policy_allows_it() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.toml");

            let cwd = root.path().join("nested");
            std::fs::create_dir_all(&cwd).expect("cwd");

            let ceilings = path_list(&[root.path()]);
            let result = engine()
                .find_vault(&input_with_ceilings(&cwd, &ceilings))
                .expect("resolution succeeds");

            assert!(result.marker.is_some());
        }

        #[test]
        fn returns_none_at_ceiling_when_policy_rejects_it() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.toml");

            let cwd = root.path().join("nested");
            std::fs::create_dir_all(&cwd).expect("cwd");

            let ceilings = path_list(&[root.path()]);
            let result = engine_no_ceiling_markers()
                .find_vault(&input_with_ceilings(&cwd, &ceilings))
                .expect("resolution succeeds");

            assert_eq!(result.root, None);
        }

        #[test]
        fn returns_error_when_explicit_path_is_missing() {
            let cwd = tempdir().expect("cwd");
            let missing = cwd.path().join("missing");

            let error = engine()
                .find_vault(&DiscoveryInput {
                    flag_path: Some(&missing),
                    env_path: None,
                    cwd: cwd.path(),
                    ceiling_dirs_raw: None,
                })
                .expect_err("missing explicit path should fail");

            assert_eq!(
                error.to_string(),
                format!(
                    "Explicit vault path does not exist: {}",
                    missing.display()
                ),
            );
        }

        #[test]
        fn returns_error_when_explicit_path_is_file() {
            let cwd = tempdir().expect("cwd");
            let file_path = cwd.path().join("file.txt");
            std::fs::write(&file_path, "x").expect("write file");

            let error = engine()
                .find_vault(&DiscoveryInput {
                    flag_path: Some(&file_path),
                    env_path: None,
                    cwd: cwd.path(),
                    ceiling_dirs_raw: None,
                })
                .expect_err("explicit file path should fail");

            assert_eq!(
                error.to_string(),
                format!(
                    "Explicit vault path is not a directory: {}",
                    file_path.display()
                ),
            );
        }

        #[test]
        fn returns_error_when_environment_path_is_missing() {
            let cwd = tempdir().expect("cwd");
            let missing = cwd.path().join("missing");

            let error = engine()
                .find_vault(&DiscoveryInput {
                    flag_path: None,
                    env_path: Some(&missing),
                    cwd: cwd.path(),
                    ceiling_dirs_raw: None,
                })
                .expect_err("missing environment path should fail");

            assert_eq!(
                error.to_string(),
                format!(
                    "Environment vault path does not exist: {}",
                    missing.display()
                ),
            );
        }

        #[test]
        fn returns_error_when_environment_path_is_file() {
            let cwd = tempdir().expect("cwd");
            let file_path = cwd.path().join("file.txt");
            std::fs::write(&file_path, "x").expect("write file");

            let error = engine()
                .find_vault(&DiscoveryInput {
                    flag_path: None,
                    env_path: Some(&file_path),
                    cwd: cwd.path(),
                    ceiling_dirs_raw: None,
                })
                .expect_err("env file path should fail");

            assert_eq!(
                error.to_string(),
                format!(
                    "Environment vault path is not a directory: {}",
                    file_path.display()
                ),
            );
        }

        #[test]
        fn returns_warnings_when_segments_are_mixed() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.toml");

            let stop = root.path().join("stop");
            let cwd = stop.join("nested");
            std::fs::create_dir_all(&cwd).expect("cwd");

            let raw = env::join_paths([
                OsString::from(""),
                OsString::from("   "),
                stop.as_os_str().to_os_string(),
                root.path().join("missing").as_os_str().to_os_string(),
            ])
            .expect("join paths");

            let result = engine()
                .find_vault(&DiscoveryInput {
                    flag_path: None,
                    env_path: None,
                    cwd: &cwd,
                    ceiling_dirs_raw: Some(raw.as_os_str()),
                })
                .expect("resolution succeeds");

            assert!(
                result
                    .warnings
                    .contains(&VaultDiscoveryWarning::EmptyCeilingSegment)
            );
            assert!(result.warnings.iter().any(|warning| matches!(
                warning,
                VaultDiscoveryWarning::InvalidCeilingSegment { .. }
            )));
        }
    }

    mod find_global {
        use super::*;
        #[cfg(target_os = "linux")]
        use crate::discovery::diagnostics::{
            DiscoveryWarning, GlobalDiscoveryWarning,
        };

        fn input<'a>(
            env_config_file: Option<&'a Path>,
            xdg_config_base: Option<&'a Path>,
            user_config_base: Option<&'a Path>,
            system_config_base: Option<&'a Path>,
        ) -> GlobalDiscoveryInput<'a> {
            GlobalDiscoveryInput {
                env_config_file,
                xdg_config_base,
                user_config_base,
                system_config_base,
                suppress_global: false,
            }
        }

        #[test]
        fn returns_marker_when_environment_config_file_exists() {
            let global_dir = tempdir().expect("global dir");
            let marker_path = write_marker(global_dir.path(), "lithos.toml");

            let result = engine()
                .find_global(&input(Some(&marker_path), None, None, None))
                .expect("global resolution succeeds");

            let marker = result.marker.expect("global marker is discovered");
            assert_eq!(
                marker.path,
                marker_path.canonicalize().expect("canonical marker")
            );
        }

        #[test]
        fn returns_none_when_no_global_config_exists() {
            let root = tempdir().expect("root");
            let missing_env = root.path().join("missing.toml");
            let xdg = root.path().join("xdg");
            let user = root.path().join("user");
            let system = root.path().join("system");
            std::fs::create_dir_all(&xdg).expect("xdg");
            std::fs::create_dir_all(&user).expect("user");
            std::fs::create_dir_all(&system).expect("system");

            let result = engine()
                .find_global(&input(
                    Some(&missing_env),
                    Some(&xdg),
                    Some(&user),
                    Some(&system),
                ))
                .expect("global resolution succeeds");

            assert_eq!(result.marker, None);
            assert_eq!(result.source, None);
            assert!(result.alternatives.is_empty());
            assert!(result.warnings.is_empty());
        }

        #[test]
        fn returns_none_when_global_lookup_is_suppressed() {
            let global_dir = tempdir().expect("global dir");
            let marker_path = write_marker(global_dir.path(), "lithos.toml");

            let result = engine()
                .find_global(&GlobalDiscoveryInput {
                    env_config_file: Some(&marker_path),
                    xdg_config_base: Some(global_dir.path()),
                    user_config_base: Some(global_dir.path()),
                    system_config_base: Some(global_dir.path()),
                    suppress_global: true,
                })
                .expect("global resolution succeeds");

            assert_eq!(result.marker, None);
            assert_eq!(result.alternatives, Vec::<FoundRootMarker>::new());
            assert_eq!(result.source, None);
        }

        #[test]
        fn prefers_environment_file_over_global_base_directories() {
            let root = tempdir().expect("root");
            let env_path = write_marker(root.path(), "env/lithos.json");
            let xdg = root.path().join("xdg");
            let user = root.path().join("user");
            let system = root.path().join("system");
            write_marker(&xdg, "lithos.toml");
            write_marker(&user, "lithos.toml");
            write_marker(&system, "lithos.toml");

            let result = engine()
                .find_global(&input(
                    Some(&env_path),
                    Some(&xdg),
                    Some(&user),
                    Some(&system),
                ))
                .expect("global resolution succeeds");
            let marker = result.marker.expect("marker is discovered");

            assert_eq!(result.source, Some(GlobalSourceType::EnvVar));
            assert_eq!(
                marker.path,
                env_path.canonicalize().expect("canonical env marker")
            );
        }

        #[test]
        fn prefers_xdg_config_over_user_and_system_config() {
            let root = tempdir().expect("root");
            let xdg = root.path().join("xdg");
            let user = root.path().join("user");
            let system = root.path().join("system");
            let xdg_path = write_marker(&xdg, "lithos.json");
            write_marker(&user, "lithos.toml");
            write_marker(&system, "lithos.toml");

            let result = engine()
                .find_global(&input(
                    None,
                    Some(&xdg),
                    Some(&user),
                    Some(&system),
                ))
                .expect("global resolution succeeds");
            let marker = result.marker.expect("marker is discovered");

            assert_eq!(result.source, Some(GlobalSourceType::XdgConfig));
            assert_eq!(
                marker.path,
                xdg_path.canonicalize().expect("canonical xdg marker")
            );
        }

        #[test]
        fn prefers_user_config_over_system_config() {
            let root = tempdir().expect("root");
            let user = root.path().join("user");
            let system = root.path().join("system");
            let user_path = write_marker(&user, "lithos.yaml");
            write_marker(&system, "lithos.toml");

            let result = engine()
                .find_global(&input(None, None, Some(&user), Some(&system)))
                .expect("global resolution succeeds");
            let marker = result.marker.expect("marker is discovered");

            assert_eq!(result.source, Some(GlobalSourceType::UserConfig));
            assert_eq!(
                marker.path,
                user_path.canonicalize().expect("canonical user marker")
            );
        }

        #[test]
        fn returns_same_tier_alternatives_without_selected_marker() {
            let root = tempdir().expect("root");
            let xdg = root.path().join("xdg");
            let selected_path = write_marker(&xdg, "lithos.toml");
            let json_path = write_marker(&xdg, "lithos.json");
            let yaml_path = write_marker(&xdg, "lithos.yaml");
            let system = root.path().join("system");
            write_marker(&system, "lithos.toml");

            let result = engine()
                .find_global(&input(None, Some(&xdg), None, Some(&system)))
                .expect("global resolution succeeds");
            let marker = result.marker.expect("marker is discovered");
            let selected =
                selected_path.canonicalize().expect("canonical toml");

            assert_eq!(marker.path, selected);
            assert_eq!(result.alternatives.len(), 2);
            assert!(
                !result.alternatives.iter().any(|alt| alt.path == selected)
            );
            assert!(result.alternatives.iter().any(|alt| {
                alt.path == json_path.canonicalize().expect("canonical json")
            }));
            assert!(result.alternatives.iter().any(|alt| {
                alt.path == yaml_path.canonicalize().expect("canonical yaml")
            }));
        }

        #[test]
        fn continues_to_system_config_when_higher_tiers_are_missing() {
            let root = tempdir().expect("root");
            let missing_env = root.path().join("missing.toml");
            let xdg = root.path().join("xdg");
            let user = root.path().join("user");
            let system = root.path().join("system");
            std::fs::create_dir_all(&xdg).expect("xdg");
            std::fs::create_dir_all(&user).expect("user");
            let system_path = write_marker(&system, "lithos.yml");

            let result = engine()
                .find_global(&input(
                    Some(&missing_env),
                    Some(&xdg),
                    Some(&user),
                    Some(&system),
                ))
                .expect("global resolution succeeds");
            let marker = result.marker.expect("marker is discovered");

            assert_eq!(result.source, Some(GlobalSourceType::SystemConfig));
            assert_eq!(
                marker.path,
                system_path.canonicalize().expect("canonical system marker")
            );
        }

        #[test]
        fn returns_none_when_env_config_file_has_unrecognised_extension() {
            let dir = tempdir().expect("dir");
            let conf_path = dir.path().join("lithos.conf");
            std::fs::write(&conf_path, "").expect("write conf file");

            let result = engine()
                .find_global(&input(Some(&conf_path), None, None, None))
                .expect("global resolution succeeds");

            assert_eq!(
                result.marker, None,
                "unrecognised extension should yield no marker"
            );
            assert_eq!(result.source, None);
        }

        // Case-correction relies on reading directory entries and comparing
        // names with `eq_ignore_ascii_case`. On case-sensitive filesystems
        // (Linux) `Lithos.TOML` and `lithos.toml` are distinct; only the
        // mis-cased entry is present so the warning fires deterministically.
        // On macOS (HFS+, case-insensitive) `probe_exact` may already
        // resolve `lithos.toml` via the OS, so the mis-cased probe path
        // may not be reached — gate to Linux only to keep the assertion
        // semantics stable.
        #[cfg(target_os = "linux")]
        #[test]
        fn returns_warning_when_global_filename_has_incorrect_case() {
            let root = tempdir().expect("root");
            let xdg = root.path().join("xdg");
            let resolved_path = write_marker(&xdg, "Lithos.TOML");
            let requested_path = xdg.join("lithos.toml");

            let result = engine()
                .find_global(&input(None, Some(&xdg), None, None))
                .expect("global resolution succeeds");
            let marker = result.marker.expect("marker is discovered");

            assert_eq!(marker.format, StructuredFileFormat::Toml);
            assert_eq!(
                marker.path,
                resolved_path.canonicalize().expect("canonical marker")
            );
            assert_eq!(result.warnings.len(), 1);
            assert_eq!(
                result.warnings.first(),
                Some(&DiscoveryWarning::Global(
                    GlobalDiscoveryWarning::CaseCorrection {
                        requested: requested_path,
                        resolved: resolved_path
                            .canonicalize()
                            .expect("canonical marker"),
                    },
                ))
            );
        }
    }
}
