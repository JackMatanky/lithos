use std::collections::HashMap;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{
    composition::{Composition, InsertionPosition, Section},
    events::{TemplateCreated, TemplateEvents},
    syntax::PlaceholderSyntax,
    validation::{validate_content, validate_structure},
    variable::VariableDefinition,
};
use crate::{errors::DomainError, validation};

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

/// Aggregate root representing a reusable template.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Meaningful logical ordering of aggregate fields"
)]
pub struct Template {
    /// UUID v7 identity.
    id: Uuid,
    /// Unique template name.
    name: String,
    /// Template content.
    content: String,
    /// Syntax used for placeholders.
    syntax: PlaceholderSyntax,
    /// Variable definitions with types and constraints.
    variables: HashMap<String, VariableDefinition>,
    /// Optional parent template for composition.
    extends: Option<String>,
    /// Metadata for template management.
    metadata: Metadata,
    /// Domain events pending emission.
    #[serde(skip)]
    pending_events: Vec<TemplateEvents>,
}

#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Function ordering optimized for logical flow over strict alphabetical order"
)]
#[expect(
    clippy::pattern_type_mismatch,
    reason = "Matching on reference to enum with owned variants for borrow checker compliance"
)]
impl Template {
    /// Adds a domain event to the pending events collection.
    #[inline]
    fn add_event(&mut self, event: TemplateEvents) {
        self.pending_events.push(event);
    }

    /// Composes a template from a base and a composition.
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` if composition validation fails.
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_domain::{Template, TemplateComposition, TemplateMetadata, TemplateSection, InsertionPosition};
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
    /// let mut templates = HashMap::new();
    /// templates.insert(base.name().to_string(), base.clone());
    /// let composed = Template::compose(&base, &composition, &templates)?;
    /// assert!(composed.extends().is_some());
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    #[inline]
    pub fn compose(
        base: &Self,
        composition: &Composition,
        templates: &HashMap<String, Template>,
    ) -> Result<Self, DomainError> {
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

        template.add_event(TemplateEvents::TemplateCreated(
            TemplateCreated::new(id, name, chrono::Utc::now().timestamp()),
        ));

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
    /// Returns `DomainError` if validation fails (name format, size limits, etc).
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::{Template, VariableDefinition, TemplateMetadata};
    /// # use std::collections::HashMap;
    /// let mut variables = HashMap::new();
    /// variables.insert(
    ///     "title".to_string(),
    ///     VariableDefinition::String {
    ///         default: Some("Daily".to_string()),
    ///         max_length: None,
    ///         min_length: None,
    ///         pattern: None,
    ///     },
    /// );
    /// let template = Template::new(
    ///     "daily".to_string(),
    ///     "# {{title}}".to_string(),
    ///     variables,
    ///     None,
    ///     TemplateMetadata::default(),
    /// )
    /// .unwrap();
    /// assert_eq!(template.name(), "daily");
    /// ```
    #[inline]
    pub fn new(
        name: String,
        content: String,
        variables: HashMap<String, VariableDefinition>,
        extends: Option<String>,
        metadata: Metadata,
    ) -> Result<Self, DomainError> {
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

        template.add_event(TemplateEvents::TemplateCreated(
            TemplateCreated::new(id, name, chrono::Utc::now().timestamp()),
        ));

        Ok(template)
    }

    /// Returns a reference to pending domain events.
    #[inline]
    #[must_use]
    pub fn pending_events(&self) -> &[TemplateEvents] {
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
    pub fn take_events(&mut self) -> Vec<TemplateEvents> {
        std::mem::take(&mut self.pending_events)
    }

    /// Returns the template's variable definitions.
    #[inline]
    #[must_use]
    pub const fn variables(&self) -> &HashMap<String, VariableDefinition> {
        &self.variables
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
                #[expect(
                    clippy::arithmetic_side_effects,
                    reason = "Index logic"
                )]
                let insert_pos = pos + placeholder.len();
                content.insert(insert_pos, '\n');
                #[expect(
                    clippy::arithmetic_side_effects,
                    reason = "Index logic"
                )]
                content.insert_str(insert_pos + 1, section_content);
            } else {
                content.insert_str(pos, section_content);
                #[expect(
                    clippy::arithmetic_side_effects,
                    reason = "Index logic"
                )]
                content.insert(pos + section_content.len(), '\n');
            }
        }
    }

    /// Validates template business rules.
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` if placeholders are unbalanced.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::{Template, TemplateMetadata};
    /// # use std::collections::HashMap;
    /// let template = Template::new(
    ///     "basic".to_string(),
    ///     "Hello {{name}}".to_string(),
    ///     HashMap::new(),
    ///     None,
    ///     TemplateMetadata::default(),
    /// )
    /// .unwrap();
    /// template.validate().unwrap();
    /// ```
    #[inline]
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_structure(
            &self.content,
            &self.syntax.prefix,
            &self.syntax.suffix,
        )?;
        Ok(())
    }

    fn validate_name(name: &str) -> Result<(), DomainError> {
        if name.is_empty() {
            return Err(DomainError::EmptyTemplateName);
        }
        if name.len() > 64 {
            return Err(DomainError::TemplateNameTooLong(name.len()));
        }
        if !validation::is_alphanumeric_name(name) {
            return Err(DomainError::InvalidTemplateName(name.to_owned()));
        }
        Ok(())
    }

    #[expect(clippy::iter_over_hash_type, reason = "Validation only")]
    fn validate_variable_definitions(
        variables: &HashMap<String, VariableDefinition>,
    ) -> Result<(), DomainError> {
        Self::validate_max_variables_not_exceeded(variables.len())?;

        for var_name in variables.keys() {
            Self::validate_variable_name(var_name)?;
        }
        Ok(())
    }

    fn validate_variable_name(name: &str) -> Result<(), DomainError> {
        if name.is_empty() {
            return Err(DomainError::EmptyVariableName);
        }
        if name.len() > 32 {
            return Err(DomainError::VariableNameTooLong(name.len()));
        }
        if !validation::is_identifier_name(name) {
            return Err(DomainError::InvalidVariableName(name.to_owned()));
        }
        Self::validate_variable_name_not_reserved(name)?;
        Ok(())
    }

    fn validate_max_variables_not_exceeded(
        count: usize,
    ) -> Result<(), DomainError> {
        if count > 50 {
            return Err(DomainError::MaxVariablesExceeded(count));
        }
        Ok(())
    }

    fn validate_variable_name_not_reserved(
        name: &str,
    ) -> Result<(), DomainError> {
        if RESERVED_WORDS.contains(&name) {
            return Err(DomainError::InvalidVariableName(format!(
                "Variable name '{name}' is a reserved word"
            )));
        }
        Ok(())
    }
}

/// Metadata for template management.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Metadata {
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Template description.
    pub description: Option<String>,
    /// Tags for categorization.
    pub tags: Vec<String>,
    /// Last modification timestamp.
    pub updated_at: DateTime<Utc>,
    /// Template version.
    pub version: Option<String>,
}

impl Default for Metadata {
    #[inline]
    fn default() -> Self {
        Self {
            created_at: Utc::now(),
            description: None,
            tags: Vec::new(),
            updated_at: Utc::now(),
            version: None,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Unit tests use unwrap/expect for readability"
)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    mod new {
        use super::*;

        #[test]
        fn accessors_return_expected_values() {
            // GIVEN: a new template aggregate
            let mut template = Template::new(
                "base".to_owned(),
                "Hello".to_owned(),
                HashMap::new(),
                None,
                Metadata::default(),
            )
            .unwrap();

            // WHEN: reading template accessors
            let event_count = template.pending_events().len();

            // THEN: accessors expose expected data
            assert_eq!(template.name(), "base");
            assert_eq!(template.content(), "Hello");
            assert!(template.extends().is_none());
            assert!(!template.has_variables());
            assert_eq!(event_count, 1);
            assert_eq!(template.take_events().len(), 1);
        }

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
            let error = result.unwrap_err();

            // THEN: a validation error describes the unbalanced syntax
            assert!(error.to_string().contains("Unbalanced"));
        }
    }

    use lithos_test_utils::data::properties::valid_identifier;

    proptest! {
        #[test]
        fn should_validate_template_name_format_across_edge_cases(name in valid_identifier()) {
            // GIVEN: a generated valid identifier
            let input = name;

            // WHEN: constructing a template with the identifier
            let result = Template::new(
                input,
                "content".to_owned(),
                HashMap::new(),
                None,
                Metadata::default(),
            );

            // THEN: construction succeeds
            prop_assert!(result.is_ok());
        }
    }

    #[test]
    fn should_compose_templates_with_sections() {
        // GIVEN: a base template and a composition
        let base = Template::new(
            "base".to_owned(),
            "Base: {{v}}".to_owned(),
            [(
                "v".to_owned(),
                VariableDefinition::Boolean {
                    default: None,
                },
            )]
            .into_iter()
            .collect(),
            None,
            Metadata::default(),
        )
        .unwrap();

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
            variable_overrides: [("v".to_owned(), serde_json::json!(true))]
                .into_iter()
                .collect(),
        };

        let templates =
            [("base".to_owned(), base.clone())].into_iter().collect();

        // WHEN: composing the template
        let composed =
            Template::compose(&base, &composition, &templates).unwrap();

        // THEN: content is correctly assembled
        assert!(composed.content().starts_with("Header\n"));
        assert!(composed.content().ends_with("\nFooter"));
        assert!(composed.content().contains("Base: {{v}}\nInside"));
        assert_eq!(composed.extends(), Some("base"));
    }

    #[test]
    fn apply_sections_handles_missing_variable() {
        // GIVEN: a template without variables
        let base = Template::new(
            "b".to_owned(),
            "no var".to_owned(),
            HashMap::new(),
            None,
            Metadata::default(),
        )
        .unwrap();

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
