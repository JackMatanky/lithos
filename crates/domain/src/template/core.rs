use std::collections::HashMap;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{
    composition::{Composition, InsertionPosition, Section},
    events::TemplateCreated,
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

/// Domain events that can be emitted by the Template aggregate.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum DomainEvent {
    /// Template was created.
    TemplateCreated(TemplateCreated),
}

/// Aggregate root representing a reusable template.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "pub(crate) used for internal consistency with other domain models"
)]
#[expect(
    clippy::partial_pub_fields,
    reason = "pending_events is internally managed"
)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Meaningful logical ordering of aggregate fields"
)]
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
    #[serde(skip)]
    pub(crate) pending_events: Vec<DomainEvent>,
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
    pub fn add_event(&mut self, event: DomainEvent) {
        self.pending_events.push(event);
    }

    /// Composes a template from a base and a composition.
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` if composition validation fails.
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

        template.add_event(DomainEvent::TemplateCreated(TemplateCreated::new(
            id,
            name,
            chrono::Utc::now().timestamp(),
        )));

        Ok(template)
    }

    /// Creates a new template aggregate with validation.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails (name format, size limits, etc).
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

        template.add_event(DomainEvent::TemplateCreated(TemplateCreated::new(
            id,
            name,
            chrono::Utc::now().timestamp(),
        )));

        Ok(template)
    }

    /// Returns a reference to pending domain events.
    #[inline]
    #[must_use]
    pub fn pending_events(&self) -> &[DomainEvent] {
        &self.pending_events
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

    /// Returns and clears pending domain events.
    #[inline]
    #[must_use]
    pub fn take_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.pending_events)
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
            return Err(DomainError::ValidationFailed(
                "Template name cannot be empty".to_owned(),
            ));
        }
        if name.len() > 64 {
            return Err(DomainError::ValidationFailed(
                "Template name too long".to_owned(),
            ));
        }
        if !validation::is_alphanumeric_name(name) {
            return Err(DomainError::ValidationFailed(format!(
                "Invalid template name: {name}"
            )));
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
            return Err(DomainError::ValidationFailed(
                "Variable name cannot be empty".to_owned(),
            ));
        }
        if name.len() > 32 {
            return Err(DomainError::ValidationFailed(
                "Variable name too long".to_owned(),
            ));
        }
        if !validation::is_identifier_name(name) {
            return Err(DomainError::ValidationFailed(format!(
                "Invalid variable name: {name}"
            )));
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
            return Err(DomainError::ValidationFailed(format!(
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
mod tests {
    use proptest::prelude::*;

    use super::*;

    mod new {
        use super::*;

        #[test]
        fn should_create_template_when_attributes_are_valid() {
            let mut variables = HashMap::new();
            variables.insert(
                "title".to_owned(),
                VariableDefinition::String {
                    default: Some("Default".to_owned()),
                    max_length: None,
                    min_length: None,
                    pattern: None,
                },
            );

            let result = Template::new(
                "daily-note".to_owned(),
                "# {{title}}".to_owned(),
                variables,
                None,
                Metadata::default(),
            );

            assert!(
                result.is_ok(),
                "Expected valid template creation to succeed"
            );
        }

        #[test]
        fn should_reject_template_when_name_format_is_invalid() {
            let invalid_long_name = "a".repeat(65);
            let names = vec!["", "Invalid Name", "name!", &invalid_long_name];
            for name in names {
                let result = Template::new(
                    name.to_owned(),
                    "content".to_owned(),
                    HashMap::new(),
                    None,
                    Metadata::default(),
                );
                assert!(result.is_err(), "Expected error for name: {name}");
            }
        }

        #[test]
        fn should_reject_template_when_unbalanced_placeholders() {
            let result = Template::new(
                "unbalanced".to_owned(),
                "{{open but no close".to_owned(),
                HashMap::new(),
                None,
                Metadata::default(),
            );
            assert!(result.is_err());
            assert!(
                result.unwrap_err().to_string().contains("Unbalanced"),
                "Expected unbalanced error"
            );
        }
    }

    proptest! {
        #[test]
        fn should_validate_template_name_format_across_edge_cases(name in "[a-zA-Z0-9_-]{1,64}") {
            let result = Template::new(
                name,
                "content".to_owned(),
                HashMap::new(),
                None,
                Metadata::default(),
            );
            prop_assert!(result.is_ok());
        }
    }
}
