//! Minimal AST types for note ingestion.
//!
//! The AST is intentionally small and structural. It records block boundaries,
//! inline text fragments, and byte ranges, but does not embed domain semantics.
//! List and block-quote structure are represented as nested nodes so the raw
//! layer can traverse the tree without relying on parser-specific state. Note
//! that paragraphs only appear when the pulldown-cmark event stream emits them
//! (tight list items may omit paragraph tags entirely).

use std::fmt;

use crate::note::position::SourceByteRange;

/// Minimal AST node wrapper with byte range information.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Node {
    kind: NodeKind,
    range: SourceByteRange,
}

impl Node {
    /// Creates a new AST node.
    #[inline]
    #[must_use]
    pub const fn new(kind: NodeKind, range: SourceByteRange) -> Self {
        Self {
            kind,
            range,
        }
    }

    /// Returns the node kind.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> &NodeKind {
        &self.kind
    }

    /// Returns the byte range for this node.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }
}

/// Structural node types required by raw extraction.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum NodeKind {
    /// Heading block with inline text.
    Heading {
        /// Heading level, from 1 through 6.
        level: u8,
        /// Heading text with inline styles preserved.
        text: Text,
        /// Inline links captured within the heading text.
        links: Vec<InlineLink>,
    },
    /// Paragraph block with inline text.
    Paragraph {
        /// Paragraph text with inline styles preserved.
        text: Text,
        /// Inline links captured within the paragraph text.
        links: Vec<InlineLink>,
    },
    /// List container with ordered or unordered metadata.
    List {
        /// Ordered or unordered list type, as reported by the parser.
        list_type: ListStyle,
        /// List items in source order.
        items: Vec<Node>,
    },
    /// List item with optional task marker.
    ListItem {
        /// List item text with inline styles preserved.
        text: Text,
        /// Task marker presence emitted by the parser.
        ///
        /// `Some(true)` means checked, `Some(false)` means unchecked.
        task_marker: Option<bool>,
        /// Inline links captured within the list item text.
        links: Vec<InlineLink>,
        /// Nested nodes contained by this list item.
        children: Vec<Node>,
    },
    /// Code block boundary used for sectioning and tag exclusion.
    CodeBlock {
        /// Whether the block is fenced.
        fenced: bool,
        /// Optional fence info string.
        info: Option<Box<str>>,
        /// Raw code block text as emitted by pulldown-cmark.
        text: Box<str>,
    },
    /// Block quote boundary, including callouts.
    BlockQuote {
        /// Optional callout kind.
        kind: Option<BlockQuoteKind>,
        /// Nested nodes contained by this block quote.
        nodes: Vec<Node>,
    },
}

impl NodeKind {
    /// Returns true when the node kind captures inline text containers.
    #[inline]
    #[must_use]
    pub const fn is_text_container(&self) -> bool {
        matches!(
            self,
            Self::Heading { .. }
                | Self::Paragraph { .. }
                | Self::ListItem { .. }
        )
    }
}

/// Inline link captured within a text container.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct InlineLink {
    style: LinkStyle,
    is_embed: bool,
    target: Box<str>,
    alias: Text,
    range: SourceByteRange,
}

impl InlineLink {
    /// Creates a new inline link descriptor.
    #[inline]
    #[must_use]
    pub fn new(
        style: LinkStyle,
        is_embed: bool,
        target: Box<str>,
        alias: Text,
        range: SourceByteRange,
    ) -> Self {
        Self {
            style,
            is_embed,
            target,
            alias,
            range,
        }
    }

    /// Returns the link style.
    #[inline]
    #[must_use]
    pub const fn style(&self) -> LinkStyle {
        self.style
    }

    /// Returns true when this link is an embed.
    #[inline]
    #[must_use]
    pub const fn is_embed(&self) -> bool {
        self.is_embed
    }

    /// Returns the raw target string.
    #[inline]
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Returns the alias text captured from the link body.
    #[inline]
    #[must_use]
    pub fn alias(&self) -> &Text {
        &self.alias
    }

    /// Returns the byte range for this inline link.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }
}

/// Collection of inline text fragments for a node.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Text {
    nodes: Vec<TextNode>,
}

impl Text {
    /// Creates a text collection from nodes.
    #[inline]
    #[must_use]
    pub fn new(nodes: Vec<TextNode>) -> Self {
        Self {
            nodes,
        }
    }

    /// Returns the text nodes in source order.
    #[inline]
    #[must_use]
    pub fn nodes(&self) -> &[TextNode] {
        &self.nodes
    }

    /// Returns true when no text nodes are present.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Concatenates text nodes into a boxed string.
    #[must_use]
    pub fn to_boxed_str(&self) -> Box<str> {
        let mut out = String::with_capacity(self.byte_len());
        for node in &self.nodes {
            out.push_str(node.content());
        }
        out.into_boxed_str()
    }

    fn byte_len(&self) -> usize {
        self.nodes.iter().map(|node| node.content().len()).sum()
    }

    /// Appends a text node to the collection.
    #[inline]
    pub fn append(&mut self, node: TextNode) {
        self.nodes.push(node);
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for node in &self.nodes {
            f.write_str(node.content())?;
        }
        Ok(())
    }
}

/// Single inline text fragment with style and origin metadata.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct TextNode {
    content: Box<str>,
    style: TextStyle,
    origin: TextOrigin,
    range: SourceByteRange,
}

impl TextNode {
    /// Creates a new text node.
    #[inline]
    #[must_use]
    pub fn new(
        content: Box<str>,
        style: TextStyle,
        origin: TextOrigin,
        range: SourceByteRange,
    ) -> Self {
        Self {
            content,
            style,
            origin,
            range,
        }
    }

    /// Returns the raw text content.
    #[inline]
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns the inline style applied to this fragment.
    #[inline]
    #[must_use]
    pub const fn style(&self) -> TextStyle {
        self.style
    }

    /// Returns the origin classification for this fragment.
    #[inline]
    #[must_use]
    pub const fn origin(&self) -> TextOrigin {
        self.origin
    }

    /// Returns the byte range for this fragment.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }
}

/// Inline style variants preserved in text nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TextStyle {
    /// Unstyled text.
    Plain,
    /// Inline code span.
    Code,
    /// Emphasis span.
    Emphasis,
    /// Strong emphasis span.
    Strong,
    /// Strikethrough span.
    Strikethrough,
}

/// Classification for the source of a text fragment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TextOrigin {
    /// Regular document text.
    Normal,
    /// Link alias text, excluded from tag scanning in raw extraction.
    LinkAlias,
}

/// List type metadata for list item nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ListStyle {
    /// Ordered list with a starting number.
    Ordered {
        /// Starting index for the ordered list.
        start: u64,
    },
    /// Unordered list marker.
    Unordered,
}

/// Link style metadata for link nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LinkStyle {
    /// Obsidian-style wiki link.
    Wiki,
    /// Markdown link.
    Markdown,
}

/// Optional callout kinds for block quotes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BlockQuoteKind {
    /// `> [!note]`.
    Note,
    /// `> [!tip]`.
    Tip,
    /// `> [!important]`.
    Important,
    /// `> [!warning]`.
    Warning,
    /// `> [!caution]`.
    Caution,
}
