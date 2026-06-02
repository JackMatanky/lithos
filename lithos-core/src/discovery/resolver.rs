//! Phase 1 vault root resolution and discovery marker location.
//!
//! This module implements the first phase of the configuration discovery
//! process: finding the "vault root" directory and the "marker" file (e.g.,
//! `lithos.toml`) that defines it.
//!
//! Resolution follows a strict precedence chain:
//! 1. **Explicit Flag**: A path provided directly via CLI.
//! 2. **Environment Variable**: A path provided via the `LITHOS_VAULT`
//!    environment variable.
//! 3. **Ascending Discovery**: Searching upwards from the current working
//!    directory until a marker file is found or a "ceiling" directory is
//!    reached.

use std::{
    collections::HashSet,
    env,
    ffi::OsStr,
    io, iter,
    path::{Path, PathBuf},
};

use crate::{
    discovery::{DiscoveredConfigPath, RootResolutionWarning},
    fs::format::StructuredFileFormat,
};

/// The primary engine for resolving a vault root and its discovery marker.
///
/// `RootResolver` implements the policy-driven search for the configuration
/// entry point of a vault.
#[allow(
    dead_code,
    reason = "Phase-1 resolver seam is implemented before orchestration wiring"
)]
#[derive(Debug, Default)]
pub(crate) struct RootResolver {
    policy: RootResolutionPolicy,
}

#[allow(
    dead_code,
    reason = "Phase-1 resolver seam is implemented before orchestration wiring"
)]
impl RootResolver {
    /// Creates a new resolver with the specified resolution
    /// [`RootResolutionPolicy`].
    #[allow(
        dead_code,
        reason = "Phase-1 resolver seam is implemented before orchestration \
                  wiring"
    )]
    pub(crate) const fn new(policy: RootResolutionPolicy) -> Self {
        Self {
            policy,
        }
    }

    /// Resolves the vault root based on the provided [`RootResolverInput`].
    ///
    /// This method evaluates overrides (explicit flag, environment) before
    /// performing an ascending search from the current working directory.
    ///
    /// # Errors
    ///
    /// Returns [`RootResolutionError`] if:
    /// - An explicit or environment path is provided but does not exist or is
    ///   not a directory.
    /// - The current directory cannot be canonicalized.
    /// - A filesystem error occurs during discovery (e.g., permission denied
    ///   during canonicalization).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let resolver = RootResolver::default();
    /// let input = RootResolverInput {
    ///     explicit_vault_path: None,
    ///     env_vault_path: None,
    ///     cwd: Path::new("."),
    ///     ceiling_dirs_raw: None,
    /// };
    ///
    /// let result = resolver.resolve(input)?;
    /// if let Some(root) = result.root {
    ///     println!("Resolved vault root: {}", root.display());
    /// }
    /// ```
    pub(crate) fn resolve(
        &self,
        input: RootResolverInput<'_>,
    ) -> Result<RootResolutionResult, RootResolutionError> {
        if let Some(path) = input.explicit_vault_path {
            return Self::resolve_override(
                path,
                RootResolutionSource::ExplicitFlag,
            );
        }

        if let Some(path) = input.env_vault_path {
            return Self::resolve_override(
                path,
                RootResolutionSource::EnvironmentVariable,
            );
        }

        let mut warnings = Vec::new();
        let ceilings =
            Self::parse_ceilings(input.ceiling_dirs_raw, &mut warnings);
        let resolution = self.resolve_ascending(input.cwd, &ceilings)?;
        let source = resolution
            .root
            .as_ref()
            .map(|_| RootResolutionSource::AscendingDiscovery);

        Ok(RootResolutionResult {
            root: resolution.root,
            marker: resolution.marker,
            source,
            warnings,
        })
    }

    /// Resolves a vault root from a forced override path.
    fn resolve_override(
        path: &Path,
        source: RootResolutionSource,
    ) -> Result<RootResolutionResult, RootResolutionError> {
        let root = Self::validate_override(path, source)?;
        Ok(RootResolutionResult {
            root: Some(root),
            marker: None,
            source: Some(source),
            warnings: Vec::new(),
        })
    }

    /// Validates that an override path exists, is a directory, and can be
    /// canonicalized.
    fn validate_override(
        path: &Path,
        source: RootResolutionSource,
    ) -> Result<PathBuf, RootResolutionError> {
        if !path.exists() {
            return Err(match source {
                RootResolutionSource::ExplicitFlag => {
                    RootResolutionError::ExplicitPathMissing {
                        path: path.to_path_buf(),
                    }
                }
                RootResolutionSource::EnvironmentVariable => {
                    RootResolutionError::EnvironmentPathMissing {
                        path: path.to_path_buf(),
                    }
                }
                RootResolutionSource::AscendingDiscovery => {
                    RootResolutionError::CanonicalizePath {
                        path: path.to_path_buf(),
                        source: io::Error::new(
                            io::ErrorKind::NotFound,
                            "ascending discovery path is missing",
                        ),
                    }
                }
            });
        }

        if !path.is_dir() {
            return Err(match source {
                RootResolutionSource::ExplicitFlag => {
                    RootResolutionError::ExplicitPathNotDirectory {
                        path: path.to_path_buf(),
                    }
                }
                RootResolutionSource::EnvironmentVariable => {
                    RootResolutionError::EnvironmentPathNotDirectory {
                        path: path.to_path_buf(),
                    }
                }
                RootResolutionSource::AscendingDiscovery => {
                    RootResolutionError::CanonicalizePath {
                        path: path.to_path_buf(),
                        source: io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "ascending discovery path is not a directory",
                        ),
                    }
                }
            });
        }

        path.canonicalize().map_err(|error| {
            RootResolutionError::CanonicalizePath {
                path: path.to_path_buf(),
                source: error,
            }
        })
    }

    /// Performs the ascending discovery search starting from `cwd`.
    fn resolve_ascending(
        &self,
        cwd: &Path,
        ceilings: &HashSet<PathBuf>,
    ) -> Result<AscendingResolution, RootResolutionError> {
        let start = cwd.canonicalize().map_err(|source| {
            RootResolutionError::CurrentDirectoryCanonicalize {
                path: cwd.to_path_buf(),
                source,
            }
        })?;

        let mut visited = HashSet::new();

        let walk = iter::successors(Some(start), |path| {
            // Short-circuit if we hit a loop
            if !visited.insert(path.clone()) {
                return None;
            }

            // Termination conditions: stop if we are at a ceiling
            if ceilings.contains(path) {
                return None;
            }

            path.parent()
                .map(Path::canonicalize)
                .and_then(Result::ok)
                .filter(|parent| parent != path)
        });

        for current in walk {
            if !self.policy.allow_marker_at_ceiling
                && ceilings.contains(&current)
            {
                break;
            }

            if let Some(marker) = Self::discover_marker(&current)? {
                return Ok(AscendingResolution::found(current, marker));
            }

            if ceilings.contains(&current) {
                break;
            }
        }

        Ok(AscendingResolution::not_found())
    }

    /// Checks a specific directory for any supported root marker files.
    fn discover_marker(
        root: &Path,
    ) -> Result<Option<DiscoveredConfigPath>, RootResolutionError> {
        let prefixes = ["lithos", ".lithos", ".lithos/config"];
        for prefix in prefixes {
            for format in StructuredFileFormat::PRECEDENCE {
                let ext = format.extension();
                let filename = format!("{prefix}.{ext}");
                let path = root.join(filename);

                if !path.is_file() {
                    continue;
                }

                let canonical = match path.canonicalize() {
                    Ok(c) => c,
                    Err(source) => {
                        return Err(RootResolutionError::CanonicalizePath {
                            path,
                            source,
                        });
                    }
                };

                return Ok(Some(DiscoveredConfigPath {
                    base: root.to_path_buf(),
                    path: canonical,
                    format,
                }));
            }
        }
        Ok(None)
    }

    /// Parses a raw string of ceiling directories (e.g., from an environment
    /// variable).
    fn parse_ceilings(
        ceiling_dirs_raw: Option<&OsStr>,
        warnings: &mut Vec<RootResolutionWarning>,
    ) -> HashSet<PathBuf> {
        ceiling_dirs_raw
            .map(env::split_paths)
            .into_iter()
            .flatten()
            .filter_map(|segment| {
                let s = segment.to_string_lossy();
                let trimmed = s.trim();

                if trimmed.is_empty() {
                    warnings.push(RootResolutionWarning::EmptyCeilingSegment);
                    return None;
                }

                let path = PathBuf::from(trimmed);
                match path.canonicalize() {
                    Ok(canonical) if canonical.is_dir() => Some(canonical),
                    _ => {
                        warnings.push(
                            RootResolutionWarning::InvalidCeilingSegment {
                                segment: path,
                            },
                        );
                        None
                    }
                }
            })
            .collect()
    }
}

/// Input parameters for the root resolution process.
#[allow(
    dead_code,
    reason = "Phase-1 resolver seam is implemented before orchestration wiring"
)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct RootResolverInput<'a> {
    /// Explicit vault path from CLI flags.
    pub(crate) explicit_vault_path: Option<&'a Path>,
    /// Forced vault path from the environment.
    pub(crate) env_vault_path: Option<&'a Path>,
    /// The directory to start ascending search from.
    pub(crate) cwd: &'a Path,
    /// Raw ceiling directory string (platform-specific separator).
    pub(crate) ceiling_dirs_raw: Option<&'a OsStr>,
}

/// The result of a successful (though potentially empty) root resolution.
#[allow(
    dead_code,
    reason = "Phase-1 resolver seam is implemented before orchestration wiring"
)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RootResolutionResult {
    /// The absolute path to the resolved vault root.
    pub(crate) root: Option<PathBuf>,
    /// The discovery marker found at the root, if resolved via ascending
    /// discovery.
    pub(crate) marker: Option<DiscoveredConfigPath>,
    /// The origin of the resolution.
    pub(crate) source: Option<RootResolutionSource>,
    /// Non-fatal warnings encountered during resolution (e.g., invalid ceiling
    /// segments).
    pub(crate) warnings: Vec<RootResolutionWarning>,
}

/// The origin of a resolved vault root.
#[allow(
    dead_code,
    reason = "Phase-1 resolver seam is implemented before orchestration wiring"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RootResolutionSource {
    /// Provided via an explicit CLI flag.
    ExplicitFlag,
    /// Provided via an environment variable.
    EnvironmentVariable,
    /// Discovered by searching upwards from the current directory.
    AscendingDiscovery,
}

/// Policy configuration for the [`RootResolver`].
#[allow(
    dead_code,
    reason = "Phase-1 resolver seam is implemented before orchestration wiring"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RootResolutionPolicy {
    /// Whether to allow a discovery marker to be valid even if it is located
    /// exactly at a ceiling directory.
    pub(crate) allow_marker_at_ceiling: bool,
}

impl Default for RootResolutionPolicy {
    fn default() -> Self {
        Self {
            allow_marker_at_ceiling: true,
        }
    }
}

/// Errors that can occur during vault root resolution.
#[allow(
    dead_code,
    reason = "Phase-1 resolver seam is implemented before orchestration wiring"
)]
#[derive(Debug, thiserror::Error)]
pub(crate) enum RootResolutionError {
    /// The path provided via CLI flag does not exist.
    #[error("Explicit vault path does not exist: {path}")]
    ExplicitPathMissing {
        path: PathBuf,
    },
    /// The path provided via CLI flag exists but is not a directory.
    #[error("Explicit vault path is not a directory: {path}")]
    ExplicitPathNotDirectory {
        path: PathBuf,
    },
    /// The path provided via environment variable does not exist.
    #[error("Environment vault path does not exist: {path}")]
    EnvironmentPathMissing {
        path: PathBuf,
    },
    /// The path provided via environment variable exists but is not a
    /// directory.
    #[error("Environment vault path is not a directory: {path}")]
    EnvironmentPathNotDirectory {
        path: PathBuf,
    },
    /// Failed to canonicalize the current working directory.
    #[error("Failed to canonicalize current directory {path}: {source}")]
    CurrentDirectoryCanonicalize {
        path: PathBuf,
        source: io::Error,
    },
    /// A general filesystem error occurred during path canonicalization.
    #[error("Failed to canonicalize path {path}: {source}")]
    CanonicalizePath {
        path: PathBuf,
        source: io::Error,
    },
}

/// Internal helper for tracking the result of an ascending discovery walk.
struct AscendingResolution {
    root: Option<PathBuf>,
    marker: Option<DiscoveredConfigPath>,
}

impl AscendingResolution {
    /// Constructs a successful resolution.
    fn found(root: PathBuf, marker: DiscoveredConfigPath) -> Self {
        Self {
            root: Some(root),
            marker: Some(marker),
        }
    }

    /// Constructs a resolution where no marker was found.
    fn not_found() -> Self {
        Self {
            root: None,
            marker: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, fs, path::Path};

    use tempfile::tempdir;

    use super::*;

    mod fixtures {
        use super::*;

        pub(super) fn path_list(paths: &[&Path]) -> OsString {
            env::join_paths(paths.iter().copied()).expect("joins path list")
        }

        pub(super) fn resolver() -> RootResolver {
            RootResolver::new(RootResolutionPolicy::default())
        }

        pub(super) fn resolver_rejecting_ceiling_markers() -> RootResolver {
            RootResolver::new(RootResolutionPolicy {
                allow_marker_at_ceiling: false,
            })
        }

        pub(super) fn input_from_cwd(cwd: &Path) -> RootResolverInput<'_> {
            RootResolverInput {
                explicit_vault_path: None,
                env_vault_path: None,
                cwd,
                ceiling_dirs_raw: None,
            }
        }

        pub(super) fn input_with_ceilings<'a>(
            cwd: &'a Path,
            ceilings: &'a OsString,
        ) -> RootResolverInput<'a> {
            RootResolverInput {
                explicit_vault_path: None,
                env_vault_path: None,
                cwd,
                ceiling_dirs_raw: Some(ceilings.as_os_str()),
            }
        }

        pub(super) fn write_marker(root: &Path, relative: &str) -> PathBuf {
            let path = root.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create marker parent dir");
            }
            fs::write(&path, "").expect("write marker file");
            path
        }
    }

    mod resolve {
        use super::{fixtures::*, *};

        #[test]
        fn returns_explicit_source_when_explicit_path_exists() {
            let explicit = tempdir().expect("explicit dir");
            let env_dir = tempdir().expect("env dir");
            let cwd = tempdir().expect("cwd");

            let result = resolver()
                .resolve(RootResolverInput {
                    explicit_vault_path: Some(explicit.path()),
                    env_vault_path: Some(env_dir.path()),
                    cwd: cwd.path(),
                    ceiling_dirs_raw: None,
                })
                .expect("resolution succeeds");

            assert_eq!(result.source, Some(RootResolutionSource::ExplicitFlag));
        }

        #[test]
        fn returns_explicit_root_when_explicit_path_exists() {
            let explicit = tempdir().expect("explicit dir");
            let env_dir = tempdir().expect("env dir");
            let cwd = tempdir().expect("cwd");

            let result = resolver()
                .resolve(RootResolverInput {
                    explicit_vault_path: Some(explicit.path()),
                    env_vault_path: Some(env_dir.path()),
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

            let result = resolver()
                .resolve(RootResolverInput {
                    explicit_vault_path: None,
                    env_vault_path: Some(env_dir.path()),
                    cwd: cwd.path(),
                    ceiling_dirs_raw: None,
                })
                .expect("resolution succeeds");

            assert_eq!(
                result.source,
                Some(RootResolutionSource::EnvironmentVariable)
            );
        }

        #[test]
        fn returns_environment_root_when_environment_path_exists() {
            let env_dir = tempdir().expect("env dir");
            let cwd = tempdir().expect("cwd");

            let result = resolver()
                .resolve(RootResolverInput {
                    explicit_vault_path: None,
                    env_vault_path: Some(env_dir.path()),
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

            let result = resolver()
                .resolve(RootResolverInput {
                    explicit_vault_path: Some(explicit.path()),
                    env_vault_path: Some(env_dir.path()),
                    cwd: cwd.path(),
                    ceiling_dirs_raw: None,
                })
                .expect("resolution succeeds");

            assert_eq!(result.source, Some(RootResolutionSource::ExplicitFlag));
        }

        #[test]
        fn returns_ascending_source_when_marker_is_found() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.toml");

            let result = resolver()
                .resolve(input_from_cwd(root.path()))
                .expect("resolution succeeds");

            assert_eq!(
                result.source,
                Some(RootResolutionSource::AscendingDiscovery)
            );
        }
    }

    mod validate_override {
        use super::{fixtures::*, *};

        #[test]
        fn returns_error_when_explicit_path_is_missing() {
            let cwd = tempdir().expect("cwd");
            let missing = cwd.path().join("missing");

            let error = resolver()
                .resolve(RootResolverInput {
                    explicit_vault_path: Some(&missing),
                    env_vault_path: None,
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
            fs::write(&file_path, "x").expect("write file");

            let error = resolver()
                .resolve(RootResolverInput {
                    explicit_vault_path: Some(&file_path),
                    env_vault_path: None,
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

            let error = resolver()
                .resolve(RootResolverInput {
                    explicit_vault_path: None,
                    env_vault_path: Some(&missing),
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
            fs::write(&file_path, "x").expect("write file");

            let error = resolver()
                .resolve(RootResolverInput {
                    explicit_vault_path: None,
                    env_vault_path: Some(&file_path),
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
    }

    mod resolve_ascending {
        use super::{fixtures::*, *};

        #[test]
        fn returns_root_when_marker_exists_in_cwd() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.toml");

            let result = resolver()
                .resolve(input_from_cwd(root.path()))
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
            fs::create_dir_all(&child).expect("child");

            let result = resolver()
                .resolve(RootResolverInput {
                    explicit_vault_path: None,
                    env_vault_path: None,
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
            fs::create_dir_all(&child).expect("child");

            let result = resolver()
                .resolve(input_from_cwd(&child))
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

            let result = resolver()
                .resolve(input_from_cwd(root.path()))
                .expect("resolution succeeds");

            assert_eq!(result.root, None);
        }

        #[test]
        fn returns_none_when_ceiling_stops_before_marker() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.toml");

            let stop = root.path().join("level1");
            let cwd = stop.join("level2");
            fs::create_dir_all(&cwd).expect("cwd");

            let ceilings = path_list(&[&stop]);
            let result = resolver()
                .resolve(input_with_ceilings(&cwd, &ceilings))
                .expect("resolution succeeds");

            assert_eq!(result.root, None);
        }

        #[test]
        fn returns_marker_at_ceiling_when_policy_allows_it() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.toml");

            let cwd = root.path().join("nested");
            fs::create_dir_all(&cwd).expect("cwd");

            let ceilings = path_list(&[root.path()]);
            let result = resolver()
                .resolve(input_with_ceilings(&cwd, &ceilings))
                .expect("resolution succeeds");

            assert!(result.marker.is_some());
        }

        #[test]
        fn returns_none_at_ceiling_when_policy_rejects_it() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.toml");

            let cwd = root.path().join("nested");
            fs::create_dir_all(&cwd).expect("cwd");

            let ceilings = path_list(&[root.path()]);
            let result = resolver_rejecting_ceiling_markers()
                .resolve(input_with_ceilings(&cwd, &ceilings))
                .expect("resolution succeeds");

            assert_eq!(result.root, None);
        }

        #[cfg(unix)]
        #[test]
        fn returns_none_when_symlink_loop_repeats_path() {
            use std::os::unix::fs as unix_fs;

            let root = tempdir().expect("root");
            let base = root.path().join("base");
            fs::create_dir_all(&base).expect("base");
            let loop_link = base.join("loop");
            unix_fs::symlink(&base, &loop_link).expect("symlink");
            let cwd = loop_link.join("loop").join("loop");

            let result = resolver()
                .resolve(input_from_cwd(&cwd))
                .expect("resolution succeeds");

            assert_eq!(result.root, None);
        }
    }

    mod discover_marker {
        use super::{fixtures::*, *};

        #[test]
        fn returns_root_config_marker_first() {
            let root = tempdir().expect("root");
            write_marker(root.path(), ".lithos.toml");
            let expected_path = write_marker(root.path(), "lithos.toml");

            let marker = RootResolver::discover_marker(root.path())
                .expect("marker lookup succeeds")
                .expect("marker exists");

            assert_eq!(
                marker.path,
                expected_path.canonicalize().expect("canonical marker")
            );
        }

        #[test]
        fn returns_hidden_root_marker_when_root_marker_absent() {
            let root = tempdir().expect("root");
            let expected_path = write_marker(root.path(), ".lithos.toml");

            let marker = RootResolver::discover_marker(root.path())
                .expect("marker lookup succeeds")
                .expect("marker exists");

            assert_eq!(
                marker.path,
                expected_path.canonicalize().expect("canonical marker")
            );
        }

        #[test]
        fn returns_config_directory_marker_when_other_markers_absent() {
            let root = tempdir().expect("root");
            let expected_path =
                write_marker(root.path(), ".lithos/config.toml");

            let marker = RootResolver::discover_marker(root.path())
                .expect("marker lookup succeeds")
                .expect("marker exists");

            assert_eq!(
                marker.path,
                expected_path.canonicalize().expect("canonical marker")
            );
        }

        #[test]
        fn returns_toml_marker_before_json_marker() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.json");
            write_marker(root.path(), "lithos.toml");

            let marker = RootResolver::discover_marker(root.path())
                .expect("marker lookup succeeds")
                .expect("marker exists");

            assert_eq!(marker.format, StructuredFileFormat::Toml);
        }

        #[test]
        fn returns_none_when_no_marker_file_exists() {
            let root = tempdir().expect("root");

            let marker = RootResolver::discover_marker(root.path())
                .expect("marker lookup succeeds");

            assert_eq!(marker, None);
        }
    }

    mod parse_ceilings {
        use super::{fixtures::*, *};

        #[test]
        fn returns_empty_set_when_env_is_absent() {
            let mut warnings = Vec::new();

            let ceilings = RootResolver::parse_ceilings(None, &mut warnings);

            assert!(ceilings.is_empty());
        }

        #[test]
        fn returns_canonical_ceiling_when_segment_is_valid() {
            let root = tempdir().expect("root");
            let raw = path_list(&[root.path()]);
            let mut warnings = Vec::new();

            let ceilings = RootResolver::parse_ceilings(
                Some(raw.as_os_str()),
                &mut warnings,
            );

            assert!(ceilings.contains(
                &root.path().canonicalize().expect("canonical root")
            ));
        }

        #[test]
        fn returns_warning_when_segment_is_empty() {
            let raw = OsString::from(":");
            let mut warnings = Vec::new();

            RootResolver::parse_ceilings(Some(raw.as_os_str()), &mut warnings);

            assert!(
                warnings.contains(&RootResolutionWarning::EmptyCeilingSegment)
            );
        }

        #[test]
        fn returns_warning_when_segment_is_whitespace() {
            let raw = OsString::from("   ");
            let mut warnings = Vec::new();

            RootResolver::parse_ceilings(Some(raw.as_os_str()), &mut warnings);

            assert!(
                warnings.contains(&RootResolutionWarning::EmptyCeilingSegment)
            );
        }

        #[test]
        fn returns_warning_when_segment_is_missing() {
            let root = tempdir().expect("root");
            let missing = root.path().join("missing");
            let raw = path_list(&[&missing]);
            let mut warnings = Vec::new();

            RootResolver::parse_ceilings(Some(raw.as_os_str()), &mut warnings);

            assert!(result_contains_invalid_ceiling(&warnings));
        }

        #[test]
        fn returns_warning_when_segment_is_file() {
            let root = tempdir().expect("root");
            let file_path = root.path().join("file.txt");
            fs::write(&file_path, "x").expect("write file");
            let raw = path_list(&[&file_path]);
            let mut warnings = Vec::new();

            RootResolver::parse_ceilings(Some(raw.as_os_str()), &mut warnings);

            assert!(result_contains_invalid_ceiling(&warnings));
        }

        #[test]
        fn preserves_valid_ceilings_when_other_segments_are_invalid() {
            let root = tempdir().expect("root");
            let valid = root.path().join("stop");
            fs::create_dir_all(&valid).expect("valid ceiling");
            let raw = env::join_paths([
                OsString::from(""),
                valid.as_os_str().to_os_string(),
                root.path().join("missing").as_os_str().to_os_string(),
            ])
            .expect("join paths");
            let mut warnings = Vec::new();

            let ceilings = RootResolver::parse_ceilings(
                Some(raw.as_os_str()),
                &mut warnings,
            );

            assert!(ceilings.contains(
                &valid.canonicalize().expect("canonical valid ceiling")
            ));
        }

        fn result_contains_invalid_ceiling(
            warnings: &[RootResolutionWarning],
        ) -> bool {
            warnings.iter().any(|warning| {
                matches!(
                    warning,
                    RootResolutionWarning::InvalidCeilingSegment { .. }
                )
            })
        }
    }

    mod diagnostics {
        use super::{fixtures::*, *};

        #[test]
        fn returns_warnings_when_segments_are_mixed() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.toml");

            let stop = root.path().join("stop");
            let cwd = stop.join("nested");
            fs::create_dir_all(&cwd).expect("cwd");

            let raw = env::join_paths([
                OsString::from(""),
                OsString::from("   "),
                stop.as_os_str().to_os_string(),
                root.path().join("missing").as_os_str().to_os_string(),
            ])
            .expect("join paths");

            let result = resolver()
                .resolve(RootResolverInput {
                    explicit_vault_path: None,
                    env_vault_path: None,
                    cwd: &cwd,
                    ceiling_dirs_raw: Some(raw.as_os_str()),
                })
                .expect("resolution succeeds");

            assert!(
                result
                    .warnings
                    .contains(&RootResolutionWarning::EmptyCeilingSegment)
            );
            assert!(result.warnings.iter().any(|warning| matches!(
                warning,
                RootResolutionWarning::InvalidCeilingSegment { .. }
            )));
        }
    }
}
