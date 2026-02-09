//! Config bounded context aggregate root.
//!
//! This module defines the `Config` aggregate root that represents the final
//! resolved configuration for a vault operation. It handles the merging logic
//! between global settings and vault-specific overrides.
//!
//! # Constraints
//! - **Precedence**: Vault-specific configuration always overrides global
//!   configuration.
//! - **Defaults**: Sensible system defaults are applied when both global and
//!   vault configurations are missing specific fields.
//! - **Immutability**: Once built, the configuration is immutable and serves as
//!   the "Source of Truth" for the current execution context.
//! - **Validation**: All paths and enums are strictly validated during the
//!   build phase.

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
    global::{Global, Paths as GlobalPaths},
    logging::Logging,
    paths::{CacheDir, FileName, Schema, SchemasDir, Template, TemplatesDir},
    raw,
    task::TaskConfig,
    vault::{Metadata, Vault, VaultId, VaultRoot},
};

/// Resolved vault filesystem configuration (merged).
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
pub struct ResolvedVaultPaths {
    /// Cache directory for vault.
    pub cache_dir: CacheDir,
    /// Schema configuration for vault.
    pub schema: Schema,
    /// Template configuration for vault.
    pub template: Template,
}

/// Monotonic version identifier for merged configs.
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
                message: "version must be >= 1".to_owned().into(),
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

impl ResolvedVaultPaths {
    #[inline]
    /// Validate resolved vault filesystem paths.
    ///
    /// # Errors
    /// Returns `ConfigError` if schema or template validation fails.
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.schema.validate()?;
        self.template.validate()?;
        Ok(())
    }
}

/// Merged configuration result from global and vault configurations.
///
/// This struct represents the final merged configuration after applying
/// domain constraints for precedence (vault overrides global). The
/// configuration is immutable once created and represents the complete runtime
/// configuration for a vault operation.
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
    /// Global filesystem configuration.
    pub global_filesystem: GlobalPaths,
    /// Vault filesystem configuration.
    pub vault_filesystem: ResolvedVaultPaths,
    /// Merged frontmatter configuration.
    pub frontmatter: Frontmatter,
    /// Merged task configuration.
    pub task: TaskConfig,
    /// Domain events pending emission (not persisted).
    #[serde(skip)]
    pending_events: Vec<Events>,
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Self {
            frontmatter: Frontmatter::default(),
            global_filesystem: GlobalPaths::default(),
            logging: Logging::default(),
            pending_events: vec![],
            task: TaskConfig::default(),
            vault_filesystem: ResolvedVaultPaths {
                cache_dir: CacheDir::default(),
                schema: Schema::default(),
                template: Template::default(),
            },
            vault_metadata: Metadata::default(),
        }
    }
}

impl Config {
    /// Adds a domain event to the pending events collection.
    #[inline]
    fn add_event(&mut self, event: Events) {
        self.pending_events.push(event);
    }

    /// Build a new Config by combining optional Global and Vault configurations
    /// with domain constraints.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if `vault_path` is empty.
    /// Returns `ConfigError::ValidationFailed` if metadata is invalid.
    ///
    /// # Examples
    /// ```
    /// # use std::path::PathBuf;
    /// # use lithos_core::config::{
    /// #   aggregate::Config,
    /// #   global::Global,
    /// #   vault::{Vault, VaultId, VaultRoot}
    /// # };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let global = Global::default();
    /// let vault = Vault::default();
    /// let config = Config::build(
    ///     Some(&global),
    ///     VaultId::new(),
    ///     VaultRoot::try_new(PathBuf::from("/vault"))?,
    ///     &vault,
    /// )?;
    /// assert_eq!(
    ///     config.vault_metadata.root().as_path(),
    ///     PathBuf::from("/vault").as_path(),
    ///     "Vault root should match"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn build(
        global: Option<&Global>,
        vault_id: VaultId,
        vault_root: VaultRoot,
        vault: &Vault,
    ) -> Result<Self, ConfigError> {
        let vault_metadata = Metadata::new(vault_id, vault_root, None, None)?;

        let global_filesystem =
            global.map(|g| g.filesystem().clone()).unwrap_or_default();

        let schema_defaults = Schema::default();
        let template_defaults = Template::default();

        let schemas_dir = vault
            .filesystem()
            .schema()
            .schemas_dir
            .clone()
            .or_else(|| {
                global.map(|g| g.filesystem().schema().schemas_dir().clone())
            })
            .unwrap_or_else(|| schema_defaults.schemas_dir().clone());

        let property_bank_filename = vault
            .filesystem()
            .schema()
            .property_bank_filename
            .clone()
            .or_else(|| {
                global.map(|g| {
                    g.filesystem().schema().property_bank_filename().clone()
                })
            })
            .unwrap_or_else(|| {
                schema_defaults.property_bank_filename().clone()
            });

        let templates_dir = vault
            .filesystem()
            .template()
            .templates_dir()
            .cloned()
            .or_else(|| {
                global
                    .map(|g| g.filesystem().template().templates_dir().clone())
            })
            .unwrap_or_else(|| template_defaults.templates_dir().clone());

        let cache_dir = vault
            .filesystem()
            .cache_dir()
            .cloned()
            .unwrap_or_else(CacheDir::default);

        let vault_filesystem = ResolvedVaultPaths {
            cache_dir,
            schema: Schema::new(schemas_dir, property_bank_filename),
            template: Template::new(templates_dir),
        };

        let frontmatter = Config::merge_frontmatter(
            global.map(Global::frontmatter),
            vault.frontmatter(),
        );

        let logging =
            Config::merge_logging(global.map(Global::logging), vault.logging());

        let task =
            Config::merge_task(global.and_then(Global::task), vault.task());

        // Step 5: Construct the final strictly-validated aggregate
        let mut config = Self {
            frontmatter,
            global_filesystem,
            logging,
            task,
            vault_filesystem,
            vault_metadata,
            pending_events: vec![],
        };

        config.add_event(Events::ConfigUpdated(ConfigUpdated::new(
            "merged".to_owned(),
            chrono::Utc::now().timestamp(),
        )));

        // Step 5: Final invariant check
        config.validate()?;

        Ok(config)
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
    fn opt_filename(
        opt: &Option<String>,
        field_name: &str,
    ) -> Result<Option<FileName>, ConfigError> {
        opt.as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| {
                FileName::try_new(s.clone()).map_err(|e| {
                    ConfigError::ValidationFailed {
                        field: field_name.to_owned().into(),
                        message: format!("invalid filename: {e}").into(),
                    }
                })
            })
            .transpose()
    }

    /// Default property bank filename - guaranteed valid constant.
    #[expect(
        clippy::expect_used,
        clippy::disallowed_methods,
        reason = "Compile-time constant 'property_bank.json' is guaranteed \
                  valid"
    )]
    fn default_property_bank_filename() -> FileName {
        FileName::try_new("property_bank.json")
            .expect("property_bank.json is a valid constant filename")
    }

    /// Build global filesystem paths from merged config.
    fn make_global_filesystem(
        fs: &raw::RawFilesystemConfig,
    ) -> Result<GlobalPaths, ConfigError> {
        let schemas_dir = Self::opt_path_to_domain(
            &fs.schemas_dir,
            SchemasDir::try_new,
            "schemas_dir",
        )?
        .unwrap_or_default();
        let property_bank_filename = Self::opt_filename(
            &fs.property_bank_filename,
            "property_bank_filename",
        )?
        .unwrap_or_else(Self::default_property_bank_filename);
        let templates_dir = Self::opt_path_to_domain(
            &fs.templates_dir,
            TemplatesDir::try_new,
            "templates_dir",
        )?
        .unwrap_or_default();

        Ok(GlobalPaths::new(
            Schema::new(schemas_dir, property_bank_filename),
            Template::new(templates_dir),
        ))
    }

    /// Build vault filesystem paths from merged config.
    fn make_vault_filesystem(
        fs: &raw::RawFilesystemConfig,
    ) -> Result<ResolvedVaultPaths, ConfigError> {
        let cache_dir = Self::opt_path_to_domain(
            &fs.cache_dir,
            CacheDir::try_new,
            "cache_dir",
        )?
        .unwrap_or_default();
        let schemas_dir = Self::opt_path_to_domain(
            &fs.schemas_dir,
            SchemasDir::try_new,
            "schemas_dir",
        )?
        .unwrap_or_default();
        let property_bank_filename = Self::opt_filename(
            &fs.property_bank_filename,
            "property_bank_filename",
        )?
        .unwrap_or_else(Self::default_property_bank_filename);
        let templates_dir = Self::opt_path_to_domain(
            &fs.templates_dir,
            TemplatesDir::try_new,
            "templates_dir",
        )?
        .unwrap_or_default();

        Ok(ResolvedVaultPaths {
            cache_dir,
            schema: Schema::new(schemas_dir, property_bank_filename),
            template: Template::new(templates_dir),
        })
    }

    /// Build a Config directly from a merged `RawConfig` using Figment.
    ///
    /// This is the Phase 1 entry point that simplifies the config building
    /// by using the pre-merged `RawConfig` from `build_merged_raw()`. The
    /// `vault_root` is required to construct vault metadata.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if `vault_root` is empty.
    /// Returns `ConfigError::ValidationFailed` if metadata is invalid.
    #[inline]
    pub fn from_merged_raw(
        raw: &raw::RawConfig,
        vault_id: VaultId,
        vault_root: VaultRoot,
    ) -> Result<Self, ConfigError> {
        let vault_metadata = Metadata::new(vault_id, vault_root, None, None)?;

        let fs = &raw.filesystem;

        let global_filesystem = Self::make_global_filesystem(fs)?;
        let vault_filesystem = Self::make_vault_filesystem(fs)?;

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
            global_filesystem,
            logging,
            task,
            vault_filesystem,
            vault_metadata,
            pending_events: vec![],
        };

        config.add_event(Events::ConfigUpdated(ConfigUpdated::new(
            "merged".to_owned(),
            chrono::Utc::now().timestamp(),
        )));

        config.validate()?;
        Ok(config)
    }

    /// Merge frontmatter configurations applying defaults where needed.
    pub(crate) fn merge_frontmatter(
        global: Option<&Frontmatter>,
        vault: Option<&Frontmatter>,
    ) -> Frontmatter {
        vault.cloned().or_else(|| global.cloned()).unwrap_or_default()
    }

    /// Merge logging configurations applying defaults where needed.
    pub(crate) fn merge_logging(
        global: Option<&Logging>,
        vault: Option<&Logging>,
    ) -> Logging {
        vault.cloned().or_else(|| global.cloned()).unwrap_or_default()
    }

    /// Merge task configurations applying defaults where needed.
    pub(crate) fn merge_task(
        global: Option<&TaskConfig>,
        vault: Option<&TaskConfig>,
    ) -> TaskConfig {
        vault.cloned().or_else(|| global.cloned()).unwrap_or_default()
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

    /// Validate configuration against critical domain constraints.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if any required field is empty.
    /// Returns `ConfigError::ValidationFailed` if metadata is invalid.
    ///
    /// # Examples
    /// ```
    /// # use std::path::PathBuf;
    /// # use lithos_core::config::{
    /// #   aggregate::Config,
    /// #   global::Global,
    /// #   vault::{Vault, VaultId, VaultRoot}
    /// # };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let global = Global::default();
    /// let vault = Vault::default();
    /// let config = Config::build(
    ///     Some(&global),
    ///     VaultId::new(),
    ///     VaultRoot::try_new(PathBuf::from("/vault"))?,
    ///     &vault,
    /// )?;
    /// config.validate()?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate all component parts
        self.vault_metadata.validate()?;
        self.global_filesystem.validate()?;
        self.vault_filesystem.validate()?;
        self.frontmatter.validate()?;
        self.logging.validate()?;

        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures and submodules for readability."
)]
mod tests {
    // # LINT_DISABLE_REASON: Standard test utilities and behavioral
    // verification patterns.
    #[expect(
        clippy::disallowed_methods,
        reason = "Fixture helpers use expect for deterministic setup."
    )]
    mod fixtures {
        use std::path::PathBuf;

        use super::super::*;
        use crate::config::{
            frontmatter::FrontmatterKey,
            logging::LogLevel,
            paths::{
                FileName, SchemaOverrides, SchemasDir, TemplateOverrides,
                TemplatesDir,
            },
        };

        pub fn vault_id() -> VaultId {
            VaultId::new()
        }

        pub fn vault_root(path: &str) -> VaultRoot {
            VaultRoot::try_new(PathBuf::from(path)).expect("vault_root")
        }

        fn schema(dir: &str, file: &str) -> Schema {
            let schemas_dir =
                SchemasDir::try_new(PathBuf::from(dir)).expect("schemas_dir");
            let file_name = FileName::try_new(file).expect("file_name");
            Schema::new(schemas_dir, file_name)
        }

        fn template(dir: &str) -> Template {
            let templates_dir = TemplatesDir::try_new(PathBuf::from(dir))
                .expect("templates_dir");
            Template::new(templates_dir)
        }

        pub fn frontmatter(
            alias_key: &str,
            date_created_key: &str,
            date_modified_key: &str,
            file_class_key: &str,
            title_key: &str,
        ) -> Frontmatter {
            Frontmatter::new(
                FrontmatterKey::try_new(alias_key).expect("alias_key"),
                FrontmatterKey::try_new(date_created_key)
                    .expect("date_created_key"),
                FrontmatterKey::try_new(date_modified_key)
                    .expect("date_modified_key"),
                FrontmatterKey::try_new(file_class_key)
                    .expect("file_class_key"),
                FrontmatterKey::try_new(title_key).expect("title_key"),
            )
        }

        /// Test fixture: Create sample global configuration with system
        /// defaults.
        pub fn sample_global_config() -> Global {
            Global::new(
                GlobalPaths::new(
                    schema("schemas", "property_bank.json"),
                    template("templates"),
                ),
                frontmatter(
                    "aliases",
                    "date_created",
                    "date_modified",
                    "file_class",
                    "title",
                ),
                Logging::new(LogLevel::Info),
                None,
                None,
            )
        }

        /// Test fixture: Create sample vault configuration with user overrides.
        pub fn sample_vault_config() -> Vault {
            use crate::config::vault::Paths as VaultPaths;
            Vault::new(
                VaultPaths::new(
                    None,
                    SchemaOverrides::new(
                        Some(
                            schema("schemas", "property_bank.json")
                                .schemas_dir()
                                .clone(),
                        ),
                        Some(
                            schema("schemas", "property_bank.json")
                                .property_bank_filename()
                                .clone(),
                        ),
                    ),
                    TemplateOverrides::new(Some(
                        template("custom_templates").templates_dir().clone(),
                    )),
                ),
                Some(frontmatter(
                    "aliases", "created", "modified", "type", "title",
                )),
                Some(Logging::new(LogLevel::Debug)),
                None,
            )
        }

        pub fn merged_config_with_sample_overrides() -> Config {
            let global = sample_global_config();
            let vault = sample_vault_config();
            Config::build(
                Some(&global),
                vault_id(),
                vault_root("/vault"),
                &vault,
            )
            .expect("Config build should succeed with sample data")
        }

        pub fn merged_config_with_empty_inputs() -> Config {
            let global = Global::default();
            let vault = Vault::default();

            Config::build(
                Some(&global),
                vault_id(),
                vault_root("/vault"),
                &vault,
            )
            .expect("Merge with empty values should succeed")
        }

        pub fn config_with_cleared_events() -> Config {
            let mut config = Config::default();
            let events = config.take_events();
            drop(events);
            config
        }
    }

    use super::*;
    use crate::config::paths::CacheDir;

    mod integrity {
        use super::*;

        #[test]
        fn config_version_initial_is_one() {
            assert_eq!(ConfigVersion::initial().value(), 1);
        }

        #[test]
        #[expect(
            clippy::panic_in_result_fn,
            reason = "Test uses assert_eq! which can panic."
        )]
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
            let _: ConfigError = result.expect_err("should reject zero");
        }

        #[test]
        #[expect(
            clippy::panic_in_result_fn,
            reason = "Test uses assert_eq! which can panic."
        )]
        fn config_version_try_from_accepts_positive()
        -> Result<(), Box<dyn std::error::Error>> {
            let v = ConfigVersion::try_from(42)?;
            assert_eq!(v.value(), 42);
            Ok(())
        }

        /// 3.3-UNIT-020: `build_handles_missing_global`.
        /// Priority: P1.
        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic config construction."
        )]
        fn build_handles_missing_global_sets_vault_path() {
            let vault = Vault::default();

            let config = Config::build(
                None,
                fixtures::vault_id(),
                fixtures::vault_root("/vault"),
                &vault,
            )
            .expect("Config build should succeed");

            assert_eq!(
                config.vault_metadata.root().as_path(),
                fixtures::vault_root("/vault").as_path(),
                "Vault path should match input"
            );
        }

        /// 3.3-UNIT-020: `build_handles_missing_global`.
        /// Priority: P1.
        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic config construction."
        )]
        fn build_handles_missing_global_applies_global_defaults() {
            let vault = Vault::default();

            let config = Config::build(
                None,
                fixtures::vault_id(),
                fixtures::vault_root("/vault"),
                &vault,
            )
            .expect("Config build should succeed");

            assert_eq!(
                config.global_filesystem.schema().schemas_dir().as_path(),
                std::path::Path::new("schemas"),
                "Global schema dir should use defaults"
            );
        }

        /// 3.3-UNIT-021: `config_manages_domain_events`.
        /// Priority: P1.
        #[test]
        fn pending_events_empty_after_take_events() {
            let config = fixtures::config_with_cleared_events();
            assert!(
                config.pending_events().is_empty(),
                "Config should have no pending events after take_events()"
            );
        }

        /// 3.3-UNIT-021: `config_manages_domain_events`.
        /// Priority: P1.
        #[test]
        fn add_event_records_pending_event() {
            let mut config = fixtures::config_with_cleared_events();
            let event =
                Events::ConfigUpdated(ConfigUpdated::new("test".to_owned(), 0));
            config.add_event(event);

            assert_eq!(
                config.pending_events().len(),
                1,
                "Config should have 1 pending event after adding"
            );
        }

        /// 3.3-UNIT-021: `config_manages_domain_events`.
        /// Priority: P1.
        #[test]
        fn take_events_returns_added_event() {
            let mut config = fixtures::config_with_cleared_events();
            let event =
                Events::ConfigUpdated(ConfigUpdated::new("test".to_owned(), 0));
            config.add_event(event);

            assert_eq!(
                config.take_events().len(),
                1,
                "take_events() should return 1 event"
            );
        }

        /// 3.3-UNIT-021: `config_manages_domain_events`.
        /// Priority: P1.
        #[test]
        fn take_events_clears_pending_events() {
            let mut config = fixtures::config_with_cleared_events();
            let event =
                Events::ConfigUpdated(ConfigUpdated::new("test".to_owned(), 0));
            config.add_event(event);
            let _events_first = config.take_events();

            assert!(
                config.pending_events().is_empty(),
                "Config should have no pending events after take_events()"
            );
        }

        /// 3.3-UNIT-019:
        /// `merge_frontmatter_handles_various_input_combinations`.
        /// Priority: P1.
        #[test]
        fn merge_frontmatter_prefers_vault_values() {
            let g = fixtures::frontmatter(
                "aliases",
                "date_created",
                "date_modified",
                "file_class",
                "gt",
            );
            let v = fixtures::frontmatter(
                "aliases",
                "date_created",
                "date_modified",
                "file_class",
                "vt",
            );

            assert_eq!(
                Config::merge_frontmatter(Some(&g), Some(&v))
                    .title_key()
                    .as_str(),
                "vt",
            );
        }

        /// 3.3-UNIT-019:
        /// `merge_frontmatter_handles_various_input_combinations`.
        /// Priority: P1.
        #[test]
        fn merge_frontmatter_uses_global_when_vault_missing() {
            let g = fixtures::frontmatter(
                "aliases",
                "date_created",
                "date_modified",
                "file_class",
                "gt",
            );

            assert_eq!(
                Config::merge_frontmatter(Some(&g), None).title_key().as_str(),
                "gt",
            );
        }

        /// 3.3-UNIT-019:
        /// `merge_frontmatter_handles_various_input_combinations`.
        /// Priority: P1.
        #[test]
        fn merge_frontmatter_uses_defaults_when_all_missing() {
            assert_eq!(
                Config::merge_frontmatter(None, None).title_key().as_str(),
                "title",
            );
        }

        /// 3.3-UNIT-017: `supports_clone_debug_and_partial_eq`.
        /// Priority: P3.
        #[test]
        fn debug_trait_produces_output() {
            let config = fixtures::merged_config_with_sample_overrides();
            let debug_str = format!("{config:?}");
            assert!(
                !debug_str.is_empty(),
                "Debug derivation should produce non-empty string"
            );
        }

        /// 3.3-UNIT-017: `supports_clone_debug_and_partial_eq`.
        /// Priority: P3.
        #[test]
        fn clone_trait_preserves_equality() {
            let config = fixtures::merged_config_with_sample_overrides();
            let cloned = config.clone();
            assert_eq!(
                config, cloned,
                "Cloned config must be equal to original"
            );
        }

        /// 3.3-UNIT-017: `supports_clone_debug_and_partial_eq`.
        /// Priority: P3.
        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic config construction."
        )]
        fn merge_is_equivalent_for_identical_inputs() {
            let global = fixtures::sample_global_config();
            let vault = fixtures::sample_vault_config();
            let vault_id = fixtures::vault_id();
            let config = Config::build(
                Some(&global),
                vault_id,
                fixtures::vault_root("/vault"),
                &vault.clone(),
            )
            .expect("First merge for trait verification failed");
            let mut config = config;
            let _events = config.take_events();
            let config2 = Config::build(
                Some(&global),
                vault_id,
                fixtures::vault_root("/vault"),
                &vault,
            )
            .expect("Second merge for trait verification failed");
            let mut config2 = config2;
            let _events_second = config2.take_events();
            assert_eq!(
                config, config2,
                "Merged configs with identical input must be equal (PartialEq)"
            );
        }
    }

    mod merge {
        use super::*;
        use crate::config::logging::LogLevel;

        /// 3.3-UNIT-014: `falls_back_to_defaults_when_inputs_are_empty`.
        /// Priority: P1.
        #[test]
        fn defaults_apply_to_cache_dir() {
            let config = fixtures::merged_config_with_empty_inputs();
            assert_eq!(
                config.vault_filesystem.cache_dir.as_path(),
                CacheDir::default().as_path(),
                "Should fall back to default cache_dir"
            );
        }

        /// 3.3-UNIT-014: `falls_back_to_defaults_when_inputs_are_empty`.
        /// Priority: P1.
        #[test]
        fn defaults_apply_to_templates_dir() {
            let config = fixtures::merged_config_with_empty_inputs();
            let default_template = Template::default();
            assert_eq!(
                config.vault_filesystem.template.templates_dir().as_path(),
                default_template.templates_dir().as_path(),
                "Should fall back to default templates_dir"
            );
        }

        /// 3.3-UNIT-014: `falls_back_to_defaults_when_inputs_are_empty`.
        /// Priority: P1.
        #[test]
        fn defaults_apply_to_schemas_dir() {
            let config = fixtures::merged_config_with_empty_inputs();
            let default_schema = Schema::default();
            assert_eq!(
                config.vault_filesystem.schema.schemas_dir().as_path(),
                default_schema.schemas_dir().as_path(),
                "Should fall back to default schemas_dir"
            );
        }

        /// 3.3-UNIT-014: `falls_back_to_defaults_when_inputs_are_empty`.
        /// Priority: P1.
        #[test]
        fn defaults_apply_to_property_bank_filename() {
            let config = fixtures::merged_config_with_empty_inputs();
            let default_schema = Schema::default();
            assert_eq!(
                config
                    .vault_filesystem
                    .schema
                    .property_bank_filename()
                    .as_str(),
                default_schema.property_bank_filename().as_str(),
                "Should fall back to default property_bank_filename"
            );
        }

        /// 3.3-UNIT-014: `falls_back_to_defaults_when_inputs_are_empty`.
        /// Priority: P1.
        #[test]
        fn defaults_apply_to_log_level() {
            let config = fixtures::merged_config_with_empty_inputs();
            let default_logging = Logging::default();
            assert_eq!(
                config.logging.log_level(),
                default_logging.log_level(),
                "Should fall back to default log_level"
            );
        }

        /// 3.3-UNIT-014: `falls_back_to_defaults_when_inputs_are_empty`.
        /// Priority: P1.
        #[test]
        fn defaults_apply_to_frontmatter_file_class_key() {
            let config = fixtures::merged_config_with_empty_inputs();
            let default_frontmatter = Frontmatter::default();
            assert_eq!(
                config.frontmatter.file_class_key().as_str(),
                default_frontmatter.file_class_key().as_str(),
                "Should fall back to default file_class_key"
            );
        }

        /// 3.3-UNIT-014: `falls_back_to_defaults_when_inputs_are_empty`.
        /// Priority: P1.
        #[test]
        fn defaults_apply_to_frontmatter_title_key() {
            let config = fixtures::merged_config_with_empty_inputs();
            let default_frontmatter = Frontmatter::default();
            assert_eq!(
                config.frontmatter.title_key().as_str(),
                default_frontmatter.title_key().as_str(),
                "Should fall back to default title_key"
            );
        }

        /// 3.3-UNIT-014: `falls_back_to_defaults_when_inputs_are_empty`.
        /// Priority: P1.
        #[test]
        fn defaults_apply_to_frontmatter_alias_key() {
            let config = fixtures::merged_config_with_empty_inputs();
            let default_frontmatter = Frontmatter::default();
            assert_eq!(
                config.frontmatter.alias_key().as_str(),
                default_frontmatter.alias_key().as_str(),
                "Should fall back to default alias_key"
            );
        }

        /// 3.3-UNIT-015: `merge_is_idempotent`.
        /// Priority: P1.
        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic config construction."
        )]
        fn merge_is_idempotent() {
            // GIVEN: the same global and vault configs
            let global = fixtures::sample_global_config();
            let vault = fixtures::sample_vault_config();
            let vault_id = fixtures::vault_id();

            // WHEN: merging the same inputs multiple times
            let merged1 = Config::build(
                Some(&global),
                vault_id,
                fixtures::vault_root("/vault"),
                &vault.clone(),
            )
            .expect("First merge should succeed");
            let mut merged1 = merged1;
            let _events_first = merged1.take_events();

            let merged2 = Config::build(
                Some(&global),
                vault_id,
                fixtures::vault_root("/vault"),
                &vault,
            )
            .expect("Second merge should succeed");
            let mut merged2 = merged2;
            let _events_second = merged2.take_events();

            // THEN: results should be identical
            assert_eq!(
                merged1, merged2,
                "Repeated merges with same input must yield identical output"
            );
        }

        /// 3.3-UNIT-013: `vault_values_take_precedence_over_global`.
        /// Priority: P0.
        #[test]
        fn vault_templates_dir_overrides_global() {
            let merged = fixtures::merged_config_with_sample_overrides();
            assert_eq!(
                merged.vault_filesystem.template.templates_dir().as_path(),
                std::path::Path::new("custom_templates"),
                "Vault filesystem should have custom templates"
            );
        }

        /// 3.3-UNIT-013: `vault_values_take_precedence_over_global`.
        /// Priority: P0.
        #[test]
        fn vault_log_level_overrides_global() {
            let merged = fixtures::merged_config_with_sample_overrides();
            assert_eq!(
                merged.logging.log_level(),
                LogLevel::Debug,
                "Log level should override global"
            );
        }

        /// 3.3-UNIT-013: `vault_values_take_precedence_over_global`.
        /// Priority: P0.
        #[test]
        fn vault_frontmatter_file_class_overrides_global() {
            let merged = fixtures::merged_config_with_sample_overrides();
            assert_eq!(
                merged.frontmatter.file_class_key().as_str(),
                "type",
                "File class key should override global"
            );
        }

        /// 3.3-UNIT-013: `vault_values_take_precedence_over_global`.
        /// Priority: P0.
        #[test]
        fn vault_frontmatter_date_created_overrides_global() {
            let merged = fixtures::merged_config_with_sample_overrides();
            assert_eq!(
                merged.frontmatter.date_created_key().as_str(),
                "created",
                "Date created key should override global"
            );
        }
    }

    mod proptests {
        use std::path::PathBuf;

        use proptest::{prelude::*, test_runner::TestRunner};

        use super::*;
        use crate::config::{
            paths::{SchemaOverrides, TemplateOverrides, TemplatesDir},
            vault::{Paths as VaultPaths, VaultRoot},
        };

        // 3.3-UNIT-012: `merge_handles_various_path_lengths`.
        // Priority: P2.
        #[test]
        fn merge_handles_various_path_lengths()
        -> Result<(), Box<dyn std::error::Error>> {
            let mut runner = TestRunner::deterministic();
            let strategy = (
                "[a-zA-Z0-9][a-zA-Z0-9/_-]{0,199}",
                "[a-zA-Z0-9][a-zA-Z0-9/_-]{0,99}",
            );

            let run_result =
                runner.run(&strategy, |(vault_path, templates_dir)| {
                    // GIVEN a global config and generated vault path/template
                    // overrides
                    let global = fixtures::sample_global_config();
                    let templates_dir = TemplatesDir::try_new(PathBuf::from(
                        templates_dir.clone(),
                    ))
                    .map_err(|error| {
                        proptest::test_runner::TestCaseError::fail(format!(
                            "templates_dir should be valid, got: {error:?}"
                        ))
                    })?;
                    let vault_config = Vault::new(
                        VaultPaths::new(
                            None,
                            SchemaOverrides::new(None, None),
                            TemplateOverrides::new(Some(templates_dir)),
                        ),
                        None,
                        None,
                        None,
                    );

                    // WHEN building a config from the generated inputs
                    let root = VaultRoot::try_new(
                        PathBuf::from("/").join(&vault_path),
                    )
                    .map_err(|error| {
                        proptest::test_runner::TestCaseError::fail(format!(
                            "VaultRoot::try_new should succeed for \
                             '{vault_path}', got: {error:?}"
                        ))
                    })?;
                    let root_clone = root.clone();
                    let result = Config::build(
                        Some(&global),
                        fixtures::vault_id(),
                        root,
                        &vault_config,
                    );

                    let config = result.map_err(|error| {
                        proptest::test_runner::TestCaseError::fail(format!(
                            "Config::build should succeed for vault_path \
                             '{vault_path}', got: {error:?}"
                        ))
                    })?;
                    prop_assert_eq!(
                        config.vault_metadata.root().as_path(),
                        root_clone.as_path(),
                        "Valid paths should preserve vault_path"
                    );
                    Ok(())
                });
            run_result.map_err(|e| {
                format!("Proptest run should succeed, got: {e:?}")
            })?;

            Ok(())
        }
    }

    mod validation {
        use std::path::PathBuf;

        use super::*;
        use crate::config::{
            logging::LogLevel,
            vault::{Paths as VaultPaths, VaultRoot},
        };

        /// 3.3-UNIT-016: `enforces_required_fields_and_enum_constraints`.
        /// Priority: P0.
        #[test]
        fn valid_config_passes_validation() {
            // GIVEN: a vault config with specific field values
            let global = fixtures::sample_global_config();
            let vault = Vault::new(
                VaultPaths::default(),
                None,
                Some(Logging::new(LogLevel::Info)),
                None,
            );

            // WHEN: attempting to merge the configs
            let result = Config::build(
                Some(&global),
                fixtures::vault_id(),
                fixtures::vault_root("/vault"),
                &vault,
            );

            assert!(
                matches!(&result, Ok(config) if config.validate().is_ok()),
                "Expected valid config, got: {result:?}"
            );
        }

        /// 3.3-UNIT-016: `enforces_required_fields_and_enum_constraints`.
        /// Priority: P0.
        #[test]
        fn empty_path_reports_validation_failed() {
            let result = VaultRoot::try_new(PathBuf::from(""));

            assert!(
                matches!(
                    &result,
                    Err(ConfigError::ValidationFailed { field, .. })
                        if field.as_ref() == "vault_root"
                ),
                "Expected ValidationFailed for vault_root, got: {result:?}"
            );
        }

        /// 3.3-UNIT-016: `enforces_required_fields_and_enum_constraints`.
        /// Priority: P0.
        #[test]
        fn invalid_log_level_reports_invalid_enum() {
            let result = LogLevel::try_from("invalid".to_owned());

            assert!(
                matches!(
                    &result,
                    Err(ConfigError::InvalidEnumValue { field, .. })
                        if field.as_ref() == "log_level"
                ),
                "Expected InvalidEnumValue for log_level, got: {result:?}"
            );
        }
    }
}
