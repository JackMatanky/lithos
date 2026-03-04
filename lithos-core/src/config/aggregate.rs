//! Config aggregate root and versioning.
//!
//! This module provides the [`Config`] aggregate, which represents the
//! fully-merged and validated configuration state for a vault. It also
//! defines [`Version`] for tracking configuration history.

use tracing::instrument;

use super::{
    error::ConfigError,
    events::{ConfigUpdated, Events},
    frontmatter::Frontmatter,
    logging::Logging,
    paths::{ArchivedPaths, Paths},
    raw,
    task::Task,
    vault::{Metadata, VaultId, VaultRoot},
};

/// Fully-resolved and validated configuration for a vault.
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
#[rkyv(bytecheck(bounds()))]
#[non_exhaustive]
pub struct Config {
    /// Version number for this merged config snapshot.
    version: Version,
    /// Vault metadata with versioning and naming.
    vault_metadata: Metadata,
    /// Merged logging configuration.
    logging: Logging,
    /// Merged paths configuration.
    paths: Paths,
    /// Merged frontmatter configuration.
    frontmatter: Frontmatter,
    /// Merged task configuration.
    task: Task,
    /// Domain events pending emission (not persisted).
    #[serde(skip)]
    #[rkyv(with = rkyv::with::Skip)]
    pending_events: Vec<Events>,
}

impl Config {
    /// Return the vault metadata.
    #[inline]
    #[must_use]
    pub const fn vault_metadata(&self) -> &Metadata {
        &self.vault_metadata
    }

    /// Return the version of this configuration.
    #[inline]
    #[must_use]
    pub const fn version(&self) -> Version {
        self.version
    }

    /// Return the logging configuration.
    #[inline]
    #[must_use]
    pub const fn logging(&self) -> &Logging {
        &self.logging
    }

    /// Return the paths configuration.
    #[inline]
    #[must_use]
    pub const fn paths(&self) -> &Paths {
        &self.paths
    }

    /// Return the frontmatter configuration.
    #[inline]
    #[must_use]
    pub const fn frontmatter(&self) -> &Frontmatter {
        &self.frontmatter
    }

    /// Return the task configuration.
    #[inline]
    #[must_use]
    pub const fn task(&self) -> &Task {
        &self.task
    }

    /// Build validated Config from Figment-merged raw configuration.
    ///
    /// # Errors
    /// Returns [`ConfigError`] if the raw configuration fails validation rules.
    #[inline]
    #[instrument(
        skip(raw, vault_root),
        level = "debug",
        fields(operation = "build_config", vault_id = %vault_id, version = %version)
    )]
    pub fn build(
        raw: &raw::RawConfig,
        vault_id: VaultId,
        vault_root: VaultRoot,
        version: Version,
    ) -> Result<Self, ConfigError> {
        let vault_metadata = Metadata::new(vault_id, vault_root, None, None)?;

        let logging = raw
            .logging
            .as_ref()
            .map(|x| x.clone().try_into())
            .transpose()?
            .unwrap_or_default();

        let paths = (&raw.paths).try_into()?;

        let frontmatter = raw
            .frontmatter
            .as_ref()
            .map(|x| x.clone().try_into())
            .transpose()?
            .unwrap_or_default();

        let task = raw
            .task
            .as_ref()
            .map(|x| Task::try_from_raw(x.clone()))
            .transpose()?
            .unwrap_or_default();

        let mut config = Self {
            version,
            frontmatter,
            paths,
            logging,
            task,
            vault_metadata,
            pending_events: vec![],
        };

        config.add_event(Events::ConfigUpdated(ConfigUpdated::new(
            "merged",
            chrono::Utc::now().timestamp(),
        )));

        Ok(config)
    }

    /// Returns a reference to pending domain events.
    #[inline]
    #[must_use]
    pub fn pending_events(&self) -> &[Events] {
        &self.pending_events
    }

    /// Returns and clears pending domain events.
    #[inline]
    #[must_use]
    pub fn take_events(&mut self) -> Vec<Events> {
        std::mem::take(&mut self.pending_events)
    }

    /// Adds a domain event to the pending events collection.
    #[inline]
    fn add_event(&mut self, event: Events) {
        self.pending_events.push(event);
    }
}

impl ArchivedConfig {
    /// Return the paths configuration.
    #[inline]
    #[must_use]
    pub const fn paths(&self) -> &ArchivedPaths {
        &self.paths
    }
}

/// Monotonically increasing version number for configuration snapshots.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(bytecheck(bounds()))]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Version(u64);

impl Version {
    #[inline]
    #[must_use]
    /// Return the initial version value.
    pub const fn initial() -> Self {
        Self(1)
    }

    #[inline]
    #[must_use]
    /// Return the numeric version value.
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Return the next version, or an overflow error.
    ///
    /// # Errors
    /// Returns [`ConfigError::ValidationFailed`] if the version number
    /// overflows.
    #[inline]
    pub fn next(self) -> Result<Self, ConfigError> {
        self.0.checked_add(1).map(Self).ok_or_else(|| {
            ConfigError::ValidationFailed {
                field: "config_version".into(),
                message: "config version overflow".into(),
            }
        })
    }
}

impl std::fmt::Display for Version {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u64> for Version {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err(ConfigError::ValidationFailed {
                field: "config_version".into(),
                message: "config version cannot be zero".into(),
            });
        }
        Ok(Self(value))
    }
}

/// Unix timestamp in seconds since epoch.
///
/// Used for tracking file modification times and metadata recording times.
/// Supports config staleness detection by comparing file timestamps.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[serde(transparent)]
#[non_exhaustive]
pub struct Timestamp(u64);

impl Timestamp {
    /// Returns the current UTC timestamp in seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::config::aggregate::Timestamp;
    ///
    /// let now = Timestamp::now();
    /// assert!(now.as_secs() > 0);
    /// ```
    #[inline]
    #[must_use]
    pub fn now() -> Self {
        let secs =
            chrono::Utc::now().timestamp().max(0).try_into().unwrap_or(0);
        Self(secs)
    }

    /// Creates a timestamp from seconds since epoch.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::config::aggregate::Timestamp;
    ///
    /// let ts = Timestamp::from_secs(1000);
    /// assert_eq!(ts.as_secs(), 1000);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_secs(secs: u64) -> Self {
        Self(secs)
    }

    /// Returns the timestamp as seconds since epoch.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::config::aggregate::Timestamp;
    ///
    /// let ts = Timestamp::from_secs(1000);
    /// assert_eq!(ts.as_secs(), 1000);
    /// ```
    #[inline]
    #[must_use]
    pub const fn as_secs(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for Timestamp {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {

    mod fixtures {
        use std::path::PathBuf;

        use super::super::*;
        use crate::config::{
            frontmatter::RawFrontmatter,
            logging::RawLogging,
            vault::{AppVersion, VaultName},
        };

        pub fn vault_id() -> VaultId {
            VaultId::new()
        }

        pub fn vault_root(path: &str) -> VaultRoot {
            VaultRoot::try_new(PathBuf::from(path)).expect("vault_root")
        }

        /// Create a Config with test values. Only available in tests.
        pub fn test_config() -> Config {
            let test_root = VaultRoot::try_new("/test-vault".into())
                .expect("test vault root must be valid");
            let test_name = VaultName::from_root(&test_root);
            let test_version = AppVersion::try_new(env!("CARGO_PKG_VERSION"))
                .expect("package version is non-empty");

            Config {
                version: Version::initial(),
                vault_metadata: Metadata::new(
                    VaultId::new(),
                    test_root,
                    Some(test_name),
                    Some(test_version),
                )
                .expect("test metadata must be valid"),
                logging: Logging::default(),
                paths: Paths::default(),
                frontmatter: Frontmatter::default(),
                task: Task::default(),
                pending_events: vec![],
            }
        }

        pub fn merged_config_with_sample_overrides() -> Config {
            let raw = raw::RawConfig {
                paths: raw::RawPathsConfig {
                    schemas_dir: Some("schemas".to_owned()),
                    templates_dir: Some("custom_templates".to_owned()),
                    property_bank_file: Some("property_bank.json".to_owned()),
                    cache_dir: Some(".lithos".to_owned()),
                },
                frontmatter: Some(RawFrontmatter {
                    alias_key: Some("aliases".to_owned()),
                    date_created_key: Some("created".to_owned()),
                    date_modified_key: Some("modified".to_owned()),
                    file_class_key: Some("type".to_owned()),
                    title_key: Some("title".to_owned()),
                }),
                logging: Some(RawLogging {
                    log_level: Some("debug".to_owned()),
                }),
                task: None,
                trusted_vaults: None,
            };
            Config::build(
                &raw,
                vault_id(),
                vault_root("/vault"),
                Version::initial(),
            )
            .expect("Config build should succeed with sample data")
        }

        pub fn merged_config_with_empty_inputs() -> Config {
            let raw = raw::RawConfig::default();
            Config::build(
                &raw,
                vault_id(),
                vault_root("/vault"),
                Version::initial(),
            )
            .expect("Merge with empty values should succeed")
        }

        pub fn config_with_cleared_events() -> Config {
            let mut config = test_config();
            let _events: Vec<Events> = config.take_events();
            config
        }
    }

    use super::*;

    mod integrity {
        use super::*;

        #[test]
        fn version_initial_is_one() {
            assert_eq!(Version::initial().value(), 1);
        }

        #[test]
        fn version_next_increments_value() {
            let v1 = Version::initial();
            let v2 = v1.next().unwrap();
            assert_eq!(v2.value(), 2);
        }

        #[test]
        fn version_try_from_rejects_zero() {
            let result = Version::try_from(0);
            result.unwrap_err();
        }

        #[test]
        fn version_try_from_accepts_positive() {
            let v = Version::try_from(42).unwrap();
            assert_eq!(v.value(), 42);
        }

        #[test]
        fn builds_handles_missing_global_sets_vault_path() {
            let raw = raw::RawConfig::default();
            let config = Config::build(
                &raw,
                fixtures::vault_id(),
                fixtures::vault_root("/vault"),
                Version::initial(),
            )
            .unwrap();
            assert_eq!(
                config.vault_metadata().root().as_path(),
                std::path::Path::new("/vault")
            );
        }

        #[test]
        fn build_handles_missing_global_applies_global_defaults() {
            let raw = raw::RawConfig::default();
            let config = Config::build(
                &raw,
                fixtures::vault_id(),
                fixtures::vault_root("/vault"),
                Version::initial(),
            )
            .unwrap();
            assert_eq!(
                config.paths().schema.schemas_dir().as_path(),
                std::path::Path::new("schemas")
            );
        }

        #[test]
        fn add_event_records_pending_event() {
            let mut config = fixtures::config_with_cleared_events();
            config.add_event(Events::ConfigUpdated(ConfigUpdated::new(
                "test", 0,
            )));
            assert_eq!(config.pending_events().len(), 1);
        }

        #[test]
        fn take_events_clears_pending_events() {
            let mut config = fixtures::config_with_cleared_events();
            config.add_event(Events::ConfigUpdated(ConfigUpdated::new(
                "test", 0,
            )));
            let _events: Vec<Events> = config.take_events();
            assert!(config.pending_events().is_empty());
        }
    }

    mod merge {
        use super::*;

        #[test]
        fn defaults_apply_to_cache_dir() {
            let config = fixtures::merged_config_with_empty_inputs();
            assert_eq!(
                config.paths().cache.cache_dir().as_path(),
                std::path::Path::new(".cache")
            );
        }

        #[test]
        fn defaults_apply_to_templates_dir() {
            let config = fixtures::merged_config_with_empty_inputs();
            assert_eq!(
                config.paths().template.templates_dir().as_path(),
                std::path::Path::new("templates")
            );
        }

        #[test]
        fn vault_templates_dir_overrides_global() {
            let merged = fixtures::merged_config_with_sample_overrides();
            assert_eq!(
                merged.paths().template.templates_dir().as_path(),
                std::path::Path::new("custom_templates")
            );
        }
    }

    mod build_tests {
        use super::*;

        #[test]
        fn builds_config_from_empty_raw() {
            let raw = raw::RawConfig::default();
            let config = Config::build(
                &raw,
                fixtures::vault_id(),
                fixtures::vault_root("/vault"),
                Version::initial(),
            )
            .unwrap();
            assert_eq!(
                config.paths().schema.schemas_dir().as_path(),
                std::path::Path::new("schemas")
            );
            assert_eq!(
                config.paths().property_bank.as_str(),
                "property_bank.json"
            );
        }

        #[test]
        fn applies_paths_fields_from_raw() {
            let raw = raw::RawConfig {
                paths: raw::RawPathsConfig {
                    cache_dir: Some(".lithos-cache".to_owned()),
                    schemas_dir: Some("my-schemas".to_owned()),
                    property_bank_file: Some("bank.json".to_owned()),
                    templates_dir: Some("my-templates".to_owned()),
                },
                ..Default::default()
            };
            let config = Config::build(
                &raw,
                fixtures::vault_id(),
                fixtures::vault_root("/vault"),
                Version::initial(),
            )
            .unwrap();
            assert_eq!(
                config.paths().schema.schemas_dir().as_path(),
                std::path::Path::new("my-schemas")
            );
            assert_eq!(
                config.paths().cache.cache_dir().as_path(),
                std::path::Path::new(".lithos-cache")
            );
        }
    }
}
