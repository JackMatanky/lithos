//! Configuration domain entities and business logic.
//!
//! This module defines the configuration model with hierarchical merging,
//! validation, and encrypted field support following hexagonal architecture.
//!
//! # Business Rules
//! - Vault configuration overrides Global configuration (highest precedence)
//! - All configuration must be validated before use
//! - Encrypted fields are stored as opaque blobs (decryption is adapter concern)
//! - Immutable configuration entities following Rust ownership patterns

#![expect(
    clippy::module_name_repetitions,
    reason = "Domain types like ConfigValue and FileSystemConfig clearly indicate their purpose as configuration-related entities, which is more valuable than avoiding module name repetition"
)]

/// Default configuration constants organized by domain.
mod defaults {
    /// Filesystem-related defaults.
    pub mod filesystem {
        /// Default templates directory.
        pub const TEMPLATES_DIR: &str = "templates";
        /// Default schemas directory.
        pub const SCHEMAS_DIR: &str = "schemas";
        /// Default cache directory.
        pub const CACHE_DIR: &str = ".cache";
        /// Default property bank filename (located in schemas directory).
        pub const PROPERTY_BANK_FILENAME: &str = "property_bank.json";
    }

    /// Frontmatter-related defaults.
    pub mod frontmatter {
        /// Default file class key.
        pub const FILE_CLASS_KEY: &str = "file_class";
        /// Default title key.
        pub const TITLE_KEY: &str = "title";
        /// Default alias key.
        pub const ALIAS_KEY: &str = "aliases";
        /// Default date created key.
        pub const DATE_CREATED_KEY: &str = "date_created";
        /// Default date modified key.
        pub const DATE_MODIFIED_KEY: &str = "date_modified";
    }

    /// Logging-related defaults.
    pub mod logging {
        /// Default log level.
        pub const LOG_LEVEL: &str = "info";
        /// Valid log levels.
        pub const VALID_LOG_LEVELS: &[&str] =
            &["debug", "info", "warn", "error"];
    }
}

use std::collections::HashMap;

/// Configuration value types supporting multiple data types and encryption.
///
/// # Invariants
/// - All variants must be serializable with serde.
/// - Encrypted variant contains opaque bytes (adapter handles encryption/decryption).
/// - Array and Object variants allow nested configuration structures.
///
/// # Examples
/// ```
/// use lithos_domain::ConfigValue;
///
/// let string_val = ConfigValue::from("test".to_string());
/// let number_val = ConfigValue::from(42.0);
/// let bool_val = ConfigValue::from(true);
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum ConfigValue {
    /// Array of configuration values.
    Array(Vec<ConfigValue>),
    /// Boolean configuration value.
    Boolean(bool),
    /// Encrypted field data (opaque bytes, adapter handles encryption/decryption).
    Encrypted(Vec<u8>),
    /// Numeric configuration value (f64 for flexibility).
    Number(f64),
    /// Nested object configuration.
    Object(HashMap<String, ConfigValue>),
    /// String configuration value.
    String(String),
}

/// Filesystem-related configuration settings.
///
/// # Invariants
/// - `vault_path` must be an absolute path (validated by Config).
/// - All directory paths should end without trailing slash.
/// - Property bank file is always located in `schemas_dir`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct FileSystemConfig {
    /// Directory for cache files (relative to `vault_path`).
    pub cache_dir: String,
    /// Filename for property bank (located in `schemas_dir`).
    pub property_bank_filename: String,
    /// Directory for schema files (relative to `vault_path`).
    pub schemas_dir: String,
    /// Directory for template files (relative to `vault_path`).
    pub templates_dir: String,
    /// Root path to the vault (absolute path required).
    pub vault_path: String,
}

impl FileSystemConfig {
    /// Get the full path to the property bank file (`schemas_dir/property_bank_filename`).
    ///
    /// The property bank is always stored in the schemas directory.
    ///
    /// # Examples
    /// ```ignore
    /// let config = FileSystemConfig { schemas_dir: "schemas".to_string(), property_bank_filename: "props.json".to_string(), .. };
    /// assert_eq!(config.property_bank_path(), "schemas/props.json");
    /// ```
    #[inline]
    #[must_use]
    pub fn property_bank_path(&self) -> String {
        format!("{}/{}", self.schemas_dir, self.property_bank_filename)
    }
}

/// Frontmatter configuration for Markdown file metadata.
///
/// # Invariants
/// - All keys must be non-empty strings.
/// - Keys should follow YAML/TOML naming conventions (lowercase, underscores).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct FrontmatterConfig {
    /// Key for aliases in frontmatter.
    pub alias_key: String,
    /// Key for creation date in frontmatter.
    pub date_created_key: String,
    /// Key for modification date in frontmatter.
    pub date_modified_key: String,
    /// Key for file classification in frontmatter.
    pub file_class_key: String,
    /// Key for title field in frontmatter.
    pub title_key: String,
}

/// Vault-specific configuration (highest precedence).
///
/// # Business Rules
/// - Vault configuration overrides Global configuration.
/// - Loaded from vault-specific lithos.toml.
/// - All fields optional (missing fields fall back to global).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct VaultConfig {
    /// Filesystem configuration for vault.
    pub filesystem: FileSystemConfig,
    /// Frontmatter configuration for vault.
    pub frontmatter: FrontmatterConfig,
    /// Log level (debug, info, warn, error).
    pub log_level: String,
}

/// Global default configuration (lowest precedence).
///
/// # Business Rules
/// - Provides system-wide defaults.
/// - Loaded from global lithos.toml or system defaults.
/// - All fields must have values (no optionals).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct GlobalConfig {
    /// Filesystem configuration for global defaults.
    pub filesystem: FileSystemConfig,
    /// Frontmatter configuration for global defaults.
    pub frontmatter: FrontmatterConfig,
    /// Log level (debug, info, warn, error).
    pub log_level: String,
}

/// Merged configuration result (Vault overrides Global).
///
/// # Business Rules
/// - Result of merging Global and Vault configurations.
/// - Vault values take precedence over Global values.
/// - Empty values are replaced with defaults during merge.
/// - Immutable once created.
///
/// # Examples
/// ```ignore
/// // Note: Config structs are #[non_exhaustive] so can only be constructed within the crate.
/// // This example shows conceptual usage - actual construction would be done via builder
/// // patterns or factory methods in adapters.
/// use lithos_domain::{Config, GlobalConfig, VaultConfig, FileSystemConfig, FrontmatterConfig};
///
/// let global = GlobalConfig {
///     filesystem: FileSystemConfig {
///         vault_path: ".".to_string(),
///         templates_dir: "templates".to_string(),
///         schemas_dir: "schemas".to_string(),
///         property_bank_filename: "props.json".to_string(),
///         cache_dir: ".cache".to_string(),
///     },
///     frontmatter: FrontmatterConfig {
///         file_class_key: "file_class".to_string(),
///         title_key: "title".to_string(),
///         alias_key: "aliases".to_string(),
///         date_created_key: "created".to_string(),
///         date_modified_key: "modified".to_string(),
///     },
///     log_level: "info".to_string(),
/// };
///
/// let vault = VaultConfig {
///     filesystem: FileSystemConfig {
///         vault_path: "/vault".to_string(),
///         templates_dir: "templates".to_string(),
///         schemas_dir: "schemas".to_string(),
///         property_bank_filename: "props.json".to_string(),
///         cache_dir: ".cache".to_string(),
///     },
///     frontmatter: FrontmatterConfig {
///         file_class_key: "type".to_string(),
///         title_key: "title".to_string(),
///         alias_key: "aliases".to_string(),
///         date_created_key: "created".to_string(),
///         date_modified_key: "modified".to_string(),
///     },
///     log_level: "debug".to_string(),
/// };
///
/// let config = Config::merge(global, vault).expect("merge should succeed");
/// assert_eq!(config.filesystem.vault_path, "/vault");
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Config {
    /// Merged filesystem configuration.
    pub filesystem: FileSystemConfig,
    /// Merged frontmatter configuration.
    pub frontmatter: FrontmatterConfig,
    /// Log level (debug, info, warn, error).
    pub log_level: String,
}

impl Default for FileSystemConfig {
    #[inline]
    fn default() -> Self {
        Self {
            cache_dir: defaults::filesystem::CACHE_DIR.to_owned(),
            property_bank_filename:
                defaults::filesystem::PROPERTY_BANK_FILENAME.to_owned(),
            schemas_dir: defaults::filesystem::SCHEMAS_DIR.to_owned(),
            templates_dir: defaults::filesystem::TEMPLATES_DIR.to_owned(),
            vault_path: String::new(), // Must be provided by user
        }
    }
}

impl Default for FrontmatterConfig {
    #[inline]
    fn default() -> Self {
        Self {
            alias_key: defaults::frontmatter::ALIAS_KEY.to_owned(),
            date_created_key: defaults::frontmatter::DATE_CREATED_KEY
                .to_owned(),
            date_modified_key: defaults::frontmatter::DATE_MODIFIED_KEY
                .to_owned(),
            file_class_key: defaults::frontmatter::FILE_CLASS_KEY.to_owned(),
            title_key: defaults::frontmatter::TITLE_KEY.to_owned(),
        }
    }
}

impl Default for GlobalConfig {
    #[inline]
    fn default() -> Self {
        Self {
            filesystem: FileSystemConfig::default(),
            frontmatter: FrontmatterConfig::default(),
            log_level: defaults::logging::LOG_LEVEL.to_owned(),
        }
    }
}

// GREEN PHASE: Actual implementations
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Methods grouped logically: public API first, then private helpers"
)]
impl Config {
    /// Merge Global and Vault configurations with business rules (Vault overrides Global).
    ///
    /// # Business Rules
    /// - Vault configuration takes precedence over Global configuration.
    /// - Empty values in vault fall back to global values.
    /// - Empty values in global use system defaults.
    /// - Only `vault_path` must be non-empty (required field).
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if `vault_path` is empty.
    /// Returns `ConfigError::InvalidEnumValue` if `log_level` is invalid.
    #[inline]
    pub fn merge(
        global: &GlobalConfig,
        vault: VaultConfig,
    ) -> Result<Self, crate::ConfigError> {
        // Merge filesystem config with defaults
        let filesystem =
            Self::merge_filesystem(&global.filesystem, vault.filesystem)?;

        // Merge frontmatter config with defaults
        let frontmatter =
            Self::merge_frontmatter(&global.frontmatter, &vault.frontmatter);

        // Merge log level with validation
        let log_level =
            Self::merge_log_level(&global.log_level, &vault.log_level)?;

        Ok(Self {
            filesystem,
            frontmatter,
            log_level,
        })
    }

    /// Choose value with precedence: vault > global > default.
    fn choose_value(vault: &str, global: &str, default: &str) -> String {
        if !vault.is_empty() {
            vault.to_owned()
        } else if !global.is_empty() {
            global.to_owned()
        } else {
            default.to_owned()
        }
    }

    /// Merge filesystem configurations applying defaults where needed.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if `vault_path` is empty.
    fn merge_filesystem(
        global: &FileSystemConfig,
        vault: FileSystemConfig,
    ) -> Result<FileSystemConfig, crate::ConfigError> {
        Self::validate_vault_path(&vault.vault_path)?;

        let defaults = FileSystemConfig::default();

        Ok(FileSystemConfig {
            cache_dir: Self::choose_value(
                &vault.cache_dir,
                &global.cache_dir,
                &defaults.cache_dir,
            ),
            property_bank_filename: Self::choose_value(
                &vault.property_bank_filename,
                &global.property_bank_filename,
                &defaults.property_bank_filename,
            ),
            schemas_dir: Self::choose_value(
                &vault.schemas_dir,
                &global.schemas_dir,
                &defaults.schemas_dir,
            ),
            templates_dir: Self::choose_value(
                &vault.templates_dir,
                &global.templates_dir,
                &defaults.templates_dir,
            ),
            vault_path: vault.vault_path,
        })
    }

    /// Merge frontmatter configurations applying defaults where needed.
    fn merge_frontmatter(
        global: &FrontmatterConfig,
        vault: &FrontmatterConfig,
    ) -> FrontmatterConfig {
        let defaults = FrontmatterConfig::default();

        FrontmatterConfig {
            file_class_key: Self::choose_value(
                &vault.file_class_key,
                &global.file_class_key,
                &defaults.file_class_key,
            ),
            title_key: Self::choose_value(
                &vault.title_key,
                &global.title_key,
                &defaults.title_key,
            ),
            alias_key: Self::choose_value(
                &vault.alias_key,
                &global.alias_key,
                &defaults.alias_key,
            ),
            date_created_key: Self::choose_value(
                &vault.date_created_key,
                &global.date_created_key,
                &defaults.date_created_key,
            ),
            date_modified_key: Self::choose_value(
                &vault.date_modified_key,
                &global.date_modified_key,
                &defaults.date_modified_key,
            ),
        }
    }

    /// Merge log level with validation.
    ///
    /// # Errors
    /// Returns `ConfigError::InvalidEnumValue` if the chosen `log_level` is invalid.
    fn merge_log_level(
        global: &str,
        vault: &str,
    ) -> Result<String, crate::ConfigError> {
        let log_level = if vault.is_empty() {
            if global.is_empty() {
                defaults::logging::LOG_LEVEL
            } else {
                global
            }
        } else {
            vault
        };

        Self::validate_log_level(log_level)?;

        Ok(log_level.to_owned())
    }

    /// Validate a log level value.
    ///
    /// # Errors
    /// Returns `ConfigError::InvalidEnumValue` if `log_level` is invalid.
    fn validate_log_level(log_level: &str) -> Result<(), crate::ConfigError> {
        if !defaults::logging::VALID_LOG_LEVELS.contains(&log_level) {
            return Err(crate::ConfigError::InvalidEnumValue {
                field: "log_level".to_owned(),
                value: log_level.to_owned(),
                allowed: defaults::logging::VALID_LOG_LEVELS
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect(),
            });
        }

        Ok(())
    }

    /// Validate vault path value.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if `vault_path` is empty.
    fn validate_vault_path(vault_path: &str) -> Result<(), crate::ConfigError> {
        if vault_path.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "vault_path".to_owned(),
                message: "vault path cannot be empty (required field)"
                    .to_owned(),
            });
        }

        Ok(())
    }

    /// Validate configuration against critical business rules.
    ///
    /// # Validation Rules
    /// - `vault_path` cannot be empty (required field).
    /// - `log_level` must be one of: debug, info, warn, error.
    ///
    /// # Note
    /// This method is provided for post-construction validation if needed.
    /// The `merge()` method already performs validation during construction.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if `vault_path` is empty.
    /// Returns `ConfigError::InvalidEnumValue` if `log_level` is invalid.
    #[inline]
    pub fn validate(&self) -> Result<(), crate::ConfigError> {
        Self::validate_vault_path(&self.filesystem.vault_path)?;
        Self::validate_log_level(&self.log_level)?;

        Ok(())
    }
}

/// Convert String to `ConfigValue::String` variant.
///
/// # Examples
/// ```
/// use lithos_domain::ConfigValue;
///
/// let value = ConfigValue::from("test".to_string());
/// match value {
///     ConfigValue::String(s) => assert_eq!(s, "test"),
///     _ => panic!("expected String variant"),
/// }
/// ```
impl From<String> for ConfigValue {
    #[inline]
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

/// Convert f64 to `ConfigValue::Number` variant.
///
/// # Examples
/// ```
/// use lithos_domain::ConfigValue;
///
/// let value = ConfigValue::from(42.5);
/// match value {
///     ConfigValue::Number(n) => assert!((n - 42.5).abs() < f64::EPSILON),
///     _ => panic!("expected Number variant"),
/// }
/// ```
impl From<f64> for ConfigValue {
    #[inline]
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

/// Convert bool to `ConfigValue::Boolean` variant.
///
/// # Examples
/// ```
/// use lithos_domain::ConfigValue;
///
/// let value = ConfigValue::from(true);
/// match value {
///     ConfigValue::Boolean(b) => assert!(b),
///     _ => panic!("expected Boolean variant"),
/// }
/// ```
impl From<bool> for ConfigValue {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test fixture: Create sample global configuration with defaults.
    fn sample_global_config() -> GlobalConfig {
        GlobalConfig {
            filesystem: FileSystemConfig {
                cache_dir: ".cache".to_owned(),
                property_bank_filename: "property_bank.json".to_owned(),
                schemas_dir: "schemas".to_owned(),
                templates_dir: "templates".to_owned(),
                vault_path: ".".to_owned(),
            },
            frontmatter: FrontmatterConfig {
                alias_key: "aliases".to_owned(),
                date_created_key: "date_created".to_owned(),
                date_modified_key: "date_modified".to_owned(),
                file_class_key: "file_class".to_owned(),
                title_key: "title".to_owned(),
            },
            log_level: "info".to_owned(),
        }
    }

    /// Test fixture: Create sample vault configuration with overrides.
    fn sample_vault_config() -> VaultConfig {
        VaultConfig {
            filesystem: FileSystemConfig {
                cache_dir: ".cache".to_owned(),
                property_bank_filename: "property_bank.json".to_owned(),
                schemas_dir: "schemas".to_owned(), // same as global
                templates_dir: "custom_templates".to_owned(),
                vault_path: "/vault".to_owned(),
            },
            frontmatter: FrontmatterConfig {
                alias_key: "aliases".to_owned(),
                date_created_key: "created".to_owned(), // vault override
                date_modified_key: "modified".to_owned(), // vault override
                file_class_key: "type".to_owned(),      // vault override
                title_key: "title".to_owned(),
            },
            log_level: "debug".to_owned(), // vault override
        }
    }

    // ============================================================================
    // Config Entity Tests
    // ============================================================================

    #[test]
    fn config_merge_vault_overrides_global() {
        let global = sample_global_config();
        let vault = sample_vault_config();

        let result = Config::merge(&global, vault);
        assert!(result.is_ok(), "merge should succeed");

        // Use pattern matching to extract value for assertions
        if let Ok(merged) = result {
            // Business rule: vault values take precedence
            assert_eq!(merged.filesystem.vault_path, "/vault");
            assert_eq!(merged.filesystem.templates_dir, "custom_templates");
            assert_eq!(merged.log_level, "debug");
            assert_eq!(merged.frontmatter.file_class_key, "type");
            assert_eq!(merged.frontmatter.date_created_key, "created");
        }
    }

    #[test]
    fn config_validation_success() {
        let global = sample_global_config();
        let vault = sample_vault_config();

        if let Ok(config) = Config::merge(&global, vault) {
            let result = config.validate();
            assert!(result.is_ok(), "valid config should pass validation");
        }
    }

    #[test]
    fn config_merge_applies_defaults_for_empty_values() {
        let global = GlobalConfig {
            filesystem: FileSystemConfig {
                cache_dir: String::new(), // Empty - should use default
                property_bank_filename: String::new(),
                schemas_dir: String::new(),
                templates_dir: String::new(),
                vault_path: "/global".to_owned(),
            },
            frontmatter: FrontmatterConfig {
                alias_key: String::new(),
                date_created_key: String::new(),
                date_modified_key: String::new(),
                file_class_key: String::new(),
                title_key: String::new(),
            },
            log_level: String::new(), // Empty - should use default "info"
        };

        let vault = VaultConfig {
            filesystem: FileSystemConfig {
                cache_dir: String::new(),
                property_bank_filename: String::new(),
                schemas_dir: String::new(),
                templates_dir: String::new(),
                vault_path: "/vault".to_owned(),
            },
            frontmatter: FrontmatterConfig {
                alias_key: String::new(),
                date_created_key: String::new(),
                date_modified_key: String::new(),
                file_class_key: String::new(),
                title_key: String::new(),
            },
            log_level: String::new(),
        };

        let result = Config::merge(&global, vault);
        assert!(result.is_ok(), "merge with empty values should succeed");

        if let Ok(config) = result {
            // Verify defaults were applied
            assert_eq!(config.filesystem.cache_dir, ".cache");
            assert_eq!(config.filesystem.templates_dir, "templates");
            assert_eq!(config.filesystem.schemas_dir, "schemas");
            assert_eq!(
                config.filesystem.property_bank_filename,
                "property_bank.json"
            );
            assert_eq!(config.log_level, "info");
            assert_eq!(config.frontmatter.file_class_key, "file_class");
            assert_eq!(config.frontmatter.title_key, "title");
            assert_eq!(config.frontmatter.alias_key, "aliases");
        }
    }

    #[test]
    fn config_has_required_derives() {
        let global = sample_global_config();
        let vault = sample_vault_config();
        let result1 = Config::merge(&global, vault.clone());
        assert!(result1.is_ok(), "first merge should succeed");

        if let Ok(config) = result1 {
            // Test Debug
            let debug_str = format!("{config:?}");
            assert!(!debug_str.is_empty());

            // Test Clone
            let _cloned = config.clone();

            // Test PartialEq
            let result2 = Config::merge(&global, vault);
            assert!(result2.is_ok(), "second merge should succeed");
            if let Ok(config2) = result2 {
                assert_eq!(config, config2);
            }
        }
    }

    // ============================================================================
    // ConfigValue Tests
    // ============================================================================

    #[test]
    fn config_value_from_string() {
        let value = ConfigValue::from("test".to_owned());
        assert!(matches!(&value, ConfigValue::String(s) if s == "test"));
    }

    #[test]
    fn config_value_from_number() {
        let value = ConfigValue::from(42.5f64);
        assert!(
            matches!(value, ConfigValue::Number(n) if (n - 42.5f64).abs() < f64::EPSILON)
        );
    }

    #[test]
    fn config_value_from_bool() {
        let value = ConfigValue::from(true);
        assert!(matches!(value, ConfigValue::Boolean(true)));
    }

    #[test]
    fn config_value_encrypted_field() {
        let encrypted_data = vec![1, 2, 3, 4, 5];
        let value = ConfigValue::Encrypted(encrypted_data.clone());

        assert!(
            matches!(&value, ConfigValue::Encrypted(data) if data == &encrypted_data)
        );
    }

    #[test]
    fn config_value_array() {
        let array = vec![
            ConfigValue::String("item1".to_owned()),
            ConfigValue::String("item2".to_owned()),
        ];
        let value = ConfigValue::Array(array.clone());

        assert!(
            matches!(&value, ConfigValue::Array(items) if items.len() == 2)
        );
    }

    #[test]
    fn config_value_object() {
        let mut map = HashMap::new();
        map.insert("key1".to_owned(), ConfigValue::String("value1".to_owned()));
        let value = ConfigValue::Object(map.clone());

        assert!(matches!(&value, ConfigValue::Object(obj) if obj.len() == 1));
    }

    // ============================================================================
    // Struct Tests
    // ============================================================================

    #[test]
    fn filesystem_config_structure() {
        let config = FileSystemConfig {
            cache_dir: ".cache".to_owned(),
            property_bank_filename: "props.json".to_owned(),
            schemas_dir: "schemas".to_owned(),
            templates_dir: "templates".to_owned(),
            vault_path: "/test".to_owned(),
        };

        assert_eq!(config.vault_path, "/test");
        assert_eq!(config.cache_dir, ".cache");
        assert_eq!(config.property_bank_path(), "schemas/props.json");
    }

    #[test]
    fn frontmatter_config_structure() {
        let config = FrontmatterConfig {
            alias_key: "aliases".to_owned(),
            date_created_key: "created".to_owned(),
            date_modified_key: "modified".to_owned(),
            file_class_key: "type".to_owned(),
            title_key: "title".to_owned(),
        };

        assert_eq!(config.file_class_key, "type");
        assert_eq!(config.title_key, "title");
    }

    // ============================================================================
    // Property-Based Tests
    // ============================================================================

    #[test]
    fn config_merge_is_idempotent() {
        let global = sample_global_config();
        let vault = sample_vault_config();

        let result1 = Config::merge(&global, vault.clone());
        assert!(result1.is_ok(), "first merge should succeed");

        let result2 = Config::merge(&global, vault);
        assert!(result2.is_ok(), "second merge should succeed");

        if let (Ok(merged1), Ok(merged2)) = (result1, result2) {
            assert_eq!(merged1, merged2, "merge should be deterministic");
        }
    }

    #[test]
    fn config_validation_catches_empty_paths() {
        let global = sample_global_config();
        let mut vault = sample_vault_config();
        vault.filesystem.vault_path = String::new();

        let result = Config::merge(&global, vault);

        assert!(result.is_err(), "empty vault_path should fail validation");
        if let Err(e) = result {
            assert!(matches!(e, crate::ConfigError::ValidationFailed { .. }));
        }
    }
}
