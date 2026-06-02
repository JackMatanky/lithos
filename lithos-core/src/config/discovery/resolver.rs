use std::{
    collections::HashSet,
    env,
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
};

use super::{
    contracts::DiscoveredConfigFile,
    diagnostics::RootResolutionWarning,
    location::{ConfigLocation, LocalConfigLocation},
};
use crate::fs::format::StructuredFileFormat;

#[allow(
    dead_code,
    reason = "Phase-1 resolver seam is implemented before orchestration wiring"
)]
const LOCAL_MARKER_LOCATIONS: [LocalConfigLocation; 3] = [
    LocalConfigLocation::RootConfigFile,
    LocalConfigLocation::HiddenRootConfigFile,
    LocalConfigLocation::ConfigDirectoryFile,
];

#[allow(
    dead_code,
    reason = "Phase-1 resolver seam is implemented before orchestration wiring"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RootResolutionPolicy {
    allow_marker_at_ceiling: bool,
}

impl Default for RootResolutionPolicy {
    fn default() -> Self {
        Self {
            allow_marker_at_ceiling: true,
        }
    }
}

struct AscendingResolution {
    root: Option<PathBuf>,
    marker: Option<DiscoveredConfigFile>,
}

impl AscendingResolution {
    fn found(root: PathBuf, marker: DiscoveredConfigFile) -> Self {
        Self {
            root: Some(root),
            marker: Some(marker),
        }
    }

    fn not_found() -> Self {
        Self {
            root: None,
            marker: None,
        }
    }
}

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

    fn resolve_ascending(
        &self,
        cwd: &Path,
        ceilings: &HashSet<PathBuf>,
    ) -> Result<AscendingResolution, RootResolutionError> {
        let mut current = cwd.canonicalize().map_err(|source| {
            RootResolutionError::CurrentDirectoryCanonicalize {
                path: cwd.to_path_buf(),
                source,
            }
        })?;

        let mut visited = HashSet::new();

        loop {
            if !visited.insert(current.clone()) {
                return Ok(AscendingResolution::not_found());
            }

            if !self.policy.allow_marker_at_ceiling
                && ceilings.contains(&current)
            {
                return Ok(AscendingResolution::not_found());
            }

            if let Some(marker) = Self::discover_marker(&current)? {
                return Ok(AscendingResolution::found(current, marker));
            }

            if ceilings.contains(&current) {
                return Ok(AscendingResolution::not_found());
            }

            let Some(parent) = current.parent() else {
                return Ok(AscendingResolution::not_found());
            };

            let parent_canonical = parent.canonicalize().map_err(|source| {
                RootResolutionError::CanonicalizePath {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;

            if parent_canonical == current {
                return Ok(AscendingResolution::not_found());
            }

            current = parent_canonical;
        }
    }

    fn discover_marker(
        root: &Path,
    ) -> Result<Option<DiscoveredConfigFile>, RootResolutionError> {
        for location in LOCAL_MARKER_LOCATIONS {
            for format in StructuredFileFormat::PRECEDENCE {
                let path = location.candidate_path(root, format);
                if !path.is_file() {
                    continue;
                }

                let canonical = path.canonicalize().map_err(|source| {
                    RootResolutionError::CanonicalizePath {
                        path: path.clone(),
                        source,
                    }
                })?;
                return Ok(Some(DiscoveredConfigFile {
                    location: ConfigLocation::Local(location),
                    base: root.to_path_buf(),
                    path: canonical,
                    format,
                }));
            }
        }

        Ok(None)
    }

    fn parse_ceilings(
        ceiling_dirs_raw: Option<&OsStr>,
        warnings: &mut Vec<RootResolutionWarning>,
    ) -> HashSet<PathBuf> {
        let Some(raw) = ceiling_dirs_raw else {
            return HashSet::new();
        };

        let mut ceilings = HashSet::new();

        for segment in env::split_paths(raw) {
            let trimmed = segment.to_string_lossy().trim().to_owned();
            if trimmed.is_empty() {
                warnings.push(RootResolutionWarning::EmptyCeilingSegment);
                continue;
            }

            let path = PathBuf::from(&trimmed);
            let Ok(canonical) = path.canonicalize() else {
                warnings.push(RootResolutionWarning::InvalidCeilingSegment {
                    segment: path,
                });
                continue;
            };

            if !canonical.is_dir() {
                warnings.push(RootResolutionWarning::InvalidCeilingSegment {
                    segment: PathBuf::from(trimmed),
                });
                continue;
            }

            ceilings.insert(canonical);
        }

        ceilings
    }
}

#[allow(
    dead_code,
    reason = "Phase-1 resolver seam is implemented before orchestration wiring"
)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct RootResolverInput<'a> {
    pub(crate) explicit_vault_path: Option<&'a Path>,
    pub(crate) env_vault_path: Option<&'a Path>,
    pub(crate) cwd: &'a Path,
    pub(crate) ceiling_dirs_raw: Option<&'a OsStr>,
}

#[allow(
    dead_code,
    reason = "Phase-1 resolver seam is implemented before orchestration wiring"
)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RootResolutionResult {
    pub(crate) root: Option<PathBuf>,
    pub(crate) marker: Option<DiscoveredConfigFile>,
    pub(crate) source: Option<RootResolutionSource>,
    pub(crate) warnings: Vec<RootResolutionWarning>,
}

#[allow(
    dead_code,
    reason = "Phase-1 resolver seam is implemented before orchestration wiring"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RootResolutionSource {
    ExplicitFlag,
    EnvironmentVariable,
    AscendingDiscovery,
}

#[allow(
    dead_code,
    reason = "Phase-1 resolver seam is implemented before orchestration wiring"
)]
#[derive(Debug, thiserror::Error)]
pub(crate) enum RootResolutionError {
    #[error("Explicit vault path does not exist: {path}")]
    ExplicitPathMissing {
        path: PathBuf,
    },
    #[error("Explicit vault path is not a directory: {path}")]
    ExplicitPathNotDirectory {
        path: PathBuf,
    },
    #[error("Environment vault path does not exist: {path}")]
    EnvironmentPathMissing {
        path: PathBuf,
    },
    #[error("Environment vault path is not a directory: {path}")]
    EnvironmentPathNotDirectory {
        path: PathBuf,
    },
    #[error("Failed to canonicalize current directory {path}: {source}")]
    CurrentDirectoryCanonicalize {
        path: PathBuf,
        source: io::Error,
    },
    #[error("Failed to canonicalize path {path}: {source}")]
    CanonicalizePath {
        path: PathBuf,
        source: io::Error,
    },
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, fs, path::Path};

    use tempfile::tempdir;

    use super::*;

    fn path_list(paths: &[&Path]) -> OsString {
        env::join_paths(paths.iter().copied()).expect("joins path list")
    }

    fn resolver() -> RootResolver {
        RootResolver::new(RootResolutionPolicy::default())
    }

    mod precedence {
        use super::*;

        #[test]
        fn returns_explicit_resolution_when_explicit_and_env_are_both_present()
        {
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
            assert_eq!(
                result.root,
                Some(
                    explicit.path().canonicalize().expect("canonical explicit")
                )
            );
        }

        #[test]
        fn returns_environment_resolution_when_explicit_is_not_present() {
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
            assert_eq!(
                result.root,
                Some(env_dir.path().canonicalize().expect("canonical env"))
            );
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn returns_typed_error_when_explicit_path_does_not_exist() {
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
        fn returns_typed_error_when_environment_path_is_not_directory() {
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

    mod lookup {
        use super::*;

        #[test]
        fn returns_marker_file_when_marker_exists_in_ancestor() {
            let root = tempdir().expect("root");
            let marker_path = root.path().join("lithos.toml");
            fs::write(&marker_path, "").expect("marker");

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
            let marker = result.marker.expect("marker is discovered");

            assert_eq!(
                result.root,
                Some(root.path().canonicalize().expect("canonical root"))
            );
            assert_eq!(
                result.source,
                Some(RootResolutionSource::AscendingDiscovery)
            );
            assert_eq!(
                marker.base,
                root.path().canonicalize().expect("canonical root")
            );
            assert_eq!(
                marker.location,
                ConfigLocation::Local(LocalConfigLocation::RootConfigFile)
            );
            assert_eq!(
                marker.path,
                marker_path.canonicalize().expect("canonical marker")
            );
            assert_eq!(marker.format, StructuredFileFormat::Toml);
        }

        #[test]
        fn returns_not_found_when_ceiling_stops_before_marker() {
            let root = tempdir().expect("root");
            fs::write(root.path().join("lithos.toml"), "").expect("marker");

            let stop = root.path().join("level1");
            let cwd = stop.join("level2");
            fs::create_dir_all(&cwd).expect("cwd");

            let ceilings = path_list(&[&stop]);
            let result = resolver()
                .resolve(RootResolverInput {
                    explicit_vault_path: None,
                    env_vault_path: None,
                    cwd: &cwd,
                    ceiling_dirs_raw: Some(ceilings.as_os_str()),
                })
                .expect("resolution succeeds");

            assert_eq!(result.root, None);
            assert_eq!(result.marker, None);
            assert_eq!(result.source, None);
        }

        #[cfg(unix)]
        #[test]
        fn returns_not_found_when_visited_paths_repeat_via_symlink_loop() {
            use std::os::unix::fs as unix_fs;

            let root = tempdir().expect("root");
            let base = root.path().join("base");
            fs::create_dir_all(&base).expect("base");
            let loop_link = base.join("loop");
            unix_fs::symlink(&base, &loop_link).expect("symlink");
            let cwd = loop_link.join("loop").join("loop");

            let result = resolver()
                .resolve(RootResolverInput {
                    explicit_vault_path: None,
                    env_vault_path: None,
                    cwd: &cwd,
                    ceiling_dirs_raw: None,
                })
                .expect("resolution succeeds");

            assert_eq!(result.root, None);
        }
    }

    mod diagnostics {
        use super::*;

        #[test]
        fn returns_warnings_and_honors_valid_ceiling_when_segments_are_mixed() {
            let root = tempdir().expect("root");
            fs::write(root.path().join("lithos.toml"), "").expect("marker");

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

            assert_eq!(result.root, None);
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
