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

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

/// An iterator that ascends from a directory up to defined boundary ceilings.
///
/// This walker is zero-allocation during traversal as it operates on purely
/// lexical parents of the starting path. It stops when a parent directory
/// matches one of the provided `ceilings`.
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

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {

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
        fn yields_ceiling_when_start_is_ceiling_if_allowed() {
            let root = tempdir().expect("root");
            let start = root.path().canonicalize().expect("canonical");
            let mut ceilings = HashSet::new();
            ceilings.insert(start.clone());

            let results: Vec<&Path> = ascent(&start, &ceilings, true).collect();
            assert_eq!(results, vec![start.as_path()]);
        }

        #[test]
        fn yields_none_when_start_is_ceiling_if_not_allowed() {
            let root = tempdir().expect("root");
            let start = root.path().canonicalize().expect("canonical");
            let mut ceilings = HashSet::new();
            ceilings.insert(start.clone());

            let results: Vec<&Path> =
                ascent(&start, &ceilings, false).collect();
            assert!(results.is_empty());
        }
    }
}
