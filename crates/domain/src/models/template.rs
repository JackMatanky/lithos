//! Template domain entities and business logic.
//!
//! This module defines the Template aggregate root and its associated subentities:
//! VariableDefinition, TemplateComposition, TemplateSection, and TemplateMetadata.
//!
//! # Business Rules
//! - Template IDs use UUID v7 for stable, time-ordered identity.
//! - Names must follow regex `^[a-zA-Z0-9_-]+$` and be max 64 characters.
//! - Variable names must follow regex `^[a-zA-Z_][a-zA-Z0-9_]*$` and be max 32 characters.
//! - Circular compositions are prohibited (detected via DFS).
//! - Composition depth is limited to 5 to prevent stack overflow.
//! - Maximum of 50 variables per template.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::errors::DomainError;

/// Aggregate root representing a reusable template.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Template {
    /// UUID v7 identity.
    pub id: Uuid,
    /// Unique template name.
    pub name: String,
    /// Template content (MiniJinja-compatible syntax).
    pub content: String,
    /// Variable definitions with types and constraints.
    pub variables: HashMap<String, VariableDefinition>,
    /// Optional parent template for composition.
    pub extends: Option<String>,
    /// Metadata for template management.
    pub metadata: TemplateMetadata,
}

impl Template {
    /// Creates a new template aggregate with validation.
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` in RED phase.
    #[inline]
    pub fn new(
        _name: String,
        _content: String,
        _variables: HashMap<String, VariableDefinition>,
        _extends: Option<String>,
        _metadata: TemplateMetadata,
    ) -> Result<Self, DomainError> {
        // RED PHASE: Not implemented
        Err(DomainError::ValidationFailed("Not implemented".to_owned()))
    }

    /// Validates template business rules.
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` in RED phase.
    #[inline]
    pub fn validate(&self) -> Result<(), DomainError> {
        // RED PHASE: Not implemented
        Err(DomainError::ValidationFailed("Not implemented".to_owned()))
    }

    /// Composes a template from a base and a composition.
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` in RED phase.
    #[inline]
    pub fn compose(
        _base: &Template,
        _composition: &TemplateComposition,
    ) -> Result<Self, DomainError> {
        // RED PHASE: Not implemented
        Err(DomainError::ValidationFailed("Not implemented".to_owned()))
    }
}

/// Metadata for template management.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TemplateMetadata {
    /// Template description.
    pub description: Option<String>,
    /// Template version.
    pub version: Option<String>,
    /// Tags for categorization.
    pub tags: Vec<String>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp.
    pub updated_at: DateTime<Utc>,
}

impl Default for TemplateMetadata {
    fn default() -> Self {
        Self {
            description: None,
            version: None,
            tags: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// Type-safe variable definition with validation constraints.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum VariableDefinition {
    /// String variable.
    String {
        /// Default value.
        default: Option<String>,
        /// Minimum length.
        min_length: Option<usize>,
        /// Maximum length.
        max_length: Option<usize>,
        /// Regex pattern.
        pattern: Option<String>,
    },
    /// Number variable.
    Number {
        /// Default value.
        default: Option<f64>,
        /// Minimum value.
        min: Option<f64>,
        /// Maximum value.
        max: Option<f64>,
    },
    /// Boolean variable.
    Boolean {
        /// Default value.
        default: Option<bool>,
    },
    /// Date variable.
    Date {
        /// Default value.
        default: Option<String>,
        /// ISO 8601 format string.
        format: Option<String>,
    },
    /// File reference variable.
    File {
        /// Default value.
        default: Option<String>,
        /// Allowed file types.
        file_types: Option<Vec<String>>,
    },
}

impl VariableDefinition {
    /// Validates a value against this definition.
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` in RED phase.
    #[inline]
    pub fn validate_value(
        &self,
        _value: &serde_json::Value,
    ) -> Result<(), DomainError> {
        // RED PHASE: Not implemented
        Err(DomainError::ValidationFailed("Not implemented".to_owned()))
    }
}

/// Template composition for modular template building.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TemplateComposition {
    /// Base template name.
    pub base_template: String,
    /// Variable overrides for base template.
    pub variable_overrides: HashMap<String, serde_json::Value>,
    /// Additional content sections to append.
    pub additional_sections: Vec<TemplateSection>,
    /// Child templates to include.
    pub includes: Vec<String>,
}

impl TemplateComposition {
    /// Detects circular references in composition.
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` in RED phase.
    #[inline]
    pub fn detect_cycles(
        &self,
        _depth: usize,
        _templates: &HashMap<String, Template>,
    ) -> Result<(), DomainError> {
        // RED PHASE: Not implemented
        Err(DomainError::ValidationFailed("Not implemented".to_owned()))
    }

    /// Validates composition business rules.
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` in RED phase.
    #[inline]
    pub fn validate(&self, _base: &Template) -> Result<(), DomainError> {
        // RED PHASE: Not implemented
        Err(DomainError::ValidationFailed("Not implemented".to_owned()))
    }
}

/// Template section for composition.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TemplateSection {
    /// Section name.
    pub name: String,
    /// Section content.
    pub content: String,
    /// Insertion point.
    pub position: InsertionPosition,
}

/// Insertion point for template sections.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum InsertionPosition {
    /// Insert before named variable.
    BeforeVariable(String),
    /// Insert after named variable.
    AfterVariable(String),
    /// Insert at template start.
    Beginning,
    /// Insert at template end.
    End,
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    mod template {
        use super::*;

        #[test]
        fn creates_valid_template_successfully() {
            let metadata = TemplateMetadata::default();
            let mut variables = HashMap::new();
            variables.insert(
                "title".to_owned(),
                VariableDefinition::String {
                    default: Some("Default".to_owned()),
                    min_length: None,
                    max_length: None,
                    pattern: None,
                },
            );

            let result = Template::new(
                "daily-note".to_owned(),
                "# {{title}}".to_owned(),
                variables,
                None,
                metadata,
            );

            // In RED phase, this fails because implementation returns error
            assert!(
                result.is_ok(),
                "Expected valid template creation to succeed"
            );
        }

        #[test]
        fn rejects_invalid_template_names() {
            let names = vec![
                "".to_owned(),
                "Invalid Name".to_owned(),
                "too--many--dashes".to_owned(),
                "name!".to_owned(),
                "a".repeat(65),
            ];
            for name in names {
                let result = Template::new(
                    name.clone(),
                    "content".to_owned(),
                    HashMap::new(),
                    None,
                    TemplateMetadata::default(),
                );
                assert!(
                    result.is_err(),
                    "Expected name '{}' to be rejected",
                    name
                );
            }
        }

        #[test]
        fn rejects_large_content() {
            let large_content = "a".repeat(1024 * 1024 + 1); // 1MB + 1
            let result = Template::new(
                "large".to_owned(),
                large_content,
                HashMap::new(),
                None,
                TemplateMetadata::default(),
            );
            assert!(matches!(
                result,
                Err(DomainError::TemplateContentTooLarge(_))
            ));
        }

        #[test]
        fn rejects_too_many_variables() {
            let mut variables = HashMap::new();
            for i in 0..51 {
                variables.insert(
                    format!("var{}", i),
                    VariableDefinition::Boolean {
                        default: None,
                    },
                );
            }
            let result = Template::new(
                "many-vars".to_owned(),
                "content".to_owned(),
                variables,
                None,
                TemplateMetadata::default(),
            );
            assert!(matches!(
                result,
                Err(DomainError::MaxVariablesExceeded(_))
            ));
        }

        proptest! {
            #[test]
            fn validates_template_name_format(name in "[a-zA-Z0-9_-]{1,64}") {
                let result = Template::new(
                    name,
                    "content".to_owned(),
                    HashMap::new(),
                    None,
                    TemplateMetadata::default(),
                );
                // Should be Ok(..) eventually, but fails in RED phase
                prop_assert!(result.is_ok());
            }
        }
    }

    mod variables {
        use super::*;

        #[test]
        fn rejects_invalid_variable_names() {
            let names = vec!["", "123var", "var-name", "var name", "if", "for"];
            for name in names {
                let mut variables = HashMap::new();
                variables.insert(
                    name.to_owned(),
                    VariableDefinition::Boolean {
                        default: None,
                    },
                );
                let result = Template::new(
                    "test".to_owned(),
                    "content".to_owned(),
                    variables,
                    None,
                    TemplateMetadata::default(),
                );
                assert!(
                    result.is_err(),
                    "Expected variable name '{}' to be rejected",
                    name
                );
            }
        }

        #[test]
        fn validates_string_constraints() {
            let def = VariableDefinition::String {
                default: None,
                min_length: Some(3),
                max_length: Some(10),
                pattern: Some("^[a-z]+$".to_owned()),
            };

            assert!(def.validate_value(&serde_json::json!("abc")).is_ok());
            assert!(def.validate_value(&serde_json::json!("ab")).is_err());
            assert!(
                def.validate_value(&serde_json::json!("abcdefghijk")).is_err()
            );
            assert!(def.validate_value(&serde_json::json!("ABC")).is_err());
        }

        #[test]
        fn validates_number_constraints() {
            let def = VariableDefinition::Number {
                default: None,
                min: Some(1.0),
                max: Some(10.0),
            };

            assert!(def.validate_value(&serde_json::json!(5.0)).is_ok());
            assert!(def.validate_value(&serde_json::json!(0.5)).is_err());
            assert!(def.validate_value(&serde_json::json!(10.5)).is_err());
        }
    }

    mod composition {
        use super::*;

        #[test]
        fn detects_direct_circular_composition() {
            let mut templates = HashMap::new();
            let base = Template {
                id: Uuid::now_v7(),
                name: "A".to_owned(),
                content: "content".to_owned(),
                variables: HashMap::new(),
                extends: None,
                metadata: TemplateMetadata::default(),
            };
            templates.insert("A".to_owned(), base);

            let composition = TemplateComposition {
                base_template: "A".to_owned(),
                variable_overrides: HashMap::new(),
                additional_sections: Vec::new(),
                includes: vec!["A".to_owned()],
            };

            let result = composition.detect_cycles(0, &templates);
            assert!(matches!(result, Err(DomainError::CircularComposition(_))));
        }

        #[test]
        fn detects_indirect_circular_composition() {
            let mut templates = HashMap::new();

            let a = Template {
                id: Uuid::now_v7(),
                name: "A".to_owned(),
                content: "content".to_owned(),
                variables: HashMap::new(),
                extends: None,
                metadata: TemplateMetadata::default(),
            };
            let b = Template {
                id: Uuid::now_v7(),
                name: "B".to_owned(),
                content: "content".to_owned(),
                variables: HashMap::new(),
                extends: None,
                metadata: TemplateMetadata::default(),
            };
            templates.insert("A".to_owned(), a);
            templates.insert("B".to_owned(), b);

            unimplemented!(
                "RED PHASE: Cycle detection test for indirect cycles"
            );
        }

        #[test]
        fn enforces_max_depth_limit() {
            let composition = TemplateComposition {
                base_template: "base".to_owned(),
                variable_overrides: HashMap::new(),
                additional_sections: Vec::new(),
                includes: Vec::new(),
            };

            let templates = HashMap::new();
            let result = composition.detect_cycles(6, &templates);
            assert!(matches!(
                result,
                Err(DomainError::CompositionDepthExceeded(6))
            ));
        }

        #[test]
        fn validates_override_type_consistency() {
            let mut variables = HashMap::new();
            variables.insert(
                "count".to_owned(),
                VariableDefinition::Number {
                    default: None,
                    min: None,
                    max: None,
                },
            );

            let base = Template {
                id: Uuid::now_v7(),
                name: "base".to_owned(),
                content: "content".to_owned(),
                variables,
                extends: None,
                metadata: TemplateMetadata::default(),
            };

            let mut overrides = HashMap::new();
            overrides
                .insert("count".to_owned(), serde_json::json!("not a number"));

            let composition = TemplateComposition {
                base_template: "base".to_owned(),
                variable_overrides: overrides,
                additional_sections: Vec::new(),
                includes: Vec::new(),
            };

            let result = composition.validate(&base);
            assert!(matches!(
                result,
                Err(DomainError::VariableTypeMismatch { .. })
            ));
        }
    }
}

/// Test fixtures for deterministic template data.
#[cfg(test)]
pub mod fixtures {
    use super::*;

    /// Creates an example template for testing.
    pub fn example_template() -> Template {
        let mut variables = HashMap::new();
        variables.insert(
            "title".to_owned(),
            VariableDefinition::String {
                default: Some("Untitled".to_owned()),
                min_length: Some(1),
                max_length: Some(100),
                pattern: None,
            },
        );

        Template {
            id: Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0003),
            name: "example".to_owned(),
            content: "# {{title}}".to_owned(),
            variables,
            extends: None,
            metadata: TemplateMetadata::default(),
        }
    }
}
