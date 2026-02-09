//! Global-level configuration settings.
//!
//! This module defines the [`Global`] configuration, which contains settings
//! that apply across all vaults (e.g., trusted vault paths).

#![expect(
    clippy::exhaustive_enums,
    reason = "rkyv generates exhaustive archived enums"
)]

use std::{collections::HashMap, path::PathBuf};

use super::{
    error::ConfigError, frontmatter::Frontmatter, logging::Logging,
    paths::Paths, raw::RawTrustedVaults, task::TaskConfig,
};

/// System-wide configuration settings.
///
/// `Global` contains settings that are defined at the system level and
/// shared across all vaults, such as the list of trusted vault paths.
///
/// # Examples
///
/// ```rust
/// use lithos_core::config::global::Global;
///
/// let global = Global::default();
/// assert!(global.trusted_vaults().is_none());
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[non_exhaustive]
pub struct Global {
    /// Paths configuration for global defaults.
    paths: Paths,
    /// Frontmatter configuration for global defaults.
    frontmatter: Frontmatter,
    /// Logging configuration for global defaults.
    logging: Logging,
    /// Trusted vaults configuration.
    trusted_vaults: Option<TrustedVaults>,
    /// Task configuration overrides.
    task: Option<TaskConfig>,
}

impl Default for Global {
    #[inline]
    fn default() -> Self {
        Self {
            paths: Paths::default(),
            frontmatter: Frontmatter::default(),
            logging: Logging::default(),
            trusted_vaults: None,
            task: None,
        }
    }
}

impl Global {
    #[inline]
    #[must_use]
    /// Create a global configuration.
    pub fn new(
        paths: Paths,
        frontmatter: Frontmatter,
        logging: Logging,
        trusted_vaults: Option<TrustedVaults>,
        task: Option<TaskConfig>,
    ) -> Self {
        Self {
            paths,
            frontmatter,
            logging,
            trusted_vaults,
            task,
        }
    }

    #[inline]
    #[must_use]
    /// Return global paths settings.
    pub fn paths(&self) -> &Paths {
        &self.paths
    }

    #[inline]
    #[must_use]
    /// Return global frontmatter settings.
    pub fn frontmatter(&self) -> &Frontmatter {
        &self.frontmatter
    }

    #[inline]
    #[must_use]
    /// Return global logging settings.
    pub fn logging(&self) -> &Logging {
        &self.logging
    }

    #[inline]
    #[must_use]
    /// Return trusted vaults configuration.
    pub fn trusted_vaults(&self) -> Option<&TrustedVaults> {
        self.trusted_vaults.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return global task configuration defaults.
    pub fn task(&self) -> Option<&TaskConfig> {
        self.task.as_ref()
    }
}

/// Trusted vaults configuration supporting list or map format.
///
/// This enum allows defining trusted vaults either as a simple list of paths
/// or as a mapping from aliases to paths.
///
/// # Examples
///
/// ```rust
/// # use std::collections::HashMap;
/// # use lithos_core::config::global::{TrustedVaults, TrustedVaultPath, TrustedVaultList, TrustedVaultMap};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // List format
/// let list = TrustedVaults::List(TrustedVaultList::new(vec![
///     TrustedVaultPath::try_new("/vaults/alpha".into())?
/// ]));
///
/// // Map format
/// let mut map = HashMap::new();
/// map.insert("beta".into(), TrustedVaultPath::try_new("/vaults/beta".into())?);
/// let map = TrustedVaults::Map(TrustedVaultMap::new(map));
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[serde(untagged)]
#[non_exhaustive]
pub enum TrustedVaults {
    /// List format for trusted vault paths.
    List(TrustedVaultList),
    /// Map format for trusted vault paths with aliases.
    Map(TrustedVaultMap),
}

/// List of validated trusted vault paths.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct TrustedVaultList(
    /// Internal storage for vault list.
    Vec<TrustedVaultPath>,
);

impl TrustedVaultList {
    /// Create a new trusted vault list.
    #[inline]
    #[must_use]
    pub fn new(paths: Vec<TrustedVaultPath>) -> Self {
        Self(paths)
    }
}

/// Map of trusted vault aliases to validated paths.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct TrustedVaultMap(
    /// Internal storage for vault map.
    HashMap<Box<str>, TrustedVaultPath>,
);

impl TrustedVaultMap {
    /// Create a new trusted vault map.
    #[inline]
    #[must_use]
    pub fn new(map: HashMap<Box<str>, TrustedVaultPath>) -> Self {
        Self(map)
    }
}

/// A validated path to a trusted vault (absolute).
///
/// # Invariants
///
/// - Must be an absolute path on the filesystem.
/// - Must not be empty.
///
/// # Errors
///
/// Returns [`ConfigError::ValidationFailed`] if the provided path is relative.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[serde(try_from = "String", into = "String")]
#[non_exhaustive]
pub struct TrustedVaultPath(
    /// Internal path storage.
    #[rkyv(with = rkyv::with::AsString)]
    PathBuf,
);

impl TrustedVaultPath {
    #[inline]
    /// Creates a validated trusted vault path.
    ///
    /// # Errors
    /// Returns [`ConfigError`] if the path is not absolute or is empty.
    pub fn try_new(path: PathBuf) -> Result<Self, ConfigError> {
        if path.as_os_str().is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "trusted_vault_path".to_owned().into(),
                message: "path cannot be empty".to_owned().into(),
            });
        }
        if !path.is_absolute() {
            return Err(ConfigError::ValidationFailed {
                field: "trusted_vault_path".to_owned().into(),
                message: format!(
                    "path must be absolute: {}",
                    path.to_string_lossy()
                )
                .into(),
            });
        }
        Ok(Self(path))
    }

    #[inline]
    #[must_use]
    /// Return the inner path.
    pub fn as_path(&self) -> &std::path::Path {
        &self.0
    }
}

impl TryFrom<String> for TrustedVaultPath {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, ConfigError> {
        Self::try_new(PathBuf::from(value))
    }
}

impl From<TrustedVaultPath> for String {
    #[inline]
    fn from(path: TrustedVaultPath) -> Self {
        path.0.to_string_lossy().into_owned()
    }
}

impl TryFrom<RawTrustedVaults> for TrustedVaults {
    type Error = ConfigError;

    #[inline]
    fn try_from(raw: RawTrustedVaults) -> Result<Self, ConfigError> {
        match raw {
            RawTrustedVaults::List(list) => {
                if list.is_empty() {
                    return Err(ConfigError::ValidationFailed {
                        field: "trusted_vaults".to_owned().into(),
                        message: "list cannot be empty".to_owned().into(),
                    });
                }
                let paths = list
                    .into_iter()
                    .map(TrustedVaultPath::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Self::List(TrustedVaultList(paths)))
            }
            RawTrustedVaults::Map(map) => {
                if map.is_empty() {
                    return Err(ConfigError::ValidationFailed {
                        field: "trusted_vaults".to_owned().into(),
                        message: "map cannot be empty".to_owned().into(),
                    });
                }
                let paths = map
                    .into_iter()
                    .map(|(k, v)| {
                        Ok((k.into_boxed_str(), TrustedVaultPath::try_from(v)?))
                    })
                    .collect::<Result<HashMap<_, _>, _>>()?;
                Ok(Self::Map(TrustedVaultMap(paths)))
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    clippy::disallowed_methods,
    reason = "Test modules have relaxed rules"
)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    mod constructor {
        use super::*;

        #[test]
        fn trusted_vault_path_rejects_empty() {
            let result = TrustedVaultPath::try_new(PathBuf::from(""));
            assert!(
                result.is_err(),
                "TrustedVaultPath should reject empty path"
            );
        }

        #[test]
        fn trusted_vault_path_rejects_relative() {
            let result =
                TrustedVaultPath::try_new(PathBuf::from("relative/path"));
            assert!(
                result.is_err(),
                "TrustedVaultPath should reject relative path"
            );
        }

        #[test]
        #[expect(
            clippy::panic_in_result_fn,
            reason = "Test uses assert_eq! which can panic."
        )]
        fn trusted_vault_path_accepts_absolute()
        -> Result<(), Box<dyn std::error::Error>> {
            let path = if cfg!(windows) {
                "C:\\vault"
            } else {
                "/vault"
            };
            let result = TrustedVaultPath::try_new(PathBuf::from(path))?;
            assert_eq!(result.as_path().to_string_lossy(), path);
            Ok(())
        }
    }

    mod defaults {
        use super::super::*;

        #[test]
        fn global_defaults_have_no_trusted_vaults() {
            let global = Global::default();

            assert!(
                global.trusted_vaults().is_none(),
                "Default trusted_vaults should be None"
            );
        }

        #[test]
        fn global_defaults_have_no_task_config() {
            let global = Global::default();

            assert!(
                global.task().is_none(),
                "Default task config should be None"
            );
        }

        #[test]
        fn global_defaults_use_info_log_level() {
            let global = Global::default();

            assert_eq!(
                global.logging().log_level_str(),
                "info",
                "Default log level should be 'info'"
            );
        }
    }

    mod validation {
        use std::collections::HashMap;

        use super::*;
        use crate::config::raw::RawTrustedVaults;

        #[test]
        fn trusted_vaults_accepts_list_only() {
            // GIVEN: trusted vaults configured with list format
            let raw = RawTrustedVaults::List(vec!["/vaults/alpha".to_owned()]);

            // WHEN: creating trusted vaults
            let result = TrustedVaults::try_from(raw);

            // THEN: it succeeds
            result.unwrap();
        }

        #[test]
        fn trusted_vaults_accepts_map_only() {
            // GIVEN: trusted vaults configured with map format
            let mut map = HashMap::new();
            map.insert("alpha".to_owned(), "/vaults/alpha".to_owned());
            let raw = RawTrustedVaults::Map(map);

            // WHEN: creating trusted vaults
            let result = TrustedVaults::try_from(raw);

            // THEN: it succeeds
            result.unwrap();
        }

        #[test]
        fn trusted_vaults_rejects_empty_map() {
            // GIVEN: trusted vaults configured with empty map format
            let raw = RawTrustedVaults::Map(HashMap::new());

            // WHEN: creating trusted vaults
            let result = TrustedVaults::try_from(raw);

            // THEN: validation fails because map is empty
            assert!(
                result.is_err(),
                "Expected validation failure when map is empty"
            );
        }

        #[test]
        fn trusted_vaults_rejects_no_entries() {
            // GIVEN: trusted vaults configured with no list or map
            let raw = RawTrustedVaults::List(Vec::new());

            // WHEN: creating trusted vaults
            let result = TrustedVaults::try_from(raw);

            // THEN: validation fails because list is empty
            assert!(
                result.is_err(),
                "Expected validation failure when no trusted vaults are \
                 provided"
            );
        }
    }
}
