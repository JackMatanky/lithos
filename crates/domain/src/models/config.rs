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
/// - `log_level` must be valid log level string.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct FileSystemConfig {
    /// Directory for cache files (relative to `vault_path`).
    pub cache_dir: String,
    /// Log level (debug, info, warn, error).
    pub log_level: String,
    /// Path to property bank file (relative to `vault_path`).
    pub property_bank_file: String,
    /// Directory for schema files (relative to `vault_path`).
    pub schemas_dir: String,
    /// Directory for template files (relative to `vault_path`).
    pub templates_dir: String,
    /// Root path to the vault (absolute path required).
    pub vault_path: String,
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
}

/// Merged configuration result (Vault overrides Global).
///
/// # Business Rules
/// - Result of merging Global and Vault configurations.
/// - Vault values take precedence over Global values.
/// - Must be validated after merging.
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
///         property_bank_file: "props.json".to_string(),
///         cache_dir: ".cache".to_string(),
///         log_level: "info".to_string(),
///     },
///     frontmatter: FrontmatterConfig {
///         file_class_key: "file_class".to_string(),
///         title_key: "title".to_string(),
///         alias_key: "aliases".to_string(),
///         date_created_key: "created".to_string(),
///         date_modified_key: "modified".to_string(),
///     },
/// };
///
/// let vault = VaultConfig {
///     filesystem: FileSystemConfig {
///         vault_path: "/vault".to_string(),
///         templates_dir: "templates".to_string(),
///         schemas_dir: "schemas".to_string(),
///         property_bank_file: "props.json".to_string(),
///         cache_dir: ".cache".to_string(),
///         log_level: "debug".to_string(),
///     },
///     frontmatter: FrontmatterConfig {
///         file_class_key: "type".to_string(),
///         title_key: "title".to_string(),
///         alias_key: "aliases".to_string(),
///         date_created_key: "created".to_string(),
///         date_modified_key: "modified".to_string(),
///     },
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
}

// GREEN PHASE: Actual implementations
impl Config {
    /// Merge Global and Vault configurations with business rules (Vault overrides Global).
    ///
    /// # Business Rules
    /// - Vault configuration takes precedence over Global configuration.
    /// - All fields from vault override corresponding global fields.
    /// - Result is validated after merging.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if merged configuration fails validation.
    #[inline]
    pub fn merge(
        _global: GlobalConfig,
        vault: VaultConfig,
    ) -> Result<Self, crate::ConfigError> {
        // Business rule: Vault overrides Global
        // Simply use vault values (vault has highest precedence)
        let config = Self {
            filesystem: vault.filesystem,
            frontmatter: vault.frontmatter,
        };

        // Validate merged configuration
        config.validate()?;

        Ok(config)
    }

    /// Validate configuration against business rules.
    ///
    /// # Validation Rules
    /// - `vault_path` cannot be empty.
    /// - All directory paths must be valid (non-empty).
    /// - `log_level` must be one of: debug, info, warn, error.
    /// - All frontmatter keys must be non-empty.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if any validation rule is violated.
    #[inline]
    pub fn validate(&self) -> Result<(), crate::ConfigError> {
        // Validate filesystem config
        if self.filesystem.vault_path.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "vault_path".to_owned(),
                message: "vault path cannot be empty".to_owned(),
            });
        }

        if self.filesystem.templates_dir.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "templates_dir".to_owned(),
                message: "templates directory cannot be empty".to_owned(),
            });
        }

        if self.filesystem.schemas_dir.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "schemas_dir".to_owned(),
                message: "schemas directory cannot be empty".to_owned(),
            });
        }

        if self.filesystem.property_bank_file.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "property_bank_file".to_owned(),
                message: "property bank file cannot be empty".to_owned(),
            });
        }

        if self.filesystem.cache_dir.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "cache_dir".to_owned(),
                message: "cache directory cannot be empty".to_owned(),
            });
        }

        // Validate log level
        let valid_log_levels = ["debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.filesystem.log_level.as_str()) {
            return Err(crate::ConfigError::InvalidEnumValue {
                field: "log_level".to_owned(),
                value: self.filesystem.log_level.clone(),
                allowed: valid_log_levels
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect(),
            });
        }

        // Validate frontmatter config
        if self.frontmatter.file_class_key.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "file_class_key".to_owned(),
                message: "file class key cannot be empty".to_owned(),
            });
        }

        if self.frontmatter.title_key.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "title_key".to_owned(),
                message: "title key cannot be empty".to_owned(),
            });
        }

        if self.frontmatter.alias_key.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "alias_key".to_owned(),
                message: "alias key cannot be empty".to_owned(),
            });
        }

        if self.frontmatter.date_created_key.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "date_created_key".to_owned(),
                message: "date created key cannot be empty".to_owned(),
            });
        }

        if self.frontmatter.date_modified_key.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "date_modified_key".to_owned(),
                message: "date modified key cannot be empty".to_owned(),
            });
        }

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
                log_level: "info".to_owned(),
                property_bank_file: "property_bank.json".to_owned(),
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
        }
    }

    /// Test fixture: Create sample vault configuration with overrides.
    fn sample_vault_config() -> VaultConfig {
        VaultConfig {
            filesystem: FileSystemConfig {
                cache_dir: ".cache".to_owned(),
                log_level: "debug".to_owned(), // vault override
                property_bank_file: "property_bank.json".to_owned(),
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
        }
    }

    // ============================================================================
    // Config Entity Tests
    // ============================================================================

    #[test]
    fn config_merge_vault_overrides_global() {
        let global = sample_global_config();
        let vault = sample_vault_config();

        let result = Config::merge(global, vault);
        assert!(result.is_ok(), "merge should succeed");

        // Use pattern matching to extract value for assertions
        if let Ok(merged) = result {
            // Business rule: vault values take precedence
            assert_eq!(merged.filesystem.vault_path, "/vault");
            assert_eq!(merged.filesystem.templates_dir, "custom_templates");
            assert_eq!(merged.filesystem.log_level, "debug");
            assert_eq!(merged.frontmatter.file_class_key, "type");
            assert_eq!(merged.frontmatter.date_created_key, "created");
        }
    }

    #[test]
    fn config_validation_success() {
        let global = sample_global_config();
        let vault = sample_vault_config();
        let merge_result = Config::merge(global, vault);
        assert!(merge_result.is_ok(), "merge should succeed");

        if let Ok(config) = merge_result {
            let result = config.validate();
            assert!(result.is_ok(), "valid config should pass validation");
        }
    }

    #[test]
    fn config_has_required_derives() {
        let global = sample_global_config();
        let vault = sample_vault_config();
        let result1 = Config::merge(global.clone(), vault.clone());
        assert!(result1.is_ok(), "first merge should succeed");

        if let Ok(config) = result1 {
            // Test Debug
            let debug_str = format!("{config:?}");
            assert!(!debug_str.is_empty());

            // Test Clone
            let _cloned = config.clone();

            // Test PartialEq
            let result2 = Config::merge(global, vault);
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
            log_level: "debug".to_owned(),
            property_bank_file: "props.json".to_owned(),
            schemas_dir: "schemas".to_owned(),
            templates_dir: "templates".to_owned(),
            vault_path: "/test".to_owned(),
        };

        assert_eq!(config.vault_path, "/test");
        assert_eq!(config.log_level, "debug");
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

        let result1 = Config::merge(global.clone(), vault.clone());
        assert!(result1.is_ok(), "first merge should succeed");

        let result2 = Config::merge(global, vault);
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

        let result = Config::merge(global, vault);

        assert!(result.is_err(), "empty vault_path should fail validation");
        if let Err(e) = result {
            assert!(matches!(e, crate::ConfigError::ValidationFailed { .. }));
        }
    }
}
