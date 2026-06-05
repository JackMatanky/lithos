//! Logic for examining individual directories for configuration marker files.

use std::path::Path;

use super::{
    diagnostics::{DiscoveryWarning, GlobalDiscoveryWarning},
    engine::FoundRootMarker,
    error::DiscoveryError,
};
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

/// Standard marker patterns used for global config resolution.
#[allow(dead_code, reason = "Phase-2 seam; wired into global discovery")]
pub(crate) const GLOBAL_MARKER_FILES: &[MarkerPattern] = &[MarkerPattern {
    prefix: "lithos",
    is_nested: false,
}];

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

impl GlobalConfigProbe {
    /// Examine the given directory and append non-fatal warnings for corrected
    /// global config filename casing.
    pub(crate) fn probe_with_warnings(
        dir: &Path,
        warnings: &mut Vec<DiscoveryWarning>,
    ) -> Result<Option<Vec<FoundRootMarker>>, DiscoveryError> {
        let mut markers = Self::probe_exact(dir)?;
        markers.extend(Self::probe_mis_cased(dir, warnings)?);
        let mut unique = Vec::new();
        for marker in markers {
            if unique
                .iter()
                .any(|existing: &FoundRootMarker| existing.path == marker.path)
            {
                continue;
            }
            unique.push(marker);
        }

        if unique.is_empty() {
            Ok(None)
        } else {
            Ok(Some(unique))
        }
    }

    fn probe_exact(dir: &Path) -> Result<Vec<FoundRootMarker>, DiscoveryError> {
        GLOBAL_MARKER_FILES
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

                Some(Self::marker_from_path(dir, &path, *format))
            })
            .collect()
    }

    fn probe_mis_cased(
        dir: &Path,
        warnings: &mut Vec<DiscoveryWarning>,
    ) -> Result<Vec<FoundRootMarker>, DiscoveryError> {
        if !dir.is_dir() {
            return Ok(vec![]);
        }

        let entries = std::fs::read_dir(dir).map_err(|source| {
            DiscoveryError::ReadDirectory {
                path: dir.to_path_buf(),
                source,
            }
        })?;

        let mut markers = Vec::new();
        for entry in entries {
            let path = entry
                .map_err(|source| DiscoveryError::ReadDirectory {
                    path: dir.to_path_buf(),
                    source,
                })?
                .path();
            let Some((requested, format)) =
                Self::mis_cased_candidate(dir, &path)
            else {
                continue;
            };

            let marker = Self::marker_from_path(dir, &path, format)?;
            warnings.push(DiscoveryWarning::GlobalResolution(
                GlobalDiscoveryWarning::CaseCorrection {
                    requested,
                    resolved: marker.path.clone(),
                },
            ));
            markers.push(marker);
        }

        Ok(markers)
    }

    fn mis_cased_candidate(
        dir: &Path,
        path: &Path,
    ) -> Option<(std::path::PathBuf, StructuredFileFormat)> {
        if !path.is_file() {
            return None;
        }
        let file_name = path.file_name().and_then(|name| name.to_str())?;

        for pattern in GLOBAL_MARKER_FILES {
            for format in StructuredFileFormat::PRECEDENCE {
                let expected =
                    format!("{}.{}", pattern.prefix, format.extension());
                if file_name == expected
                    || !file_name.eq_ignore_ascii_case(&expected)
                {
                    continue;
                }

                return Some((dir.join(expected), format));
            }
        }

        None
    }

    fn marker_from_path(
        base: &Path,
        path: &Path,
        format: StructuredFileFormat,
    ) -> Result<FoundRootMarker, DiscoveryError> {
        path.canonicalize()
            .map(|canonical| FoundRootMarker {
                base: base.to_path_buf(),
                path: canonical,
                format,
            })
            .map_err(|source| DiscoveryError::CanonicalizePath {
                path: path.to_path_buf(),
                source,
            })
    }
}

impl DiscoveryProbe<Vec<FoundRootMarker>> for GlobalConfigProbe {
    /// Examine the given directory and return all discovered global config
    /// markers.
    fn probe(
        &self,
        dir: &Path,
    ) -> Result<Option<Vec<FoundRootMarker>>, DiscoveryError> {
        Self::probe_with_warnings(dir, &mut Vec::new())
    }
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

    mod global_config_probe {
        use super::*;
        use crate::discovery::diagnostics::{
            DiscoveryWarning, GlobalDiscoveryWarning,
        };

        #[test]
        fn returns_markers_when_global_config_files_exist() {
            let root = tempdir().expect("root");
            let expected_path = write_marker(root.path(), "lithos.toml");

            let probe = GlobalConfigProbe;
            let markers = probe
                .probe(root.path())
                .expect("marker lookup succeeds")
                .expect("markers exist");

            assert_eq!(markers.len(), 1);
            assert_eq!(
                markers.first().expect("marker").path,
                expected_path.canonicalize().expect("canonical marker")
            );
        }

        #[test]
        fn returns_warning_when_global_config_file_is_mis_cased() {
            let root = tempdir().expect("root");
            let expected_path = write_marker(root.path(), "Lithos.TOML");
            let requested_path = root.path().join("lithos.toml");
            let mut warnings = Vec::new();

            let markers = GlobalConfigProbe::probe_with_warnings(
                root.path(),
                &mut warnings,
            )
            .expect("marker lookup succeeds")
            .expect("markers exist");

            assert_eq!(markers.len(), 1);
            assert_eq!(
                markers.first().expect("marker").path,
                expected_path.canonicalize().expect("canonical marker")
            );
            assert_eq!(warnings.len(), 1);
            assert_eq!(
                warnings.first(),
                Some(&DiscoveryWarning::GlobalResolution(
                    GlobalDiscoveryWarning::CaseCorrection {
                        requested: requested_path,
                        resolved: expected_path
                            .canonicalize()
                            .expect("canonical marker"),
                    },
                ))
            );
        }

        #[cfg(unix)]
        #[test]
        fn returns_error_when_global_config_directory_cannot_be_read() {
            use std::os::unix::fs::PermissionsExt;

            let root = tempdir().expect("root");
            let original_permissions =
                fs::metadata(root.path()).expect("metadata").permissions();
            fs::set_permissions(root.path(), fs::Permissions::from_mode(0o000))
                .expect("remove permissions");

            let error = GlobalConfigProbe::probe_with_warnings(
                root.path(),
                &mut Vec::new(),
            )
            .expect_err("unreadable directory should fail");

            fs::set_permissions(root.path(), original_permissions)
                .expect("restore permissions");
            assert!(matches!(
                error,
                DiscoveryError::ReadDirectory { path, .. }
                    if path == root.path()
            ));
        }
    }
}
