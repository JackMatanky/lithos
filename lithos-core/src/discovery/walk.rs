use std::{
    collections::HashSet,
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use super::diagnostics::VaultDiscoveryWarning;

#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) struct AscendingWalker {
    current: Option<PathBuf>,
    visited: HashSet<PathBuf>,
    ceilings: HashSet<PathBuf>,
}

impl AscendingWalker {
    pub(crate) fn new(start: PathBuf, ceilings: HashSet<PathBuf>) -> Self {
        Self {
            current: Some(start),
            visited: HashSet::new(),
            ceilings,
        }
    }
}

impl Iterator for AscendingWalker {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take()?;

        if !self.visited.insert(current.clone()) {
            return None;
        }

        let parent = current
            .parent()
            .map(Path::canonicalize)
            .and_then(Result::ok)
            .filter(|parent| parent != &current);

        self.current = parent;
        Some(current)
    }
}

#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) struct DiscoveryBoundaries {
    pub(crate) start_dir: PathBuf,
    pub(crate) ceilings: HashSet<PathBuf>,
}

#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
impl DiscoveryBoundaries {
    pub(crate) fn new(start_dir: PathBuf, ceilings: HashSet<PathBuf>) -> Self {
        Self {
            start_dir,
            ceilings,
        }
    }

    pub(crate) fn start_dir(&self) -> &Path {
        &self.start_dir
    }

    pub(crate) fn ceilings(&self) -> &HashSet<PathBuf> {
        &self.ceilings
    }

    pub(crate) fn parse_ceilings(
        ceiling_dirs_raw: Option<&OsStr>,
        warnings: &mut Vec<VaultDiscoveryWarning>,
    ) -> HashSet<PathBuf> {
        ceiling_dirs_raw
            .map(env::split_paths)
            .into_iter()
            .flatten()
            .filter_map(|segment| {
                let s = segment.to_string_lossy();
                let trimmed = s.trim();

                if trimmed.is_empty() {
                    warnings.push(VaultDiscoveryWarning::EmptyCeilingSegment);
                    return None;
                }

                let path = PathBuf::from(trimmed);
                match path.canonicalize() {
                    Ok(canonical) if canonical.is_dir() => Some(canonical),
                    _ => {
                        warnings.push(
                            VaultDiscoveryWarning::InvalidCeilingSegment {
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

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use tempfile::tempdir;

    use super::*;

    mod ascending_walker {
        use super::*;

        fn walker(
            start: PathBuf,
            ceilings: HashSet<PathBuf>,
        ) -> AscendingWalker {
            AscendingWalker::new(start, ceilings)
        }

        #[test]
        fn yields_start_dir_first() {
            let root = tempdir().expect("root");
            let start = root.path().canonicalize().expect("canonical");
            let mut w = walker(start.clone(), HashSet::new());
            assert_eq!(w.next(), Some(start));
        }

        #[test]
        fn yields_parent_after_start() {
            let root = tempdir().expect("root");
            let child = root.path().join("a").join("b");
            std::fs::create_dir_all(&child).expect("child");
            let start = child.canonicalize().expect("canonical");
            let parent = root.path().canonicalize().expect("parent canonical");

            let results: Vec<PathBuf> = walker(start, HashSet::new()).collect();
            assert!(results.contains(&parent));
        }

        #[test]
        fn stops_at_root() {
            let root = tempdir().expect("root");
            let child = root.path().join("a");
            std::fs::create_dir_all(&child).expect("child");
            let start = child.canonicalize().expect("canonical");

            let results: Vec<PathBuf> =
                walker(start.clone(), HashSet::new()).collect();
            assert_eq!(results.first(), Some(&start));
            assert!(results.len() >= 2);
        }

        #[test]
        fn yields_ceiling_directory() {
            let root = tempdir().expect("root");
            let stop = root.path().join("stop");
            let cwd = stop.join("deep");
            std::fs::create_dir_all(&cwd).expect("cwd");
            let start = cwd.canonicalize().expect("canonical start");
            let ceiling = stop.canonicalize().expect("canonical ceiling");

            let mut ceilings = HashSet::new();
            ceilings.insert(ceiling.clone());
            let results: Vec<PathBuf> = walker(start, ceilings).collect();
            assert!(results.contains(&ceiling));
        }

        #[cfg(unix)]
        #[test]
        fn stops_at_symlink_loop() {
            use std::os::unix::fs as unix_fs;

            let root = tempdir().expect("root");
            let base = root.path().join("base");
            std::fs::create_dir_all(&base).expect("base");
            let loop_link = base.join("loop");
            unix_fs::symlink(&base, &loop_link).expect("symlink");
            let cwd = loop_link.join("loop").join("loop");

            let start = cwd.canonicalize().expect("canonical start");
            let results: Vec<PathBuf> = walker(start, HashSet::new()).collect();
            assert!(!results.is_empty());
        }

        #[test]
        fn starts_from_ceiling_and_yields_it() {
            let root = tempdir().expect("root");
            let start = root.path().canonicalize().expect("canonical");
            let mut ceilings = HashSet::new();
            ceilings.insert(start.clone());

            let results: Vec<PathBuf> = walker(start, ceilings).collect();
            assert!(!results.is_empty());
        }
    }

    mod discovery_boundaries {
        use super::*;

        #[test]
        fn returns_start_dir_and_ceilings() {
            let start = PathBuf::from("/tmp");
            let mut ceilings = HashSet::new();
            ceilings.insert(PathBuf::from("/"));
            let b = DiscoveryBoundaries::new(start.clone(), ceilings.clone());
            assert_eq!(b.start_dir(), &start);
            assert_eq!(b.ceilings(), &ceilings);
        }
    }

    mod parse_ceilings_tests {
        use super::*;

        fn path_list(paths: &[&Path]) -> OsString {
            env::join_paths(paths.iter().copied()).expect("join paths")
        }

        #[test]
        fn returns_empty_set_when_env_is_absent() {
            let mut warnings = Vec::new();
            let ceilings =
                DiscoveryBoundaries::parse_ceilings(None, &mut warnings);
            assert!(ceilings.is_empty());
        }

        #[test]
        fn returns_canonical_ceiling_when_segment_is_valid() {
            let root = tempdir().expect("root");
            let raw = path_list(&[root.path()]);
            let mut warnings = Vec::new();

            let ceilings = DiscoveryBoundaries::parse_ceilings(
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

            DiscoveryBoundaries::parse_ceilings(
                Some(raw.as_os_str()),
                &mut warnings,
            );

            assert!(
                warnings.contains(&VaultDiscoveryWarning::EmptyCeilingSegment)
            );
        }

        #[test]
        fn returns_warning_when_segment_is_whitespace() {
            let raw = OsString::from("   ");
            let mut warnings = Vec::new();

            DiscoveryBoundaries::parse_ceilings(
                Some(raw.as_os_str()),
                &mut warnings,
            );

            assert!(
                warnings.contains(&VaultDiscoveryWarning::EmptyCeilingSegment)
            );
        }

        #[test]
        fn returns_warning_when_segment_is_missing() {
            let root = tempdir().expect("root");
            let missing = root.path().join("missing");
            let raw = path_list(&[&missing]);
            let mut warnings = Vec::new();

            DiscoveryBoundaries::parse_ceilings(
                Some(raw.as_os_str()),
                &mut warnings,
            );

            assert!(result_contains_invalid_ceiling(&warnings));
        }

        #[test]
        fn returns_warning_when_segment_is_file() {
            let root = tempdir().expect("root");
            let file_path = root.path().join("file.txt");
            std::fs::write(&file_path, "x").expect("write file");
            let raw = path_list(&[&file_path]);
            let mut warnings = Vec::new();

            DiscoveryBoundaries::parse_ceilings(
                Some(raw.as_os_str()),
                &mut warnings,
            );

            assert!(result_contains_invalid_ceiling(&warnings));
        }

        #[test]
        fn preserves_valid_ceilings_when_other_segments_are_invalid() {
            let root = tempdir().expect("root");
            let valid = root.path().join("stop");
            std::fs::create_dir_all(&valid).expect("valid ceiling");
            let raw = env::join_paths([
                OsString::from(""),
                valid.as_os_str().to_os_string(),
                root.path().join("missing").as_os_str().to_os_string(),
            ])
            .expect("join paths");
            let mut warnings = Vec::new();

            let ceilings = DiscoveryBoundaries::parse_ceilings(
                Some(raw.as_os_str()),
                &mut warnings,
            );

            assert!(ceilings.contains(
                &valid.canonicalize().expect("canonical valid ceiling")
            ));
        }

        fn result_contains_invalid_ceiling(
            warnings: &[VaultDiscoveryWarning],
        ) -> bool {
            warnings.iter().any(|warning| {
                matches!(
                    warning,
                    VaultDiscoveryWarning::InvalidCeilingSegment { .. }
                )
            })
        }
    }
}
