//! Vault-specific configuration structures.
//!
//! This module contains configuration types that are specific to vault-level
//! configuration, including filesystem settings, metadata, and vault overrides.
#![allow(
    clippy::exhaustive_structs,
    reason = "rkyv generates Archived types with public fields"
)]

use super::{
    error::ConfigError,
    types::{Frontmatter, Logging, Schema, Template},
};

/// Vault metadata with versioning and naming.
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
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct Metadata {
    /// Human-readable name for the vault.
    pub name: String,
    /// Root path to the vault (absolute path required).
    pub path: String,
    /// Schema version for the vault.
    pub version: SchemaVersion,
}

/// Vault filesystem configuration (vault-scoped).
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
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct Paths {
    /// Cache directory for vault.
    pub cache_dir: String,
    /// Schema configuration for vault.
    pub schema: Schema,
    /// Template configuration for vault.
    pub template: Template,
}

/// Schema version for the vault.
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
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct SchemaVersion(pub String);

/// Vault-specific configuration (highest precedence).
///
/// # Business Rules
/// - Vault configuration overrides Global configuration.
/// - Loaded from vault-specific lithos.toml.
/// - All fields optional (missing fields fall back to global).
#[derive(
    Debug,
    Default,
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
pub struct Vault {
    /// Filesystem configuration for vault.
    pub filesystem: Paths,
    /// Frontmatter configuration for vault (optional overrides).
    pub frontmatter: Option<Frontmatter>,
    /// Logging configuration for vault (optional overrides).
    pub logging: Option<Logging>,
}

impl Default for Metadata {
    #[inline]
    fn default() -> Self {
        Self {
            name: String::new(),
            path: String::new(),
            version: SchemaVersion::default(),
        }
    }
}

impl Default for Paths {
    #[inline]
    fn default() -> Self {
        Self {
            cache_dir: ".cache".to_owned(),
            schema: Schema::default(),
            template: Template::default(),
        }
    }
}

impl Default for SchemaVersion {
    #[inline]
    fn default() -> Self {
        Self::new(None)
    }
}

impl std::ops::Deref for SchemaVersion {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Metadata {
    /// Create new vault metadata, deriving name from path if not provided.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if path is empty.
    ///
    /// # Examples
    /// ```
    /// # use lithos_core::config::vault::Metadata;
    /// let metadata =
    ///     Metadata::new("/vaults/work".to_string(), None, None).unwrap();
    /// assert_eq!(metadata.path, "/vaults/work");
    /// ```
    #[inline]
    pub fn new(
        path: String,
        name: Option<String>,
        version: Option<String>,
    ) -> Result<Self, ConfigError> {
        let derived_name = name
            .or_else(|| {
                std::path::Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(std::borrow::ToOwned::to_owned)
            })
            .unwrap_or_default();

        let metadata = Self {
            path,
            name: derived_name,
            version: SchemaVersion::new(version),
        };
        metadata.validate()?;
        Ok(metadata)
    }

    /// Validate vault metadata.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if path is empty.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.path.is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "vault_path".to_owned().into(),
                message: "vault path cannot be empty (required field)"
                    .to_owned()
                    .into(),
            });
        }
        Ok(())
    }
}

impl Paths {
    /// Create new vault filesystem configuration.
    #[inline]
    #[must_use]
    pub fn new(cache_dir: String, schema: Schema, template: Template) -> Self {
        Self {
            cache_dir,
            schema,
            template,
        }
    }

    /// Validate vault filesystem configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigConfigError::ValidationFailed` if `cache_dir` is empty or
    /// if schema/template validation fails.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.schema.validate()?;
        self.template.validate()?;
        if self.cache_dir.is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "cache_dir".to_owned().into(),
                message: "cache directory cannot be empty".to_owned().into(),
            });
        }
        Ok(())
    }
}

impl SchemaVersion {
    /// Create new schema version, defaulting to binary version if not provided.
    #[inline]
    #[must_use]
    pub fn new(version: Option<String>) -> Self {
        Self(version.unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_owned()))
    }
}

impl Vault {
    /// Create new vault configuration.
    #[inline]
    #[must_use]
    pub fn new(
        filesystem: Paths,
        frontmatter: Option<Frontmatter>,
        logging: Option<Logging>,
    ) -> Self {
        Self {
            filesystem,
            frontmatter,
            logging,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Test module uses Result::unwrap() for ergonomic arrangement and \
              assertions. Acceptable in test-only code paths."
)]
mod tests {
    use super::{Metadata, Paths};

    #[test]
    fn derives_metadata_from_vault_path() {
        // GIVEN: a vault path
        let vault_path = "/vaults/work".to_owned();

        // WHEN: building metadata from the path
        let metadata = Metadata::new(vault_path.clone(), None, None).unwrap();

        // THEN: version and name defaults are applied
        assert_eq!(
            &*metadata.version,
            env!("CARGO_PKG_VERSION"),
            "Expected version default to be set"
        );
        assert_eq!(
            metadata.name, "work",
            "Expected vault name to default to directory basename"
        );
        assert_eq!(metadata.path, vault_path, "Expected path to match input");
    }

    #[test]
    fn filesystem_validate_passes_with_defaults() {
        // GIVEN: default filesystem config
        let filesystem = Paths::default();

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
    fn filesystem_validate_rejects_invalid_template() {
        // GIVEN: a vault filesystem with invalid template config
        let mut filesystem = Paths::default();
        filesystem.template.templates_dir = String::new();

        // WHEN: validating
        let result = filesystem.validate();

        // THEN: it fails
        assert!(
            result.is_err(),
            "Filesystem with empty paths should fail validation"
        );
    }

    #[test]
    fn rejects_empty_cache_dir() {
        // GIVEN: a filesystem with empty cache_dir
        let filesystem = Paths {
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

        // WHEN: building metadata from the empty path
        let result = Metadata::new(vault_path.to_owned(), None, None);

        // THEN: validation fails with a required field error
        assert!(
            result.is_err(),
            "Expected validation failure for empty vault_path"
        );
    }
}
