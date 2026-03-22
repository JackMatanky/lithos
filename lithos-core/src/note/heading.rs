//! Heading entities for document structure.
//!
//! Provides heading levels, text validation, and parsed heading values for
//! markdown documents.

use super::{
    error::{HeadingError, LinkError, NoteError},
    raw::RawHeading,
};
use crate::note::position::SourceByteOffset;

/// Represents a heading within a note.
///
/// Headings (H1-H6) mark structural points in the document.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::{heading::{Heading, HeadingLevel}, position::SourceByteOffset};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let level = HeadingLevel::try_new(1)?;
/// let heading =
///     Heading::try_new(level, "Project Overview", SourceByteOffset::new(0))?;
///
/// assert_eq!(heading.text(), "Project Overview");
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Heading {
    /// Heading level (1-6, corresponding to # through ######).
    level: HeadingLevel,
    /// Heading text content.
    text: HeadingText,
    /// Character position in the source document.
    position: SourceByteOffset,
}

impl Heading {
    /// Creates a new heading with validation.
    ///
    /// # Errors
    /// Returns [`HeadingError::EmptyContent`] if heading text is empty.
    #[inline]
    pub fn try_new<T: Into<Box<str>>>(
        level: HeadingLevel,
        text: T,
        position: SourceByteOffset,
    ) -> Result<Self, HeadingError> {
        let text = HeadingText::try_from(text.into())?;

        Ok(Self {
            level,
            text,
            position,
        })
    }

    /// Returns the heading level.
    #[inline]
    #[must_use]
    pub const fn level(&self) -> HeadingLevel {
        self.level
    }

    /// Returns the character position in the source document.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }

    /// Returns the heading text content.
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        self.text.as_str()
    }
}

impl TryFrom<&RawHeading<'_>> for Heading {
    type Error = NoteError;

    #[inline]
    fn try_from(raw: &RawHeading<'_>) -> Result<Self, Self::Error> {
        let level = HeadingLevel::try_new(raw.level)?;
        Heading::try_new(level, raw.text.as_ref(), raw.position)
            .map_err(Into::into)
    }
}

/// Heading level (1-6).
///
/// # Errors
///
/// Returns [`HeadingError::InvalidLevel`] if the level is not between 1 and 6.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::heading::HeadingLevel;
/// let h1 = HeadingLevel::try_new(1).unwrap();
/// assert_eq!(h1.as_u8(), 1);
///
/// assert!(HeadingLevel::try_new(7).is_err());
/// ```
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct HeadingLevel(u8);

impl HeadingLevel {
    /// Creates a new `HeadingLevel`, validating it is between 1 and 6.
    ///
    /// # Errors
    /// Returns an error if the level is not in the range 1..=6.
    #[inline]
    pub fn try_new(level: u8) -> Result<Self, HeadingError> {
        if (1..=6).contains(&level) {
            Ok(Self(level))
        } else {
            Err(HeadingError::InvalidLevel {
                level: u32::from(level),
            })
        }
    }

    /// Returns the raw level value.
    #[inline]
    #[must_use]
    pub const fn as_u8(&self) -> u8 {
        self.0
    }
}

/// Validated heading text content.
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
pub struct HeadingText(Box<str>);

impl HeadingText {
    /// Creates a validated heading text value.
    ///
    /// # Errors
    ///
    /// Returns [`HeadingError::EmptyContent`] if the text is empty.
    #[inline]
    pub fn try_new(value: &str) -> Result<Self, HeadingError> {
        Self::try_from(value)
    }

    /// Creates a validated heading text value for link anchors.
    ///
    /// # Errors
    ///
    /// Returns [`LinkError::EmptyHeadingAnchor`] if the text is empty.
    #[inline]
    pub fn try_new_anchor(value: &str) -> Result<Self, LinkError> {
        let text = value.trim();
        if text.is_empty() {
            return Err(LinkError::EmptyHeadingAnchor);
        }
        Ok(Self(text.into()))
    }

    /// Returns the underlying text as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<Box<str>> for HeadingText {
    type Error = HeadingError;

    #[inline]
    fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            return Err(HeadingError::EmptyContent);
        }
        Ok(Self(value))
    }
}

impl TryFrom<&str> for HeadingText {
    type Error = HeadingError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(Box::<str>::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary_heading() -> Heading {
        Heading::try_new(
            HeadingLevel::try_new(3).unwrap(),
            "Summary",
            SourceByteOffset::from(22u32),
        )
        .unwrap()
    }

    fn implementation_heading() -> Heading {
        Heading::try_new(
            HeadingLevel::try_new(2).unwrap(),
            "Implementation",
            SourceByteOffset::from(10u32),
        )
        .unwrap()
    }

    #[test]
    fn heading_level_accessor_returns_level() {
        let heading = summary_heading();
        assert_eq!(heading.level().as_u8(), 3, "Heading level should be 3");
    }

    #[test]
    fn heading_text_accessor_returns_text() {
        let heading = summary_heading();
        assert_eq!(
            heading.text(),
            "Summary",
            "Heading text should be 'Summary'"
        );
    }

    #[test]
    fn heading_position_accessor_returns_position() {
        let heading = summary_heading();
        assert_eq!(
            heading.position(),
            SourceByteOffset::from(22u32),
            "Heading position should be 22"
        );
    }

    #[test]
    fn new_heading_sets_level() {
        let heading = implementation_heading();
        assert_eq!(heading.level().as_u8(), 2, "Heading level should be 2");
    }

    #[test]
    fn new_heading_sets_text() {
        let heading = implementation_heading();
        assert_eq!(
            heading.text(),
            "Implementation",
            "Heading text should be 'Implementation'"
        );
    }

    #[test]
    fn new_heading_sets_position() {
        let heading = implementation_heading();
        assert_eq!(
            heading.position(),
            SourceByteOffset::from(10u32),
            "Heading position should be 10"
        );
    }

    #[test]
    fn heading_level_validation_rejects_invalid_values() {
        let level = 7;
        let result = HeadingLevel::try_new(level);
        assert!(
            result.is_err(),
            "Invalid heading level (7) should be rejected"
        );
    }

    #[test]
    fn new_returns_error_for_empty_text() {
        let level = HeadingLevel::try_new(1).unwrap();
        let text: String = "   ".into();
        let pos = SourceByteOffset::from(0u32);
        let result = Heading::try_new(level, text, pos);
        assert!(
            matches!(result, Err(HeadingError::EmptyContent)),
            "Empty heading text should be rejected, got: {result:?}"
        );
    }
}
