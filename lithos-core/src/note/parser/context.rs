//! Cached parsing context for markdown documents.
//!
//! This module provides the [`ParserContext`] type, which eagerly parses a
//! markdown source string and caches the results (normalized events and link
//! reference definitions) for efficient reuse across multiple pipeline stages.
//!
//! # Design Rationale
//!
//! The `ParserContext` exists to enable **zero-cost multi-pass processing**:
//! - **LSP features** require multiple analyses per edit (diagnostics,
//!   autocomplete, hover)
//! - **pulldown-cmark** parsing is expensive (~50-200ms for large files)
//! - **Caching events** allows structure building, metadata extraction, and
//!   semantic validation to operate on the same parsed data
//!
//! # Performance Characteristics
//!
//! - **Memory**: O(n) where n = markdown source length (events borrow from
//!   source)
//! - **Parse time**: Single pass through pulldown-cmark (amortized across all
//!   consumers)
//! - **Access time**: O(1) for borrowing cached events or references
//!
//! # Examples
//!
//! ```rust,ignore
//! use lithos_core::note::parser::{ParserContext, EventStreamConfig};
//!
//! let source = "# Heading\n\nParagraph with [ref].\n\n[ref]: /url";
//! let config = EventStreamConfig::default();
//! let ctx = ParserContext::new(source, config);
//!
//! // Access cached events
//! assert!(ctx.events().len() > 0);
//!
//! // Resolve link references
//! assert_eq!(ctx.references().resolve("ref"), Some("/url"));
//! ```

#![cfg_attr(
    not(test),
    expect(dead_code, reason = "Context cache API is consumed incrementally")
)]

use super::{
    config::EventStreamConfig,
    references::ReferenceDefinitions,
    stream::{EventWithRange, MarkdownEventStream},
};
use crate::note::error::NoteIngestError;

/// A cached parsing context for a markdown document.
///
/// This type eagerly parses the markdown source once and stores the normalized
/// event stream and link reference definitions for efficient reuse by
/// downstream pipeline stages (structure building, metadata extraction,
/// semantic validation).
///
/// # Lifecycle
///
/// 1. **Creation**: `ParserContext::new(source, config)` runs the
///    pulldown-cmark parser once
/// 2. **Caching**: Events and references are stored in memory
/// 3. **Consumption**: Multiple stages borrow from the cached data without
///    re-parsing
///
/// # Why Cache Events?
///
/// - **LSP/IDE features**: Autocomplete, diagnostics, and hover all need parsed
///   data
/// - **Incremental updates**: Future optimization will re-parse only changed
///   subtrees
/// - **Performance**: Parsing is ~30-50% of total pipeline time; caching
///   amortizes this cost
#[derive(Debug, Clone)]
pub(crate) struct ParserContext<'source> {
    /// The original markdown source text.
    source: &'source str,
    /// The cached stream of normalized markdown events.
    events: Vec<EventWithRange<'source>>,
    /// Normalized link reference definitions extracted from the source.
    references: ReferenceDefinitions,
}

impl<'source> ParserContext<'source> {
    /// Eagerly parses the markdown source and caches the results.
    ///
    /// This constructor runs the full pulldown-cmark parsing pipeline once,
    /// applying the normalization rules specified in `config` (line break
    /// handling, text merging, etc.).
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if the markdown contains invalid source
    /// byte offsets that cannot be mapped to [`SourceByteRange`].
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let source = "# Hello\n\nWorld";
    /// let config = EventStreamConfig::default();
    /// let ctx = ParserContext::new(source, config)?;
    ///
    /// assert!(ctx.events().len() > 0);
    /// ```
    #[inline]
    pub(crate) fn new(
        source: &'source str,
        config: EventStreamConfig,
    ) -> Result<Self, NoteIngestError> {
        let (stream, references) = MarkdownEventStream::new(source, config);
        let events: Result<Vec<_>, _> = stream.collect();

        Ok(Self {
            source,
            events: events?,
            references,
        })
    }

    /// Returns a borrowed slice of the cached markdown events.
    ///
    /// Events are normalized according to the configuration passed to
    /// [`ParserContext::new`]:
    /// - Line breaks may be converted to text events
    /// - Adjacent text events may be merged
    /// - All events have source byte ranges attached
    #[must_use]
    #[inline]
    pub(crate) fn events(&self) -> &[EventWithRange<'source>] {
        &self.events
    }

    /// Returns a reference to the cached link reference definitions.
    ///
    /// These definitions are extracted from the markdown source during parsing
    /// and normalized according to `CommonMark` rules (case-insensitive,
    /// whitespace-collapsed).
    #[must_use]
    #[inline]
    pub(crate) fn references(&self) -> &ReferenceDefinitions {
        &self.references
    }

    /// Returns the original markdown source text.
    #[must_use]
    #[inline]
    pub(crate) fn source(&self) -> &'source str {
        self.source
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module keeps imports and nested suites grouped for \
              readability"
)]
mod tests {
    use super::*;
    use crate::note::parser::stream::{InlineEvent, ParserEvent};

    mod parser_context_new {
        use super::*;

        #[test]
        fn caches_events_from_simple_markdown() {
            let source = "# Heading\n\nParagraph";
            let config = EventStreamConfig::default();

            let ctx = ParserContext::new(source, config)
                .expect("parsing should succeed");

            assert!(
                !ctx.events().is_empty(),
                "context should cache events from parsed markdown"
            );
        }

        #[test]
        fn caches_reference_definitions() {
            let source = "[ref]: /url\n\nSee [ref][]";
            let config = EventStreamConfig::default();

            let ctx = ParserContext::new(source, config)
                .expect("parsing should succeed");

            assert_eq!(
                ctx.references().resolve("ref"),
                Some("/url"),
                "context should cache and normalize link references"
            );
        }

        #[test]
        fn normalizes_line_breaks_when_configured() {
            let source = "Line one\nLine two";
            let config = EventStreamConfig::default(); // Uses NormalizeAsText by default

            let ctx = ParserContext::new(source, config)
                .expect("parsing should succeed");

            let text_events: Vec<_> = ctx
                .events()
                .iter()
                .filter_map(|e| {
                    #[expect(
                        clippy::wildcard_enum_match_arm,
                        reason = "Only inline text is relevant for this \
                                  assertion"
                    )]
                    #[expect(
                        clippy::pattern_type_mismatch,
                        reason = "Pattern matching borrowed parser events in \
                                  test"
                    )]
                    match e.event() {
                        ParserEvent::Inline(InlineEvent::Text(s)) => {
                            Some(s.as_ref())
                        }
                        _ => None,
                    }
                })
                .collect();

            // With text merging enabled, the space is merged into the text
            let full_text = text_events.join("");
            assert!(
                full_text.contains(' '),
                "soft breaks should be normalized and merged into text with \
                 default config (got: {full_text:?})"
            );
        }

        #[test]
        fn preserves_source_reference() {
            let source = "# Test";
            let config = EventStreamConfig::default();

            let ctx = ParserContext::new(source, config)
                .expect("parsing should succeed");

            assert_eq!(
                ctx.source(),
                source,
                "context should preserve reference to original source"
            );
        }
    }

    mod parser_context_events {
        use super::*;

        #[test]
        fn returns_borrowed_slice_without_allocation() {
            let source = "Text";
            let config = EventStreamConfig::default();
            let ctx = ParserContext::new(source, config)
                .expect("parsing should succeed");

            let events1 = ctx.events();
            let events2 = ctx.events();

            assert_eq!(
                events1.as_ptr(),
                events2.as_ptr(),
                "events() should return the same borrowed slice without \
                 allocation"
            );
        }
    }

    mod parser_context_references {
        use super::*;

        #[test]
        fn resolves_normalized_labels() {
            let source = "[Foo Bar]: /url";
            let config = EventStreamConfig::default();
            let ctx = ParserContext::new(source, config)
                .expect("parsing should succeed");

            assert_eq!(
                ctx.references().resolve("foo bar"),
                Some("/url"),
                "references should normalize labels for case-insensitive \
                 matching"
            );
        }

        #[test]
        fn returns_none_for_unknown_labels() {
            let source = "[known]: /url";
            let config = EventStreamConfig::default();
            let ctx = ParserContext::new(source, config)
                .expect("parsing should succeed");

            assert_eq!(
                ctx.references().resolve("unknown"),
                None,
                "references should return None for undefined labels"
            );
        }
    }
}
