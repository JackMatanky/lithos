//! Event wrappers for markdown tokens.
//!
//! This module defines the core data artifacts produced by the
//! [`MarkdownEventStream`](crate::note::parser::stream::MarkdownEventStream).
//! It isolates the rest of the parsing pipeline from raw `pulldown-cmark`
//! iterator details.

use pulldown_cmark::Event;

use crate::note::position::SourceByteRange;

/// An event paired with its original source byte range.
///
/// This struct guarantees that every parsed token is strictly bound to its
/// exact origin within the unparsed source document. This allows downstream
/// scanners and compilers to report diagnostics that accurately highlight the
/// original text rather than a normalized projection.
///
/// # Examples
///
/// ```rust,ignore
/// // Note: Cannot run doctest for pub(crate) types from external test crate
/// use lithos_core::note::position::{SourceByteRange, SourceByteOffset};
/// use pulldown_cmark::{Event, CowStr};
/// use lithos_core::note::parser::event::SpannedEvent;
///
/// let start = SourceByteOffset::new(0);
/// let end = SourceByteOffset::new(5);
/// let span = SourceByteRange::new(start, end).unwrap();
///
/// let event = SpannedEvent::new(
///     Event::Text(CowStr::Borrowed("hello")),
///     span,
/// );
///
/// assert_eq!(event.span.len(), 5);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "Internal event fields are accessible within the crate"
)]
pub(crate) struct SpannedEvent<'source> {
    /// The underlying markdown event emitted by the parser.
    pub(crate) event: Event<'source>,
    /// The exact source byte range covering the parsed text for this event.
    pub(crate) span: SourceByteRange,
}

impl<'source> SpannedEvent<'source> {
    /// Creates a new `SpannedEvent`.
    ///
    /// # Errors
    ///
    /// Returns a `NoteIngestError` if the provided span is invalid.
    #[must_use]
    #[inline]
    pub(crate) const fn new(
        event: Event<'source>,
        span: SourceByteRange,
    ) -> Self {
        Self {
            event,
            span,
        }
    }
}
