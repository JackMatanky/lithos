//! Raw (serde) configuration input types.
//!
//! These types represent unvalidated configuration shapes loaded from files.
//! They are converted into validated domain types via `TryFrom`.

#![allow(
    missing_docs,
    reason = "Raw config DTOs mirror file schema; field docs pending."
)]

use std::collections::HashMap;

/// Raw global configuration input.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawGlobal {
    /// Filesystem configuration for global defaults.
    pub filesystem: Option<RawGlobalPaths>,
    /// Frontmatter configuration overrides.
    pub frontmatter: Option<RawFrontmatter>,
    /// Logging configuration overrides.
    pub logging: Option<RawLogging>,
    /// Trusted vaults configuration.
    pub trusted_vaults: Option<RawTrustedVaults>,
    /// Task configuration (global defaults).
    pub task: Option<RawTaskConfig>,
}

/// Raw vault configuration input.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawVault {
    /// Filesystem configuration for vault overrides.
    pub filesystem: Option<RawVaultPaths>,
    /// Frontmatter configuration overrides.
    pub frontmatter: Option<RawFrontmatter>,
    /// Logging configuration overrides.
    pub logging: Option<RawLogging>,
    /// Task configuration overrides.
    pub task: Option<RawTaskConfig>,
}

/// Raw global filesystem configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[non_exhaustive]
pub struct RawGlobalPaths {
    /// Schema-related filesystem config.
    pub schema: Option<RawSchemaPaths>,
    /// Template-related filesystem config.
    pub template: Option<RawTemplatePaths>,
}

/// Raw vault filesystem configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[non_exhaustive]
pub struct RawVaultPaths {
    /// Cache directory override.
    pub cache_dir: Option<String>,
    /// Schema-related filesystem config.
    pub schema: Option<RawSchemaPaths>,
    /// Template-related filesystem config.
    pub template: Option<RawTemplatePaths>,
}

/// Raw schema filesystem configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[non_exhaustive]
pub struct RawSchemaPaths {
    /// Directory containing schema files.
    pub schemas_dir: Option<String>,
    /// Property bank filename.
    pub property_bank_filename: Option<String>,
}

/// Raw template filesystem configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[non_exhaustive]
pub struct RawTemplatePaths {
    /// Directory containing template files.
    pub templates_dir: Option<String>,
}

/// Raw frontmatter configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[non_exhaustive]
pub struct RawFrontmatter {
    /// Frontmatter key for aliases.
    pub alias_key: Option<String>,
    /// Frontmatter key for created date.
    pub date_created_key: Option<String>,
    /// Frontmatter key for modified date.
    pub date_modified_key: Option<String>,
    /// Frontmatter key for file classification.
    pub file_class_key: Option<String>,
    /// Frontmatter key for title.
    pub title_key: Option<String>,
}

/// Raw logging configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[non_exhaustive]
pub struct RawLogging {
    /// Logging verbosity level.
    pub log_level: Option<String>,
}

/// Raw trusted vaults configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum RawTrustedVaults {
    /// List of trusted vault paths.
    List(Vec<String>),
    /// Map of aliases to trusted vault paths.
    Map(HashMap<String, String>),
}

/// Raw task configuration.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawTaskConfig {
    /// Enable task parsing.
    pub enabled: Option<bool>,
    /// Tags that identify tasks.
    pub task_tags: Option<Vec<String>>,
    /// Status mapping from name to symbol.
    pub status: Option<HashMap<String, char>>,
    /// Date field configuration.
    pub dates: Option<RawTaskDates>,
    /// Custom task field specs.
    pub fields: Option<HashMap<String, RawTaskFieldSpec>>,
    /// Indexing configuration for task fields.
    pub indexing: Option<RawIndexingConfig>,
}

/// Raw task dates configuration.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawTaskDates {
    /// Due date field spec.
    pub due: Option<RawDateFieldSpec>,
    /// Created date field spec.
    pub created: Option<RawDateFieldSpec>,
    /// Reminder date field spec.
    pub reminder: Option<RawDateFieldSpec>,
    /// Completed date field spec.
    pub completed: Option<RawDateFieldSpec>,
}

/// Raw date field specification.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawDateFieldSpec {
    /// Keyword used in task metadata.
    pub keyword: String,
    /// Optional emoji prefix for the field.
    pub emoji: Option<char>,
    /// Chrono datetime format string.
    pub format: String,
}

/// Raw indexing configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawIndexingConfig {
    /// Names of task fields to index.
    pub indexed_fields: Option<Vec<String>>,
}

/// Raw task field specification (type inferred by shape).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum RawTaskFieldSpec {
    /// Enum field definition.
    Enum {
        /// Field keyword.
        keyword: String,
        /// Allowed values.
        values: Vec<String>,
    },
    /// Integer field definition.
    Integer {
        /// Field keyword.
        keyword: String,
        #[serde(default)]
        /// Minimum allowed value.
        min: Option<i64>,
        #[serde(default)]
        /// Maximum allowed value.
        max: Option<i64>,
    },
    /// Floating-point field definition.
    Float {
        /// Field keyword.
        keyword: String,
        #[serde(default)]
        /// Minimum allowed value.
        min: Option<f64>,
        #[serde(default)]
        /// Maximum allowed value.
        max: Option<f64>,
    },
    /// Datetime field definition.
    DateTime {
        /// Field keyword.
        keyword: String,
        /// Chrono datetime format string.
        format: String,
    },
    /// String field definition.
    String {
        /// Field keyword.
        keyword: String,
        #[serde(default)]
        /// Optional regex pattern constraint.
        pattern: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{global::Global, vault::Vault};

    #[test]
    fn deserializes_unknown_keys_without_error() {
        let toml = r#"
unknown_key = "value"

[logging]
log_level = "info"
"#;

        let parsed: Result<RawGlobal, _> = toml::from_str(toml);
        assert!(parsed.is_ok(), "Unknown keys should be ignored");
    }

    #[test]
    fn conversion_rejects_invalid_log_level() {
        let raw = RawGlobal {
            filesystem: None,
            frontmatter: None,
            logging: Some(RawLogging {
                log_level: Some("verbose".to_owned()),
            }),
            trusted_vaults: None,
            task: None,
        };

        let result = Global::try_from(raw);
        assert!(result.is_err(), "Invalid log level should be rejected");
    }

    #[test]
    fn conversion_rejects_absolute_template_dir_in_vault() {
        let raw = RawVault {
            filesystem: Some(RawVaultPaths {
                cache_dir: None,
                schema: None,
                template: Some(RawTemplatePaths {
                    templates_dir: Some("/abs".to_owned()),
                }),
            }),
            frontmatter: None,
            logging: None,
            task: None,
        };

        let result = Vault::try_from(raw);
        assert!(result.is_err(), "Absolute templates_dir should be rejected");
    }
}
