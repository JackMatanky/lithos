//! Vault-specific overrides and metadata.
//!
//! This module defines the [`Vault`] configuration, which contains
//! vault-specific settings and overrides for global defaults. It also
//! manages [`VaultId`] and [`VaultRoot`].

use std::path::{Path, PathBuf};

use rkyv::{Archive, Deserialize, Serialize};
use trace_fs::DirPath;
use trace_utils::UuidV7;

use super::{
    cache::{CacheConfig, CacheDir},
    error::ConfigError,
    frontmatter::Frontmatter,
    logging::Logging,
    schema::{PropertyBankFile, SchemaConfig, SchemaDir},
    task::Task,
    template::{TemplateConfig, TemplateDir},
};

// ----------------------------------------------------------- //
//                  Fundamental Building Blocks                //
// ----------------------------------------------------------- //

// ----------------------------------------------------------- //
//                    Building Block Types                     //
// ----------------------------------------------------------- //

/// Vault unique identity using UUID v7.
///
/// UUID v7 is used for its time-ordered properties, which helps with
/// database indexing and debugging.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct VaultId(pub(crate) UuidV7);

impl VaultId {
    /// Create a new unique vault identity.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(UuidV7::new())
    }

    /// Return the inner UUID v7.
    #[inline]
    #[must_use]
    pub const fn as_uuid_v7(&self) -> &UuidV7 {
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
        write!(f, "{}", self.0.as_uuid())
    }
}
/// A validated, absolute path to a vault root.
///
/// # Invariants
///
/// - Must be a validated directory path.
///
/// # Errors
///
/// Returns [`ConfigError::ValidationFailed`] if the path is invalid.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct VaultRoot(DirPath);

impl VaultRoot {
    /// Creates a validated vault root path.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the path is not a valid
    /// directory.
    #[inline]
    pub fn try_new(path: PathBuf) -> Result<Self, ConfigError> {
        let dir = DirPath::try_from(path).map_err(|error| {
            ConfigError::ValidationFailed {
                field: "vault_root".into(),
                message: format!("invalid vault_root: {error}").into(),
            }
        })?;
        Ok(Self(dir))
    }

    /// Creates a vault root from an already-validated directory path.
    #[inline]
    #[must_use]
    pub const fn from_dir_path(path: DirPath) -> Self {
        Self(path)
    }

    /// Returns the validated directory path.
    #[inline]
    #[must_use]
    pub const fn as_dir_path(&self) -> &DirPath {
        &self.0
    }

    /// Return the inner path.
    #[inline]
    #[must_use]
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }

    /// Return this path as a string suitable for DB key lookups.
    #[inline]
    #[must_use]
    pub fn as_key(&self) -> String {
        self.as_path().to_string_lossy().into_owned()
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
        root.0.as_path().to_string_lossy().into_owned()
    }
}

impl Default for VaultRoot {
    #[inline]
    #[expect(clippy::expect_used, reason = "Root path is guaranteed non-empty")]
    fn default() -> Self {
        Self::try_new(PathBuf::from("/")).expect("root path is non-empty")
    }
}

// ----------------------------------------------------------- //
//                     Main Domain Types                      //
// ----------------------------------------------------------- //

/// Vault-specific configuration overrides.
///
/// `Vault` contains settings that are specific to a single vault and
/// override the global defaults. It covers paths, frontmatter, and
/// logging settings.
///
/// # Examples
///
/// ```rust
/// use trace_settings::config::vault::Vault;
///
/// let vault = Vault::default();
/// assert!(vault.logging().is_none());
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct Vault {
    /// Version number for this vault config.
    version: VaultVersion,
    /// Overridden logging settings.
    logging: Option<Logging>,
    /// Overridden cache settings.
    cache: Option<CacheConfig>,
    /// Overridden template settings.
    template: Option<TemplateConfig>,
    /// Overridden schema settings.
    schema: Option<SchemaConfig>,
    /// Overridden frontmatter settings.
    frontmatter: Option<Frontmatter>,
    /// Overridden task settings.
    task: Option<Task>,
}

impl Default for Vault {
    #[inline]
    fn default() -> Self {
        Self {
            version: VaultVersion::initial(),
            logging: None,
            cache: None,
            template: None,
            schema: None,
            frontmatter: None,
            task: None,
        }
    }
}

impl Vault {
    /// Create vault-specific configuration.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Domain constructor with all optional override groups"
    )]
    pub const fn new(
        version: VaultVersion,
        logging: Option<Logging>,
        cache: Option<CacheConfig>,
        template: Option<TemplateConfig>,
        schema: Option<SchemaConfig>,
        frontmatter: Option<Frontmatter>,
        task: Option<Task>,
    ) -> Self {
        Self {
            version,
            logging,
            cache,
            template,
            schema,
            frontmatter,
            task,
        }
    }

    /// Creates vault-specific configuration from raw path overrides.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if any configured path is
    /// invalid.
    #[inline]
    pub fn try_from_components(
        cache: Option<&super::raw::RawCacheConfig>,
        template: Option<&super::raw::RawTemplateConfig>,
        schema: Option<&super::raw::RawSchemaConfig>,
    ) -> Result<Self, ConfigError> {
        Ok(Self {
            cache: cache.map(parse_cache).transpose()?.flatten(),
            template: template.map(parse_template).transpose()?.flatten(),
            schema: schema.map(parse_schema).transpose()?.flatten(),
            ..Self::default()
        })
    }

    /// Return the version of this vault config.
    #[inline]
    #[must_use]
    pub const fn version(&self) -> VaultVersion {
        self.version
    }

    /// Return the overridden cache settings, if set.
    #[inline]
    #[must_use]
    pub fn cache(&self) -> Option<&CacheConfig> {
        self.cache.as_ref()
    }

    /// Return the overridden template settings, if set.
    #[inline]
    #[must_use]
    pub fn template(&self) -> Option<&TemplateConfig> {
        self.template.as_ref()
    }

    /// Return the overridden schema settings, if set.
    #[inline]
    #[must_use]
    pub fn schema(&self) -> Option<&SchemaConfig> {
        self.schema.as_ref()
    }

    /// Return the overridden frontmatter settings, if set.
    #[inline]
    #[must_use]
    pub fn frontmatter(&self) -> Option<&Frontmatter> {
        self.frontmatter.as_ref()
    }

    /// Return the overridden task settings, if set.
    #[inline]
    #[must_use]
    pub fn task(&self) -> Option<&Task> {
        self.task.as_ref()
    }

    /// Return the overridden logging settings, if set.
    #[inline]
    #[must_use]
    pub fn logging(&self) -> Option<&Logging> {
        self.logging.as_ref()
    }
}

fn parse_cache(
    raw: &super::raw::RawCacheConfig,
) -> Result<Option<CacheConfig>, ConfigError> {
    raw.directory
        .as_ref()
        .filter(|value| !value.is_empty())
        .map(|value| CacheDir::try_new(Path::new(value)).map(CacheConfig::new))
        .transpose()
}

fn parse_template(
    raw: &super::raw::RawTemplateConfig,
) -> Result<Option<TemplateConfig>, ConfigError> {
    raw.directory
        .as_ref()
        .filter(|value| !value.is_empty())
        .map(|value| {
            TemplateDir::try_new(Path::new(value)).map(TemplateConfig::new)
        })
        .transpose()
}

fn parse_schema(
    raw: &super::raw::RawSchemaConfig,
) -> Result<Option<SchemaConfig>, ConfigError> {
    let schema_dir = raw
        .directory
        .as_ref()
        .filter(|value| !value.is_empty())
        .map(|value| SchemaDir::try_new(Path::new(value)))
        .transpose()?;

    let property_bank_file = raw
        .property_bank_file
        .as_ref()
        .filter(|value| !value.is_empty())
        .map(|value| PropertyBankFile::try_new(value.clone()))
        .transpose()?;

    if schema_dir.is_none() && property_bank_file.is_none() {
        return Ok(None);
    }

    Ok(Some(SchemaConfig::new(
        schema_dir.unwrap_or_default(),
        property_bank_file.unwrap_or_default(),
    )))
}

/// Metadata for a specific vault.
///
/// This struct holds the identity, root path, and versioning information
/// for a vault. It is stored as part of the [`Config`] aggregate.
///
/// # Invariants
///
/// - **Vault Identity**: Every vault must have a unique UUID v7 identifier.
/// - **Vault Root**: The root path of a vault must be a non-empty directory
///   path.
/// - **Schema Version**: Every vault is associated with a specific schema
///   version.
///
/// # Examples
///
/// ```rust,no_run
/// # use trace_settings::config::vault::{Metadata, VaultId, VaultRoot};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let metadata = Metadata::new(
///     VaultId::new(),
///     VaultRoot::try_new("/vaults/work".into())?,
///     Some("Work".into()),
///     None,
/// )?;
///
/// assert_eq!(metadata.name(), "Work");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct Metadata {
    /// Unique identity of the vault.
    id: VaultId,
    /// Absolute path to the vault root on disk.
    root: VaultRoot,
    /// Human-readable name of the vault.
    name: VaultName,
    /// Version of the vault schema/Traces that created it.
    version: AppVersion,
}

impl Metadata {
    /// Creates vault metadata.
    ///
    /// # Errors
    /// Returns [`ConfigError`] if the vault root validation fails or if the
    /// default schema version cannot be constructed.
    #[inline]
    pub fn new(
        id: VaultId,
        root: VaultRoot,
        name: Option<VaultName>,
        version: Option<AppVersion>,
    ) -> Result<Self, ConfigError> {
        let name = name.unwrap_or_else(|| VaultName::from_root(&root));
        let version =
            match version {
                Some(v) => v,
                None => AppVersion::try_new(env!("CARGO_PKG_VERSION"))
                    .map_err(|e| ConfigError::ValidationFailed {
                        field: "version".into(),
                        message: format!("default version invalid: {e}").into(),
                    })?,
            };

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
        self.name.as_str()
    }

    /// Return the vault version.
    #[inline]
    #[must_use]
    pub fn version(&self) -> &AppVersion {
        &self.version
    }
}

// ----------------------------------------------------------- //
//                     Supporting Types                       //
// ----------------------------------------------------------- //

/// Version number for vault configuration staleness tracking.
///
/// Incremented each time the vault config file changes. Used to determine
/// whether the cached merged config needs rebuilding.
///
/// # Version Sequence
///
/// - Starts at 1 (not 0)
/// - Increments on each vault config file change
/// - Independent of `GlobalVersion` and `Config::Version`
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct VaultVersion(u64);

impl VaultVersion {
    /// Returns the initial version.
    ///
    /// # Examples
    /// ```
    /// use trace_settings::vault::VaultVersion;
    ///
    /// let version = VaultVersion::initial();
    /// assert_eq!(version.value(), 1);
    /// ```
    #[inline]
    #[must_use]
    pub const fn initial() -> Self {
        Self(1)
    }

    /// Returns the numeric version value.
    ///
    /// # Examples
    /// ```
    /// use trace_settings::vault::VaultVersion;
    ///
    /// let version = VaultVersion::initial();
    /// assert_eq!(version.value(), 1);
    /// ```
    #[inline]
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Returns the next version, or an overflow error.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the version number
    /// overflows.
    ///
    /// # Examples
    /// ```
    /// use trace_settings::vault::VaultVersion;
    ///
    /// let version = VaultVersion::initial();
    /// let next = version.next().expect("version increment succeeded");
    /// assert_eq!(next.value(), 2);
    /// ```
    #[inline]
    pub fn next(self) -> Result<Self, ConfigError> {
        self.0.checked_add(1).map(Self).ok_or_else(|| {
            ConfigError::ValidationFailed {
                field: "vault_version".into(),
                message: "vault version overflow".into(),
            }
        })
    }
}

impl std::fmt::Display for VaultVersion {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u64> for VaultVersion {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err(ConfigError::ValidationFailed {
                field: "vault_version".into(),
                message: "vault version cannot be zero".into(),
            });
        }
        Ok(Self(value))
    }
}
/// Version identifier for the Traces application.
///
/// This type ensures that version strings are not empty and represent
/// a valid application version.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct AppVersion(Box<str>);

impl AppVersion {
    /// Creates a new application version.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the version string is
    /// empty.
    #[inline]
    pub fn try_new<T: Into<Box<str>>>(value: T) -> Result<Self, ConfigError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "version".into(),
                message: "version cannot be empty".into(),
            });
        }
        Ok(Self(value))
    }

    /// Return the version as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for AppVersion {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<AppVersion> for String {
    #[inline]
    fn from(version: AppVersion) -> Self {
        version.0.into_string()
    }
}

impl std::fmt::Display for AppVersion {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
/// A validated vault name.
///
/// This type ensures that vault names are not empty and are user-friendly.
/// The name can be explicitly provided or derived from the vault root path.
///
/// # Invariants
///
/// - Must not be empty.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct VaultName(Box<str>);

impl VaultName {
    /// Creates a validated vault name from explicit user input.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the name is empty.
    #[inline]
    pub fn try_new<T: Into<Box<str>>>(value: T) -> Result<Self, ConfigError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ConfigError::ValidationFailed {
                field: "name".into(),
                message: "vault name cannot be empty".into(),
            });
        }
        Ok(Self(value))
    }

    /// Derives a vault name from the last component of the vault root path.
    #[inline]
    #[must_use]
    pub fn from_root(root: &VaultRoot) -> Self {
        let name = root.as_path().file_name().map_or_else(
            || "unnamed".to_owned(),
            |n| n.to_string_lossy().into_owned(),
        );
        Self(name.into_boxed_str())
    }

    /// Return the name as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for VaultName {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<&str> for VaultName {
    #[inline]
    fn from(value: &str) -> Self {
        // Safe because &str is always non-empty when converted from string
        // literal
        Self(value.to_owned().into_boxed_str())
    }
}

impl From<VaultName> for String {
    #[inline]
    fn from(name: VaultName) -> Self {
        name.0.into_string()
    }
}

impl std::fmt::Display for VaultName {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test modules have relaxed rules"
)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::config::logging::{LogLevel, Logging};

    mod version {
        use super::*;

        #[test]
        fn initial_is_one() {
            assert_eq!(VaultVersion::initial().value(), 1);
        }

        #[test]
        fn next_increments_value() {
            let v = VaultVersion::initial();
            let next = v.next().expect("increment should succeed");
            assert_eq!(next.value(), 2);
        }

        #[test]
        fn try_from_accepts_positive() {
            let v = VaultVersion::try_from(42).expect("valid version");
            assert_eq!(v.value(), 42);
        }

        #[test]
        fn try_from_rejects_zero() {
            let result = VaultVersion::try_from(0);
            assert!(result.is_err(), "VaultVersion should reject zero");
        }

        #[test]
        fn display_shows_value() {
            let v = VaultVersion::initial();
            assert_eq!(v.to_string(), "1");
        }
    }

    mod fixtures {
        use super::super::*;

        pub fn vault_root(path: std::path::PathBuf) -> VaultRoot {
            VaultRoot::try_new(path).expect("valid vault root")
        }

        pub fn vault_id() -> VaultId {
            VaultId::new()
        }
    }

    mod constructor {
        use super::*;

        #[test]
        fn vault_new_constructs_with_given_values() {
            let version = VaultVersion::initial();
            let cache = CacheConfig::default();
            let template = TemplateConfig::default();
            let schema = SchemaConfig::default();
            let logging = Logging::new(LogLevel::Debug);
            let vault = Vault::new(
                version,
                Some(logging.clone()),
                Some(cache.clone()),
                Some(template.clone()),
                Some(schema.clone()),
                None,
                None,
            );

            assert_eq!(vault.version(), version);
            assert_eq!(vault.cache(), Some(&cache));
            assert_eq!(vault.template(), Some(&template));
            assert_eq!(vault.schema(), Some(&schema));
            assert_eq!(vault.logging(), Some(&logging));
        }

        #[test]
        fn metadata_new_derives_from_vault_path() {
            // GIVEN: a vault path
            let temp_root = tempfile::tempdir().expect("temp dir should exist");
            let work_dir = temp_root.path().join("work");
            std::fs::create_dir_all(&work_dir)
                .expect("work dir should be created");
            let vault_root = fixtures::vault_root(work_dir);
            let vault_id = fixtures::vault_id();

            // WHEN: building metadata from the path
            let metadata =
                Metadata::new(vault_id, vault_root.clone(), None, None)
                    .unwrap();

            // THEN: version and name defaults are applied
            assert_eq!(
                metadata.version().as_str(),
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
        }

        #[test]
        fn cache_dir_rejects_empty() {
            // GIVEN: empty cache_dir
            use trace_fs::path::RelativeDirPath;
            let result = RelativeDirPath::try_new("");

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

    mod path_overrides {
        use super::*;

        #[test]
        #[expect(deprecated, reason = "testing deprecated accessor behavior")]
        fn returns_cache_template_and_schema_overrides_from_raw_paths() {
            let raw_cache = crate::config::raw::RawCacheConfig {
                directory: Some(".vault-cache".to_owned()),
            };
            let raw_template = crate::config::raw::RawTemplateConfig {
                directory: Some("vault-templates".to_owned()),
            };
            let raw_schema = crate::config::raw::RawSchemaConfig {
                directory: Some("vault-schemas".to_owned()),
                property_bank_file: Some("vault-bank.json".to_owned()),
            };

            let vault = Vault::try_from_components(
                Some(&raw_cache),
                Some(&raw_template),
                Some(&raw_schema),
            )
            .expect("vault path overrides should validate");

            assert_eq!(
                vault
                    .cache()
                    .expect("cache override should exist")
                    .cache_dir()
                    .as_relative_dir()
                    .as_str(),
                ".vault-cache",
                "vault cache override should be retained"
            );
            assert_eq!(
                vault
                    .template()
                    .expect("template override should exist")
                    .template_dir()
                    .as_relative_dir()
                    .as_str(),
                "vault-templates",
                "vault template override should be retained"
            );
            assert_eq!(
                vault
                    .schema()
                    .expect("schema override should exist")
                    .schema_dir()
                    .as_relative_dir()
                    .as_str(),
                "vault-schemas",
                "vault schema override should be retained"
            );
        }
    }

    mod validation {
        #[test]
        fn templates_dir_rejects_absolute() {
            // GIVEN: invalid template dir (absolute)
            use trace_fs::path::RelativeDirPath;
            let result = RelativeDirPath::try_new("/abs");

            // THEN: it fails
            assert!(
                result.is_err(),
                "RelativeDirPath should reject absolute paths"
            );
        }
    }
}
