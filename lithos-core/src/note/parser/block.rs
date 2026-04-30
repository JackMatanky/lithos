//! Block domain types for markdown document structure.
//!
//! This module provides the core AST data structures that represent the
//! hierarchical structure of a parsed markdown document. The AST follows
//! CommonMark semantics, distinguishing between **leaf blocks**
//! (content-bearing) and **container blocks** (structure-bearing).
//!
//! # Design Philosophy
//!
//! - **Minimal AST**: Only structure and content, no metadata extraction
//! - **CommonMark-aligned**: Block types map directly to CommonMark spec
//! - **Zero-copy where possible**: Events borrow from source via lifetimes
//! - **Explicit nesting**: Container blocks have `children` fields

use super::types::{FrontmatterFormat, HeadingLevel, RangedEvent};
use crate::note::position::SourceByteRange;

/// A complete block in the markdown document tree.
///
/// Each block represents a single structural element from the markdown source,
/// such as a paragraph, heading, list, or code block. Blocks form a tree
/// structure through the `children` fields in container block variants.
///
/// # Lifecycle
///
/// Blocks are created during AST building by finalizing temporary processing
/// nodes. Once created, blocks are immutable and can be safely borrowed across
/// multiple pipeline stages.
///
/// # Examples
///
/// ```rust,ignore
/// // A simple paragraph block
/// let block = Block {
///     kind: BlockKind::Leaf(LeafBlockKind::Paragraph {
///         events: vec![],
///     }),
///     span: SourceByteRange::new(start, end)?,
/// };
///
/// // Text/scannable projection is derived from `text::TextSequence`.
/// ```
#[derive(Debug, Clone, PartialEq)]
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "module-private struct with deliberate pub(crate) fields"
)]
pub(crate) struct Block<'source> {
    /// The type and content of this block.
    pub(crate) kind: BlockKind<'source>,
    /// The complete source byte range (both start and end known).
    pub(crate) span: SourceByteRange,
}

/// The type and content of a markdown block.
///
/// This enum distinguishes between **leaf blocks** (which contain inline
/// content like text and code spans) and **container blocks** (which contain
/// other blocks). The structure closely mirrors the `CommonMark` specification.
///
/// # Leaf vs Container
///
/// - **Leaf blocks**: Store `events: Vec<RangedEvent>` (inline content)
/// - **Container blocks**: Store `children: Vec<Block>` (nested blocks)
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub(crate) enum BlockKind<'source> {
    /// Content-bearing block variant wrapper.
    Leaf(LeafBlockKind<'source>),
    /// Structure-bearing block variant wrapper.
    Container(ContainerBlockKind<'source>),
}

/// Content-bearing markdown block variants.
///
/// Leaf blocks either contain inline event streams (paragraph/heading),
/// canonical text payloads (code/frontmatter), or standalone markers
/// (`ThematicBreak`). They do not own child blocks.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub(crate) enum LeafBlockKind<'source> {
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
        /// Flattened code text content.
        text: String,
    },
    /// YAML/TOML frontmatter block.
    Frontmatter {
        /// Source frontmatter flavor (YAML: `---`, TOML: `+++`).
        format: FrontmatterFormat,
        /// Flattened frontmatter body text.
        text: String,
    },
    /// Thematic break (horizontal rule: `---`, `***`, or `___`).
    ThematicBreak,
}

/// Structure-bearing markdown block variants.
///
/// Container blocks own nested child blocks and carry structural metadata.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub(crate) enum ContainerBlockKind<'source> {
    /// Blockquote containing nested blocks.
    BlockQuote {
        /// Nested blocks contained by this quote.
        children: Vec<Block<'source>>,
    },
    /// Ordered or unordered list containing list items.
    List {
        /// Ordered/unordered list metadata.
        kind: super::types::ListKind,
        /// Nested list item blocks.
        children: Vec<Block<'source>>,
    },
    /// Individual list item (can contain paragraphs, sublists, etc.).
    ListItem {
        /// Nesting depth (0 = root, 1 = first level, etc.).
        depth: u32,
        /// Byte range of parent list item when nested.
        parent_span: Option<SourceByteRange>,
        /// Checkbox state:
        /// - `is_checked == Some(true)`: Checked task `- [x] Done`.
        /// - `is_checked == Some(false)`: Unchecked task `- [ ] Todo`.
        /// - `is_checked == None`: Regular list item `- Item`.
        is_checked: Option<bool>,
        /// Child blocks (paragraphs, code, sublists, etc.).
        children: Vec<Block<'source>>,
    },
}
