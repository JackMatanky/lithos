//! Local config candidate discovery.

use std::{io, path::Path};

use super::{
    contracts::DiscoveredConfigFile,
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
        fn returns_correct_paths_for_all_location_variants() {
            let root = tempdir().unwrap();

            // 1. RootConfigFile
            let root_path = root.path().join("lithos.toml");
            std::fs::write(&root_path, "").unwrap();
            let got_root = find_local_config_candidates(
                root.path(),
                LocalConfigLocation::RootConfigFile,
            )
            .unwrap();
            assert_eq!(
                got_root.first().unwrap().path,
                root_path.canonicalize().unwrap()
            );

            // 2. HiddenRootConfigFile
            let hidden_path = root.path().join(".lithos.toml");
            std::fs::write(&hidden_path, "").unwrap();
            let got_hidden = find_local_config_candidates(
                root.path(),
                LocalConfigLocation::HiddenRootConfigFile,
            )
            .unwrap();
            assert_eq!(
                got_hidden.first().unwrap().path,
                hidden_path.canonicalize().unwrap()
            );

            // 3. ConfigDirectoryFile
            std::fs::create_dir(root.path().join(".lithos")).unwrap();
            let dir_path = root.path().join(".lithos").join("config.toml");
            std::fs::write(&dir_path, "").unwrap();
            let got_dir = find_local_config_candidates(
                root.path(),
                LocalConfigLocation::ConfigDirectoryFile,
            )
            .unwrap();
            assert_eq!(
                got_dir.first().unwrap().path,
                dir_path.canonicalize().unwrap()
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
}
