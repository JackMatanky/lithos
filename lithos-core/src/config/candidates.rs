//! Local config candidate discovery and deterministic selection.
//!
//! This module provides tools for finding existing configuration files across
//! supported formats and selecting a single winner based on precedence and
//! format stability rules.

use std::{io, path::Path};

use crate::{
    config::{
        diagnostics::{ConfigWarning, FormatDiscoveryWarning},
        location::{ConfigLocation, LocalConfigLocation},
        root::DiscoveredConfigFile,
    },
    fs::format::StructuredFileFormat,
};

/// Result of selecting a single candidate from multiple discovered formats.
///
/// Provides a strongly-typed outcome of the deterministic selection logic,
/// carrying both the winner and any ambiguity warning generated.
#[allow(
    dead_code,
    reason = "Phase-2 seam; wired in once pipeline integration lands"
)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ConfigSelectionResult {
    /// The final selected config candidate.
    pub(crate) candidate: DiscoveredConfigFile,
    /// A structured warning if multiple formats were available.
    pub(crate) warning: Option<ConfigWarning>,
}

/// Finds all existing local config candidates for a logical location.
///
/// This function probes the filesystem for all supported structured formats
/// at the specified [`LocalConfigLocation`]. It returns a list of all files
/// that exist, ordered by the precedence defined in
/// [`StructuredFileFormat::PRECEDENCE`].
///
/// # Errors
///
/// Returns [`io::Result`] if path canonicalization fails for any discovered
/// file.
#[allow(
    dead_code,
    reason = "Phase-2 seam; wired in once pipeline integration lands"
)]
pub(crate) fn find_local_config_candidates(
    root: &Path,
    location: LocalConfigLocation,
) -> io::Result<Vec<DiscoveredConfigFile>> {
    StructuredFileFormat::PRECEDENCE
        .into_iter()
        .filter_map(|format| {
            let path = location.candidate_path(root, format);
            path.exists().then_some((format, path))
        })
        .map(|(format, path)| {
            path.canonicalize().map(|canonical| DiscoveredConfigFile {
                location: ConfigLocation::Local(location),
                base: root.to_path_buf(),
                path: canonical,
                format,
            })
        })
        .collect()
}

/// Selects a single config candidate using precedence and format-stability.
///
/// This is a deterministic decision function that chooses the final candidate
/// from a list of discovered files.
///
/// ### Selection Strategy
///
/// 1. **Stability**: If `persisted_format` is provided and a candidate with
///    that exact format exists, it is selected regardless of precedence rank.
/// 2. **Precedence**: If no persisted match exists, the candidate with the
///    highest precedence rank (defined by [`StructuredFileFormat::rank`]) is
///    selected.
///
/// ### Ambiguity Handling
///
/// If multiple candidates are present, a [`ConfigWarning::Format`] warning
/// is generated and emitted via `tracing::warn!`, even if a successful
/// selection is made. This warning is also returned inside the
/// [`ConfigSelectionResult`] for deterministic testing and CLI reporting.
///
/// # Examples
///
/// ```ignore
/// let result = select_config_candidate(candidates, Some(StructuredFileFormat::Json));
/// if let Some(selection) = result {
///     assert_eq!(selection.candidate.format, StructuredFileFormat::Json);
/// }
/// ```
#[allow(
    dead_code,
    reason = "Phase-2 seam; wired in once pipeline integration lands"
)]
pub(crate) fn select_config_candidate(
    mut candidates: Vec<DiscoveredConfigFile>,
    persisted_format: Option<StructuredFileFormat>,
) -> Option<ConfigSelectionResult> {
    if candidates.is_empty() {
        return None;
    }

    if candidates.len() == 1 {
        return candidates.pop().map(|candidate| ConfigSelectionResult {
            candidate,
            warning: None,
        });
    }

    let selected_idx = persisted_format
        .and_then(|fmt| candidates.iter().position(|c| c.format == fmt))
        .unwrap_or_else(|| {
            candidates
                .iter()
                .enumerate()
                .min_by_key(|(_, c)| c.format.rank())
                .map_or(0, |(idx, _)| idx)
        });

    let candidate = candidates.swap_remove(selected_idx);

    // Consume the remaining candidates to build the warning without redundant
    // clones
    let mut paths: Vec<_> = candidates.into_iter().map(|c| c.path).collect();
    paths.push(candidate.path.clone());

    let warning = ConfigWarning::Format(FormatDiscoveryWarning::Ambiguity {
        base: candidate.base.clone(),
        candidates: paths,
    });

    tracing::warn!(
        ?warning,
        "Multiple configuration formats discovered for the same location."
    );

    Some(ConfigSelectionResult {
        candidate,
        warning: Some(warning),
    })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use tempfile::{TempDir, tempdir};

    use super::*;

    mod fixtures {
        use super::*;

        pub(super) fn local_candidate(
            base: &Path,
            file_name: &str,
            format: StructuredFileFormat,
        ) -> DiscoveredConfigFile {
            DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::RootConfigFile,
                ),
                base: base.to_path_buf(),
                path: base.join(file_name),
                format,
            }
        }

        pub(super) fn write_file(root: &Path, relative: &str) -> PathBuf {
            let path = root.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create config parent dir");
            }
            fs::write(&path, "").expect("write config file");
            path
        }

        pub(super) struct CurrentDirGuard {
            original: PathBuf,
        }

        impl CurrentDirGuard {
            #[expect(
                clippy::disallowed_methods,
                reason = "Relative path tests need to change current dir"
            )]
            pub(super) fn enter(path: &Path) -> Self {
                let original =
                    std::env::current_dir().expect("read original cwd");
                std::env::set_current_dir(path).expect("set test cwd");
                Self {
                    original,
                }
            }
        }

        impl Drop for CurrentDirGuard {
            fn drop(&mut self) {
                let _ = std::env::set_current_dir(&self.original);
            }
        }

        pub(super) fn temp_vault() -> TempDir {
            tempdir().expect("create temp vault")
        }
    }

    mod find_local_config_candidates {
        use super::{fixtures::*, *};

        #[test]
        fn returns_empty_vec_when_no_files_exist() {
            let root = temp_vault();

            let got = find_local_config_candidates(
                root.path(),
                LocalConfigLocation::RootConfigFile,
            )
            .expect("find candidates");

            assert!(got.is_empty());
        }

        #[test]
        fn returns_candidate_when_single_format_exists() {
            let root = temp_vault();
            write_file(root.path(), "lithos.toml");

            let got = find_local_config_candidates(
                root.path(),
                LocalConfigLocation::RootConfigFile,
            )
            .expect("find candidates");

            assert_eq!(got.len(), 1);
        }

        #[test]
        fn returns_candidates_ordered_by_format_precedence() {
            let root = temp_vault();
            write_file(root.path(), "lithos.toml");
            write_file(root.path(), "lithos.json");

            let got = find_local_config_candidates(
                root.path(),
                LocalConfigLocation::RootConfigFile,
            )
            .expect("find candidates");
            let formats: Vec<_> =
                got.iter().map(|candidate| candidate.format).collect();

            assert_eq!(formats, vec![
                StructuredFileFormat::Toml,
                StructuredFileFormat::Json
            ]);
        }

        #[test]
        fn returns_root_config_path_for_root_location() {
            let root = temp_vault();
            let config_path = write_file(root.path(), "lithos.toml");

            let got = find_local_config_candidates(
                root.path(),
                LocalConfigLocation::RootConfigFile,
            )
            .expect("find candidates");
            let candidate = got.first().expect("candidate exists");

            assert_eq!(
                candidate.path,
                config_path.canonicalize().expect("canonical config path")
            );
        }

        #[test]
        fn returns_hidden_root_path_for_hidden_location() {
            let root = temp_vault();
            let config_path = write_file(root.path(), ".lithos.toml");

            let got = find_local_config_candidates(
                root.path(),
                LocalConfigLocation::HiddenRootConfigFile,
            )
            .expect("find candidates");
            let candidate = got.first().expect("candidate exists");

            assert_eq!(
                candidate.path,
                config_path.canonicalize().expect("canonical config path")
            );
        }

        #[test]
        fn returns_config_directory_path_for_directory_location() {
            let root = temp_vault();
            let config_path = write_file(root.path(), ".lithos/config.toml");

            let got = find_local_config_candidates(
                root.path(),
                LocalConfigLocation::ConfigDirectoryFile,
            )
            .expect("find candidates");
            let candidate = got.first().expect("candidate exists");

            assert_eq!(
                candidate.path,
                config_path.canonicalize().expect("canonical config path")
            );
        }

        #[test]
        fn returns_absolute_paths_when_root_is_relative() {
            let root = temp_vault();
            write_file(root.path(), "lithos.toml");
            let parent = root.path().parent().expect("temp root has parent");
            let _guard = CurrentDirGuard::enter(parent);

            let relative_root =
                Path::new(root.path().file_name().expect("temp root has name"));

            let got = find_local_config_candidates(
                relative_root,
                LocalConfigLocation::RootConfigFile,
            )
            .expect("find candidates");
            let candidate = got.first().expect("candidate exists");

            assert!(candidate.path.is_absolute());
        }
    }

    mod select_config_candidate {
        use super::{fixtures::*, *};

        #[test]
        fn returns_none_when_candidates_are_empty() {
            let got = select_config_candidate(vec![], None);
            assert!(got.is_none());
        }

        #[test]
        fn returns_candidate_when_single_candidate_exists() {
            let base = Path::new("/vault");
            let candidate = local_candidate(
                base,
                "lithos.toml",
                StructuredFileFormat::Toml,
            );

            let got = select_config_candidate(vec![candidate], None)
                .expect("expected a candidate to be selected");

            assert_eq!(got.candidate.format, StructuredFileFormat::Toml);
        }

        #[test]
        fn returns_no_warning_when_single_candidate_exists() {
            let base = PathBuf::from("/vault");
            let candidate = local_candidate(
                &base,
                "lithos.toml",
                StructuredFileFormat::Toml,
            );

            let got = select_config_candidate(vec![candidate], None)
                .expect("expected a candidate to be selected");

            assert_eq!(got.warning, None);
        }

        #[test]
        fn returns_persisted_format_when_match_exists() {
            let base = PathBuf::from("/vault");
            let toml_candidate = local_candidate(
                &base,
                "lithos.toml",
                StructuredFileFormat::Toml,
            );
            let json_candidate = local_candidate(
                &base,
                "lithos.json",
                StructuredFileFormat::Json,
            );

            let got = select_config_candidate(
                vec![toml_candidate, json_candidate],
                Some(StructuredFileFormat::Json),
            )
            .expect("expected a candidate to be selected");

            assert_eq!(got.candidate.format, StructuredFileFormat::Json);
        }

        #[test]
        fn returns_warning_when_multiple_formats_exist() {
            let base = PathBuf::from("/vault");
            let toml_candidate = local_candidate(
                &base,
                "lithos.toml",
                StructuredFileFormat::Toml,
            );
            let json_candidate = local_candidate(
                &base,
                "lithos.json",
                StructuredFileFormat::Json,
            );

            let got = select_config_candidate(
                vec![toml_candidate, json_candidate],
                Some(StructuredFileFormat::Json),
            )
            .expect("expected a candidate to be selected");

            let expected_warning =
                ConfigWarning::Format(FormatDiscoveryWarning::Ambiguity {
                    base,
                    candidates: vec![
                        PathBuf::from("/vault/lithos.toml"),
                        PathBuf::from("/vault/lithos.json"),
                    ],
                });
            assert_eq!(got.warning, Some(expected_warning));
        }

        #[test]
        fn returns_highest_precedence_when_no_persisted_match_exists() {
            let base = PathBuf::from("/vault");
            let yaml_candidate = local_candidate(
                &base,
                "lithos.yaml",
                StructuredFileFormat::Yaml,
            );
            let json_candidate = local_candidate(
                &base,
                "lithos.json",
                StructuredFileFormat::Json,
            );

            let got = select_config_candidate(
                vec![yaml_candidate, json_candidate],
                Some(StructuredFileFormat::Toml),
            )
            .expect("expected a candidate to be selected");

            assert_eq!(got.candidate.format, StructuredFileFormat::Json);
        }

        #[test]
        fn returns_warning_candidates_in_input_order() {
            let base = PathBuf::from("/vault");
            let yaml_candidate = local_candidate(
                &base,
                "lithos.yaml",
                StructuredFileFormat::Yaml,
            );
            let json_candidate = local_candidate(
                &base,
                "lithos.json",
                StructuredFileFormat::Json,
            );

            let got = select_config_candidate(
                vec![yaml_candidate, json_candidate],
                Some(StructuredFileFormat::Toml),
            )
            .expect("expected a candidate to be selected");

            let expected_warning =
                ConfigWarning::Format(FormatDiscoveryWarning::Ambiguity {
                    base,
                    candidates: vec![
                        PathBuf::from("/vault/lithos.yaml"),
                        PathBuf::from("/vault/lithos.json"),
                    ],
                });
            assert_eq!(got.warning, Some(expected_warning));
        }

        #[test]
        fn returns_no_warning_when_directly_constructed_with_none() {
            let base = PathBuf::from("/vault");
            let candidate = local_candidate(
                &base,
                "lithos.toml",
                StructuredFileFormat::Toml,
            );
            let result = ConfigSelectionResult {
                candidate,
                warning: None,
            };
            assert!(result.warning.is_none());
        }
    }
}
