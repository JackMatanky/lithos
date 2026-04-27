//! Event stream processing and normalization.
//!
//! This module provides the `MarkdownEventStream` component, which acts as
//! a strict boundary between `pulldown-cmark` and the rest of the parsing
//! pipeline. It is responsible for instantiating the underlying parser,
//! managing the iterator state, handling event transformations (like break
//! normalization), and extracting link reference definitions safely.

use std::ops::Range;

use pulldown_cmark::{
    CowStr, Event, OffsetIter, Parser, utils::TextMergeWithOffset,
};

use super::{
    config::{BreakPolicy, EventStreamConfig},
    event::SpannedEvent,
    references::ReferenceDefinitions,
};
use crate::note::{error::NoteIngestError, position::SourceByteRange};

/// A normalized stream of markdown events with unified spans and reference
/// handling.
///
/// This acts as the facade adapter between the raw `pulldown-cmark` library
/// and the internal `Lithos` parsing pipeline. It yields `SpannedEvent`
/// structures instead of raw tuples.
pub(crate) struct MarkdownEventStream<'source> {
    state: StreamState<'source>,
    references: ReferenceDefinitions,
}

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "New adapter layer not yet integrated")
)]
impl<'source> MarkdownEventStream<'source> {
    /// Creates a new event stream with the provided configuration.
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
    /// let mut stream = MarkdownEventStream::new(source, config);
    ///
    /// assert!(stream.next().is_some());
    /// ```
    #[must_use]
    #[inline]
    pub(crate) fn new(source: &'source str, config: EventStreamConfig) -> Self {
        let parser = Parser::new_ext(source, config.options);

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
            policy: config.break_policy,
        };

        let state = if config.merge_text {
            StreamState::Merged(TextMergeWithOffset::new(normalized))
        } else {
            StreamState::Unmerged(normalized)
        };

        Self {
            state,
            references,
        }
    }

    /// Returns the normalized reference link definitions.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Note: Cannot run doctest for pub(crate) types from external test crate
    /// use lithos_core::note::parser::stream::MarkdownEventStream;
    /// use lithos_core::note::parser::config::EventStreamConfig;
    ///
    /// let source = "[foo]: /url\n[foo][]";
    /// let config = EventStreamConfig::default();
    /// let stream = MarkdownEventStream::new(source, config);
    ///
    /// let defs = stream.references();
    /// assert_eq!(defs.resolve("foo"), Some("/url"));
    /// ```
    #[must_use]
    #[inline]
    pub(crate) fn references(&self) -> &ReferenceDefinitions {
        &self.references
    }
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Implementing all iterator methods is unnecessary for this \
              public wrapper"
)]
impl<'source> Iterator for MarkdownEventStream<'source> {
    type Item = Result<SpannedEvent<'source>, NoteIngestError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.state.next().map(|(event, range)| {
            let span = SourceByteRange::try_from(range)
                .map_err(NoteIngestError::Domain)?;
            Ok(SpannedEvent::new(event, span))
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

    mod markdown_event_stream_references {
        use super::*;

        #[test]
        fn extracts_reference_definitions_from_source() {
            let source = "[foo]: /url\n\n[foo][]";
            let stream =
                MarkdownEventStream::new(source, EventStreamConfig::default());

            assert_eq!(
                stream.references().resolve("foo"),
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
                    .all(|event| event.span.start() <= event.span.end()),
                "all emitted events should map to valid source span ranges"
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
    ) -> Vec<SpannedEvent<'_>> {
        MarkdownEventStream::new(source, config)
            .collect::<Result<Vec<_>, _>>()
            .expect("event stream should not produce invalid spans")
    }

    fn collect_text_payloads(events: &[SpannedEvent<'_>]) -> Vec<String> {
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
