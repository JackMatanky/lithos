//! Raw (serde) configuration input types (DTOs).
//!
//! This module defines the [`RawConfig`] and supporting types used for
//! deserialization from TOML/YAML/JSON files before validation.

#![allow(
    missing_docs,
    reason = "Raw config DTOs mirror file schema; field docs pending."
)]

use std::collections::HashMap;

use super::{frontmatter::RawFrontmatter, logging::RawLogging};

// ----------------------------------------------------------- //
//                  Raw Config Aggregate Root                  //
// ----------------------------------------------------------- //

/// Unified raw configuration for Figment merge.
///
/// This struct serves as the primary Data Transfer Object (DTO) for
/// deserializing configuration from files. All fields are optional to
/// support deep merging across multiple layers (defaults, global, vault).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawConfig {
    /// Logging configuration.
    pub logging: Option<RawLogging>,

    /// Path configuration (deeply mergeable across layers).
    #[serde(default)]
    pub paths: RawPathsConfig,

    /// Trusted vaults (global-only, ignored at vault layer).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trusted_vaults: Option<RawTrustedVaults>,

    /// Frontmatter configuration.
    pub frontmatter: Option<RawFrontmatter>,

    /// Task configuration.
    pub task: Option<RawTaskConfig>,
}

/// Raw path configuration input.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPathsConfig {
    /// Cache directory (typically vault-specific).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_dir: Option<String>,

    /// Templates directory (can override at any layer).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub templates_dir: Option<String>,

    /// Schema directory (can override at any layer).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas_dir: Option<String>,

    /// Property bank filename (can override at any layer).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_bank_file: Option<String>,
}

/// Raw task configuration input.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawTaskConfig {
    /// Whether task processing is enabled for this vault.
    pub enabled: Option<bool>,
    /// List of hashtags that identify a line as a task.
    pub task_tags: Option<Vec<String>>,
    /// Map of checkbox symbols to status names.
    pub status: Option<HashMap<String, char>>,
    /// Configuration for date fields in tasks.
    pub dates: Option<RawTaskDates>,
    /// Configuration for custom metadata fields in tasks.
    pub fields: Option<HashMap<String, RawFieldSpec>>,
    /// Configuration for indexing task fields.
    pub indexing: Option<RawIndexingConfig>,
}

/// Configuration for date fields in tasks.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawTaskDates {
    /// Configuration for the 'created' date field.
    pub created: Option<RawDateFieldSpec>,
    /// Configuration for the 'due' date field.
    pub due: Option<RawDateFieldSpec>,
    /// Configuration for the 'completed' date field.
    pub completed: Option<RawDateFieldSpec>,
    /// Configuration for the 'reminder' date field.
    pub reminder: Option<RawDateFieldSpec>,
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

            let parsed: Result<RawConfig, _> = toml::from_str(toml);
            assert!(parsed.is_ok(), "Unknown keys should be ignored");
        }
    }

    mod unified_raw_config {
        use super::*;

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Tests use unwrap() for deterministic TOML parsing"
        )]
        fn raw_config_deserializes_from_toml() {
            let toml = r#"
                [paths]
                schemas_dir = "custom-schemas"
                templates_dir = "custom-templates"

                [logging]
                log_level = "debug"
            "#;

            let raw: RawConfig = toml::from_str(toml).unwrap();
            assert_eq!(
                raw.paths.schemas_dir.as_deref(),
                Some("custom-schemas")
            );
            assert_eq!(
                raw.paths.templates_dir.as_deref(),
                Some("custom-templates")
            );
            assert_eq!(
                raw.logging.unwrap().log_level.as_deref(),
                Some("debug")
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Tests use unwrap() for deterministic TOML parsing"
        )]
        fn raw_config_supports_partial_paths() {
            let toml = r#"
                [paths]
                cache_dir = ".cache"
                # schemas_dir omitted - will merge from lower layer
            "#;

            let raw: RawConfig = toml::from_str(toml).unwrap();
            assert_eq!(raw.paths.cache_dir.as_deref(), Some(".cache"));
            assert_eq!(raw.paths.schemas_dir, None);
        }

        #[test]
        fn raw_config_defaults_to_empty() {
            let raw = RawConfig::default();

            assert!(raw.paths.cache_dir.is_none());
            assert!(raw.paths.schemas_dir.is_none());
            assert!(raw.paths.property_bank_file.is_none());
            assert!(raw.paths.templates_dir.is_none());
            assert!(raw.frontmatter.is_none());
            assert!(raw.logging.is_none());
            assert!(raw.task.is_none());
            assert!(raw.trusted_vaults.is_none());
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Tests use unwrap() for deterministic TOML parsing"
        )]
        fn raw_paths_config_all_fields_optional() {
            let toml = "
                [paths]
                # All fields omitted - should deserialize successfully
            ";

            let raw: RawConfig = toml::from_str(toml).unwrap();
            let fs = raw.paths;

            assert!(fs.cache_dir.is_none());
            assert!(fs.schemas_dir.is_none());
            assert!(fs.property_bank_file.is_none());
            assert!(fs.templates_dir.is_none());
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Tests use unwrap() for deterministic TOML parsing"
        )]
        fn raw_config_with_all_sections() {
            let toml = r#"
                [paths]
                schemas_dir = "schemas"
                cache_dir = ".cache"

                [logging]
                log_level = "debug"

                [frontmatter]
                title_key = "title"
            "#;

            let raw: RawConfig = toml::from_str(toml).unwrap();
            assert_eq!(raw.paths.schemas_dir.as_deref(), Some("schemas"));
            assert_eq!(raw.paths.cache_dir.as_deref(), Some(".cache"));
            assert!(raw.logging.is_some());
            assert!(raw.frontmatter.is_some());
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Tests use unwrap() for deterministic TOML parsing"
        )]
        fn raw_config_serializes_and_roundtrips() {
            let original = RawConfig {
                paths: RawPathsConfig {
                    cache_dir: Some(".cache".to_owned()),
                    schemas_dir: Some("schemas".to_owned()),
                    property_bank_file: Some("bank.json".to_owned()),
                    templates_dir: Some("templates".to_owned()),
                },
                frontmatter: None,
                logging: None,
                task: None,
                trusted_vaults: None,
            };

            let toml_string = toml::to_string(&original).unwrap();
            let deserialized: RawConfig = toml::from_str(&toml_string).unwrap();

            assert_eq!(deserialized.paths.cache_dir, original.paths.cache_dir);
            assert_eq!(
                deserialized.paths.schemas_dir,
                original.paths.schemas_dir
            );
        }
    }
}
