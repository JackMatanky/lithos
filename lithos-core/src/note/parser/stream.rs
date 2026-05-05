//! Event stream processing and normalization.
//!
//! This module provides the `MarkdownEventStream` component and maps
//! pulldown events into parser IR (`types::RangedEvent`). The stream acts as a
//! facade adapter between `pulldown-cmark` and the rest of the parsing
//! pipeline, handling event normalization and extracting link reference
//! definitions.

use std::ops::Range;

use pulldown_cmark::{
    CowStr, Event, OffsetIter, Parser, utils::TextMergeWithOffset,
};

use super::{
    config::{BreakPolicy, EventRetentionPolicy, EventStreamConfig},
    references::ReferenceDefinitions,
    types::{
        BlockEnd, BlockStart, FrontmatterFormat, HeadingLevel,
        InlineDelimiterEnd, InlineDelimiterStart, InlineToken, LinkKind,
        ListKind, ParserEvent, RangedEvent,
    },
};
use crate::note::{error::NoteIngestError, position::SourceByteRange};

// ═══════════════════════════════════════════════════════════════════════════
// MARKDOWN EVENT STREAM - ITERATOR OVER EVENTS
// ═══════════════════════════════════════════════════════════════════════════

/// A normalized stream of markdown events.
///
/// This acts as the facade adapter between the raw `pulldown-cmark` library
/// and the internal `Lithos` parsing pipeline. It yields `RangedEvent`
/// structures instead of raw tuples.
///
/// Reference definitions are extracted during construction and returned
/// separately to avoid duplicate storage.
pub(crate) struct MarkdownEventStream<'source> {
    state: StreamState<'source>,
}

impl<'source> MarkdownEventStream<'source> {
    /// Creates a new event stream with the provided configuration.
    ///
    /// Returns a tuple of `(stream, references)` to avoid duplicate storage of
    /// reference definitions. The caller (typically `ParserContext`) owns the
    /// references.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Note: Cannot run doctest for pub(crate) types from external test crate
    /// use lithos_core::note::parser::stream::MarkdownEventStream;
    /// use lithos_core::note::parser::config::EventStreamConfig;
    ///
    /// let source = "Here is some markdown text.";
    /// let config = EventStreamConfig::default();
    /// let (mut stream, references) = MarkdownEventStream::new(source, config);
    ///
    /// assert!(stream.next().is_some());
    /// ```
    #[must_use]
    #[inline]
    pub(crate) fn new(
        source: &'source str,
        config: EventStreamConfig,
    ) -> (Self, ReferenceDefinitions) {
        let parser = Parser::new_ext(source, config.options());

        let raw_refs = parser
            .reference_definitions()
            .iter()
            .map(|(label, def)| {
                (label.to_owned(), def.dest.clone().into_string())
            })
            .collect();
        let references = ReferenceDefinitions::new(raw_refs);

        let offset_iter = parser.into_offset_iter();
        let break_policy_iter = BreakPolicyIter {
            inner: offset_iter,
            policy: config.break_policy(),
        };
        let state = if config.merge_text() {
            let merged = TextMergeWithOffset::new(break_policy_iter);
            StreamState::Merged(EventAdapterIter {
                inner: merged,
                retention: config.retention(),
            })
        } else {
            StreamState::Unmerged(EventAdapterIter {
                inner: break_policy_iter,
                retention: config.retention(),
            })
        };

        (
            Self {
                state,
            },
            references,
        )
    }
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Implementing all iterator methods is unnecessary for this \
              public wrapper"
)]
impl<'source> Iterator for MarkdownEventStream<'source> {
    type Item = Result<RangedEvent<'source>, NoteIngestError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.state.next()
    }
}

/// Inner state of the event stream, supporting merged and unmerged text.
#[non_exhaustive]
enum StreamState<'source> {
    /// Text events are dynamically merged together.
    Merged(EventAdapterIter<'source, MergedOffsetIter<'source>>),
    /// Text events remain separated.
    Unmerged(EventAdapterIter<'source, UnmergedOffsetIter<'source>>),
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Implementing all iterator methods is unnecessary for this \
              internal wrapper"
)]
impl<'source> Iterator for StreamState<'source> {
    type Item = Result<RangedEvent<'source>, NoteIngestError>;

    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Ergonomics for mutable reference match"
    )]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            StreamState::Merged(iter) => iter.next(),
            StreamState::Unmerged(iter) => iter.next(),
        }
    }
}

type MergedOffsetIter<'source> =
    TextMergeWithOffset<'source, UnmergedOffsetIter<'source>>;

type UnmergedOffsetIter<'source> = BreakPolicyIter<
    'source,
    OffsetIter<'source, pulldown_cmark::DefaultBrokenLinkCallback>,
>;

struct BreakPolicyIter<'source, I>
where
    I: Iterator<Item = (Event<'source>, Range<usize>)>,
{
    inner: I,
    policy: BreakPolicy,
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Internal iterator wrapper only requires next()"
)]
impl<'source, I> Iterator for BreakPolicyIter<'source, I>
where
    I: Iterator<Item = (Event<'source>, Range<usize>)>,
{
    type Item = (Event<'source>, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(event, range)| {
            #[expect(
                clippy::wildcard_enum_match_arm,
                reason = "Pass through non-break events without mutation"
            )]
            let normalized = match event {
                Event::SoftBreak => {
                    if let Some(replacement) =
                        self.policy.soft_break_replacement()
                    {
                        Event::Text(CowStr::Borrowed(replacement))
                    } else {
                        Event::SoftBreak
                    }
                }
                Event::HardBreak => {
                    if let Some(replacement) =
                        self.policy.hard_break_replacement()
                    {
                        Event::Text(CowStr::Borrowed(replacement))
                    } else {
                        Event::HardBreak
                    }
                }
                other => other,
            };
            (normalized, range)
        })
    }
}

/// An iterator that adapts and normalizes pulldown-cmark events.
pub(crate) struct EventAdapterIter<'source, I>
where
    I: Iterator<Item = (Event<'source>, Range<usize>)>,
{
    inner: I,
    retention: EventRetentionPolicy,
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Implementing all iterator methods is unnecessary for this \
              internal wrapper"
)]
impl<'source, I> Iterator for EventAdapterIter<'source, I>
where
    I: Iterator<Item = (Event<'source>, Range<usize>)>,
{
    type Item = Result<RangedEvent<'source>, NoteIngestError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        for (event, byte_range) in self.inner.by_ref() {
            let range = match SourceByteRange::try_from(byte_range) {
                Ok(range) => range,
                Err(error) => return Some(Err(NoteIngestError::Domain(error))),
            };

            let parser_event = match ParserEventMapper::try_from_event(
                &event,
                self.retention,
                Some(range),
            ) {
                Ok(Some(parser_event)) => parser_event,
                Ok(None) => continue,
                Err(error) => return Some(Err(error)),
            };

            return Some(Ok(RangedEvent::new(parser_event, range)));
        }
        None
    }
}

#[inline]
fn cow_to_cow<'source>(s: &CowStr<'source>) -> std::borrow::Cow<'source, str> {
    match s {
        CowStr::Borrowed(s) => std::borrow::Cow::Borrowed(s),
        CowStr::Boxed(s) => std::borrow::Cow::Owned(s.to_string()),
        CowStr::Inlined(s) => std::borrow::Cow::Owned(s.to_string()),
    }
}

/// Mapper that converts `pulldown-cmark` events into neutral parser IR.
///
/// This component enforces the `EventRetentionPolicy` and handles low-level
/// normalization, such as stripping delimiters from code and math tokens.
#[derive(Debug, Clone, Copy)]
struct ParserEventMapper;

impl ParserEventMapper {
    /// Maps a raw pulldown event to a parser event.
    ///
    /// Returns `Ok(Some(event))` if the event should be preserved, `Ok(None)`
    /// if it should be dropped according to policy, or `Err` if it violates
    /// a rejection policy.
    ///
    /// # Normalization Contract
    ///
    /// - **Code/Math**: Delimiters (backticks, `$`) are stripped. Only the
    ///   content is preserved in the token.
    /// - **Line Breaks**: Soft and hard breaks are normalized according to the
    ///   configured `BreakPolicy` before reaching this mapper.
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Pattern matches borrowed parser events by design"
    )]
    fn try_from_event<'source>(
        event: &Event<'source>,
        retention: EventRetentionPolicy,
        range: Option<SourceByteRange>,
    ) -> Result<Option<ParserEvent<'source>>, NoteIngestError> {
        match event {
            Event::Start(tag) => {
                Self::try_from_start_tag(tag, retention, range)
            }
            Event::End(tag_end) => {
                Self::try_from_end_tag(*tag_end, retention, range)
            }
            Event::Text(text) => Ok(Some(ParserEvent::Inline(
                InlineToken::Text(cow_to_cow(text)),
            ))),
            Event::Code(code) => Ok(Some(ParserEvent::Inline(
                InlineToken::InlineCode(cow_to_cow(code)),
            ))),
            Event::InlineHtml(html) | Event::Html(html) => Ok(Some(
                ParserEvent::Inline(InlineToken::Html(cow_to_cow(html))),
            )),
            Event::Rule => Ok(Some(ParserEvent::ThematicBreak)),
            Event::SoftBreak | Event::HardBreak => {
                retention.enforce_unknown_inline("line_break", range)?;
                Ok(None)
            }
            Event::FootnoteReference(reference) => {
                Ok(Some(ParserEvent::Inline(InlineToken::FootnoteReference(
                    cow_to_cow(reference),
                ))))
            }
            Event::InlineMath(content) => {
                Ok(Some(ParserEvent::Inline(InlineToken::Math {
                    kind: super::types::MathKind::Inline,
                    content: cow_to_cow(content),
                })))
            }
            Event::DisplayMath(content) => {
                Ok(Some(ParserEvent::Inline(InlineToken::Math {
                    kind: super::types::MathKind::Display,
                    content: cow_to_cow(content),
                })))
            }
            Event::TaskListMarker(checked) => {
                Ok(Some(ParserEvent::TaskListMarker(*checked)))
            }
        }
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Pattern matches borrowed pulldown tags by design"
    )]
    fn try_from_start_tag<'source>(
        tag: &pulldown_cmark::Tag<'source>,
        retention: EventRetentionPolicy,
        range: Option<SourceByteRange>,
    ) -> Result<Option<ParserEvent<'source>>, NoteIngestError> {
        if let Ok(block_start) = BlockStart::try_from(tag) {
            return Ok(Some(ParserEvent::BlockStart(block_start)));
        }
        if let Ok(inline_start) = InlineDelimiterStart::try_from(tag) {
            return Ok(Some(ParserEvent::Inline(InlineToken::DelimiterStart(
                inline_start,
            ))));
        }

        if let pulldown_cmark::Tag::HtmlBlock = tag {
            retention.enforce_unknown_block("start_tag_extension", range)?;
        }
        Ok(None)
    }

    fn try_from_end_tag<'source>(
        tag_end: pulldown_cmark::TagEnd,
        retention: EventRetentionPolicy,
        range: Option<SourceByteRange>,
    ) -> Result<Option<ParserEvent<'source>>, NoteIngestError> {
        if let Ok(block_end) = BlockEnd::try_from(&tag_end) {
            return Ok(Some(ParserEvent::BlockEnd(block_end)));
        }
        if let Ok(inline_end) = InlineDelimiterEnd::try_from(&tag_end) {
            return Ok(Some(ParserEvent::Inline(InlineToken::DelimiterEnd(
                inline_end,
            ))));
        }

        if let pulldown_cmark::TagEnd::HtmlBlock = tag_end {
            retention.enforce_unknown_block("end_tag_extension", range)?;
        }
        Ok(None)
    }
}

impl<'source> TryFrom<&pulldown_cmark::Tag<'source>> for BlockStart<'source> {
    type Error = ();

    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "Delegates unsupported tags to caller"
    )]
    fn try_from(
        tag: &pulldown_cmark::Tag<'source>,
    ) -> Result<Self, Self::Error> {
        match tag {
            pulldown_cmark::Tag::Paragraph => Ok(Self::Paragraph),
            pulldown_cmark::Tag::Heading {
                level,
                ..
            } => Ok(Self::Heading {
                level: (*level).into(),
            }),
            pulldown_cmark::Tag::BlockQuote(_) => Ok(Self::BlockQuote),
            pulldown_cmark::Tag::List(start) => Ok(Self::List {
                kind: (*start).into(),
            }),
            pulldown_cmark::Tag::Item => Ok(Self::ListItem),
            pulldown_cmark::Tag::CodeBlock(code_kind) => {
                let language = match code_kind {
                    pulldown_cmark::CodeBlockKind::Indented => None,
                    pulldown_cmark::CodeBlockKind::Fenced(info) => {
                        if info.is_empty() {
                            None
                        } else {
                            Some(info.clone())
                        }
                    }
                };
                Ok(Self::CodeBlock {
                    info_string: language.as_ref().map(cow_to_cow),
                })
            }
            pulldown_cmark::Tag::MetadataBlock(format) => {
                Ok(Self::Frontmatter {
                    format: (*format).into(),
                })
            }
            _ => Err(()),
        }
    }
}

impl TryFrom<&pulldown_cmark::TagEnd> for BlockEnd {
    type Error = ();

    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "Delegates unsupported tags to caller"
    )]
    fn try_from(tag_end: &pulldown_cmark::TagEnd) -> Result<Self, Self::Error> {
        match tag_end {
            pulldown_cmark::TagEnd::Paragraph => Ok(Self::Paragraph),
            pulldown_cmark::TagEnd::Heading(_) => Ok(Self::Heading),
            pulldown_cmark::TagEnd::BlockQuote(_) => Ok(Self::BlockQuote),
            pulldown_cmark::TagEnd::List(_) => Ok(Self::List),
            pulldown_cmark::TagEnd::Item => Ok(Self::ListItem),
            pulldown_cmark::TagEnd::CodeBlock => Ok(Self::CodeBlock),
            pulldown_cmark::TagEnd::MetadataBlock(_) => Ok(Self::Frontmatter),
            _ => Err(()),
        }
    }
}

impl<'source> TryFrom<&pulldown_cmark::Tag<'source>>
    for InlineDelimiterStart<'source>
{
    type Error = ();

    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "Delegates unsupported tags to caller"
    )]
    fn try_from(
        tag: &pulldown_cmark::Tag<'source>,
    ) -> Result<Self, Self::Error> {
        match tag {
            pulldown_cmark::Tag::Emphasis => Ok(Self::Emphasis),
            pulldown_cmark::Tag::Strong => Ok(Self::Strong),
            pulldown_cmark::Tag::Strikethrough => Ok(Self::Strikethrough),
            pulldown_cmark::Tag::Superscript => Ok(Self::Superscript),
            pulldown_cmark::Tag::Subscript => Ok(Self::Subscript),
            pulldown_cmark::Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            } => Ok(Self::Link {
                kind: (*link_type).into(),
                destination: cow_to_cow(dest_url),
                title: cow_to_cow(title),
                label: cow_to_cow(id),
            }),
            pulldown_cmark::Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            } => Ok(Self::Image {
                kind: (*link_type).into(),
                destination: cow_to_cow(dest_url),
                title: cow_to_cow(title),
                label: cow_to_cow(id),
            }),
            _ => Err(()),
        }
    }
}

impl TryFrom<&pulldown_cmark::TagEnd> for InlineDelimiterEnd {
    type Error = ();

    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "Delegates unsupported tags to caller"
    )]
    fn try_from(tag_end: &pulldown_cmark::TagEnd) -> Result<Self, Self::Error> {
        match tag_end {
            pulldown_cmark::TagEnd::Emphasis => Ok(Self::Emphasis),
            pulldown_cmark::TagEnd::Strong => Ok(Self::Strong),
            pulldown_cmark::TagEnd::Strikethrough => Ok(Self::Strikethrough),
            pulldown_cmark::TagEnd::Superscript => Ok(Self::Superscript),
            pulldown_cmark::TagEnd::Subscript => Ok(Self::Subscript),
            pulldown_cmark::TagEnd::Link => Ok(Self::Link),
            pulldown_cmark::TagEnd::Image => Ok(Self::Image),
            _ => Err(()),
        }
    }
}

impl From<pulldown_cmark::HeadingLevel> for HeadingLevel {
    fn from(value: pulldown_cmark::HeadingLevel) -> Self {
        match value {
            pulldown_cmark::HeadingLevel::H1 => Self::H1,
            pulldown_cmark::HeadingLevel::H2 => Self::H2,
            pulldown_cmark::HeadingLevel::H3 => Self::H3,
            pulldown_cmark::HeadingLevel::H4 => Self::H4,
            pulldown_cmark::HeadingLevel::H5 => Self::H5,
            pulldown_cmark::HeadingLevel::H6 => Self::H6,
        }
    }
}

impl From<Option<u64>> for ListKind {
    fn from(value: Option<u64>) -> Self {
        value.map_or(Self::Unordered, Self::Ordered)
    }
}

impl From<pulldown_cmark::MetadataBlockKind> for FrontmatterFormat {
    fn from(value: pulldown_cmark::MetadataBlockKind) -> Self {
        match value {
            pulldown_cmark::MetadataBlockKind::YamlStyle => Self::Yaml,
            pulldown_cmark::MetadataBlockKind::PlusesStyle => Self::Toml,
        }
    }
}

impl From<pulldown_cmark::LinkType> for LinkKind {
    fn from(value: pulldown_cmark::LinkType) -> Self {
        match value {
            pulldown_cmark::LinkType::Inline => Self::Inline,
            pulldown_cmark::LinkType::Reference => Self::Reference,
            pulldown_cmark::LinkType::ReferenceUnknown => {
                Self::ReferenceUnknown
            }
            pulldown_cmark::LinkType::Collapsed => Self::Collapsed,
            pulldown_cmark::LinkType::CollapsedUnknown => {
                Self::CollapsedUnknown
            }
            pulldown_cmark::LinkType::Shortcut => Self::Shortcut,
            pulldown_cmark::LinkType::ShortcutUnknown => Self::ShortcutUnknown,
            pulldown_cmark::LinkType::Autolink => Self::Autolink,
            pulldown_cmark::LinkType::Email => Self::Email,
            pulldown_cmark::LinkType::WikiLink {
                has_pothole,
            } => Self::WikiLink {
                has_pothole,
            },
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module keeps imports and nested suites grouped for \
              readability"
)]
mod tests {
    use pulldown_cmark::Options;

    use super::*;
    use crate::note::position::SourceByteOffset;

    mod ranged_event {
        use pulldown_cmark::CowStr;

        use super::*;

        #[test]
        fn new_preserves_text_event_payload() {
            let range = SourceByteRange::new(
                SourceByteOffset::new(0),
                SourceByteOffset::new(5),
            )
            .expect("test range should be valid");

            let event = RangedEvent::new(
                ParserEvent::Inline(InlineToken::Text(
                    CowStr::Borrowed("hello").into(),
                )),
                range,
            );

            assert!(
                matches!(event.event(), ParserEvent::Inline(InlineToken::Text(text)) if text.as_ref() == "hello"),
                "event accessor should preserve text payload"
            );
        }

        #[test]
        fn new_preserves_source_range() {
            let range = SourceByteRange::new(
                SourceByteOffset::new(10),
                SourceByteOffset::new(17),
            )
            .expect("test range should be valid");

            let event = RangedEvent::new(ParserEvent::ThematicBreak, range);

            assert_eq!(
                event.range(),
                range,
                "range accessor should preserve original source range"
            );
        }

        #[test]
        fn try_from_converts_valid_tuple() {
            let event = ParserEvent::Inline(InlineToken::Text(
                CowStr::Borrowed("test").into(),
            ));
            let byte_range = 0..4;

            let result = RangedEvent::try_from((event, byte_range));

            assert!(
                result.is_ok(),
                "TryFrom should succeed for valid byte range"
            );
            let event_with_range = result.unwrap();
            assert_eq!(event_with_range.range().start().as_usize(), 0);
            assert_eq!(event_with_range.range().end().as_usize(), 4);
        }

        #[test]
        fn try_from_rejects_invalid_range() {
            let event = ParserEvent::Inline(InlineToken::Text(
                CowStr::Borrowed("test").into(),
            ));
            // Explicitly construct invalid range to test error handling
            #[expect(
                clippy::reversed_empty_ranges,
                reason = "Testing error handling for invalid ranges"
            )]
            let invalid_range = 10..5; // end before start

            let result = RangedEvent::try_from((event, invalid_range));

            assert!(
                result.is_err(),
                "TryFrom should fail for invalid byte range"
            );
        }
    }

    mod markdown_event_stream_references {
        use super::*;

        #[test]
        fn extracts_reference_definitions_from_source() {
            let source = "[foo]: /url\n\n[foo][]";
            let (_stream, references) =
                MarkdownEventStream::new(source, EventStreamConfig::default());

            assert_eq!(
                references.resolve("foo"),
                Some("/url"),
                "stream should expose normalized reference definitions"
            );
        }
    }

    mod markdown_event_stream_merging {
        use super::*;

        #[test]
        fn merge_text_true_combines_soft_break_text_fragments() {
            let events = collect_events(
                "a\nb",
                test_config(BreakPolicy::NormalizeAsText, true),
            );

            let texts = collect_text_payloads(&events);

            assert_eq!(
                texts,
                vec!["a b".to_owned()],
                "merged stream should coalesce normalized soft breaks into \
                 one text event"
            );
        }

        #[test]
        fn merge_text_false_keeps_soft_break_text_fragments_separate() {
            let events = collect_events(
                "a\nb",
                test_config(BreakPolicy::NormalizeAsText, false),
            );

            let texts = collect_text_payloads(&events);

            assert_eq!(
                texts,
                vec!["a".to_owned(), " ".to_owned(), "b".to_owned()],
                "unmerged stream should preserve separate text fragments"
            );
        }
    }

    mod event_adapter_iter_break_policy {
        use super::*;

        #[test]
        fn preserve_policy_drops_hard_break_events_from_ir() {
            let events = collect_events(
                "a\\\nb",
                test_config(BreakPolicy::Preserve, false),
            );

            assert!(
                events.iter().all(|event| !matches!(
                    event.event(),
                    ParserEvent::Inline(InlineToken::Text(text))
                        if text.as_ref() == "\n"
                )),
                "preserve policy should not emit hard-break newline text in IR"
            );
        }

        #[test]
        fn hard_as_newline_policy_rewrites_hard_break_to_text() {
            let events = collect_events(
                "a\\\nb",
                test_config(BreakPolicy::HardAsNewLine, false),
            );

            assert!(
                events.iter().any(|event| {
                    matches!(event.event(), ParserEvent::Inline(InlineToken::Text(text)) if text.as_ref() == "\n")
                }),
                "hard-as-newline policy should map hard breaks to newline text events"
            );
        }

        #[test]
        fn soft_as_space_policy_drops_hard_break_events_from_ir() {
            let events = collect_events(
                "a\\\nb",
                test_config(BreakPolicy::SoftAsSpace, false),
            );

            assert!(
                events.iter().all(|event| !matches!(
                    event.event(),
                    ParserEvent::Inline(InlineToken::Text(text))
                        if text.as_ref() == "\n"
                )),
                "soft-as-space policy should not emit hard-break newline text"
            );
        }
    }

    mod event_adapter_iter_math_contract {
        use super::*;
        use crate::note::parser::types::MathKind;

        #[test]
        fn inline_math_event_is_emitted_as_math_token() {
            let events =
                collect_events("a $x+y$ b", EventStreamConfig::default());

            assert!(
                events.iter().any(|event| {
                    matches!(
                        event.event(),
                        ParserEvent::Inline(InlineToken::Math { kind: MathKind::Inline, content }) if content.as_ref() == "x+y"
                    )
                }),
                "inline math payload should be preserved as typed IR token"
            );
        }

        #[test]
        fn display_math_event_is_emitted_as_math_token() {
            let events = collect_events(
                "$$\\na^2 + b^2\\n$$",
                EventStreamConfig::default(),
            );

            assert!(
                events.iter().any(|event| {
                    matches!(
                        event.event(),
                        ParserEvent::Inline(InlineToken::Math { kind: MathKind::Display, content })
                            if content.as_ref().contains("a^2 + b^2")
                    )
                }),
                "display math payload should be preserved as typed IR token"
            );
        }

        #[test]
        fn inline_code_payload_is_content_only_without_backticks() {
            let events =
                collect_events("pre `x+y` post", EventStreamConfig::default());

            assert!(
                events.iter().any(|event| {
                    matches!(
                        event.event(),
                        ParserEvent::Inline(InlineToken::InlineCode(content))
                            if content.as_ref() == "x+y"
                    )
                }),
                "inline code payload should be delimiter-stripped content"
            );
            assert!(
                events.iter().all(|event| {
                    !matches!(
                        event.event(),
                        ParserEvent::Inline(InlineToken::InlineCode(content))
                            if content.as_ref().contains('`')
                    )
                }),
                "inline code payload must not contain backtick delimiters"
            );
        }

        #[test]
        fn merge_text_does_not_collapse_math_into_text() {
            let events =
                collect_events("a $x+y$ b", EventStreamConfig::default());

            assert!(
                events.iter().any(|event| {
                    matches!(
                        event.event(),
                        ParserEvent::Inline(InlineToken::Math { kind: MathKind::Inline, content })
                            if content.as_ref() == "x+y"
                    )
                }),
                "math token should remain explicit under merge_text=true"
            );

            assert!(
                events.iter().all(|event| {
                    !matches!(
                        event.event(),
                        ParserEvent::Inline(InlineToken::Text(text))
                            if text.as_ref().contains("x+y")
                    )
                }),
                "text merging should not absorb math payload into plain text"
            );
        }
    }

    mod event_adapter_iter_extension_support {
        use super::*;

        #[test]
        fn footnote_events_are_emitted_as_dedicated_ir_tokens() {
            let events = collect_events(
                "[^n]\n\n[^n]: Footnote body",
                EventStreamConfig::new(
                    Options::ENABLE_FOOTNOTES,
                    BreakPolicy::NormalizeAsText,
                    true,
                ),
            );

            assert!(
                events.iter().any(|event| {
                    matches!(
                        event.event(),
                        ParserEvent::Inline(InlineToken::FootnoteReference(label))
                            if label.as_ref() == "n"
                    )
                }),
                "footnote reference should be emitted as dedicated IR token"
            );
        }

        #[test]
        fn table_content_events_are_emitted_as_text() {
            let events = collect_events(
                "| h |\n| - |\n| c |",
                EventStreamConfig::new(
                    Options::ENABLE_TABLES,
                    BreakPolicy::NormalizeAsText,
                    true,
                ),
            );

            assert!(
                events.iter().any(|event| {
                    matches!(
                        event.event(),
                        ParserEvent::Inline(InlineToken::Text(text)) if text.contains('h')
                    )
                }),
                "table content should be emitted as text"
            );
        }

        #[test]
        fn reject_policy_returns_error_for_unknown_block_extensions() {
            let config = EventStreamConfig::with_policy(
                crate::note::parser::config::ExtensionsPolicy::new(
                    crate::note::parser::config::ExtensionFlags::EMPTY,
                    crate::note::parser::config::MetadataPolicy::None,
                ),
                crate::note::parser::config::EventRetentionPolicy::new(
                    crate::note::parser::config::UnknownEventPolicy::Reject,
                    crate::note::parser::config::UnknownEventPolicy::Drop,
                ),
                BreakPolicy::NormalizeAsText,
                true,
            );

            // Using HTML block which is still unmapped/unknown
            let (stream, _references) =
                MarkdownEventStream::new("<div></div>", config);
            let result = stream.collect::<Result<Vec<_>, _>>();

            assert!(
                result.is_err(),
                "reject policy should fail on unknown block events (like HTML \
                 blocks)"
            );
        }

        #[test]
        fn policy_fail_on_unsupported() {
            use crate::note::parser::config::{
                BreakPolicy, EventRetentionPolicy, EventStreamConfig,
                ExtensionFlags, ExtensionsPolicy, MetadataPolicy,
                UnknownEventPolicy,
            };

            let config = EventStreamConfig::with_policy(
                ExtensionsPolicy::new(
                    ExtensionFlags::EMPTY,
                    MetadataPolicy::None,
                ),
                EventRetentionPolicy::new(
                    UnknownEventPolicy::Reject,
                    UnknownEventPolicy::Reject,
                ),
                BreakPolicy::Preserve, /* Ensure SoftBreak is treated as
                                        * unknown */
                true,
            );

            let source = "a\nb";
            let (stream, _) = MarkdownEventStream::new(source, config);
            let result = stream.collect::<Result<Vec<_>, _>>();

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err,
                NoteIngestError::Domain(crate::note::error::NoteError::Parse(
                    crate::note::error::NoteParseError::PolicyViolation {
                        policy: "unknown_inline",
                        observed: "line_break",
                        ..
                    }
                ))
            ));
        }

        #[test]
        fn policy_strip_unsupported() {
            use crate::note::parser::config::{
                BreakPolicy, EventRetentionPolicy, EventStreamConfig,
                ExtensionFlags, ExtensionsPolicy, MetadataPolicy,
                UnknownEventPolicy,
            };

            let config = EventStreamConfig::with_policy(
                ExtensionsPolicy::new(
                    ExtensionFlags::EMPTY,
                    MetadataPolicy::None,
                ),
                EventRetentionPolicy::new(
                    UnknownEventPolicy::Drop,
                    UnknownEventPolicy::Drop,
                ),
                BreakPolicy::Preserve, /* Ensure SoftBreak is treated as
                                        * unknown */
                true,
            );

            let source = "a\nb";
            let (stream, _) = MarkdownEventStream::new(source, config);
            let result = stream.collect::<Result<Vec<_>, _>>();

            assert!(result.is_ok());
            let events = result.unwrap();

            // Should contain "a" and "b" but NO line_break events
            for event in events {
                assert!(!matches!(
                    event.event(),
                    ParserEvent::Inline(InlineToken::LineBreak(_))
                ));
            }
        }
    }

    mod markdown_event_stream_span_mapping {
        use super::*;

        #[test]
        fn maps_all_events_to_valid_source_spans() {
            let events = collect_events(
                "# heading\n\ntext",
                EventStreamConfig::default(),
            );

            assert!(
                events
                    .iter()
                    .all(|event| event.range().start() <= event.range().end()),
                "all emitted events should map to valid source ranges"
            );
        }
    }

    fn test_config(
        break_policy: BreakPolicy,
        merge_text: bool,
    ) -> EventStreamConfig {
        EventStreamConfig::new(Options::empty(), break_policy, merge_text)
    }

    fn collect_events(
        source: &str,
        config: EventStreamConfig,
    ) -> Vec<RangedEvent<'_>> {
        let (stream, _references) = MarkdownEventStream::new(source, config);
        stream
            .collect::<Result<Vec<_>, _>>()
            .expect("event stream should not produce invalid spans")
    }

    fn collect_text_payloads(events: &[RangedEvent<'_>]) -> Vec<String> {
        events
            .iter()
            .filter_map(|event| {
                #[expect(
                    clippy::pattern_type_mismatch,
                    reason = "Matching borrowed enum payload inside iterator \
                              adapter"
                )]
                if let ParserEvent::Inline(InlineToken::Text(text)) =
                    event.event()
                {
                    Some(text.to_string())
                } else {
                    None
                }
            })
            .collect()
    }
}
