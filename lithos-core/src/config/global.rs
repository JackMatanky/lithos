//! Global configuration structures.
//!
//! This module contains configuration types that are specific to global-level
//! configuration, including filesystem settings, trusted vaults, and global
//! defaults.
#![allow(
    clippy::exhaustive_structs,
    reason = "rkyv generates Archived types with public fields"
)]

use std::collections::HashMap;

use super::{
    error::ConfigError,
    types::{Frontmatter, Logging, Schema, Template},
};

/// Global default configuration (lowest precedence).
///
/// # Business Rules
/// - Provides system-wide defaults.
/// - Loaded from global lithos.toml or system defaults.
/// - All fields must have values (no optionals).
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
pub struct Global {
    /// Filesystem configuration for global defaults.
    pub filesystem: Paths,
    /// Frontmatter configuration for global defaults.
    pub frontmatter: Frontmatter,
    /// Logging configuration for global defaults.
    pub logging: Logging,
    /// Trusted vaults configuration.
    pub trusted_vaults: Option<TrustedVaults>,
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
    pub schema: Schema,
    /// Template configuration for global library.
    pub template: Template,
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
#[non_exhaustive]
pub struct TrustedVaults {
    /// List format for trusted vault paths.
    pub list: Option<Vec<String>>,
    /// Map format for trusted vault paths with aliases.
    pub map: Option<HashMap<String, String>>,
}

impl Default for Global {
    #[inline]
    fn default() -> Self {
        Self {
            filesystem: Paths::default(),
            frontmatter: Frontmatter::default(),
            logging: Logging::default(),
            trusted_vaults: None,
        }
    }
}

impl Paths {
    /// Validate global filesystem configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigConfigError::ValidationFailed` if schema or template
    /// validation fails.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.schema.validate()?;
        self.template.validate()?;
        Ok(())
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
    /// let trusted = TrustedVaultsConfig {
    ///     list: Some(vec!["/vault".to_string()]),
    ///     map: None,
    /// };
    /// trusted.validate().unwrap();
    /// ```
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Matching on tuple of references (&Option<T>, \
                      &Option<U>) requires this structure to avoid moving \
                      fields. Pattern binding on &self members is idiomatic \
                      for validation of mutually exclusive fields."
        )]
        match (&self.list, &self.map) {
            (Some(_), Some(_)) => Err(ConfigError::ValidationFailed {
                field: "trusted_vaults".to_owned().into(),
                message: "cannot specify both list and map format"
                    .to_owned()
                    .into(),
            }),
            (None, None) => Err(ConfigError::ValidationFailed {
                field: "trusted_vaults".to_owned().into(),
                message: "must specify either list or map format"
                    .to_owned()
                    .into(),
            }),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TrustedVaults;

    #[test]
    fn accepts_trusted_vaults_with_list_only() {
        // GIVEN: trusted vaults configured with list format
        let trusted = TrustedVaults {
            list: Some(vec!["/vaults/alpha".to_owned()]),
            map: None,
        };

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
    fn accepts_trusted_vaults_with_map_only() {
        // GIVEN: trusted vaults configured with map format
        let trusted = TrustedVaults {
            list: None,
            map: Some(
                [("alpha".to_owned(), "/vaults/alpha".to_owned())]
                    .into_iter()
                    .collect(),
            ),
        };

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
    fn filesystem_validate_rejects_invalid_schema() {
        // GIVEN: a filesystem config with an invalid schema path
        let mut filesystem = super::Paths::default();
        filesystem.schema.schemas_dir = String::new();

        // WHEN: validating
        let result = filesystem.validate();

        // THEN: it fails
        assert!(result.is_err());
    }

    #[test]
    fn global_defaults_have_expected_shape() {
        // GIVEN: default global config
        let global = super::Global::default();

        // THEN: default structures are populated
        assert!(global.trusted_vaults.is_none());
        assert_eq!(global.logging.log_level, "info");
    }

    #[test]
    fn rejects_trusted_vaults_with_list_and_map() {
        // GIVEN: trusted vaults configured with both list and map formats
        let trusted = TrustedVaults {
            list: Some(vec!["/vaults/alpha".to_owned()]),
            map: Some(
                [("alpha".to_owned(), "/vaults/alpha".to_owned())]
                    .into_iter()
                    .collect(),
            ),
        };

        // WHEN: validating the trusted vault configuration
        let result = trusted.validate();

        // THEN: validation fails because formats are mixed
        assert!(
            result.is_err(),
            "Expected validation failure when list and map are both set"
        );
    }

    #[test]
    fn rejects_trusted_vaults_with_no_entries() {
        // GIVEN: trusted vaults configured with no list or map
        let trusted = TrustedVaults {
            list: None,
            map: None,
        };

        // WHEN: validating the trusted vault configuration
        let result = trusted.validate();

        // THEN: validation fails because no format is provided
        assert!(
            result.is_err(),
            "Expected validation failure when no trusted vaults are provided"
        );
    }
}
