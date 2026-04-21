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

/// An iterator that adapts and normalizes pulldown-cmark events.
///
/// This iterator executes standard pipeline transformations, such as
/// normalizing implicit `SoftBreak`s and `HardBreak`s into standard text
/// tokens. Because it operates on the raw `(Event<'a>, Range<usize>)` tuple, it
/// can safely be composed within `pulldown-cmark`'s `TextMergeWithOffset`
/// utility.
pub struct EventAdapterIter<'source> {
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
            let new_event = match (event, self.policy) {
                (
                    Event::SoftBreak,
                    BreakPolicy::SoftAsSpace | BreakPolicy::NormalizeAsText,
                ) => Event::Text(CowStr::Borrowed(" ")),
                (
                    Event::HardBreak,
                    BreakPolicy::HardAsNewLine | BreakPolicy::NormalizeAsText,
                ) => Event::Text(CowStr::Borrowed("\n")),
                // Add future event adaptations here
                (other, _) => other,
            };
            (new_event, range)
        })
    }
}

/// Inner state of the event stream, supporting merged and unmerged text.
#[non_exhaustive]
pub enum StreamState<'source> {
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

/// A normalized stream of markdown events with unified spans and reference
/// handling.
///
/// This acts as the facade adapter between the raw `pulldown-cmark` library
/// and the internal `Lithos` parsing pipeline. It yields `SpannedEvent`
/// structures instead of raw tuples.
pub struct MarkdownEventStream<'source> {
    state: StreamState<'source>,
    references: ReferenceDefinitions,
}

impl<'source> MarkdownEventStream<'source> {
    /// Creates a new event stream with the provided configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::parser::stream::MarkdownEventStream;
    /// # use lithos_core::note::parser::config::EventStreamConfig;
    /// let source = "Here is some markdown text.";
    /// let config = EventStreamConfig::default();
    /// let mut stream = MarkdownEventStream::new(source, &config);
    ///
    /// assert!(stream.next().is_some());
    /// ```
    #[must_use]
    #[inline]
    pub fn new(source: &'source str, config: &EventStreamConfig) -> Self {
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
    /// ```
    /// # use lithos_core::note::parser::stream::MarkdownEventStream;
    /// # use lithos_core::note::parser::config::EventStreamConfig;
    /// let source = "[foo]: /url\n[foo][]";
    /// let config = EventStreamConfig::default();
    /// let stream = MarkdownEventStream::new(source, &config);
    ///
    /// let defs = stream.references();
    /// assert_eq!(defs.resolve("foo"), Some("/url"));
    /// ```
    #[must_use]
    #[inline]
    pub fn references(&self) -> &ReferenceDefinitions {
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
