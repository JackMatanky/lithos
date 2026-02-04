//! Template aggregate root and composition logic.
//!
//! Handles template lifecycle, variable definitions, and hierarchical
//! composition through section insertion.
#![allow(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv Archive derive generates non-exhaustive archived types"
)]

use std::{collections::HashMap, sync::LazyLock};

use chrono::{DateTime, Utc};
use regex::Regex;
use uuid::Uuid;

use super::{
    composition::{Composition, InsertionPosition, Section},
    error::TemplateError,
    events::{Events, TemplateCreated},
    syntax::PlaceholderSyntax,
    validation::{validate_content, validate_structure},
    variable::VariableDefinition,
};
use crate::patterns;

const RESERVED_WORDS: &[&str] = &[
    "block",
    "call",
    "elif",
    "else",
    "endblock",
    "endcall",
    "endfilter",
    "endfor",
    "endmacro",
    "endif",
    "endwith",
    "extends",
    "false",
    "filter",
    "for",
    "if",
    "import",
    "in",
    "include",
    "macro",
    "none",
    "set",
    "true",
    "with",
];

/// Metadata for template management.
///
/// # Examples
/// ```
/// # use lithos_core::template::aggregate::Metadata;
/// let metadata = Metadata::default();
/// assert!(metadata.tags.is_empty(), "New metadata should have empty tags");
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
pub struct Metadata {
    /// Template description.
    pub description: Option<String>,
    /// Template version.
    pub version: Option<String>,
    /// Tags for categorization.
    pub tags: Vec<String>,
    /// Creation timestamp.
    #[rkyv(with = crate::ser::DateTimeAsI64)]
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp.
    #[rkyv(with = crate::ser::DateTimeAsI64)]
    pub updated_at: DateTime<Utc>,
}

/// Aggregate root representing a reusable template.
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
pub struct Template {
    /// UUID v7 identity.
    pub id: Uuid,
    /// Unique template name.
    pub name: String,
    /// Template content.
    pub content: String,
    /// Syntax used for placeholders.
    pub syntax: PlaceholderSyntax,
    /// Variable definitions with types and constraints.
    pub variables: HashMap<String, VariableDefinition>,
    /// Optional parent template for composition.
    pub extends: Option<String>,
    /// Metadata for template management.
    pub metadata: Metadata,
    /// Domain events pending emission.
    #[rkyv(with = rkyv::with::Skip)]
    #[serde(skip)]
    pub pending_events: Vec<Events>,
}

impl Default for Metadata {
    #[inline]
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

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Function ordering optimized for logical flow; matching on \
              reference to enum avoids borrow checker friction"
)]
impl Template {
    /// Adds a domain event to the pending events collection.
    #[inline]
    fn add_event(&mut self, event: Events) {
        self.pending_events.push(event);
    }

    /// Composes a template from a base and a composition.
    ///
    /// # Errors
    /// Returns `TemplateError::ValidationFailed` if composition validation
    /// fails.
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
    /// let mut templates = HashMap::new();
    /// templates.insert(base.name.to_string(), base.clone());
    /// let composed = Template::compose(&base, &composition, &templates)?;
    /// assert!(composed.extends().is_some(), "Composed template should extend base");
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    #[inline]
    pub fn compose(
        base: &Self,
        composition: &Composition,
        templates: &HashMap<String, Template>,
    ) -> Result<Self, TemplateError> {
        composition.validate(base)?;
        composition.detect_cycles(templates)?;

        let mut final_content = base.content.clone();
        base.apply_sections(
            &mut final_content,
            &composition.additional_sections,
        );

        let id = Uuid::now_v7();
        let name = format!("{}-composed", base.name);
        let mut template = Self {
            content: final_content,
            extends: Some(base.name.clone()),
            id,
            metadata: Metadata::default(),
            name: name.clone(),
            pending_events: vec![],
            syntax: base.syntax.clone(),
            variables: base.variables.clone(),
        };

        template.add_event(Events::TemplateCreated(TemplateCreated::new(
            id,
            name,
            chrono::Utc::now().timestamp(),
        )));

        Ok(template)
    }

    /// Returns the template's content.
    #[inline]
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns true if the template defines any variables.
    #[inline]
    #[must_use]
    pub fn has_variables(&self) -> bool {
        !self.variables.is_empty()
    }

    /// Returns the name of the template this one extends, if any.
    #[inline]
    #[must_use]
    pub fn extends(&self) -> Option<&str> {
        self.extends.as_deref()
    }

    /// Returns the template's unique identifier.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Returns the template's metadata.
    #[inline]
    #[must_use]
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Returns the template's unique name.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Creates a new template aggregate with validation.
    ///
    /// # Errors
    /// Returns `TemplateError` if validation fails (name format, size limits,
    /// etc).
    ///
    /// # Examples
    /// ```
    /// # use lithos_core::template::aggregate::{Template, Metadata};
    /// # use lithos_core::template::variable::VariableDefinition;
    /// # use std::collections::HashMap;
    /// let mut variables = HashMap::new();
    /// variables.insert("title".to_string(), VariableDefinition::String {
    ///     default: Some("Daily".to_string()),
    ///     max_length: None,
    ///     min_length: None,
    ///     pattern: None,
    /// });
    /// let template = Template::new(
    ///     "daily".to_string(),
    ///     "# {{title}}".to_string(),
    ///     variables,
    ///     None,
    ///     Metadata::default(),
    /// )
    /// .unwrap();
    /// assert_eq!(template.name, "daily");
    /// ```
    #[inline]
    pub fn new(
        name: String,
        content: String,
        variables: HashMap<String, VariableDefinition>,
        extends: Option<String>,
        metadata: Metadata,
    ) -> Result<Self, TemplateError> {
        Self::validate_name(&name)?;
        validate_content(&content)?;
        Self::validate_variable_definitions(&variables)?;

        let id = Uuid::now_v7();
        let mut template = Self {
            content,
            extends,
            id,
            metadata,
            name: name.clone(),
            pending_events: vec![],
            syntax: PlaceholderSyntax::default(),
            variables,
        };

        template.validate()?;

        template.add_event(Events::TemplateCreated(TemplateCreated::new(
            id,
            name,
            chrono::Utc::now().timestamp(),
        )));

        Ok(template)
    }

    /// Returns a reference to pending domain events.
    #[inline]
    #[must_use]
    pub fn pending_events(&self) -> &[Events] {
        &self.pending_events
    }

    /// Returns the template's placeholder syntax.
    #[inline]
    #[must_use]
    pub const fn syntax(&self) -> &PlaceholderSyntax {
        &self.syntax
    }

    /// Returns and clears pending domain events.
    #[inline]
    #[must_use]
    pub fn take_events(&mut self) -> Vec<Events> {
        std::mem::take(&mut self.pending_events)
    }

    /// Applies additional sections to content.
    fn apply_sections(&self, content: &mut String, sections: &[Section]) {
        for section in sections {
            match &section.position {
                InsertionPosition::Beginning => {
                    content.insert(0, '\n');
                    content.insert_str(0, &section.content);
                }
                InsertionPosition::End => {
                    content.push('\n');
                    content.push_str(&section.content);
                }
                InsertionPosition::BeforeVariable(var_name) => {
                    self.insert_relative_to_variable(
                        content,
                        var_name,
                        &section.content,
                        false,
                    );
                }
                InsertionPosition::AfterVariable(var_name) => {
                    self.insert_relative_to_variable(
                        content,
                        var_name,
                        &section.content,
                        true,
                    );
                }
            }
        }
    }

    fn insert_relative_to_variable(
        &self,
        content: &mut String,
        var_name: &str,
        section_content: &str,
        after: bool,
    ) {
        let placeholder = self.syntax.wrap(var_name);
        if let Some(pos) = content.find(&placeholder) {
            if after {
                let insert_pos = pos.saturating_add(placeholder.len());
                content.insert(insert_pos, '\n');
                content
                    .insert_str(insert_pos.saturating_add(1), section_content);
            } else {
                content.insert_str(pos, section_content);
                content.insert(pos.saturating_add(section_content.len()), '\n');
            }
        }
    }

    /// Validates template constraints.
    ///
    /// # Errors
    /// Returns `TemplateError::ValidationFailed` if placeholders are
    /// unbalanced.
    ///
    /// # Examples
    /// ```
    /// # use lithos_core::template::aggregate::{Template, Metadata};
    /// # use std::collections::HashMap;
    /// let template = Template::new(
    ///     "basic".to_string(),
    ///     "Hello {{name}}".to_string(),
    ///     HashMap::new(),
    ///     None,
    ///     Metadata::default(),
    /// )
    /// .unwrap();
    /// template.validate().unwrap();
    /// ```
    #[inline]
    pub fn validate(&self) -> Result<(), TemplateError> {
        validate_structure(
            &self.content,
            &self.syntax.prefix,
            &self.syntax.suffix,
        )?;
        Ok(())
    }

    /// Validates a template name according to domain constraints.
    ///
    /// # Errors
    /// Returns `TemplateError` if the name is empty, too long, or contains
    /// invalid characters.
    #[inline]
    pub fn validate_name(name: &str) -> Result<(), TemplateError> {
        static RE: LazyLock<Result<Regex, regex::Error>> =
            LazyLock::new(|| Regex::new(patterns::ALPHANUMERIC_NAME));

        if name.is_empty() {
            return Err(TemplateError::EmptyTemplateName);
        }
        if name.len() > 64 {
            return Err(TemplateError::TemplateNameTooLong(name.len()));
        }

        let re = RE.as_ref().map_err(|error| {
            TemplateError::ValidationFailed(format!(
                "Invalid template name regex: {error}"
            ))
        })?;

        if !re.is_match(name) {
            return Err(TemplateError::InvalidTemplateName(name.to_owned()));
        }
        Ok(())
    }

    #[expect(
        clippy::iter_over_hash_type,
        reason = "Validation checks all variable definitions for correctness. \
                  HashMap iteration order is irrelevant—all entries must pass \
                  validation regardless of order."
    )]
    fn validate_variable_definitions(
        variables: &HashMap<String, VariableDefinition>,
    ) -> Result<(), TemplateError> {
        Self::validate_max_variables_not_exceeded(variables.len())?;

        for var_name in variables.keys() {
            Self::validate_variable_name(var_name)?;
        }
        Ok(())
    }

    /// Validates a variable name according to domain constraints.
    ///
    /// # Errors
    /// Returns `TemplateError` if the name is empty, too long, contains
    /// invalid characters, or is a reserved word.
    #[inline]
    pub fn validate_variable_name(name: &str) -> Result<(), TemplateError> {
        static RE: LazyLock<Result<Regex, regex::Error>> =
            LazyLock::new(|| Regex::new(patterns::IDENTIFIER_NAME));

        if name.is_empty() {
            return Err(TemplateError::EmptyVariableName);
        }
        if name.len() > 32 {
            return Err(TemplateError::VariableNameTooLong(name.len()));
        }

        let re = RE.as_ref().map_err(|error| {
            TemplateError::ValidationFailed(format!(
                "Invalid variable name regex: {error}"
            ))
        })?;

        if !re.is_match(name) {
            return Err(TemplateError::InvalidVariableName(name.to_owned()));
        }
        Self::validate_variable_name_not_reserved(name)?;
        Ok(())
    }

    fn validate_max_variables_not_exceeded(
        count: usize,
    ) -> Result<(), TemplateError> {
        if count > 50 {
            return Err(TemplateError::MaxVariablesExceeded(count));
        }
        Ok(())
    }

    fn validate_variable_name_not_reserved(
        name: &str,
    ) -> Result<(), TemplateError> {
        if RESERVED_WORDS.contains(&name) {
            return Err(TemplateError::InvalidVariableName(format!(
                "Variable name '{name}' is a reserved word"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    mod new {
        use super::*;

        /// 3.4-UNIT-022: `accessors_return_expected_values`.
        /// Priority: P1.
        #[test]
        fn accessors_return_expected_values() {
            // GIVEN: a new template aggregate
            let template_result = Template::new(
                "base".to_owned(),
                "Hello".to_owned(),
                HashMap::new(),
                None,
                Metadata::default(),
            );
            assert!(
                template_result.is_ok(),
                "Expected valid template, got: {template_result:?}"
            );
            let Ok(mut template) = template_result else {
                return;
            };

            // WHEN: reading template accessors
            let event_count = template.pending_events().len();

            // THEN: accessors expose expected data
            assert_eq!(template.name, "base", "Template name should be 'base'");
            assert_eq!(
                template.content, "Hello",
                "Template content should be 'Hello'"
            );
            assert!(
                template.extends().is_none(),
                "Template should not extend another template"
            );
            assert!(
                !template.has_variables(),
                "Template should have no variables"
            );
            assert_eq!(event_count, 1, "Template should have 1 pending event");
            assert_eq!(
                template.take_events().len(),
                1,
                "Taking events should return 1 event"
            );
        }

        /// 3.4-UNIT-023: `should_reject_template_when_name_format_is_invalid`.
        /// Priority: P0.
        #[test]
        fn should_reject_template_when_name_format_is_invalid() {
            // GIVEN: invalid template names
            let invalid_long_name = "a".repeat(65);
            let names = vec!["", "Invalid Name", "name!", &invalid_long_name];

            // WHEN: creating templates with invalid names
            for name in names {
                let result = Template::new(
                    name.to_owned(),
                    "content".to_owned(),
                    HashMap::new(),
                    None,
                    Metadata::default(),
                );

                // THEN: validation rejects the name
                assert!(result.is_err(), "Expected error for name: {name}");
            }
        }

        /// 3.4-UNIT-024: `should_reject_template_when_unbalanced_placeholders`.
        /// Priority: P0.
        #[test]
        fn should_reject_template_when_unbalanced_placeholders() {
            // GIVEN: a template with unbalanced placeholders
            let result = Template::new(
                "unbalanced".to_owned(),
                "{{open but no close".to_owned(),
                HashMap::new(),
                None,
                Metadata::default(),
            );

            // WHEN: validation runs during construction
            assert!(
                result.is_err(),
                "Expected unbalanced template to fail, got: {result:?}"
            );
            let Err(error) = result else {
                return;
            };

            // THEN: a validation error describes the unbalanced syntax
            assert!(error.to_string().contains("Unbalanced"));
        }
    }

    // 3.4-UNIT-025: `should_validate_template_name_format_across_edge_cases`.
    // Priority: P2.
    #[test]
    fn should_validate_template_name_format_across_edge_cases() {
        use proptest::{prelude::*, test_runner::TestRunner};

        let mut runner = TestRunner::deterministic();
        let strategy = "[a-zA-Z0-9_-]{1,64}";

        let run_result = runner.run(&strategy, |name| {
            // GIVEN: a generated valid identifier
            // WHEN: constructing a template with the identifier
            let result = Template::new(
                name.clone(),
                "content".to_owned(),
                HashMap::new(),
                None,
                Metadata::default(),
            );

            // THEN: construction succeeds
            prop_assert!(
                result.is_ok(),
                "Template with valid name '{}' should be created",
                name
            );
            Ok(())
        });
        assert!(
            run_result.is_ok(),
            "Proptest run should succeed, got: {run_result:?}"
        );
    }

    /// 3.4-UNIT-026: `should_compose_templates_with_sections`.
    /// Priority: P1.
    #[test]
    fn should_compose_templates_with_sections() {
        // GIVEN: a base template and a composition
        let base_result = Template::new(
            "base".to_owned(),
            "Base: {{v}}".to_owned(),
            [("v".to_owned(), VariableDefinition::Boolean {
                default: None,
            })]
            .into_iter()
            .collect(),
            None,
            Metadata::default(),
        );
        assert!(
            base_result.is_ok(),
            "Expected valid base template, got: {base_result:?}"
        );
        let Ok(base) = base_result else {
            return;
        };

        let composition = Composition {
            base_template: "base".to_owned(),
            additional_sections: vec![
                Section {
                    name: "top".to_owned(),
                    content: "Header".to_owned(),
                    position: InsertionPosition::Beginning,
                },
                Section {
                    name: "bottom".to_owned(),
                    content: "Footer".to_owned(),
                    position: InsertionPosition::End,
                },
                Section {
                    name: "mid".to_owned(),
                    content: "Inside".to_owned(),
                    position: InsertionPosition::AfterVariable("v".to_owned()),
                },
            ],
            includes: vec![],
            variable_overrides: [(
                "v".to_owned(),
                serde_json::Value::Bool(true),
            )]
            .into_iter()
            .collect(),
        };

        let templates =
            [("base".to_owned(), base.clone())].into_iter().collect();

        // WHEN: composing the template
        let composed_result =
            Template::compose(&base, &composition, &templates);
        assert!(
            composed_result.is_ok(),
            "Expected compose to succeed, got: {composed_result:?}"
        );
        let Ok(composed) = composed_result else {
            return;
        };

        // THEN: content is correctly assembled
        assert!(composed.content.starts_with("Header\n"));
        assert!(composed.content.ends_with("\nFooter"));
        assert!(composed.content.contains("Base: {{v}}\nInside"));
        assert_eq!(composed.extends(), Some("base"));
    }

    /// 3.4-UNIT-027: `apply_sections_handles_missing_variable`.
    /// Priority: P2.
    #[test]
    fn apply_sections_handles_missing_variable() {
        // GIVEN: a template without variables
        let base_result = Template::new(
            "b".to_owned(),
            "no var".to_owned(),
            HashMap::new(),
            None,
            Metadata::default(),
        );
        assert!(
            base_result.is_ok(),
            "Expected valid base template, got: {base_result:?}"
        );
        let Ok(base) = base_result else {
            return;
        };

        // WHEN: applying a section relative to a missing variable
        let mut content = "no var".to_owned();
        let sections = vec![Section {
            name: "s".to_owned(),
            content: "cont".to_owned(),
            position: InsertionPosition::AfterVariable("missing".to_owned()),
        }];
        base.apply_sections(&mut content, &sections);

        // THEN: content remains unchanged
        assert_eq!(content, "no var");
    }
}
