//! Structural entities for document organization.
//!
//! Provides [`crate::note::structure::Section`] to model sequential document
//! structure alongside headings.

#![expect(
    clippy::exhaustive_enums,
    reason = "rkyv generates exhaustive archived enums"
)]

use super::error::{LinkError, NoteError};
use crate::note::{
    heading::Heading,
    position::{SourceByteOffset, SourceByteRange},
    raw::{RawBlockRef, RawSection, RawSectionKind},
};

/// Section kinds aligned with Obsidian cached metadata.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum SectionKind {
    /// A paragraph block.
    Paragraph,
    /// A heading block.
    Heading,
    /// A list block.
    List,
    /// A code block.
    Code,
    /// A block quote.
    BlockQuote,
    /// A callout block quote.
    Callout,
    /// A table block.
    Table,
    /// A frontmatter block.
    Frontmatter,
    /// Other or unknown block type.
    Other(Box<str>),
}

/// Represents a top-level section within a note.
///
/// Sections are derived from root-level markdown blocks (heading, paragraph,
/// list, etc.). Heading sections optionally store their heading value.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::{structure::{Section, SectionKind}, position::SourceByteRange, position::SourceByteOffset};
/// let range = SourceByteRange::new(
///     SourceByteOffset::new(0),
///     SourceByteOffset::new(50),
/// )?;
/// let section = Section::new(SectionKind::Paragraph, None, range);
///
/// assert!(section.heading().is_none());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Section {
    /// Section kind.
    kind: SectionKind,
    /// Optional heading that starts this section (None for content before
    /// first heading).
    heading: Option<Heading>,
    /// Character range in the source document.
    range: SourceByteRange,
}

impl Section {
    /// Creates a new section.
    #[inline]
    #[must_use]
    pub fn new(
        kind: SectionKind,
        heading: Option<Heading>,
        range: SourceByteRange,
    ) -> Self {
        Self {
            kind,
            heading,
            range,
        }
    }

    /// Returns the section kind.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> &SectionKind {
        &self.kind
    }

    /// Returns the optional heading that starts this section.
    #[inline]
    #[must_use]
    pub const fn heading(&self) -> Option<&Heading> {
        self.heading.as_ref()
    }

    /// Returns the character range in the source document.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }
}

impl TryFrom<RawSection> for Section {
    type Error = NoteError;

    #[inline]
    fn try_from(raw: RawSection) -> Result<Self, Self::Error> {
        let kind = match raw.kind() {
            RawSectionKind::Heading => SectionKind::Heading,
            RawSectionKind::Paragraph => SectionKind::Paragraph,
            RawSectionKind::CodeBlock => SectionKind::Code,
            RawSectionKind::BlockQuote => SectionKind::BlockQuote,
            RawSectionKind::List => SectionKind::List,
            RawSectionKind::Frontmatter => SectionKind::Frontmatter,
        };
        Ok(Section::new(kind, None, raw.range()))
    }
}

/// Validated block reference identifier.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct BlockRefId(Box<str>);

impl BlockRefId {
    /// Creates a validated block reference identifier.
    ///
    /// # Errors
    ///
    /// Returns [`LinkError::EmptyBlockRefAnchor`] if the identifier is empty.
    #[inline]
    pub fn try_new(value: &str) -> Result<Self, LinkError> {
        let text = value.trim();
        if text.is_empty() {
            return Err(LinkError::EmptyBlockRefAnchor);
        }
        Ok(Self(text.into()))
    }

    /// Returns the block reference identifier as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Block reference entry with position.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct BlockRef {
    id: BlockRefId,
    position: SourceByteOffset,
}

impl BlockRef {
    /// Creates a new block reference entry.
    #[inline]
    #[must_use]
    pub fn new(id: BlockRefId, position: SourceByteOffset) -> Self {
        Self {
            id,
            position,
        }
    }

    /// Returns the block reference id.
    #[inline]
    #[must_use]
    pub fn id(&self) -> &BlockRefId {
        &self.id
    }

    /// Returns the source byte position of the block id.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }
}

impl TryFrom<RawBlockRef> for BlockRef {
    type Error = NoteError;

    #[inline]
    fn try_from(raw: RawBlockRef) -> Result<Self, Self::Error> {
        let id = BlockRefId::try_new(raw.id())?;
        Ok(BlockRef::new(id, raw.position()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::{heading::HeadingLevel, position::SourceByteOffset};

    struct SectionWithTitle {
        section: Section,
        range: SourceByteRange,
    }

    fn intro_heading() -> Heading {
        Heading::try_new(
            HeadingLevel::try_new(1).unwrap(),
            "Intro",
            SourceByteOffset::from(0u32),
        )
        .unwrap()
    }

    fn section_with_intro() -> Section {
        let start = SourceByteOffset::from(0u32);
        let end = SourceByteOffset::from(4u32);
        Section::new(
            SectionKind::Heading,
            Some(intro_heading()),
            SourceByteRange::new(start, end).unwrap(),
        )
    }

    fn section_with_title() -> SectionWithTitle {
        let heading = Some(
            Heading::try_new(
                HeadingLevel::try_new(1).unwrap(),
                "Title",
                SourceByteOffset::from(0u32),
            )
            .unwrap(),
        );
        let range = SourceByteRange::new(
            SourceByteOffset::from(0u32),
            SourceByteOffset::from(15u32),
        )
        .unwrap();
        let section = Section::new(SectionKind::Heading, heading, range);
        SectionWithTitle {
            section,
            range,
        }
    }

    #[test]
    fn section_heading_accessor_returns_heading() {
        let section = section_with_intro();
        assert!(
            matches!(section.heading(), Some(heading) if heading.text() == "Intro"),
            "Section heading text should be 'Intro'"
        );
    }

    #[test]
    fn section_range_accessor_returns_range() {
        let SectionWithTitle {
            section,
            range,
        } = section_with_title();
        assert_eq!(section.range(), range, "Section range should match");
    }

    #[test]
    fn section_heading_none() {
        let range = SourceByteRange::new(
            SourceByteOffset::from(0u32),
            SourceByteOffset::from(10u32),
        )
        .unwrap();
        let section = Section::new(SectionKind::Paragraph, None, range);
        assert!(section.heading().is_none(), "Heading should be None");
    }
}
