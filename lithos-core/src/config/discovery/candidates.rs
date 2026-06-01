//! Local config candidate discovery.

use std::{io, path::Path};

use super::{
    contracts::{
        ConfigSelectionResult, DiscoveredConfigFile, DiscoveryWarning,
    },
    location::{ConfigLocation, LocalConfigLocation},
};
use crate::fs::format::StructuredFileFormat;

/// Finds all existing local config candidates for a location.
///
/// Enumerates candidates by iterating through structured format precedence.
#[allow(
    dead_code,
    reason = "Phase-2 seam introduced before full pipeline integration"
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

/// Selects a single config candidate with format precedence and stability.
///
/// Prefers `persisted_format` if available and present in `candidates`.
/// Otherwise, falls back to `StructuredFileFormat::PRECEDENCE`.
#[allow(
    dead_code,
    reason = "Phase-2 seam introduced before full pipeline integration"
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

    let base = candidates.first()?.base.clone();
    let paths = candidates.iter().map(|c| c.path.clone()).collect::<Vec<_>>();
    let warning = DiscoveryWarning::FormatAmbiguity {
        base,
        candidates: paths,
    };

    tracing::warn!(
        ?warning,
        "Multiple configuration formats discovered for the same location."
    );

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

    Some(ConfigSelectionResult {
        candidate,
        warning: Some(warning),
    })
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    mod lookup {
        use super::*;

        #[test]
        fn returns_empty_vec_when_no_files_exist() {
            let root = tempdir().unwrap();
            let location = LocalConfigLocation::RootConfigFile;

            let got =
                find_local_config_candidates(root.path(), location).unwrap();

            assert!(got.is_empty());
        }

        #[test]
        fn returns_single_candidate_when_one_format_exists() {
            let root = tempdir().unwrap();
            let config_path = root.path().join("lithos.toml");
            std::fs::write(&config_path, "").unwrap();

            let location = LocalConfigLocation::RootConfigFile;
            let got =
                find_local_config_candidates(root.path(), location).unwrap();

            assert_eq!(got.len(), 1);
            assert_eq!(
                got.first().unwrap().path,
                config_path.canonicalize().unwrap()
            );
        }

        #[test]
        fn returns_multiple_candidates_ordered_by_precedence() {
            let root = tempdir().unwrap();
            let toml_path = root.path().join("lithos.toml");
            let json_path = root.path().join("lithos.json");
            std::fs::write(&toml_path, "").unwrap();
            std::fs::write(&json_path, "").unwrap();

            let location = LocalConfigLocation::RootConfigFile;
            let got =
                find_local_config_candidates(root.path(), location).unwrap();

            assert_eq!(got.len(), 2);
            assert_eq!(got.first().unwrap().format, StructuredFileFormat::Toml);
            assert_eq!(got.get(1).unwrap().format, StructuredFileFormat::Json);
        }

        #[test]
        fn returns_correct_path_for_root_config_file_location() {
            let root = tempdir().unwrap();
            let config_path = root.path().join("lithos.toml");
            std::fs::write(&config_path, "").unwrap();

            let got = find_local_config_candidates(
                root.path(),
                LocalConfigLocation::RootConfigFile,
            )
            .unwrap();

            assert_eq!(
                got.first().unwrap().path,
                config_path.canonicalize().unwrap()
            );
        }

        #[test]
        fn returns_correct_path_for_hidden_root_config_file_location() {
            let root = tempdir().unwrap();
            let config_path = root.path().join(".lithos.toml");
            std::fs::write(&config_path, "").unwrap();

            let got = find_local_config_candidates(
                root.path(),
                LocalConfigLocation::HiddenRootConfigFile,
            )
            .unwrap();

            assert_eq!(
                got.first().unwrap().path,
                config_path.canonicalize().unwrap()
            );
        }

        #[test]
        fn returns_correct_path_for_config_directory_file_location() {
            let root = tempdir().unwrap();
            std::fs::create_dir(root.path().join(".lithos")).unwrap();
            let config_path = root.path().join(".lithos").join("config.toml");
            std::fs::write(&config_path, "").unwrap();

            let got = find_local_config_candidates(
                root.path(),
                LocalConfigLocation::ConfigDirectoryFile,
            )
            .unwrap();

            assert_eq!(
                got.first().unwrap().path,
                config_path.canonicalize().unwrap()
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Tests need to change working directory to verify \
                      relative path resolution"
        )]
        fn returns_absolute_paths_when_root_is_relative() {
            let root = tempdir().unwrap();
            let config_path = root.path().join("lithos.toml");
            std::fs::write(&config_path, "").unwrap();

            // Change to parent of root to use relative path
            let original_dir = std::env::current_dir().unwrap();
            std::env::set_current_dir(root.path().parent().unwrap()).unwrap();

            let relative_root = Path::new(root.path().file_name().unwrap());
            let location = LocalConfigLocation::RootConfigFile;

            let result = find_local_config_candidates(relative_root, location);

            // Restore current dir
            std::env::set_current_dir(original_dir).unwrap();

            let got = result.unwrap();
            assert_eq!(got.len(), 1);
            assert!(got.first().unwrap().path.is_absolute());
            assert_eq!(
                got.first().unwrap().path,
                config_path.canonicalize().unwrap()
            );
        }
    }

    mod selection {
        use std::path::PathBuf;

        use super::*;

        #[test]
        fn returns_none_when_candidates_are_empty() {
            let got = select_config_candidate(vec![], None);
            assert!(got.is_none());
        }

        #[test]
        fn returns_candidate_when_single_candidate_exists() {
            let candidate = DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::RootConfigFile,
                ),
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };

            let got = select_config_candidate(vec![candidate], None).unwrap();

            assert_eq!(got.warning, None);
            assert_eq!(got.candidate.format, StructuredFileFormat::Toml);
        }

        #[test]
        fn returns_persisted_match_with_warning_when_multiple_candidates_exist()
        {
            let base = PathBuf::from("/vault");
            let toml_candidate = DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::RootConfigFile,
                ),
                base: base.clone(),
                path: base.join("lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            let json_candidate = DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::RootConfigFile,
                ),
                base: base.clone(),
                path: base.join("lithos.json"),
                format: StructuredFileFormat::Json,
            };

            // json is lower precedence than toml, but we pass json as persisted
            let got = select_config_candidate(
                vec![toml_candidate, json_candidate],
                Some(StructuredFileFormat::Json),
            )
            .unwrap();

            assert_eq!(got.candidate.format, StructuredFileFormat::Json);
            assert_eq!(got.candidate.path, base.join("lithos.json"));

            let expected_warning = DiscoveryWarning::FormatAmbiguity {
                base,
                candidates: vec![
                    PathBuf::from("/vault/lithos.toml"),
                    PathBuf::from("/vault/lithos.json"),
                ],
            };
            assert_eq!(got.warning, Some(expected_warning));
        }

        #[test]
        fn returns_highest_precedence_with_warning_when_no_persisted_match_exists()
         {
            let base = PathBuf::from("/vault");
            let yaml_candidate = DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::RootConfigFile,
                ),
                base: base.clone(),
                path: base.join("lithos.yaml"),
                format: StructuredFileFormat::Yaml,
            };
            let json_candidate = DiscoveredConfigFile {
                location: ConfigLocation::Local(
                    LocalConfigLocation::RootConfigFile,
                ),
                base: base.clone(),
                path: base.join("lithos.json"),
                format: StructuredFileFormat::Json,
            };

            // yaml has rank 2, json has rank 1. So json wins. No persisted
            // match (or matches neither).
            let got = select_config_candidate(
                vec![yaml_candidate, json_candidate],
                Some(StructuredFileFormat::Toml),
            )
            .unwrap();

            assert_eq!(got.candidate.format, StructuredFileFormat::Json);
            assert_eq!(got.candidate.path, base.join("lithos.json"));

            let expected_warning = DiscoveryWarning::FormatAmbiguity {
                base,
                candidates: vec![
                    PathBuf::from("/vault/lithos.yaml"),
                    PathBuf::from("/vault/lithos.json"),
                ],
            };
            assert_eq!(got.warning, Some(expected_warning));
        }
    }
}
