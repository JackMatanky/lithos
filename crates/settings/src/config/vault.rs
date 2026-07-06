//! Vault-specific overrides.
//!
//! This module defines the [`LocalConfig`] configuration, which contains
//! vault-specific settings and overrides for global defaults. It also
//! manages `VaultId` and `VaultRoot`.

use std::path::{Path, PathBuf};

use rkyv::{Archive, Deserialize, Serialize};
use traces_fs::{DirPath, FilePath};
use traces_utils::UuidV7;

use super::{
    cache::{CacheConfig, CacheDir},
    error::ConfigError,
    frontmatter::Frontmatter,
    logging::Logging,
    raw::RawConfig,
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
/// `LocalConfig` contains settings that are specific to a single vault and
/// override the global defaults. It covers paths, frontmatter, and logging
/// settings, and carries the vault-root [`DirPath`], the config-file
/// [`FilePath`], and a derived vault `name`.
///
/// Construct one from a [`RawConfig`] and its backing paths via
/// [`TryFrom<(RawConfig, DirPath, FilePath)>`].
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct LocalConfig {
    /// Version number for this vault config.
    version: VaultVersion,
    /// Vault root directory this config applies to.
    base: DirPath,
    /// Path to the config file this config was built from.
    path: FilePath,
    /// Human-readable vault name derived from the base directory basename.
    name: Box<str>,
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

impl LocalConfig {
    /// Create vault-specific configuration.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Domain constructor with all optional override groups"
    )]
    pub fn new(
        version: VaultVersion,
        base: DirPath,
        path: FilePath,
        name: Box<str>,
        logging: Option<Logging>,
        cache: Option<CacheConfig>,
        template: Option<TemplateConfig>,
        schema: Option<SchemaConfig>,
        frontmatter: Option<Frontmatter>,
        task: Option<Task>,
    ) -> Self {
        Self {
            version,
            base,
            path,
            name,
            logging,
            cache,
            template,
            schema,
            frontmatter,
            task,
        }
    }

    /// Return the version of this vault config.
    #[inline]
    #[must_use]
    pub const fn version(&self) -> VaultVersion {
        self.version
    }

    /// Return the vault root directory this config applies to.
    #[inline]
    #[must_use]
    pub const fn base(&self) -> &DirPath {
        &self.base
    }

    /// Return the path of the config file this config was built from.
    #[inline]
    #[must_use]
    pub const fn path(&self) -> &FilePath {
        &self.path
    }

    /// Return the vault name derived from the base directory basename.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
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

impl TryFrom<(RawConfig, DirPath, FilePath)> for LocalConfig {
    type Error = ConfigError;

    /// Builds a validated [`LocalConfig`] from a raw config, its vault-root
    /// base directory, and its config-file path.
    ///
    /// The tuple carries the already-validated [`DirPath`] (vault root) and
    /// [`FilePath`] (config file); filesystem I/O happens at the caller, not
    /// here. The vault `name` is derived from the base directory basename,
    /// falling back to `"unnamed"` when the base has no final component.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] with `field: "trusted_vaults"`
    /// when the forbidden `trusted_vaults` field is present (it is
    /// global-scoped), or when any contained field fails validation.
    #[inline]
    fn try_from(
        (raw, base, path): (RawConfig, DirPath, FilePath),
    ) -> Result<Self, Self::Error> {
        if raw.trusted_vaults.is_some() {
            return Err(ConfigError::ValidationFailed {
                field: "trusted_vaults".into(),
                message: format!(
                    "trusted_vaults is forbidden in local config ({}); it is \
                     global-scoped",
                    path.as_path().display()
                )
                .into(),
            });
        }

        let name = name_from_base(&base);
        let logging = raw.logging.map(Logging::try_from).transpose()?;
        let cache = raw.cache.as_ref().map(parse_cache).transpose()?.flatten();
        let template =
            raw.template.as_ref().map(parse_template).transpose()?.flatten();
        let schema =
            raw.schema.as_ref().map(parse_schema).transpose()?.flatten();
        let frontmatter =
            raw.frontmatter.map(Frontmatter::try_from).transpose()?;
        let task = raw.task.map(Task::try_from).transpose()?;

        Ok(Self {
            version: VaultVersion::initial(),
            base,
            path,
            name,
            logging,
            cache,
            template,
            schema,
            frontmatter,
            task,
        })
    }
}

/// Derives a vault name from the last component of the vault root path.
///
/// Falls back to `"unnamed"` when the path has no final component.
fn name_from_base(base: &DirPath) -> Box<str> {
    base.as_path()
        .file_name()
        .map_or_else(
            || "unnamed".to_owned(),
            |n| n.to_string_lossy().into_owned(),
        )
        .into_boxed_str()
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
/// - Independent of `GlobalVersion` and `Version`
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
    /// use traces_settings::vault::VaultVersion;
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
    /// use traces_settings::vault::VaultVersion;
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
    /// use traces_settings::vault::VaultVersion;
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
/// Test-only fixtures for constructing [`LocalConfig`] values.
#[cfg(test)]
pub(crate) mod fixtures {
    use tempfile::{NamedTempFile, TempDir};
    use traces_fs::{DirPath, FilePath};

    use super::LocalConfig;

    /// Builds a default [`LocalConfig`] backed by fresh temp paths.
    ///
    /// Returns the [`TempDir`] and [`NamedTempFile`] guards so the backing
    /// base directory and config file outlive the returned config for the
    /// duration of the test.
    pub(crate) fn local_config() -> (TempDir, NamedTempFile, LocalConfig) {
        let base_dir = TempDir::new().expect("temp dir created");
        let base = DirPath::try_new(base_dir.path().to_path_buf())
            .expect("temp dir is a valid DirPath");
        let file = NamedTempFile::new().expect("temp file created");
        let path = FilePath::try_new(file.path().to_path_buf())
            .expect("temp file is a valid FilePath");
        let config = LocalConfig::try_from((
            crate::config::raw::RawConfig::default(),
            base,
            path,
        ))
        .expect("default raw config converts");
        (base_dir, file, config)
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
        use pretty_assertions::assert_eq;

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

    mod constructor {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn vault_new_constructs_with_given_values() {
            let version = VaultVersion::initial();
            let base_dir = tempfile::tempdir().expect("temp dir should exist");
            let base =
                traces_fs::DirPath::try_new(base_dir.path().to_path_buf())
                    .expect("valid base dir");
            let file =
                tempfile::NamedTempFile::new().expect("temp file created");
            let path = traces_fs::FilePath::try_new(file.path().to_path_buf())
                .expect("valid config path");
            let cache = CacheConfig::default();
            let template = TemplateConfig::default();
            let schema = SchemaConfig::default();
            let logging = Logging::new(LogLevel::Debug);
            let vault = LocalConfig::new(
                version,
                base.clone(),
                path.clone(),
                "vault".into(),
                Some(logging.clone()),
                Some(cache.clone()),
                Some(template.clone()),
                Some(schema.clone()),
                None,
                None,
            );

            assert_eq!(vault.version(), version);
            assert_eq!(vault.base().as_path(), base.as_path());
            assert_eq!(vault.path().as_path(), path.as_path());
            assert_eq!(vault.name(), "vault");
            assert_eq!(vault.cache(), Some(&cache));
            assert_eq!(vault.template(), Some(&template));
            assert_eq!(vault.schema(), Some(&schema));
            assert_eq!(vault.logging(), Some(&logging));
        }

        #[test]
        fn cache_dir_rejects_empty() {
            // GIVEN: empty cache_dir
            use traces_fs::path::RelativeDirPath;
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

    mod validation {
        #[test]
        fn templates_dir_rejects_absolute() {
            // GIVEN: invalid template dir (absolute)
            use traces_fs::path::RelativeDirPath;
            let result = RelativeDirPath::try_new("/abs");

            // THEN: it fails
            assert!(
                result.is_err(),
                "RelativeDirPath should reject absolute paths"
            );
        }
    }

    mod try_from_raw {
        use pretty_assertions::assert_eq;
        use tempfile::{NamedTempFile, TempDir};
        use traces_fs::{DirPath, FilePath};

        use super::*;
        use crate::config::raw::{
            RawCacheConfig, RawConfig, RawLogging, RawSchemaConfig,
            RawTemplateConfig, RawTrustedVaults,
        };

        /// A base dir and config file backed by fresh temp paths.
        ///
        /// Returns the guards so the backing paths outlive the returned
        /// [`DirPath`]/[`FilePath`] for the test's duration.
        fn base_and_path() -> (TempDir, NamedTempFile, DirPath, FilePath) {
            let base_dir = TempDir::new().expect("temp dir created");
            let base = DirPath::try_new(base_dir.path().to_path_buf())
                .expect("temp dir is a valid DirPath");
            let file = NamedTempFile::new().expect("temp file created");
            let path = FilePath::try_new(file.path().to_path_buf())
                .expect("temp file is a valid FilePath");
            (base_dir, file, base, path)
        }

        #[test]
        #[expect(deprecated, reason = "testing deprecated accessor behavior")]
        fn accepts_valid_raw_and_preserves_overrides() {
            let (_base_guard, _file_guard, base, path) = base_and_path();
            let raw = RawConfig {
                logging: Some(RawLogging {
                    log_level: Some("debug".to_owned()),
                }),
                cache: Some(RawCacheConfig {
                    directory: Some(".vault-cache".to_owned()),
                }),
                template: Some(RawTemplateConfig {
                    directory: Some("vault-templates".to_owned()),
                }),
                schema: Some(RawSchemaConfig {
                    directory: Some("vault-schemas".to_owned()),
                    property_bank_file: Some("vault-bank.json".to_owned()),
                }),
                ..RawConfig::default()
            };

            let local = LocalConfig::try_from((raw, base, path))
                .expect("valid raw config should convert");

            assert_eq!(
                local.logging().expect("logging override preserved").level(),
                LogLevel::Debug,
                "logging override should be preserved from raw"
            );
            assert_eq!(
                local
                    .cache()
                    .expect("cache override preserved")
                    .cache_dir()
                    .as_relative_dir()
                    .as_str(),
                ".vault-cache",
                "cache override should be preserved from raw"
            );
            assert_eq!(
                local
                    .template()
                    .expect("template override preserved")
                    .template_dir()
                    .as_relative_dir()
                    .as_str(),
                "vault-templates",
                "template override should be preserved from raw"
            );
            assert_eq!(
                local
                    .schema()
                    .expect("schema override preserved")
                    .schema_dir()
                    .as_relative_dir()
                    .as_str(),
                "vault-schemas",
                "schema override should be preserved from raw"
            );
            assert_eq!(
                local
                    .schema()
                    .expect("schema override preserved")
                    .property_bank_file()
                    .as_str(),
                "vault-bank.json",
                "property_bank override should be preserved from raw"
            );
        }

        #[test]
        fn rejects_trusted_vaults_as_forbidden_field() {
            let (_base_guard, _file_guard, base, path) = base_and_path();
            let raw = RawConfig {
                trusted_vaults: Some(RawTrustedVaults::List(vec![
                    "/vaults/alpha".to_owned(),
                ])),
                ..RawConfig::default()
            };

            let expected_path = path.as_path().display().to_string();
            let error = LocalConfig::try_from((raw, base, path))
                .expect_err("trusted_vaults must be rejected in local config");

            assert!(
                matches!(
                    &error,
                    ConfigError::ValidationFailed { field, message }
                        if &**field == "trusted_vaults"
                            && message.contains(&expected_path)
                ),
                "expected ValidationFailed on 'trusted_vaults' including \
                 source path {expected_path:?}, got {error:?}"
            );
        }

        #[test]
        fn derives_name_from_base_basename() {
            let temp_root = TempDir::new().expect("temp dir created");
            let vault_dir = temp_root.path().join("my-vault");
            std::fs::create_dir_all(&vault_dir)
                .expect("vault dir should be created");
            let base = DirPath::try_new(vault_dir).expect("valid base DirPath");
            let file = NamedTempFile::new().expect("temp file created");
            let path = FilePath::try_new(file.path().to_path_buf())
                .expect("valid config FilePath");

            let local =
                LocalConfig::try_from((RawConfig::default(), base, path))
                    .expect("raw config should convert");

            assert_eq!(
                local.name(),
                "my-vault",
                "name should be derived from the base directory basename"
            );
        }

        #[test]
        fn carries_base_path_and_name() {
            let (_base_guard, _file_guard, base, path) = base_and_path();

            let local = LocalConfig::try_from((
                RawConfig::default(),
                base.clone(),
                path.clone(),
            ))
            .expect("raw config should convert");

            assert_eq!(
                local.base().as_path(),
                base.as_path(),
                "base should be carried onto the config"
            );
            assert_eq!(
                local.path().as_path(),
                path.as_path(),
                "path should be carried onto the config"
            );
            assert!(
                !local.name().is_empty(),
                "derived name should be non-empty"
            );
        }
    }
}
