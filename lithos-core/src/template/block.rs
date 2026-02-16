#![allow(
    clippy::exhaustive_enums,
    clippy::impl_trait_in_params,
    clippy::missing_inline_in_public_items,
    reason = "Domain entities require exhaustive enums for storage and trait \
              impls for flexibility"
)]

use rkyv::{Archive, Deserialize, Serialize};

/// Defines how a child block relates to a parent block in composition.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum BlockStrategy {
    /// Replace parent's block entirely (default).
    Replace,
    /// Call parent's block first, then append ours.
    Extend,
    /// Append our content, then call parent's block.
    Prepend,
}

/// Metadata for a single block in template composition.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct TemplateBlock {
    /// Block identifier.
    name: Box<str>,
    /// Block content (raw text).
    content: Box<str>,
    /// Composition strategy.
    strategy: BlockStrategy,
}

impl TemplateBlock {
    /// Creates a new template block.
    pub fn new(
        name: impl Into<Box<str>>,
        content: impl Into<Box<str>>,
        strategy: BlockStrategy,
    ) -> Self {
        Self {
            name: name.into(),
            content: content.into(),
            strategy,
        }
    }

    /// Returns the block name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the block content.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns the composition strategy.
    #[must_use]
    pub const fn strategy(&self) -> BlockStrategy {
        self.strategy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_block_with_replace_strategy() {
        let block =
            TemplateBlock::new("test", "content", BlockStrategy::Replace);
        assert_eq!(block.name(), "test");
        assert_eq!(block.content(), "content");
        assert_eq!(block.strategy(), BlockStrategy::Replace);
    }

    #[test]
    fn creates_block_with_extend_strategy() {
        let block =
            TemplateBlock::new("test", "content", BlockStrategy::Extend);
        assert_eq!(block.strategy(), BlockStrategy::Extend);
    }

    #[test]
    fn creates_block_with_prepend_strategy() {
        let block =
            TemplateBlock::new("test", "content", BlockStrategy::Prepend);
        assert_eq!(block.strategy(), BlockStrategy::Prepend);
    }
}
