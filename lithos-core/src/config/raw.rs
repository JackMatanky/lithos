//! Raw (serde) configuration input types.
//!
//! ## Purpose
//!
//! These types are **serde-only Data Transfer Objects (DTOs)** that serve as
//! the deserialization boundary between configuration files (TOML/YAML/JSON)
//! and validated domain models.
//!
//! ## Architecture Pattern
//!
//! ```text
//! Config File (TOML)
//!     ↓ serde::Deserialize
//! Raw* Types (unvalidated, optional fields)
//!     ↓ TryFrom<Raw*> (validation happens here)
//! Domain Types (validated invariants, typed enums)
//!     ↓ rkyv::Archive
//! Database (redb, zero-copy bytes)
//! ```
//!
//! ## Key Characteristics
//!
//! - **No Implementation**: Raw types have zero methods by design
//! - **All Optional Fields**: Accept flexible/partial configuration input
//! - **No Validation**: Can contain empty strings, invalid values,
//!   contradictions
//! - **String Enums**: Accept any string, validation happens in `TryFrom`
//! - **Not Persisted**: Only validated domain types are stored in the database
//!
//! ## Validation Boundary & Co-location
//!
//! **Domain-specific Raw types are co-located** with their validated types for
//! better readability. The aggregate Raw types remain in this module.
//!
//! Validation occurs in `TryFrom<Raw*>` implementations in the same file:
//!
//! | Raw Type | Validated Type | Location | Validates |
//! |----------|---------------|----------|-----------|
//! | `RawFrontmatter` | `Frontmatter` | `frontmatter.rs` ← co-located | Non-empty keys |
//! | `RawLogging` | `Logging` | `logging.rs` ← co-located | Log level enum |
//! | `RawSchemaPaths` | `Schema` | `paths.rs` ← co-located | Path validity |
//! | `RawTemplatePaths` | `Template` | `paths.rs` ← co-located | Path validity |
//! | `RawGlobal` | `Global` | `global.rs` | Aggregation |
//! | `RawVault` | `Vault` | `vault.rs` | Aggregation |
//! | `RawGlobalPaths` | `Paths` | `global.rs` | Path aggregation |
//! | `RawVaultPaths` | `Paths` | `vault.rs` | Path aggregation |
//! | `RawTaskConfig` | `TaskConfig` | `task.rs` | Complex rules |
//! | `RawTrustedVaults` | `TrustedVaults` | `global.rs` | List/map format |
//!
//! The domain-specific Raw types are imported by this module (for use in the
//! aggregate structs) but defined alongside their validated types.
//!
//! ## Design Rationale
//!
//! Separating Raw types from validated domain types enables:
//!
//! - **Flexible Parsing**: Accept typos, wrong types, partial configs
//! - **Clear Error Messages**: Report which file field failed validation
//! - **Default Handling**: `None` fields trigger default value logic
//! - **Independent Evolution**: File format can change without breaking domain
//!
//! See `docs/design/001-config-models.md` Section 3.2.1 for full rationale.

#![allow(
    missing_docs,
    reason = "Raw config DTOs mirror file schema; field docs pending."
)]

use std::collections::HashMap;

use super::{
    frontmatter::RawFrontmatter,
    logging::RawLogging,
    paths::{CacheDir, SchemaOverrides, TemplateOverrides},
};

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
    pub schema: Option<SchemaOverrides>,
    /// Template-related filesystem config.
    pub template: Option<TemplateOverrides>,
}

/// Raw vault filesystem configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[non_exhaustive]
pub struct RawVaultPaths {
    /// Cache directory override.
    pub cache_dir: Option<CacheDir>,
    /// Schema-related filesystem config.
    pub schema: Option<SchemaOverrides>,
    /// Template-related filesystem config.
    pub template: Option<TemplateOverrides>,
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
    use crate::config::global::Global;

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
        // Validation now happens at deserialization time for Overrides.
        // We simulate deserialization failure by checking if we can construct
        // it invalidly. But since we can't construct an invalid
        // TemplatesDir via safe API, we test deserialization directly.
        let toml = r#"
[filesystem.template]
templates_dir = "/abs"
"#;
        let parsed: Result<RawVault, _> = toml::from_str(toml);
        assert!(
            parsed.is_err(),
            "Absolute templates_dir should be rejected during deserialization"
        );
    }
}
