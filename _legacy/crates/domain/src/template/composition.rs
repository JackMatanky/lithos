use std::collections::{HashMap, HashSet};

use super::aggregate::Template;
use crate::errors::DomainError;

/// Template composition for modular template building.
///
/// # Examples
/// ```ignore
/// # use lithos_domain::{
///     InsertionPosition,
///     Template,
///     TemplateMetadata,
///     TemplateSection
/// };
/// # use lithos_domain::TemplateComposition;
/// # use std::collections::HashMap;
/// # fn run() -> Result<(), lithos_domain::DomainError> {
/// let base = Template::new(
///     "base".to_string(),
///     "Hello {{name}}".to_string(),
///     HashMap::new(),
///     None,
///     TemplateMetadata::default(),
/// )?;
/// let composition = TemplateComposition {
///     additional_sections: vec![TemplateSection {
///         name: "footer".to_string(),
///         content: "--".to_string(),
///         position: InsertionPosition::End,
///     }],
///     base_template: base.name().to_string(),
///     includes: Vec::new(),
///     variable_overrides: HashMap::new(),
/// };
/// assert_eq!(composition.additional_sections.len(), 1);
/// # Ok(())
/// # }
/// # run().unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Composition {
    /// Base template name.
    pub base_template: String,
    /// Additional content sections to append.
    pub additional_sections: Vec<Section>,
    /// Child templates to include.
    pub includes: Vec<String>,
    /// Variable overrides for base template.
    pub variable_overrides: HashMap<String, serde_json::Value>,
}

/// Internal context for DFS cycle detection.
struct DfsContext<'context> {
    templates: &'context HashMap<String, Template>,
    visited: &'context mut HashSet<String>,
    stack: &'context mut HashSet<String>,
}

#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Function ordering optimized for logical flow over strict \
              alphabetical order"
)]
impl Composition {
    fn check_template_dependencies(
        &self,
        template: &Template,
        depth: usize,
        ctx: &mut DfsContext<'_>,
        current_name: &str,
    ) -> Result<(), DomainError> {
        if let Some(parent_name) = template.extends() {
            #[expect(
                clippy::arithmetic_side_effects,
                reason = "DFS depth tracking: depth + 1 cannot overflow. Max \
                          recursion depth is validated (MAX_COMPOSITION_DEPTH \
                          = 10), so depth stays bounded and safe."
            )]
            self.dfs_check(parent_name, depth + 1, ctx)?;
        }

        if current_name == self.base_template {
            for include in &self.includes {
                #[expect(
                    clippy::arithmetic_side_effects,
                    reason = "DFS depth tracking: depth + 1 cannot overflow. \
                              Max recursion depth is validated \
                              (MAX_COMPOSITION_DEPTH = 10), so depth stays \
                              bounded and safe."
                )]
                self.dfs_check(include, depth + 1, ctx)?;
            }
        }
        Ok(())
    }

    fn dfs_check(
        &self,
        current_name: &str,
        depth: usize,
        ctx: &mut DfsContext<'_>,
    ) -> Result<(), DomainError> {
        Self::validate_depth_within_limit(depth)?;
        Self::validate_not_in_stack(current_name, ctx.stack)?;

        if ctx.visited.contains(current_name) {
            return Ok(());
        }

        ctx.visited.insert(current_name.to_owned());
        ctx.stack.insert(current_name.to_owned());

        if let Some(template) = ctx.templates.get(current_name) {
            self.check_template_dependencies(
                template,
                depth,
                ctx,
                current_name,
            )?;
        }

        ctx.stack.remove(current_name);
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
        let mut ctx = DfsContext {
            templates,
            visited: &mut visited,
            stack: &mut stack,
        };

        self.dfs_check(&self.base_template, 0, &mut ctx)
    }

    fn validate_depth_within_limit(depth: usize) -> Result<(), DomainError> {
        if depth > 5 {
            return Err(DomainError::CompositionDepthExceeded(depth));
        }
        Ok(())
    }

    fn validate_not_in_stack(
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
    /// Returns `DomainError::VariableTypeMismatch` if override value
    /// incompatible.
    #[inline]
    #[expect(
        clippy::iter_over_hash_type,
        reason = "Validation checks all variable overrides against base \
                  template definitions. HashMap iteration order is \
                  irrelevant—all entries must be validated."
    )]
    pub fn validate(&self, base: &Template) -> Result<(), DomainError> {
        for (name, value) in &self.variable_overrides {
            let def = base
                .variables()
                .get(name)
                .ok_or_else(|| DomainError::VariableNotFound(name.clone()))?;
            def.validate_value(value).map_err(|e| {
                if let DomainError::InvalidType {
                    expected,
                    ..
                } = e
                {
                    DomainError::VariableTypeMismatch {
                        name: name.clone().into(),
                        expected: expected.into(),
                        actual: format!("{value:?}").into(),
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
    /// Section name.
    pub name: String,
    /// Section content.
    pub content: String,
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
#[expect(
    clippy::disallowed_methods,
    reason = "Test module uses Result::expect() for ergonomic arrangement and \
              assertions. Acceptable in test-only code paths."
)]
mod tests {
    use super::*;
    use crate::{InputSpec, template::TemplateMetadata};

    /// 3.4-UNIT-028: `should_detect_circular_composition`.
    /// Priority: P0.
    #[test]
    fn should_detect_circular_composition() {
        // GIVEN a template that extends itself
        let mut templates = HashMap::new();
        let base = Template::new(
            "A".to_owned(),
            "content".to_owned(),
            HashMap::new(),
            Some("A".to_owned()),
            TemplateMetadata::default(),
        )
        .expect("Valid template setup");
        templates.insert("A".to_owned(), base);

        let composition = Composition {
            additional_sections: Vec::new(),
            base_template: "A".to_owned(),
            includes: Vec::new(),
            variable_overrides: HashMap::new(),
        };

        // WHEN detecting cycles
        let result = composition.detect_cycles(&templates);

        // THEN it must return a CircularComposition error
        assert!(matches!(result, Err(DomainError::CircularComposition(_))));
    }

    /// 3.4-UNIT-029: `should_detect_circular_include`.
    /// Priority: P0.
    #[test]
    fn should_detect_circular_include() {
        // GIVEN a base template with a self-include
        let mut templates = HashMap::new();
        let base = Template::new(
            "A".to_owned(),
            "content".to_owned(),
            HashMap::new(),
            None,
            TemplateMetadata::default(),
        )
        .expect("Valid template setup");
        templates.insert("A".to_owned(), base);

        let composition = Composition {
            additional_sections: Vec::new(),
            base_template: "A".to_owned(),
            includes: vec!["A".to_owned()],
            variable_overrides: HashMap::new(),
        };

        // WHEN detecting cycles
        let result = composition.detect_cycles(&templates);

        // THEN it reports a circular composition error
        assert!(matches!(result, Err(DomainError::CircularComposition(_))));
    }

    /// 3.4-UNIT-030: `validate_rejects_variable_type_mismatch`.
    /// Priority: P1.
    #[test]
    fn validate_rejects_variable_type_mismatch() {
        // GIVEN: a base template with a string variable
        let mut variables = HashMap::new();
        variables.insert("title".to_owned(), InputSpec::String {
            default: None,
            max_length: None,
            min_length: None,
            pattern: None,
        });
        let base = Template::new(
            "base".to_owned(),
            "Hello {{title}}".to_owned(),
            variables,
            None,
            TemplateMetadata::default(),
        )
        .unwrap();

        // WHEN: overriding with an incompatible value
        let mut overrides = HashMap::new();
        overrides.insert("title".to_owned(), serde_json::json!(42i64));
        let composition = Composition {
            additional_sections: Vec::new(),
            base_template: "base".to_owned(),
            includes: Vec::new(),
            variable_overrides: overrides,
        };

        // THEN: validation reports a type mismatch
        let result = composition.validate(&base);
        assert!(matches!(
            result,
            Err(DomainError::VariableTypeMismatch { .. })
        ));
    }
}
