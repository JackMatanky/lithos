//! Vault-specific configuration structures.
//!
//! This module contains configuration types that are specific to vault-level
//! configuration, including filesystem settings, metadata, and vault overrides.

use std::path::{Path, PathBuf};

use uuid::Uuid;

use super::{
    error::ConfigError,
    frontmatter::Frontmatter,
    paths::{FileName, SchemasDir, TemplatesDir},
    raw::{RawVault, RawVaultPaths},
    task::TaskConfig,
    types::Logging,
};

/// Stable vault identity.
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
pub struct VaultId(
    /// Internal UUID storage.
    Uuid,
);

/// Validated vault root path (absolute).
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
pub struct VaultRoot(
    /// Internal path storage.
    #[rkyv(with = rkyv::with::AsString)]
    PathBuf,
);

/// Canonical path key for lookup.
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
pub struct VaultPathKey(
    /// Internal key storage.
    Box<str>,
);

/// Vault-relative cache directory.
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
pub struct CacheDir(
    /// Internal path storage.
    #[rkyv(with = rkyv::with::AsString)]
    PathBuf,
);

/// Schema path overrides at the vault layer.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct SchemaOverrides {
    /// Overridden schemas directory.
    schemas_dir: Option<SchemasDir>,
    /// Overridden property bank filename.
    property_bank_filename: Option<FileName>,
}

/// Template path overrides at the vault layer.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct TemplateOverrides {
    /// Overridden templates directory.
    templates_dir: Option<TemplatesDir>,
}

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
    /// Stable vault identity.
    id: VaultId,
    /// Human-readable name for the vault.
    name: Box<str>,
    /// Root path to the vault (absolute path required).
    root: VaultRoot,
    /// Schema version for the vault.
    version: SchemaVersion,
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
    cache_dir: Option<CacheDir>,
    /// Schema configuration for vault.
    schema: SchemaOverrides,
    /// Template configuration for vault.
    template: TemplateOverrides,
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
pub struct SchemaVersion(
    /// Internal version storage.
    String,
);

/// Vault-specific configuration.
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
#[non_exhaustive]
pub struct Vault {
    /// Filesystem configuration for vault.
    filesystem: Paths,
    /// Frontmatter configuration for vault (optional overrides).
    frontmatter: Option<Frontmatter>,
    /// Logging configuration for vault (optional overrides).
    logging: Option<Logging>,
    /// Task configuration overrides.
    task: Option<TaskConfig>,
}

impl Default for Metadata {
    #[inline]
    fn default() -> Self {
        Self {
            id: VaultId::nil(),
            name: Box::from(""),
            root: VaultRoot(PathBuf::from("/")),
            version: SchemaVersion::default(),
        }
    }
}

impl Default for Paths {
    #[inline]
    fn default() -> Self {
        Self {
            cache_dir: None,
            schema: SchemaOverrides::default(),
            template: TemplateOverrides::default(),
        }
    }
}

impl SchemaOverrides {
    #[inline]
    #[must_use]
    /// Create schema override values.
    pub fn new(
        schemas_dir: Option<SchemasDir>,
        property_bank_filename: Option<FileName>,
    ) -> Self {
        Self {
            schemas_dir,
            property_bank_filename,
        }
    }

    #[inline]
    #[must_use]
    /// Return the overridden schemas directory, if set.
    pub fn schemas_dir(&self) -> Option<&SchemasDir> {
        self.schemas_dir.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return the overridden property bank filename, if set.
    pub fn property_bank_filename(&self) -> Option<&FileName> {
        self.property_bank_filename.as_ref()
    }
}

impl TemplateOverrides {
    #[inline]
    #[must_use]
    /// Create template override values.
    pub fn new(templates_dir: Option<TemplatesDir>) -> Self {
        Self {
            templates_dir,
        }
    }

    #[inline]
    #[must_use]
    /// Return the overridden templates directory, if set.
    pub fn templates_dir(&self) -> Option<&TemplatesDir> {
        self.templates_dir.as_ref()
    }
}

impl Metadata {
    #[inline]
    #[must_use]
    /// Return the vault identifier.
    pub fn id(&self) -> VaultId {
        self.id
    }

    #[inline]
    #[must_use]
    /// Return the vault display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    #[must_use]
    /// Return the vault root path.
    pub fn root(&self) -> &VaultRoot {
        &self.root
    }

    #[inline]
    #[must_use]
    /// Return the schema version.
    pub fn version(&self) -> &SchemaVersion {
        &self.version
    }

    /// Create new vault metadata, deriving name from path if not provided.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if path is empty.
    #[inline]
    pub fn new(
        id: VaultId,
        root: VaultRoot,
        name: Option<String>,
        version: Option<String>,
    ) -> Result<Self, ConfigError> {
        let derived_name = name
            .or_else(|| {
                root.as_path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(std::borrow::ToOwned::to_owned)
            })
            .unwrap_or_default();

        let metadata = Self {
            id,
            root,
            name: derived_name.into(),
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
        Ok(())
    }
}

impl Paths {
    #[inline]
    #[must_use]
    /// Return the cache directory, if set.
    pub fn cache_dir(&self) -> Option<&CacheDir> {
        self.cache_dir.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return schema override settings.
    pub fn schema(&self) -> &SchemaOverrides {
        &self.schema
    }

    #[inline]
    #[must_use]
    /// Return template override settings.
    pub fn template(&self) -> &TemplateOverrides {
        &self.template
    }

    /// Create a new vault filesystem configuration.
    #[inline]
    #[must_use]
    pub fn new(
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

    /// Validate vault filesystem configuration.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if `cache_dir` is empty or if
    /// schema/template validation fails.
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}

impl Vault {
    #[inline]
    #[must_use]
    /// Return vault filesystem settings.
    pub fn filesystem(&self) -> &Paths {
        &self.filesystem
    }

    #[inline]
    #[must_use]
    /// Return frontmatter overrides, if set.
    pub fn frontmatter(&self) -> Option<&Frontmatter> {
        self.frontmatter.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return logging overrides, if set.
    pub fn logging(&self) -> Option<&Logging> {
        self.logging.as_ref()
    }

    #[inline]
    #[must_use]
    /// Return task configuration overrides, if set.
    pub fn task(&self) -> Option<&TaskConfig> {
        self.task.as_ref()
    }

    /// Create new vault configuration.
    #[inline]
    #[must_use]
    pub fn new(
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
}

impl Default for SchemaVersion {
    #[inline]
    fn default() -> Self {
        Self::new(None)
    }
}

impl std::fmt::Display for SchemaVersion {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn validate_relative_path(
    field: &'static str,
    path: &Path,
) -> Result<(), ConfigError> {
    if path.as_os_str().is_empty() {
        return Err(ConfigError::ValidationFailed {
            field: field.to_owned().into(),
            message: format!("{field} cannot be empty").into(),
        });
    }
    if path.is_absolute() {
        return Err(ConfigError::ValidationFailed {
            field: field.to_owned().into(),
            message: format!("{field} must be vault-relative").into(),
        });
    }
    if path
        .components()
        .any(|component| component == std::path::Component::ParentDir)
    {
        return Err(ConfigError::ValidationFailed {
            field: field.to_owned().into(),
            message: format!("{field} must not contain parent components")
                .into(),
        });
    }
    Ok(())
}

impl VaultId {
    #[inline]
    #[must_use]
    /// Create a new vault identifier.
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    #[inline]
    #[must_use]
    /// Return the nil vault identifier.
    pub fn nil() -> Self {
        Self(Uuid::nil())
    }

    #[inline]
    #[must_use]
    /// Return the underlying UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
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

impl VaultRoot {
    #[inline]
    /// Create a validated vault root path.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the path is empty or not
    /// absolute.
    pub fn try_new(root: PathBuf) -> Result<Self, ConfigError> {
        if root.as_os_str().is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "vault_root".to_owned().into(),
                message: "vault root cannot be empty".to_owned().into(),
            });
        }
        if !root.is_absolute() {
            return Err(ConfigError::ValidationFailed {
                field: "vault_root".to_owned().into(),
                message: "vault root must be absolute".to_owned().into(),
            });
        }
        Ok(Self(root))
    }

    #[inline]
    #[must_use]
    /// Return the root path.
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl TryFrom<String> for VaultRoot {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(PathBuf::from(value))
    }
}

impl From<VaultRoot> for String {
    #[inline]
    fn from(value: VaultRoot) -> Self {
        value.0.to_string_lossy().into_owned()
    }
}

impl From<VaultId> for String {
    #[inline]
    fn from(value: VaultId) -> Self {
        value.0.to_string()
    }
}

impl VaultPathKey {
    #[inline]
    #[must_use]
    /// Create a vault path key from a root.
    pub fn from_root(root: &VaultRoot) -> Self {
        Self(root.as_path().to_string_lossy().into())
    }

    #[inline]
    #[must_use]
    /// Return the path key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for VaultPathKey {
    type Error = ConfigError;

    #[inline]
    /// Create a vault path key from a string.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the key is empty.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "vault_path_key".to_owned().into(),
                message: "vault path key cannot be empty".to_owned().into(),
            });
        }
        Ok(Self(value.into()))
    }
}

impl From<VaultPathKey> for String {
    #[inline]
    fn from(value: VaultPathKey) -> Self {
        value.0.into()
    }
}

impl CacheDir {
    #[inline]
    /// Create a validated cache directory path.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if the path is invalid.
    pub fn try_new(path: PathBuf) -> Result<Self, ConfigError> {
        validate_relative_path("cache_dir", &path)?;
        Ok(Self(path))
    }

    #[inline]
    #[must_use]
    /// Return the cache directory path.
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl Default for CacheDir {
    #[inline]
    fn default() -> Self {
        Self(PathBuf::from(".cache"))
    }
}

impl TryFrom<String> for CacheDir {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(PathBuf::from(value))
    }
}

impl From<CacheDir> for String {
    #[inline]
    fn from(value: CacheDir) -> Self {
        value.0.to_string_lossy().into_owned()
    }
}

impl SchemaVersion {
    /// Create new schema version, defaulting to binary version if not provided.
    #[inline]
    #[must_use]
    pub fn new(version: Option<String>) -> Self {
        Self(version.unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_owned()))
    }

    #[inline]
    #[must_use]
    /// Return the schema version as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<RawVaultPaths> for Paths {
    type Error = ConfigError;

    #[inline]
    /// Build vault filesystem paths from raw input.
    ///
    /// # Errors
    /// Returns `ConfigError` if any path validation fails.
    fn try_from(raw: RawVaultPaths) -> Result<Self, Self::Error> {
        let cache_dir = raw
            .cache_dir
            .map(|value| CacheDir::try_new(PathBuf::from(value)))
            .transpose()?;
        let schema = match raw.schema {
            Some(schema) => SchemaOverrides::new(
                schema
                    .schemas_dir
                    .map(|value| {
                        SchemasDir::try_new_with_field(
                            "schemas_dir",
                            PathBuf::from(value),
                        )
                    })
                    .transpose()?,
                schema
                    .property_bank_filename
                    .map(|value| {
                        FileName::try_new_with_field(
                            "property_bank_filename",
                            value,
                        )
                    })
                    .transpose()?,
            ),
            None => SchemaOverrides::default(),
        };
        let template = match raw.template {
            Some(template) => TemplateOverrides::new(
                template
                    .templates_dir
                    .map(|value| {
                        TemplatesDir::try_new_with_field(
                            "templates_dir",
                            PathBuf::from(value),
                        )
                    })
                    .transpose()?,
            ),
            None => TemplateOverrides::default(),
        };

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
        let task = raw.task.map(TaskConfig::try_from).transpose()?;

        Ok(Vault::new(filesystem, frontmatter, logging, task))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{CacheDir, Metadata, Paths, TemplatesDir, VaultId, VaultRoot};

    #[test]
    fn derives_metadata_from_vault_path() -> Result<(), String> {
        // GIVEN: a vault path
        let vault_root = VaultRoot::try_new(PathBuf::from("/vaults/work"))
            .map_err(|err| format!("VaultRoot::try_new failed: {err:?}"))?;

        // WHEN: building metadata from the path
        let metadata =
            Metadata::new(VaultId::new(), vault_root.clone(), None, None)
                .map_err(|err| format!("Metadata::new failed: {err:?}"))?;

        // THEN: version and name defaults are applied
        if metadata.version().as_str() != env!("CARGO_PKG_VERSION") {
            return Err("Expected version default to be set".to_owned());
        }
        if metadata.name() != "work" {
            return Err("Expected vault name to default to directory basename"
                .to_owned());
        }
        if metadata.root().as_path() != vault_root.as_path() {
            return Err("Expected path to match input".to_owned());
        }

        Ok(())
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
        // GIVEN: invalid template dir (absolute)
        let result = TemplatesDir::try_new(PathBuf::from("/abs"));

        // THEN: it fails
        assert!(result.is_err(), "TemplatesDir should reject absolute paths");
    }

    #[test]
    fn rejects_empty_cache_dir() {
        // GIVEN: empty cache_dir
        let result = CacheDir::try_new(PathBuf::from(""));

        // THEN: validation fails for cache_dir
        assert!(result.is_err(), "Expected validation failure for cache_dir");
    }

    #[test]
    fn rejects_empty_vault_path() {
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
