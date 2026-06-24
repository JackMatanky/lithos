//! Global-level configuration settings.
//!
//! This module defines the [`Global`] configuration, which contains settings
//! that apply across all vaults (e.g., trusted vault paths).

#![expect(
    clippy::exhaustive_enums,
    reason = "rkyv generates exhaustive archived enum types"
)]

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use rkyv::{Archive, Deserialize, Serialize};
use trace_fs::DirPath;

use super::{
    error::ConfigError,
    frontmatter::Frontmatter,
    logging::Logging,
    raw::RawTrustedVaults,
    schema::{PropertyBankFile, SchemaConfig, SchemaDir},
    task::Task,
    template::{TemplateConfig, TemplateDir},
};

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
    /// use trace_settings::config::global::GlobalVersion;
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
    /// use trace_settings::config::global::GlobalVersion;
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
    /// use trace_settings::config::global::GlobalVersion;
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
/// use trace_settings::config::global::Global;
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
    /// Template configuration override.
    template: Option<TemplateConfig>,
    /// Schema configuration override.
    schema: Option<SchemaConfig>,
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
            template: None,
            schema: None,
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
        template: Option<TemplateConfig>,
        schema: Option<SchemaConfig>,
        trusted_vaults: Option<TrustedVaults>,
        frontmatter: Frontmatter,
        task: Option<Task>,
    ) -> Self {
        Self {
            version,
            logging,
            template,
            schema,
            trusted_vaults,
            frontmatter,
            task,
        }
    }

    /// Creates a global configuration from raw path overrides.
    ///
    /// Global path overrides ignore `cache_dir`; cache is vault-scoped.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if any configured path is
    /// invalid.
    #[inline]
    pub fn try_from_components(
        template: Option<&super::raw::RawTemplateConfig>,
        schema: Option<&super::raw::RawSchemaConfig>,
    ) -> Result<Self, ConfigError> {
        Ok(Self {
            template: template.map(parse_template).transpose()?.flatten(),
            schema: schema.map(parse_schema).transpose()?.flatten(),
            ..Self::default()
        })
    }

    #[inline]
    #[must_use]
    /// Return the version of this global config.
    pub const fn version(&self) -> GlobalVersion {
        self.version
    }

    #[inline]
    #[must_use]
    /// Return the global template override, if set.
    pub fn template(&self) -> Option<&TemplateConfig> {
        self.template.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return the global schema override, if set.
    pub fn schema(&self) -> Option<&SchemaConfig> {
        self.schema.as_ref()
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

fn parse_template(
    raw: &super::raw::RawTemplateConfig,
) -> Result<Option<TemplateConfig>, ConfigError> {
    raw.directory
        .as_ref()
        .filter(|value| !value.is_empty())
        .map(|value| {
            TemplateDir::try_new(Path::new(value)).map(TemplateConfig::new)
        })
        .transpose()
}

fn parse_schema(
    raw: &super::raw::RawSchemaConfig,
) -> Result<Option<SchemaConfig>, ConfigError> {
    let schema_dir = raw
        .directory
        .as_ref()
        .filter(|value| !value.is_empty())
        .map(|value| SchemaDir::try_new(Path::new(value)))
        .transpose()?;

    let property_bank_file = raw
        .property_bank_file
        .as_ref()
        .filter(|value| !value.is_empty())
        .map(|value| PropertyBankFile::try_new(value.clone()))
        .transpose()?;

    if schema_dir.is_none() && property_bank_file.is_none() {
        return Ok(None);
    }

    Ok(Some(SchemaConfig::new(
        schema_dir.unwrap_or_default(),
        property_bank_file.unwrap_or_default(),
    )))
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
/// # use trace_settings::config::global::{
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
/// This type validates that the path is non-empty and absolute. It performs
/// only syntactic validation — no filesystem I/O. Use [`to_dir_path`] to
/// convert to a filesystem-backed [`DirPath`].
///
/// # Invariants
///
/// - Must be an absolute path string.
/// - Must not be empty.
///
/// # Errors
///
/// Returns [`ConfigError::ValidationFailed`] if the provided path is empty
/// or relative.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct TrustedVaultPath(
    /// Internal path storage.
    Box<str>,
);

impl TrustedVaultPath {
    #[inline]
    /// Creates a validated trusted vault path.
    ///
    /// # Errors
    /// Returns [`ConfigError`] if the path is not absolute or is empty.
    #[allow(
        clippy::needless_pass_by_value,
        reason = "public API takes PathBuf for ownership ergonomics"
    )]
    pub fn try_new(path: PathBuf) -> Result<Self, ConfigError> {
        if path.as_os_str().is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "trusted_vault_path".into(),
                message: "trusted vault path cannot be empty".into(),
            });
        }
        if !path.is_absolute() {
            return Err(ConfigError::ValidationFailed {
                field: "trusted_vault_path".into(),
                message: format!(
                    "trusted vault path must be absolute: {}",
                    path.display()
                )
                .into(),
            });
        }
        Ok(Self(path.to_string_lossy().into_owned().into_boxed_str()))
    }

    #[inline]
    #[must_use]
    /// Return the inner path as a [`Path`] reference.
    pub fn as_path(&self) -> &Path {
        Path::new(&*self.0)
    }

    #[inline]
    #[must_use]
    /// Return the inner path as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[inline]
    /// Convert to [`DirPath`] for filesystem operations.
    ///
    /// # Errors
    /// Returns [`ConfigError`] if the path does not refer to an existing
    /// directory.
    pub fn to_dir_path(&self) -> Result<DirPath, ConfigError> {
        DirPath::try_new(PathBuf::from(self.0.as_ref())).map_err(|e| {
            ConfigError::ValidationFailed {
                field: "trusted_vault_path".into(),
                message: format!("invalid vault directory path: {e}").into(),
            }
        })
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
        String::from(path.0)
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

        #[test]
        fn trusted_vault_path_to_dir_path_fails_for_nonexistent() {
            let path = if cfg!(windows) {
                "C:\\_traces_test_nonexistent"
            } else {
                "/_traces_test_nonexistent_dir_2026"
            };
            let tvp = TrustedVaultPath::try_new(PathBuf::from(path))
                .expect("syntactic validation should pass");
            let result = tvp.to_dir_path();
            assert!(
                result.is_err(),
                "to_dir_path should fail for nonexistent dir"
            );
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

    mod path_overrides {
        use super::*;

        #[test]
        fn returns_template_and_schema_overrides_from_raw_paths() {
            let raw_template = crate::config::raw::RawTemplateConfig {
                directory: Some("global-templates".to_owned()),
            };
            let raw_schema = crate::config::raw::RawSchemaConfig {
                directory: Some("global-schemas".to_owned()),
                property_bank_file: Some("global-bank.json".to_owned()),
            };

            let global = Global::try_from_components(
                Some(&raw_template),
                Some(&raw_schema),
            )
            .expect("global path overrides should validate");

            assert_eq!(
                global
                    .template()
                    .expect("template override should exist")
                    .template_dir()
                    .as_relative_dir()
                    .as_str(),
                "global-templates",
                "global template override should be retained"
            );
            assert_eq!(
                global
                    .schema()
                    .expect("schema override should exist")
                    .schema_dir()
                    .as_relative_dir()
                    .as_str(),
                "global-schemas",
                "global schema override should be retained"
            );
            assert_eq!(
                global
                    .schema()
                    .expect("schema override should exist")
                    .property_bank_file()
                    .as_str(),
                "global-bank.json",
                "global property bank override should be retained under \
                 schema config"
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
