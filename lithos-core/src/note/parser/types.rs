//! CommonMark-oriented parser IR types.
//!
//! This module defines the parser-owned intermediate representation shared by
//! stream adaptation, structure building, and scanner-facing extraction.
//! The naming intentionally follows CommonMark terminology (`block`, `inline`,
//! `list item`, `thematic break`, etc.) to avoid introducing project-specific
//! jargon.
//!
//! # Scope
//!
//! - Represents markdown semantics required by Lithos ingestion.
//! - Excludes renderer-specific concepts.
//! - Carries source ranges via [`RangedEvent`] for diagnostics.

#![cfg_attr(
    not(test),
    expect(dead_code, reason = "Parser IR migration is staged")
)]
use std::{borrow::Cow, ops::Range};

use crate::note::{error::NoteIngestError, position::SourceByteRange};

/// Parser event with attached source byte range.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RangedEvent<'source> {
    event: ParserEvent<'source>,
    range: SourceByteRange,
}

impl<'source> RangedEvent<'source> {
    /// Creates a ranged parser event.
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

    /// Returns the parser event.
    #[must_use]
    #[inline]
    pub(crate) const fn event(&self) -> &ParserEvent<'source> {
        &self.event
    }

    /// Returns source byte range.
    #[must_use]
    #[inline]
    pub(crate) const fn range(&self) -> SourceByteRange {
        self.range
    }

    /// Consumes the event and returns its parts.
    #[must_use]
    #[inline]
    pub(crate) fn into_parts(self) -> (ParserEvent<'source>, SourceByteRange) {
        (self.event, self.range)
    }
}

impl<'source> TryFrom<(ParserEvent<'source>, Range<usize>)>
    for RangedEvent<'source>
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

/// Parser IR event used between adapter and structure phases.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ParserEvent<'source> {
    /// Start of a block element.
    BlockStart(BlockStart<'source>),
    /// End of a block element.
    BlockEnd(BlockEnd),
    /// Inline content token.
    Inline(InlineToken<'source>),
    /// Task list marker emitted within list item content.
    TaskListMarker(bool),
    /// Thematic break (`---`, `***`, `___`).
    ThematicBreak,
}

/// Block start token.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BlockStart<'source> {
    Paragraph,
    Heading {
        level: HeadingLevel,
    },
    BlockQuote,
    List {
        kind: ListKind,
    },
    ListItem,
    CodeBlock {
        info_string: Option<Cow<'source, str>>,
    },
    Frontmatter {
        format: FrontmatterFormat,
    },
}

/// Block end token.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum BlockEnd {
    Paragraph,
    Heading,
    BlockQuote,
    List,
    ListItem,
    CodeBlock,
    Frontmatter,
}

/// Inline token.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InlineToken<'source> {
    DelimiterStart(InlineDelimiterStart<'source>),
    DelimiterEnd(InlineDelimiterEnd),
    Text(Cow<'source, str>),
    InlineCode(Cow<'source, str>),
    Html(Cow<'source, str>),
    #[expect(
        dead_code,
        reason = "Line break IR is enabled in a later parser phase"
    )]
    LineBreak(LineBreakKind),
    Math {
        kind: MathKind,
        content: Cow<'source, str>,
    },
}

/// Start delimiter for inline spans.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InlineDelimiterStart<'source> {
    Emphasis,
    Strong,
    Strikethrough,
    Superscript,
    Subscript,
    Link {
        kind: LinkKind,
        destination: Cow<'source, str>,
        title: Cow<'source, str>,
        label: Cow<'source, str>,
    },
    Image {
        kind: LinkKind,
        destination: Cow<'source, str>,
        title: Cow<'source, str>,
        label: Cow<'source, str>,
    },
    _Marker(std::marker::PhantomData<&'source str>),
}

/// End delimiter for inline spans.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum InlineDelimiterEnd {
    Emphasis,
    Strong,
    Strikethrough,
    Superscript,
    Subscript,
    Link,
    Image,
}

/// Heading levels (`#` through `######`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl HeadingLevel {
    /// Converts to numeric heading level.
    #[must_use]
    #[inline]
    pub(crate) const fn as_u8(self) -> u8 {
        match self {
            Self::H1 => 1,
            Self::H2 => 2,
            Self::H3 => 3,
            Self::H4 => 4,
            Self::H5 => 5,
            Self::H6 => 6,
        }
    }
}

/// List kind from `CommonMark` list syntax.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ListKind {
    Unordered,
    Ordered(u64),
}

/// Frontmatter format.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum FrontmatterFormat {
    Yaml,
    Toml,
}

/// Link syntactic kind.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum LinkKind {
    Inline,
    Reference,
    ReferenceUnknown,
    Collapsed,
    CollapsedUnknown,
    Shortcut,
    ShortcutUnknown,
    Autolink,
    Email,
    WikiLink {
        has_pothole: bool,
    },
}

/// Line break kind.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[expect(
    dead_code,
    reason = "Line break IR is enabled in a later parser phase"
)]
pub(crate) enum LineBreakKind {
    Soft,
    Hard,
}

/// Math token kind.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum MathKind {
    Inline,
    Display,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod ranged_event {
        use super::*;

        #[test]
        fn try_from_valid_range_creates_event() {
            let event = ParserEvent::ThematicBreak;
            let ranged = RangedEvent::try_from((event.clone(), 0..3))
                .expect("valid range must construct event");

            assert_eq!(ranged.event(), &event);
            assert_eq!(ranged.range().len(), 3);
            assert!(!ranged.range().is_empty());
        }

        #[expect(
            clippy::reversed_empty_ranges,
            reason = "Intentional invalid range to test \
                      SourceByteRange::try_from error"
        )]
        #[test]
        fn try_from_invalid_range_returns_domain_error() {
            let event = ParserEvent::ThematicBreak;
            let invalid = 5..4;
            let result = RangedEvent::try_from((event, invalid));

            assert!(matches!(result, Err(NoteIngestError::Domain(_))));
        }
    }

    mod heading_level {
        use super::*;

        #[test]
        fn heading_level_maps_to_expected_number() {
            assert_eq!(HeadingLevel::H1.as_u8(), 1);
            assert_eq!(HeadingLevel::H2.as_u8(), 2);
            assert_eq!(HeadingLevel::H3.as_u8(), 3);
            assert_eq!(HeadingLevel::H4.as_u8(), 4);
            assert_eq!(HeadingLevel::H5.as_u8(), 5);
            assert_eq!(HeadingLevel::H6.as_u8(), 6);
        }
    }

    mod parser_event_shapes {
        use super::*;

        #[test]
        fn ordered_list_kind_preserves_start_number() {
            let event = ParserEvent::BlockStart(BlockStart::List {
                kind: ListKind::Ordered(42),
            });

            assert!(matches!(
                event,
                ParserEvent::BlockStart(BlockStart::List {
                    kind: ListKind::Ordered(42)
                })
            ));
        }

        #[test]
        fn math_inline_token_keeps_kind_and_content() {
            let token = InlineToken::Math {
                kind: MathKind::Display,
                content: Cow::Borrowed("x^2 + y^2"),
            };

            assert!(matches!(
                token,
                InlineToken::Math {
                    kind: MathKind::Display,
                    content,
                } if content == "x^2 + y^2"
            ));
        }
    }
}
