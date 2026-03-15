//! Minimal AST types for note ingestion.

use crate::note::position::SourceByteRange;

/// Minimal node wrapper with byte range information.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct AstNode {
    kind: AstNodeKind,
    range: SourceByteRange,
}

impl AstNode {
    /// Create a new AST node.
    #[inline]
    #[must_use]
    pub const fn new(kind: AstNodeKind, range: SourceByteRange) -> Self {
        Self {
            kind,
            range,
        }
    }

    /// Return the node kind.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> &AstNodeKind {
        &self.kind
    }

    /// Return the byte range for this node.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }
}

/// Minimal structural node types required by note extraction.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AstNodeKind {
    /// Heading block with inline text.
    Heading {
        /// Heading level (1–6).
        level: u8,
        /// Heading text with inline styles preserved.
        text: Text,
    },
    /// Paragraph block with inline text.
    Paragraph {
        /// Paragraph text with inline styles preserved.
        text: Text,
    },
    /// List start marker with list type metadata.
    ListStart {
        /// Ordered or unordered list type.
        list_type: AstListType,
    },
    /// List end marker.
    ListEnd,
    /// List item with optional task marker.
    ListItem {
        /// List item text with inline styles preserved.
        text: Text,
        /// Task marker state if this list item is a task.
        task: Option<bool>,
    },
    /// Code block boundary for sectioning and tag exclusion.
    CodeBlock {
        /// Whether the block is fenced.
        fenced: bool,
        /// Optional fence info string.
        info: Option<Box<str>>,
    },
    /// Block quote boundary (including callouts).
    BlockQuote {
        /// Optional callout kind.
        kind: Option<AstBlockQuoteKind>,
    },
    /// Link or embed with alias text captured separately.
    Link {
        /// Link style (wiki or markdown).
        style: AstLinkStyle,
        /// Whether this is an embed.
        is_embed: bool,
        /// Raw target string as produced by pulldown-cmark.
        target: Box<str>,
        /// Alias text captured from link text.
        alias: Text,
    },
}

/// Inline text collection for a node.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Text {
    nodes: Vec<TextNode>,
}

impl Text {
    /// Create a text collection from nodes.
    #[inline]
    #[must_use]
    pub fn new(nodes: Vec<TextNode>) -> Self {
        Self {
            nodes,
        }
    }

    /// Return the text nodes in source order.
    #[inline]
    #[must_use]
    pub fn nodes(&self) -> &[TextNode] {
        &self.nodes
    }

    /// Return true when no text nodes are present.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Concatenate text nodes into a single string.
    #[must_use]
    pub fn to_string(&self) -> String {
        let mut out = String::new();
        for node in &self.nodes {
            out.push_str(node.content());
        }
        out
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
    /// Create a new text node.
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

    /// Return the raw text content.
    #[inline]
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Return the inline style applied to this fragment.
    #[inline]
    #[must_use]
    pub const fn style(&self) -> TextStyle {
        self.style
    }

    /// Return the origin classification for this fragment.
    #[inline]
    #[must_use]
    pub const fn origin(&self) -> TextOrigin {
        self.origin
    }

    /// Return the byte range for this fragment.
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

/// Classification for where a text fragment came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TextOrigin {
    /// Regular document text.
    Normal,
    /// Link alias text (excluded from tag scanning).
    LinkAlias,
}

/// List type metadata for list item nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AstListType {
    /// Ordered list with starting number.
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
pub enum AstLinkStyle {
    /// Obsidian-style wiki link.
    Wiki,
    /// Markdown link.
    Markdown,
}

/// Optional callout kinds for block quotes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AstBlockQuoteKind {
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
