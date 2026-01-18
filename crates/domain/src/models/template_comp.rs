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
    /// Detects circular references in composition.
    ///
    /// # Errors
    /// Returns `DomainError::CompositionDepthExceeded` if depth > 5.
    /// Returns `DomainError::CircularComposition` if cycle detected.
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on enum references"
    )]
    pub fn detect_cycles(
        &self,
        depth: usize,
        templates: &HashMap<String, Template>,
    ) -> Result<(), DomainError> {
        if depth > 5 {
            return Err(DomainError::CompositionDepthExceeded(depth));
        }

        // Check if base template exists
        if let Some(base) = templates.get(&self.base_template)
            && let Some(parent_name) = &base.extends
            && parent_name == &self.base_template
        {
            return Err(DomainError::CircularComposition(
                self.base_template.clone(),
            ));
        }

        // Check includes for circularity
        for include in &self.includes {
            if include == &self.base_template {
                return Err(DomainError::CircularComposition(include.clone()));
            }
        }

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
    use uuid::Uuid;

    use super::{super::template::Metadata, *};

    #[test]
    fn detects_direct_circular_composition() {
        let mut templates = HashMap::new();
        let base = Template {
            content: "content".to_owned(),
            extends: None,
            id: Uuid::now_v7(),
            metadata: Metadata::default(),
            name: "A".to_owned(),
            pending_events: vec![],
            variables: HashMap::new(),
        };
        templates.insert("A".to_owned(), base);

        let composition = Composition {
            additional_sections: Vec::new(),
            base_template: "A".to_owned(),
            includes: vec!["A".to_owned()],
            variable_overrides: HashMap::new(),
        };

        let result = composition.detect_cycles(0, &templates);
        assert!(matches!(result, Err(DomainError::CircularComposition(_))));
    }

    #[test]
    fn enforces_max_depth_limit() {
        let composition = Composition {
            additional_sections: Vec::new(),
            base_template: "base".to_owned(),
            includes: Vec::new(),
            variable_overrides: HashMap::new(),
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
            variables,
        };

        let mut overrides = HashMap::new();
        overrides.insert("count".to_owned(), serde_json::json!("not a number"));

        let composition = Composition {
            additional_sections: Vec::new(),
            base_template: "base".to_owned(),
            includes: vec![],
            variable_overrides: overrides,
        };

        let result = composition.validate(&base);
        assert!(matches!(
            result,
            Err(DomainError::VariableTypeMismatch { .. })
        ));
    }
}
