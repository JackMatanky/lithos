//! Global config candidate collection.

use traces_fs::{DirPath, FilePath};

use super::{probe::exact_probe, targets::GLOBAL_CONFIG_TARGETS};
use crate::candidate::CandidatePath;

pub(crate) fn global_collect(
    suppress: bool,
    flag: Option<&FilePath>,
    env: Option<&FilePath>,
    platform_dirs: &[DirPath],
) -> Vec<CandidatePath> {
    if suppress {
        return Vec::new();
    }
    if let Some(path) = flag {
        return candidate_from_file(path).into_iter().collect();
    }
    if let Some(path) = env {
        return candidate_from_file(path).into_iter().collect();
    }

    platform_dirs
        .iter()
        .flat_map(|dir| exact_probe(dir, GLOBAL_CONFIG_TARGETS))
        .collect()
}

fn candidate_from_file(path: &FilePath) -> Option<CandidatePath> {
    let base = path
        .as_path()
        .parent()
        .and_then(|parent| DirPath::try_new(parent.to_path_buf()).ok())?;
    Some(CandidatePath::new(base, path.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod precedence {
        use pretty_assertions::assert_eq;

        use super::*;

        fn file(root: &tempfile::TempDir, name: &str) -> FilePath {
            let path = root.path().join(name);
            std::fs::write(&path, "").expect("file");
            FilePath::try_new(path).expect("file path")
        }

        #[test]
        fn flag_overrides_env() {
            let root = tempfile::tempdir().expect("root");
            let flag = file(&root, "flag.toml");
            let env = file(&root, "env.toml");

            let candidates =
                global_collect(false, Some(&flag), Some(&env), &[]);

            assert_eq!(
                candidates.first().map(CandidatePath::path),
                Some(&flag)
            );
        }

        #[test]
        fn env_overrides_platform_dirs() {
            let root = tempfile::tempdir().expect("root");
            let env = file(&root, "env.toml");
            let platform = root.path().join("platform");
            std::fs::create_dir_all(&platform).expect("platform");
            std::fs::write(platform.join("traces.toml"), "")
                .expect("platform file");
            let platform = DirPath::try_new(platform).expect("platform path");

            let candidates =
                global_collect(false, None, Some(&env), &[platform]);

            assert_eq!(candidates.first().map(CandidatePath::path), Some(&env));
            assert_eq!(candidates.len(), 1);
        }

        #[test]
        fn suppressed_returns_empty() {
            let root = tempfile::tempdir().expect("root");
            let flag = file(&root, "flag.toml");

            let candidates = global_collect(true, Some(&flag), None, &[]);

            assert!(candidates.is_empty());
        }

        #[test]
        fn platform_dir_probes_traces_config_target() {
            let root = tempfile::tempdir().expect("root");
            let config = root.path().join("traces");
            std::fs::create_dir_all(&config).expect("traces dir");
            std::fs::write(config.join("config.toml"), "")
                .expect("global config");
            let platform =
                DirPath::try_new(root.path().to_path_buf()).expect("platform");

            let candidates = global_collect(false, None, None, &[platform]);

            assert_eq!(candidates.len(), 1);
            assert_eq!(
                candidates.first().map(|c| c.path().as_path()),
                Some(config.join("config.toml").as_path())
            );
        }

        #[test]
        fn platform_dir_ignores_vault_only_markers() {
            let root = tempfile::tempdir().expect("root");
            let dot = root.path().join(".traces");
            std::fs::create_dir_all(&dot).expect("dot dir");
            std::fs::write(dot.join("config.toml"), "").expect("dot marker");
            let platform =
                DirPath::try_new(root.path().to_path_buf()).expect("platform");

            let candidates = global_collect(false, None, None, &[platform]);

            assert!(
                candidates.is_empty(),
                "global collection must not match vault-only markers"
            );
        }
    }
}
