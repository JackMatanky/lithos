use std::path::Path;

use super::{engine::FoundRootMarker, error::DiscoveryError};
use crate::fs::format::StructuredFileFormat;

#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) trait DiscoveryProbe<Output> {
    fn probe(&self, dir: &Path) -> Result<Option<Output>, DiscoveryError>;
}

#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) struct MarkerPattern {
    pub(crate) prefix: &'static str,
    pub(crate) is_nested: bool,
}

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

#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) struct VaultRootProbe;

impl DiscoveryProbe<FoundRootMarker> for VaultRootProbe {
    fn probe(
        &self,
        dir: &Path,
    ) -> Result<Option<FoundRootMarker>, DiscoveryError> {
        ROOT_MARKER_FILES
            .iter()
            .flat_map(|pattern| {
                StructuredFileFormat::PRECEDENCE
                    .iter()
                    .map(move |format| (pattern, format))
            })
            .find_map(|(pattern, format)| {
                let ext = format.extension();
                let filename = format!("{}.{}", pattern.prefix, ext);
                let path = dir.join(&filename);

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
            .transpose()
    }
}

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
        fn returns_root_config_marker_first() {
            let root = tempdir().expect("root");
            write_marker(root.path(), ".lithos.toml");
            let expected_path = write_marker(root.path(), "lithos.toml");

            let probe = VaultRootProbe;
            let marker = probe
                .probe(root.path())
                .expect("marker lookup succeeds")
                .expect("marker exists");

            assert_eq!(
                marker.path,
                expected_path.canonicalize().expect("canonical marker")
            );
        }

        #[test]
        fn returns_hidden_root_marker_when_root_marker_absent() {
            let root = tempdir().expect("root");
            let expected_path = write_marker(root.path(), ".lithos.toml");

            let probe = VaultRootProbe;
            let marker = probe
                .probe(root.path())
                .expect("marker lookup succeeds")
                .expect("marker exists");

            assert_eq!(
                marker.path,
                expected_path.canonicalize().expect("canonical marker")
            );
        }

        #[test]
        fn returns_config_directory_marker_when_other_markers_absent() {
            let root = tempdir().expect("root");
            let expected_path =
                write_marker(root.path(), ".lithos/config.toml");

            let probe = VaultRootProbe;
            let marker = probe
                .probe(root.path())
                .expect("marker lookup succeeds")
                .expect("marker exists");

            assert_eq!(
                marker.path,
                expected_path.canonicalize().expect("canonical marker")
            );
        }

        #[test]
        fn returns_toml_marker_before_json_marker() {
            let root = tempdir().expect("root");
            write_marker(root.path(), "lithos.json");
            write_marker(root.path(), "lithos.toml");

            let probe = VaultRootProbe;
            let marker = probe
                .probe(root.path())
                .expect("marker lookup succeeds")
                .expect("marker exists");

            assert_eq!(marker.format, StructuredFileFormat::Toml);
        }

        #[test]
        fn returns_none_when_no_marker_file_exists() {
            let root = tempdir().expect("root");

            let probe = VaultRootProbe;
            let marker =
                probe.probe(root.path()).expect("marker lookup succeeds");

            assert_eq!(marker, None);
        }
    }
}
