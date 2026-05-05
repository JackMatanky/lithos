//! Block domain types for markdown document structure.
//!
//! This module defines the core [`Block`] type and its variants. Blocks
//! represent structural elements like paragraphs, headings, and lists.
//!
//! # Block Lifecycle (Type-State Pattern)
//!
//! To ensure structural integrity during parsing, blocks use a type-state
//! pattern to distinguish between their lifecycle phases:
//!
//! 1. **[`Open`]**: A block that is currently being assembled. It stores its
//!    start offset and accumulates raw events or text fragments.
//! 2. **[`Closed`]**: A finalized AST node. It has a confirmed
//!    [`SourceByteRange`] and its content (like code block text) is
//!    materialized.
//!
//! This pattern prevents logic errors such as attempting to read the content of
//! a block that hasn't finished parsing or attaching children to a finalized
//! leaf.

use super::{
    text::TextSequence,
    types::{BlockEnd, FrontmatterFormat, HeadingLevel, RangedEvent},
};
use crate::note::{
    error::{NoteIngestError, NoteParseError},
    position::{SourceByteOffset, SourceByteRange},
};

/// Trait for block state markers in the type-state pattern.
pub(crate) trait BlockState: std::fmt::Debug {
    /// The leaf data type for this state (CodeBlock/Frontmatter).
    type LeafData<'source>: Clone + PartialEq + std::fmt::Debug;
    /// The position type for this state.
    type Position: Clone + PartialEq + std::fmt::Debug;
}

/// Marker for an open block (currently being parsed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Open;
impl BlockState for Open {
    type LeafData<'source> = Vec<RangedEvent<'source>>;
    type Position = SourceByteOffset;
}

/// Marker for a closed block (finalized AST node).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Closed;
impl BlockState for Closed {
    type LeafData<'source> = String;
    type Position = SourceByteRange;
}

/// Type alias for the position of a block in state `S`.
pub(crate) type BlockPosition<S> = <S as BlockState>::Position;

/// A markdown block in the document tree.
///
/// Each block represents a single structural element from the markdown source,
/// such as a paragraph, heading, list, or code block. Blocks form a tree
/// structure through the `children` fields in container block variants.
///
/// # Lifecycle
///
/// Blocks are created during AST building as `Block<'source, Open>` nodes.
/// Once a block's full span and content are known, it is finalized into a
/// `Block<'source, Closed>` node via [`Block::close`].
#[derive(Debug, Clone, PartialEq)]
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "module-private struct with deliberate pub(crate) fields"
)]
pub(crate) struct Block<'source, S: BlockState = Closed> {
    /// The type and content of this block.
    pub(crate) kind: BlockKind<'source, S>,
    /// The source position (offset for Open, range for Closed).
    pub(crate) span: BlockPosition<S>,
}

impl<'source, S: BlockState> Block<'source, S> {
    /// Returns a slice of the child blocks for this block.
    ///
    /// If this is a leaf block, returns an empty slice.
    #[must_use]
    #[inline]
    pub(crate) fn children(&self) -> &[Block<'source, Closed>] {
        self.kind.children()
    }
}

impl<'source> Block<'source, Open> {
    /// Finalizes an open block into a closed one.
    ///
    /// This transitions the block from its "open" parsing state to its "closed"
    /// AST state, calculating the final source range and transforming leaf
    /// data (like code block text) into its canonical form.
    pub(crate) fn close(
        self,
        end: SourceByteOffset,
    ) -> Result<Block<'source, Closed>, NoteIngestError> {
        let span = SourceByteRange::new(self.span, end).map_err(|e| {
            NoteIngestError::Parse(NoteParseError::InvalidTopology {
                code: "parser.structure.invalid_block_range",
                detail: format!(
                    "failed to construct range for closed block: {e}"
                )
                .into(),
                range: None,
            })
        })?;

        let kind = match self.kind {
            BlockKind::Leaf(leaf) => BlockKind::Leaf(match leaf {
                LeafBlockKind::Paragraph {
                    events,
                } => LeafBlockKind::Paragraph {
                    events,
                },
                LeafBlockKind::Heading {
                    level,
                    events,
                } => LeafBlockKind::Heading {
                    level,
                    events,
                },
                LeafBlockKind::CodeBlock {
                    language,
                    text,
                } => LeafBlockKind::CodeBlock {
                    language,
                    text: TextSequence::from_events(&text).as_plain_text(),
                },
                LeafBlockKind::Frontmatter {
                    format,
                    text,
                } => LeafBlockKind::Frontmatter {
                    format,
                    text: TextSequence::from_events(&text).as_plain_text(),
                },
                LeafBlockKind::ThematicBreak => LeafBlockKind::ThematicBreak,
            }),
            BlockKind::Container(container) => BlockKind::Container(container),
        };

        Ok(Block {
            kind,
            span,
        })
    }
}

/// The type and content of a markdown block.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub(crate) enum BlockKind<'source, S: BlockState = Closed> {
    /// Content-bearing block variant wrapper.
    Leaf(LeafBlockKind<'source, S>),
    /// Structure-bearing block variant wrapper.
    Container(ContainerBlockKind<'source>),
}

impl<'source> BlockKind<'source, Open> {
    /// Pushes an inline event to a leaf block.
    pub(crate) fn push_inline_event(
        &mut self,
        event: RangedEvent<'source>,
    ) -> Result<(), NoteParseError> {
        match self {
            Self::Leaf(leaf) => leaf.push_inline_event(event),
            Self::Container(_) => Err(NoteParseError::InvalidTopology {
                code: "parser.structure.push_inline_to_container",
                detail: "cannot push inline events to a container block".into(),
                range: Some(event.range()),
            }),
        }
    }

    /// Pushes a finalized child block to a container block.
    pub(crate) fn push_child(
        &mut self,
        child: Block<'source, Closed>,
    ) -> Result<(), NoteParseError> {
        match self {
            Self::Container(container) => {
                container.push_child(child);
                Ok(())
            }
            Self::Leaf(_) => Err(NoteParseError::InvalidTopology {
                code: "parser.structure.push_child_to_leaf",
                detail: "cannot push child blocks to a leaf block".into(),
                range: Some(child.span),
            }),
        }
    }
}

impl<'source, S: BlockState> BlockKind<'source, S> {
    /// Returns the expected end token for this block kind.
    #[must_use]
    pub(crate) fn expected_end(&self) -> BlockEnd {
        match self {
            Self::Leaf(leaf) => match leaf {
                LeafBlockKind::Paragraph {
                    ..
                }
                | LeafBlockKind::ThematicBreak => BlockEnd::Paragraph, /* Paragraph for thematic break is a fallback */
                LeafBlockKind::Heading {
                    ..
                } => BlockEnd::Heading,
                LeafBlockKind::CodeBlock {
                    ..
                } => BlockEnd::CodeBlock,
                LeafBlockKind::Frontmatter {
                    ..
                } => BlockEnd::Frontmatter,
            },
            Self::Container(container) => match container {
                ContainerBlockKind::BlockQuote {
                    ..
                } => BlockEnd::BlockQuote,
                ContainerBlockKind::List {
                    ..
                } => BlockEnd::List,
                ContainerBlockKind::ListItem {
                    ..
                } => BlockEnd::ListItem,
            },
        }
    }

    /// Returns a slice of the child blocks for this block kind.
    ///
    /// If this is a leaf block, returns an empty slice.
    #[must_use]
    #[inline]
    pub(crate) fn children(&self) -> &[Block<'source, Closed>] {
        match self {
            Self::Container(container) => match container {
                ContainerBlockKind::BlockQuote {
                    children,
                }
                | ContainerBlockKind::List {
                    children,
                    ..
                }
                | ContainerBlockKind::ListItem {
                    children,
                    ..
                } => children,
            },
            Self::Leaf(_) => &[],
        }
    }
}

/// Content-bearing markdown block variants.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub(crate) enum LeafBlockKind<'source, S: BlockState = Closed> {
    /// Paragraph block containing inline content.
    Paragraph {
        /// Inline events captured for the paragraph span.
        events: Vec<RangedEvent<'source>>,
    },
    /// Heading block (H1-H6) with inline content.
    Heading {
        /// Heading level metadata.
        level: HeadingLevel,
        /// Inline events captured for the heading span.
        events: Vec<RangedEvent<'source>>,
    },
    /// Fenced or indented code block.
    CodeBlock {
        /// Optional fenced language info string.
        language: Option<Box<str>>,
        /// Code text content (Events in Open, String in Closed).
        text: S::LeafData<'source>,
    },
    /// YAML/TOML frontmatter block.
    Frontmatter {
        /// Source frontmatter flavor (YAML: `---`, TOML: `+++`).
        format: FrontmatterFormat,
        /// Frontmatter body text (Events in Open, String in Closed).
        text: S::LeafData<'source>,
    },
    /// Thematic break (horizontal rule: `---`, `***`, or `___`).
    ThematicBreak,
}

impl<'source> LeafBlockKind<'source, Open> {
    /// Pushes an inline event to the leaf block's buffer.
    pub(crate) fn push_inline_event(
        &mut self,
        event: RangedEvent<'source>,
    ) -> Result<(), NoteParseError> {
        match self {
            Self::Paragraph {
                events,
            }
            | Self::Heading {
                events,
                ..
            } => {
                events.push(event);
                Ok(())
            }
            Self::CodeBlock {
                text,
                ..
            }
            | Self::Frontmatter {
                text,
                ..
            } => {
                text.push(event);
                Ok(())
            }
            Self::ThematicBreak => Err(NoteParseError::InvalidTopology {
                code: "parser.structure.push_inline_to_thematic_break",
                detail: "cannot push inline events to a thematic break".into(),
                range: Some(event.range()),
            }),
        }
    }
}

/// Structure-bearing markdown block variants.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub(crate) enum ContainerBlockKind<'source> {
    /// Blockquote containing nested blocks.
    BlockQuote {
        /// Nested blocks contained by this quote.
        children: Vec<Block<'source, Closed>>,
    },
    /// Ordered or unordered list containing list items.
    List {
        /// Ordered/unordered list metadata.
        kind: super::types::ListKind,
        /// Nested list item blocks.
        children: Vec<Block<'source, Closed>>,
    },
    /// Individual list item (can contain paragraphs, sublists, etc.).
    ListItem {
        /// Nesting depth (0 = root, 1 = first level, etc.).
        depth: u32,
        /// Byte offset of the parent list item start when nested.
        parent_pos: Option<SourceByteOffset>,
        /// Checkbox state.
        is_checked: Option<bool>,
        /// Child blocks.
        children: Vec<Block<'source, Closed>>,
    },
}

impl<'source> ContainerBlockKind<'source> {
    /// Pushes a finalized child block into this container.
    pub(crate) fn push_child(&mut self, child: Block<'source, Closed>) {
        match self {
            Self::BlockQuote {
                children,
            }
            | Self::List {
                children,
                ..
            }
            | Self::ListItem {
                children,
                ..
            } => {
                children.push(child);
            }
        }
    }
}
