//! Global configuration structures.
//!
//! This module contains configuration types that are specific to global-level
//! configuration, including filesystem settings, trusted vaults, and global
//! defaults.

use std::collections::HashMap;

use super::types::{Frontmatter, Logging, Schema, Template};

/// Global filesystem configuration (global template/schema library).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[derive(Default)]
pub struct Filesystem {
    /// Schema configuration for global library.
    pub schema: Schema,
    /// Template configuration for global library.
    pub template: Template,
}

impl Filesystem {
    /// Validate global filesystem configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::ValidationFailed` if schema or template validation
    /// fails.
    #[inline]
    pub fn validate(&self) -> Result<(), crate::ConfigError> {
        self.schema.validate()?;
        self.template.validate()?;
        Ok(())
    }
}

/// Global default configuration (lowest precedence).
///
/// # Business Rules
/// - Provides system-wide defaults.
/// - Loaded from global lithos.toml or system defaults.
/// - All fields must have values (no optionals).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Global {
    /// Filesystem configuration for global defaults.
    pub filesystem: Filesystem,
    /// Frontmatter configuration for global defaults.
    pub frontmatter: Frontmatter,
    /// Logging configuration for global defaults.
    pub logging: Logging,
    /// Trusted vaults configuration.
    pub trusted_vaults: Option<TrustedVaults>,
}

impl Default for Global {
    #[inline]
    fn default() -> Self {
        Self {
            filesystem: Filesystem::default(),
            frontmatter: Frontmatter::default(),
            logging: Logging::default(),
            trusted_vaults: None,
        }
    }
}

/// Trusted vaults configuration supporting list or map format.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct TrustedVaults {
    /// List format for trusted vault paths.
    pub list: Option<Vec<String>>,
    /// Map format for trusted vault paths with aliases.
    pub map: Option<HashMap<String, String>>,
}

impl TrustedVaults {
    /// Validate trusted vaults configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::ValidationFailed` if both list and map are
    /// specified, or if neither is specified.
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_domain::TrustedVaultsConfig;
    /// let trusted = TrustedVaultsConfig {
    ///     list: Some(vec!["/vault".to_string()]),
    ///     map: None,
    /// };
    /// trusted.validate().unwrap();
    /// ```
    #[inline]
    pub fn validate(&self) -> Result<(), crate::ConfigError> {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Pattern matching on Option types requires this structure"
        )]
        match (&self.list, &self.map) {
            (Some(_), Some(_)) => Err(crate::ConfigError::ValidationFailed {
                field: "trusted_vaults".to_owned().into(),
                message: "cannot specify both list and map format"
                    .to_owned()
                    .into(),
            }),
            (None, None) => Err(crate::ConfigError::ValidationFailed {
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
        #[expect(
            clippy::disallowed_methods,
            reason = "Test validation uses unwrap for clarity"
        )]
        result.unwrap();
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
        #[expect(
            clippy::disallowed_methods,
            reason = "Test validation uses unwrap for clarity"
        )]
        result.unwrap();
    }

    #[test]
    fn filesystem_validate_passes_with_defaults() {
        // GIVEN: a default filesystem config
        let filesystem = super::Filesystem::default();

        // WHEN: validating
        let result = filesystem.validate();

        // THEN: it succeeds
        #[expect(
            clippy::disallowed_methods,
            reason = "Test validation uses unwrap for clarity"
        )]
        result.unwrap();
    }

    #[test]
    fn filesystem_validate_rejects_invalid_schema() {
        // GIVEN: a filesystem config with an invalid schema path
        let mut filesystem = super::Filesystem::default();
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
