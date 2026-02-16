//! Generates MiniJinja source code from template metadata.
//!
//! This module converts the domain representation of a template (including
//! inheritance and blocks) into a valid MiniJinja template string.

use crate::template::{aggregate::Template, block::BlockStrategy};

/// Generator for `MiniJinja` source code.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct SourceGenerator;

impl SourceGenerator {
    /// Generates `MiniJinja` source code from a `Template` aggregate.
    ///
    /// The generator handles:
    /// - Inheritance via `{% extends "..." %}`
    /// - Block definitions and overrides via `{% block ... %}`
    /// - Composition strategies (Replace, Extend, Prepend) via `super()`
    #[must_use]
    #[inline]
    pub fn generate(template: &Template) -> String {
        // Pre-allocate some space to reduce re-allocations
        let mut source = String::with_capacity(1024);

        // 1. Handle inheritance
        if let Some(parent) = template.extends() {
            source.push_str("{% extends \"");
            source.push_str(parent);
            source.push_str("\" %}\n");
        }

        // 2. Handle blocks
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
}

#[cfg(test)]
#[expect(clippy::disallowed_methods, reason = "Tests use unwrap")]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::template::block::TemplateBlock;

    #[test]
    fn generates_base_template_with_blocks() {
        let template = Template::new(
            "base",
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

        let source = SourceGenerator::generate(&template);

        assert!(
            source.contains("{% block content %}Base Content{% endblock %}")
        );
        assert!(source.contains("{% block footer %}Base Footer{% endblock %}"));
        assert!(!source.contains("{% extends"));
    }

    #[test]
    fn generates_child_template_with_inheritance() {
        let template = Template::new(
            "child",
            Some("base"),
            vec![TemplateBlock::new(
                "content",
                "Child Content",
                BlockStrategy::Replace,
            )],
            HashMap::new(),
        )
        .unwrap();

        let source = SourceGenerator::generate(&template);

        assert!(source.contains(r#"{% extends "base" %}"#));
        assert!(
            source.contains("{% block content %}Child Content{% endblock %}")
        );
    }

    #[test]
    fn generates_block_with_extend_strategy() {
        let template = Template::new(
            "child",
            Some("base"),
            vec![TemplateBlock::new("content", "Extra", BlockStrategy::Extend)],
            HashMap::new(),
        )
        .unwrap();

        let source = SourceGenerator::generate(&template);

        assert!(
            source.contains(
                "{% block content %}{{ super() }}Extra{% endblock %}"
            )
        );
    }

    #[test]
    fn generates_block_with_prepend_strategy() {
        let template = Template::new(
            "child",
            Some("base"),
            vec![TemplateBlock::new(
                "content",
                "Extra",
                BlockStrategy::Prepend,
            )],
            HashMap::new(),
        )
        .unwrap();

        let source = SourceGenerator::generate(&template);

        assert!(
            source.contains(
                "{% block content %}Extra{{ super() }}{% endblock %}"
            )
        );
    }
}
