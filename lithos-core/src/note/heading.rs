//! Heading entities for document structure.
//!
//! Provides heading levels, text validation, and parsed heading values for
//! markdown documents.

use super::error::{LinkError, NoteError, NoteMetadataError};
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

/// Builder for accumulating heading data during parsing.
#[derive(Debug)]
pub(crate) struct HeadingAccumulator {
    level: HeadingLevel,
    text: String,
    position: SourceByteOffset,
}

impl HeadingAccumulator {
    #[inline]
    pub(crate) fn new(level: HeadingLevel, position: SourceByteOffset) -> Self {
        Self {
            level,
            text: String::new(),
            position,
        }
    }

    #[inline]
    pub(crate) fn push_text(&mut self, text: &str) {
        self.text.push_str(text);
    }

    #[inline]
    pub(crate) fn push_break(&mut self) {
        self.text.push(' ');
    }

    #[inline]
    pub(crate) fn build(self) -> Result<Heading, NoteError> {
        Heading::try_new(self.level, self.text, self.position)
    }
}

impl Heading {
    /// Creates a new heading with validation.
    ///
    /// # Errors
    /// Returns `NoteError::Metadata` if heading text is empty.
    #[inline]
    pub fn try_new<T: Into<Box<str>>>(
        level: HeadingLevel,
        text: T,
        position: SourceByteOffset,
    ) -> Result<Self, NoteError> {
        let text = HeadingText::try_from_boxed(text.into())?;

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

/// Heading level (1-6).
///
/// # Errors
///
/// Returns [`NoteError::Structure`] if the level is not between 1 and 6.
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
    pub fn try_new(level: u8) -> Result<Self, NoteError> {
        if (1..=6).contains(&level) {
            Ok(Self(level))
        } else {
            Err(NoteError::Structure(
                "invalid heading level: must be between 1 and 6",
            ))
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
    /// Returns [`NoteError::Metadata`] if the text is empty.
    #[inline]
    pub fn try_new(value: &str) -> Result<Self, NoteError> {
        Self::try_from_boxed(value.into())
    }

    /// Creates a validated heading text value for link anchors.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Link`] if the text is empty.
    #[inline]
    pub fn try_new_anchor(value: &str) -> Result<Self, NoteError> {
        let text = value.trim();
        if text.is_empty() {
            return Err(NoteError::Link(LinkError::EmptyHeadingAnchor));
        }
        Ok(Self(text.into()))
    }

    /// Creates a validated heading text value from a boxed string.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Metadata`] if the text is empty.
    #[inline]
    pub fn try_from_boxed(value: Box<str>) -> Result<Self, NoteError> {
        if value.trim().is_empty() {
            return Err(NoteError::Metadata(
                NoteMetadataError::HeadingTextEmpty,
            ));
        }
        Ok(Self(value))
    }

    /// Returns the underlying text as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
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
            matches!(
                result,
                Err(NoteError::Metadata(NoteMetadataError::HeadingTextEmpty))
            ),
            "Empty heading text should be rejected, got: {result:?}"
        );
    }
}
