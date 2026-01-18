use std::collections::{HashMap, HashSet};

use super::core::Template;
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

#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Function ordering optimized for logical flow over strict alphabetical order"
)]
impl Composition {
    fn check_template_dependencies(
        &self,
        template: &Template,
        depth: usize,
        templates: &HashMap<String, Template>,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
        current_name: &str,
    ) -> Result<(), DomainError> {
        #[expect(clippy::pattern_type_mismatch, reason = "DFS logic")]
        if let Some(parent_name) = &template.extends {
            #[expect(clippy::arithmetic_side_effects, reason = "DFS logic")]
            self.dfs_check(parent_name, depth + 1, templates, visited, stack)?;
        }

        if current_name == self.base_template {
            for include in &self.includes {
                #[expect(
                    clippy::arithmetic_side_effects,
                    reason = "DFS logic"
                )]
                self.dfs_check(include, depth + 1, templates, visited, stack)?;
            }
        }
        Ok(())
    }

    fn dfs_check(
        &self,
        current_name: &str,
        depth: usize,
        templates: &HashMap<String, Template>,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
    ) -> Result<(), DomainError> {
        Self::ensure_depth_within_limit(depth)?;
        Self::ensure_not_in_stack(current_name, stack)?;

        if visited.contains(current_name) {
            return Ok(());
        }

        visited.insert(current_name.to_owned());
        stack.insert(current_name.to_owned());

        if let Some(template) = templates.get(current_name) {
            self.check_template_dependencies(
                template,
                depth,
                templates,
                visited,
                stack,
                current_name,
            )?;
        }

        stack.remove(current_name);
        Ok(())
    }

    /// Detects circular references in composition using DFS.
    ///
    /// # Errors
    /// Returns `DomainError::CompositionDepthExceeded` if depth > 5.
    /// Returns `DomainError::CircularComposition` if cycle detected.
    #[inline]
    pub fn detect_cycles(
        &self,
        templates: &HashMap<String, Template>,
    ) -> Result<(), DomainError> {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();

        self.dfs_check(
            &self.base_template,
            0,
            templates,
            &mut visited,
            &mut stack,
        )
    }

    fn ensure_depth_within_limit(depth: usize) -> Result<(), DomainError> {
        if depth > 5 {
            return Err(DomainError::CompositionDepthExceeded(depth));
        }
        Ok(())
    }

    fn ensure_not_in_stack(
        name: &str,
        stack: &HashSet<String>,
    ) -> Result<(), DomainError> {
        if stack.contains(name) {
            return Err(DomainError::CircularComposition(name.to_owned()));
        }
        Ok(())
    }

    /// Validates composition business rules.
    ///
    /// # Errors
    /// Returns `DomainError::VariableNotFound` if override key missing in base.
    /// Returns `DomainError::VariableTypeMismatch` if override value incompatible.
    #[inline]
    #[expect(clippy::iter_over_hash_type, reason = "Validation only")]
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
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::models::template::{Metadata, syntax::PlaceholderSyntax};

    #[test]
    fn should_detect_circular_composition() {
        let mut templates = HashMap::new();
        let base = Template {
            content: "content".to_owned(),
            extends: Some("A".to_owned()), // Circular extension
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
            includes: Vec::new(),
            variable_overrides: HashMap::new(),
        };

        let result = composition.detect_cycles(&templates);
        assert!(matches!(result, Err(DomainError::CircularComposition(_))));
    }

    #[test]
    fn should_detect_circular_include() {
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

        let result = composition.detect_cycles(&templates);
        assert!(matches!(result, Err(DomainError::CircularComposition(_))));
    }
}
