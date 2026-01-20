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
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::FrontmatterConfig;
    /// let frontmatter = FrontmatterConfig::default();
    /// frontmatter.validate().unwrap();
    /// ```
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
                    field: name.to_owned().into(),
                    message: format!("{name} cannot be empty").into(),
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
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::LoggingConfig;
    /// let logging = LoggingConfig::default();
    /// logging.validate().unwrap();
    /// ```
    #[inline]
    pub fn validate(&self) -> Result<(), crate::ConfigError> {
        let valid_levels = ["debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.log_level.as_str()) {
            return Err(crate::ConfigError::InvalidEnumValue {
                field: "log_level".to_owned().into(),
                value: self.log_level.clone().into(),
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
                field: "schemas_dir".to_owned().into(),
                message: "schemas directory cannot be empty".to_owned().into(),
            });
        }
        if self.property_bank_filename.is_empty() {
            return Err(crate::ConfigError::ValidationFailed {
                field: "property_bank_filename".to_owned().into(),
                message: "property bank filename cannot be empty"
                    .to_owned()
                    .into(),
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
                field: "templates_dir".to_owned().into(),
                message: "templates directory cannot be empty"
                    .to_owned()
                    .into(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Frontmatter, Logging, Schema, SettingValue, Template};

    /// 3.3-UNIT-022: `converts_from_string`.
    /// Priority: P3.
    #[test]
    fn converts_from_string() {
        // GIVEN a string value for configuration
        let input = "test".to_owned();

        // WHEN converting into a SettingValue
        let value = SettingValue::from(input.clone());

        // THEN the string variant is produced
        assert_eq!(
            value,
            SettingValue::String(input),
            "Conversion from String to SettingValue failed"
        );
    }

    /// 3.3-UNIT-023: `converts_from_f64`.
    /// Priority: P3.
    #[test]
    fn converts_from_f64() {
        // GIVEN a floating point value for configuration
        let input = 42.5f64;

        // WHEN converting into a SettingValue
        let value = SettingValue::from(input);

        // THEN the number variant is produced
        assert_eq!(
            value,
            SettingValue::Number(42.5f64),
            "Conversion from f64 to SettingValue failed"
        );
    }

    /// 3.3-UNIT-024: `converts_from_bool`.
    /// Priority: P3.
    #[test]
    fn converts_from_bool() {
        // GIVEN a boolean configuration value
        let input = true;

        // WHEN converting into a SettingValue
        let value = SettingValue::from(input);

        // THEN the boolean variant is produced
        assert_eq!(
            value,
            SettingValue::Boolean(true),
            "Conversion from bool to SettingValue failed"
        );
    }

    /// 3.3-UNIT-025: `stores_opaque_encrypted_bytes`.
    /// Priority: P1.
    #[test]
    fn stores_opaque_encrypted_bytes() {
        // GIVEN encrypted data
        let raw = vec![1, 2, 3, 4];

        // WHEN creating an encrypted config value
        let value = SettingValue::Encrypted(raw.clone());

        // THEN debug output must not reveal raw bytes
        let debug = format!("{value:?}");
        assert!(debug.contains("***"), "Expected masked debug output");
    }

    /// 3.3-UNIT-026: `frontmatter_validate_rejects_empty_keys`.
    /// Priority: P0.
    #[test]
    fn frontmatter_validate_rejects_empty_keys() {
        // GIVEN frontmatter with empty key
        let frontmatter = Frontmatter {
            title_key: String::new(),
            ..Frontmatter::default()
        };

        // WHEN validating
        let result = frontmatter.validate();

        // THEN it fails
        assert!(result.is_err());
    }

    /// 3.3-UNIT-027: `logging_rejects_invalid_levels`.
    /// Priority: P0.
    #[test]
    fn logging_rejects_invalid_levels() {
        // GIVEN an invalid log level
        let logging = Logging {
            log_level: "verbose".to_owned(),
        };

        // WHEN validating
        let result = logging.validate();

        // THEN it fails with invalid enum
        assert!(matches!(
            result,
            Err(crate::ConfigError::InvalidEnumValue { .. })
        ));
    }

    /// 3.3-UNIT-028: `schema_validate_rejects_empty_paths`.
    /// Priority: P0.
    #[test]
    fn schema_validate_rejects_empty_paths() {
        // GIVEN schema config with empty fields
        let schema = Schema {
            schemas_dir: String::new(),
            property_bank_filename: String::new(),
        };

        // WHEN validating
        let result = schema.validate();

        // THEN validation fails
        assert!(result.is_err());
    }

    /// 3.3-UNIT-029: `stores_nested_arrays`.
    /// Priority: P2.
    #[test]
    fn stores_nested_arrays() {
        // GIVEN nested configuration values
        let array = vec![
            SettingValue::String("item1".to_owned()),
            SettingValue::String("item2".to_owned()),
        ];

        // WHEN storing them in an array variant
        let value = SettingValue::Array(array.clone());

        // THEN the array preserves the nested values
        assert_eq!(
            value,
            SettingValue::Array(array),
            "Array variant should store nested SettingValues"
        );
    }

    /// 3.3-UNIT-030: `stores_nested_objects`.
    /// Priority: P2.
    #[test]
    fn stores_nested_objects() {
        // GIVEN nested configuration values in a map
        let mut map = std::collections::HashMap::new();
        map.insert(
            "key1".to_owned(),
            SettingValue::String("value1".to_owned()),
        );

        // WHEN storing them in an object variant
        let value = SettingValue::Object(map.clone());

        // THEN the map preserves the nested values
        assert_eq!(
            value,
            SettingValue::Object(map),
            "Object variant should store HashMap of SettingValues"
        );
    }

    /// 3.3-UNIT-031: `converts_from_vector_of_values`.
    /// Priority: P3.
    #[test]
    fn converts_from_vector_of_values() {
        // GIVEN a vector of configuration values
        let array = vec![
            SettingValue::String("item1".to_owned()),
            SettingValue::Number(42.0),
        ];

        // WHEN converting into a SettingValue
        let value = SettingValue::from(array.clone());

        // THEN the array variant is produced
        assert_eq!(
            value,
            SettingValue::Array(array),
            "From<Vec<SettingValue>> conversion failed"
        );
    }

    /// 3.3-UNIT-032: `converts_from_hashmap_of_values`.
    /// Priority: P3.
    #[test]
    fn converts_from_hashmap_of_values() {
        // GIVEN a hashmap of configuration values
        let mut map = std::collections::HashMap::new();
        map.insert(
            "key1".to_owned(),
            SettingValue::String("value1".to_owned()),
        );

        // WHEN converting into a SettingValue
        let value = SettingValue::from(map.clone());

        // THEN the object variant is produced
        assert_eq!(
            value,
            SettingValue::Object(map),
            "From<HashMap<String, SettingValue>> conversion failed"
        );
    }

    /// 3.3-UNIT-033: `masks_encrypted_variant_in_debug_logs`.
    /// Priority: P1.
    #[test]
    fn masks_encrypted_variant_in_debug_logs() {
        // GIVEN an encrypted configuration value
        let val = SettingValue::Encrypted(vec![1, 2, 3]);

        // WHEN formatting for debug output
        let debug_str = format!("{val:?}");

        // THEN the raw bytes are masked
        assert!(
            !debug_str.contains("1, 2, 3"),
            "Debug output must not contain raw encrypted bytes"
        );
        assert!(
            debug_str.contains("***"),
            "Debug output must contain mask characters"
        );
    }

    /// 3.3-UNIT-034: `constructs_valid_property_bank_path`.
    /// Priority: P1.
    #[test]
    fn constructs_valid_property_bank_path() {
        // GIVEN schema configuration with explicit paths
        let schema = Schema {
            schemas_dir: "schemas".to_owned(),
            property_bank_filename: "props.json".to_owned(),
        };

        // WHEN deriving the property bank path
        let path = schema.property_bank_path();

        // THEN the path is composed from schema settings
        assert_eq!(
            path, "schemas/props.json",
            "property_bank_path logic works"
        );
    }

    /// 3.3-UNIT-035: `preserves_frontmatter_key_mappings`.
    /// Priority: P1.
    #[test]
    fn preserves_frontmatter_key_mappings() {
        // GIVEN explicit frontmatter mappings
        let config = Frontmatter {
            alias_key: "aliases".to_owned(),
            date_created_key: "created".to_owned(),
            date_modified_key: "modified".to_owned(),
            file_class_key: "type".to_owned(),
            title_key: "title".to_owned(),
        };

        // WHEN accessing the key mappings
        let file_class = &config.file_class_key;
        let title = &config.title_key;

        // THEN the mappings match the configured values
        assert_eq!(file_class, "type", "file_class_key mapping mismatch");
        assert_eq!(title, "title", "title_key mapping mismatch");
    }

    /// 3.3-UNIT-036: `rejects_empty_templates_dir`.
    /// Priority: P0.
    #[test]
    fn rejects_empty_templates_dir() {
        // GIVEN a template config with an empty templates_dir
        let template = Template {
            templates_dir: String::new(),
        };

        // WHEN validating the template configuration
        let result = template.validate();

        // THEN the validation fails with the templates_dir field
        assert!(
            result.is_err(),
            "Expected validation failure for empty templates_dir"
        );
        if let Err(crate::ConfigError::ValidationFailed {
            field,
            ..
        }) = result
        {
            assert_eq!(
                field.as_ref(),
                "templates_dir",
                "Expected templates_dir validation failure"
            );
        }
    }
}
