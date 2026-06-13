//! Invocation context supplied to Discovery by the application layer.

use std::{ffi::OsStr, path::Path};

use crate::{
    discovery::error::{
        DiscoveryError, EnvironmentOverrideError, FlagOverrideError,
    },
    fs::{DirPath, FilePath},
};

/// Per-invocation context Discovery needs before it can resolve candidates.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(dead_code, reason = "Contract slice; wired into discovery later")]
pub(crate) struct DiscoveryContext<'a> {
    /// CLI-derived invocation values.
    flags: DiscoveryFlags,
    /// Environment-derived invocation values.
    env: DiscoveryEnv<'a>,
    /// Active path anchor supplied by the Bootstrapper.
    anchor: DirPath,
}

#[allow(dead_code, reason = "Contract slice; wired into discovery later")]
impl<'a> DiscoveryContext<'a> {
    /// Creates a discovery invocation context from app-owned runtime inputs.
    #[inline]
    pub(crate) fn new(
        flags: DiscoveryFlags,
        env: DiscoveryEnv<'a>,
        anchor: &Path,
    ) -> Result<Self, DiscoveryError> {
        let anchor =
            DirPath::try_new(anchor.to_path_buf()).map_err(|source| {
                DiscoveryError::InvalidAnchorDirectory {
                    path: anchor.to_path_buf(),
                    source,
                }
            })?;

        Ok(Self {
            flags,
            env,
            anchor,
        })
    }

    /// Returns CLI-derived invocation values.
    #[inline]
    #[must_use]
    pub(crate) fn flags(&self) -> &DiscoveryFlags {
        &self.flags
    }

    /// Returns environment-derived invocation values.
    #[inline]
    #[must_use]
    pub(crate) fn env(&self) -> &DiscoveryEnv<'a> {
        &self.env
    }

    /// Returns the active context anchor path.
    #[inline]
    #[must_use]
    pub(crate) fn anchor(&self) -> &DirPath {
        &self.anchor
    }
}

/// CLI-derived discovery invocation fields.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(dead_code, reason = "Contract slice; wired into discovery later")]
pub(crate) struct DiscoveryFlags {
    /// Config file path supplied by CLI.
    config_file: Option<FilePath>,
    /// Vault directory path supplied by CLI.
    vault_dir: Option<DirPath>,
    /// Whether global config lookup is disabled for this invocation.
    suppress_global: bool,
}

#[allow(dead_code, reason = "Contract slice; wired into discovery later")]
impl DiscoveryFlags {
    /// Creates CLI-derived discovery invocation fields.
    #[inline]
    pub(crate) fn new(
        config_file: Option<&Path>,
        vault_dir: Option<&Path>,
        suppress_global: bool,
    ) -> Result<Self, DiscoveryError> {
        let config_file = config_file
            .map(|path| {
                FilePath::try_new(path.to_path_buf()).map_err(|source| {
                    FlagOverrideError::GlobalConfigPathNotFile {
                        path: path.to_path_buf(),
                        source,
                    }
                })
            })
            .transpose()?;
        let vault_dir = vault_dir
            .map(|path| {
                DirPath::try_new(path.to_path_buf()).map_err(|source| {
                    FlagOverrideError::VaultPathNotDirectory {
                        path: path.to_path_buf(),
                        source,
                    }
                })
            })
            .transpose()?;

        Ok(Self {
            config_file,
            vault_dir,
            suppress_global,
        })
    }

    /// Returns the CLI config file override, if present.
    #[inline]
    #[must_use]
    pub(crate) fn config_file(&self) -> Option<&FilePath> {
        self.config_file.as_ref()
    }

    /// Returns the CLI vault directory override, if present.
    #[inline]
    #[must_use]
    pub(crate) fn vault_dir(&self) -> Option<&DirPath> {
        self.vault_dir.as_ref()
    }

    /// Returns whether global config lookup is disabled for this invocation.
    #[inline]
    #[must_use]
    pub(crate) fn suppress_global(&self) -> bool {
        self.suppress_global
    }
}

/// Environment-derived discovery invocation fields.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(dead_code, reason = "Contract slice; wired into discovery later")]
pub(crate) struct DiscoveryEnv<'a> {
    /// Config file path supplied by environment.
    config_file: Option<FilePath>,
    /// Vault directory path supplied by environment.
    vault_dir: Option<DirPath>,
    /// Raw platform-specific ceiling directory data.
    ceiling_dirs_raw: Option<&'a OsStr>,
}

#[allow(dead_code, reason = "Contract slice; wired into discovery later")]
impl<'a> DiscoveryEnv<'a> {
    /// Creates environment-derived discovery invocation fields.
    #[inline]
    pub(crate) fn new(
        config_file: Option<&Path>,
        vault_dir: Option<&Path>,
        ceiling_dirs_raw: Option<&'a OsStr>,
    ) -> Result<Self, DiscoveryError> {
        let config_file = config_file
            .map(|path| {
                FilePath::try_new(path.to_path_buf()).map_err(|source| {
                    EnvironmentOverrideError::GlobalConfigPathNotFile {
                        path: path.to_path_buf(),
                        source,
                    }
                })
            })
            .transpose()?;
        let vault_dir = vault_dir
            .map(|path| {
                DirPath::try_new(path.to_path_buf()).map_err(|source| {
                    EnvironmentOverrideError::from_vault_path_error(
                        path.to_path_buf(),
                        source,
                    )
                })
            })
            .transpose()?;

        Ok(Self {
            config_file,
            vault_dir,
            ceiling_dirs_raw,
        })
    }

    /// Returns the environment config file override, if present.
    #[inline]
    #[must_use]
    pub(crate) fn config_file(&self) -> Option<&FilePath> {
        self.config_file.as_ref()
    }

    /// Returns the environment vault directory override, if present.
    #[inline]
    #[must_use]
    pub(crate) fn vault_dir(&self) -> Option<&DirPath> {
        self.vault_dir.as_ref()
    }

    /// Returns raw platform-specific ceiling directory data.
    #[inline]
    #[must_use]
    pub(crate) fn ceiling_dirs_raw(&self) -> Option<&'a OsStr> {
        self.ceiling_dirs_raw
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;
    use crate::fs::path::{DirPath, FilePath};

    mod fixtures {
        pub(super) fn valid_config_file(
            dir: &tempfile::TempDir,
        ) -> std::path::PathBuf {
            let path = dir.path().join("lithos.toml");
            std::fs::write(&path, "").expect("write config file");
            path
        }
    }

    mod discovery_flags {
        use super::*;

        mod constructor {
            use super::*;

            #[test]
            fn returns_flags_with_all_fields_when_paths_are_valid() {
                let root = tempfile::tempdir().expect("root dir");
                let config = fixtures::valid_config_file(&root);

                let flags = DiscoveryFlags::new(
                    Some(config.as_path()),
                    Some(root.path()),
                    true,
                )
                .expect("valid flags");

                assert_eq!(
                    flags.config_file().map(FilePath::as_path),
                    Some(config.as_path()),
                    "config_file should match the provided path"
                );
                assert_eq!(
                    flags.vault_dir().map(DirPath::as_path),
                    Some(root.path()),
                    "vault_dir should match the provided path"
                );
                assert!(
                    flags.suppress_global(),
                    "suppress_global should be set"
                );
            }

            #[test]
            fn returns_flags_with_none_fields_when_no_overrides_given() {
                let flags = DiscoveryFlags::new(None, None, false)
                    .expect("flags with no overrides");
                assert!(
                    flags.config_file().is_none(),
                    "config_file should be None"
                );
                assert!(
                    flags.vault_dir().is_none(),
                    "vault_dir should be None"
                );
                assert!(
                    !flags.suppress_global(),
                    "suppress_global should be false"
                );
            }
        }

        mod validation {
            use super::*;

            #[test]
            fn rejects_config_file_when_path_is_a_directory() {
                let dir = tempfile::tempdir().expect("dir");
                let err = DiscoveryFlags::new(Some(dir.path()), None, false)
                    .expect_err("directory is not a valid config file");
                assert_eq!(
                    err.to_string(),
                    format!(
                        "Explicit config file is not a file: {}",
                        dir.path().display()
                    )
                );
            }

            #[test]
            fn rejects_vault_dir_when_path_is_a_file() {
                let file = tempfile::NamedTempFile::new().expect("file");
                let err = DiscoveryFlags::new(None, Some(file.path()), false)
                    .expect_err("file is not a valid vault directory");
                assert_eq!(
                    err.to_string(),
                    format!(
                        "Explicit vault path is not a directory: {}",
                        file.path().display()
                    )
                );
            }
        }
    }

    mod discovery_env {
        use super::*;

        mod constructor {
            use super::*;

            #[test]
            fn returns_env_with_all_fields_when_paths_are_valid() {
                let root = tempfile::tempdir().expect("root dir");
                let config = fixtures::valid_config_file(&root);
                let ceiling_dirs = OsStr::new("/repo:/workspace");

                let env = DiscoveryEnv::new(
                    Some(config.as_path()),
                    Some(root.path()),
                    Some(ceiling_dirs),
                )
                .expect("valid env");

                assert_eq!(
                    env.config_file().map(FilePath::as_path),
                    Some(config.as_path()),
                    "config_file should match the provided path"
                );
                assert_eq!(
                    env.vault_dir().map(DirPath::as_path),
                    Some(root.path()),
                    "vault_dir should match the provided path"
                );
                assert_eq!(
                    env.ceiling_dirs_raw(),
                    Some(ceiling_dirs),
                    "ceiling_dirs_raw should match the provided value"
                );
            }

            #[test]
            fn returns_env_with_none_fields_when_no_overrides_given() {
                let env = DiscoveryEnv::new(None, None, None)
                    .expect("env with no overrides");
                assert!(
                    env.config_file().is_none(),
                    "config_file should be None"
                );
                assert!(env.vault_dir().is_none(), "vault_dir should be None");
                assert!(
                    env.ceiling_dirs_raw().is_none(),
                    "ceiling_dirs_raw should be None"
                );
            }
        }

        mod validation {
            use super::*;

            #[test]
            fn rejects_vault_dir_when_path_is_a_file() {
                let file = tempfile::NamedTempFile::new().expect("file");
                let err = DiscoveryEnv::new(None, Some(file.path()), None)
                    .expect_err("file is not a valid vault directory");
                assert_eq!(
                    err.to_string(),
                    format!(
                        "Environment vault path is not a directory: {}",
                        file.path().display()
                    )
                );
            }
        }
    }

    mod discovery_context {
        use super::*;

        mod constructor {
            use super::*;

            #[test]
            fn returns_context_preserving_flags_env_and_anchor() {
                let flag_root = tempfile::tempdir().expect("flag root");
                let env_root = tempfile::tempdir().expect("env root");
                let anchor_root = tempfile::tempdir().expect("anchor root");
                let flag_config = fixtures::valid_config_file(&flag_root);
                let env_config = fixtures::valid_config_file(&env_root);
                let ceiling_dirs = OsStr::new("/repo:/workspace");

                let flags = DiscoveryFlags::new(
                    Some(flag_config.as_path()),
                    Some(flag_root.path()),
                    true,
                )
                .expect("valid flags");
                let env = DiscoveryEnv::new(
                    Some(env_config.as_path()),
                    Some(env_root.path()),
                    Some(ceiling_dirs),
                )
                .expect("valid env");
                let context =
                    DiscoveryContext::new(flags, env, anchor_root.path())
                        .expect("valid context");

                assert_eq!(
                    context.anchor().as_path(),
                    anchor_root.path(),
                    "anchor should match the provided cwd"
                );
                assert_eq!(
                    context.flags().config_file().map(FilePath::as_path),
                    Some(flag_config.as_path()),
                    "flags config_file should be preserved"
                );
                assert_eq!(
                    context.flags().vault_dir().map(DirPath::as_path),
                    Some(flag_root.path()),
                    "flags vault_dir should be preserved"
                );
                assert!(
                    context.flags().suppress_global(),
                    "suppress_global should be set"
                );
                assert_eq!(
                    context.env().config_file().map(FilePath::as_path),
                    Some(env_config.as_path()),
                    "env config_file should be preserved"
                );
                assert_eq!(
                    context.env().vault_dir().map(DirPath::as_path),
                    Some(env_root.path()),
                    "env vault_dir should be preserved"
                );
                assert_eq!(
                    context.env().ceiling_dirs_raw(),
                    Some(ceiling_dirs),
                    "ceiling_dirs_raw should be preserved"
                );
            }

            #[test]
            fn rejects_anchor_when_path_does_not_exist() {
                let flags = DiscoveryFlags::new(None, None, false)
                    .expect("empty flags");
                let env =
                    DiscoveryEnv::new(None, None, None).expect("empty env");

                let err = DiscoveryContext::new(
                    flags,
                    env,
                    std::path::Path::new("/nonexistent/anchor"),
                )
                .expect_err("nonexistent anchor should be rejected");

                assert!(
                    matches!(
                        err,
                        DiscoveryError::InvalidAnchorDirectory { .. }
                    ),
                    "expected InvalidAnchorDirectory, got: {err:?}"
                );
            }
        }
    }
}
