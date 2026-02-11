//! Config aggregate root and versioning.
//!
//! This module provides the [`Config`] aggregate, which represents the
//! fully-merged and validated configuration state for a vault. It also
//! defines [`Version`] for tracking configuration history.
//!
//! # Invariants
//!
//! - **Always Valid**: Once constructed, `Config` is guaranteed to be
//!   internally consistent and valid for use throughout the system.
//! - **Immutability**: Once built, the configuration is immutable and serves as
//!   the "Source of Truth" for the current execution context.
//! - **Validation**: All paths and enums are strictly validated during the
//!   build phase. Construction of a [`Config`] instance is impossible without
//!   satisfying all domain constraints.
//! - **Layered Configuration**: The aggregate enforces a clear precedence
//!   hierarchy for settings (vault overrides → global → system defaults).

#![expect(
    clippy::partial_pub_fields,
    reason = "Aggregate root requires mixed visibility for domain events"
)]

use tracing::instrument;

use super::{
    error::ConfigError,
    events::{ConfigUpdated, Events},
    frontmatter::Frontmatter,
    logging::Logging,
    paths::Paths,
    raw,
    task::Task,
    vault::{Metadata, VaultId, VaultRoot},
};

// ----------------------------------------------------------- //
//                    Config Aggregate Root                    //
// ----------------------------------------------------------- //

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
/// # use lithos_core::config::{
/// #     aggregate::Config,
/// #     vault::{VaultId, VaultRoot},
/// #     ingest
/// # };
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let vault_root_path = Path::new("/tmp/vault");
/// # let vault_id = VaultId::new();
/// # let vault_root = VaultRoot::try_new(vault_root_path.to_path_buf())?;
/// // Ingest and build the aggregate
/// let raw = ingest::build_merged_raw(vault_root_path)?;
/// let config = Config::build(&raw, vault_id, vault_root)?;
///
/// // Access nested configuration values via sub-structs
/// assert!(config.logging.log_level_str() == "info");
/// assert_eq!(config.paths.schema.schemas_dir().as_path(), Path::new("schemas"));
/// assert_eq!(config.frontmatter.title_key().as_str(), "title");
/// assert!(config.task.enabled());
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
    pub task: Task,
    /// Domain events pending emission (not persisted).
    #[serde(skip)]
    #[rkyv(with = rkyv::with::Skip)]
    pending_events: Vec<Events>,
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
    #[instrument(skip(raw, vault_root), level = "debug", fields(operation = "build_config", vault_id = %vault_id))]
    pub fn build(
        raw: &raw::RawConfig,
        vault_id: VaultId,
        vault_root: VaultRoot,
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
            .map(|x| Task::from_raw(x.clone()))
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
}

// ----------------------------------------------------------- //
//               Versioning & Persistence Types                //
// ----------------------------------------------------------- //

/// Monotonically increasing version number for configuration snapshots.
///
/// This type ensures that configuration versions are positive integers
/// and provides safe incrementing logic.
///
/// # Invariants
///
/// - A `Version` must be greater than zero.
///
/// # Examples
///
/// ```rust
/// use lithos_core::config::aggregate::Version;
///
/// let version = Version::initial();
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
                field: "config_version".to_owned().into(),
                message: "config version cannot be zero".to_owned().into(),
            });
        }
        Ok(Self(value))
    }
}

// Persisted configuration record with version metadata.
// DEPRECATED: Not used in current CQRS design. Retained for potential future
// use. #[derive(
//     Debug,
//     Clone,
//     PartialEq,
//     serde::Serialize,
//     serde::Deserialize,
//     rkyv::Archive,
//     rkyv::Serialize,
//     rkyv::Deserialize,
// )]
// #[non_exhaustive]
// pub struct Record {
//     /// Vault identifier.
//     pub vault_id: VaultId,
//     /// Merged config version.
//     pub version: Version,
//     /// Unix timestamp for creation.
//     pub created_at: i64,
//     /// Merged configuration snapshot.
//     pub config: Config,
// }

// Active config pointer for a vault.
// DEPRECATED: Not used in current CQRS design. Retained for potential future
// use. #[derive(
//     Debug,
//     Clone,
//     PartialEq,
//     serde::Serialize,
//     serde::Deserialize,
//     rkyv::Archive,
//     rkyv::Serialize,
//     rkyv::Deserialize,
// )]
// #[rkyv(derive(Debug))]
// #[non_exhaustive]
// pub struct ActiveConfig {
//     /// Vault identifier.
//     pub vault_id: VaultId,
//     /// Active merged version.
//     pub version: Version,
// }

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

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
            Config::build(&raw, vault_id(), vault_root("/vault"))
                .expect("Config build should succeed with sample data")
        }

        pub fn merged_config_with_empty_inputs() -> Config {
            let raw = raw::RawConfig::default();
            Config::build(&raw, vault_id(), vault_root("/vault"))
                .expect("Merge with empty values should succeed")
        }

        pub fn config_with_cleared_events() -> Config {
            let mut config = test_config();
            let _ = config.take_events();
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
        fn version_next_increments_value()
        -> Result<(), Box<dyn std::error::Error>> {
            let v1 = Version::initial();
            let v2 = v1.next()?;
            assert_eq!(v2.value(), 2);
            Ok(())
        }

        #[test]
        fn version_try_from_rejects_zero() {
            let result = Version::try_from(0);
            result.unwrap_err();
        }

        #[test]
        fn version_try_from_accepts_positive()
        -> Result<(), Box<dyn std::error::Error>> {
            let v = Version::try_from(42)?;
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
                    property_bank_file: Some("bank.json".to_owned()),
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
