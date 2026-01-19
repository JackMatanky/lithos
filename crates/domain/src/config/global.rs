//! Global configuration structures.
//!
//! This module contains configuration types that are specific to global-level
//! configuration, including filesystem settings, trusted vaults, and global defaults.

use std::collections::HashMap;

use super::types::{Frontmatter, Logging, Schema, Template};

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
    /// Returns `ConfigError::ValidationFailed` if both list and map are specified,
    /// or if neither is specified.
    #[inline]
    pub fn validate(&self) -> Result<(), crate::ConfigError> {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Pattern matching on Option types requires this structure"
        )]
        match (&self.list, &self.map) {
            (Some(_), Some(_)) => Err(crate::ConfigError::ValidationFailed {
                field: "trusted_vaults".to_owned(),
                message: "cannot specify both list and map format".to_owned(),
            }),
            (None, None) => Err(crate::ConfigError::ValidationFailed {
                field: "trusted_vaults".to_owned(),
                message: "must specify either list or map format".to_owned(),
            }),
            _ => Ok(()),
        }
    }
}

/// Global filesystem configuration (global template/schema library).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[derive(Default)]
#[expect(
    clippy::module_name_repetitions,
    reason = "Struct name matches module name which is conventional for configuration types"
)]
pub struct GlobalFilesystem {
    /// Schema configuration for global library.
    pub schema: Schema,
    /// Template configuration for global library.
    pub template: Template,
}

impl GlobalFilesystem {
    /// Validate global filesystem configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::ValidationFailed` if schema or template validation fails.
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
    pub filesystem: GlobalFilesystem,
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
            filesystem: GlobalFilesystem::default(),
            frontmatter: Frontmatter::default(),
            logging: Logging::default(),
            trusted_vaults: None,
        }
    }
}
