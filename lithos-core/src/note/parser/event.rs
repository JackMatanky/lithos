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
/// ```
/// # use lithos_core::note::parser::event::SpannedEvent;
/// # use lithos_core::note::position::{SourceByteRange, SourceByteOffset};
/// # use pulldown_cmark::{Event, CowStr};
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
pub struct SpannedEvent<'source> {
    /// The underlying markdown event emitted by the parser.
    pub event: Event<'source>,
    /// The exact source byte range covering the parsed text for this event.
    pub span: SourceByteRange,
}

impl<'source> SpannedEvent<'source> {
    /// Creates a new `SpannedEvent`.
    #[must_use]
    #[inline]
    pub const fn new(event: Event<'source>, span: SourceByteRange) -> Self {
        Self {
            event,
            span,
        }
    }
}
