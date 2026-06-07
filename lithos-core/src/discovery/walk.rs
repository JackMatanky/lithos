//! Directory traversal and boundary enforcement for ascending discovery.
//!
//! This module implements the mechanics of walking up the directory tree to
//! find configuration markers. It enforces "ceiling" boundaries to prevent
//! discovery from escaping authorized project or system areas.
//!
//! # Main Components
//!
//! - [`BoundedAscent`]: An iterator that yields parent directories from a
//!   starting path up to a set of boundary ceilings.
//! - [`DiscoveryBoundaries`]: Manages the starting directory and the collection
//!   of parsed and validated ceiling paths.
//!
//! # Ceiling Parsing
//!
//! Ceiling paths are typically provided as a platform-specific path list string
//! (e.g., from an environment variable). The
//! [`DiscoveryBoundaries::parse_ceilings`] method handles splitting,
//! canonicalization, and validation of these paths.

use std::{
    collections::HashSet,
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use super::diagnostics::VaultDiscoveryWarning;

/// An iterator that ascends from a directory up to defined boundary ceilings.
///
/// This walker is zero-allocation during traversal as it operates on purely
/// lexical parents of the starting path. It stops when a parent directory
/// matches one of the provided `ceilings`.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) struct BoundedAscent<'a> {
    current: Option<&'a Path>,
    ceilings: &'a HashSet<PathBuf>,
    allow_marker_at_ceiling: bool,
}

impl<'a> BoundedAscent<'a> {
    /// Creates a new bounded walker starting at `start`.
    pub(crate) fn new(
        start: &'a Path,
        ceilings: &'a HashSet<PathBuf>,
        allow_marker_at_ceiling: bool,
    ) -> Self {
        Self {
            current: Some(start),
            ceilings,
            allow_marker_at_ceiling,
        }
    }
}

impl<'a> Iterator for BoundedAscent<'a> {
    type Item = &'a Path;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        let is_ceiling = self.ceilings.contains(current);

        self.current = if is_ceiling {
            None
        } else {
            current.parent()
        };

        if is_ceiling && !self.allow_marker_at_ceiling {
            None
        } else {
            Some(current)
        }
    }
}

/// Start and stop constraints for the discovery traversal.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) struct DiscoveryBoundaries {
    pub(crate) start_dir: PathBuf,
    pub(crate) ceilings: HashSet<PathBuf>,
}

#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
impl DiscoveryBoundaries {
    /// Creates a new boundaries container.
    pub(crate) fn new(start_dir: PathBuf, ceilings: HashSet<PathBuf>) -> Self {
        Self {
            start_dir,
            ceilings,
        }
    }

    /// The starting directory for the walk.
    pub(crate) fn start_dir(&self) -> &Path {
        &self.start_dir
    }

    /// The set of ceiling directories that bound the walk.
    pub(crate) fn ceilings(&self) -> &HashSet<PathBuf> {
        &self.ceilings
    }

    /// Parses a raw platform-specific path list into a set of validated ceiling
    /// directories.
    ///
    /// Segments are split using the platform's path separator (e.g., `:` on
    /// Unix, `;` on Windows).
    ///
    /// # Diagnostics
    ///
    /// Non-fatal issues like empty segments or non-existent directories are
    /// reported via the `warnings` vector using [`VaultDiscoveryWarning`].
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

    mod bounded_ascent {
        use super::*;

        fn ascent<'a>(
            start: &'a Path,
            ceilings: &'a HashSet<PathBuf>,
            allow_marker_at_ceiling: bool,
        ) -> BoundedAscent<'a> {
            BoundedAscent::new(start, ceilings, allow_marker_at_ceiling)
        }

        #[test]
        fn yields_start_dir_first() {
            let root = tempdir().expect("root");
            let start = root.path().canonicalize().expect("canonical");
            let ceilings = HashSet::new();
            let mut w = ascent(&start, &ceilings, true);
            assert_eq!(w.next(), Some(start.as_path()));
        }

        #[test]
        fn yields_parent_after_start() {
            let root = tempdir().expect("root");
            let child = root.path().join("a").join("b");
            std::fs::create_dir_all(&child).expect("child");
            let start = child.canonicalize().expect("canonical");
            let parent = root.path().canonicalize().expect("parent canonical");
            let ceilings = HashSet::new();

            let results: Vec<&Path> = ascent(&start, &ceilings, true).collect();
            assert!(results.contains(&parent.as_path()));
        }

        #[test]
        fn stops_at_root() {
            let root = tempdir().expect("root");
            let child = root.path().join("a");
            std::fs::create_dir_all(&child).expect("child");
            let start = child.canonicalize().expect("canonical");
            let ceilings = HashSet::new();

            let results: Vec<&Path> = ascent(&start, &ceilings, true).collect();
            assert_eq!(results.first(), Some(&start.as_path()));
            assert!(results.len() >= 2);
        }

        #[test]
        fn yields_ceiling_directory_if_allowed() {
            let root = tempdir().expect("root");
            let stop = root.path().join("stop");
            let cwd = stop.join("deep");
            std::fs::create_dir_all(&cwd).expect("cwd");
            let start = cwd.canonicalize().expect("canonical start");
            let ceiling = stop.canonicalize().expect("canonical ceiling");

            let mut ceilings = HashSet::new();
            ceilings.insert(ceiling.clone());
            let results: Vec<&Path> = ascent(&start, &ceilings, true).collect();
            assert!(results.contains(&ceiling.as_path()));
            assert_eq!(results.last(), Some(&ceiling.as_path()));
        }

        #[test]
        fn stops_before_ceiling_if_not_allowed() {
            let root = tempdir().expect("root");
            let stop = root.path().join("stop");
            let cwd = stop.join("deep");
            std::fs::create_dir_all(&cwd).expect("cwd");
            let start = cwd.canonicalize().expect("canonical start");
            let ceiling = stop.canonicalize().expect("canonical ceiling");

            let mut ceilings = HashSet::new();
            ceilings.insert(ceiling.clone());
            let results: Vec<&Path> =
                ascent(&start, &ceilings, false).collect();
            assert!(!results.contains(&ceiling.as_path()));
        }

        #[test]
        fn starts_from_ceiling_and_yields_it_if_allowed() {
            let root = tempdir().expect("root");
            let start = root.path().canonicalize().expect("canonical");
            let mut ceilings = HashSet::new();
            ceilings.insert(start.clone());

            let results: Vec<&Path> = ascent(&start, &ceilings, true).collect();
            assert_eq!(results, vec![start.as_path()]);
        }

        #[test]
        fn starts_from_ceiling_and_yields_none_if_not_allowed() {
            let root = tempdir().expect("root");
            let start = root.path().canonicalize().expect("canonical");
            let mut ceilings = HashSet::new();
            ceilings.insert(start.clone());

            let results: Vec<&Path> =
                ascent(&start, &ceilings, false).collect();
            assert!(results.is_empty());
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
