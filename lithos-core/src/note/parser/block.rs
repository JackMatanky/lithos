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

#![cfg_attr(
    not(test),
    expect(dead_code, reason = "Parser block model is consumed incrementally")
)]

use pulldown_cmark::{CowStr, MetadataBlockKind};

use super::stream::{EventWithRange, InlineEvent, ParserEvent};
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
/// // Extract text using helper method
/// assert_eq!(block.text(), Some("Hello".to_string()));
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

impl Block<'_> {
    /// Extract plain text from inline events (lazy evaluation).
    ///
    /// Returns `Some(String)` for leaf blocks that contain inline content
    /// (Paragraph, Heading). Returns `None` for container blocks, code blocks,
    /// and other non-scannable blocks.
    ///
    /// # Performance
    ///
    /// This method allocates a new `String` on each call. For repeated access,
    /// cache the result.
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Pattern matching borrowed enum variants in block helper"
    )]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "BlockKind is non_exhaustive; fallback keeps helper \
                  forward-compatible"
    )]
    pub(crate) fn text(&self) -> Option<String> {
        match &self.kind {
            BlockKind::Leaf(
                LeafBlockKind::Paragraph {
                    events,
                }
                | LeafBlockKind::Heading {
                    events,
                    ..
                },
            ) => Some(inline_events_text(events)),
            _ => None,
        }
    }

    /// Returns true if this block should be scanned for metadata.
    ///
    /// Code blocks and frontmatter return false (we don't scan code or
    /// frontmatter content for tags/fields). All other blocks return true.
    #[must_use]
    #[inline]
    #[expect(dead_code, reason = "Consumed by downstream scanner traversal")]
    pub(crate) fn is_scannable(&self) -> bool {
        !matches!(
            self.kind,
            BlockKind::Leaf(
                LeafBlockKind::CodeBlock { .. }
                    | LeafBlockKind::Frontmatter { .. }
            )
        )
    }
}

/// Extract plain text from inline text events.
#[must_use]
pub(crate) fn inline_events_text(events: &[EventWithRange<'_>]) -> String {
    events
        .iter()
        .filter_map(|e| {
            #[expect(
                clippy::pattern_type_mismatch,
                reason = "Matching on borrowed enum variant from accessor"
            )]
            #[expect(
                clippy::wildcard_enum_match_arm,
                reason = "ParserEvent is non_exhaustive at this matching layer"
            )]
            match e.event() {
                ParserEvent::Inline(InlineEvent::Text(s)) => Some(s.as_ref()),
                _ => None,
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

/// The type and content of a markdown block.
///
/// This enum distinguishes between **leaf blocks** (which contain inline
/// content like text and code spans) and **container blocks** (which contain
/// other blocks). The structure closely mirrors the `CommonMark` specification.
///
/// # Leaf vs Container
///
/// - **Leaf blocks**: Store `events: Vec<EventWithRange>` (inline content)
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
        events: Vec<EventWithRange<'source>>,
    },
    /// Heading block (H1-H6) with inline content.
    Heading {
        /// Heading level metadata.
        level: HeadingLevel,
        /// Inline events captured for the heading span.
        events: Vec<EventWithRange<'source>>,
    },
    /// Fenced or indented code block.
    CodeBlock {
        /// Optional fenced language info string.
        language: Option<CowStr<'source>>,
        /// Flattened code text content.
        text: String,
    },
    /// YAML/TOML frontmatter block.
    Frontmatter {
        /// Source frontmatter flavor (YAML: `---`, TOML: `+++`).
        format: MetadataBlockKind,
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
        kind: ListKind,
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

/// Heading level (H1 through H6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl From<pulldown_cmark::HeadingLevel> for HeadingLevel {
    fn from(level: pulldown_cmark::HeadingLevel) -> Self {
        match level {
            pulldown_cmark::HeadingLevel::H1 => Self::H1,
            pulldown_cmark::HeadingLevel::H2 => Self::H2,
            pulldown_cmark::HeadingLevel::H3 => Self::H3,
            pulldown_cmark::HeadingLevel::H4 => Self::H4,
            pulldown_cmark::HeadingLevel::H5 => Self::H5,
            pulldown_cmark::HeadingLevel::H6 => Self::H6,
        }
    }
}

impl HeadingLevel {
    /// Convert to numeric level (1-6).
    #[must_use]
    #[inline]
    #[expect(dead_code, reason = "Used by downstream heading projections")]
    pub(crate) const fn as_u8(self) -> u8 {
        match self {
            Self::H1 => 1,
            Self::H2 => 2,
            Self::H3 => 3,
            Self::H4 => 4,
            Self::H5 => 5,
            Self::H6 => 6,
        }
    }
}

/// List type (ordered or unordered).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ListKind {
    /// Unordered list (`-`, `*`, or `+` markers).
    Unordered,
    /// Ordered list (`1.`, `2.`, etc. markers).
    Ordered {
        /// Starting number (usually 1, but can be any positive integer).
        start: u64,
    },
}
