//! Document structure subentities for Note aggregate.
//!
//! Provides heading-based organization and section content management for
//! notes. Headings (H1-H6) mark structural points in the document, while
//! sections group content between headings.

use super::error::NoteError;
use crate::note::types::{HeadingLevel, SourceByteOffset, SourceByteRange};

/// Represents a heading within a note.
///
/// Headings provide document structure and are used to generate
/// table of contents and section organization.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Heading {
    /// Heading level (1-6, corresponding to # through ######).
    level: HeadingLevel,
    /// Heading text content.
    text: Box<str>,
    /// Character position in the source document.
    position: SourceByteOffset,
}

impl Heading {
    /// Returns the heading level.
    #[inline]
    #[must_use]
    pub const fn level(&self) -> HeadingLevel {
        self.level
    }

    /// Creates a new heading with validation.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::note::{
    ///     structure::Heading,
    ///     types::{HeadingLevel, SourceByteOffset},
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let level = HeadingLevel::try_new(2)?;
    /// let pos = SourceByteOffset::from(10u32);
    /// let heading = Heading::new(level, "Implementation".to_string(), pos)?;
    /// assert_eq!(heading.level().as_u8(), 2, "Heading level should match");
    /// assert_eq!(heading.text(), "Implementation", "Heading text should match");
    /// assert_eq!(heading.position(), pos, "Heading position should match");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns `NoteError::ValidationFailed` if heading text is empty.
    #[inline]
    pub fn new<T: Into<Box<str>>>(
        level: HeadingLevel,
        text: T,
        position: SourceByteOffset,
    ) -> Result<Self, NoteError> {
        let text = text.into();
        if text.trim().is_empty() {
            return Err(NoteError::ValidationFailed(
                "Heading text cannot be empty".to_owned(),
            ));
        }

        Ok(Self {
            level,
            text,
            position,
        })
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
        &self.text
    }
}

/// Represents a content section within a note.
///
/// Sections organize note content between headings, providing
/// structural organization for large documents.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Section {
    /// Optional heading that starts this section (None for content before
    /// first heading).
    heading: Option<Heading>,
    /// Section content text.
    content: Box<str>,
    /// Character range in the source document.
    range: SourceByteRange,
}

impl Section {
    /// Returns the section content text.
    #[inline]
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns the optional heading that starts this section.
    #[inline]
    #[must_use]
    pub const fn heading(&self) -> Option<&Heading> {
        self.heading.as_ref()
    }

    /// Creates a new section.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::note::{
    ///     structure::{Heading, Section},
    ///     types::{HeadingLevel, SourceByteOffset, SourceByteRange},
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let level = HeadingLevel::try_new(1)?;
    /// let pos = SourceByteOffset::from(10u32);
    /// let range = SourceByteRange::new(pos, SourceByteOffset::from(50u32));
    /// let section = Section::new(
    ///     Some(Heading::new(level, "Title", pos)?),
    ///     "Content here...",
    ///     range,
    /// );
    /// assert_eq!(section.range(), range, "Section range should match");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn new<T: Into<Box<str>>>(
        heading: Option<Heading>,
        content: T,
        range: SourceByteRange,
    ) -> Self {
        Self {
            heading,
            content: content.into(),
            range,
        }
    }

    /// Returns the character range in the source document.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Tests use assertions in Result-returning functions."
)]
mod tests {
    use super::*;

    mod fixtures {
        use super::*;

        pub fn summary_heading() -> Result<Heading, NoteError> {
            Heading::new(
                HeadingLevel::try_new(3)?,
                "Summary".to_owned(),
                SourceByteOffset::from(22u32),
            )
        }

        pub fn implementation_heading() -> Result<Heading, NoteError> {
            Heading::new(
                HeadingLevel::try_new(2)?,
                "Implementation".to_owned(),
                SourceByteOffset::from(10u32),
            )
        }

        pub fn intro_heading() -> Result<Heading, NoteError> {
            Heading::new(
                HeadingLevel::try_new(1)?,
                "Intro".to_owned(),
                SourceByteOffset::from(0u32),
            )
        }

        pub fn section_with_intro() -> Result<Section, NoteError> {
            let start = SourceByteOffset::from(0u32);
            let end = SourceByteOffset::from(4u32);
            Ok(Section::new(
                Some(intro_heading()?),
                "Body".to_owned(),
                SourceByteRange::new(start, end),
            ))
        }

        #[expect(
            clippy::type_complexity,
            reason = "Fixture returns a complex tuple for test setup \
                      convenience."
        )]
        pub fn section_with_title()
        -> Result<(Section, Option<Heading>, SourceByteRange), NoteError>
        {
            let heading = Some(Heading::new(
                HeadingLevel::try_new(1)?,
                "Title",
                SourceByteOffset::from(0u32),
            )?);
            let range = SourceByteRange::new(
                SourceByteOffset::from(0u32),
                SourceByteOffset::from(15u32),
            );
            let section =
                Section::new(heading.clone(), "Section content", range);
            Ok((section, heading, range))
        }
    }

    mod heading {
        use super::*;

        #[test]
        fn heading_level_accessor_returns_level() -> Result<(), NoteError> {
            let heading = fixtures::summary_heading()?;
            assert_eq!(heading.level().as_u8(), 3, "Heading level should be 3");
            Ok(())
        }

        #[test]
        fn heading_text_accessor_returns_text() -> Result<(), NoteError> {
            let heading = fixtures::summary_heading()?;
            assert_eq!(
                heading.text(),
                "Summary",
                "Heading text should be 'Summary'"
            );
            Ok(())
        }

        #[test]
        fn heading_position_accessor_returns_position() -> Result<(), NoteError>
        {
            let heading = fixtures::summary_heading()?;
            assert_eq!(
                heading.position(),
                SourceByteOffset::from(22u32),
                "Heading position should be 22"
            );
            Ok(())
        }

        #[test]
        fn new_heading_sets_level() -> Result<(), NoteError> {
            let heading = fixtures::implementation_heading()?;
            assert_eq!(heading.level().as_u8(), 2, "Heading level should be 2");
            Ok(())
        }

        #[test]
        fn new_heading_sets_text() -> Result<(), NoteError> {
            let heading = fixtures::implementation_heading()?;
            assert_eq!(
                heading.text(),
                "Implementation",
                "Heading text should be 'Implementation'"
            );
            Ok(())
        }

        #[test]
        fn new_heading_sets_position() -> Result<(), NoteError> {
            let heading = fixtures::implementation_heading()?;
            assert_eq!(
                heading.position(),
                SourceByteOffset::from(10u32),
                "Heading position should be 10"
            );
            Ok(())
        }

        #[test]
        fn heading_level_validation_rejects_invalid_values() {
            // GIVEN: an invalid heading level
            let level = 7;

            // WHEN: creating a heading level
            let result = HeadingLevel::try_new(level);

            // THEN: it returns an error
            assert!(
                result.is_err(),
                "Invalid heading level (7) should be rejected"
            );
        }

        #[test]
        fn new_returns_error_for_empty_text() -> Result<(), NoteError> {
            // GIVEN: empty heading text
            let level = HeadingLevel::try_new(1)?;
            let text = "   ".to_owned();
            let pos = SourceByteOffset::from(0u32);

            // WHEN: creating a new heading
            let result = Heading::new(level, text, pos);

            // THEN: it returns ValidationFailed
            assert!(
                matches!(result, Err(NoteError::ValidationFailed(_))),
                "Empty heading text should be rejected, got: {result:?}"
            );
            Ok(())
        }
    }

    mod section {
        use super::*;

        #[test]
        fn section_content_accessor_returns_content() -> Result<(), NoteError> {
            let section = fixtures::section_with_intro()?;
            assert_eq!(
                section.content(),
                "Body",
                "Section content should be 'Body'"
            );
            Ok(())
        }

        #[test]
        fn section_heading_accessor_returns_heading() -> Result<(), NoteError> {
            let section = fixtures::section_with_intro()?;
            assert!(
                matches!(section.heading(), Some(heading) if heading.text() == "Intro"),
                "Section heading text should be 'Intro'"
            );
            Ok(())
        }

        #[test]
        fn section_range_accessor_returns_range() -> Result<(), NoteError> {
            let section = fixtures::section_with_intro()?;
            let expected_range = SourceByteRange::new(
                SourceByteOffset::from(0u32),
                SourceByteOffset::from(4u32),
            );
            assert_eq!(
                section.range(),
                expected_range,
                "Section range should be 0..4"
            );
            Ok(())
        }

        #[test]
        fn new_section_sets_heading() -> Result<(), NoteError> {
            let (section, heading, _range) = fixtures::section_with_title()?;
            assert_eq!(
                section.heading(),
                heading.as_ref(),
                "Section heading should match input"
            );
            Ok(())
        }

        #[test]
        fn new_section_sets_content() -> Result<(), NoteError> {
            let (section, _heading, _range) = fixtures::section_with_title()?;
            assert_eq!(
                section.content(),
                "Section content",
                "Section content should match input"
            );
            Ok(())
        }

        #[test]
        fn new_section_sets_range() -> Result<(), NoteError> {
            let (section, _heading, range) = fixtures::section_with_title()?;
            assert_eq!(
                section.range(),
                range,
                "Section range should match input"
            );
            Ok(())
        }
    }
}
