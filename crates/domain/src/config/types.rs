//! Configuration types shared between vault and global contexts.
//!
//! This module contains the fundamental configuration types that are used
//! by both vault and global configuration contexts.

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

/// Convert String to `Value::String` variant.
impl From<String> for SettingValue {
    #[inline]
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

/// Convert f64 to `Value::Number` variant.
impl From<f64> for SettingValue {
    #[inline]
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

/// Convert bool to `Value::Boolean` variant.
impl From<bool> for SettingValue {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

/// Convert Vec<ConfigValue> to `Value::Array` variant.
impl From<Vec<SettingValue>> for SettingValue {
    #[inline]
    fn from(value: Vec<SettingValue>) -> Self {
        Self::Array(value)
    }
}

/// Convert `HashMap`<String, `SettingValue`> to `SettingValue::Object` variant.
impl From<HashMap<String, SettingValue>> for SettingValue {
    #[inline]
    fn from(value: HashMap<String, SettingValue>) -> Self {
        Self::Object(value)
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

impl Default for Frontmatter {
    #[inline]
    fn default() -> Self {
        Self {
            alias_key: "aliases".to_owned(),
            date_created_key: "date_created".to_owned(),
            date_modified_key: "date_modified".to_owned(),
            file_class_key: "file_class".to_owned(),
            title_key: "title".to_owned(),
        }
    }
}

impl Frontmatter {
    /// Validate frontmatter configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::ValidationFailed` if any frontmatter key is empty.
    #[inline]
    pub fn validate(&self) -> Result<(), crate::ConfigError> {
        let fields = [
            ("alias_key", &self.alias_key),
            ("date_created_key", &self.date_created_key),
            ("date_modified_key", &self.date_modified_key),
            ("file_class_key", &self.file_class_key),
            ("title_key", &self.title_key),
        ];

        for (name, value) in fields {
            if value.is_empty() {
                return Err(crate::ConfigError::ValidationFailed {
                    field: name.to_owned(),
                    message: format!("{name} cannot be empty"),
                });
            }
        }

        Ok(())
    }
}

/// Logging configuration with validation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Logging {
    /// Log level (debug, info, warn, error).
    pub log_level: String,
}

impl Default for Logging {
    #[inline]
    fn default() -> Self {
        Self {
            log_level: "info".to_owned(),
        }
    }
}

impl Logging {
    /// Validate logging configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::InvalidEnumValue` if `log_level` is not one of: debug, info, warn, error.
    #[inline]
    pub fn validate(&self) -> Result<(), crate::ConfigError> {
        let valid_levels = ["debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.log_level.as_str()) {
            return Err(crate::ConfigError::InvalidEnumValue {
                field: "log_level".to_owned(),
                value: self.log_level.clone(),
                allowed: valid_levels
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect(),
            });
        }
        Ok(())
    }
}

/// Schema configuration (schemas directory).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Schema {
    /// Property bank filename (stored in `schemas_dir`).
    pub property_bank_filename: String,
    /// Directory containing schema files.
    pub schemas_dir: String,
}

impl Default for Schema {
    #[inline]
    fn default() -> Self {
        Self {
            schemas_dir: "schemas".to_owned(),
            property_bank_filename: "property_bank.json".to_owned(),
        }
    }
}

impl Schema {
    /// Get the full path to the property bank file (`schemas_dir/property_bank_filename`).
    ///
    /// The property bank is always stored in the schemas directory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use lithos_domain::SchemaConfig;
    /// let schema = SchemaConfig::default();
    ///
    /// assert_eq!(schema.property_bank_path(), "schemas/property_bank.json");
    /// ```
    #[inline]
    #[must_use]
    pub fn property_bank_path(&self) -> String {
        format!("{}/{}", self.schemas_dir, self.property_bank_filename)
    }

    /// Validate schema configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::ValidationFailed` if `schemas_dir` or `property_bank_filename` is empty.
    #[inline]
    pub fn validate(&self) -> Result<(), crate::ConfigError> {
        if self.schemas_dir.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "schemas_dir".to_owned(),
                message: "schemas directory cannot be empty".to_owned(),
            });
        }
        if self.property_bank_filename.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "property_bank_filename".to_owned(),
                message: "property bank filename cannot be empty".to_owned(),
            });
        }
        Ok(())
    }
}

/// Template configuration (templates directory).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Template {
    /// Directory containing template files.
    pub templates_dir: String,
}

impl Default for Template {
    #[inline]
    fn default() -> Self {
        Self {
            templates_dir: "templates".to_owned(),
        }
    }
}

impl Template {
    /// Validate template configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::ValidationFailed` if `templates_dir` is empty.
    #[inline]
    pub fn validate(&self) -> Result<(), crate::ConfigError> {
        if self.templates_dir.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "templates_dir".to_owned(),
                message: "templates directory cannot be empty".to_owned(),
            });
        }
        Ok(())
    }
}
