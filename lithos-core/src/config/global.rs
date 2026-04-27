//! Global-level configuration settings.
//!
//! This module defines the [`Global`] configuration, which contains settings
//! that apply across all vaults (e.g., trusted vault paths).

#![expect(
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    reason = "rkyv generates exhaustive archived types"
)]

use std::{collections::HashMap, path::PathBuf};

use rkyv::{Archive, Deserialize, Serialize};

use super::{
    error::ConfigError,
    frontmatter::Frontmatter,
    logging::Logging,
    paths::{PropertyBank, Schema, Template},
    raw::RawTrustedVaults,
    task::Task,
};
use crate::fs::AbsolutePath;

/// Global-level paths configuration (without cache).
///
/// Unlike the resolved [`crate::config::paths::Paths`], this struct uses
/// [`Option`] for all fields to represent partial overrides of vault-level
/// defaults.
#[derive(Debug, Clone, PartialEq, Default, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct Paths {
    /// Overridden template settings.
    pub template: Option<Template>,
    /// Overridden schema settings.
    pub schema: Option<Schema>,
    /// Overridden property bank filename.
    pub property_bank: Option<PropertyBank>,
}

impl Paths {
    /// Create global paths settings.
    #[inline]
    #[must_use]
    pub const fn new(
        template: Option<Template>,
        schema: Option<Schema>,
        property_bank: Option<PropertyBank>,
    ) -> Self {
        Self {
            template,
            schema,
            property_bank,
        }
    }
}

impl TryFrom<&super::raw::RawPathsConfig> for Paths {
    type Error = ConfigError;

    /// Convert raw paths configuration into global Paths.
    ///
    /// Global paths do not include cache (cache is vault-specific).
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::ValidationFailed`] if any path is invalid.
    #[inline]
    fn try_from(raw: &super::raw::RawPathsConfig) -> Result<Self, Self::Error> {
        // Parse template directory (if present)
        let template = raw
            .templates_dir
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| {
                Template::try_new(PathBuf::from(s)).map_err(|e| {
                    ConfigError::ValidationFailed {
                        field: "templates_dir".into(),
                        message: format!("invalid templates_dir: {e}").into(),
                    }
                })
            })
            .transpose()?;

        // Parse schema directory (if present)
        let schema = raw
            .schemas_dir
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| {
                Schema::try_new(PathBuf::from(s)).map_err(|e| {
                    ConfigError::ValidationFailed {
                        field: "schemas_dir".into(),
                        message: format!("invalid schemas_dir: {e}").into(),
                    }
                })
            })
            .transpose()?;

        // Parse property bank filename (if present)
        let property_bank = raw
            .property_bank_file
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| {
                PropertyBank::try_new(s.clone()).map_err(|e| {
                    ConfigError::ValidationFailed {
                        field: "property_bank_file".into(),
                        message: format!("invalid property_bank_file: {e}")
                            .into(),
                    }
                })
            })
            .transpose()?;

        Ok(Self::new(template, schema, property_bank))
    }
}

/// Version number for global configuration staleness tracking.
///
/// Incremented each time the global config file changes. Used to determine
/// whether the cached merged config needs rebuilding.
///
/// # Version Sequence
///
/// - Starts at 1 (not 0)
/// - Increments on each global config file change
/// - Independent of `VaultVersion` and `Config::Version`
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct GlobalVersion(u64);

impl GlobalVersion {
    /// Returns the initial version.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::config::global::GlobalVersion;
    ///
    /// let version = GlobalVersion::initial();
    /// assert_eq!(version.value(), 1);
    /// ```
    #[inline]
    #[must_use]
    pub const fn initial() -> Self {
        Self(1)
    }

    /// Returns the numeric version value.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::config::global::GlobalVersion;
    ///
    /// let version = GlobalVersion::initial();
    /// assert_eq!(version.value(), 1);
    /// ```
    #[inline]
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Returns the next version, or an overflow error.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the version number
    /// overflows.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::config::global::GlobalVersion;
    ///
    /// let version = GlobalVersion::initial();
    /// let next = version.next().expect("version increment succeeded");
    /// assert_eq!(next.value(), 2);
    /// ```
    #[inline]
    pub fn next(self) -> Result<Self, ConfigError> {
        self.0.checked_add(1).map(Self).ok_or_else(|| {
            ConfigError::ValidationFailed {
                field: "global_version".into(),
                message: "global version overflow".into(),
            }
        })
    }
}

impl std::fmt::Display for GlobalVersion {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u64> for GlobalVersion {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err(ConfigError::ValidationFailed {
                field: "global_version".into(),
                message: "global version cannot be zero".into(),
            });
        }
        Ok(Self(value))
    }
}

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
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Global {
    /// Version number for this global config.
    version: GlobalVersion,
    /// Logging configuration for global defaults.
    logging: Logging,
    /// Paths configuration for global defaults (without cache).
    paths: Paths,
    /// Trusted vaults configuration.
    trusted_vaults: Option<TrustedVaults>,
    /// Frontmatter configuration for global defaults.
    frontmatter: Frontmatter,
    /// Task configuration overrides.
    task: Option<Task>,
}

impl Default for Global {
    #[inline]
    fn default() -> Self {
        Self {
            version: GlobalVersion::initial(),
            logging: Logging::default(),
            paths: Paths::default(),
            trusted_vaults: None,
            frontmatter: Frontmatter::default(),
            task: None,
        }
    }
}

impl Global {
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Domain constructor with all required fields; builder \
                  pattern adds complexity without benefit"
    )]
    /// Create a global configuration.
    pub fn new(
        version: GlobalVersion,
        logging: Logging,
        paths: Paths,
        trusted_vaults: Option<TrustedVaults>,
        frontmatter: Frontmatter,
        task: Option<Task>,
    ) -> Self {
        Self {
            version,
            logging,
            paths,
            trusted_vaults,
            frontmatter,
            task,
        }
    }

    #[inline]
    #[must_use]
    /// Return the version of this global config.
    pub const fn version(&self) -> GlobalVersion {
        self.version
    }

    #[inline]
    #[must_use]
    /// Return global paths settings.
    pub fn paths(&self) -> &Paths {
        &self.paths
    }

    #[inline]
    #[must_use]
    /// Return trusted vaults configuration.
    pub fn trusted_vaults(&self) -> Option<&TrustedVaults> {
        self.trusted_vaults.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return global frontmatter settings.
    pub fn frontmatter(&self) -> &Frontmatter {
        &self.frontmatter
    }

    #[inline]
    #[must_use]
    /// Return global task configuration defaults.
    pub fn task(&self) -> Option<&Task> {
        self.task.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return global logging settings.
    pub fn logging(&self) -> &Logging {
        &self.logging
    }
}

impl TryFrom<&super::raw::RawConfig> for Global {
    type Error = ConfigError;

    /// Convert raw configuration into validated Global config.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] if any field fails validation.
    #[inline]
    fn try_from(raw: &super::raw::RawConfig) -> Result<Self, Self::Error> {
        // Convert logging (use default if not present)
        let logging = raw
            .logging
            .as_ref()
            .map(|l| Logging::try_from(l.clone()))
            .transpose()?
            .unwrap_or_default();

        // Convert paths to global::Paths (without cache)
        let paths = Paths::try_from(&raw.paths)?;

        // Convert trusted vaults (None if not present)
        let trusted_vaults = raw
            .trusted_vaults
            .as_ref()
            .map(|tv| TrustedVaults::try_from(tv.clone()))
            .transpose()?;

        // Convert frontmatter (use default if not present)
        let frontmatter = raw
            .frontmatter
            .as_ref()
            .map(|f| Frontmatter::try_from(f.clone()))
            .transpose()?
            .unwrap_or_default();

        // Convert task (None if not present)
        let task =
            raw.task.as_ref().map(|t| Task::try_from(t.clone())).transpose()?;

        // Version will be set by Command layer when recording
        let version = GlobalVersion::initial();

        Ok(Self::new(
            version,
            logging,
            paths,
            trusted_vaults,
            frontmatter,
            task,
        ))
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
/// # use lithos_core::config::global::{
/// #     TrustedVaults,
/// #     TrustedVaultPath,
/// #     TrustedVaultList,
/// #     TrustedVaultMap
/// # };
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // List format
/// let list = TrustedVaults::List(TrustedVaultList::new(vec![
///     TrustedVaultPath::try_new("/vaults/alpha".into())?,
/// ]));
///
/// // Map format
/// let mut map = HashMap::new();
/// map.insert(
///     "beta".into(),
///     TrustedVaultPath::try_new("/vaults/beta".into())?,
/// );
/// let map = TrustedVaults::Map(TrustedVaultMap::new(map));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum TrustedVaults {
    /// List format for trusted vault paths.
    List(TrustedVaultList),
    /// Map format for trusted vault paths with aliases.
    Map(TrustedVaultMap),
}

/// List of validated trusted vault paths.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
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
/// This is a wrapper around [`AbsolutePath`] that provides a domain-specific
/// name for trusted vault paths.
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
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct TrustedVaultPath(
    /// Internal path storage.
    AbsolutePath,
);

impl TrustedVaultPath {
    #[inline]
    /// Creates a validated trusted vault path.
    ///
    /// # Errors
    /// Returns [`ConfigError`] if the path is not absolute or is empty.
    pub fn try_new(path: PathBuf) -> Result<Self, ConfigError> {
        let path = AbsolutePath::try_from(path).map_err(|e| {
            ConfigError::ValidationFailed {
                field: "trusted_vault_path".into(),
                message: e.to_string().into(),
            }
        })?;
        Ok(Self(path))
    }

    #[inline]
    #[must_use]
    /// Return the inner path.
    pub fn as_path(&self) -> &std::path::Path {
        self.0.as_path()
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
        path.0.as_path().to_string_lossy().into_owned()
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
                        field: "trusted_vaults".into(),
                        message: "list cannot be empty".into(),
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
                        field: "trusted_vaults".into(),
                        message: "map cannot be empty".into(),
                    });
                }
                let paths = map
                    .into_iter()
                    .map(
                        |(k, v)| -> Result<
                            (Box<str>, TrustedVaultPath),
                            ConfigError,
                        > {
                            Ok((
                                k.into_boxed_str(),
                                TrustedVaultPath::try_from(v)?,
                            ))
                        },
                    )
                    .collect::<Result<HashMap<_, _>, ConfigError>>()?;
                Ok(Self::Map(TrustedVaultMap(paths)))
            }
        }
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test modules have relaxed rules"
)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    mod version {
        use super::*;

        #[test]
        fn initial_is_one() {
            assert_eq!(GlobalVersion::initial().value(), 1);
        }

        #[test]
        fn next_increments_value() {
            let v = GlobalVersion::initial();
            let next = v.next().expect("increment should succeed");
            assert_eq!(next.value(), 2);
        }

        #[test]
        fn try_from_accepts_positive() {
            let v = GlobalVersion::try_from(42).expect("valid version");
            assert_eq!(v.value(), 42);
        }

        #[test]
        fn try_from_rejects_zero() {
            let result = GlobalVersion::try_from(0);
            assert!(result.is_err(), "GlobalVersion should reject zero");
        }

        #[test]
        fn display_shows_value() {
            let v = GlobalVersion::initial();
            assert_eq!(v.to_string(), "1");
        }
    }

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
        fn trusted_vault_path_accepts_absolute() {
            let path = if cfg!(windows) {
                "C:\\vault"
            } else {
                "/vault"
            };
            let result =
                TrustedVaultPath::try_new(PathBuf::from(path)).unwrap();
            assert_eq!(result.as_path().to_string_lossy(), path);
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
                global.logging().level_str(),
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
