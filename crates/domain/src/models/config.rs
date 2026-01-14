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
///
/// ```rust
/// # use lithos_domain::ConfigValue;
/// # use std::collections::HashMap;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create from primitives
/// let string_val = ConfigValue::from("test".to_string());
/// let number_val = ConfigValue::from(42.0);
/// let bool_val = ConfigValue::from(true);
///
/// // Create complex nested structures
/// let mut obj_map = HashMap::new();
/// obj_map.insert("key".to_string(), string_val);
/// let object_val = ConfigValue::from(obj_map);
///
/// let array_val = ConfigValue::from(vec![number_val, bool_val]);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum SettingValue {
    /// Array of configuration values.
    Array(Vec<SettingValue>),
    /// Boolean configuration value.
    Boolean(bool),
    /// Encrypted field data (opaque bytes, adapter handles encryption/decryption).
    Encrypted(Vec<u8>),
    /// Numeric configuration value (f64 for flexibility).
    Number(f64),
    /// Nested object configuration.
    Object(HashMap<String, SettingValue>),
    /// String configuration value.
    String(String),
}

impl std::fmt::Debug for SettingValue {
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics are preferred here for clarity, and pattern_type_mismatch is overly pedantic for this Debug implementation."
    )]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Array(arr) => f.debug_tuple("Array").field(arr).finish(),
            Self::Boolean(b) => f.debug_tuple("Boolean").field(b).finish(),
            Self::Encrypted(_) => {
                f.debug_tuple("Encrypted").field(&"***").finish()
            }
            Self::Number(n) => f.debug_tuple("Number").field(n).finish(),
            Self::Object(map) => f.debug_tuple("Object").field(map).finish(),
            Self::String(s) => f.debug_tuple("String").field(s).finish(),
        }
    }
}

/// Filesystem-related configuration settings.
///
/// # Invariants
/// - `vault_path` must be an absolute path (validated by Config).
/// - All directory paths should end without trailing slash.
/// - Property bank file is always located in `schemas_dir`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct FileSystem {
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

impl FileSystem {
    /// Get the full path to the property bank file (`schemas_dir/property_bank_filename`).
    ///
    /// The property bank is always stored in the schemas directory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lithos_domain::FileSystemConfig;
    /// let mut config = FileSystemConfig::default();
    /// config.schemas_dir = "schemas".to_string();
    /// config.property_bank_filename = "props.json".to_string();
    ///
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
pub struct Frontmatter {
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
#[derive(
    Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct Vault {
    /// Filesystem configuration for vault.
    pub filesystem: FileSystem,
    /// Frontmatter configuration for vault.
    pub frontmatter: Frontmatter,
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
pub struct Global {
    /// Filesystem configuration for global defaults.
    pub filesystem: FileSystem,
    /// Frontmatter configuration for global defaults.
    pub frontmatter: Frontmatter,
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
///
/// ```rust
/// # use lithos_domain::{Config, GlobalConfig, VaultConfig, FileSystemConfig};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let global = GlobalConfig::default();
/// let mut vault = VaultConfig::default();
/// vault.filesystem.vault_path = "/vault".to_string();
///
/// let config = Config::merge(&global, vault)?;
/// assert_eq!(config.filesystem.vault_path, "/vault");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Config {
    /// Merged filesystem configuration.
    pub filesystem: FileSystem,
    /// Merged frontmatter configuration.
    pub frontmatter: Frontmatter,
    /// Log level (debug, info, warn, error).
    pub log_level: String,
}

impl Default for FileSystem {
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

impl Default for Frontmatter {
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

impl Default for Global {
    #[inline]
    fn default() -> Self {
        Self {
            filesystem: FileSystem::default(),
            frontmatter: Frontmatter::default(),
            log_level: defaults::logging::LOG_LEVEL.to_owned(),
        }
    }
}

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
        global: &Global,
        vault: Vault,
    ) -> Result<Self, crate::ConfigError> {
        // Step 1: Pre-validate required Vault Path
        Self::validate_vault_path(&vault.filesystem.vault_path)?;

        // Step 2: Merge values with precedence
        let filesystem =
            Self::merge_filesystem(&global.filesystem, vault.filesystem);

        let frontmatter =
            Self::merge_frontmatter(&global.frontmatter, &vault.frontmatter);

        let log_level =
            Self::merge_log_level(&global.log_level, &vault.log_level)?;

        // Step 3: Construct the final strictly-validated aggregate
        let config = Self {
            filesystem,
            frontmatter,
            log_level,
        };

        // Step 4: Final invariant check
        config.validate_internal()?;

        Ok(config)
    }

    /// Choose value with precedence: vault > global > default.
    #[inline]
    #[must_use]
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
    fn merge_filesystem(global: &FileSystem, vault: FileSystem) -> FileSystem {
        let defaults = FileSystem::default();

        FileSystem {
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
        }
    }

    /// Merge frontmatter configurations applying defaults where needed.
    fn merge_frontmatter(
        global: &Frontmatter,
        vault: &Frontmatter,
    ) -> Frontmatter {
        let defaults = Frontmatter::default();

        Frontmatter {
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

    /// Validate configuration against critical business rules (internal check).
    fn validate_internal(&self) -> Result<(), crate::ConfigError> {
        // Step 1: Validate Filesystem Completeness
        let filesystem_fields = [
            ("cache_dir", &self.filesystem.cache_dir),
            ("property_bank_filename", &self.filesystem.property_bank_filename),
            ("schemas_dir", &self.filesystem.schemas_dir),
            ("templates_dir", &self.filesystem.templates_dir),
            ("vault_path", &self.filesystem.vault_path),
        ];

        Self::validate_fields(&filesystem_fields)?;

        // Step 2: Validate Frontmatter Completeness
        let metadata_fields = [
            ("alias_key", &self.frontmatter.alias_key),
            ("date_created_key", &self.frontmatter.date_created_key),
            ("date_modified_key", &self.frontmatter.date_modified_key),
            ("file_class_key", &self.frontmatter.file_class_key),
            ("title_key", &self.frontmatter.title_key),
        ];

        Self::validate_fields(&metadata_fields)?;

        // Step 3: Validate Log Level
        Self::validate_log_level(&self.log_level)?;

        Ok(())
    }

    /// Validate that all fields in the provided slice are non-empty.
    ///
    /// # Errors
    /// Returns `ConfigError::ValidationFailed` if any field is empty.
    fn validate_fields(
        fields: &[(&str, &String)],
    ) -> Result<(), crate::ConfigError> {
        for &(name, value) in fields {
            if value.is_empty() {
                return Err(crate::ConfigError::ValidationFailed {
                    field: (*name).to_owned(),
                    message: format!("{name} cannot be empty after merge"),
                });
            }
        }
        Ok(())
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lithos_domain::{Config, GlobalConfig, VaultConfig};
    /// // Create a valid config via merge
    /// let global = GlobalConfig::default();
    /// let mut vault = VaultConfig::default();
    /// vault.filesystem.vault_path = "/vault".to_string();
    ///
    /// let mut config = Config::merge(&global, vault).unwrap();
    /// assert!(config.validate().is_ok());
    ///
    /// // Make it invalid
    /// config.filesystem.vault_path = String::new();
    /// assert!(config.validate().is_err());
    /// ```
    #[inline]
    pub fn validate(&self) -> Result<(), crate::ConfigError> {
        Self::validate_vault_path(&self.filesystem.vault_path)?;
        Self::validate_log_level(&self.log_level)?;

        Ok(())
    }
}

/// Convert String to `Value::String` variant.
///
/// # Examples
/// ```
/// use lithos_domain::ConfigValue;
///
/// let value = ConfigValue::from("test".to_string());
/// assert_eq!(value, ConfigValue::String("test".to_string()));
/// ```
impl From<String> for SettingValue {
    #[inline]
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

/// Convert f64 to `Value::Number` variant.
///
/// # Examples
/// ```
/// use lithos_domain::ConfigValue;
///
/// let value = ConfigValue::from(42.5);
/// assert_eq!(value, ConfigValue::Number(42.5));
/// ```
impl From<f64> for SettingValue {
    #[inline]
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

/// Convert bool to `Value::Boolean` variant.
///
/// # Examples
/// ```
/// use lithos_domain::ConfigValue;
///
/// let value = ConfigValue::from(true);
/// assert_eq!(value, ConfigValue::Boolean(true));
/// ```
impl From<bool> for SettingValue {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

/// Convert Vec<ConfigValue> to `Value::Array` variant.
///
/// # Examples
/// ```
/// use lithos_domain::ConfigValue;
///
/// let array = vec![ConfigValue::String("item".to_string())];
/// let value = ConfigValue::from(array.clone());
/// assert_eq!(value, ConfigValue::Array(array));
/// ```
impl From<Vec<SettingValue>> for SettingValue {
    #[inline]
    fn from(value: Vec<SettingValue>) -> Self {
        Self::Array(value)
    }
}

/// Convert `HashMap`<String, `SettingValue`> to `SettingValue::Object` variant.
///
/// # Examples
/// ```
/// use lithos_domain::ConfigValue;
/// use std::collections::HashMap;
///
/// let mut map = HashMap::new();
/// map.insert("key".to_string(), ConfigValue::String("value".to_string()));
/// let value = ConfigValue::from(map.clone());
/// assert_eq!(value, ConfigValue::Object(map));
/// ```
impl From<HashMap<String, SettingValue>> for SettingValue {
    #[inline]
    fn from(value: HashMap<String, SettingValue>) -> Self {
        Self::Object(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Config::merge Tests
    // ============================================================================

    mod merge {
        use super::*;

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test expects merge to succeed, unwrap is appropriate for test clarity"
        )]
        fn vault_values_take_precedence_over_global() {
            // Mitigates R-008: Config Merge Logic
            let global = sample_global_config();
            let vault = sample_vault_config();

            let merged = Config::merge(&global, vault).unwrap();

            // Business rule: vault values take precedence
            assert_eq!(
                merged.filesystem.vault_path, "/vault",
                "Vault path should override global"
            );
            assert_eq!(
                merged.filesystem.templates_dir, "custom_templates",
                "Templates directory should override global"
            );
            assert_eq!(
                merged.log_level, "debug",
                "Log level should override global"
            );
            assert_eq!(
                merged.frontmatter.file_class_key, "type",
                "File class key should override global"
            );
            assert_eq!(
                merged.frontmatter.date_created_key, "created",
                "Date created key should override global"
            );
        }

        #[test]
        fn falls_back_to_defaults_when_inputs_are_empty() {
            let global = Global {
                filesystem: FileSystem {
                    cache_dir: String::new(), // Empty - should use default
                    property_bank_filename: String::new(),
                    schemas_dir: String::new(),
                    templates_dir: String::new(),
                    vault_path: "/global".to_owned(),
                },
                frontmatter: Frontmatter {
                    alias_key: String::new(),
                    date_created_key: String::new(),
                    date_modified_key: String::new(),
                    file_class_key: String::new(),
                    title_key: String::new(),
                },
                log_level: String::new(), // Empty - should use default "info"
            };

            let vault = Vault {
                filesystem: FileSystem {
                    cache_dir: String::new(),
                    property_bank_filename: String::new(),
                    schemas_dir: String::new(),
                    templates_dir: String::new(),
                    vault_path: "/vault".to_owned(),
                },
                frontmatter: Frontmatter {
                    alias_key: String::new(),
                    date_created_key: String::new(),
                    date_modified_key: String::new(),
                    file_class_key: String::new(),
                    title_key: String::new(),
                },
                log_level: String::new(),
            };

            let result = Config::merge(&global, vault);
            assert!(
                result.is_ok(),
                "Merge with empty values should succeed, got: {result:?}"
            );

            if let Ok(config) = result {
                // Verify defaults were applied
                assert_eq!(
                    config.filesystem.cache_dir, ".cache",
                    "Should fall back to default cache_dir"
                );
                assert_eq!(
                    config.filesystem.templates_dir, "templates",
                    "Should fall back to default templates_dir"
                );
                assert_eq!(
                    config.filesystem.schemas_dir, "schemas",
                    "Should fall back to default schemas_dir"
                );
                assert_eq!(
                    config.filesystem.property_bank_filename,
                    "property_bank.json",
                    "Should fall back to default property_bank_filename"
                );
                assert_eq!(
                    config.log_level, "info",
                    "Should fall back to default log_level"
                );
                assert_eq!(
                    config.frontmatter.file_class_key, "file_class",
                    "Should fall back to default file_class_key"
                );
                assert_eq!(
                    config.frontmatter.title_key, "title",
                    "Should fall back to default title_key"
                );
                assert_eq!(
                    config.frontmatter.alias_key, "aliases",
                    "Should fall back to default alias_key"
                );
            }
        }

        #[test]
        fn merge_is_idempotent() {
            let global = sample_global_config();
            let vault = sample_vault_config();

            let result1 = Config::merge(&global, vault.clone());
            assert!(result1.is_ok(), "First merge should succeed");

            let result2 = Config::merge(&global, vault);
            assert!(result2.is_ok(), "Second merge should succeed");

            if let (Ok(merged1), Ok(merged2)) = (result1, result2) {
                assert_eq!(
                    merged1, merged2,
                    "Repeated merges with same input must yield identical output"
                );
            }
        }
    }

    // ============================================================================
    // Config::validate Tests
    // ============================================================================

    mod validate {
        use rstest::rstest;

        use super::*;

        #[rstest]
        #[case::valid_config("/vault", "info", None)]
        #[case::empty_path("", "info", Some("vault_path"))]
        #[case::invalid_log_level("/vault", "invalid", Some("log_level"))]
        fn enforces_required_fields_and_enum_constraints(
            #[case] path: &str,
            #[case] level: &str,
            #[case] expected_error_field: Option<&str>,
        ) {
            let global = sample_global_config();
            let mut vault = sample_vault_config();
            vault.filesystem.vault_path = path.to_owned();
            vault.log_level = level.to_owned();

            let result = Config::merge(&global, vault);

            match expected_error_field {
                None => {
                    assert!(
                        result.is_ok(),
                        "Configuration with path='{path}' and level='{level}' should be valid, but failed: {result:?}"
                    );
                    if let Ok(config) = result {
                        assert!(
                            config.validate().is_ok(),
                            "Explicit validate() call should also pass for valid config"
                        );
                    }
                }
                Some(field_name) => {
                    let err =
                        result.expect_err("Validation should have failed");

                    // # LINT_DISABLE_REASON: Wildcard match is necessary for test resilience against new error variants. Panic is standard for test failure.
                    #[expect(
                        clippy::wildcard_enum_match_arm,
                        clippy::panic,
                        reason = "Test safety boundary"
                    )]
                    match err {
                        crate::ConfigError::ValidationFailed {
                            field,
                            ..
                        }
                        | crate::ConfigError::InvalidEnumValue {
                            field,
                            ..
                        } => {
                            assert_eq!(
                                field, field_name,
                                "Error reported for wrong field"
                            );
                        }
                        _ => {
                            panic!(
                                "Expected a validation-related error, found: {err:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    // ============================================================================
    // ConfigValue Tests
    // ============================================================================

    mod config_value {
        use SettingValue as ConfigValue;

        use super::*;

        #[test]
        fn converts_from_string() {
            let value = ConfigValue::from("test".to_owned());
            assert_eq!(
                value,
                ConfigValue::String("test".to_owned()),
                "Conversion from String to ConfigValue failed"
            );
        }

        #[test]
        fn converts_from_f64() {
            let value = ConfigValue::from(42.5f64);
            assert_eq!(
                value,
                ConfigValue::Number(42.5f64),
                "Conversion from f64 to ConfigValue failed"
            );
        }

        #[test]
        fn converts_from_bool() {
            let value = ConfigValue::from(true);
            assert_eq!(
                value,
                ConfigValue::Boolean(true),
                "Conversion from bool to ConfigValue failed"
            );
        }

        #[test]
        fn stores_opaque_encrypted_bytes() {
            let encrypted_data = vec![1, 2, 3, 4, 5];
            let value = ConfigValue::Encrypted(encrypted_data.clone());

            assert_eq!(
                value,
                ConfigValue::Encrypted(encrypted_data),
                "Encrypted variant should store raw bytes correctly"
            );
        }

        #[test]
        fn stores_nested_arrays() {
            let array = vec![
                ConfigValue::String("item1".to_owned()),
                ConfigValue::String("item2".to_owned()),
            ];
            let value = ConfigValue::Array(array.clone());

            assert_eq!(
                value,
                ConfigValue::Array(array),
                "Array variant should store nested ConfigValues"
            );
        }

        #[test]
        fn stores_nested_objects() {
            let mut map = HashMap::new();
            map.insert(
                "key1".to_owned(),
                ConfigValue::String("value1".to_owned()),
            );
            let value = ConfigValue::Object(map.clone());

            assert_eq!(
                value,
                ConfigValue::Object(map),
                "Object variant should store HashMap of ConfigValues"
            );
        }

        #[test]
        fn converts_from_vector_of_values() {
            let array = vec![
                ConfigValue::String("item1".to_owned()),
                ConfigValue::Number(42.0),
            ];
            let value = ConfigValue::from(array.clone());

            assert_eq!(
                value,
                ConfigValue::Array(array),
                "From<Vec<ConfigValue>> conversion failed"
            );
        }

        #[test]
        fn converts_from_hashmap_of_values() {
            let mut map = HashMap::new();
            map.insert(
                "key1".to_owned(),
                ConfigValue::String("value1".to_owned()),
            );
            let value = ConfigValue::from(map.clone());

            assert_eq!(
                value,
                ConfigValue::Object(map),
                "From<HashMap<String, ConfigValue>> conversion failed"
            );
        }

        #[test]
        fn masks_encrypted_variant_in_debug_logs() {
            // Mitigates R-007: Encryption Exposure
            let val = ConfigValue::Encrypted(vec![1, 2, 3]);
            let debug_str = format!("{val:?}");
            assert!(
                !debug_str.contains("1, 2, 3"),
                "Debug output must not contain raw encrypted bytes"
            );
            assert!(
                debug_str.contains("***"),
                "Debug output must contain mask characters"
            );
        }
    }

    // ============================================================================
    // Structural Integrity Tests
    // ============================================================================

    mod integrity {
        use super::*;

        #[test]
        fn supports_clone_debug_and_partial_eq() {
            let global = sample_global_config();
            let vault = sample_vault_config();
            let result1 = Config::merge(&global, vault.clone());
            assert!(
                result1.is_ok(),
                "First merge for trait verification failed: {result1:?}"
            );

            if let Ok(config) = result1 {
                // Test Debug
                let debug_str = format!("{config:?}");
                assert!(
                    !debug_str.is_empty(),
                    "Debug derivation should produce non-empty string"
                );

                // Test Clone
                let cloned = config.clone();
                assert_eq!(
                    config, cloned,
                    "Cloned config must be equal to original"
                );

                // Test PartialEq
                let result2 = Config::merge(&global, vault);
                assert!(
                    result2.is_ok(),
                    "Second merge for trait verification failed"
                );
                if let Ok(config2) = result2 {
                    assert_eq!(
                        config, config2,
                        "Merged configs with identical input must be equal (PartialEq)"
                    );
                }
            }
        }

        #[test]
        fn constructs_valid_property_bank_path() {
            let config = FileSystem {
                cache_dir: ".cache".to_owned(),
                property_bank_filename: "props.json".to_owned(),
                schemas_dir: "schemas".to_owned(),
                templates_dir: "templates".to_owned(),
                vault_path: "/test".to_owned(),
            };

            assert_eq!(
                config.vault_path, "/test",
                "vault_path structure mismatch"
            );
            assert_eq!(
                config.cache_dir, ".cache",
                "cache_dir structure mismatch"
            );
            assert_eq!(
                config.property_bank_path(),
                "schemas/props.json",
                "property_bank_path logic mismatch"
            );
        }

        #[test]
        fn preserves_frontmatter_key_mappings() {
            let config = Frontmatter {
                alias_key: "aliases".to_owned(),
                date_created_key: "created".to_owned(),
                date_modified_key: "modified".to_owned(),
                file_class_key: "type".to_owned(),
                title_key: "title".to_owned(),
            };

            assert_eq!(
                config.file_class_key, "type",
                "file_class_key mapping mismatch"
            );
            assert_eq!(config.title_key, "title", "title_key mapping mismatch");
        }
    }

    /// Test fixture: Create sample global configuration with defaults.
    fn sample_global_config() -> Global {
        Global {
            filesystem: FileSystem {
                cache_dir: ".cache".to_owned(),
                property_bank_filename: "property_bank.json".to_owned(),
                schemas_dir: "schemas".to_owned(),
                templates_dir: "templates".to_owned(),
                vault_path: ".".to_owned(),
            },
            frontmatter: Frontmatter {
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
    fn sample_vault_config() -> Vault {
        Vault {
            filesystem: FileSystem {
                cache_dir: ".cache".to_owned(),
                property_bank_filename: "property_bank.json".to_owned(),
                schemas_dir: "schemas".to_owned(), // same as global
                templates_dir: "custom_templates".to_owned(),
                vault_path: "/vault".to_owned(),
            },
            frontmatter: Frontmatter {
                alias_key: "aliases".to_owned(),
                date_created_key: "created".to_owned(), // vault override
                date_modified_key: "modified".to_owned(), // vault override
                file_class_key: "type".to_owned(),      // vault override
                title_key: "title".to_owned(),
            },
            log_level: "debug".to_owned(), // vault override
        }
    }
}
