//! Logic for examining individual directories for configuration marker files.
//!
//! This module contains two probes for two eras of discovery:
//!
//! - [`exact_probe`] is the **new** linear-discovery entry point. It matches a
//!   directory against an exact list of marker filenames (from
//!   [`crate::location`]) and is what [`DiscoveryProcessor`] uses.
//! - [`FolderProbe`] is the **old** processor's probe. It iterates marker
//!   patterns × structured-format precedence and is used only by
//!   `processor_old`; it is removed together with the old discovery service in
//!   issue 07. New code must not use it.
//!
//! [`DiscoveryProcessor`]: super::processor::DiscoveryProcessor

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
///
/// **Old discovery only** — used by `processor_old`, removed in issue 07. New
/// linear discovery uses [`exact_probe`] instead.
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

/// Probe a single directory for exact marker filenames.
///
/// Joins each entry of `markers` (exact relative names such as `traces.toml`
/// or `traces/config.toml` from [`crate::location`]) onto `dir` and returns a
/// [`CandidatePath`] for each one that resolves to a regular file. Order
/// follows the `markers` slice. Non-file and non-existent markers are skipped.
/// This is the marker check used by new linear discovery.
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
        use pretty_assertions::assert_eq;

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
