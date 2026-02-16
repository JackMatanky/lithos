//! Template aggregate root and composition logic.
//!
//! Handles template lifecycle, variable definitions, and hierarchical
//! composition through section insertion.
#![allow(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    clippy::missing_inline_in_public_items,
    clippy::items_after_statements,
    clippy::self_only_used_in_recursion,
    reason = "rkyv Archive derive generates non-exhaustive archived types"
)]

use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use chrono::{DateTime, Utc};
use regex::Regex;
use uuid::Uuid;

use super::{
    block::TemplateBlock,
    composition::{Composition, InsertionPosition, Section},
    error::TemplateError,
    events::{Events, TemplateCreated},
    syntax::PlaceholderSyntax,
    validation::validate_structure,
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
    pub description: Option<Box<str>>,
    /// Template version.
    pub version: Option<Box<str>>,
    /// Tags for categorization.
    pub tags: Vec<Box<str>>,
    /// Creation timestamp.
    #[rkyv(with = crate::ser::DateTimeAsI64)]
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp.
    #[rkyv(with = crate::ser::DateTimeAsI64)]
    pub updated_at: DateTime<Utc>,
}

impl Default for Metadata {
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
    pub name: Box<str>,
    /// Optional parent template name.
    pub extends: Option<Box<str>>,
    /// Block definitions.
    pub blocks: Vec<TemplateBlock>,
    /// Variable definitions.
    pub variables: HashMap<String, VariableDefinition>,
    /// Metadata for template management.
    pub metadata: Metadata,
    /// Domain events pending emission.
    #[rkyv(with = rkyv::with::Skip)]
    #[serde(skip)]
    pub pending_events: Vec<Events>,

    /// Deprecated content field.
    #[deprecated]
    pub content: String,
    /// Deprecated syntax field.
    #[deprecated]
    pub syntax: PlaceholderSyntax,
}

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Function ordering optimized for logical flow; matching on \
              reference to enum avoids borrow checker friction"
)]
impl Template {
    /// Creates a new template aggregate with validation.
    ///
    /// # Errors
    /// Returns `TemplateError` if validation fails (name format, size limits,
    /// etc).
    #[inline]
    pub fn new(
        name: &str,
        extends: Option<&str>,
        blocks: Vec<TemplateBlock>,
        variables: HashMap<String, VariableDefinition>,
    ) -> Result<Self, TemplateError> {
        Self::validate_name(name)?;
        Self::validate_variable_definitions(&variables)?;

        // Ensure block names are unique within the template
        let mut block_names = HashSet::new();
        for block in &blocks {
            if !block_names.insert(block.name()) {
                return Err(TemplateError::ValidationFailed(format!(
                    "Duplicate block name: {}",
                    block.name()
                )));
            }
        }

        let id = Uuid::now_v7();
        let mut template = Self {
            id,
            name: name.into(),
            extends: extends.map(Into::into),
            blocks,
            variables,
            metadata: Metadata::default(),
            pending_events: vec![],
            #[expect(deprecated, reason = "Legacy field")]
            content: String::new(),
            #[expect(deprecated, reason = "Legacy field")]
            syntax: PlaceholderSyntax::default(),
        };

        template.add_event(Events::TemplateCreated(TemplateCreated::new(
            id,
            name,
            chrono::Utc::now().timestamp(),
        )));

        Ok(template)
    }

    /// Adds a domain event to the pending events collection.
    #[inline]
    fn add_event(&mut self, event: Events) {
        self.pending_events.push(event);
    }

    /// Validates composition relationships (cycle detection).
    ///
    /// # Errors
    /// Returns `TemplateError::CircularComposition` if a cycle is detected.
    /// Returns `TemplateError::CompositionDepthExceeded` if depth > 10.
    #[inline]
    pub fn validate_composition(
        &self,
        all_templates: &HashMap<&str, &Template>,
    ) -> Result<(), TemplateError> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        self.dfs(self.name(), all_templates, &mut visited, &mut stack)
    }

    fn dfs<'ctx>(
        &self,
        current: &'ctx str,
        all_templates: &HashMap<&str, &'ctx Template>,
        visited: &mut HashSet<&'ctx str>,
        stack: &mut Vec<&'ctx str>,
    ) -> Result<(), TemplateError> {
        if stack.contains(&current) {
            return Err(TemplateError::CircularComposition(format!(
                "Cycle detected: {stack:?}"
            )));
        }

        if stack.len() >= 10 {
            return Err(TemplateError::CompositionDepthExceeded(stack.len()));
        }

        if visited.contains(current) {
            return Ok(());
        }

        stack.push(current);
        visited.insert(current);

        if let Some(template) = all_templates.get(current)
            && let Some(parent) = template.extends()
        {
            self.dfs(parent, all_templates, visited, stack)?;
        }

        stack.pop();
        Ok(())
    }

    /// Returns the template's unique identifier.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Returns the template's unique name.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the name of the template this one extends, if any.
    #[inline]
    #[must_use]
    pub fn extends(&self) -> Option<&str> {
        self.extends.as_deref()
    }

    /// Returns the template's blocks.
    #[inline]
    #[must_use]
    pub fn blocks(&self) -> &[TemplateBlock] {
        &self.blocks
    }

    /// Returns the template's variable definitions.
    #[inline]
    #[must_use]
    pub fn variables(&self) -> &HashMap<String, VariableDefinition> {
        &self.variables
    }

    /// Returns the template's metadata.
    #[inline]
    #[must_use]
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Returns and clears pending domain events.
    #[inline]
    #[must_use]
    pub fn take_events(&mut self) -> Vec<Events> {
        std::mem::take(&mut self.pending_events)
    }

    /// Returns a reference to pending domain events.
    #[inline]
    #[must_use]
    pub fn pending_events(&self) -> &[Events] {
        &self.pending_events
    }

    // --- Deprecated Methods ---

    /// Returns the template's content.
    #[inline]
    #[must_use]
    #[deprecated]
    #[expect(deprecated, reason = "Legacy field")]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns true if the template defines any variables.
    #[inline]
    #[must_use]
    pub fn has_variables(&self) -> bool {
        !self.variables.is_empty()
    }

    /// Returns the template's placeholder syntax.
    #[inline]
    #[must_use]
    #[deprecated]
    #[expect(deprecated, reason = "Legacy field")]
    pub const fn syntax(&self) -> &PlaceholderSyntax {
        &self.syntax
    }

    /// Validates template constraints.
    ///
    /// # Errors
    /// Returns `TemplateError` if validation fails.
    #[inline]
    #[deprecated(note = "Template syntax validation is handled by MiniJinja.")]
    #[expect(deprecated, reason = "Legacy method")]
    pub fn validate(&self) -> Result<(), TemplateError> {
        validate_structure(
            &self.content,
            &self.syntax.prefix,
            &self.syntax.suffix,
        )?;
        Ok(())
    }

    /// Composes a template from a base and a composition.
    ///
    /// # Errors
    /// Returns `TemplateError` if composition fails.
    #[inline]
    #[deprecated(note = "Composition is handled by MiniJinja inheritance ({% \
                         extends %}).")]
    #[expect(deprecated, reason = "Legacy method")]
    pub fn compose(
        base: &Self,
        composition: &Composition,
        templates: &HashMap<&str, &Template>,
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
            id,
            name: name.clone().into(),
            extends: Some(base.name.clone()),
            blocks: base.blocks.clone(), // Naive copy, deprecated anyway
            variables: base.variables.clone(),
            metadata: Metadata::default(),
            pending_events: vec![],
            content: final_content,
            syntax: base.syntax.clone(),
        };

        template.add_event(Events::TemplateCreated(TemplateCreated::new(
            id,
            &name,
            chrono::Utc::now().timestamp(),
        )));

        Ok(template)
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

    #[expect(deprecated, reason = "Legacy method")]
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

    // --- Validation Helpers ---

    /// Validates a template name according to domain constraints.
    ///
    /// # Errors
    /// Returns `TemplateError` if the name is invalid.
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
    /// Returns `TemplateError` if the variable name is invalid.
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
#[expect(clippy::arbitrary_source_item_ordering, reason = "Test organization")]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::template::block::{BlockStrategy, TemplateBlock};

    mod fixtures {
        use super::*;

        pub fn base_template() -> Result<Template, TemplateError> {
            Template::new("base", None, vec![], HashMap::new())
        }
    }

    mod constructor {
        use super::*;

        #[expect(
            clippy::disallowed_methods,
            reason = "Test fixture uses expect for deterministic setup. \
                      Failure indicates invalid test data. Expect is \
                      idiomatic in setup."
        )]
        fn base_template() -> Template {
            fixtures::base_template().expect("Valid base template")
        }

        #[test]
        fn name_accessor_returns_template_name() {
            let template = base_template();

            assert_eq!(
                template.name(),
                "base",
                "Template name should be 'base'"
            );
        }

        #[test]
        fn extends_is_none_for_base_template() {
            let template = base_template();

            assert!(
                template.extends().is_none(),
                "Template should not extend another template"
            );
        }

        #[test]
        fn has_variables_false_for_base_template() {
            let template = base_template();

            assert!(
                !template.has_variables(),
                "Template should have no variables"
            );
        }

        #[test]
        fn pending_events_emitted_on_create() {
            let template = base_template();

            assert_eq!(
                template.pending_events().len(),
                1,
                "Template should have 1 pending event"
            );
        }

        #[test]
        fn take_events_returns_pending_events() {
            let mut template = base_template();

            assert_eq!(
                template.take_events().len(),
                1,
                "Taking events should return 1 event"
            );
        }

        #[test]
        fn take_events_clears_pending_events() {
            let mut template = base_template();

            let _events = template.take_events();

            assert!(
                template.pending_events().is_empty(),
                "Pending events should be empty after take_events"
            );
        }

        #[test]
        fn should_reject_template_when_name_is_empty() {
            let result = Template::new("", None, vec![], HashMap::new());

            assert!(result.is_err(), "Expected error for empty name");
        }

        #[test]
        fn should_reject_template_when_name_contains_spaces() {
            let result =
                Template::new("Invalid Name", None, vec![], HashMap::new());

            assert!(result.is_err(), "Expected error for name with spaces");
        }

        #[test]
        fn should_reject_template_when_name_contains_invalid_characters() {
            let result = Template::new("name!", None, vec![], HashMap::new());

            assert!(
                result.is_err(),
                "Expected error for name with invalid characters"
            );
        }

        #[test]
        fn should_reject_template_when_name_is_too_long() {
            let invalid_long_name = "a".repeat(65);
            let result =
                Template::new(&invalid_long_name, None, vec![], HashMap::new());

            assert!(result.is_err(), "Expected error for overlong name");
        }

        #[test]
        fn should_reject_duplicate_block_names() {
            let result = Template::new(
                "duplicate",
                None,
                vec![
                    TemplateBlock::new(
                        "block1",
                        "content",
                        BlockStrategy::Replace,
                    ),
                    TemplateBlock::new(
                        "block1",
                        "content2",
                        BlockStrategy::Extend,
                    ),
                ],
                HashMap::new(),
            );

            assert!(
                matches!(result, Err(TemplateError::ValidationFailed(msg)) if msg.contains("Duplicate block name"))
            );
        }
    }

    #[test]
    fn should_validate_template_name_format_across_edge_cases()
    -> Result<(), String> {
        use proptest::test_runner::TestRunner;

        let mut runner = TestRunner::deterministic();
        let strategy = "[a-zA-Z0-9_-]{1,64}";

        let run_result = runner.run(&strategy, |name| {
            let result = Template::new(&name, None, vec![], HashMap::new());

            prop_assert!(
                result.is_ok(),
                "Template with valid name '{}' should be created",
                name
            );
            Ok(())
        });
        run_result
            .map_err(|e| format!("Proptest run should succeed, got: {e:?}"))?;

        Ok(())
    }

    #[test]
    #[expect(clippy::disallowed_methods, reason = "Test")]
    fn validate_composition_detects_cycles() {
        // A -> B -> A
        let a = Template::new("A", Some("B"), vec![], HashMap::new())
            .expect("valid");
        let b = Template::new("B", Some("A"), vec![], HashMap::new())
            .expect("valid");

        let mut map = HashMap::new();
        map.insert("A", &a);
        map.insert("B", &b);

        let result = a.validate_composition(&map);
        assert!(matches!(result, Err(TemplateError::CircularComposition(_))));
    }

    #[test]
    #[expect(clippy::disallowed_methods, reason = "Test")]
    fn validate_composition_detects_self_cycle() {
        // A -> A
        let a = Template::new("A", Some("A"), vec![], HashMap::new())
            .expect("valid");

        let mut map = HashMap::new();
        map.insert("A", &a);

        let result = a.validate_composition(&map);
        assert!(matches!(result, Err(TemplateError::CircularComposition(_))));
    }

    #[test]
    #[expect(clippy::disallowed_methods, reason = "Test")]
    fn validate_composition_allows_valid_chain() {
        // A -> B -> C
        let c =
            Template::new("C", None, vec![], HashMap::new()).expect("valid");
        let b = Template::new("B", Some("C"), vec![], HashMap::new())
            .expect("valid");
        let a = Template::new("A", Some("B"), vec![], HashMap::new())
            .expect("valid");

        let mut map = HashMap::new();
        map.insert("A", &a);
        map.insert("B", &b);
        map.insert("C", &c);

        a.validate_composition(&map).expect("should be valid");
    }

    #[test]
    #[expect(clippy::disallowed_methods, reason = "Test")]
    #[expect(clippy::default_numeric_fallback, reason = "Test")]
    #[expect(clippy::iter_over_hash_type, reason = "Test")]
    fn validate_composition_detects_depth_limit() {
        // 0 -> 1 -> ... -> 10 (11 levels)
        let mut map_storage = HashMap::new();
        let mut map = HashMap::new();

        for i in 0..=10 {
            let name = i.to_string();
            let extends = if i == 10 {
                None
            } else {
                Some((i + 1).to_string())
            };
            let t = Template::new(
                &name,
                extends.as_deref(),
                vec![],
                HashMap::new(),
            )
            .expect("valid");
            map_storage.insert(name, t);
        }

        for (k, v) in &map_storage {
            map.insert(k.as_str(), v);
        }

        let start = map.get("0").expect("start template");
        let result = start.validate_composition(&map);
        assert!(matches!(
            result,
            Err(TemplateError::CompositionDepthExceeded(_))
        ));
    }
}
