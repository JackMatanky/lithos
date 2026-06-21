//! Logic for examining individual directories for configuration marker files.
//!
//! This module provides the [`FolderProbe`] for detecting marker files in a
//! single directory. It abstracts the filesystem mechanics of checking for
//! supported filename patterns and structured formats.

use std::path::Path;

use crate::{
    discovery::service::CandidatePath,
    fs::{
        format::StructuredFileFormat,
        path::{DirPath, FilePath},
    },
};

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
