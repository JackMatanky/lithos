//! Event stream processing and normalization.
//!
//! This module provides the `MarkdownEventStream` component and the
//! `EventWithRange` type. The stream acts as a facade adapter between
//! `pulldown-cmark` and the rest of the parsing pipeline, handling event
//! normalization and extracting link reference definitions.

use std::ops::Range;

use pulldown_cmark::{
    CowStr, Event, OffsetIter, Parser, utils::TextMergeWithOffset,
};

use super::{
    config::{BreakPolicy, EventStreamConfig},
    references::ReferenceDefinitions,
};
use crate::note::{error::NoteIngestError, position::SourceByteRange};

/// Parser-facing structural and inline event IR.
///
/// This enum is the boundary between `pulldown-cmark` and Lithos parser
/// components. Downstream parser components consume this representation instead
/// of raw `pulldown_cmark::Event` values.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ParserEvent<'source> {
    /// Start of a block-level element.
    BlockStart(BlockType<'source>),
    /// End of a block-level element.
    BlockEnd(BlockType<'source>),
    /// Inline-level content.
    Inline(InlineEvent<'source>),
    /// Task list marker associated with current list item.
    TaskListMarker(bool),
    /// Standalone thematic break block.
    ThematicBreak,
}

/// Inline payload type used by [`ParserEvent`].
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InlineEvent<'source> {
    Start(InlineTag<'source>),
    End(InlineTagEnd),
    Text(CowStr<'source>),
    CodeSpan(CowStr<'source>),
    Html(CowStr<'source>),
}

/// Inline boundary start tag used by [`InlineEvent`].
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InlineTag<'source> {
    Emphasis,
    Strong,
    Strikethrough,
    Link {
        link_type: pulldown_cmark::LinkType,
        dest_url: CowStr<'source>,
        title: CowStr<'source>,
        id: CowStr<'source>,
    },
    Image {
        link_type: pulldown_cmark::LinkType,
        dest_url: CowStr<'source>,
        title: CowStr<'source>,
        id: CowStr<'source>,
    },
    Superscript,
    Subscript,
    _Marker(std::marker::PhantomData<&'source str>),
}

/// Inline boundary end tag used by [`InlineEvent`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InlineTagEnd {
    Emphasis,
    Strong,
    Strikethrough,
    Link,
    Image,
    Superscript,
    Subscript,
}

/// Block boundary type used by [`ParserEvent`].
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BlockType<'source> {
    Frontmatter {
        format: pulldown_cmark::MetadataBlockKind,
    },
    Heading {
        level: pulldown_cmark::HeadingLevel,
    },
    Paragraph,
    BlockQuote,
    CodeBlock {
        language: Option<CowStr<'source>>,
    },
    List {
        start: Option<u64>,
    },
    Item,
}

impl<'source> BlockType<'source> {
    /// Converts block IR into a pulldown-cmark start tag.
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Pattern matching borrowed block variants"
    )]
    pub(crate) fn as_start_tag(&self) -> pulldown_cmark::Tag<'source> {
        match self {
            Self::Frontmatter {
                format,
            } => pulldown_cmark::Tag::MetadataBlock(*format),
            Self::Heading {
                level,
            } => pulldown_cmark::Tag::Heading {
                level: *level,
                id: None,
                classes: Vec::new(),
                attrs: Vec::new(),
            },
            Self::Paragraph => pulldown_cmark::Tag::Paragraph,
            Self::BlockQuote => pulldown_cmark::Tag::BlockQuote(None),
            Self::CodeBlock {
                language,
            } => language.as_ref().map_or(
                pulldown_cmark::Tag::CodeBlock(
                    pulldown_cmark::CodeBlockKind::Indented,
                ),
                |info| {
                    pulldown_cmark::Tag::CodeBlock(
                        pulldown_cmark::CodeBlockKind::Fenced(info.clone()),
                    )
                },
            ),
            Self::List {
                start,
            } => pulldown_cmark::Tag::List(*start),
            Self::Item => pulldown_cmark::Tag::Item,
        }
    }

    /// Converts block IR into a pulldown-cmark end tag.
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Pattern matching borrowed block variants"
    )]
    pub(crate) fn as_end_tag(&self) -> pulldown_cmark::TagEnd {
        match self {
            Self::Frontmatter {
                format,
            } => pulldown_cmark::TagEnd::MetadataBlock(*format),
            Self::Heading {
                level,
            } => pulldown_cmark::TagEnd::Heading(*level),
            Self::Paragraph => pulldown_cmark::TagEnd::Paragraph,
            Self::BlockQuote => pulldown_cmark::TagEnd::BlockQuote(None),
            Self::CodeBlock {
                ..
            } => pulldown_cmark::TagEnd::CodeBlock,
            Self::List {
                ..
            } => pulldown_cmark::TagEnd::List(false),
            Self::Item => pulldown_cmark::TagEnd::Item,
        }
    }
}

// ParserEvent mapping is owned by EventAdapterIter methods.

// ═══════════════════════════════════════════════════════════════════════════
// EVENT WITH RANGE - MARKDOWN EVENT + SOURCE LOCATION
// ═══════════════════════════════════════════════════════════════════════════

/// A markdown event paired with its original source byte range.
///
/// This struct guarantees that every parsed token is strictly bound to its
/// exact origin within the unparsed source document. This allows downstream
/// scanners and compilers to report diagnostics that accurately highlight the
/// original text rather than a normalized projection.
///
/// # Examples
///
/// ```rust,ignore
/// use lithos_core::note::position::{SourceByteRange, SourceByteOffset};
/// use lithos_core::note::parser::stream::EventWithRange;
/// use lithos_core::note::parser::stream::{InlineEvent, ParserEvent};
/// use pulldown_cmark::CowStr;
///
/// let start = SourceByteOffset::new(0);
/// let end = SourceByteOffset::new(5);
/// let range = SourceByteRange::new(start, end).unwrap();
///
/// let event = EventWithRange::new(
///     ParserEvent::Inline(InlineEvent::Text(CowStr::Borrowed("hello"))),
///     range,
/// );
///
/// assert_eq!(event.range().len(), 5);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EventWithRange<'source> {
    event: ParserEvent<'source>,
    range: SourceByteRange,
}

impl<'source> EventWithRange<'source> {
    /// Creates a new event with its source range.
    #[must_use]
    #[inline]
    pub(crate) const fn new(
        event: ParserEvent<'source>,
        range: SourceByteRange,
    ) -> Self {
        Self {
            event,
            range,
        }
    }

    /// Returns a reference to the underlying parser event.
    #[must_use]
    #[inline]
    pub(crate) const fn event(&self) -> &ParserEvent<'source> {
        &self.event
    }

    /// Returns the source byte range for this event.
    #[must_use]
    #[inline]
    pub(crate) const fn range(&self) -> SourceByteRange {
        self.range
    }
}

impl<'source> TryFrom<(ParserEvent<'source>, Range<usize>)>
    for EventWithRange<'source>
{
    type Error = NoteIngestError;

    fn try_from(
        (event, byte_range): (ParserEvent<'source>, Range<usize>),
    ) -> Result<Self, Self::Error> {
        let range = SourceByteRange::try_from(byte_range)
            .map_err(NoteIngestError::Domain)?;
        Ok(Self::new(event, range))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MARKDOWN EVENT STREAM - ITERATOR OVER EVENTS
// ═══════════════════════════════════════════════════════════════════════════

/// A normalized stream of markdown events.
///
/// This acts as the facade adapter between the raw `pulldown-cmark` library
/// and the internal `Lithos` parsing pipeline. It yields `EventWithRange`
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
            })
        } else {
            StreamState::Unmerged(EventAdapterIter {
                inner: break_policy_iter,
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
    type Item = Result<EventWithRange<'source>, NoteIngestError>;

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

#[expect(
    clippy::missing_trait_methods,
    reason = "Implementing all iterator methods is unnecessary for this \
              internal wrapper"
)]
impl<'source> Iterator for StreamState<'source> {
    type Item = Result<EventWithRange<'source>, NoteIngestError>;

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

/// An iterator that adapts and normalizes pulldown-cmark events.
pub(crate) struct EventAdapterIter<'source, I>
where
    I: Iterator<Item = (Event<'source>, Range<usize>)>,
{
    inner: I,
}

#[derive(Debug, Clone, Copy)]
struct ParserEventMapper;

impl ParserEventMapper {
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Pattern matches borrowed parser events by design"
    )]
    fn try_from_event<'source>(
        event: &Event<'source>,
    ) -> Option<ParserEvent<'source>> {
        match event {
            Event::Start(tag) => Self::try_from_start_tag(tag),
            Event::End(tag_end) => Self::try_from_end_tag(*tag_end),
            Event::Text(text) => {
                Some(ParserEvent::Inline(InlineEvent::Text(text.clone())))
            }
            Event::Code(code) => {
                Some(ParserEvent::Inline(InlineEvent::CodeSpan(code.clone())))
            }
            Event::InlineHtml(html) | Event::Html(html) => {
                Some(ParserEvent::Inline(InlineEvent::Html(html.clone())))
            }
            Event::Rule => Some(ParserEvent::ThematicBreak),
            Event::SoftBreak
            | Event::HardBreak
            | Event::FootnoteReference(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_) => None,
            Event::TaskListMarker(checked) => {
                Some(ParserEvent::TaskListMarker(*checked))
            }
        }
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Pattern matches borrowed pulldown tags by design"
    )]
    fn try_from_start_tag<'source>(
        tag: &pulldown_cmark::Tag<'source>,
    ) -> Option<ParserEvent<'source>> {
        match tag {
            pulldown_cmark::Tag::Paragraph => {
                Some(ParserEvent::BlockStart(BlockType::Paragraph))
            }
            pulldown_cmark::Tag::Heading {
                level,
                ..
            } => Some(ParserEvent::BlockStart(BlockType::Heading {
                level: *level,
            })),
            pulldown_cmark::Tag::BlockQuote(_) => {
                Some(ParserEvent::BlockStart(BlockType::BlockQuote))
            }
            pulldown_cmark::Tag::List(start) => {
                Some(ParserEvent::BlockStart(BlockType::List {
                    start: *start,
                }))
            }
            pulldown_cmark::Tag::Item => {
                Some(ParserEvent::BlockStart(BlockType::Item))
            }
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
                Some(ParserEvent::BlockStart(BlockType::CodeBlock {
                    language,
                }))
            }
            pulldown_cmark::Tag::MetadataBlock(format) => {
                Some(ParserEvent::BlockStart(BlockType::Frontmatter {
                    format: *format,
                }))
            }
            pulldown_cmark::Tag::Emphasis => Some(ParserEvent::Inline(
                InlineEvent::Start(InlineTag::Emphasis),
            )),
            pulldown_cmark::Tag::Strong => {
                Some(ParserEvent::Inline(InlineEvent::Start(InlineTag::Strong)))
            }
            pulldown_cmark::Tag::Strikethrough => Some(ParserEvent::Inline(
                InlineEvent::Start(InlineTag::Strikethrough),
            )),
            pulldown_cmark::Tag::Superscript => Some(ParserEvent::Inline(
                InlineEvent::Start(InlineTag::Superscript),
            )),
            pulldown_cmark::Tag::Subscript => Some(ParserEvent::Inline(
                InlineEvent::Start(InlineTag::Subscript),
            )),
            pulldown_cmark::Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            } => {
                Some(ParserEvent::Inline(InlineEvent::Start(InlineTag::Link {
                    link_type: *link_type,
                    dest_url: dest_url.clone(),
                    title: title.clone(),
                    id: id.clone(),
                })))
            }
            pulldown_cmark::Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            } => Some(ParserEvent::Inline(InlineEvent::Start(
                InlineTag::Image {
                    link_type: *link_type,
                    dest_url: dest_url.clone(),
                    title: title.clone(),
                    id: id.clone(),
                },
            ))),
            pulldown_cmark::Tag::HtmlBlock
            | pulldown_cmark::Tag::Table(_)
            | pulldown_cmark::Tag::TableHead
            | pulldown_cmark::Tag::TableRow
            | pulldown_cmark::Tag::TableCell
            | pulldown_cmark::Tag::FootnoteDefinition(_)
            | pulldown_cmark::Tag::DefinitionList
            | pulldown_cmark::Tag::DefinitionListTitle
            | pulldown_cmark::Tag::DefinitionListDefinition => None,
        }
    }

    fn try_from_end_tag<'source>(
        tag_end: pulldown_cmark::TagEnd,
    ) -> Option<ParserEvent<'source>> {
        match tag_end {
            pulldown_cmark::TagEnd::Paragraph => {
                Some(ParserEvent::BlockEnd(BlockType::Paragraph))
            }
            pulldown_cmark::TagEnd::Heading(level) => {
                Some(ParserEvent::BlockEnd(BlockType::Heading {
                    level,
                }))
            }
            pulldown_cmark::TagEnd::BlockQuote(_) => {
                Some(ParserEvent::BlockEnd(BlockType::BlockQuote))
            }
            pulldown_cmark::TagEnd::List(_) => {
                Some(ParserEvent::BlockEnd(BlockType::List {
                    start: None,
                }))
            }
            pulldown_cmark::TagEnd::Item => {
                Some(ParserEvent::BlockEnd(BlockType::Item))
            }
            pulldown_cmark::TagEnd::CodeBlock => {
                Some(ParserEvent::BlockEnd(BlockType::CodeBlock {
                    language: None,
                }))
            }
            pulldown_cmark::TagEnd::MetadataBlock(format) => {
                Some(ParserEvent::BlockEnd(BlockType::Frontmatter {
                    format,
                }))
            }
            pulldown_cmark::TagEnd::Emphasis => Some(ParserEvent::Inline(
                InlineEvent::End(InlineTagEnd::Emphasis),
            )),
            pulldown_cmark::TagEnd::Strong => Some(ParserEvent::Inline(
                InlineEvent::End(InlineTagEnd::Strong),
            )),
            pulldown_cmark::TagEnd::Strikethrough => Some(ParserEvent::Inline(
                InlineEvent::End(InlineTagEnd::Strikethrough),
            )),
            pulldown_cmark::TagEnd::Superscript => Some(ParserEvent::Inline(
                InlineEvent::End(InlineTagEnd::Superscript),
            )),
            pulldown_cmark::TagEnd::Subscript => Some(ParserEvent::Inline(
                InlineEvent::End(InlineTagEnd::Subscript),
            )),
            pulldown_cmark::TagEnd::Link => {
                Some(ParserEvent::Inline(InlineEvent::End(InlineTagEnd::Link)))
            }
            pulldown_cmark::TagEnd::Image => {
                Some(ParserEvent::Inline(InlineEvent::End(InlineTagEnd::Image)))
            }
            pulldown_cmark::TagEnd::HtmlBlock
            | pulldown_cmark::TagEnd::Table
            | pulldown_cmark::TagEnd::TableHead
            | pulldown_cmark::TagEnd::TableRow
            | pulldown_cmark::TagEnd::TableCell
            | pulldown_cmark::TagEnd::FootnoteDefinition
            | pulldown_cmark::TagEnd::DefinitionList
            | pulldown_cmark::TagEnd::DefinitionListTitle
            | pulldown_cmark::TagEnd::DefinitionListDefinition => None,
        }
    }
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
    type Item = Result<EventWithRange<'source>, NoteIngestError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        for (event, byte_range) in self.inner.by_ref() {
            let Some(parser_event) = ParserEventMapper::try_from_event(&event)
            else {
                continue;
            };

            let range = match SourceByteRange::try_from(byte_range) {
                Ok(range) => range,
                Err(error) => return Some(Err(NoteIngestError::Domain(error))),
            };

            return Some(Ok(EventWithRange::new(parser_event, range)));
        }
        None
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

    mod event_with_range {
        use pulldown_cmark::CowStr;

        use super::*;

        #[test]
        fn new_preserves_text_event_payload() {
            let range = SourceByteRange::new(
                SourceByteOffset::new(0),
                SourceByteOffset::new(5),
            )
            .expect("test range should be valid");

            let event = EventWithRange::new(
                ParserEvent::Inline(InlineEvent::Text(CowStr::Borrowed(
                    "hello",
                ))),
                range,
            );

            assert!(
                matches!(event.event(), ParserEvent::Inline(InlineEvent::Text(text)) if text.as_ref() == "hello"),
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

            let event = EventWithRange::new(ParserEvent::ThematicBreak, range);

            assert_eq!(
                event.range(),
                range,
                "range accessor should preserve original source range"
            );
        }

        #[test]
        fn try_from_converts_valid_tuple() {
            let event = ParserEvent::Inline(InlineEvent::Text(
                CowStr::Borrowed("test"),
            ));
            let byte_range = 0..4;

            let result = EventWithRange::try_from((event, byte_range));

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
            let event = ParserEvent::Inline(InlineEvent::Text(
                CowStr::Borrowed("test"),
            ));
            // Explicitly construct invalid range to test error handling
            #[expect(
                clippy::reversed_empty_ranges,
                reason = "Testing error handling for invalid ranges"
            )]
            let invalid_range = 10..5; // end before start

            let result = EventWithRange::try_from((event, invalid_range));

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
                    ParserEvent::Inline(InlineEvent::Text(text))
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
                    matches!(event.event(), ParserEvent::Inline(InlineEvent::Text(text)) if text.as_ref() == "\n")
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
                    ParserEvent::Inline(InlineEvent::Text(text))
                        if text.as_ref() == "\n"
                )),
                "soft-as-space policy should not emit hard-break newline text"
            );
        }
    }

    mod event_adapter_iter_math_contract {
        use super::*;

        #[test]
        fn inline_math_event_is_not_emitted_into_parser_ir() {
            let events =
                collect_events("a $x+y$ b", EventStreamConfig::default());

            assert!(
                !events.iter().any(|event| {
                    matches!(
                        event.event(),
                        ParserEvent::Inline(InlineEvent::Text(text)) if text.as_ref() == "x+y"
                    )
                }),
                "math payload should not surface as plain inline text when math events are dropped"
            );
        }

        #[test]
        fn display_math_event_is_not_emitted_into_parser_ir() {
            let events = collect_events(
                "$$\\na^2 + b^2\\n$$",
                EventStreamConfig::default(),
            );

            assert!(
                !events.iter().any(|event| {
                    matches!(
                        event.event(),
                        ParserEvent::Inline(InlineEvent::Text(text))
                            if text.as_ref().contains("a^2 + b^2")
                    )
                }),
                "display math payload should not surface as plain inline text \
                 with current mapper contract"
            );
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
    ) -> Vec<EventWithRange<'_>> {
        let (stream, _references) = MarkdownEventStream::new(source, config);
        stream
            .collect::<Result<Vec<_>, _>>()
            .expect("event stream should not produce invalid spans")
    }

    fn collect_text_payloads(events: &[EventWithRange<'_>]) -> Vec<String> {
        events
            .iter()
            .filter_map(|event| {
                #[expect(
                    clippy::pattern_type_mismatch,
                    reason = "Matching borrowed enum payload inside iterator \
                              adapter"
                )]
                if let ParserEvent::Inline(InlineEvent::Text(text)) =
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
