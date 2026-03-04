//! Emits MiniJinja source code from template metadata.
//!
//! This module converts the domain representation of a template (including
//! inheritance and blocks) into a valid MiniJinja template string.

use super::FilterName;
use crate::template::{
    aggregate::Template, block::BlockStrategy, value::InputSpec,
};

/// Emitter for `MiniJinja` source code.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct Emitter;

impl Emitter {
    /// Emits `MiniJinja` source code from a `Template` aggregate.
    ///
    /// The emitter handles:
    /// - Inheritance via `{% extends "..." %}`
    /// - Block definitions and overrides via `{% block ... %}`
    /// - Composition strategies (Replace, Extend, Prepend) via `super()`
    /// - Input constraint enforcement via filter chains
    #[must_use]
    #[inline]
    pub fn emit(template: &Template) -> String {
        // Pre-allocate some space to reduce re-allocations
        let mut source = String::with_capacity(1024);

        // 1. Handle inheritance
        if let Some(parent) = template.extends() {
            source.push_str("{% extends \"");
            source.push_str(parent.as_str());
            source.push_str("\" %}\n");
        }

        // 2. Add input validation helpers (macros or set statements)
        // For now, we rely on the user using the inputs in blocks.
        // In a future iteration, we could automatically wrap input usage.

        // 3. Handle blocks
        for block in template.blocks() {
            source.push_str("{% block ");
            source.push_str(block.name());
            source.push_str(" %}");

            match block.strategy() {
                BlockStrategy::Replace => {
                    source.push_str(block.content());
                }
                BlockStrategy::Extend => {
                    source.push_str("{{ super() }}");
                    source.push_str(block.content());
                }
                BlockStrategy::Prepend => {
                    source.push_str(block.content());
                    source.push_str("{{ super() }}");
                }
            }

            source.push_str("{% endblock %}\n");
        }

        source
    }

    /// Maps an `InputSpec` to a list of filter names and their arguments.
    ///
    /// This logic was moved from the domain to the adapter to maintain
    /// boundary purity.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics are preferred over explicit 'ref' patterns \
                  for readability."
    )]
    pub fn get_filter_chain(
        spec: &InputSpec,
    ) -> Vec<(FilterName, serde_json::Value)> {
        match spec {
            InputSpec::String {
                pattern,
                length,
                ..
            } => {
                let mut chain = Vec::new();
                if let Some(p) = pattern {
                    chain.push((
                        FilterName::VALIDATE_PATTERN,
                        serde_json::json!({ "pattern": *p }),
                    ));
                }
                if let Some(min) = length.min() {
                    chain.push((
                        FilterName::VALIDATE_LENGTH,
                        serde_json::json!({ "min": min }),
                    ));
                }
                if let Some(max) = length.max() {
                    Self::combine_or_push_length_filter(&mut chain, max);
                }
                chain
            }
            InputSpec::Number {
                bounds,
                ..
            } => {
                let mut args = serde_json::Map::new();
                if let Some(min) = bounds.min() {
                    args.insert("min".to_owned(), min.into());
                }
                if let Some(max) = bounds.max() {
                    args.insert("max".to_owned(), max.into());
                }
                if args.is_empty() {
                    vec![]
                } else {
                    vec![(
                        FilterName::VALIDATE_RANGE,
                        serde_json::Value::Object(args),
                    )]
                }
            }
            InputSpec::File {
                file_types: Some(types),
                ..
            } => {
                vec![(
                    FilterName::VALIDATE_FILE_TYPE,
                    serde_json::json!({ "types": types }),
                )]
            }
            InputSpec::Date {
                format: Some(f),
                ..
            } => {
                vec![(
                    FilterName::DATE_FORMAT,
                    serde_json::json!({ "format": *f }),
                )]
            }
            InputSpec::Boolean {
                ..
            }
            | InputSpec::Date {
                ..
            }
            | InputSpec::File {
                ..
            } => vec![],
        }
    }

    /// Helper to combine or push a length filter to reduce nesting.
    fn combine_or_push_length_filter(
        chain: &mut Vec<(FilterName, serde_json::Value)>,
        max: usize,
    ) {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Explicitly matching on mutable reference to combine \
                      filter args"
        )]
        if let Some((FilterName::VALIDATE_LENGTH, args)) = chain.last_mut() {
            if let Some(obj) = args.as_object_mut() {
                obj.insert("max".to_owned(), max.into());
            }
        } else {
            chain.push((
                FilterName::VALIDATE_LENGTH,
                serde_json::json!({ "max": max }),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{
        bounds::Bounds,
        template::{
            aggregate::TemplateName, block::TemplateBlock, value::InputSpec,
        },
    };

    #[test]
    fn emits_base_template_with_blocks() {
        let name = TemplateName::try_from("base").unwrap();
        let template = Template::try_new(
            &name,
            None,
            vec![
                TemplateBlock::new(
                    "content",
                    "Base Content",
                    BlockStrategy::Replace,
                ),
                TemplateBlock::new(
                    "footer",
                    "Base Footer",
                    BlockStrategy::Replace,
                ),
            ],
            HashMap::new(),
        )
        .unwrap();

        let source = Emitter::emit(&template);

        assert!(
            source.contains("{% block content %}Base Content{% endblock %}")
        );
        assert!(source.contains("{% block footer %}Base Footer{% endblock %}"));
        assert!(!source.contains("{% extends"));
    }

    #[test]
    #[expect(
        clippy::indexing_slicing,
        reason = "Test uses fixed indices for verification"
    )]
    #[expect(
        clippy::default_numeric_fallback,
        reason = "Test values are for comparison"
    )]
    fn maps_input_spec_to_filters() {
        let spec = InputSpec::String {
            default: None,
            length: Bounds::Range {
                min: 5,
                max: 10,
            },
            pattern: Some("^[A-Z]".into()),
        };

        let chain = Emitter::get_filter_chain(&spec);
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].0, FilterName::VALIDATE_PATTERN);
        assert_eq!(chain[1].0, FilterName::VALIDATE_LENGTH);
        assert_eq!(chain[1].1["min"], 5);
        assert_eq!(chain[1].1["max"], 10);
    }
}
