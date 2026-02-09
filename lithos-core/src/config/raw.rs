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
    paths::{SchemaOverrides, TemplateOverrides},
};

/// Raw global configuration input.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawGlobal {
    /// Filesystem configuration for global defaults.
    pub filesystem: Option<RawGlobalPaths>,
    /// Frontmatter configuration overrrides.
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

/// Raw filesystem configuration for global defaults.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawGlobalPaths {
    /// Global schema directory.
    pub schema: Option<SchemaOverrides>,
    /// Global template directory.
    pub template: Option<TemplateOverrides>,
}

/// Raw filesystem configuration for vault overrides.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawVaultPaths {
    /// Local cache directory override.
    pub cache_dir: Option<String>,
    /// Vault-specific schema overrides.
    pub schema: Option<SchemaOverrides>,
    /// Vault-specific template overrides.
    pub template: Option<TemplateOverrides>,
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
    pub fields: Option<HashMap<String, RawTaskFieldSpec>>,
    /// Configuration for indexing task fields.
    pub indexing: Option<RawIndexingConfig>,
}

/// Raw date field configuration.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawTaskDates {
    /// Configuration for the 'due' date field.
    pub due: Option<RawDateFieldSpec>,
    /// Configuration for the 'scheduled' date field.
    pub scheduled: Option<RawDateFieldSpec>,
    /// Configuration for the 'start' date field.
    pub start: Option<RawDateFieldSpec>,
    /// Configuration for the 'completed' date field.
    pub completed: Option<RawDateFieldSpec>,
    /// Configuration for the 'created' date field.
    pub created: Option<RawDateFieldSpec>,
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

/// Raw custom task field specification.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[non_exhaustive]
pub enum RawTaskFieldSpec {
    /// An integer field with optional range constraints.
    Integer {
        /// Keyword used in the task text.
        keyword: String,
        /// Minimum permitted value.
        min: Option<i64>,
        /// Maximum permitted value.
        max: Option<i64>,
    },
    /// A floating point field.
    Float {
        /// Keyword used in the task text.
        keyword: String,
        /// Minimum permitted value.
        min: Option<f64>,
        /// Maximum permitted value.
        max: Option<f64>,
    },
    /// A date/time field.
    DateTime {
        /// Keyword used in the task text.
        keyword: String,
        /// Expected format for the value.
        format: String,
    },
    /// A string field with optional pattern matching.
    String {
        /// Keyword used in the task text.
        keyword: String,
        /// Optional regex pattern for validation.
        pattern: Option<String>,
    },
    /// A categorical field with a predefined set of allowed values.
    Enum {
        /// Keyword used in the task text.
        keyword: String,
        /// List of allowed values.
        values: Vec<String>,
    },
}

/// Raw indexing configuration.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawIndexingConfig {
    /// List of field keywords that should be indexed for querying.
    pub indexed_fields: Option<Vec<String>>,
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test modules group fixtures and test logic for readability"
)]
mod tests {
    mod fixtures {
        use crate::config::{logging::RawLogging, raw::RawGlobal};

        pub fn raw_global_invalid_logging() -> RawGlobal {
            RawGlobal {
                filesystem: None,
                frontmatter: None,
                logging: Some(RawLogging {
                    log_level: Some("verbose".to_owned()),
                }),
                trusted_vaults: None,
                task: None,
            }
        }
    }

    use super::*;
    use crate::config::global::Global;

    mod formatting {
        use super::*;

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
    }

    mod conversions {
        use super::*;

        #[test]
        fn global_rejects_invalid_log_level() {
            let raw = fixtures::raw_global_invalid_logging();

            let result = Global::try_from(raw);
            assert!(result.is_err(), "Invalid log level should be rejected");
        }

        #[test]
        fn vault_rejects_absolute_template_dir() {
            // Validation now happens at deserialization time for Overrides.
            // We simulate deserialization failure by checking if we can
            // construct it invalidly. But since we can't construct an
            // invalid TemplatesDir via safe API, we test
            // deserialization directly.
            let toml = r#"
[filesystem.template]
templates_dir = "/abs"
"#;
            let parsed: Result<RawVault, _> = toml::from_str(toml);
            assert!(
                parsed.is_err(),
                "Absolute templates_dir should be rejected during \
                 deserialization"
            );
        }
    }
}
