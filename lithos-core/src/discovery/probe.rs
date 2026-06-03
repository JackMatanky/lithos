//! Logic for examining individual directories for configuration marker files.

use std::path::Path;

use super::{engine::FoundRootMarker, error::DiscoveryError};
use crate::fs::format::StructuredFileFormat;

/// Trait for types that can examine a directory and return discovery results.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) trait DiscoveryProbe<Output> {
    /// Examine the given directory and return any discovered markers.
    fn probe(&self, dir: &Path) -> Result<Option<Output>, DiscoveryError>;
}

/// Naming pattern used to identify a marker file.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) struct MarkerPattern {
    /// The filename prefix (e.g. `lithos` or `.lithos`).
    pub(crate) prefix: &'static str,
    /// Whether the marker is nested inside a configuration directory.
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

/// Probes directories for vault root markers using standard patterns.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) struct VaultRootProbe;

impl DiscoveryProbe<Vec<FoundRootMarker>> for VaultRootProbe {
    /// Examine the given directory and return all discovered vault root
    /// markers.
    ///
    /// This implementation uses an efficient, zero-allocation-per-extension
    /// path construction strategy by reusing a single `PathBuf` and
    /// mutating it.
    fn probe(
        &self,
        dir: &Path,
    ) -> Result<Option<Vec<FoundRootMarker>>, DiscoveryError> {
        let markers: Vec<FoundRootMarker> = ROOT_MARKER_FILES
            .iter()
            .flat_map(|pattern| {
                StructuredFileFormat::PRECEDENCE
                    .iter()
                    .map(move |format| (pattern, format))
            })
            .filter_map(|(pattern, format)| {
                let ext = format.extension();
                let mut path = dir.join(pattern.prefix);
                path.set_extension(ext);

                if !path.is_file() {
                    return None;
                }

                Some(
                    path.canonicalize()
                        .map(|canonical| FoundRootMarker {
                            base: dir.to_path_buf(),
                            path: canonical,
                            format: *format,
                        })
                        .map_err(|source| DiscoveryError::CanonicalizePath {
                            path,
                            source,
                        }),
                )
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
pub(crate) struct GlobalConfigProbe;

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
            let hidden_path = write_marker(root.path(), ".lithos.toml");
            let expected_path = write_marker(root.path(), "lithos.toml");

            let probe = VaultRootProbe;
            let markers = probe
                .probe(root.path())
                .expect("marker lookup succeeds")
                .expect("markers exist");

            assert_eq!(markers.len(), 2);
            assert_eq!(
                markers.first().unwrap().path,
                expected_path.canonicalize().expect("canonical marker")
            );
            assert_eq!(
                markers.get(1).unwrap().path,
                hidden_path.canonicalize().expect("canonical marker")
            );
        }

        #[test]
        fn returns_config_directory_marker_when_other_markers_absent() {
            let root = tempdir().expect("root");
            let expected_path =
                write_marker(root.path(), ".lithos/config.toml");

            let probe = VaultRootProbe;
            let markers = probe
                .probe(root.path())
                .expect("marker lookup succeeds")
                .expect("markers exist");

            assert_eq!(markers.len(), 1);
            assert_eq!(
                markers.first().unwrap().path,
                expected_path.canonicalize().expect("canonical marker")
            );
        }

        #[test]
        fn returns_multiple_formats_correctly() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.json");
            write_marker(root.path(), "lithos.toml");

            let probe = VaultRootProbe;
            let markers = probe
                .probe(root.path())
                .expect("marker lookup succeeds")
                .expect("markers exist");

            assert_eq!(markers.len(), 2);
            // Since TOML is higher precedence in StructuredFileFormat, it comes
            // first due to PRECEDENCE array iteration order.
            assert_eq!(
                markers.first().unwrap().format,
                StructuredFileFormat::Toml
            );
            assert_eq!(
                markers.get(1).unwrap().format,
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
}
