//! Derived inline text projection types.
//!
//! This module projects parser inline IR (`RangedEvent`) into stable text
//! nodes. It is policy-agnostic: consumer layers (scanner, assembler, link
//! handling) decide which nodes to include for their own use cases.
//!
//! # The Text Contract
//!
//! To ensure consistent behavior across scanning, indexing, and display, all
//! text consumers must filter `TextNode` collections using the following rules:
//!
//! 1. **Scanning (`is_scannable`)**:
//!     * **Context**: Must be `Normal`. Nodes inside `LinkLabel` or `ImageAlt`
//!       are ignored to prevent local link artifacts from being promoted to
//!       global note artifacts.
//!     * **Style**: Must NOT contain `Code`, `MathInline`, or `MathDisplay`.
//!       Content inside backticks or math delimiters is technical IR and should
//!       not be scanned for domain artifacts like tags or fields.
//!
//! 2. **Display (`is_displayable`)**:
//!     * **Style**: Must NOT contain `MathInline` or `MathDisplay`. These
//!       variants require specialized rendering and are inhibited when
//!       projecting to "plain" display text (e.g. for link labels or section
//!       summaries).
//!     * **Note**: `Code` style IS displayable as it represents literal text.

#![expect(
    clippy::pattern_type_mismatch,
    reason = "Inline event matching intentionally uses borrowed tokens"
)]

use super::types::{
    InlineDelimiterEnd, InlineDelimiterStart, InlineToken, MathKind,
    ParserEvent, RangedEvent,
};
use crate::note::position::SourceByteRange;

/// Link/image context for derived text nodes.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum TextContext {
    Normal,
    LinkLabel,
    ImageAlt,
}

/// Derived style marker for text nodes.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct TextStyle(u8);

impl TextStyle {
    pub(crate) const CODE: Self = Self(1 << 3);
    pub(crate) const EMPHASIS: Self = Self(1 << 0);
    pub(crate) const MATH_DISPLAY: Self = Self(1 << 5);
    pub(crate) const MATH_INLINE: Self = Self(1 << 4);
    pub(crate) const NONE: Self = Self(0);
    pub(crate) const STRIKETHROUGH: Self = Self(1 << 2);
    pub(crate) const STRONG: Self = Self(1 << 1);

    #[must_use]
    #[inline]
    pub(crate) const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub(crate) fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    #[inline]
    pub(crate) fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }
}

/// State tracking for projecting text sequences from an event stream.
#[derive(Clone, Debug)]
pub(crate) struct InlineStyleContext {
    styles: TextStyle,
    link_depth: u32,
    image_depth: u32,
}

impl InlineStyleContext {
    #[must_use]
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            styles: TextStyle::NONE,
            link_depth: 0,
            image_depth: 0,
        }
    }

    #[inline]
    pub(crate) fn apply_start(&mut self, start: &InlineDelimiterStart<'_>) {
        match start {
            InlineDelimiterStart::Emphasis => {
                self.styles.insert(TextStyle::EMPHASIS);
            }
            InlineDelimiterStart::Strong => {
                self.styles.insert(TextStyle::STRONG);
            }
            InlineDelimiterStart::Strikethrough => {
                self.styles.insert(TextStyle::STRIKETHROUGH);
            }
            InlineDelimiterStart::Link {
                ..
            } => {
                self.link_depth = self.link_depth.saturating_add(1);
            }
            InlineDelimiterStart::Image {
                ..
            } => {
                self.image_depth = self.image_depth.saturating_add(1);
            }
            InlineDelimiterStart::Superscript
            | InlineDelimiterStart::Subscript
            | InlineDelimiterStart::_Marker(_) => {}
        }
    }

    #[inline]
    pub(crate) fn apply_end(&mut self, end: InlineDelimiterEnd) {
        match end {
            InlineDelimiterEnd::Emphasis => {
                self.styles.remove(TextStyle::EMPHASIS);
            }
            InlineDelimiterEnd::Strong => self.styles.remove(TextStyle::STRONG),
            InlineDelimiterEnd::Strikethrough => {
                self.styles.remove(TextStyle::STRIKETHROUGH);
            }
            InlineDelimiterEnd::Link => {
                self.link_depth = self.link_depth.saturating_sub(1);
            }
            InlineDelimiterEnd::Image => {
                self.image_depth = self.image_depth.saturating_sub(1);
            }
            InlineDelimiterEnd::Superscript | InlineDelimiterEnd::Subscript => {
            }
        }
    }

    #[must_use]
    #[inline]
    pub(crate) const fn context(&self) -> TextContext {
        if self.image_depth > 0 {
            TextContext::ImageAlt
        } else if self.link_depth > 0 {
            TextContext::LinkLabel
        } else {
            TextContext::Normal
        }
    }

    #[must_use]
    #[inline]
    pub(crate) fn create_node(
        &self,
        text: Box<str>,
        extra_style: Option<TextStyle>,
        range: SourceByteRange,
    ) -> TextNode {
        let mut styles = self.styles;
        if let Some(extra) = extra_style {
            styles.insert(extra);
        }
        TextNode::new(text, styles, self.context(), range)
    }
}

/// Derived text node with style stack and source span.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TextNode {
    text: Box<str>,
    styles: TextStyle,
    context: TextContext,
    range: SourceByteRange,
}

impl TextNode {
    #[must_use]
    #[inline]
    pub(crate) const fn new(
        text: Box<str>,
        styles: TextStyle,
        context: TextContext,
        range: SourceByteRange,
    ) -> Self {
        Self {
            text,
            styles,
            context,
            range,
        }
    }

    #[must_use]
    #[inline]
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    #[inline]
    #[cfg(test)]
    pub(crate) const fn styles(&self) -> TextStyle {
        self.styles
    }

    #[must_use]
    #[inline]
    #[cfg(test)]
    pub(crate) const fn context(&self) -> TextContext {
        self.context
    }

    #[must_use]
    #[inline]
    pub(crate) const fn range(&self) -> SourceByteRange {
        self.range
    }

    /// Returns true if the node is eligible for artifact scanning (tags, etc).
    #[must_use]
    #[inline]
    pub(crate) const fn is_scannable(&self) -> bool {
        matches!(self.context, TextContext::Normal)
            && !self.styles.contains(TextStyle::CODE)
            && !self.styles.contains(TextStyle::MATH_INLINE)
            && !self.styles.contains(TextStyle::MATH_DISPLAY)
    }

    /// Returns true if the node is eligible for display (e.g. in link labels).
    #[must_use]
    #[inline]
    pub(crate) const fn is_displayable(&self) -> bool {
        !self.styles.contains(TextStyle::MATH_INLINE)
            && !self.styles.contains(TextStyle::MATH_DISPLAY)
    }
}

/// Ordered collection of derived text nodes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct TextSequence {
    nodes: Vec<TextNode>,
}

impl TextSequence {
    #[must_use]
    #[inline]
    #[cfg_attr(not(test), expect(dead_code, reason = "Used in tests"))]
    pub(crate) const fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }

    #[must_use]
    #[inline]
    pub(crate) const fn from_nodes(nodes: Vec<TextNode>) -> Self {
        Self {
            nodes,
        }
    }

    #[inline]
    #[cfg_attr(not(test), expect(dead_code, reason = "Used in tests"))]
    pub(crate) fn push(&mut self, node: TextNode) {
        self.nodes.push(node);
    }

    #[must_use]
    #[inline]
    pub(crate) fn nodes(&self) -> &[TextNode] {
        &self.nodes
    }

    #[must_use]
    #[inline]
    #[cfg_attr(not(test), expect(dead_code, reason = "Used in tests"))]
    pub(crate) fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Renders plain text from all nodes in order.
    #[must_use]
    pub(crate) fn as_plain_text(&self) -> String {
        self.nodes.iter().map(TextNode::text).collect()
    }

    /// Renders displayable text from nodes, filtering out non-displayable
    /// content (e.g. math) according to the Text Contract.
    #[must_use]
    pub(crate) fn as_displayable_text(&self) -> String {
        self.nodes
            .iter()
            .filter(|node| node.is_displayable())
            .map(TextNode::text)
            .collect()
    }

    /// Returns source-covering span of first..last node.
    #[must_use]
    pub(crate) fn covering_range(&self) -> Option<SourceByteRange> {
        let first = self.nodes.first()?;
        let last = self.nodes.last()?;
        SourceByteRange::new(first.range().start(), last.range().end()).ok()
    }

    /// Projects parser events into text nodes without consumer policy.
    #[must_use]
    pub(crate) fn from_events(events: &[RangedEvent<'_>]) -> Self {
        let mut nodes = Vec::new();
        let mut style_ctx = InlineStyleContext::new();

        for event in events {
            match event.event() {
                ParserEvent::Inline(InlineToken::DelimiterStart(start)) => {
                    style_ctx.apply_start(start);
                }
                ParserEvent::Inline(InlineToken::DelimiterEnd(end)) => {
                    style_ctx.apply_end(*end);
                }
                ParserEvent::Inline(InlineToken::Text(text)) => {
                    nodes.push(style_ctx.create_node(
                        Box::from(text.as_ref()),
                        None,
                        event.range(),
                    ));
                }
                ParserEvent::Inline(InlineToken::InlineCode(text)) => {
                    nodes.push(style_ctx.create_node(
                        Box::from(text.as_ref()),
                        Some(TextStyle::CODE),
                        event.range(),
                    ));
                }
                ParserEvent::Inline(InlineToken::Math {
                    kind,
                    content,
                }) => {
                    let extra = match kind {
                        MathKind::Inline => TextStyle::MATH_INLINE,
                        MathKind::Display => TextStyle::MATH_DISPLAY,
                    };
                    nodes.push(style_ctx.create_node(
                        Box::from(content.as_ref()),
                        Some(extra),
                        event.range(),
                    ));
                }
                ParserEvent::Inline(InlineToken::Html(html)) => {
                    nodes.push(style_ctx.create_node(
                        Box::from(html.as_ref()),
                        None,
                        event.range(),
                    ));
                }
                ParserEvent::Inline(InlineToken::LineBreak(_)) => {
                    nodes.push(style_ctx.create_node(
                        "".into(),
                        None,
                        event.range(),
                    ));
                }
                ParserEvent::Inline(InlineToken::FootnoteReference(label)) => {
                    nodes.push(style_ctx.create_node(
                        Box::from(format!("[^{label}]").as_str()),
                        None,
                        event.range(),
                    ));
                }
                ParserEvent::BlockStart(_)
                | ParserEvent::BlockEnd(_)
                | ParserEvent::TaskListMarker(_)
                | ParserEvent::ThematicBreak => {}
            }
        }

        Self::from_nodes(nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_range() -> SourceByteRange {
        SourceByteRange::try_from(1..4).expect("valid range")
    }

    #[test]
    fn text_node_accessors_return_constructor_values() {
        let range = sample_range();
        let mut styles = TextStyle::STRONG;
        styles.insert(TextStyle::CODE);
        let node = TextNode::new(
            "hello".into(),
            styles,
            TextContext::LinkLabel,
            range,
        );

        assert_eq!(node.text(), "hello");
        assert!(node.styles().contains(TextStyle::STRONG));
        assert!(node.styles().contains(TextStyle::CODE));
        assert_eq!(node.context(), TextContext::LinkLabel);
        assert_eq!(node.range(), range);
    }

    #[test]
    fn text_sequence_push_preserves_order() {
        let mut sequence = TextSequence::new();
        let first = TextNode::new(
            "a".into(),
            TextStyle::EMPHASIS,
            TextContext::Normal,
            sample_range(),
        );
        let second = TextNode::new(
            "b".into(),
            TextStyle::STRONG,
            TextContext::Normal,
            sample_range(),
        );

        sequence.push(first.clone());
        sequence.push(second.clone());

        assert_eq!(sequence.nodes(), [first, second]);
        assert!(!sequence.is_empty());
    }

    #[test]
    fn from_events_marks_link_text_with_link_label_context() {
        use crate::note::parser::types::{
            InlineDelimiterEnd, InlineDelimiterStart, LinkKind,
        };

        let start = RangedEvent::try_from((
            ParserEvent::Inline(InlineToken::DelimiterStart(
                InlineDelimiterStart::Link {
                    kind: LinkKind::Inline,
                    destination: "a".into(),
                    title: "".into(),
                    label: "".into(),
                },
            )),
            0..1,
        ))
        .expect("start event");
        let text = RangedEvent::try_from((
            ParserEvent::Inline(InlineToken::Text("tag".into())),
            1..4,
        ))
        .expect("text event");
        let end = RangedEvent::try_from((
            ParserEvent::Inline(InlineToken::DelimiterEnd(
                InlineDelimiterEnd::Link,
            )),
            4..5,
        ))
        .expect("end event");

        let sequence = TextSequence::from_events(&[start, text, end]);

        assert_eq!(sequence.as_plain_text(), "tag");
        assert_eq!(
            sequence.nodes().first().map(TextNode::context),
            Some(TextContext::LinkLabel)
        );
    }

    #[test]
    fn is_scannable_filters_appropriately() {
        let range = sample_range();
        let normal = TextNode::new(
            "a".into(),
            TextStyle::NONE,
            TextContext::Normal,
            range,
        );
        let link = TextNode::new(
            "a".into(),
            TextStyle::NONE,
            TextContext::LinkLabel,
            range,
        );
        let code = TextNode::new(
            "a".into(),
            TextStyle::CODE,
            TextContext::Normal,
            range,
        );
        let math = TextNode::new(
            "a".into(),
            TextStyle::MATH_INLINE,
            TextContext::Normal,
            range,
        );

        assert!(normal.is_scannable());
        assert!(!link.is_scannable());
        assert!(!code.is_scannable());
        assert!(!math.is_scannable());
    }

    #[test]
    fn is_displayable_filters_math_only() {
        let range = sample_range();
        let normal = TextNode::new(
            "a".into(),
            TextStyle::NONE,
            TextContext::Normal,
            range,
        );
        let code = TextNode::new(
            "a".into(),
            TextStyle::CODE,
            TextContext::Normal,
            range,
        );
        let math = TextNode::new(
            "a".into(),
            TextStyle::MATH_INLINE,
            TextContext::Normal,
            range,
        );

        assert!(normal.is_displayable());
        assert!(code.is_displayable());
        assert!(!math.is_displayable());
    }

    #[test]
    fn covering_range_handles_edge_cases() {
        let r1 = SourceByteRange::try_from(1..5).unwrap();
        let r2 = SourceByteRange::try_from(5..10).unwrap();

        let n1 =
            TextNode::new("a".into(), TextStyle::NONE, TextContext::Normal, r1);
        let n2 =
            TextNode::new("b".into(), TextStyle::NONE, TextContext::Normal, r2);

        let empty = TextSequence::new();
        let single = TextSequence::from_nodes(vec![n1.clone()]);
        let multiple = TextSequence::from_nodes(vec![n1, n2]);

        assert_eq!(empty.covering_range(), None);
        assert_eq!(single.covering_range(), Some(r1));
        assert_eq!(
            multiple.covering_range(),
            Some(SourceByteRange::try_from(1..10).unwrap())
        );
    }

    #[test]
    fn from_events_includes_html_and_linebreak() {
        use crate::note::parser::types::LineBreakKind;

        let html_ev = RangedEvent::try_from((
            ParserEvent::Inline(InlineToken::Html("<div>".into())),
            0..5,
        ))
        .unwrap();
        let br_ev = RangedEvent::try_from((
            ParserEvent::Inline(InlineToken::LineBreak(LineBreakKind::Hard)),
            5..7,
        ))
        .unwrap();

        let sequence = TextSequence::from_events(&[html_ev, br_ev]);

        assert_eq!(sequence.nodes().len(), 2);
        assert_eq!(sequence.nodes().first().map(TextNode::text), Some("<div>"));
        assert_eq!(sequence.nodes().get(1).map(TextNode::text), Some(""));
        assert_eq!(
            sequence.covering_range(),
            Some(SourceByteRange::try_from(0..7).unwrap())
        );
    }
}
