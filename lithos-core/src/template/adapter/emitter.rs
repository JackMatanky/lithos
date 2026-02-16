//! Emits MiniJinja source code from template metadata.
//!
//! This module converts the domain representation of a template (including
//! inheritance and blocks) into a valid MiniJinja template string.

use crate::template::{aggregate::Template, block::BlockStrategy};

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
    use crate::template::{aggregate::TemplateName, block::TemplateBlock};

    #[test]
    fn emits_base_template_with_blocks() {
        let name = TemplateName::try_from("base").unwrap();
        let template = Template::new(
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
    fn emits_child_template_with_inheritance() {
        let name = TemplateName::try_from("child").unwrap();
        let parent = TemplateName::try_from("base").unwrap();
        let template = Template::new(
            &name,
            Some(parent),
            vec![TemplateBlock::new(
                "content",
                "Child Content",
                BlockStrategy::Replace,
            )],
            HashMap::new(),
        )
        .unwrap();

        let source = Emitter::emit(&template);

        assert!(source.contains(r#"{% extends "base" %}"#));
        assert!(
            source.contains("{% block content %}Child Content{% endblock %}")
        );
    }

    #[test]
    fn emits_block_with_extend_strategy() {
        let name = TemplateName::try_from("child").unwrap();
        let parent = TemplateName::try_from("base").unwrap();
        let template = Template::new(
            &name,
            Some(parent),
            vec![TemplateBlock::new("content", "Extra", BlockStrategy::Extend)],
            HashMap::new(),
        )
        .unwrap();

        let source = Emitter::emit(&template);

        assert!(
            source.contains(
                "{% block content %}{{ super() }}Extra{% endblock %}"
            )
        );
    }

    #[test]
    fn emits_block_with_prepend_strategy() {
        let name = TemplateName::try_from("child").unwrap();
        let parent = TemplateName::try_from("base").unwrap();
        let template = Template::new(
            &name,
            Some(parent),
            vec![TemplateBlock::new(
                "content",
                "Extra",
                BlockStrategy::Prepend,
            )],
            HashMap::new(),
        )
        .unwrap();

        let source = Emitter::emit(&template);

        assert!(
            source.contains(
                "{% block content %}Extra{{ super() }}{% endblock %}"
            )
        );
    }
}
