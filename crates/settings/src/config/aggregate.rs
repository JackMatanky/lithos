//! AppConfig aggregate root and versioning.
//!
//! This module provides the [`AppConfig`] aggregate, which represents the
//! fully-merged and validated configuration state for a vault. It also
//! defines `Version` for tracking configuration history.

use std::{collections::HashMap, sync::Arc};

use rkyv::with::Skip;

use super::{
    cache::{CacheConfig, CacheConfigSpec},
    error::ConfigError,
    events::{ConfigUpdated, Events},
    frontmatter::{Frontmatter, FrontmatterConfigSpec},
    logging::Logging,
    schema::{SchemaConfig, SchemaConfigSpec},
    task::{Task, TaskConfigSpec, TemporalSlot},
    template::{TemplateConfig, TemplateConfigSpec},
    vault::Metadata,
};

/// Fully-resolved and validated configuration for a vault.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(bytecheck(bounds()))]
#[non_exhaustive]
pub struct AppConfig {
    /// Version number for this merged config snapshot.
    version: Version,
    /// Vault metadata with versioning and naming.
    vault_metadata: Metadata,
    /// Merged logging configuration.
    logging: Logging,
    /// Merged cache configuration.
    cache: CacheConfig,
    /// Merged template configuration.
    template: TemplateConfig,
    /// Merged schema configuration.
    schema: SchemaConfig,
    /// Merged frontmatter configuration.
    frontmatter: Frontmatter,
    /// Merged task configuration.
    task: Task,
    /// Domain events pending emission (not persisted).
    #[rkyv(with = Skip)]
    pending_events: Vec<Events>,
}

impl AppConfig {
    /// Construct a validated `AppConfig` aggregate from validated parts.
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
        cache: CacheConfig,
        template: TemplateConfig,
        schema: SchemaConfig,
        frontmatter: Frontmatter,
        task: Task,
    ) -> Self {
        let mut config = Self {
            version,
            vault_metadata,
            logging,
            cache,
            template,
            schema,
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

    /// Return the cache configuration.
    #[inline]
    #[must_use]
    pub const fn cache(&self) -> &CacheConfig {
        &self.cache
    }

    /// Return the template configuration.
    #[inline]
    #[must_use]
    pub const fn template(&self) -> &TemplateConfig {
        &self.template
    }

    /// Return the schema configuration.
    #[inline]
    #[must_use]
    pub const fn schema(&self) -> &SchemaConfig {
        &self.schema
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
    /// # Errors
    ///
    /// Returns [`ConfigError::ValidationFailed`] if the schema directory or
    /// property bank declarations cannot be projected into config-spec types.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use traces_settings::config::aggregate::AppConfig;
    /// # fn example(config: &AppConfig) {
    /// let spec = config.to_schema_spec().expect("schema spec should build");
    /// // spec.root() returns vault root
    /// // spec.schema_directory_path() resolves absolute schemas directory
    /// // spec.property_bank_file_path() resolves absolute property bank file
    /// # }
    /// ```
    #[inline]
    pub fn to_schema_spec(&self) -> Result<SchemaConfigSpec, ConfigError> {
        use traces_fs::path::RelativeFilePath;

        let root = self.vault_metadata.root().as_dir_path().clone();

        let schema_directory =
            self.schema.schema_dir().as_relative_dir().clone();
        let property_bank_path = self.schema.property_bank_relative_path();
        let property_bank_file = RelativeFilePath::try_new(
            property_bank_path.to_string_lossy().as_ref(),
        )
        .map_err(|error| ConfigError::ValidationFailed {
            field: "paths.property_bank_file".into(),
            message: format!("invalid property bank declaration: {error}")
                .into(),
        })?;

        Ok(SchemaConfigSpec::new(root, schema_directory, property_bank_file))
    }

    /// Builds a template configuration specification for template discovery.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::ValidationFailed`] if the template directory
    /// declaration cannot be projected into config-spec types.
    #[inline]
    pub fn to_template_spec(&self) -> Result<TemplateConfigSpec, ConfigError> {
        let root = self.vault_metadata.root().as_dir_path().clone();
        let directory = self.template.template_dir().as_relative_dir().clone();

        Ok(TemplateConfigSpec::new(root, directory))
    }

    /// Builds a cache configuration specification for cache consumers.
    ///
    /// # Deprecation
    ///
    /// Use [`crate::DiscoveryResult`]'s cache root accessor instead.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::ValidationFailed`] if the cache directory
    /// declaration cannot be projected into config-spec types.
    #[deprecated(
        since = "0.1.0",
        note = "Use `DiscoveryResult::cache_root()` instead."
    )]
    #[expect(
        deprecated,
        reason = "to_cache_spec() is itself deprecated pending migration to \
                  DiscoveryResult::cache_root()"
    )]
    #[inline]
    pub fn to_cache_spec(&self) -> Result<CacheConfigSpec, ConfigError> {
        let root = self.vault_metadata.root().as_dir_path().clone();
        let directory = self.cache.cache_dir().as_relative_dir().clone();
        Ok(CacheConfigSpec::new(root, directory))
    }

    /// Creates the configured cache directory on disk.
    ///
    /// # Errors
    ///
    /// Returns a [`ConfigError`] if the directory cannot be created.
    #[inline]
    pub fn create_cache_dir(&self) -> Result<(), ConfigError> {
        std::fs::create_dir_all(
            self.vault_metadata
                .root()
                .as_path()
                .join(self.cache.directory().as_relative_dir().as_str()),
        )
        .map_err(ConfigError::from)
    }

    /// Create a new `AppConfig` with the specified version, keeping all other
    /// fields unchanged.
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

impl From<&AppConfig> for TemplateConfigSpec {
    #[inline]
    fn from(config: &AppConfig) -> Self {
        Self::new(
            config.vault_metadata.root().as_dir_path().clone(),
            config.template.template_dir().as_relative_dir().clone(),
        )
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

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
pub(crate) mod fixtures {
    use std::{
        ffi::OsStr,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;
    use crate::config::{
        raw::{RawFrontmatter, RawLogging},
        vault::{AppVersion, VaultId, VaultName, VaultRoot},
    };

    pub fn vault_id() -> VaultId {
        VaultId::new()
    }

    pub fn vault_root(path: &str) -> VaultRoot {
        let basename = std::path::Path::new(path)
            .file_name()
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| OsStr::new("vault"));
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_millis();
        let dir = std::env::temp_dir()
            .join(format!("traces-test-{millis}"))
            .join(basename);
        std::fs::create_dir_all(&dir).expect("test vault dir should exist");
        VaultRoot::try_new(dir).expect("vault_root")
    }

    /// Create an `AppConfig` with test values. Only available in tests.
    pub fn test_config() -> AppConfig {
        let test_root = vault_root("/test-vault");
        let test_name = VaultName::from_root(&test_root);
        let test_version = AppVersion::try_new(env!("CARGO_PKG_VERSION"))
            .expect("package version is non-empty");

        AppConfig {
            version: Version::initial(),
            vault_metadata: Metadata::new(
                VaultId::new(),
                test_root,
                Some(test_name),
                Some(test_version),
            )
            .expect("test metadata must be valid"),
            logging: Logging::default(),
            cache: crate::config::cache::CacheConfig::default(),
            template: crate::config::template::TemplateConfig::default(),
            schema: crate::config::schema::SchemaConfig::default(),
            frontmatter: Frontmatter::default(),
            task: Task::default(),
            pending_events: vec![],
        }
    }

    pub fn merged_config_with_sample_overrides() -> AppConfig {
        let vault = crate::config::raw::RawConfig {
            logging: Some(RawLogging {
                log_level: Some("debug".to_owned()),
            }),
            schema: Some(crate::config::raw::RawSchemaConfig {
                directory: Some("schemas".to_owned()),
                property_bank_file: Some("property_bank.json".to_owned()),
            }),
            template: Some(crate::config::raw::RawTemplateConfig {
                directory: Some("custom_templates".to_owned()),
            }),
            cache: Some(crate::config::raw::RawCacheConfig {
                directory: Some(".traces".to_owned()),
            }),
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
        .expect("AppConfig build should succeed with sample data")
    }

    pub fn merged_config_with_empty_inputs() -> AppConfig {
        crate::config::builder::build_from_layers(
            None,
            None,
            vault_id(),
            vault_root("/vault"),
            Version::initial(),
        )
        .expect("Merge with empty values should succeed")
    }

    pub fn config_with_cleared_events() -> AppConfig {
        let mut config = test_config();
        let _events: Vec<Events> = config.take_events();
        config
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

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
            let vault_root = fixtures::vault_root("/vault");
            let config = crate::config::builder::build_from_layers(
                None,
                None,
                fixtures::vault_id(),
                vault_root.clone(),
                Version::initial(),
            )
            .unwrap();
            assert_eq!(
                config.vault_metadata().root().as_path(),
                vault_root.as_path()
            );
        }

        #[test]
        fn root_resolution_comes_from_vault_root_param_not_config() {
            let vault_root = fixtures::vault_root("/vault-param-root");
            let vault = crate::config::raw::RawConfig {
                ..Default::default()
            };
            let config = crate::config::builder::build_from_layers(
                None,
                Some(&vault),
                fixtures::vault_id(),
                vault_root.clone(),
                Version::initial(),
            )
            .expect("should build config without vault_path");

            assert_eq!(
                config.vault_metadata().root().as_path(),
                vault_root.as_path(),
                "Vault root should come from the explicit param, not config \
                 struct"
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
                config.schema().schema_dir().as_relative_dir().as_str(),
                "schemas"
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
        #[expect(deprecated, reason = "testing deprecated accessor behavior")]
        fn defaults_apply_to_cache_dir() {
            let config = fixtures::merged_config_with_empty_inputs();
            assert_eq!(
                config.cache().cache_dir().as_relative_dir().as_str(),
                ".cache"
            );
        }

        #[test]
        fn defaults_apply_to_templates_dir() {
            let config = fixtures::merged_config_with_empty_inputs();
            assert_eq!(
                config.template().template_dir().as_relative_dir().as_str(),
                "templates"
            );
        }

        #[test]
        fn vault_templates_dir_overrides_global() {
            let merged = fixtures::merged_config_with_sample_overrides();
            assert_eq!(
                merged.template().template_dir().as_relative_dir().as_str(),
                "custom_templates"
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
                config.schema().schema_dir().as_relative_dir().as_str(),
                "schemas"
            );
            assert_eq!(
                config.schema().property_bank_file().as_str(),
                "property_bank.json"
            );
        }

        #[test]
        #[expect(deprecated, reason = "testing deprecated accessor behavior")]
        fn applies_paths_fields_from_raw() {
            let vault = crate::config::raw::RawConfig {
                schema: Some(crate::config::raw::RawSchemaConfig {
                    directory: Some("my-schemas".to_owned()),
                    property_bank_file: Some("bank.json".to_owned()),
                }),
                template: Some(crate::config::raw::RawTemplateConfig {
                    directory: Some("my-templates".to_owned()),
                }),
                cache: Some(crate::config::raw::RawCacheConfig {
                    directory: Some(".traces-cache".to_owned()),
                }),
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
                config.schema().schema_dir().as_relative_dir().as_str(),
                "my-schemas"
            );
            assert_eq!(
                config.cache().cache_dir().as_relative_dir().as_str(),
                ".traces-cache"
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

            let vault = crate::config::raw::RawConfig {
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

            let spec =
                config.to_schema_spec().expect("schema spec should build");

            // Should be absolute paths (vault root + relative paths)
            assert_eq!(
                spec.schema_directory_path()
                    .expect("schema directory path should resolve")
                    .as_path(),
                schemas_dir.as_path(),
                "Directory should be absolute (vault root + schemas_dir)"
            );
            assert_eq!(
                spec.property_bank_file_path()
                    .expect("property bank file path should resolve")
                    .as_path(),
                property_bank.as_path(),
                "Property bank should be absolute (vault root + path)"
            );
            assert!(
                spec.schema_directory_path()
                    .expect("schema directory path should resolve")
                    .is_absolute(),
                "Directory path should be absolute"
            );
            assert!(
                spec.property_bank_file_path()
                    .expect("property bank file path should resolve")
                    .is_absolute(),
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

            let vault = crate::config::raw::RawConfig {
                schema: Some(crate::config::raw::RawSchemaConfig {
                    directory: Some("custom-schemas".to_owned()),
                    property_bank_file: Some("custom-bank.json".to_owned()),
                }),
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

            let spec =
                config.to_schema_spec().expect("schema spec should build");

            // Should be absolute paths (vault root + custom relative paths)
            assert_eq!(
                spec.schema_directory_path()
                    .expect("schema directory path should resolve")
                    .as_path(),
                custom_schemas.as_path()
            );
            assert_eq!(
                spec.property_bank_file_path()
                    .expect("property bank file path should resolve")
                    .as_path(),
                custom_bank.as_path()
            );
        }

        #[test]
        fn to_schema_spec_returns_result_without_panicking() {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let schemas_dir = root.path().join("schemas");
            std::fs::create_dir_all(&schemas_dir)
                .expect("schemas dir should be created");
            let property_bank = schemas_dir.join("property_bank.json");
            std::fs::write(&property_bank, "{}")
                .expect("property bank should be writable");

            let vault = crate::config::raw::RawConfig {
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

            let result = config.to_schema_spec();
            assert!(
                result.is_ok(),
                "to_schema_spec should return Ok for valid config"
            );
        }
    }

    mod resolved_path_config {
        use super::*;

        #[test]
        #[expect(deprecated, reason = "testing deprecated accessor behavior")]
        fn returns_default_split_path_configs_from_empty_raw() {
            let config = crate::config::builder::build_from_layers(
                None,
                None,
                fixtures::vault_id(),
                fixtures::vault_root("/vault"),
                Version::initial(),
            )
            .expect("empty raw layers should build default config");

            assert_eq!(
                config.cache().cache_dir().as_relative_dir().as_str(),
                ".cache",
                "resolved config should expose default cache config"
            );
            assert_eq!(
                config.template().template_dir().as_relative_dir().as_str(),
                "templates",
                "resolved config should expose default template config"
            );
            assert_eq!(
                config.schema().schema_dir().as_relative_dir().as_str(),
                "schemas",
                "resolved config should expose default schema config"
            );
            assert_eq!(
                config.schema().property_bank_file().as_str(),
                "property_bank.json",
                "resolved config should expose default property bank file"
            );
        }

        #[test]
        #[expect(deprecated, reason = "testing deprecated accessor behavior")]
        fn applies_path_fields_from_raw_to_split_configs() {
            let vault = crate::config::raw::RawConfig {
                schema: Some(crate::config::raw::RawSchemaConfig {
                    directory: Some("my-schemas".to_owned()),
                    property_bank_file: Some("bank.json".to_owned()),
                }),
                template: Some(crate::config::raw::RawTemplateConfig {
                    directory: Some("my-templates".to_owned()),
                }),
                cache: Some(crate::config::raw::RawCacheConfig {
                    directory: Some(".traces-cache".to_owned()),
                }),
                ..Default::default()
            };
            let config = crate::config::builder::build_from_layers(
                None,
                Some(&vault),
                fixtures::vault_id(),
                fixtures::vault_root("/vault"),
                Version::initial(),
            )
            .expect("raw vault paths should build resolved config");

            assert_eq!(
                config.cache().cache_dir().as_relative_dir().as_str(),
                ".traces-cache",
                "raw cache_dir should populate CacheConfig"
            );
            assert_eq!(
                config.template().template_dir().as_relative_dir().as_str(),
                "my-templates",
                "raw templates_dir should populate TemplateConfig"
            );
            assert_eq!(
                config.schema().schema_dir().as_relative_dir().as_str(),
                "my-schemas",
                "raw schemas_dir should populate SchemaConfig"
            );
            assert_eq!(
                config.schema().property_bank_file().as_str(),
                "bank.json",
                "raw property_bank_file should populate SchemaConfig"
            );
        }
    }

    mod config_specs {
        use super::*;

        #[test]
        fn create_cache_dir_creates_configured_cache_directory() {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let cache = root.path().join(".traces-cache").join("nested");
            let vault = crate::config::raw::RawConfig {
                cache: Some(crate::config::raw::RawCacheConfig {
                    directory: Some(".traces-cache/nested".to_owned()),
                }),
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

            assert!(
                !cache.exists(),
                "test should start without the configured cache directory"
            );

            config.create_cache_dir().expect("cache dir should be created");

            assert!(
                cache.is_dir(),
                "configured cache directory should exist on disk"
            );
        }

        #[test]
        fn to_schema_spec_respects_custom_schema_config() {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let custom_schemas = root.path().join("custom-schemas");
            std::fs::create_dir_all(&custom_schemas)
                .expect("custom schemas dir should be created");
            let custom_bank = custom_schemas.join("custom-bank.json");
            std::fs::write(&custom_bank, "{}")
                .expect("custom property bank should be writable");
            let vault = crate::config::raw::RawConfig {
                schema: Some(crate::config::raw::RawSchemaConfig {
                    directory: Some("custom-schemas".to_owned()),
                    property_bank_file: Some("custom-bank.json".to_owned()),
                }),
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

            assert!(spec.is_ok(), "schema spec should build: {:?}", spec.err());
            let spec = spec.expect("result checked as ok");
            assert_eq!(
                spec.schema_directory_path()
                    .expect("schema directory should resolve")
                    .as_path(),
                custom_schemas.as_path(),
                "schema spec should use SchemaConfig schema directory"
            );
            assert_eq!(
                spec.property_bank_file_path()
                    .expect("property bank file should resolve")
                    .as_path(),
                custom_bank.as_path(),
                "schema spec should derive property bank path from \
                 SchemaConfig"
            );
        }

        #[test]
        fn to_template_spec_respects_custom_template_config() {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let templates = root.path().join("custom-templates");
            std::fs::create_dir_all(&templates)
                .expect("templates dir should be created");
            let vault = crate::config::raw::RawConfig {
                template: Some(crate::config::raw::RawTemplateConfig {
                    directory: Some("custom-templates".to_owned()),
                }),
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

            let spec = config.to_template_spec();

            assert!(
                spec.is_ok(),
                "template spec should build: {:?}",
                spec.err()
            );
            assert_eq!(
                spec.expect("result checked as ok")
                    .to_dir_path()
                    .expect("template dir should resolve")
                    .as_path(),
                templates.as_path(),
                "template spec should use TemplateConfig directory"
            );
        }

        #[test]
        #[expect(deprecated, reason = "testing deprecated method behavior")]
        fn to_cache_spec_respects_custom_cache_config() {
            let root = tempfile::tempdir().expect("temp dir should be created");
            let cache = root.path().join(".traces-cache");
            std::fs::create_dir_all(&cache)
                .expect("cache dir should be created");
            let vault = crate::config::raw::RawConfig {
                cache: Some(crate::config::raw::RawCacheConfig {
                    directory: Some(".traces-cache".to_owned()),
                }),
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

            let spec = config.to_cache_spec();

            assert!(spec.is_ok(), "cache spec should build: {:?}", spec.err());
            assert_eq!(
                spec.expect("result checked as ok")
                    .to_dir_path()
                    .expect("cache dir should resolve")
                    .as_path(),
                cache.as_path(),
                "cache spec should use CacheConfig directory"
            );
        }
    }

    mod template_spec {
        use super::*;

        mod conversions {
            use super::*;

            #[test]
            fn returns_vault_root_from_representative_config() {
                let config = fixtures::merged_config_with_empty_inputs();

                let spec =
                    crate::config::template::TemplateConfigSpec::from(&config);

                assert_eq!(
                    spec.root().as_path(),
                    config.vault_metadata().root().as_path()
                );
            }

            #[test]
            fn returns_default_template_directory_from_representative_config() {
                let config = fixtures::merged_config_with_empty_inputs();

                let spec =
                    crate::config::template::TemplateConfigSpec::from(&config);

                assert_eq!(spec.as_relative_dir().as_str(), "templates");
            }
        }

        mod create {
            use super::*;

            #[test]
            fn returns_vault_root_from_representative_config() {
                let config = fixtures::merged_config_with_empty_inputs();

                let spec = config
                    .to_template_spec()
                    .expect("template spec should build");

                assert_eq!(
                    spec.root().as_path(),
                    config.vault_metadata().root().as_path()
                );
            }

            #[test]
            fn returns_default_template_directory_from_representative_config() {
                let config = fixtures::merged_config_with_empty_inputs();

                let spec = config
                    .to_template_spec()
                    .expect("template spec should build");

                assert_eq!(spec.as_relative_dir().as_str(), "templates");
            }

            #[test]
            fn returns_template_spec_with_configured_template_directory() {
                let config = fixtures::merged_config_with_sample_overrides();

                let spec = config
                    .to_template_spec()
                    .expect("template spec should build");

                assert_eq!(spec.as_relative_dir().as_str(), "custom_templates");
            }
        }
    }
}
