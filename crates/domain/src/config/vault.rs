//! Vault-specific configuration structures.
//!
//! This module contains configuration types that are specific to vault-level
//! configuration, including filesystem settings, metadata, and vault overrides.

use super::types::{Frontmatter, Logging, Schema, Template};

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
                field: "cache_dir".to_owned().into(),
                message: "cache directory cannot be empty".to_owned().into(),
            });
        }
        Ok(())
    }
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

impl Metadata {
    /// Derive vault name from vault path (defaults to directory basename).
    fn derive_vault_name(vault_path: &str) -> Option<String> {
        std::path::Path::new(vault_path)
            .file_name()
            .and_then(|n| n.to_str())
            .map(std::borrow::ToOwned::to_owned)
    }

    /// Create vault metadata with defaults derived from vault path.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::VaultMetadata;
    /// let metadata = VaultMetadata::new("/vaults/work".to_string());
    /// assert_eq!(metadata.vault_path, "/vaults/work");
    /// ```
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
                field: "vault_path".to_owned().into(),
                message: "vault path cannot be empty".to_owned().into(),
            });
        }
        Ok(())
    }

    /// Validate vault path value before creating vault metadata.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if `vault_path` is empty.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::VaultMetadata;
    /// VaultMetadata::validate_vault_path("/vault").unwrap();
    /// ```
    #[inline]
    pub fn validate_vault_path(
        vault_path: &str,
    ) -> Result<(), crate::ConfigError> {
        if vault_path.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "vault_path".to_owned().into(),
                message: "vault path cannot be empty (required field)"
                    .to_owned()
                    .into(),
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

#[cfg(test)]
mod tests {
    use super::{Filesystem, Metadata};

    #[test]
    fn derives_metadata_from_vault_path() {
        // GIVEN: a vault path
        let vault_path = "/vaults/work".to_owned();

        // WHEN: building metadata from the path
        let metadata = Metadata::new(vault_path.clone());

        // THEN: schema_version and name defaults are applied
        assert!(
            metadata.schema_version.is_some(),
            "Expected schema_version default to be set"
        );
        assert_eq!(
            metadata.name.as_deref(),
            Some("work"),
            "Expected vault name to default to directory basename"
        );
        assert_eq!(
            metadata.vault_path, vault_path,
            "Expected vault_path to match input"
        );
    }

    #[test]
    fn filesystem_validate_passes_with_defaults() {
        // GIVEN: default filesystem config
        let filesystem = Filesystem::default();

        // WHEN: validating
        let result = filesystem.validate();

        // THEN: it succeeds
        #[expect(
            clippy::disallowed_methods,
            reason = "Test assertion uses unwrap for clarity"
        )]
        result.unwrap();
    }

    #[test]
    fn filesystem_validate_rejects_invalid_template() {
        // GIVEN: a vault filesystem with invalid template config
        let mut filesystem = Filesystem::default();
        filesystem.template.templates_dir = String::new();

        // WHEN: validating
        let result = filesystem.validate();

        // THEN: it fails
        assert!(result.is_err());
    }

    #[test]
    fn rejects_empty_cache_dir() {
        // GIVEN: a filesystem with empty cache_dir
        let filesystem = Filesystem {
            cache_dir: String::new(),
            schema: super::Schema::default(),
            template: super::Template::default(),
        };

        // WHEN: validating the filesystem configuration
        let result = filesystem.validate();

        // THEN: validation fails for cache_dir
        assert!(
            result.is_err(),
            "Expected validation failure for empty cache_dir"
        );
    }

    #[test]
    fn rejects_empty_vault_path() {
        // GIVEN: an empty vault path
        let vault_path = "";

        // WHEN: validating the vault path
        let result = Metadata::validate_vault_path(vault_path);

        // THEN: validation fails with a required field error
        assert!(
            result.is_err(),
            "Expected validation failure for empty vault_path"
        );
    }
}
