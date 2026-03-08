//! Structural entities for document organization.
//!
//! Provides [`crate::note::structure::Section`] to model sequential document
//! structure alongside headings.

use super::error::{LinkError, NoteError};
use crate::note::{heading::Heading, position::SourceByteRange};

/// Represents a content section within a note.
///
/// A section groups content between headings. Content before the first heading
/// is represented as a section with `None` for the heading field.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::{structure::Section, position::SourceByteRange, position::SourceByteOffset};
/// let range = SourceByteRange::new(
///     SourceByteOffset::new(0),
///     SourceByteOffset::new(50),
/// )?;
/// let section = Section::new(None, range);
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
    pub fn new(heading: Option<Heading>, range: SourceByteRange) -> Self {
        Self {
            heading,
            range,
        }
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
    /// Returns [`NoteError::Link`] if the identifier is empty.
    #[inline]
    pub fn try_new(value: &str) -> Result<Self, NoteError> {
        let text = value.trim();
        if text.is_empty() {
            return Err(NoteError::Link(LinkError::EmptyBlockRefAnchor));
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
        let section = Section::new(heading, range);
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
        let section = Section::new(None, range);
        assert!(section.heading().is_none(), "Heading should be None");
    }
}
