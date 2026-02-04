//! Template composition for modular template building.
//!
//! This module defines composition types with rkyv serialization support.
#![allow(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv Archive derive generates non-exhaustive archived types"
)]

use std::collections::{HashMap, HashSet};

use super::{aggregate::Template, error::TemplateError};

/// Template composition for modular template building.
///
/// # Examples
/// ```ignore
/// # use lithos_core::template::aggregate::{Template, Metadata};
/// # use lithos_core::template::composition::{Composition, Section, InsertionPosition};
/// # use std::collections::HashMap;
/// # fn run() -> Result<(), lithos_core::template::error::TemplateError> {
/// let base = Template::new(
///     "base".to_string(),
///     "Hello {{name}}".to_string(),
///     HashMap::new(),
///     None,
///     Metadata::default(),
/// )?;
/// let composition = Composition {
///     additional_sections: vec![Section {
///         name: "footer".to_string(),
///         content: "--".to_string(),
///         position: InsertionPosition::End,
///     }],
///     base_template: base.name.to_string(),
///     includes: Vec::new(),
///     variable_overrides: HashMap::new(),
/// };
/// assert_eq!(composition.additional_sections.len(), 1);
/// # Ok(())
/// # }
/// # run().unwrap();
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Composition {
    /// Base template name.
    pub base_template: String,
    /// Additional content sections to append.
    pub additional_sections: Vec<Section>,
    /// Child templates to include.
    pub includes: Vec<String>,
    /// Variable overrides for base template.
    #[rkyv(with = rkyv::with::Skip)]
    pub variable_overrides: HashMap<String, serde_json::Value>,
}

/// Internal context for DFS cycle detection.
struct DfsContext<'context> {
    templates: &'context HashMap<String, Template>,
    visited: &'context mut HashSet<String>,
    stack: &'context mut HashSet<String>,
}

/// Insertion point for template sections.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
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

/// Template section for composition.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Section {
    /// Section name.
    pub name: String,
    /// Section content.
    pub content: String,
    /// Insertion point.
    pub position: InsertionPosition,
}

impl Composition {
    fn check_template_dependencies(
        &self,
        template: &Template,
        depth: usize,
        ctx: &mut DfsContext<'_>,
        current_name: &str,
    ) -> Result<(), TemplateError> {
        if let Some(parent_name) = template.extends() {
            self.dfs_check(parent_name, depth.saturating_add(1), ctx)?;
        }

        if current_name == self.base_template {
            for include in &self.includes {
                self.dfs_check(include, depth.saturating_add(1), ctx)?;
            }
        }
        Ok(())
    }

    fn dfs_check(
        &self,
        current_name: &str,
        depth: usize,
        ctx: &mut DfsContext<'_>,
    ) -> Result<(), TemplateError> {
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
    /// Returns `TemplateError::CompositionDepthExceeded` if depth > 5.
    /// Returns `TemplateError::CircularComposition` if cycle detected.
    #[inline]
    pub fn detect_cycles(
        &self,
        templates: &HashMap<String, Template>,
    ) -> Result<(), TemplateError> {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        let mut ctx = DfsContext {
            templates,
            visited: &mut visited,
            stack: &mut stack,
        };

        self.dfs_check(&self.base_template, 0, &mut ctx)
    }

    fn validate_depth_within_limit(depth: usize) -> Result<(), TemplateError> {
        if depth > 5 {
            return Err(TemplateError::CompositionDepthExceeded(depth));
        }
        Ok(())
    }

    fn validate_not_in_stack(
        name: &str,
        stack: &HashSet<String>,
    ) -> Result<(), TemplateError> {
        if stack.contains(name) {
            return Err(TemplateError::CircularComposition(name.to_owned()));
        }
        Ok(())
    }

    /// Validates composition constraints.
    ///
    /// # Errors
    /// Returns `TemplateError::VariableNotFound` if override key missing in
    /// base. Returns `TemplateError::VariableTypeMismatch` if override
    /// value incompatible.
    #[inline]
    #[expect(
        clippy::iter_over_hash_type,
        reason = "HashMap iteration order is irrelevant."
    )]
    pub fn validate(&self, base: &Template) -> Result<(), TemplateError> {
        for (name, value) in &self.variable_overrides {
            let def = base
                .variables
                .get(name)
                .ok_or_else(|| TemplateError::VariableNotFound(name.clone()))?;
            def.validate_value(value).map_err(|e| {
                if let TemplateError::InvalidType {
                    expected,
                    ..
                } = e
                {
                    TemplateError::VariableTypeMismatch {
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

#[cfg(test)]
mod tests {
    /// Test fixtures and builders for Template composition tests.
    mod fixtures {
        use super::super::*;
        use crate::template::{
            aggregate::Metadata, variable::VariableDefinition,
        };

        /// Builder for creating test Template instances.
        pub struct TemplateTestBuilder {
            name: String,
            content: String,
            variables: HashMap<String, VariableDefinition>,
            extends: Option<String>,
            metadata: Metadata,
        }

        impl TemplateTestBuilder {
            pub fn new(name: &str) -> Self {
                Self {
                    name: name.to_owned(),
                    content: "default content".to_owned(),
                    variables: HashMap::new(),
                    extends: None,
                    metadata: Metadata::default(),
                }
            }

            pub fn with_content(mut self, content: &str) -> Self {
                self.content = content.to_owned();
                self
            }

            pub fn extending(mut self, parent: &str) -> Self {
                self.extends = Some(parent.to_owned());
                self
            }

            pub fn with_variable(
                mut self,
                name: &str,
                var: VariableDefinition,
            ) -> Self {
                self.variables.insert(name.to_owned(), var);
                self
            }

            pub fn build(self) -> Result<Template, TemplateError> {
                Template::new(
                    self.name,
                    self.content,
                    self.variables,
                    self.extends,
                    self.metadata,
                )
            }
        }
    }

    use super::*;
    use crate::template::variable::VariableDefinition;

    /// 3.4-UNIT-028: `should_detect_circular_composition`.
    /// Priority: P0.
    #[test]
    fn should_detect_circular_composition() {
        use fixtures::TemplateTestBuilder;

        // GIVEN a template that extends itself
        let mut templates = HashMap::new();
        let base_result = TemplateTestBuilder::new("A")
            .with_content("content")
            .extending("A")
            .build();
        assert!(base_result.is_ok(), "Valid template setup: {base_result:?}");
        let Ok(base) = base_result else {
            return;
        };
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
        assert!(
            matches!(result, Err(TemplateError::CircularComposition(_))),
            "Expected CircularComposition error for self-extending template, \
             got: {result:?}"
        );
    }

    /// 3.4-UNIT-029: `should_detect_circular_include`.
    /// Priority: P0.
    #[test]
    fn should_detect_circular_include() {
        use fixtures::TemplateTestBuilder;

        // GIVEN a base template with a self-include
        let mut templates = HashMap::new();
        let base_result =
            TemplateTestBuilder::new("A").with_content("content").build();
        assert!(base_result.is_ok(), "Valid template setup: {base_result:?}");
        let Ok(base) = base_result else {
            return;
        };
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
        assert!(
            matches!(result, Err(TemplateError::CircularComposition(_))),
            "Expected CircularComposition error for self-including template, \
             got: {result:?}"
        );
    }

    /// 3.4-UNIT-030: `validate_rejects_variable_type_mismatch`.
    /// Priority: P1.
    #[test]
    fn validate_rejects_variable_type_mismatch() {
        use fixtures::TemplateTestBuilder;

        // GIVEN: a base template with a string variable
        let base = TemplateTestBuilder::new("base")
            .with_content("Hello {{title}}")
            .with_variable("title", VariableDefinition::String {
                default: None,
                max_length: None,
                min_length: None,
                pattern: None,
            })
            .build();
        assert!(base.is_ok(), "Valid template setup: {base:?}");
        let Ok(base) = base else {
            return;
        };

        // WHEN: overriding with an incompatible value
        let mut overrides = HashMap::new();
        overrides.insert(
            "title".to_owned(),
            serde_json::Value::Number(serde_json::Number::from(42i64)),
        );
        let composition = Composition {
            additional_sections: Vec::new(),
            base_template: "base".to_owned(),
            includes: Vec::new(),
            variable_overrides: overrides,
        };

        // THEN: validation reports a type mismatch
        let result = composition.validate(&base);
        assert!(
            matches!(result, Err(TemplateError::VariableTypeMismatch { .. })),
            "Expected VariableTypeMismatch error when overriding String with \
             Number, got: {result:?}"
        );
    }
}
