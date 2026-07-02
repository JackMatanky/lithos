//! Candidate filtering for discovery output.

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::candidate::CandidatePath;

/// Canonical filesystem key for a candidate, falling back to the lexical path
/// when the file cannot be canonicalized (e.g. a broken symlink).
fn canonical_key(candidate: &CandidatePath) -> PathBuf {
    candidate
        .path()
        .as_path()
        .canonicalize()
        .unwrap_or_else(|_| candidate.path().as_path().to_path_buf())
}

/// Deduplicate keeping the **first** occurrence per canonical path.
///
/// Used for global candidates, where inputs arrive in precedence order
/// (flag → env → platform) and the first match wins.
pub(crate) fn dedupe(candidates: Vec<CandidatePath>) -> Vec<CandidatePath> {
    let mut seen = HashSet::new();
    let mut kept = Vec::new();

    for candidate in candidates {
        if seen.insert(canonical_key(&candidate)) {
            kept.push(candidate);
        }
    }

    kept
}

/// Deduplicate keeping the **last** occurrence per canonical path, preserving
/// the relative order of the surviving candidates.
///
/// Used for local candidates, which arrive outer-ancestor → nearest-ancestor.
/// When the same config is reachable from more than one ancestor, the nearest
/// (deepest) occurrence wins while the output stays in outer → nearest order.
pub(crate) fn dedupe_keep_last(
    candidates: Vec<CandidatePath>,
) -> Vec<CandidatePath> {
    let mut last_index: HashMap<PathBuf, usize> = HashMap::new();
    for (index, candidate) in candidates.iter().enumerate() {
        last_index.insert(canonical_key(candidate), index);
    }

    candidates
        .into_iter()
        .enumerate()
        .filter(|(index, candidate)| {
            last_index.get(&canonical_key(candidate)) == Some(index)
        })
        .map(|(_, candidate)| candidate)
        .collect()
}

pub(crate) fn filter_ignored(
    candidates: Vec<CandidatePath>,
    ignored_paths: &[PathBuf],
) -> Vec<CandidatePath> {
    candidates
        .into_iter()
        .filter(|candidate| {
            let key =
                candidate.path().as_path().canonicalize().unwrap_or_else(
                    |_| candidate.path().as_path().to_path_buf(),
                );
            !ignored_paths.iter().any(|ignored| ignored == &key)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use traces_fs::{DirPath, FilePath};

    use super::*;

    fn candidate(path: PathBuf) -> CandidatePath {
        let parent = path.parent().expect("candidate path should have parent");
        let base = DirPath::try_new(parent.to_path_buf()).expect("base dir");
        let file = FilePath::try_new(path).expect("file path");
        CandidatePath::new(base, file)
    }

    mod dedupe {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn removes_symlink_duplicates_and_preserves_first_occurrence() {
            let root = tempfile::tempdir().expect("root");
            let original = root.path().join("traces.toml");
            let link = root.path().join("link.toml");
            std::fs::write(&original, "").expect("original");
            #[cfg(unix)]
            std::os::unix::fs::symlink(&original, &link).expect("symlink");
            #[cfg(windows)]
            std::os::windows::fs::symlink_file(&original, &link)
                .expect("symlink");

            let candidates = super::dedupe(vec![
                candidate(link.clone()),
                candidate(original.clone()),
            ]);

            assert_eq!(candidates.len(), 1);
            assert_eq!(
                candidates.first().map(|candidate| candidate.path().as_path()),
                Some(link.as_path())
            );
        }
    }

    mod dedupe_keep_last {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn keeps_last_occurrence_and_preserves_order() {
            let root = tempfile::tempdir().expect("root");
            let outer = root.path().join("outer.toml");
            let inner = root.path().join("inner.toml");
            let link = root.path().join("link.toml");
            std::fs::write(&outer, "").expect("outer");
            std::fs::write(&inner, "").expect("inner");
            #[cfg(unix)]
            std::os::unix::fs::symlink(&inner, &link).expect("symlink");
            #[cfg(windows)]
            std::os::windows::fs::symlink_file(&inner, &link).expect("symlink");

            // outer, then a symlink to inner, then inner itself: the symlink
            // and inner share a canonical key, so the nearest (inner) wins.
            let kept = super::dedupe_keep_last(vec![
                candidate(outer.clone()),
                candidate(link),
                candidate(inner.clone()),
            ]);

            let paths: Vec<_> =
                kept.iter().map(|c| c.path().as_path().to_path_buf()).collect();
            assert_eq!(paths, vec![outer, inner]);
        }
    }

    mod ignored {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn filters_ignored_paths() {
            let root = tempfile::tempdir().expect("root");
            let ignored = root.path().join("ignored.toml");
            let kept = root.path().join("kept.toml");
            std::fs::write(&ignored, "").expect("ignored");
            std::fs::write(&kept, "").expect("kept");

            let candidates = filter_ignored(
                vec![candidate(ignored.clone()), candidate(kept.clone())],
                &[ignored.canonicalize().expect("canonical ignored")],
            );

            assert_eq!(candidates.len(), 1);
            assert_eq!(
                candidates.first().map(|candidate| candidate.path().as_path()),
                Some(kept.as_path())
            );
        }
    }
}
