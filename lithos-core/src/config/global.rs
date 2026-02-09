//! Global configuration structures.
//!
//! This module contains configuration types that are specific to global-level
//! configuration, including filesystem settings, trusted vaults, and global
//! defaults.

#![expect(
    clippy::exhaustive_enums,
    reason = "rkyv generates exhaustive archived enums"
)]
#![expect(
    clippy::pattern_type_mismatch,
    reason = "Matching on &self with TrustedVaults enum - rust auto-borrows \
              fields"
)]

use std::{collections::HashMap, path::PathBuf};

use super::{
    error::ConfigError,
    frontmatter::Frontmatter,
    logging::Logging,
    paths::{Schema, Template},
    raw::{RawGlobal, RawGlobalPaths, RawTrustedVaults},
    task::TaskConfig,
};

/// Global default configuration.
///
/// # Constraints
/// - Provides system-wide defaults.
/// - Loaded from global lithos.toml or system defaults.
/// - The global configuration has the lowest precedence in the configuration
///   hierarchy.
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
    /// Filesystem configuration for global defaults.
    filesystem: Paths,
    /// Frontmatter configuration for global defaults.
    frontmatter: Frontmatter,
    /// Logging configuration for global defaults.
    logging: Logging,
    /// Trusted vaults configuration.
    trusted_vaults: Option<TrustedVaults>,
    /// Task configuration overrides.
    task: Option<TaskConfig>,
}

/// Global filesystem configuration (global template/schema library).
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    Default,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct Paths {
    /// Schema configuration for global library.
    schema: Schema,
    /// Template configuration for global library.
    template: Template,
}

/// Trusted vaults configuration supporting list or map format.
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

/// List of trusted vault paths.
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

/// Map of trusted vault aliases to paths.
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

/// Validated path to a trusted vault (absolute).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
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

impl Default for Global {
    #[inline]
    fn default() -> Self {
        Self {
            filesystem: Paths::default(),
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
        filesystem: Paths,
        frontmatter: Frontmatter,
        logging: Logging,
        trusted_vaults: Option<TrustedVaults>,
        task: Option<TaskConfig>,
    ) -> Self {
        Self {
            filesystem,
            frontmatter,
            logging,
            trusted_vaults,
            task,
        }
    }

    #[inline]
    #[must_use]
    /// Return global filesystem settings.
    pub fn filesystem(&self) -> &Paths {
        &self.filesystem
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

impl Paths {
    #[inline]
    #[must_use]
    /// Create global filesystem settings.
    pub const fn new(schema: Schema, template: Template) -> Self {
        Self {
            schema,
            template,
        }
    }

    #[inline]
    #[must_use]
    /// Return global schema settings.
    pub const fn schema(&self) -> &Schema {
        &self.schema
    }

    #[inline]
    #[must_use]
    /// Return global template settings.
    pub const fn template(&self) -> &Template {
        &self.template
    }

    #[inline]
    /// Validate global filesystem configuration.
    ///
    /// # Errors
    /// Returns [`ConfigError`] if paths are invalid.
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.schema.validate()?;
        self.template.validate()?;
        Ok(())
    }
}

impl TrustedVaults {
    #[inline]
    /// Validate trusted vaults configuration.
    ///
    /// # Errors
    /// Returns [`ConfigError`] if formatting is invalid.
    pub fn validate(&self) -> Result<(), ConfigError> {
        match self {
            Self::List(list) => list.validate(),
            Self::Map(map) => map.validate(),
        }
    }
}

impl TrustedVaultList {
    #[inline]
    /// Validate trusted vault list.
    ///
    /// # Errors
    /// Returns [`ConfigError`] if list is empty.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.0.is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "trusted_vaults".to_owned().into(),
                message: "list cannot be empty".to_owned().into(),
            });
        }
        Ok(())
    }
}

impl TrustedVaultMap {
    #[inline]
    /// Validate trusted vault map.
    ///
    /// # Errors
    /// Returns [`ConfigError`] if map is empty.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.0.is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "trusted_vaults".to_owned().into(),
                message: "map cannot be empty".to_owned().into(),
            });
        }
        Ok(())
    }
}

impl TrustedVaultPath {
    #[inline]
    /// Create a validated trusted vault path.
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

impl TryFrom<RawGlobal> for Global {
    type Error = ConfigError;

    #[inline]
    fn try_from(raw: RawGlobal) -> Result<Self, ConfigError> {
        let filesystem = raw
            .filesystem
            .map(Paths::try_from)
            .transpose()?
            .unwrap_or_default();
        let frontmatter = raw
            .frontmatter
            .map(Frontmatter::try_from)
            .transpose()?
            .unwrap_or_default();
        let logging =
            raw.logging.map(Logging::try_from).transpose()?.unwrap_or_default();
        let trusted_vaults =
            raw.trusted_vaults.map(TrustedVaults::try_from).transpose()?;
        let task = raw.task.map(TaskConfig::from_raw).transpose()?;

        Ok(Self {
            filesystem,
            frontmatter,
            logging,
            trusted_vaults,
            task,
        })
    }
}

impl TryFrom<RawGlobalPaths> for Paths {
    type Error = ConfigError;

    #[inline]
    fn try_from(raw: RawGlobalPaths) -> Result<Self, ConfigError> {
        let schema = raw.schema.map(Schema::from).unwrap_or_default();
        let template = raw.template.map(Template::from).unwrap_or_default();

        Ok(Self {
            schema,
            template,
        })
    }
}

impl TryFrom<RawTrustedVaults> for TrustedVaults {
    type Error = ConfigError;

    #[inline]
    fn try_from(raw: RawTrustedVaults) -> Result<Self, ConfigError> {
        match raw {
            RawTrustedVaults::List(list) => {
                let paths = list
                    .into_iter()
                    .map(TrustedVaultPath::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Self::List(TrustedVaultList(paths)))
            }
            RawTrustedVaults::Map(map) => {
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
mod tests {
    mod fixtures {
        use std::path::PathBuf;

        use super::super::TrustedVaultPath;

        #[expect(
            clippy::disallowed_methods,
            reason = "expect is permitted in test setup"
        )]
        pub fn trusted_vault_path(path: &str) -> TrustedVaultPath {
            TrustedVaultPath::try_new(PathBuf::from(path)).expect("valid path")
        }
    }

    mod constructor {
        use std::path::PathBuf;

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

    mod validation {
        use super::*;

        #[test]
        fn trusted_vaults_accepts_list_only() {
            // GIVEN: trusted vaults configured with list format
            let trusted = TrustedVaults::List(TrustedVaultList(vec![
                fixtures::trusted_vault_path("/vaults/alpha"),
            ]));

            // WHEN: validating the trusted vault configuration
            let result = trusted.validate();

            // THEN: validation succeeds
            assert!(
                result.is_ok(),
                "Validation should succeed, but got: {:?}",
                result.err()
            );
        }

        #[test]
        fn trusted_vaults_accepts_map_only() {
            // GIVEN: trusted vaults configured with map format
            let trusted = TrustedVaults::Map(TrustedVaultMap(
                [(
                    Box::from("alpha"),
                    fixtures::trusted_vault_path("/vaults/alpha"),
                )]
                .into_iter()
                .collect(),
            ));

            // WHEN: validating the trusted vault configuration
            let result = trusted.validate();

            // THEN: validation succeeds
            assert!(
                result.is_ok(),
                "Validation should succeed, but got: {:?}",
                result.err()
            );
        }

        #[test]
        fn trusted_vaults_rejects_empty_map() {
            // GIVEN: trusted vaults configured with empty map format
            let trusted = TrustedVaults::Map(TrustedVaultMap(HashMap::new()));

            // WHEN: validating the trusted vault configuration
            let result = trusted.validate();

            // THEN: validation fails because map is empty
            assert!(
                result.is_err(),
                "Expected validation failure when map is empty"
            );
        }

        #[test]
        fn trusted_vaults_rejects_no_entries() {
            // GIVEN: trusted vaults configured with no list or map
            let trusted = TrustedVaults::List(TrustedVaultList(Vec::new()));

            // WHEN: validating the trusted vault configuration
            let result = trusted.validate();

            // THEN: validation fails because no format is provided
            assert!(
                result.is_err(),
                "Expected validation failure when no trusted vaults are \
                 provided"
            );
        }
    }

    mod defaults {
        use super::super::*;

        #[test]
        fn global_defaults_have_no_trusted_vaults() {
            let global = Global::default();

            assert!(
                global.trusted_vaults.is_none(),
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

    use std::collections::HashMap;

    use super::{
        TrustedVaultList, TrustedVaultMap, TrustedVaultPath, TrustedVaults,
    };
}
