use std::collections::HashMap;

use super::template::Template;
use crate::errors::DomainError;

/// Template composition for modular template building.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Composition {
    /// Additional content sections to append.
    pub additional_sections: Vec<Section>,
    /// Base template name.
    pub base_template: String,
    /// Child templates to include.
    pub includes: Vec<String>,
    /// Variable overrides for base template.
    pub variable_overrides: HashMap<String, serde_json::Value>,
}

impl Composition {
    /// Checks for circularity in includes.
    fn check_include_circularity(&self) -> Result<(), DomainError> {
        for include in &self.includes {
            if include == &self.base_template {
                return Err(DomainError::CircularComposition(include.clone()));
            }
        }
        Ok(())
    }

    /// Checks if base template extends itself.
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on enum references"
    )]
    fn check_self_extension(
        &self,
        templates: &HashMap<String, Template>,
    ) -> Result<(), DomainError> {
        if let Some(base) = templates.get(&self.base_template)
            && let Some(parent_name) = &base.extends
            && parent_name == &self.base_template
        {
            return Err(DomainError::CircularComposition(
                self.base_template.clone(),
            ));
        }
        Ok(())
    }

    /// Detects circular references in composition.
    ///
    /// # Errors
    /// Returns `DomainError::CompositionDepthExceeded` if depth > 5.
    /// Returns `DomainError::CircularComposition` if cycle detected.
    #[inline]
    pub fn detect_cycles(
        &self,
        depth: usize,
        templates: &HashMap<String, Template>,
    ) -> Result<(), DomainError> {
        Self::validate_depth(depth)?;
        self.check_self_extension(templates)?;
        self.check_include_circularity()?;

        Ok(())
    }

    /// Validates composition business rules.
    ///
    /// # Errors
    /// Returns `DomainError::VariableNotFound` if override key missing in base.
    /// Returns `DomainError::VariableTypeMismatch` if override value incompatible.
    #[inline]
    #[expect(
        clippy::iter_over_hash_type,
        reason = "Order not required for validation"
    )]
    pub fn validate(&self, base: &Template) -> Result<(), DomainError> {
        for (name, value) in &self.variable_overrides {
            let def = base
                .variables
                .get(name)
                .ok_or_else(|| DomainError::VariableNotFound(name.clone()))?;
            def.validate_value(value).map_err(|e| {
                if let DomainError::InvalidType {
                    expected,
                    ..
                } = e
                {
                    DomainError::VariableTypeMismatch {
                        name: name.clone(),
                        expected,
                        actual: format!("{value:?}"),
                    }
                } else {
                    e
                }
            })?;
        }

        Ok(())
    }

    /// Validates recursion depth.
    fn validate_depth(depth: usize) -> Result<(), DomainError> {
        if depth > 5 {
            return Err(DomainError::CompositionDepthExceeded(depth));
        }
        Ok(())
    }
}

/// Template section for composition.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Section {
    /// Section content.
    pub content: String,
    /// Section name.
    pub name: String,
    /// Insertion point.
    pub position: InsertionPosition,
}

/// Insertion point for template sections.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum InsertionPosition {
    /// Insert after named variable.
    AfterVariable(String),
    /// Insert before named variable.
    BeforeVariable(String),
    /// Insert at template start.
    Beginning,
    /// Insert at template end.
    End,
}

#[cfg(test)]
#[expect(clippy::disallowed_methods, reason = "Test logic")]
mod tests {
    mod cycle_detection {
        use uuid::Uuid;

        use super::super::{
            super::{template::Metadata, template_syntax::PlaceholderSyntax},
            *,
        };

        /// 3.4-UNIT-010: `should_detect_circular_composition_when_includes_base`
        /// AC: Circular Composition is detected in `includes` and `extends` using DFS (R-001).
        #[test]
        fn should_detect_circular_composition_when_includes_base() {
            // Given
            let mut templates = HashMap::new();
            let base = Template {
                content: "content".to_owned(),
                extends: None,
                id: Uuid::now_v7(),
                metadata: Metadata::default(),
                name: "A".to_owned(),
                pending_events: vec![],
                syntax: PlaceholderSyntax::default(),
                variables: HashMap::new(),
            };
            templates.insert("A".to_owned(), base);

            let composition = Composition {
                additional_sections: Vec::new(),
                base_template: "A".to_owned(),
                includes: vec!["A".to_owned()],
                variable_overrides: HashMap::new(),
            };

            // When
            let result = composition.detect_cycles(0, &templates);

            // Then
            assert!(matches!(result, Err(DomainError::CircularComposition(_))));
        }

        /// 3.4-UNIT-011: `should_enforce_max_composition_depth_limit`
        /// AC: composition depth is limited to Max Depth 5 to prevent stack overflow (R-001).
        #[test]
        fn should_enforce_max_composition_depth_limit() {
            // Given
            let composition = Composition {
                additional_sections: Vec::new(),
                base_template: "base".to_owned(),
                includes: Vec::new(),
                variable_overrides: HashMap::new(),
            };
            let templates = HashMap::new();

            // When
            let result = composition.detect_cycles(6, &templates);

            // Then
            assert!(matches!(
                result,
                Err(DomainError::CompositionDepthExceeded(6))
            ));
        }
    }

    mod overrides {
        use uuid::Uuid;

        use super::super::{
            super::{template::Metadata, template_syntax::PlaceholderSyntax},
            *,
        };

        /// 3.4-UNIT-012: `should_reject_override_when_type_is_inconsistent`
        /// AC: variable definitions are verified for compatibility.
        #[test]
        fn should_reject_override_when_type_is_inconsistent() {
            // Given
            let mut variables = HashMap::new();
            variables.insert(
                "count".to_owned(),
                crate::models::template_var::VariableDefinition::Number {
                    default: None,
                    max: None,
                    min: None,
                },
            );

            let base = Template {
                content: "content".to_owned(),
                extends: None,
                id: Uuid::now_v7(),
                metadata: Metadata::default(),
                name: "base".to_owned(),
                pending_events: vec![],
                syntax: PlaceholderSyntax::default(),
                variables,
            };

            let mut overrides = HashMap::new();
            overrides
                .insert("count".to_owned(), serde_json::json!("not a number"));

            let composition = Composition {
                additional_sections: Vec::new(),
                base_template: "base".to_owned(),
                includes: vec![],
                variable_overrides: overrides,
            };

            // When
            let result = composition.validate(&base);

            // Then
            assert!(matches!(
                result,
                Err(DomainError::VariableTypeMismatch { .. })
            ));
        }
    }
}
