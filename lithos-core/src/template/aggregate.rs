//! Template aggregate root and composition logic.
//!
//! Handles template lifecycle, input specifications, and hierarchical
//! composition through native MiniJinja inheritance.
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
use rkyv::{
    Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize,
};
use uuid::Uuid;

use super::{
    block::TemplateBlock,
    error::TemplateError,
    events::{Events, TemplateCreated},
    value::InputSpec,
};

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

/// Unique template name enforced by domain constraints.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[rkyv(derive(Debug, Hash, PartialEq, Eq))]
pub struct TemplateName(pub Box<str>);

impl TemplateName {
    /// Template name validation pattern: alphanumeric, underscores, and dashes.
    ///
    /// Pattern: `^[a-zA-Z0-9_-]+$`.
    ///
    /// # Examples
    /// - Valid: `daily-note`, `MyTemplate`, `template_123`
    /// - Invalid: `template name`, `template!`, `template.txt`
    const PATTERN: &'static str = "^[a-zA-Z0-9_-]+$";

    /// Returns the template name as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TemplateName {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for TemplateName {
    type Error = TemplateError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static RE: LazyLock<Result<Regex, regex::Error>> =
            LazyLock::new(|| Regex::new(TemplateName::PATTERN));

        if value.is_empty() {
            return Err(TemplateError::EmptyTemplateName);
        }
        if value.len() > 64 {
            return Err(TemplateError::TemplateNameTooLong(value.len()));
        }

        let re = RE.as_ref().map_err(|error| {
            TemplateError::ValidationFailed(format!(
                "Invalid template name regex: {error}"
            ))
        })?;

        if !re.is_match(value) {
            return Err(TemplateError::InvalidTemplateName(value.to_owned()));
        }
        Ok(Self(value.into()))
    }
}

/// Unique input name enforced by domain constraints.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[rkyv(derive(Debug, Hash, PartialEq, Eq))]
pub struct InputName(pub Box<str>);

impl InputName {
    /// Input name validation pattern: programming-style identifiers.
    ///
    /// Pattern: `^[a-zA-Z_][a-zA-Z0-9_]*$`.
    ///
    /// Must start with a letter or underscore. May contain letters, digits, and
    /// underscores.
    ///
    /// # Examples
    /// - Valid: `title`, `my_var`, `_private`, `camelCase`
    /// - Invalid: `123var`, `my-var`, `var!`
    const PATTERN: &'static str = "^[a-zA-Z_][a-zA-Z0-9_]*$";

    /// Returns the input name as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for InputName {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for InputName {
    type Error = TemplateError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        static RE: LazyLock<Result<Regex, regex::Error>> =
            LazyLock::new(|| Regex::new(InputName::PATTERN));

        if value.is_empty() {
            return Err(TemplateError::EmptyInputName);
        }
        if value.len() > 32 {
            return Err(TemplateError::InputNameTooLong(value.len()));
        }

        let re = RE.as_ref().map_err(|error| {
            TemplateError::ValidationFailed(format!(
                "Invalid input name regex: {error}"
            ))
        })?;

        if !re.is_match(value) {
            return Err(TemplateError::InvalidInputName(value.to_owned()));
        }

        if RESERVED_WORDS.contains(&value) {
            return Err(TemplateError::InvalidInputName(format!(
                "Input name '{value}' is a reserved word"
            )));
        }

        Ok(Self(value.into()))
    }
}

/// Metadata for template management.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
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

/// Aggregate root representing a reusable template metadata schema.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Template {
    /// UUID v7 identity.
    pub id: Uuid,
    /// Unique template name.
    pub name: TemplateName,
    /// Optional parent template name.
    pub extends: Option<TemplateName>,
    /// Block definitions.
    pub blocks: Vec<TemplateBlock>,
    /// Input specifications.
    pub inputs: HashMap<InputName, InputSpec>,
    /// Metadata for template management.
    pub metadata: Metadata,
    /// Domain events pending emission.
    #[rkyv(with = rkyv::with::Skip)]
    #[serde(skip)]
    pub pending_events: Vec<Events>,
}

impl Template {
    /// Creates a new template aggregate with validation.
    ///
    /// # Errors
    /// Returns `TemplateError` if validation fails (duplicate blocks, etc).
    #[inline]
    pub fn new(
        name: &TemplateName,
        extends: Option<TemplateName>,
        blocks: Vec<TemplateBlock>,
        inputs: HashMap<InputName, InputSpec>,
    ) -> Result<Self, TemplateError> {
        if inputs.len() > 50 {
            return Err(TemplateError::MaxInputsExceeded(inputs.len()));
        }

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
            name: name.clone(),
            extends,
            blocks,
            inputs,
            metadata: Metadata::default(),
            pending_events: vec![],
        };

        template.add_event(Events::TemplateCreated(TemplateCreated::new(
            id,
            name.as_str(),
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
        all_templates: &HashMap<TemplateName, &Template>,
    ) -> Result<(), TemplateError> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        self.dfs(&self.name, all_templates, &mut visited, &mut stack)
    }

    fn dfs<'ctx>(
        &self,
        current: &'ctx TemplateName,
        all_templates: &HashMap<TemplateName, &'ctx Template>,
        visited: &mut HashSet<&'ctx TemplateName>,
        stack: &mut Vec<&'ctx TemplateName>,
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
            && let Some(parent) = template.extends.as_ref()
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
    pub fn name(&self) -> &TemplateName {
        &self.name
    }

    /// Returns the name of the template this one extends, if any.
    #[inline]
    #[must_use]
    pub fn extends(&self) -> Option<&TemplateName> {
        self.extends.as_ref()
    }

    /// Returns the template's blocks.
    #[inline]
    #[must_use]
    pub fn blocks(&self) -> &[TemplateBlock] {
        &self.blocks
    }

    /// Returns the template's input specifications.
    #[inline]
    #[must_use]
    pub fn inputs(&self) -> &HashMap<InputName, InputSpec> {
        &self.inputs
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

    /// Returns true if the template defines any inputs.
    #[inline]
    #[must_use]
    pub fn has_inputs(&self) -> bool {
        !self.inputs.is_empty()
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

        pub fn base_note() -> Result<Template, TemplateError> {
            Template::new(
                &TemplateName::try_from("base")?,
                None,
                vec![],
                HashMap::new(),
            )
        }
    }

    mod constructor {
        use super::*;

        fn base_template() -> Template {
            fixtures::base_note().expect("Valid base template")
        }

        #[test]
        fn name_accessor_returns_template_name() {
            let template = base_template();

            assert_eq!(
                template.name().as_str(),
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
        fn has_inputs_false_for_base_template() {
            let template = base_template();

            assert!(!template.has_inputs(), "Template should have no inputs");
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
            let result = TemplateName::try_from("");
            assert!(result.is_err(), "Expected error for empty name");
        }

        #[test]
        fn should_reject_template_when_name_contains_spaces() {
            let result = TemplateName::try_from("Invalid Name");
            assert!(result.is_err(), "Expected error for name with spaces");
        }

        #[test]
        fn should_reject_template_when_name_contains_invalid_characters() {
            let result = TemplateName::try_from("name!");
            assert!(
                result.is_err(),
                "Expected error for name with invalid characters"
            );
        }

        #[test]
        fn should_reject_template_when_name_is_too_long() {
            let invalid_long_name = "a".repeat(65);
            let result = TemplateName::try_from(invalid_long_name.as_str());
            assert!(result.is_err(), "Expected error for overlong name");
        }

        #[test]

        fn should_reject_duplicate_block_names() {
            let name = TemplateName::try_from("duplicate").unwrap();
            let result = Template::new(
                &name,
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
            let result = TemplateName::try_from(name.as_str());

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

    fn validate_composition_detects_cycles() {
        // A -> B -> A
        let a_name = TemplateName::try_from("A").unwrap();
        let b_name = TemplateName::try_from("B").unwrap();

        let a = Template::new(
            &a_name,
            Some(b_name.clone()),
            vec![],
            HashMap::new(),
        )
        .expect("valid");
        let b = Template::new(
            &b_name,
            Some(a_name.clone()),
            vec![],
            HashMap::new(),
        )
        .expect("valid");

        let mut map = HashMap::new();
        map.insert(a_name.clone(), &a);
        map.insert(b_name.clone(), &b);

        let result = a.validate_composition(&map);
        assert!(matches!(result, Err(TemplateError::CircularComposition(_))));
    }

    #[test]

    fn validate_composition_detects_self_cycle() {
        // A -> A
        let a_name = TemplateName::try_from("A").unwrap();
        let a = Template::new(
            &a_name,
            Some(a_name.clone()),
            vec![],
            HashMap::new(),
        )
        .expect("valid");

        let mut map = HashMap::new();
        map.insert(a_name.clone(), &a);

        let result = a.validate_composition(&map);
        assert!(matches!(result, Err(TemplateError::CircularComposition(_))));
    }

    #[test]

    fn validate_composition_allows_valid_chain() {
        // A -> B -> C
        let a_name = TemplateName::try_from("A").unwrap();
        let b_name = TemplateName::try_from("B").unwrap();
        let c_name = TemplateName::try_from("C").unwrap();

        let c = Template::new(&c_name, None, vec![], HashMap::new())
            .expect("valid");
        let b = Template::new(
            &b_name,
            Some(c_name.clone()),
            vec![],
            HashMap::new(),
        )
        .expect("valid");
        let a = Template::new(
            &a_name,
            Some(b_name.clone()),
            vec![],
            HashMap::new(),
        )
        .expect("valid");

        let mut map = HashMap::new();
        map.insert(a_name.clone(), &a);
        map.insert(b_name.clone(), &b);
        map.insert(c_name.clone(), &c);

        a.validate_composition(&map).expect("should be valid");
    }

    #[test]
    #[expect(clippy::default_numeric_fallback, reason = "Test")]
    #[expect(clippy::iter_over_hash_type, reason = "Test")]
    fn validate_composition_detects_depth_limit() {
        // 0 -> 1 -> ... -> 10 (11 levels)
        let mut map_storage = HashMap::new();
        let mut map = HashMap::new();

        for i in 0..=10 {
            let name_str = i.to_string();
            let name = TemplateName::try_from(name_str.as_str()).unwrap();
            let extends = if i == 10 {
                None
            } else {
                let next_str = (i + 1).to_string();
                Some(TemplateName::try_from(next_str.as_str()).unwrap())
            };
            let t = Template::new(&name, extends, vec![], HashMap::new())
                .expect("valid");
            map_storage.insert(name, t);
        }

        for (k, v) in &map_storage {
            map.insert(k.clone(), v);
        }

        let start_name = TemplateName::try_from("0").unwrap();
        let start = map.get(&start_name).expect("start template");
        let result = start.validate_composition(&map);
        assert!(matches!(
            result,
            Err(TemplateError::CompositionDepthExceeded(_))
        ));
    }
}
