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
    vault::VaultRoot,
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
    List(Vec<VaultRoot>),
    /// Map format for trusted vault paths with aliases.
    Map(HashMap<Box<str>, VaultRoot>),
}

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
    /// Return trusted vaults configuration, if set.
    pub fn trusted_vaults(&self) -> Option<&TrustedVaults> {
        self.trusted_vaults.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return task configuration overrides, if set.
    pub fn task(&self) -> Option<&TaskConfig> {
        self.task.as_ref()
    }
}

impl Paths {
    #[inline]
    #[must_use]
    /// Create global filesystem paths.
    pub fn new(schema: Schema, template: Template) -> Self {
        Self {
            schema,
            template,
        }
    }

    #[inline]
    #[must_use]
    /// Return schema paths configuration.
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    #[inline]
    #[must_use]
    /// Return template paths configuration.
    pub fn template(&self) -> &Template {
        &self.template
    }

    /// Validate global filesystem configuration.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if schema or template validation
    /// fails.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.schema.validate()?;
        self.template.validate()?;
        Ok(())
    }
}

impl TryFrom<RawGlobalPaths> for Paths {
    type Error = ConfigError;

    #[inline]
    fn try_from(raw: RawGlobalPaths) -> Result<Self, Self::Error> {
        let schema =
            raw.schema.map(Schema::try_from).transpose()?.unwrap_or_default();
        let template = raw
            .template
            .map(Template::try_from)
            .transpose()?
            .unwrap_or_default();
        Ok(Paths::new(schema, template))
    }
}

impl TryFrom<RawTrustedVaults> for TrustedVaults {
    type Error = ConfigError;

    #[inline]
    fn try_from(raw: RawTrustedVaults) -> Result<Self, Self::Error> {
        match raw {
            RawTrustedVaults::List(values) => {
                let mut roots = Vec::with_capacity(values.len());
                for value in values {
                    roots.push(VaultRoot::try_new(PathBuf::from(value))?);
                }
                let trusted = TrustedVaults::List(roots);
                trusted.validate()?;
                Ok(trusted)
            }
            RawTrustedVaults::Map(values) => {
                let mut roots = HashMap::new();
                let mut entries: Vec<_> = values.into_iter().collect();
                entries.sort_by(|left, right| left.0.cmp(&right.0));
                for (key, value) in entries {
                    roots.insert(
                        key.into_boxed_str(),
                        VaultRoot::try_new(PathBuf::from(value))?,
                    );
                }
                let trusted = TrustedVaults::Map(roots);
                trusted.validate()?;
                Ok(trusted)
            }
        }
    }
}

impl TryFrom<RawGlobal> for Global {
    type Error = ConfigError;

    #[inline]
    fn try_from(raw: RawGlobal) -> Result<Self, Self::Error> {
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
        let task = raw.task.map(TaskConfig::try_from).transpose()?;

        Ok(Global::new(filesystem, frontmatter, logging, trusted_vaults, task))
    }
}

impl TrustedVaults {
    /// Validate trusted vaults configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigConfigError::ValidationFailed` if both list and map are
    /// specified, or if neither is specified.
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::TrustedVaultsConfig;
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let trusted = TrustedVaultsConfig {
    ///     list: Some(vec!["/vault".to_string()]),
    ///     map: None,
    /// };
    /// trusted.validate()?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        let is_empty = match self {
            TrustedVaults::List(values) => values.is_empty(),
            TrustedVaults::Map(values) => values.is_empty(),
        };

        if is_empty {
            let vault_type = match self {
                TrustedVaults::List(_) => "list",
                TrustedVaults::Map(_) => "map",
            };
            return Err(ConfigError::ValidationFailed {
                field: "trusted_vaults".to_owned().into(),
                message: format!("trusted vault {vault_type} cannot be empty")
                    .into(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use super::TrustedVaults;
    use crate::config::{paths::SchemasDir, vault::VaultRoot};

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test uses assert! which can panic."
    )]
    fn accepts_trusted_vaults_with_list_only() -> Result<(), super::ConfigError>
    {
        // GIVEN: trusted vaults configured with list format
        let trusted = TrustedVaults::List(vec![VaultRoot::try_new(
            PathBuf::from("/vaults/alpha"),
        )?]);

        // WHEN: validating the trusted vault configuration
        let result = trusted.validate();

        // THEN: validation succeeds
        assert!(
            result.is_ok(),
            "Validation should succeed, but got: {:?}",
            result.err()
        );
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test uses assert! which can panic."
    )]
    fn accepts_trusted_vaults_with_map_only() -> Result<(), super::ConfigError>
    {
        // GIVEN: trusted vaults configured with map format
        let trusted = TrustedVaults::Map(
            [(
                Box::from("alpha"),
                VaultRoot::try_new(PathBuf::from("/vaults/alpha"))?,
            )]
            .into_iter()
            .collect(),
        );

        // WHEN: validating the trusted vault configuration
        let result = trusted.validate();

        // THEN: validation succeeds
        assert!(
            result.is_ok(),
            "Validation should succeed, but got: {:?}",
            result.err()
        );
        Ok(())
    }

    #[test]
    fn filesystem_validate_passes_with_defaults() {
        // GIVEN: a default filesystem config
        let filesystem = super::Paths::default();

        // WHEN: validating
        let result = filesystem.validate();

        // THEN: it succeeds
        assert!(
            result.is_ok(),
            "Validation should succeed, but got: {:?}",
            result.err()
        );
    }

    #[test]
    fn schemas_dir_rejects_empty_path() {
        let result = SchemasDir::try_new(PathBuf::from(""));
        assert!(result.is_err(), "SchemasDir should reject empty path");
    }

    #[test]
    fn global_defaults_have_no_trusted_vaults() {
        let global = super::Global::default();

        assert!(
            global.trusted_vaults.is_none(),
            "Default trusted_vaults should be None"
        );
    }

    #[test]
    fn global_defaults_have_no_task_config() {
        let global = super::Global::default();

        assert!(global.task().is_none(), "Default task config should be None");
    }

    #[test]
    fn global_defaults_use_info_log_level() {
        let global = super::Global::default();

        assert_eq!(
            global.logging().log_level_str(),
            "info",
            "Default log level should be 'info'"
        );
    }

    #[test]
    fn rejects_trusted_vaults_with_empty_map() {
        // GIVEN: trusted vaults configured with empty map format
        let trusted = TrustedVaults::Map(HashMap::new());

        // WHEN: validating the trusted vault configuration
        let result = trusted.validate();

        // THEN: validation fails because map is empty
        assert!(
            result.is_err(),
            "Expected validation failure when map is empty"
        );
    }

    #[test]
    fn rejects_trusted_vaults_with_no_entries() {
        // GIVEN: trusted vaults configured with no list or map
        let trusted = TrustedVaults::List(Vec::new());

        // WHEN: validating the trusted vault configuration
        let result = trusted.validate();

        // THEN: validation fails because no format is provided
        assert!(
            result.is_err(),
            "Expected validation failure when no trusted vaults are provided"
        );
    }
}
