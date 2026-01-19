//! Vault-specific configuration structures.
//!
//! This module contains configuration types that are specific to vault-level
//! configuration, including filesystem settings, metadata, and vault overrides.

use super::types::{Frontmatter, Logging, Schema, Template};

/// Vault metadata with schema versioning and naming.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Metadata {
    /// Human-readable name for the vault (defaults to directory basename).
    pub name: Option<String>,
    /// Schema version for the vault (defaults to binary version).
    pub schema_version: Option<String>,
    /// Root path to the vault (absolute path required).
    pub vault_path: String,
}

impl Default for Metadata {
    #[inline]
    fn default() -> Self {
        Self {
            schema_version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            name: None,
            vault_path: String::new(),
        }
    }
}

impl Metadata {
    /// Derive vault name from vault path (defaults to directory basename).
    fn derive_vault_name(vault_path: &str) -> Option<String> {
        std::path::Path::new(vault_path)
            .file_name()
            .and_then(|n| n.to_str())
            .map(std::borrow::ToOwned::to_owned)
    }

    /// Create vault metadata with defaults derived from vault path.
    #[inline]
    #[must_use]
    pub fn new(vault_path: String) -> Self {
        Self {
            schema_version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            name: Self::derive_vault_name(&vault_path),
            vault_path,
        }
    }

    /// Validate vault metadata.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::ValidationFailed` if `vault_path` is empty.
    #[inline]
    pub fn validate(&self) -> Result<(), crate::ConfigError> {
        if self.vault_path.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "vault_path".to_owned(),
                message: "vault path cannot be empty".to_owned(),
            });
        }
        Ok(())
    }

    /// Validate vault path value before creating vault metadata.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if `vault_path` is empty.
    #[inline]
    pub fn validate_vault_path(
        vault_path: &str,
    ) -> Result<(), crate::ConfigError> {
        if vault_path.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "vault_path".to_owned(),
                message: "vault path cannot be empty (required field)"
                    .to_owned(),
            });
        }

        Ok(())
    }
}

/// Vault filesystem configuration (vault-scoped).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Filesystem {
    /// Cache directory for vault.
    pub cache_dir: String,
    /// Schema configuration for vault.
    pub schema: Schema,
    /// Template configuration for vault.
    pub template: Template,
}

impl Default for Filesystem {
    #[inline]
    fn default() -> Self {
        Self {
            cache_dir: ".cache".to_owned(),
            schema: Schema::default(),
            template: Template::default(),
        }
    }
}

impl Filesystem {
    /// Validate vault filesystem configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::ValidationFailed` if `cache_dir` is empty or
    /// if schema/template validation fails.
    #[inline]
    pub fn validate(&self) -> Result<(), crate::ConfigError> {
        self.schema.validate()?;
        self.template.validate()?;
        if self.cache_dir.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "cache_dir".to_owned(),
                message: "cache directory cannot be empty".to_owned(),
            });
        }
        Ok(())
    }
}

/// Vault-specific configuration (highest precedence).
///
/// # Business Rules
/// - Vault configuration overrides Global configuration.
/// - Loaded from vault-specific lithos.toml.
/// - All fields optional (missing fields fall back to global).
#[derive(
    Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct Vault {
    /// Filesystem configuration for vault.
    pub filesystem: Filesystem,
    /// Frontmatter configuration for vault (optional overrides).
    pub frontmatter: Option<Frontmatter>,
    /// Logging configuration for vault (optional overrides).
    pub logging: Option<Logging>,
}
