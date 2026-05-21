//! Config aggregate root and versioning.
//!
//! This module provides the [`Config`] aggregate, which represents the
//! fully-merged and validated configuration state for a vault. It also
//! defines [`Version`] for tracking configuration history.

use std::{collections::HashMap, sync::Arc};

use rkyv::with::Skip;

use super::{
    error::ConfigError,
    events::{ConfigUpdated, Events},
    frontmatter::{Frontmatter, FrontmatterConfigSpec},
    logging::Logging,
    paths::{ArchivedPaths, Paths},
    task::{Task, TaskConfigSpec, TemporalSlot},
    vault::Metadata,
};

/// Fully-resolved and validated configuration for a vault.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
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
    #[rkyv(with = Skip)]
    pending_events: Vec<Events>,
}

impl Config {
    /// Construct a validated Config aggregate from validated parts.
    #[expect(
        clippy::too_many_arguments,
        reason = "Constructor collects validated domain components"
    )]
    #[inline]
    #[must_use]
    pub(crate) fn new(
        version: Version,
        vault_metadata: Metadata,
        logging: Logging,
        paths: Paths,
        frontmatter: Frontmatter,
        task: Task,
    ) -> Self {
        let mut config = Self {
            version,
            vault_metadata,
            logging,
            paths,
            frontmatter,
            task,
            pending_events: vec![],
        };

        config.add_event(Events::ConfigUpdated(ConfigUpdated::new(
            "merged",
            chrono::Utc::now().timestamp(),
        )));

        config
    }

    /// Return the vault metadata.
    #[inline]
    #[must_use]
    pub const fn vault_metadata(&self) -> &Metadata {
        &self.vault_metadata
    }

    /// Return the version of this configuration.
    #[inline]
    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.version
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

    /// Maps the current configuration into a lightweight contract for
    /// frontmatter extraction.
    #[inline]
    #[must_use]
    pub fn to_frontmatter_spec(&self) -> FrontmatterConfigSpec {
        FrontmatterConfigSpec {
            title_key: self.frontmatter.title().as_str().into(),
            alias_key: self.frontmatter.alias().as_str().into(),
            tags_key: self.frontmatter.tags().as_str().into(),
            file_class_key: self.frontmatter.file_class().as_str().into(),
            date_created_key: self.frontmatter.date_created().as_str().into(),
            date_modified_key: self.frontmatter.date_modified().as_str().into(),
        }
    }

    /// Maps the current configuration into a lightweight contract for task
    /// scanning and promotion.
    #[inline]
    #[must_use]
    pub fn to_task_spec(&self) -> TaskConfigSpec {
        let mut emoji_markers = Vec::new();
        let mut temporal_specs = HashMap::new();

        let date_slots = [
            (self.task.created(), TemporalSlot::Created),
            (self.task.due(), TemporalSlot::Due),
            (self.task.reminder(), TemporalSlot::Reminder),
            (self.task.completed(), TemporalSlot::Completed),
            (self.task.start(), TemporalSlot::Start),
            (self.task.scheduled(), TemporalSlot::Scheduled),
        ];

        for (opt_spec, slot) in date_slots {
            if let Some(spec) = opt_spec {
                let keyword = spec.keyword().as_str().into();
                let emoji = spec.emoji();
                if let Some(e) = emoji {
                    emoji_markers.push(e);
                }
                temporal_specs
                    .insert(keyword, (slot, Arc::new(spec.clone()), emoji));
            }
        }

        // Include default Obsidian task emojis if emoji support is enabled
        if self.task.use_emoji() {
            let defaults = [
                '\u{2795}',  // ➕ created
                '\u{1f4c5}', // 📅 due
                '\u{2705}',  // ✅ completed
                '\u{23f3}',  // ⏳ scheduled
                '\u{1f6eb}', // 🛫 start
                '\u{274c}',  // ❌ cancelled
                '\u{23f0}',  // ⏰ reminder
            ];
            for emoji in defaults {
                if !emoji_markers.contains(&emoji) {
                    emoji_markers.push(emoji);
                }
            }
        }

        let promotion_tags =
            self.task.tags().iter().map(|t| t.as_str().into()).collect();

        let mut status_mappings = HashMap::new();
        for (_, spec) in self.task.status() {
            status_mappings
                .insert(spec.symbol().value(), spec.name().as_str().into());
        }

        TaskConfigSpec {
            enabled: self.task.enabled(),
            use_emoji: self.task.use_emoji(),
            emoji_markers: emoji_markers.into_boxed_slice(),
            promotion_tags,
            status_mappings,
            temporal_specs,
            field_specs: self.task.fields().clone(),
        }
    }

    /// Builds a schema configuration specification for the discovery engine.
    ///
    /// This method constructs the full property bank path by joining the
    /// schemas directory with the property bank filename from the
    /// configuration.
    ///
    /// # Panics
    ///
    /// Never panics. The `property_bank_path()` method joins validated relative
    /// paths, and the result is guaranteed to be a valid `RelativePath`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use lithos_core::config::aggregate::Config;
    /// # fn example(config: &Config) {
    /// let spec = config.to_schema_spec();
    /// // spec.root() returns vault root
    /// // spec.directory() returns absolute path to schemas directory
    /// // spec.property_bank() returns absolute path to property bank file
    /// # }
    /// ```
    #[inline]
    #[must_use]
    #[allow(
        clippy::expect_used,
        reason = "property_bank_path() joins validated relative paths"
    )]
    pub fn to_schema_spec(&self) -> super::paths::SchemaConfigSpec {
        use super::paths::SchemaConfigSpec;
        use crate::fs::{DirPath, RelativePath};

        // Convert VaultRoot (PathBuf wrapper) to DirPath
        let root = DirPath::try_from(
            self.vault_metadata.root().as_path().to_path_buf(),
        )
        .expect("vault root should resolve to a valid directory path");

        // property_bank_path() joins validated relative paths (schemas_dir +
        // property_bank filename), so the result is guaranteed to be a
        // valid RelativePath
        let property_bank_rel =
            RelativePath::try_from(self.paths.property_bank_path())
                .expect("property bank path should be valid relative path");

        SchemaConfigSpec::new(
            root,
            self.paths.schema.schemas_dir().clone(),
            property_bank_rel,
        )
    }

    /// Create a new Config with the specified version, keeping all other fields
    /// unchanged.
    ///
    /// This is used when atomically allocating a version number during
    /// persistence.
    #[inline]
    #[must_use]
    pub fn with_version(mut self, version: Version) -> Self {
        self.version = version;
        self
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

#[cfg(test)]
pub(crate) mod fixtures {
    use std::path::PathBuf;

    use super::*;
    use crate::config::{
        raw::{RawFrontmatter, RawLogging},
        vault::{AppVersion, VaultId, VaultName, VaultRoot},
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
        let vault = crate::config::raw::RawVaultConfig {
            vault_path: "/vault".to_owned(),
            logging: Some(RawLogging {
                log_level: Some("debug".to_owned()),
            }),
            paths: crate::config::raw::RawVaultPaths {
                schemas_dir: Some("schemas".to_owned()),
                templates_dir: Some("custom_templates".to_owned()),
                property_bank_file: Some("property_bank.json".to_owned()),
                cache_dir: Some(".lithos".to_owned()),
            },
            frontmatter: Some(RawFrontmatter {
                alias_key: Some("aliases".to_owned()),
                tags_key: None,
                date_created_key: Some("created".to_owned()),
                date_modified_key: Some("modified".to_owned()),
                file_class_key: Some("type".to_owned()),
                title_key: Some("title".to_owned()),
            }),
            ..Default::default()
        };
        crate::config::builder::build_from_layers(
            None,
            Some(&vault),
            vault_id(),
            vault_root("/vault"),
            Version::initial(),
        )
        .expect("Config build should succeed with sample data")
    }

    pub fn merged_config_with_empty_inputs() -> Config {
        crate::config::builder::build_from_layers(
            None,
            None,
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

#[cfg(test)]
mod tests {
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
            let config = crate::config::builder::build_from_layers(
                None,
                None,
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
            let config = crate::config::builder::build_from_layers(
                None,
                None,
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
        fn should_map_to_frontmatter_spec() {
            let config = fixtures::test_config();
            let spec = config.to_frontmatter_spec();
            assert_eq!(spec.title_key.as_ref(), "title");
            assert_eq!(spec.alias_key.as_ref(), "aliases");
        }

        #[test]
        fn should_map_to_task_spec_with_defaults() {
            let config = fixtures::test_config();
            let spec = config.to_task_spec();
            assert!(spec.enabled);
            assert!(spec.use_emoji);
            assert!(spec.emoji_markers.contains(&'\u{1f4c5}')); // 📅
            #[expect(clippy::indexing_slicing, reason = "Test assertion")]
            {
                assert_eq!(spec.status_mappings[&'x'].as_ref(), "done");
            }
        }

        #[test]
        fn builds_config_from_empty_raw() {
            let config = crate::config::builder::build_from_layers(
                None,
                None,
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
            let vault = crate::config::raw::RawVaultConfig {
                vault_path: "/vault".to_owned(),
                paths: crate::config::raw::RawVaultPaths {
                    cache_dir: Some(".lithos-cache".to_owned()),
                    schemas_dir: Some("my-schemas".to_owned()),
                    property_bank_file: Some("bank.json".to_owned()),
                    templates_dir: Some("my-templates".to_owned()),
                },
                ..Default::default()
            };
            let config = crate::config::builder::build_from_layers(
                None,
                Some(&vault),
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

    mod schema_spec {
        use super::*;

        #[test]
        fn to_schema_spec_constructs_correct_paths() {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let schemas_dir = root.path().join("schemas");
            std::fs::create_dir_all(&schemas_dir)
                .expect("schemas dir should be created");
            let property_bank = schemas_dir.join("property_bank.json");
            std::fs::write(&property_bank, "{}")
                .expect("property bank should be writable");

            let vault = crate::config::raw::RawVaultConfig {
                vault_path: root.path().to_string_lossy().to_string(),
                ..Default::default()
            };
            let config = crate::config::builder::build_from_layers(
                None,
                Some(&vault),
                fixtures::vault_id(),
                crate::config::vault::VaultRoot::try_new(
                    root.path().to_path_buf(),
                )
                .expect("vault root should be valid"),
                Version::initial(),
            )
            .expect("config should build");

            let spec = config.to_schema_spec();

            // Should be absolute paths (vault root + relative paths)
            assert_eq!(
                spec.directory().as_path(),
                schemas_dir.as_path(),
                "Directory should be absolute (vault root + schemas_dir)"
            );
            assert_eq!(
                spec.property_bank().as_path(),
                property_bank.as_path(),
                "Property bank should be absolute (vault root + path)"
            );
            assert!(
                spec.directory().is_absolute(),
                "Directory path should be absolute"
            );
            assert!(
                spec.property_bank().is_absolute(),
                "Property bank path should be absolute"
            );
        }

        #[test]
        fn to_schema_spec_respects_custom_paths() {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let custom_schemas = root.path().join("custom-schemas");
            std::fs::create_dir_all(&custom_schemas)
                .expect("custom schemas dir should be created");
            let custom_bank = custom_schemas.join("custom-bank.json");
            std::fs::write(&custom_bank, "{}")
                .expect("custom property bank should be writable");

            let vault = crate::config::raw::RawVaultConfig {
                vault_path: root.path().to_string_lossy().to_string(),
                paths: crate::config::raw::RawVaultPaths {
                    schemas_dir: Some("custom-schemas".to_owned()),
                    property_bank_file: Some("custom-bank.json".to_owned()),
                    ..Default::default()
                },
                ..Default::default()
            };
            let config = crate::config::builder::build_from_layers(
                None,
                Some(&vault),
                fixtures::vault_id(),
                crate::config::vault::VaultRoot::try_new(
                    root.path().to_path_buf(),
                )
                .expect("vault root should be valid"),
                Version::initial(),
            )
            .unwrap();

            let spec = config.to_schema_spec();

            // Should be absolute paths (vault root + custom relative paths)
            assert_eq!(spec.directory().as_path(), custom_schemas.as_path());
            assert_eq!(spec.property_bank().as_path(), custom_bank.as_path());
        }
    }
}
