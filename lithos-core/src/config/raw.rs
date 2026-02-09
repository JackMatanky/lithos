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
//! | Raw Type           | Validated Type  | Location                      | Validates        |
//! | ------------------ | --------------- | ----------------------------- | ---------------- |
//! | `RawFrontmatter`   | `Frontmatter`   | `frontmatter.rs` ← co-located | Non-empty keys   |
//! | `RawLogging`       | `Logging`       | `logging.rs` ← co-located     | Log level enum   |
//! | `RawTrustedVaults` | `TrustedVaults` | `global.rs`                   | List/map format  |
//! | `RawTaskConfig`    | `TaskConfig`    | `task.rs`                     | Complex rules    |
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

use super::{frontmatter::RawFrontmatter, logging::RawLogging};

// ============================================================================
// Unified Raw Config Schema (Phase 1.1 - Figment Integration)
// ============================================================================

/// Unified raw configuration for Figment merge.
///
/// This replaces separate `RawGlobal` and `RawVault` with a single schema
/// that works at all layers (defaults, global, vault). All fields are
/// `Option<T>` to enable deep merging across layers.
///
/// ## Figment Merge Flow
///
/// ```text
/// Layer 1: Compiled defaults (RawConfig::default())
///     ↓ Figment::merge
/// Layer 2: Global config (~/.config/lithos/lithos.toml)
///     ↓ Figment::merge
/// Layer 3: Vault config (<vault>/.lithos/lithos.toml)
///     ↓ Figment::extract
/// Merged RawConfig (all layers combined)
/// ```
///
/// ## Example TOML
///
/// ```toml
/// # Global config
/// [paths]
/// schemas_dir = "global-schemas"
/// templates_dir = "global-templates"
///
/// # Vault config
/// [paths]
/// schemas_dir = "vault-schemas"  # Overrides global
/// cache_dir = ".cache"            # New field
/// # templates_dir inherited from global
/// ```
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawConfig {
    /// Path configuration (deeply mergeable across layers).
    #[serde(default)]
    pub paths: RawPathsConfig,

    /// Frontmatter configuration.
    pub frontmatter: Option<RawFrontmatter>,

    /// Logging configuration.
    pub logging: Option<RawLogging>,

    /// Task configuration.
    pub task: Option<RawTaskConfig>,

    /// Trusted vaults (global-only, ignored at vault layer).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trusted_vaults: Option<RawTrustedVaults>,
}

/// Path configuration with optional fields for deep merge.
///
/// All fields are `Option<T>` so Figment can deep-merge them across layers.
/// This enables vault configs to override individual fields while inheriting
/// others from global config.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPathsConfig {
    /// Cache directory (typically vault-specific).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_dir: Option<String>,

    /// Schema directory (can override at any layer).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas_dir: Option<String>,

    /// Property bank filename (can override at any layer).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_bank_filename: Option<String>,

    /// Templates directory (can override at any layer).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub templates_dir: Option<String>,
}

// Raw trusted vaults configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum RawTrustedVaults {
    /// List format.
    List(Vec<String>),
    /// Map format (alias -> path).
    Map(HashMap<String, String>),
}

// Raw task configuration input.
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

// Raw date field configuration.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawTaskDates {
    /// Configuration for the 'due' date field.
    pub due: Option<RawDateFieldSpec>,
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
#[serde(untagged)]
#[non_exhaustive]
pub enum RawTaskFieldSpec {
    /// A categorical field with a predefined set of allowed values.
    /// Matches when 'values' array is present.
    Enum(RawEnumFieldSpec),
    /// A date/time field.
    /// Matches when 'format' string is present.
    DateTime(RawDateTimeFieldSpec),
    /// A string field with optional pattern matching.
    /// Fallback variant or matches when 'pattern' is present.
    String(RawStringFieldSpec),
    /// An integer field with optional range constraints.
    /// Matches when 'min' or 'max' are integers.
    Integer(RawIntegerFieldSpec),
    /// A floating point field.
    /// Matches when 'min' or 'max' are floats.
    Float(RawFloatFieldSpec),
}

/// Raw categorical field specification.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct RawEnumFieldSpec {
    /// Keyword used in the task text.
    pub keyword: String,
    /// List of allowed values.
    pub values: Vec<String>,
}

/// Raw date/time field specification.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct RawDateTimeFieldSpec {
    /// Keyword used in the task text.
    pub keyword: String,
    /// Expected format for the value.
    pub format: String,
}

/// Raw integer field specification.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct RawIntegerFieldSpec {
    /// Keyword used in the task text.
    pub keyword: String,
    /// Minimum permitted value.
    pub min: Option<i64>,
    /// Maximum permitted value.
    pub max: Option<i64>,
}

/// Raw floating point field specification.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct RawFloatFieldSpec {
    /// Keyword used in the task text.
    pub keyword: String,
    /// Minimum permitted value.
    pub min: Option<f64>,
    /// Maximum permitted value.
    pub max: Option<f64>,
}

/// Raw string field specification.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct RawStringFieldSpec {
    /// Keyword used in the task text.
    pub keyword: String,
    /// Optional regex pattern for validation.
    pub pattern: Option<String>,
}

/// Raw indexing configuration.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawIndexingConfig {
    /// List of field keywords that should be indexed for querying.
    pub indexed_fields: Option<Vec<String>>,
}

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

    // ========================================================================
    // Phase 1.1: Unified RawConfig Tests
    // ========================================================================

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
            assert!(raw.paths.property_bank_filename.is_none());
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
            assert!(fs.property_bank_filename.is_none());
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
                    property_bank_filename: Some("bank.json".to_owned()),
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
