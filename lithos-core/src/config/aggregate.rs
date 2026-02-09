//! Config aggregate root and versioning.
//!
//! This module provides the [`Config`] aggregate, which represents the
//! fully-merged and validated configuration state for a vault. It also
//! defines [`ConfigVersion`] for tracking configuration history.
//!   vault configurations are missing specific fields.
//! - **Immutability**: Once built, the configuration is immutable and serves as
//!   the "Source of Truth" for the current execution context.
//! - **Validation**: All paths and enums are strictly validated during the
//!   build phase. Construction of a [`Config`] instance is impossible without
//!   satisfying all domain constraints.

#![expect(
    clippy::partial_pub_fields,
    reason = "Aggregate root requires mixed visibility for domain events"
)]
#![expect(
    clippy::exhaustive_structs,
    reason = "rkyv generates exhaustive archived structs"
)]

use std::path::PathBuf;

use super::{
    error::ConfigError,
    events::{ConfigUpdated, Events},
    frontmatter::Frontmatter,
    logging::Logging,
    paths::{Cache, Paths, PropertyBank, Schema, Template},
    raw,
    task::TaskConfig,
    vault::{Metadata, VaultId, VaultRoot},
};

// ============================================================================
// Config Aggregate Root
// ============================================================================

/// Fully-resolved and validated configuration for a vault.
///
/// `Config` represents the "Always Valid" state of a vault's configuration
/// after merging global settings, vault overrides, and system defaults.
/// It is the aggregate root used by the rest of the system for decision
/// making.
///
/// # Precedence Rules
///
/// 1. **Vault Overrides**: Values in the vault-specific `lithos.toml`.
/// 2. **Global Settings**: System-wide settings in the global `lithos.toml`.
/// 3. **System Defaults**: Hardcoded defaults (see [`Default`] implementation).
///
/// # Examples
///
/// ```rust
/// # use std::path::Path;
/// # use lithos_core::config::{aggregate::Config, vault::VaultId, vault::VaultRoot, ingest};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let vault_root_path = Path::new("/tmp/vault");
/// # let vault_id = VaultId::new();
/// # let vault_root = VaultRoot::try_new(vault_root_path.to_path_buf())?;
/// // Ingest and build the aggregate
/// let raw = ingest::build_merged_raw(vault_root_path)?;
/// let config = Config::build(&raw, vault_id, vault_root)?;
///
/// assert!(config.logging.log_level_str() == "info");
/// # Ok(())
/// # }
/// ```
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
#[non_exhaustive]
pub struct Config {
    /// Vault metadata with versioning and naming.
    pub vault_metadata: Metadata,
    /// Merged logging configuration.
    pub logging: Logging,
    /// Merged paths configuration.
    pub paths: Paths,
    /// Merged frontmatter configuration.
    pub frontmatter: Frontmatter,
    /// Merged task configuration.
    pub task: TaskConfig,
    /// Domain events pending emission (not persisted).
    #[serde(skip)]
    #[rkyv(with = rkyv::with::Skip)]
    pending_events: Vec<Events>,
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Self {
            frontmatter: Frontmatter::default(),
            paths: Paths::default(),
            logging: Logging::default(),
            pending_events: vec![],
            task: TaskConfig::default(),
            vault_metadata: Metadata::default(),
        }
    }
}

impl Config {
    /// Build validated Config from Figment-merged raw configuration.
    ///
    /// This is the **primary constructor** for `Config`. It takes a `RawConfig`
    /// that has already been merged across layers (defaults → global → vault)
    /// by `ingest::build_merged_raw()`, validates all fields, and constructs
    /// a fully validated domain configuration.
    ///
    /// # Errors
    /// Returns `ConfigError` if validation fails for any field.
    #[inline]
    pub fn build(
        raw: &raw::RawConfig,
        vault_id: VaultId,
        vault_root: VaultRoot,
    ) -> Result<Self, ConfigError> {
        let vault_metadata = Metadata::new(vault_id, vault_root, None, None)?;

        let fs = &raw.paths;

        let paths = Self::make_resolved_paths(fs)?;

        let frontmatter = raw
            .frontmatter
            .as_ref()
            .map(|x| x.clone().try_into())
            .transpose()?
            .unwrap_or_default();

        let logging = raw
            .logging
            .as_ref()
            .map(|x| x.clone().try_into())
            .transpose()?
            .unwrap_or_default();

        let task = raw
            .task
            .as_ref()
            .map(|x| TaskConfig::from_raw(x.clone()))
            .transpose()?
            .unwrap_or_default();

        let mut config = Self {
            frontmatter,
            paths,
            logging,
            task,
            vault_metadata,
            pending_events: vec![],
        };

        config.add_event(Events::ConfigUpdated(ConfigUpdated::new(
            "merged".to_owned(),
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

    /// Build resolved paths from merged config.
    fn make_resolved_paths(
        fs: &raw::RawPathsConfig,
    ) -> Result<Paths, ConfigError> {
        let cache = Self::opt_path_to_domain(
            &fs.cache_dir,
            Cache::try_new,
            "cache_dir",
        )?
        .unwrap_or_default();

        let schema = Self::opt_path_to_domain(
            &fs.schemas_dir,
            Schema::try_new,
            "schemas_dir",
        )?
        .unwrap_or_default();

        let property_bank = Self::opt_filename(
            &fs.property_bank_filename,
            PropertyBank::try_new,
            "property_bank_filename",
        )?
        .unwrap_or_default();

        let template = Self::opt_path_to_domain(
            &fs.templates_dir,
            Template::try_new,
            "templates_dir",
        )?
        .unwrap_or_default();

        Ok(Paths::new(cache, schema, property_bank, template))
    }

    /// Convert optional path string to domain type.
    #[expect(
        clippy::ref_option,
        reason = "API consistency with Option<T> patterns"
    )]
    fn opt_path_to_domain<T>(
        opt: &Option<String>,
        constructor: fn(PathBuf) -> Result<T, ConfigError>,
        field_name: &str,
    ) -> Result<Option<T>, ConfigError> {
        opt.as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| {
                constructor(PathBuf::from(s)).map_err(|e| {
                    ConfigError::ValidationFailed {
                        field: field_name.to_owned().into(),
                        message: format!("invalid path: {e}").into(),
                    }
                })
            })
            .transpose()
    }

    #[expect(
        clippy::ref_option,
        reason = "API consistency with Option<T> patterns"
    )]
    fn opt_filename<T>(
        opt: &Option<String>,
        constructor: fn(String) -> Result<T, ConfigError>,
        field_name: &str,
    ) -> Result<Option<T>, ConfigError> {
        opt.as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| {
                constructor(s.clone()).map_err(|e| {
                    ConfigError::ValidationFailed {
                        field: field_name.to_owned().into(),
                        message: format!("invalid filename: {e}").into(),
                    }
                })
            })
            .transpose()
    }
}

// ============================================================================
// Versioning & Persistence Types
// ============================================================================

/// Monotonically increasing version number for configuration snapshots.
///
/// This type ensures that configuration versions are positive integers
/// and provides safe incrementing logic.
///
/// # Invariants
///
/// - A `ConfigVersion` must be greater than zero.
///
/// # Examples
///
/// ```rust
/// use lithos_core::config::aggregate::ConfigVersion;
///
/// let version = ConfigVersion::initial();
/// assert_eq!(version.value(), 1);
///
/// let next = version.next().unwrap();
/// assert_eq!(next.value(), 2);
/// ```
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
pub struct ConfigVersion(u64);

impl ConfigVersion {
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

    #[inline]
    /// Return the next version, or an overflow error.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` on overflow.
    pub fn next(self) -> Result<Self, ConfigError> {
        self.0.checked_add(1).map(Self).ok_or_else(|| {
            ConfigError::ValidationFailed {
                field: "config_version".to_owned().into(),
                message: "config version overflow".to_owned().into(),
            }
        })
    }
}

impl TryFrom<u64> for ConfigVersion {
    type Error = ConfigError;

    #[inline]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err(ConfigError::ValidationFailed {
                field: "config_version".to_owned().into(),
                message: "config version cannot be zero".to_owned().into(),
            });
        }
        Ok(Self(value))
    }
}

/// Persisted merged configuration record with version metadata.
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
#[non_exhaustive]
pub struct MergedConfigRecord {
    /// Vault identifier.
    pub vault_id: VaultId,
    /// Merged config version.
    pub version: ConfigVersion,
    /// Unix timestamp for creation.
    pub created_at: i64,
    /// Merged configuration snapshot.
    pub config: Config,
}

/// Active merged config pointer for a vault.
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
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct ActiveMergedConfig {
    /// Vault identifier.
    pub vault_id: VaultId,
    /// Active merged version.
    pub version: ConfigVersion,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    clippy::let_underscore_must_use,
    clippy::let_underscore_untyped,
    clippy::panic_in_result_fn,
    reason = "Test modules have relaxed rules for unwrapping and events"
)]
mod tests {
    mod fixtures {
        use std::path::PathBuf;

        use super::super::*;
        use crate::config::{frontmatter::RawFrontmatter, logging::RawLogging};

        pub fn vault_id() -> VaultId {
            VaultId::new()
        }

        pub fn vault_root(path: &str) -> VaultRoot {
            VaultRoot::try_new(PathBuf::from(path)).expect("vault_root")
        }

        pub fn merged_config_with_sample_overrides() -> Config {
            let raw = raw::RawConfig {
                paths: raw::RawPathsConfig {
                    schemas_dir: Some("schemas".to_owned()),
                    templates_dir: Some("custom_templates".to_owned()),
                    property_bank_filename: Some(
                        "property_bank.json".to_owned(),
                    ),
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
            Config::build(&raw, vault_id(), vault_root("/vault"))
                .expect("Config build should succeed with sample data")
        }

        pub fn merged_config_with_empty_inputs() -> Config {
            let raw = raw::RawConfig::default();
            Config::build(&raw, vault_id(), vault_root("/vault"))
                .expect("Merge with empty values should succeed")
        }

        pub fn config_with_cleared_events() -> Config {
            let mut config = Config::default();
            let _ = config.take_events();
            config
        }
    }

    use super::*;

    mod integrity {
        use super::*;

        #[test]
        fn config_version_initial_is_one() {
            assert_eq!(ConfigVersion::initial().value(), 1);
        }

        #[test]
        fn config_version_next_increments_value()
        -> Result<(), Box<dyn std::error::Error>> {
            let v1 = ConfigVersion::initial();
            let v2 = v1.next()?;
            assert_eq!(v2.value(), 2);
            Ok(())
        }

        #[test]
        fn config_version_try_from_rejects_zero() {
            let result = ConfigVersion::try_from(0);
            result.unwrap_err();
        }

        #[test]
        fn config_version_try_from_accepts_positive()
        -> Result<(), Box<dyn std::error::Error>> {
            let v = ConfigVersion::try_from(42)?;
            assert_eq!(v.value(), 42);
            Ok(())
        }

        #[test]
        fn builds_handles_missing_global_sets_vault_path() {
            let raw = raw::RawConfig::default();
            let config = Config::build(
                &raw,
                fixtures::vault_id(),
                fixtures::vault_root("/vault"),
            )
            .unwrap();
            assert_eq!(
                config.vault_metadata.root().as_path(),
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
            )
            .unwrap();
            assert_eq!(
                config.paths.schema.schemas_dir().as_path(),
                std::path::Path::new("schemas")
            );
        }

        #[test]
        fn add_event_records_pending_event() {
            let mut config = fixtures::config_with_cleared_events();
            config.add_event(Events::ConfigUpdated(ConfigUpdated::new(
                "test".to_owned(),
                0,
            )));
            assert_eq!(config.pending_events().len(), 1);
        }

        #[test]
        fn take_events_clears_pending_events() {
            let mut config = fixtures::config_with_cleared_events();
            config.add_event(Events::ConfigUpdated(ConfigUpdated::new(
                "test".to_owned(),
                0,
            )));
            let _ = config.take_events();
            assert!(config.pending_events().is_empty());
        }
    }

    mod merge {
        use super::*;

        #[test]
        fn defaults_apply_to_cache_dir() {
            let config = fixtures::merged_config_with_empty_inputs();
            assert_eq!(
                config.paths.cache.cache_dir().as_path(),
                std::path::Path::new(".cache")
            );
        }

        #[test]
        fn defaults_apply_to_templates_dir() {
            let config = fixtures::merged_config_with_empty_inputs();
            assert_eq!(
                config.paths.template.templates_dir().as_path(),
                std::path::Path::new("templates")
            );
        }

        #[test]
        fn vault_templates_dir_overrides_global() {
            let merged = fixtures::merged_config_with_sample_overrides();
            assert_eq!(
                merged.paths.template.templates_dir().as_path(),
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
            )
            .unwrap();
            assert_eq!(
                config.paths.schema.schemas_dir().as_path(),
                std::path::Path::new("schemas")
            );
            assert_eq!(
                config.paths.property_bank.as_str(),
                "property_bank.json"
            );
        }

        #[test]
        fn applies_paths_fields_from_raw() {
            let raw = raw::RawConfig {
                paths: raw::RawPathsConfig {
                    cache_dir: Some(".lithos-cache".to_owned()),
                    schemas_dir: Some("my-schemas".to_owned()),
                    property_bank_filename: Some("bank.json".to_owned()),
                    templates_dir: Some("my-templates".to_owned()),
                },
                ..Default::default()
            };
            let config = Config::build(
                &raw,
                fixtures::vault_id(),
                fixtures::vault_root("/vault"),
            )
            .unwrap();
            assert_eq!(
                config.paths.schema.schemas_dir().as_path(),
                std::path::Path::new("my-schemas")
            );
            assert_eq!(
                config.paths.cache.cache_dir().as_path(),
                std::path::Path::new(".lithos-cache")
            );
        }
    }
}
