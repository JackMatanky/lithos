//! Vault-scoped configuration types.

use std::path::{Path, PathBuf};

use super::{
    error::ConfigError,
    frontmatter::Frontmatter,
    logging::Logging,
    paths::{CacheDir, SchemaOverrides, TemplateOverrides},
    raw::{RawVault, RawVaultPaths},
    task::TaskConfig,
};

/// Vault unique identity (UUID v7).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct VaultId(uuid::Uuid);

impl VaultId {
    /// Create a new unique vault identity.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }

    /// Return the raw UUID.
    #[inline]
    #[must_use]
    pub const fn uuid(&self) -> uuid::Uuid {
        self.0
    }
}

impl Default for VaultId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for VaultId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Validated path to a vault root (absolute).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[serde(try_from = "String", into = "String")]
#[non_exhaustive]
pub struct VaultRoot(#[rkyv(with = rkyv::with::AsString)] PathBuf);

impl VaultRoot {
    /// Create a validated vault root path.
    ///
    /// # Errors
    /// Returns [`ConfigError`] if the path is empty.
    #[inline]
    pub fn try_new(path: PathBuf) -> Result<Self, ConfigError> {
        if path.as_os_str().is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "vault_root".to_owned().into(),
                message: "path cannot be empty".to_owned().into(),
            });
        }
        Ok(Self(path))
    }

    /// Return the inner path.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl TryFrom<String> for VaultRoot {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, ConfigError> {
        Self::try_new(PathBuf::from(value))
    }
}

impl From<VaultRoot> for String {
    #[inline]
    fn from(root: VaultRoot) -> Self {
        root.0.to_string_lossy().into_owned()
    }
}

impl Default for VaultRoot {
    #[inline]
    fn default() -> Self {
        Self(PathBuf::from("/"))
    }
}

/// Key used for vault path mapping.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct VaultPathKey(String);

impl VaultPathKey {
    /// Create a path key from a vault root.
    #[inline]
    #[must_use]
    pub fn from_root(root: &VaultRoot) -> Self {
        Self(root.as_path().to_string_lossy().into_owned())
    }

    /// Return the key as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for VaultPathKey {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "vault_path_key".to_owned().into(),
                message: "key cannot be empty".to_owned().into(),
            });
        }
        Ok(Self(value))
    }
}

/// Metadata for a specific vault.
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
    /// Unique identity of the vault.
    id: VaultId,
    /// Absolute path to the vault root on disk.
    root: VaultRoot,
    /// Human-readable name of the vault.
    name: String,
    /// Version of the vault schema/Lithos that created it.
    version: String,
}

impl Metadata {
    /// Create vault metadata.
    ///
    /// # Errors
    /// Returns `ConfigError` if validation fails.
    #[inline]
    pub fn new(
        id: VaultId,
        root: VaultRoot,
        name: Option<String>,
        version: Option<String>,
    ) -> Result<Self, ConfigError> {
        let name = name.unwrap_or_else(|| {
            root.as_path().file_name().map_or_else(
                || "unnamed".to_owned(),
                |n| n.to_string_lossy().into_owned(),
            )
        });
        let version =
            version.unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_owned());

        Ok(Self {
            id,
            root,
            name,
            version,
        })
    }

    /// Return the vault identity.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> VaultId {
        self.id
    }

    /// Return the vault root path.
    #[inline]
    #[must_use]
    pub const fn root(&self) -> &VaultRoot {
        &self.root
    }

    /// Return the vault name.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the vault version.
    #[inline]
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Validate vault metadata.
    ///
    /// # Errors
    /// Returns `ConfigError` if validation fails.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}

impl Default for Metadata {
    #[inline]
    fn default() -> Self {
        Self {
            id: VaultId::default(),
            root: VaultRoot::default(),
            name: "unnamed".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }
}

/// Vault-specific filesystem configuration.
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
    /// Overridden cache directory.
    cache_dir: Option<CacheDir>,
    /// Overridden schema settings.
    schema: SchemaOverrides,
    /// Overridden template settings.
    template: TemplateOverrides,
}

impl Paths {
    /// Create vault-specific filesystem settings.
    #[inline]
    #[must_use]
    pub const fn new(
        cache_dir: Option<CacheDir>,
        schema: SchemaOverrides,
        template: TemplateOverrides,
    ) -> Self {
        Self {
            cache_dir,
            schema,
            template,
        }
    }

    /// Return the overridden cache directory, if set.
    #[inline]
    #[must_use]
    pub fn cache_dir(&self) -> Option<&CacheDir> {
        self.cache_dir.as_ref()
    }

    /// Return the overridden schema settings.
    #[inline]
    #[must_use]
    pub const fn schema(&self) -> &SchemaOverrides {
        &self.schema
    }

    /// Return the overridden template settings.
    #[inline]
    #[must_use]
    pub const fn template(&self) -> &TemplateOverrides {
        &self.template
    }

    /// Validate vault filesystem configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if validation fails.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}

/// Vault-specific configuration.
///
/// Contains overrides for global defaults.
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
pub struct Vault {
    /// Overridden filesystem settings.
    filesystem: Paths,
    /// Overridden frontmatter settings.
    frontmatter: Option<Frontmatter>,
    /// Overridden logging settings.
    logging: Option<Logging>,
    /// Overridden task settings.
    task: Option<TaskConfig>,
}

impl Vault {
    /// Create vault-specific configuration.
    #[inline]
    #[must_use]
    pub const fn new(
        filesystem: Paths,
        frontmatter: Option<Frontmatter>,
        logging: Option<Logging>,
        task: Option<TaskConfig>,
    ) -> Self {
        Self {
            filesystem,
            frontmatter,
            logging,
            task,
        }
    }

    /// Return the overridden filesystem settings.
    #[inline]
    #[must_use]
    pub const fn filesystem(&self) -> &Paths {
        &self.filesystem
    }

    /// Return the overridden frontmatter settings, if set.
    #[inline]
    #[must_use]
    pub fn frontmatter(&self) -> Option<&Frontmatter> {
        self.frontmatter.as_ref()
    }

    /// Return the overridden logging settings, if set.
    #[inline]
    #[must_use]
    pub fn logging(&self) -> Option<&Logging> {
        self.logging.as_ref()
    }

    /// Return the overridden task settings, if set.
    #[inline]
    #[must_use]
    pub fn task(&self) -> Option<&TaskConfig> {
        self.task.as_ref()
    }
}

impl TryFrom<RawVaultPaths> for Paths {
    type Error = ConfigError;

    #[inline]
    fn try_from(raw: RawVaultPaths) -> Result<Self, ConfigError> {
        let cache_dir = raw
            .cache_dir
            .map(|s| CacheDir::try_new(PathBuf::from(s)))
            .transpose()?;
        let schema = raw.schema.unwrap_or_default();
        let template = raw.template.unwrap_or_default();

        Ok(Paths::new(cache_dir, schema, template))
    }
}

impl TryFrom<RawVault> for Vault {
    type Error = ConfigError;

    #[inline]
    /// Build vault configuration from raw input.
    ///
    /// # Errors
    /// Returns `ConfigError` if configuration validation fails.
    fn try_from(raw: RawVault) -> Result<Self, Self::Error> {
        let filesystem = raw
            .filesystem
            .map(Paths::try_from)
            .transpose()?
            .unwrap_or_default();
        let frontmatter =
            raw.frontmatter.map(Frontmatter::try_from).transpose()?;
        let logging = raw.logging.map(Logging::try_from).transpose()?;
        let task = raw.task.map(TaskConfig::from_raw).transpose()?;

        Ok(Vault::new(filesystem, frontmatter, logging, task))
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    clippy::disallowed_methods,
    reason = "Test modules group fixtures and test logic for readability"
)]
mod tests {
    mod fixtures {
        use std::path::PathBuf;

        use super::super::{VaultId, VaultRoot};

        pub fn vault_root(path: &str) -> VaultRoot {
            VaultRoot::try_new(PathBuf::from(path)).expect("valid vault root")
        }

        pub fn vault_id() -> VaultId {
            VaultId::new()
        }
    }

    use std::path::PathBuf;

    use super::{
        CacheDir, ConfigError, Metadata, Paths, Vault, VaultPathKey, VaultRoot,
    };
    use crate::config::{
        logging::{LogLevel, Logging},
        paths::TemplatesDir,
    };

    mod constructor {
        use super::*;

        #[test]
        fn vault_new_constructs_with_given_values() {
            let paths = Paths::default();
            let logging = Logging::new(LogLevel::Debug);
            let vault =
                Vault::new(paths.clone(), None, Some(logging.clone()), None);

            assert_eq!(vault.filesystem(), &paths);
            assert_eq!(vault.logging(), Some(&logging));
        }

        #[test]
        #[expect(
            clippy::panic_in_result_fn,
            reason = "Test uses assert_eq! which can panic."
        )]
        fn metadata_new_derives_from_vault_path()
        -> Result<(), Box<dyn std::error::Error>> {
            // GIVEN: a vault path
            let vault_root = fixtures::vault_root("/vaults/work");

            // WHEN: building metadata from the path
            let metadata = Metadata::new(
                fixtures::vault_id(),
                vault_root.clone(),
                None,
                None,
            )?;

            // THEN: version and name defaults are applied
            assert_eq!(
                metadata.version(),
                env!("CARGO_PKG_VERSION"),
                "Expected version default to be set"
            );
            assert_eq!(
                metadata.name(),
                "work",
                "Expected vault name to default to directory basename"
            );
            assert_eq!(
                metadata.root().as_path(),
                vault_root.as_path(),
                "Expected path to match input"
            );

            Ok(())
        }

        #[test]
        fn cache_dir_rejects_empty() {
            // GIVEN: empty cache_dir
            let result = CacheDir::try_new(PathBuf::from(""));

            // THEN: validation fails for cache_dir
            assert!(
                result.is_err(),
                "Expected validation failure for cache_dir"
            );
        }

        #[test]
        fn vault_root_rejects_empty() {
            // GIVEN: an empty vault path
            let vault_path = "";

            // WHEN: building metadata from the empty path
            let result = VaultRoot::try_new(PathBuf::from(vault_path));

            // THEN: validation fails with a required field error
            assert!(
                result.is_err(),
                "Expected validation failure for empty vault_path"
            );
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn paths_passes_with_defaults() {
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
        fn templates_dir_rejects_absolute() {
            // GIVEN: invalid template dir (absolute)
            let result = TemplatesDir::try_new(PathBuf::from("/abs"));

            // THEN: it fails
            assert!(
                result.is_err(),
                "TemplatesDir should reject absolute paths"
            );
        }
    }

    mod conversions {
        use super::*;

        #[test]
        #[expect(
            clippy::panic_in_result_fn,
            reason = "Test uses assert_eq! which can panic."
        )]
        fn vault_path_key_from_root_preserves_path()
        -> Result<(), Box<dyn std::error::Error>> {
            let root = VaultRoot::try_new(PathBuf::from("/vault/alpha"))?;
            let key = VaultPathKey::from_root(&root);
            assert_eq!(key.as_str(), "/vault/alpha");
            Ok(())
        }

        #[test]
        fn vault_path_key_try_from_rejects_empty() {
            let result = VaultPathKey::try_from(String::new());
            let _: ConfigError =
                result.expect_err("VaultPathKey should reject empty string");
        }
    }
}
