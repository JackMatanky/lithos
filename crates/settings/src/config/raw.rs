//! Raw (serde) configuration input types (DTOs).
//!
//! This module defines raw config DTOs and supporting types used for
//! deserialization from TOML/YAML/JSON files before validation.

#![allow(
    missing_docs,
    reason = "Raw config DTOs mirror file schema; field docs pending."
)]

use std::collections::HashMap;

use traces_fs::metadata::FileMetadata;

/// Raw config parsed from the global config file.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawGlobalConfig {
    /// Logging configuration.
    pub logging: Option<RawLogging>,

    /// Template configuration.
    pub template: Option<RawTemplateConfig>,

    /// Schema configuration.
    pub schema: Option<RawSchemaConfig>,

    /// Trusted vaults (global-only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trusted_vaults: Option<RawTrustedVaults>,

    /// Frontmatter configuration.
    pub frontmatter: Option<RawFrontmatter>,

    /// Task configuration.
    pub task: Option<RawTaskConfig>,

    /// File metadata for staleness detection.
    ///
    /// Populated during discovery/parsing. Not serialized to TOML.
    #[serde(skip)]
    pub metadata: Option<FileMetadata>,
}

/// Raw config parsed from a vault config file.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawVaultConfig {
    /// Vault name override.
    pub name: Option<String>,

    /// Version override.
    pub version: Option<String>,

    /// Logging configuration.
    pub logging: Option<RawLogging>,

    /// Cache configuration.
    pub cache: Option<RawCacheConfig>,

    /// Template configuration.
    pub template: Option<RawTemplateConfig>,

    /// Schema configuration.
    pub schema: Option<RawSchemaConfig>,

    /// Frontmatter configuration.
    pub frontmatter: Option<RawFrontmatter>,

    /// Task configuration.
    pub task: Option<RawTaskConfig>,

    /// File metadata for staleness detection.
    ///
    /// Populated during discovery/parsing. Not serialized to TOML.
    #[serde(skip)]
    pub metadata: Option<FileMetadata>,
}

/// Raw cache configuration input.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawCacheConfig {
    /// Cache directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory: Option<String>,
}

/// Raw template configuration input.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawTemplateConfig {
    /// Templates directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory: Option<String>,
}

/// Raw schema configuration input.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawSchemaConfig {
    /// Schemas directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory: Option<String>,

    /// Property bank filename.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_bank_file: Option<String>,
}

/// Raw frontmatter configuration (unvalidated input).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[non_exhaustive]
pub struct RawFrontmatter {
    /// Frontmatter key for file classification.
    pub file_class_key: Option<String>,
    /// Frontmatter key for aliases.
    pub alias_key: Option<String>,
    /// Frontmatter key for tags.
    pub tags_key: Option<String>,
    /// Frontmatter key for title.
    pub title_key: Option<String>,
    /// Frontmatter key for created date.
    pub date_created_key: Option<String>,
    /// Frontmatter key for modified date.
    pub date_modified_key: Option<String>,
}

/// Raw logging configuration (unvalidated input from config files).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[non_exhaustive]
pub struct RawLogging {
    /// Logging verbosity level.
    pub log_level: Option<String>,
}

/// Raw task configuration input.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawTaskConfig {
    /// Whether task processing is enabled for this vault.
    pub enabled: Option<bool>,
    /// List of hashtags that identify a line as a task.
    pub task_tags: Option<Vec<String>>,
    /// Map of status name -> status spec.
    pub status: Option<HashMap<String, RawStatusSpec>>,
    /// Configuration for date fields in tasks.
    pub dates: Option<RawTaskDates>,
    /// Configuration for custom metadata fields in tasks.
    pub fields: Option<HashMap<String, RawFieldSpec>>,
    /// Configuration for indexing task fields.
    pub indexing: Option<RawIndexingConfig>,
    /// Configuration for task dependencies.
    pub dependencies: Option<RawTaskDependencies>,
    /// Use emoji format for task metadata.
    pub use_emoji: Option<bool>,
}

/// Configuration for date fields in tasks.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawTaskDates {
    /// Configuration for the 'created' date field.
    pub created: Option<RawDateFieldSpec>,
    /// Configuration for the 'due' date field.
    pub due: Option<RawDateFieldSpec>,
    /// Configuration for the 'start' date field.
    pub start: Option<RawDateFieldSpec>,
    /// Configuration for the 'scheduled' date field.
    pub scheduled: Option<RawDateFieldSpec>,
    /// Configuration for the 'completed' date field.
    pub completed: Option<RawDateFieldSpec>,
    /// Configuration for the 'reminder' date field.
    pub reminder: Option<RawDateFieldSpec>,
}

/// Raw status specification.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawStatusSpec {
    /// Checkbox symbol.
    pub symbol: char,
    /// Status type.
    pub status_type: super::task::StatusType,
    /// Optional next symbol.
    pub next_symbol: Option<char>,
}

/// Raw date field specification.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawDateFieldSpec {
    /// Keyword used in the task text (e.g., 'due:').
    pub keyword: String,
    /// Optional emoji used to identify the field.
    pub emoji: Option<char>,
    /// Expected format for the date value.
    pub format: String,
}

/// Raw custom field specification.
///
/// Type must be specified for disambiguation between similar-looking specs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum RawFieldSpec {
    /// A categorical field with a predefined set of allowed values.
    #[serde(alias = "enum")]
    Enum {
        /// List of allowed values.
        values: Vec<String>,
    },
    /// A date/time field.
    #[serde(alias = "datetime")]
    DateTime {
        /// Expected format for the value.
        format: String,
    },
    /// A string field with optional pattern matching.
    #[serde(alias = "string")]
    String {
        /// Optional regex pattern for validation.
        pattern: Option<String>,
    },
    /// An integer field with optional range constraints.
    #[serde(alias = "integer")]
    Integer {
        /// Minimum permitted value.
        min: Option<i64>,
        /// Maximum permitted value.
        max: Option<i64>,
    },
    /// A floating point field.
    #[serde(alias = "float")]
    Float {
        /// Minimum permitted value.
        min: Option<f64>,
        /// Maximum permitted value.
        max: Option<f64>,
    },
}

/// Raw indexing configuration.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawIndexingConfig {
    /// List of field keywords that should be indexed for querying.
    pub indexed_fields: Option<Vec<String>>,
}

/// Raw task dependency configuration.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawTaskDependencies {
    /// Whether task dependencies are enabled.
    pub enabled: Option<bool>,
}

/// Raw trusted vaults configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum RawTrustedVaults {
    /// List format.
    List(Vec<String>),
    /// Map format (alias -> path).
    Map(HashMap<String, String>),
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::*;

    mod formatting {
        use super::*;

        #[test]
        fn deserializes_unknown_keys_without_error() {
            let toml = r#"
                unknown_key = "value"

                [logging]
                log_level = "info"
            "#;

            let parsed: Result<RawGlobalConfig, _> = toml::from_str(toml);
            assert!(parsed.is_ok(), "Unknown keys should be ignored");
        }
    }

    mod raw_config_files {
        use super::*;

        #[test]
        fn without_vault_path_deserializes_successfully() {
            let toml = r#"
                [cache]
                directory = ".cache"
            "#;
            let result: Result<RawVaultConfig, _> = toml::from_str(toml);
            assert!(result.is_ok(), "Should deserialize without vault_path");
        }

        #[test]
        fn raw_vault_config_deserializes_from_toml() {
            let toml = r#"
                vault_path = "/vault"

                [schema]
                directory = "custom-schemas"

                [template]
                directory = "custom-templates"

                [logging]
                log_level = "debug"
            "#;

            let raw: RawVaultConfig = toml::from_str(toml).unwrap();
            assert_eq!(
                raw.schema.unwrap().directory.as_deref(),
                Some("custom-schemas")
            );
            assert_eq!(
                raw.template.unwrap().directory.as_deref(),
                Some("custom-templates")
            );
            assert_eq!(
                raw.logging.unwrap().log_level.as_deref(),
                Some("debug")
            );
        }

        #[test]
        fn raw_vault_config_supports_partial_paths() {
            let toml = r#"
                vault_path = "/vault"

                [cache]
                directory = ".cache"
                # schema omitted - will merge from lower layer
            "#;

            let raw: RawVaultConfig = toml::from_str(toml).unwrap();
            assert_eq!(raw.cache.unwrap().directory.as_deref(), Some(".cache"));
            assert!(raw.schema.is_none());
        }

        #[test]
        fn raw_global_config_defaults_to_empty() {
            let raw = RawGlobalConfig::default();

            assert!(raw.schema.is_none());
            assert!(raw.template.is_none());
            assert!(raw.frontmatter.is_none());
            assert!(raw.logging.is_none());
            assert!(raw.task.is_none());
            assert!(raw.trusted_vaults.is_none());
        }

        #[test]
        fn raw_global_paths_config_all_fields_optional() {
            let toml = "
                [template]
                # All fields omitted - should deserialize successfully
            ";

            let raw: RawGlobalConfig = toml::from_str(toml).unwrap();

            assert!(raw.schema.is_none());
            assert!(raw.template.unwrap().directory.is_none());
        }

        #[test]
        fn raw_vault_config_with_all_sections() {
            let toml = r#"
                vault_path = "/vault"

                [schema]
                directory = "schemas"

                [cache]
                directory = ".cache"

                [logging]
                log_level = "debug"

                [frontmatter]
                title_key = "title"
            "#;

            let raw: RawVaultConfig = toml::from_str(toml).unwrap();
            assert_eq!(
                raw.schema.unwrap().directory.as_deref(),
                Some("schemas")
            );
            assert_eq!(raw.cache.unwrap().directory.as_deref(), Some(".cache"));
            assert!(raw.logging.is_some());
            assert!(raw.frontmatter.is_some());
        }

        #[test]
        fn raw_vault_config_serializes_and_roundtrips() {
            let original = RawVaultConfig {
                cache: Some(RawCacheConfig {
                    directory: Some(".cache".to_owned()),
                }),
                schema: Some(RawSchemaConfig {
                    directory: Some("schemas".to_owned()),
                    property_bank_file: Some("bank.json".to_owned()),
                }),
                template: Some(RawTemplateConfig {
                    directory: Some("templates".to_owned()),
                }),
                name: None,
                version: None,
                frontmatter: None,
                logging: None,
                task: None,
                metadata: None,
            };

            let toml_string = toml::to_string(&original).unwrap();
            let deserialized: RawVaultConfig =
                toml::from_str(&toml_string).unwrap();

            assert_eq!(
                deserialized.cache.unwrap().directory,
                original.cache.unwrap().directory
            );
            assert_eq!(
                deserialized.schema.unwrap().directory,
                original.schema.unwrap().directory
            );
        }
    }
}
