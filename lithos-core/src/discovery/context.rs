//! Invocation context supplied to Discovery by the application layer.

use std::{ffi::OsStr, path::Path};

use crate::{
    discovery::error::{
        DiscoveryError, EnvironmentOverrideError, FlagOverrideError,
    },
    fs::{DirPath, FilePath},
};

/// Per-invocation context Discovery needs before it can resolve candidates.
///
/// The anchor (working directory) is the only required field; it is validated
/// at construction. CLI flags and environment variables are optional and
/// supplied via builder methods after the anchor is confirmed valid. Holding
/// this type guarantees the anchor, flag paths, and environment paths are
/// filesystem-valid.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryContext<'a> {
    /// Active path anchor supplied by the Bootstrapper.
    anchor: DirPath,
    /// CLI-derived invocation values.
    flags: DiscoveryFlags,
    /// Environment-derived invocation values.
    env: DiscoveryEnv<'a>,
}

impl<'a> DiscoveryContext<'a> {
    /// Creates a discovery invocation context from the filesystem anchor.
    ///
    /// The anchor is the only required input. CLI flags and environment
    /// variables default to empty and may be added via [`with_flags`] and
    /// [`with_env`].
    ///
    /// [`with_flags`]: Self::with_flags
    /// [`with_env`]: Self::with_env
    ///
    /// # Errors
    ///
    /// Returns [`DiscoveryError::InvalidAnchorDirectory`] if `anchor` does not
    /// resolve to an existing directory.
    #[inline]
    pub fn new(anchor: &Path) -> Result<Self, DiscoveryError> {
        let anchor_buf = anchor.to_path_buf();
        let anchor_dir =
            DirPath::try_new(anchor_buf.clone()).map_err(|source| {
                DiscoveryError::InvalidAnchorDirectory {
                    path: anchor_buf,
                    source,
                }
            })?;

        Ok(Self {
            anchor: anchor_dir,
            flags: DiscoveryFlags::default(),
            env: DiscoveryEnv::default(),
        })
    }

    /// Attaches CLI-derived invocation flags to the context.
    #[inline]
    #[must_use]
    pub fn with_flags(mut self, flags: DiscoveryFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Attaches environment-derived invocation values to the context.
    #[inline]
    #[must_use]
    pub fn with_env(mut self, env: DiscoveryEnv<'a>) -> Self {
        self.env = env;
        self
    }

    /// Returns CLI-derived invocation values.
    #[inline]
    #[must_use]
    pub fn flags(&self) -> &DiscoveryFlags {
        &self.flags
    }

    /// Returns environment-derived invocation values.
    #[inline]
    #[must_use]
    pub fn env(&self) -> &DiscoveryEnv<'a> {
        &self.env
    }

    /// Returns the active context anchor path.
    #[inline]
    #[must_use]
    pub fn anchor(&self) -> &DirPath {
        &self.anchor
    }
}

/// CLI-derived discovery invocation fields.
///
/// All path fields are validated at construction and stored as owned
/// [`FilePath`] / [`DirPath`] values — holding this type guarantees
/// filesystem validity. The default value has no overrides and does not
/// suppress global config lookup.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiscoveryFlags {
    /// Config file path supplied by CLI.
    config_file: Option<FilePath>,
    /// Vault directory path supplied by CLI.
    vault_dir: Option<DirPath>,
    /// Whether global config lookup is disabled for this invocation.
    suppress_global: bool,
}

impl DiscoveryFlags {
    /// Creates CLI-derived discovery invocation fields.
    ///
    /// # Errors
    ///
    /// Returns [`DiscoveryError::Flag`] wrapping a [`FlagOverrideError`] when:
    /// - `config_file` is `Some` but does not exist on the filesystem
    ///   ([`FlagOverrideError::GlobalConfigPathNotFound`]), or exists but is
    ///   not a file ([`FlagOverrideError::GlobalConfigPathNotFile`]).
    /// - `vault_dir` is `Some` but does not exist on the filesystem
    ///   ([`FlagOverrideError::VaultPathNotFound`]), or exists but is not a
    ///   directory ([`FlagOverrideError::VaultPathNotDirectory`]).
    #[inline]
    pub fn new(
        config_file: Option<&Path>,
        vault_dir: Option<&Path>,
        suppress_global: bool,
    ) -> Result<Self, DiscoveryError> {
        let config_file =
            config_file.map(Self::validate_config_file).transpose()?;
        let vault_dir = vault_dir.map(Self::validate_vault_dir).transpose()?;

        Ok(Self {
            config_file,
            vault_dir,
            suppress_global,
        })
    }

    /// Validates a config file path for a CLI flag override.
    ///
    /// Checks existence first so we can produce a `NotFound` error distinct
    /// from "exists but is not a file".
    fn validate_config_file(
        path: &Path,
    ) -> Result<FilePath, FlagOverrideError> {
        let buf = path.to_path_buf();
        if !path.exists() {
            return Err(FlagOverrideError::GlobalConfigPathNotFound {
                path: buf,
            });
        }
        FilePath::try_new(buf.clone()).map_err(|source| {
            FlagOverrideError::GlobalConfigPathNotFile {
                path: buf,
                source,
            }
        })
    }

    /// Validates a vault directory path for a CLI flag override.
    ///
    /// Checks existence first so we can produce a `NotFound` error distinct
    /// from "exists but is not a directory".
    fn validate_vault_dir(path: &Path) -> Result<DirPath, FlagOverrideError> {
        let buf = path.to_path_buf();
        if !path.exists() {
            return Err(FlagOverrideError::VaultPathNotFound {
                path: buf,
            });
        }
        DirPath::try_new(buf.clone()).map_err(|source| {
            FlagOverrideError::VaultPathNotDirectory {
                path: buf,
                source,
            }
        })
    }

    /// Returns the CLI config file override, if present.
    #[inline]
    #[must_use]
    pub fn config_file(&self) -> Option<&FilePath> {
        self.config_file.as_ref()
    }

    /// Returns the CLI vault directory override, if present.
    #[inline]
    #[must_use]
    pub fn vault_dir(&self) -> Option<&DirPath> {
        self.vault_dir.as_ref()
    }

    /// Returns whether global config lookup is disabled for this invocation.
    #[inline]
    #[must_use]
    pub fn suppress_global(&self) -> bool {
        self.suppress_global
    }
}

/// Environment-derived discovery invocation fields.
///
/// Validated path fields are stored as owned [`FilePath`] / [`DirPath`]
/// values. The ceiling-directory list is kept as a borrowed raw [`OsStr`]
/// because it is split and validated later during traversal setup. The
/// default value has no overrides and no ceiling data.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiscoveryEnv<'a> {
    /// Config file path supplied by environment.
    config_file: Option<FilePath>,
    /// Vault directory path supplied by environment.
    vault_dir: Option<DirPath>,
    /// Raw platform-specific ceiling directory data.
    ceiling_dirs_raw: Option<&'a OsStr>,
}

impl<'a> DiscoveryEnv<'a> {
    /// Creates environment-derived discovery invocation fields.
    ///
    /// # Errors
    ///
    /// Returns [`DiscoveryError::Env`] wrapping an [`EnvironmentOverrideError`]
    /// when:
    /// - `config_file` is `Some` but does not exist on the filesystem
    ///   ([`EnvironmentOverrideError::GlobalConfigPathNotFound`]), or exists
    ///   but is not a file
    ///   ([`EnvironmentOverrideError::GlobalConfigPathNotFile`]).
    /// - `vault_dir` is `Some` but does not exist on the filesystem
    ///   ([`EnvironmentOverrideError::VaultPathNotFound`]), or exists but is
    ///   not a directory ([`EnvironmentOverrideError::VaultPathNotDirectory`]).
    #[inline]
    pub fn new(
        config_file: Option<&Path>,
        vault_dir: Option<&Path>,
        ceiling_dirs_raw: Option<&'a OsStr>,
    ) -> Result<Self, DiscoveryError> {
        let config_file =
            config_file.map(Self::validate_config_file).transpose()?;
        let vault_dir = vault_dir.map(Self::validate_vault_dir).transpose()?;

        Ok(Self {
            config_file,
            vault_dir,
            ceiling_dirs_raw,
        })
    }

    /// Validates a config file path for an environment variable override.
    ///
    /// Checks existence first so we can produce a `NotFound` error distinct
    /// from "exists but is not a file".
    fn validate_config_file(
        path: &Path,
    ) -> Result<FilePath, EnvironmentOverrideError> {
        let buf = path.to_path_buf();
        if !path.exists() {
            return Err(EnvironmentOverrideError::GlobalConfigPathNotFound {
                path: buf,
            });
        }
        FilePath::try_new(buf.clone()).map_err(|source| {
            EnvironmentOverrideError::GlobalConfigPathNotFile {
                path: buf,
                source,
            }
        })
    }

    /// Validates a vault directory path for an environment variable override.
    ///
    /// Checks existence first so we can produce a `NotFound` error distinct
    /// from "exists but is not a directory".
    fn validate_vault_dir(
        path: &Path,
    ) -> Result<DirPath, EnvironmentOverrideError> {
        let buf = path.to_path_buf();
        if !path.exists() {
            return Err(EnvironmentOverrideError::VaultPathNotFound {
                path: buf,
            });
        }
        DirPath::try_new(buf.clone()).map_err(|source| {
            EnvironmentOverrideError::VaultPathNotDirectory {
                path: buf,
                source,
            }
        })
    }

    /// Returns the environment config file override, if present.
    #[inline]
    #[must_use]
    pub fn config_file(&self) -> Option<&FilePath> {
        self.config_file.as_ref()
    }

    /// Returns the environment vault directory override, if present.
    #[inline]
    #[must_use]
    pub fn vault_dir(&self) -> Option<&DirPath> {
        self.vault_dir.as_ref()
    }

    /// Returns raw platform-specific ceiling directory data.
    #[inline]
    #[must_use]
    pub fn ceiling_dirs_raw(&self) -> Option<&'a OsStr> {
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
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn returns_config_file_when_path_is_valid() {
                let root = tempfile::tempdir().expect("root dir");
                let config = fixtures::valid_config_file(&root);

                let flags =
                    DiscoveryFlags::new(Some(config.as_path()), None, false)
                        .expect("valid flags");

                assert_eq!(
                    flags.config_file().map(FilePath::as_path),
                    Some(config.as_path()),
                    "config_file should match the provided path"
                );
            }

            #[test]
            fn returns_vault_dir_when_path_is_valid() {
                let root = tempfile::tempdir().expect("root dir");

                let flags = DiscoveryFlags::new(None, Some(root.path()), false)
                    .expect("valid flags");

                assert_eq!(
                    flags.vault_dir().map(DirPath::as_path),
                    Some(root.path()),
                    "vault_dir should match the provided path"
                );
            }

            #[test]
            fn returns_suppress_global_when_set() {
                let flags =
                    DiscoveryFlags::new(None, None, true).expect("valid flags");

                assert!(
                    flags.suppress_global(),
                    "suppress_global should be set"
                );
            }

            #[test]
            fn returns_none_config_file_when_no_override_given() {
                let flags = DiscoveryFlags::new(None, None, false)
                    .expect("flags with no overrides");

                assert!(
                    flags.config_file().is_none(),
                    "config_file should be None"
                );
            }

            #[test]
            fn returns_none_vault_dir_when_no_override_given() {
                let flags = DiscoveryFlags::new(None, None, false)
                    .expect("flags with no overrides");

                assert!(
                    flags.vault_dir().is_none(),
                    "vault_dir should be None"
                );
            }

            #[test]
            fn returns_suppress_global_false_when_not_set() {
                let flags = DiscoveryFlags::new(None, None, false)
                    .expect("flags with no overrides");

                assert!(
                    !flags.suppress_global(),
                    "suppress_global should be false"
                );
            }
        }

        mod defaults {
            use super::*;

            #[test]
            fn returns_none_config_file() {
                let flags = DiscoveryFlags::default();

                assert!(
                    flags.config_file().is_none(),
                    "default config_file should be None"
                );
            }

            #[test]
            fn returns_none_vault_dir() {
                let flags = DiscoveryFlags::default();

                assert!(
                    flags.vault_dir().is_none(),
                    "default vault_dir should be None"
                );
            }

            #[test]
            fn returns_suppress_global_false() {
                let flags = DiscoveryFlags::default();

                assert!(
                    !flags.suppress_global(),
                    "default suppress_global should be false"
                );
            }
        }

        mod validation {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn rejects_config_file_when_path_does_not_exist() {
                let path = std::path::Path::new("/nonexistent/lithos.toml");
                let err = DiscoveryFlags::new(Some(path), None, false)
                    .expect_err("nonexistent path should be rejected");
                assert_eq!(
                    err.to_string(),
                    "Explicit config file path not found: \
                     /nonexistent/lithos.toml"
                );
            }

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
            fn rejects_vault_dir_when_path_does_not_exist() {
                let path = std::path::Path::new("/nonexistent/vault");
                let err = DiscoveryFlags::new(None, Some(path), false)
                    .expect_err("nonexistent path should be rejected");
                assert_eq!(
                    err.to_string(),
                    "Explicit vault path not found: /nonexistent/vault"
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
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn returns_config_file_when_path_is_valid() {
                let root = tempfile::tempdir().expect("root dir");
                let config = fixtures::valid_config_file(&root);

                let env = DiscoveryEnv::new(Some(config.as_path()), None, None)
                    .expect("valid env");

                assert_eq!(
                    env.config_file().map(FilePath::as_path),
                    Some(config.as_path()),
                    "config_file should match the provided path"
                );
            }

            #[test]
            fn returns_vault_dir_when_path_is_valid() {
                let root = tempfile::tempdir().expect("root dir");

                let env = DiscoveryEnv::new(None, Some(root.path()), None)
                    .expect("valid env");

                assert_eq!(
                    env.vault_dir().map(DirPath::as_path),
                    Some(root.path()),
                    "vault_dir should match the provided path"
                );
            }

            #[test]
            fn returns_ceiling_dirs_raw_when_provided() {
                let ceiling_dirs = OsStr::new("/repo:/workspace");

                let env = DiscoveryEnv::new(None, None, Some(ceiling_dirs))
                    .expect("valid env");

                assert_eq!(
                    env.ceiling_dirs_raw(),
                    Some(ceiling_dirs),
                    "ceiling_dirs_raw should match the provided value"
                );
            }

            #[test]
            fn returns_none_config_file_when_no_override_given() {
                let env = DiscoveryEnv::new(None, None, None)
                    .expect("env with no overrides");

                assert!(
                    env.config_file().is_none(),
                    "config_file should be None"
                );
            }

            #[test]
            fn returns_none_vault_dir_when_no_override_given() {
                let env = DiscoveryEnv::new(None, None, None)
                    .expect("env with no overrides");

                assert!(env.vault_dir().is_none(), "vault_dir should be None");
            }

            #[test]
            fn returns_none_ceiling_dirs_raw_when_not_provided() {
                let env = DiscoveryEnv::new(None, None, None)
                    .expect("env with no overrides");

                assert!(
                    env.ceiling_dirs_raw().is_none(),
                    "ceiling_dirs_raw should be None"
                );
            }
        }

        mod defaults {
            use super::*;

            #[test]
            fn returns_none_config_file() {
                let env = DiscoveryEnv::default();

                assert!(
                    env.config_file().is_none(),
                    "default config_file should be None"
                );
            }

            #[test]
            fn returns_none_vault_dir() {
                let env = DiscoveryEnv::default();

                assert!(
                    env.vault_dir().is_none(),
                    "default vault_dir should be None"
                );
            }

            #[test]
            fn returns_none_ceiling_dirs_raw() {
                let env = DiscoveryEnv::default();

                assert!(
                    env.ceiling_dirs_raw().is_none(),
                    "default ceiling_dirs_raw should be None"
                );
            }
        }

        mod validation {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn rejects_config_file_when_path_does_not_exist() {
                let path = std::path::Path::new("/nonexistent/lithos.toml");
                let err = DiscoveryEnv::new(Some(path), None, None)
                    .expect_err("nonexistent path should be rejected");
                assert_eq!(
                    err.to_string(),
                    "Environment config file path not found: \
                     /nonexistent/lithos.toml"
                );
            }

            #[test]
            fn rejects_config_file_when_path_is_a_directory() {
                let dir = tempfile::tempdir().expect("dir");
                let err = DiscoveryEnv::new(Some(dir.path()), None, None)
                    .expect_err("directory is not a valid config file");
                assert_eq!(
                    err.to_string(),
                    format!(
                        "Environment config file is not a file: {}",
                        dir.path().display()
                    )
                );
            }

            #[test]
            fn rejects_vault_dir_when_path_does_not_exist() {
                let path = std::path::Path::new("/nonexistent/vault");
                let err = DiscoveryEnv::new(None, Some(path), None)
                    .expect_err("nonexistent path should be rejected");
                assert_eq!(
                    err.to_string(),
                    "Environment vault path not found: /nonexistent/vault"
                );
            }

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
            use pretty_assertions::assert_eq;

            use super::*;

            // --- anchor ---

            #[test]
            fn returns_anchor_matching_provided_path() {
                let anchor_root = tempfile::tempdir().expect("anchor root");

                let context = DiscoveryContext::new(anchor_root.path())
                    .expect("valid context");

                assert_eq!(
                    context.anchor().as_path(),
                    anchor_root.path(),
                    "anchor should match the provided cwd"
                );
            }

            #[test]
            fn rejects_anchor_when_path_does_not_exist() {
                let err = DiscoveryContext::new(std::path::Path::new(
                    "/nonexistent/anchor",
                ))
                .expect_err("nonexistent anchor should be rejected");

                assert!(
                    matches!(
                        err,
                        DiscoveryError::InvalidAnchorDirectory { .. }
                    ),
                    "expected InvalidAnchorDirectory, got: {err:?}"
                );
            }

            // --- default flags when no with_flags called ---

            #[test]
            fn returns_default_flags_when_with_flags_not_called() {
                let anchor_root = tempfile::tempdir().expect("anchor root");

                let context = DiscoveryContext::new(anchor_root.path())
                    .expect("valid context");

                assert_eq!(
                    context.flags(),
                    &DiscoveryFlags::default(),
                    "flags should be default when with_flags not called"
                );
            }

            // --- default env when no with_env called ---

            #[test]
            fn returns_default_env_when_with_env_not_called() {
                let anchor_root = tempfile::tempdir().expect("anchor root");

                let context = DiscoveryContext::new(anchor_root.path())
                    .expect("valid context");

                assert_eq!(
                    context.env(),
                    &DiscoveryEnv::default(),
                    "env should be default when with_env not called"
                );
            }
        }

        mod with_flags {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn returns_flags_config_file_when_set() {
                let flag_root = tempfile::tempdir().expect("flag root");
                let flag_config = fixtures::valid_config_file(&flag_root);
                let anchor_root = tempfile::tempdir().expect("anchor root");
                let flags = DiscoveryFlags::new(
                    Some(flag_config.as_path()),
                    None,
                    false,
                )
                .expect("valid flags");

                let context = DiscoveryContext::new(anchor_root.path())
                    .expect("valid context")
                    .with_flags(flags);

                assert_eq!(
                    context.flags().config_file().map(FilePath::as_path),
                    Some(flag_config.as_path()),
                    "flags config_file should be preserved"
                );
            }

            #[test]
            fn returns_flags_vault_dir_when_set() {
                let flag_root = tempfile::tempdir().expect("flag root");
                let anchor_root = tempfile::tempdir().expect("anchor root");
                let flags =
                    DiscoveryFlags::new(None, Some(flag_root.path()), false)
                        .expect("valid flags");

                let context = DiscoveryContext::new(anchor_root.path())
                    .expect("valid context")
                    .with_flags(flags);

                assert_eq!(
                    context.flags().vault_dir().map(DirPath::as_path),
                    Some(flag_root.path()),
                    "flags vault_dir should be preserved"
                );
            }

            #[test]
            fn returns_suppress_global_when_set_via_flags() {
                let anchor_root = tempfile::tempdir().expect("anchor root");
                let flags =
                    DiscoveryFlags::new(None, None, true).expect("valid flags");

                let context = DiscoveryContext::new(anchor_root.path())
                    .expect("valid context")
                    .with_flags(flags);

                assert!(
                    context.flags().suppress_global(),
                    "suppress_global should be set"
                );
            }
        }

        mod with_env {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn returns_env_config_file_when_set() {
                let env_root = tempfile::tempdir().expect("env root");
                let env_config = fixtures::valid_config_file(&env_root);
                let anchor_root = tempfile::tempdir().expect("anchor root");
                let env =
                    DiscoveryEnv::new(Some(env_config.as_path()), None, None)
                        .expect("valid env");

                let context = DiscoveryContext::new(anchor_root.path())
                    .expect("valid context")
                    .with_env(env);

                assert_eq!(
                    context.env().config_file().map(FilePath::as_path),
                    Some(env_config.as_path()),
                    "env config_file should be preserved"
                );
            }

            #[test]
            fn returns_env_vault_dir_when_set() {
                let env_root = tempfile::tempdir().expect("env root");
                let anchor_root = tempfile::tempdir().expect("anchor root");
                let env = DiscoveryEnv::new(None, Some(env_root.path()), None)
                    .expect("valid env");

                let context = DiscoveryContext::new(anchor_root.path())
                    .expect("valid context")
                    .with_env(env);

                assert_eq!(
                    context.env().vault_dir().map(DirPath::as_path),
                    Some(env_root.path()),
                    "env vault_dir should be preserved"
                );
            }

            #[test]
            fn returns_ceiling_dirs_raw_when_set() {
                let anchor_root = tempfile::tempdir().expect("anchor root");
                let ceiling_dirs = OsStr::new("/repo:/workspace");
                let env = DiscoveryEnv::new(None, None, Some(ceiling_dirs))
                    .expect("valid env");

                let context = DiscoveryContext::new(anchor_root.path())
                    .expect("valid context")
                    .with_env(env);

                assert_eq!(
                    context.env().ceiling_dirs_raw(),
                    Some(ceiling_dirs),
                    "ceiling_dirs_raw should be preserved"
                );
            }
        }
    }
}
