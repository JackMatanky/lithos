//! Derived inline text projection types.
//!
//! This module projects parser inline IR (`RangedEvent`) into stable text
//! nodes. It is policy-agnostic: consumer layers (scanner, assembler, link
//! handling) decide which nodes to include for their own use cases.

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
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum TextContext {
    Normal,
    LinkLabel,
    ImageAlt,
}

/// Derived style marker for text nodes.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum TextStyle {
    Emphasis,
    Strong,
    Strikethrough,
    Code,
    MathInline,
    MathDisplay,
}

/// Derived text node with style stack and source span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextNode {
    text: Box<str>,
    styles: Vec<TextStyle>,
    context: TextContext,
    range: SourceByteRange,
}

impl TextNode {
    #[must_use]
    #[inline]
    pub(crate) fn new(
        text: Box<str>,
        styles: Vec<TextStyle>,
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
    pub(crate) fn styles(&self) -> &[TextStyle] {
        &self.styles
    }

    #[must_use]
    #[inline]
    pub(crate) const fn context(&self) -> TextContext {
        self.context
    }

    #[must_use]
    #[inline]
    pub(crate) const fn range(&self) -> SourceByteRange {
        self.range
    }
}

/// Ordered collection of derived text nodes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
    pub(crate) fn from_nodes(nodes: Vec<TextNode>) -> Self {
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

    /// Returns source-covering span of first..last node.
    #[must_use]
    pub(crate) fn covering_range(&self) -> Option<SourceByteRange> {
        let start = self.nodes.first().map(|node| node.range().start())?;
        let end = self.nodes.last().map_or(start, |node| node.range().end());
        SourceByteRange::new(start, end).ok()
    }

    /// Projects parser events into text nodes without consumer policy.
    #[must_use]
    pub(crate) fn from_events(events: &[RangedEvent<'_>]) -> Self {
        let mut nodes = Vec::new();
        let mut styles = Vec::new();
        let mut link_depth = 0u32;
        let mut image_depth = 0u32;

        for event in events {
            match event.event() {
                ParserEvent::Inline(InlineToken::DelimiterStart(start)) => {
                    match start {
                        InlineDelimiterStart::Emphasis => {
                            styles.push(TextStyle::Emphasis);
                        }
                        InlineDelimiterStart::Strong => {
                            styles.push(TextStyle::Strong);
                        }
                        InlineDelimiterStart::Strikethrough => {
                            styles.push(TextStyle::Strikethrough);
                        }
                        InlineDelimiterStart::Link {
                            ..
                        } => {
                            link_depth = link_depth.saturating_add(1);
                        }
                        InlineDelimiterStart::Image {
                            ..
                        } => {
                            image_depth = image_depth.saturating_add(1);
                        }
                        InlineDelimiterStart::Superscript
                        | InlineDelimiterStart::Subscript
                        | InlineDelimiterStart::_Marker(_) => {}
                    }
                }
                ParserEvent::Inline(InlineToken::DelimiterEnd(end)) => {
                    match end {
                        InlineDelimiterEnd::Emphasis => {
                            remove_style(&mut styles, TextStyle::Emphasis);
                        }
                        InlineDelimiterEnd::Strong => {
                            remove_style(&mut styles, TextStyle::Strong);
                        }
                        InlineDelimiterEnd::Strikethrough => {
                            remove_style(&mut styles, TextStyle::Strikethrough);
                        }
                        InlineDelimiterEnd::Link => {
                            link_depth = link_depth.saturating_sub(1);
                        }
                        InlineDelimiterEnd::Image => {
                            image_depth = image_depth.saturating_sub(1);
                        }
                        InlineDelimiterEnd::Superscript
                        | InlineDelimiterEnd::Subscript => {}
                    }
                }
                ParserEvent::Inline(InlineToken::Text(text)) => {
                    nodes.push(TextNode::new(
                        text.to_string().into_boxed_str(),
                        styles.clone(),
                        context_for_depth(link_depth, image_depth),
                        event.range(),
                    ));
                }
                ParserEvent::Inline(InlineToken::InlineCode(text)) => {
                    let mut node_styles = styles.clone();
                    node_styles.push(TextStyle::Code);
                    nodes.push(TextNode::new(
                        text.to_string().into_boxed_str(),
                        node_styles,
                        context_for_depth(link_depth, image_depth),
                        event.range(),
                    ));
                }
                ParserEvent::Inline(InlineToken::Math {
                    kind,
                    content,
                }) => {
                    let mut node_styles = styles.clone();
                    node_styles.push(match kind {
                        MathKind::Inline => TextStyle::MathInline,
                        MathKind::Display => TextStyle::MathDisplay,
                    });
                    nodes.push(TextNode::new(
                        content.to_string().into_boxed_str(),
                        node_styles,
                        context_for_depth(link_depth, image_depth),
                        event.range(),
                    ));
                }
                ParserEvent::Inline(
                    InlineToken::Html(_) | InlineToken::LineBreak(_),
                )
                | ParserEvent::BlockStart(_)
                | ParserEvent::BlockEnd(_)
                | ParserEvent::TaskListMarker(_)
                | ParserEvent::ThematicBreak => {}
            }
        }

        Self::from_nodes(nodes)
    }
}

#[must_use]
const fn context_for_depth(link_depth: u32, image_depth: u32) -> TextContext {
    if image_depth > 0 {
        TextContext::ImageAlt
    } else if link_depth > 0 {
        TextContext::LinkLabel
    } else {
        TextContext::Normal
    }
}

fn remove_style(styles: &mut Vec<TextStyle>, needle: TextStyle) {
    if let Some(position) = styles.iter().rposition(|style| *style == needle) {
        styles.remove(position);
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
        let node = TextNode::new(
            "hello".into(),
            vec![TextStyle::Strong, TextStyle::Code],
            TextContext::LinkLabel,
            range,
        );

        assert_eq!(node.text(), "hello");
        assert_eq!(node.styles(), [TextStyle::Strong, TextStyle::Code]);
        assert_eq!(node.context(), TextContext::LinkLabel);
        assert_eq!(node.range(), range);
    }

    #[test]
    fn text_sequence_push_preserves_order() {
        let mut sequence = TextSequence::new();
        let first = TextNode::new(
            "a".into(),
            vec![TextStyle::Emphasis],
            TextContext::Normal,
            sample_range(),
        );
        let second = TextNode::new(
            "b".into(),
            vec![TextStyle::Strong],
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
}
