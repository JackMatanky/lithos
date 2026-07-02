//! Logic for examining individual directories for configuration marker files.
//!
//! This module provides the [`FolderProbe`] for detecting marker files in a
//! single directory. It abstracts the filesystem mechanics of checking for
//! supported filename patterns and structured formats.

use std::path::Path;

use traces_fs::{
    format::StructuredFileFormat,
    path::{DirPath, FilePath},
};

use crate::candidate::CandidatePath;

/// Infallible directory probe that checks for marker files by iterating
/// patterns × format precedence.
///
/// All input paths are pre-validated before reaching it.
pub(crate) struct FolderProbe {
    /// Ordered marker patterns to search for.
    pub(crate) patterns: &'static [super::policy::MarkerPattern],
}

impl FolderProbe {
    /// Probes a directory for all matching marker files.
    ///
    /// Returns candidates ordered by pattern precedence then format
    /// precedence (TOML > JSON > YAML > YML).
    pub(crate) fn probe(&self, dir: &DirPath) -> Vec<CandidatePath> {
        self.probe_inner(dir.as_path())
    }

    /// Probes a raw path (used during ascending traversal where paths are
    /// filesystem paths, not validated `DirPath`).
    pub(crate) fn probe_dir(&self, dir: &Path) -> Vec<CandidatePath> {
        self.probe_inner(dir)
    }

    fn probe_inner(&self, dir: &Path) -> Vec<CandidatePath> {
        let mut results = Vec::new();
        for pattern in self.patterns {
            for format in StructuredFileFormat::PRECEDENCE {
                let mut path = dir.join(pattern.prefix);
                path.set_extension(format.extension());

                if !path.is_file() {
                    continue;
                }

                if let (Ok(base), Ok(file)) = (
                    DirPath::try_new(dir.to_path_buf()),
                    FilePath::try_new(path),
                ) {
                    results.push(CandidatePath::new(base, file));
                }
            }
        }
        results
    }
}

pub(crate) fn exact_probe(
    dir: &DirPath,
    markers: &[&str],
) -> Vec<CandidatePath> {
    markers
        .iter()
        .filter_map(|marker| {
            let path = dir.as_path().join(marker);
            if !path.is_file() {
                return None;
            }
            FilePath::try_new(path)
                .ok()
                .map(|file| CandidatePath::new(dir.clone(), file))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    mod exact_filenames {
        use super::*;

        #[test]
        fn returns_candidate_when_marker_exists() {
            let root = tempfile::tempdir().expect("root");
            let config = root.path().join("traces.toml");
            std::fs::write(&config, "").expect("config");
            let dir = DirPath::try_new(root.path().to_path_buf()).unwrap();

            let candidates = exact_probe(&dir, &["traces.toml"]);

            assert_eq!(candidates.len(), 1);
            assert_eq!(
                candidates.first().map(|candidate| candidate.path().as_path()),
                Some(config.as_path())
            );
        }

        #[test]
        fn returns_empty_when_no_marker_exists() {
            let root = tempfile::tempdir().expect("root");
            let dir = DirPath::try_new(root.path().to_path_buf()).unwrap();

            let candidates = exact_probe(&dir, &["traces.toml"]);

            assert!(candidates.is_empty());
        }

        #[test]
        fn ignores_non_marker_files() {
            let root = tempfile::tempdir().expect("root");
            std::fs::write(root.path().join("other.toml"), "").expect("other");
            let dir = DirPath::try_new(root.path().to_path_buf()).unwrap();

            let candidates = exact_probe(&dir, &["traces.toml"]);

            assert!(candidates.is_empty());
        }
    }
}
