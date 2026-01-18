use std::{collections::HashMap, sync::LazyLock};

use chrono::{DateTime, Utc};
use regex::Regex;
use uuid::Uuid;

use super::{
    template_comp::{Composition, InsertionPosition, Section},
    template_var::VariableDefinition,
};
use crate::{errors::DomainError, events::TemplateCreated};

#[expect(clippy::disallowed_methods, reason = "Static regex initialization")]
#[expect(clippy::expect_used, reason = "Static regex initialization")]
static NAME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new("^[a-zA-Z0-9_-]+$").expect("Invalid static regex literal")
});
#[expect(clippy::disallowed_methods, reason = "Static regex initialization")]
#[expect(clippy::expect_used, reason = "Static regex initialization")]
static VAR_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new("^[a-zA-Z_][a-zA-Z0-9_]*$")
        .expect("Invalid static regex literal")
});

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
pub struct Template {
    /// Template content (MiniJinja-compatible syntax).
    pub content: String,
    /// Optional parent template for composition.
    pub extends: Option<String>,
    /// UUID v7 identity.
    pub id: Uuid,
    /// Metadata for template management.
    pub metadata: Metadata,
    /// Unique template name.
    pub name: String,
    /// Domain events pending emission (not serialized).
    #[serde(skip)]
    pub(crate) pending_events: Vec<DomainEvent>,
    /// Variable definitions with types and constraints.
    pub variables: HashMap<String, VariableDefinition>,
}

impl Template {
    /// Adds a domain event to the pending events collection.
    #[inline]
    pub fn add_event(&mut self, event: DomainEvent) {
        self.pending_events.push(event);
    }

    /// Applies additional sections to content.
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on enum references"
    )]
    fn apply_sections(content: &mut String, sections: &[Section]) {
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
                    Self::insert_relative_to_variable(
                        content,
                        var_name,
                        &section.content,
                        false,
                    );
                }
                InsertionPosition::AfterVariable(var_name) => {
                    Self::insert_relative_to_variable(
                        content,
                        var_name,
                        &section.content,
                        true,
                    );
                }
            }
        }
    }

    /// Composes a template from a base and a composition.
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` if composition validation fails.
    #[inline]
    pub fn compose(
        base: &Self,
        composition: &Composition,
    ) -> Result<Self, DomainError> {
        composition.validate(base)?;

        let mut final_content = base.content.clone();
        Self::apply_sections(
            &mut final_content,
            &composition.additional_sections,
        );

        Ok(Self {
            content: final_content,
            extends: Some(base.name.clone()),
            id: Uuid::now_v7(),
            metadata: Metadata::default(),
            name: format!("{}-composed", base.name),
            pending_events: vec![],
            variables: base.variables.clone(),
        })
    }

    /// Formats a variable name as a `MiniJinja` placeholder.
    #[expect(clippy::arithmetic_side_effects, reason = "String capacity")]
    fn format_placeholder(var_name: &str) -> String {
        let mut placeholder = String::with_capacity(var_name.len() + 4);
        placeholder.push_str("{{");
        placeholder.push_str(var_name);
        placeholder.push_str("}}");
        placeholder
    }

    /// Inserts content relative to a variable placeholder.
    #[expect(clippy::arithmetic_side_effects, reason = "Index calculation")]
    fn insert_relative_to_variable(
        content: &mut String,
        var_name: &str,
        section_content: &str,
        after: bool,
    ) {
        let placeholder = Self::format_placeholder(var_name);
        if let Some(pos) = content.find(&placeholder) {
            if after {
                let insert_pos = pos + placeholder.len();
                content.insert(insert_pos, '\n');
                content.insert_str(insert_pos + 1, section_content);
            } else {
                content.insert_str(pos, section_content);
                content.insert(pos + section_content.len(), '\n');
            }
        }
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
        Self::validate_content(&content)?;
        Self::validate_variable_definitions(&variables)?;

        let id = Uuid::now_v7();
        let mut template = Self {
            content,
            extends,
            id,
            metadata,
            name: name.clone(),
            pending_events: vec![],
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

    /// Returns a reference to pending domain events without clearing them.
    #[inline]
    #[must_use]
    pub fn pending_events(&self) -> &[DomainEvent] {
        &self.pending_events
    }

    /// Returns all pending domain events and clears the collection.
    #[inline]
    #[must_use]
    pub fn take_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Validates template business rules.
    ///
    /// # Errors
    /// Currently always returns `Ok(())`.
    #[inline]
    pub fn validate(&self) -> Result<(), DomainError> {
        Ok(())
    }

    /// Validates template content.
    fn validate_content(content: &str) -> Result<(), DomainError> {
        if content.len() > 1024 * 1024 {
            return Err(DomainError::TemplateContentTooLarge(
                content.len(),
                1024 * 1024,
            ));
        }
        Ok(())
    }

    /// Validates template name format.
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
        if !NAME_RE.is_match(name) {
            return Err(DomainError::ValidationFailed(format!(
                "Invalid template name: {name}"
            )));
        }
        Ok(())
    }

    /// Validates variable definitions.
    #[expect(
        clippy::iter_over_hash_type,
        reason = "Validation order is irrelevant"
    )]
    fn validate_variable_definitions(
        variables: &HashMap<String, VariableDefinition>,
    ) -> Result<(), DomainError> {
        if variables.len() > 50 {
            return Err(DomainError::MaxVariablesExceeded(variables.len()));
        }

        for var_name in variables.keys() {
            Self::validate_variable_name(var_name)?;
        }
        Ok(())
    }

    /// Validates individual variable name.
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
        if !VAR_RE.is_match(name) {
            return Err(DomainError::ValidationFailed(format!(
                "Invalid variable name: {name}"
            )));
        }
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

    #[test]
    fn creates_valid_template_successfully() {
        let metadata = Metadata::default();
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
            metadata,
        );

        assert!(
            result.is_ok(),
            "Expected valid template creation to succeed, got {:?}",
            result.err()
        );
    }

    #[test]
    fn rejects_invalid_template_names() {
        let names = vec![
            String::new(),
            "Invalid Name".to_owned(),
            "name!".to_owned(),
            "a".repeat(65),
        ];
        for name in names {
            let result = Template::new(
                name.clone(),
                "content".to_owned(),
                HashMap::new(),
                None,
                Metadata::default(),
            );
            assert!(result.is_err(), "Expected name '{name}' to be rejected");
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
            Metadata::default(),
        );
        assert!(matches!(
            result,
            Err(DomainError::TemplateContentTooLarge(_, _))
        ));
    }

    #[test]
    fn rejects_too_many_variables() {
        let mut variables = HashMap::new();
        for i in 0i32..51i32 {
            variables.insert(
                format!("var{i}"),
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
            Metadata::default(),
        );
        assert!(matches!(result, Err(DomainError::MaxVariablesExceeded(_))));
    }

    proptest! {
        #[test]
        fn validates_template_name_format(name in "[a-zA-Z0-9_-]{1,64}") {
            let result = Template::new(
                name.clone(),
                "content".to_owned(),
                HashMap::new(),
                None,
                Metadata::default(),
            );
            prop_assert!(result.is_ok());
        }
    }
}
