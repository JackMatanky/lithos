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
/// use pulldown_cmark::{Event, CowStr};
/// use lithos_core::note::parser::stream::EventWithRange;
///
/// let start = SourceByteOffset::new(0);
/// let end = SourceByteOffset::new(5);
/// let range = SourceByteRange::new(start, end).unwrap();
///
/// let event = EventWithRange::new(
///     Event::Text(CowStr::Borrowed("hello")),
///     range,
/// );
///
/// assert_eq!(event.range().len(), 5);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EventWithRange<'source> {
    event: Event<'source>,
    range: SourceByteRange,
}

impl<'source> EventWithRange<'source> {
    /// Creates a new event with its source range.
    #[must_use]
    #[inline]
    pub(crate) const fn new(
        event: Event<'source>,
        range: SourceByteRange,
    ) -> Self {
        Self {
            event,
            range,
        }
    }

    /// Returns a reference to the underlying markdown event.
    #[must_use]
    #[inline]
    pub(crate) const fn event(&self) -> &Event<'source> {
        &self.event
    }

    /// Returns the source byte range for this event.
    #[must_use]
    #[inline]
    pub(crate) const fn range(&self) -> SourceByteRange {
        self.range
    }
}

impl<'source> TryFrom<(Event<'source>, Range<usize>)>
    for EventWithRange<'source>
{
    type Error = NoteIngestError;

    fn try_from(
        (event, byte_range): (Event<'source>, Range<usize>),
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

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "New adapter layer not yet integrated")
)]
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
        let normalized = EventAdapterIter {
            inner: offset_iter,
            policy: config.break_policy(),
        };

        let state = if config.merge_text() {
            StreamState::Merged(TextMergeWithOffset::new(normalized))
        } else {
            StreamState::Unmerged(normalized)
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
        self.state.next().map(|(event, byte_range)| {
            let range = SourceByteRange::try_from(byte_range)
                .map_err(NoteIngestError::Domain)?;
            Ok(EventWithRange::new(event, range))
        })
    }
}

/// Inner state of the event stream, supporting merged and unmerged text.
#[non_exhaustive]
pub(crate) enum StreamState<'source> {
    /// Text events are dynamically merged together.
    Merged(TextMergeWithOffset<'source, EventAdapterIter<'source>>),
    /// Text events remain separated.
    Unmerged(EventAdapterIter<'source>),
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Implementing all iterator methods is unnecessary for this \
              internal wrapper"
)]
impl<'source> Iterator for StreamState<'source> {
    type Item = (Event<'source>, Range<usize>);

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
pub(crate) struct EventAdapterIter<'source> {
    inner: OffsetIter<'source, pulldown_cmark::DefaultBrokenLinkCallback>,
    policy: BreakPolicy,
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Implementing all iterator methods is unnecessary for this \
              internal wrapper"
)]
impl<'source> Iterator for EventAdapterIter<'source> {
    type Item = (Event<'source>, Range<usize>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(event, range)| {
            let new_event = match event {
                Event::SoftBreak => {
                    if let Some(replacement) =
                        self.policy.soft_break_replacement()
                    {
                        Event::Text(CowStr::Borrowed(replacement))
                    } else {
                        event
                    }
                }
                Event::HardBreak => {
                    if let Some(replacement) =
                        self.policy.hard_break_replacement()
                    {
                        Event::Text(CowStr::Borrowed(replacement))
                    } else {
                        event
                    }
                }
                other @ (Event::Start(_)
                | Event::End(_)
                | Event::Text(_)
                | Event::Code(_)
                | Event::InlineMath(_)
                | Event::DisplayMath(_)
                | Event::Html(_)
                | Event::InlineHtml(_)
                | Event::FootnoteReference(_)
                | Event::Rule
                | Event::TaskListMarker(_)) => other,
            };
            (new_event, range)
        })
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
                Event::Text(CowStr::Borrowed("hello")),
                range,
            );

            assert!(
                matches!(event.event(), Event::Text(text) if text.as_ref() == "hello"),
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

            let event = EventWithRange::new(Event::Rule, range);

            assert_eq!(
                event.range(),
                range,
                "range accessor should preserve original source range"
            );
        }

        #[test]
        fn try_from_converts_valid_tuple() {
            let event = Event::Text(CowStr::Borrowed("test"));
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
            let event = Event::Text(CowStr::Borrowed("test"));
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
        fn preserve_policy_keeps_hard_break_events() {
            let events = collect_events(
                "a\\\nb",
                test_config(BreakPolicy::Preserve, false),
            );

            assert!(
                events
                    .iter()
                    .any(|event| matches!(event.event, Event::HardBreak)),
                "preserve policy should keep hard break events untouched"
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
                    matches!(&event.event, Event::Text(text) if text.as_ref() == "\n")
                }),
                "hard-as-newline policy should map hard breaks to newline text events"
            );
        }

        #[test]
        fn soft_as_space_policy_keeps_hard_break_events() {
            let events = collect_events(
                "a\\\nb",
                test_config(BreakPolicy::SoftAsSpace, false),
            );

            assert!(
                events
                    .iter()
                    .any(|event| matches!(event.event, Event::HardBreak)),
                "soft-as-space policy should not rewrite hard breaks"
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
                if let Event::Text(text) = &event.event {
                    Some(text.to_string())
                } else {
                    None
                }
            })
            .collect()
    }
}
