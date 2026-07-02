//! Candidate filtering for discovery output.

use std::{collections::HashSet, path::PathBuf};

use crate::candidate::CandidatePath;

pub(crate) fn dedupe(candidates: Vec<CandidatePath>) -> Vec<CandidatePath> {
    let mut seen = HashSet::new();
    let mut kept = Vec::new();

    for candidate in candidates {
        let key = candidate
            .path()
            .as_path()
            .canonicalize()
            .unwrap_or_else(|_| candidate.path().as_path().to_path_buf());
        if seen.insert(key) {
            kept.push(candidate);
        }
    }

    kept
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
        let base = DirPath::try_new(path.parent().unwrap().to_path_buf())
            .expect("base dir");
        let file = FilePath::try_new(path).expect("file path");
        CandidatePath::new(base, file)
    }

    mod dedupe {
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

    mod ignored {
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
