//! Logic for examining individual directories for configuration marker files.
//!
//! This module provides the [`DiscoveryProbe`] trait and implementations for
//! detecting marker files in a single directory. It abstracts the filesystem
//! mechanics of checking for supported filename patterns and structured
//! formats.
//!
//! # Probes
//!
//! - [`VaultRootProbe`]: Searches for root markers (e.g., `lithos.toml`,
//!   `.lithos/config.toml`) that establish a vault boundary.
//! - [`GlobalRootProbe`]: Searches for global configuration markers (e.g.,
//!   `lithos/config.json`) in standard system or user locations.
//!
//! # Patterns and Formats
//!
//! Probes use [`MarkerPattern`]s combined with supported
//! [`StructuredFileFormat`] extensions to generate and check candidate paths.

use std::path::Path;

use super::{engine::DiscoveredMarker, error::DiscoveryError};
use crate::{
    discovery::service::CandidatePath,
    fs::{
        format::StructuredFileFormat,
        path::{DirPath, FilePath},
    },
};

/// Trait for types that can examine a directory and return discovery results.
///
/// Implementations of this trait define the logic for finding specific types
/// of marker files (e.g., vault markers vs. global markers) in a given
/// directory.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) trait DiscoveryProbe<Output> {
    /// Examine the given directory and return any discovered markers.
    ///
    /// # Errors
    ///
    /// Returns a [`DiscoveryError`] if the directory cannot be read or if
    /// discovered markers cannot be canonicalized.
    fn probe(&self, dir: &Path) -> Result<Option<Output>, DiscoveryError>;
}

/// Probes directories for vault root markers using standard patterns.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) struct VaultRootProbe;

impl DiscoveryProbe<Vec<DiscoveredMarker>> for VaultRootProbe {
    fn probe(
        &self,
        dir: &Path,
    ) -> Result<Option<Vec<DiscoveredMarker>>, DiscoveryError> {
        let markers: Vec<DiscoveredMarker> = ROOT_MARKER_FILES
            .iter()
            .flat_map(|pattern| {
                StructuredFileFormat::PRECEDENCE
                    .iter()
                    .map(move |format| (pattern, format))
            })
            .filter_map(|(pattern, format)| {
                let mut path = dir.join(pattern.prefix);
                path.set_extension(format.extension());

                if !path.is_file() {
                    return None;
                }

                Some(marker_from_path(dir, &path, *format))
            })
            .collect::<Result<Vec<_>, _>>()?;

        if markers.is_empty() {
            Ok(None)
        } else {
            Ok(Some(markers))
        }
    }
}

/// Probes directories for global configuration markers.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) struct GlobalRootProbe;

impl DiscoveryProbe<Vec<DiscoveredMarker>> for GlobalRootProbe {
    fn probe(
        &self,
        dir: &Path,
    ) -> Result<Option<Vec<DiscoveredMarker>>, DiscoveryError> {
        let markers: Vec<DiscoveredMarker> = GLOBAL_MARKER_FILES
            .iter()
            .flat_map(|pattern| {
                StructuredFileFormat::PRECEDENCE
                    .iter()
                    .map(move |format| (pattern, format))
            })
            .filter_map(|(pattern, format)| {
                let mut path = dir.join(pattern.prefix);
                path.set_extension(format.extension());

                if !path.is_file() {
                    return None;
                }

                Some(marker_from_path(dir, &path, *format))
            })
            .collect::<Result<Vec<_>, _>>()?;

        if markers.is_empty() {
            Ok(None)
        } else {
            Ok(Some(markers))
        }
    }
}

/// Naming pattern used to identify a marker file.
///
/// A pattern consists of a filename prefix and a nesting flag. During probing,
/// the engine appends supported format extensions (e.g., `.toml`, `.json`) to
/// the prefix to construct candidate paths.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) struct MarkerPattern {
    /// The filename prefix (e.g. `lithos` or `.lithos`).
    pub(crate) prefix: &'static str,
    /// Whether the marker is nested inside a configuration directory.
    ///
    /// If true, the marker is expected to be at `{dir}/{prefix}.{ext}`.
    pub(crate) is_nested: bool,
}

/// Standard marker patterns used for vault root resolution.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) const ROOT_MARKER_FILES: &[MarkerPattern] = &[
    MarkerPattern {
        prefix: "lithos",
        is_nested: false,
    },
    MarkerPattern {
        prefix: ".lithos",
        is_nested: false,
    },
    MarkerPattern {
        prefix: ".lithos/config",
        is_nested: true,
    },
];

/// Standard marker patterns used for global config resolution.
#[allow(dead_code, reason = "Phase-2 seam; wired into global discovery")]
pub(crate) const GLOBAL_MARKER_FILES: &[MarkerPattern] = &[
    MarkerPattern {
        prefix: "lithos",
        is_nested: false,
    },
    MarkerPattern {
        prefix: "lithos/config",
        is_nested: true,
    },
];

/// Infallible directory probe that checks for marker files by iterating
/// patterns × format precedence.
///
/// Unlike [`VaultRootProbe`] and [`GlobalRootProbe`], this probe does not
/// canonicalize paths or return errors — all input paths are pre-validated
/// before reaching it.
#[allow(
    dead_code,
    reason = "Added for new processor; wired into discovery later"
)]
pub(crate) struct FolderProbe {
    /// Ordered marker patterns to search for.
    pub(crate) patterns: &'static [super::policy::MarkerPattern],
}

#[allow(
    dead_code,
    reason = "Added for new processor; wired into discovery later"
)]
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

fn marker_from_path(
    base: &Path,
    path: &Path,
    format: StructuredFileFormat,
) -> Result<DiscoveredMarker, DiscoveryError> {
    path.canonicalize()
        .map(|canonical| DiscoveredMarker {
            base: base.to_path_buf(),
            path: canonical,
            format,
        })
        .map_err(|source| DiscoveryError::CanonicalizePath {
            path: path.to_path_buf(),
            source,
        })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use tempfile::tempdir;

    use super::*;

    fn write_marker(root: &Path, relative: &str) -> PathBuf {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create marker parent dir");
        }
        fs::write(&path, "").expect("write marker file");
        path
    }

    mod vault_root_probe {
        use super::*;

        #[test]
        fn returns_all_markers_when_multiple_exist() {
            let root = tempdir().expect("root");
            write_marker(root.path(), ".lithos.toml");
            write_marker(root.path(), "lithos.toml");

            let probe = VaultRootProbe;
            let result = probe.probe(root.path());

            assert!(
                result.is_ok(),
                "Expected marker lookup success, got: {:?}",
                result.as_ref().err()
            );
            let markers = result.expect("checked ok").expect("markers exist");

            assert_eq!(markers.len(), 2);
        }

        #[test]
        fn returns_root_marker_before_hidden_marker() {
            let root = tempdir().expect("root");
            write_marker(root.path(), ".lithos.toml");
            let expected_path = write_marker(root.path(), "lithos.toml");

            let probe = VaultRootProbe;
            let result = probe.probe(root.path());

            assert!(
                result.is_ok(),
                "Expected marker lookup success, got: {:?}",
                result.as_ref().err()
            );
            let markers = result.expect("checked ok").expect("markers exist");

            assert_eq!(
                markers.first().expect("first marker").path,
                expected_path.canonicalize().expect("canonical marker")
            );
        }

        #[test]
        fn returns_hidden_marker_after_root_marker() {
            let root = tempdir().expect("root");
            let hidden_path = write_marker(root.path(), ".lithos.toml");
            write_marker(root.path(), "lithos.toml");

            let probe = VaultRootProbe;
            let result = probe.probe(root.path());

            assert!(
                result.is_ok(),
                "Expected marker lookup success, got: {:?}",
                result.as_ref().err()
            );
            let markers = result.expect("checked ok").expect("markers exist");

            assert_eq!(
                markers.get(1).expect("second marker").path,
                hidden_path.canonicalize().expect("canonical marker")
            );
        }

        #[test]
        fn returns_config_directory_marker_when_other_markers_absent() {
            let root = tempdir().expect("root");
            let expected_path =
                write_marker(root.path(), ".lithos/config.toml");

            let probe = VaultRootProbe;
            let result = probe.probe(root.path());

            assert!(
                result.is_ok(),
                "Expected marker lookup success, got: {:?}",
                result.as_ref().err()
            );
            let markers = result.expect("checked ok").expect("markers exist");

            assert_eq!(markers.len(), 1);
            assert_eq!(
                markers.first().expect("first marker").path,
                expected_path.canonicalize().expect("canonical marker")
            );
        }

        #[test]
        fn returns_multiple_formats_correctly() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.json");
            write_marker(root.path(), "lithos.toml");

            let probe = VaultRootProbe;
            let result = probe.probe(root.path());

            assert!(
                result.is_ok(),
                "Expected marker lookup success, got: {:?}",
                result.as_ref().err()
            );
            let markers = result.expect("checked ok").expect("markers exist");

            assert_eq!(markers.len(), 2);
        }

        #[test]
        fn returns_toml_format_before_json_format() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.json");
            write_marker(root.path(), "lithos.toml");

            let probe = VaultRootProbe;
            let result = probe.probe(root.path());

            assert!(
                result.is_ok(),
                "Expected marker lookup success, got: {:?}",
                result.as_ref().err()
            );
            let markers = result.expect("checked ok").expect("markers exist");

            assert_eq!(
                markers.first().expect("first marker").format,
                StructuredFileFormat::Toml
            );
        }

        #[test]
        fn returns_json_format_after_toml_format() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.json");
            write_marker(root.path(), "lithos.toml");

            let probe = VaultRootProbe;
            let result = probe.probe(root.path());

            assert!(
                result.is_ok(),
                "Expected marker lookup success, got: {:?}",
                result.as_ref().err()
            );
            let markers = result.expect("checked ok").expect("markers exist");

            assert_eq!(
                markers.get(1).expect("second marker").format,
                StructuredFileFormat::Json
            );
        }

        #[test]
        fn returns_none_when_no_marker_file_exists() {
            let root = tempdir().expect("root");

            let probe = VaultRootProbe;
            let markers =
                probe.probe(root.path()).expect("marker lookup succeeds");

            assert_eq!(markers, None);
        }
    }

    mod global_root_probe {
        use super::*;

        #[test]
        fn returns_markers_when_global_config_files_exist() {
            let root = tempdir().expect("root");
            let expected_path = write_marker(root.path(), "lithos.toml");

            let probe = GlobalRootProbe;
            let result = probe.probe(root.path());

            assert!(
                result.is_ok(),
                "Expected marker lookup success, got: {:?}",
                result.as_ref().err()
            );
            let markers = result.expect("checked ok").expect("markers exist");

            assert_eq!(markers.len(), 1);
            assert_eq!(
                markers.first().expect("marker").path,
                expected_path.canonicalize().expect("canonical marker")
            );
        }

        #[test]
        fn returns_nested_global_config_marker() {
            let root = tempdir().expect("root");
            let expected_path = write_marker(root.path(), "lithos/config.toml");

            let probe = GlobalRootProbe;
            let result = probe.probe(root.path());

            assert!(
                result.is_ok(),
                "Expected marker lookup success, got: {:?}",
                result.as_ref().err()
            );
            let markers = result.expect("checked ok").expect("markers exist");

            assert_eq!(markers.len(), 1);
            assert_eq!(
                markers.first().expect("marker").path,
                expected_path.canonicalize().expect("canonical marker")
            );
        }
    }
}
