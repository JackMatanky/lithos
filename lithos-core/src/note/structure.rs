//! Document structure subentities for Note aggregate.
//!
//! Provides heading-based organization and section content management for
//! notes. Headings (H1-H6) mark structural points in the document, while
//! sections group content between headings.

use super::error::NoteError;

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
    level: u8,
    /// Heading text content.
    text: Box<str>,
    /// Character position in the source document.
    position: usize,
}

impl Heading {
    /// Returns the heading level.
    #[inline]
    #[must_use]
    pub const fn level(&self) -> u8 {
        self.level
    }

    /// Creates a new heading with validation.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::note::structure::Heading;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let heading = Heading::new(2, "Implementation".to_string(), 10)?;
    /// assert_eq!(heading.level(), 2);
    /// assert_eq!(heading.text(), "Implementation");
    /// assert_eq!(heading.position(), 10);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns `NoteError::ValidationFailed` if `level` is not between 1
    /// and 6.
    #[inline]
    pub fn new(
        level: u8,
        text: String,
        position: usize,
    ) -> Result<Self, NoteError> {
        if !(1..=6).contains(&level) {
            return Err(NoteError::ValidationFailed(format!(
                "Invalid heading level: {level} (must be 1-6)"
            )));
        }

        if text.trim().is_empty() {
            return Err(NoteError::ValidationFailed(
                "Heading text cannot be empty".to_owned(),
            ));
        }

        Ok(Self {
            level,
            text: text.into(),
            position,
        })
    }

    /// Returns the character position in the source document.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> usize {
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
    range: std::ops::Range<usize>,
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
    /// use lithos_core::note::structure::{Heading, Section};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let range = 10..50;
    /// let section = Section::new(
    ///     Some(Heading::new(1, "Title".to_string(), 10)?),
    ///     "Content here...".to_string(),
    ///     range.clone(),
    /// );
    /// assert_eq!(section.range(), range);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn new(
        heading: Option<Heading>,
        content: String,
        range: std::ops::Range<usize>,
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
    pub fn range(&self) -> std::ops::Range<usize> {
        self.range.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[expect(
        clippy::disallowed_methods,
        reason = "Fixture helpers use expect for deterministic setup."
    )]
    mod fixtures {
        use super::*;

        pub fn summary_heading() -> Heading {
            Heading::new(3, "Summary".to_owned(), 22)
                .expect("Failed to create heading fixture")
        }

        pub fn implementation_heading() -> Heading {
            Heading::new(2, "Implementation".to_owned(), 10)
                .expect("Valid heading should be created")
        }

        pub fn intro_heading() -> Heading {
            Heading::new(1, "Intro".to_owned(), 0)
                .expect("Failed to create heading fixture")
        }

        pub fn section_with_intro() -> Section {
            Section::new(Some(intro_heading()), "Body".to_owned(), 0..4)
        }

        pub fn section_with_title()
        -> (Section, Option<Heading>, std::ops::Range<usize>) {
            let heading = Some(
                Heading::new(1, "Title".into(), 0)
                    .expect("Failed to create heading fixture"),
            );
            let range = 0..15;
            let section = Section::new(
                heading.clone(),
                "Section content".to_owned(),
                range.clone(),
            );
            (section, heading, range)
        }
    }

    mod heading {
        use super::*;

        #[test]
        fn heading_level_accessor_returns_level() {
            let heading = fixtures::summary_heading();
            assert_eq!(heading.level(), 3, "Heading level should be 3");
        }

        #[test]
        fn heading_text_accessor_returns_text() {
            let heading = fixtures::summary_heading();
            assert_eq!(
                heading.text(),
                "Summary",
                "Heading text should be 'Summary'"
            );
        }

        #[test]
        fn heading_position_accessor_returns_position() {
            let heading = fixtures::summary_heading();
            assert_eq!(heading.position(), 22, "Heading position should be 22");
        }

        #[test]
        fn new_heading_sets_level() {
            let heading = fixtures::implementation_heading();
            assert_eq!(heading.level(), 2, "Heading level should be 2");
        }

        #[test]
        fn new_heading_sets_text() {
            let heading = fixtures::implementation_heading();
            assert_eq!(
                heading.text(),
                "Implementation",
                "Heading text should be 'Implementation'"
            );
        }

        #[test]
        fn new_heading_sets_position() {
            let heading = fixtures::implementation_heading();
            assert_eq!(heading.position(), 10, "Heading position should be 10");
        }

        #[test]
        fn new_returns_error_for_invalid_level() {
            // GIVEN: an invalid heading level
            let level = 7;
            let text = "Invalid".to_owned();

            // WHEN: creating a new heading
            let result = Heading::new(level, text, 0);

            // THEN: it returns InvalidHeadingLevel
            assert!(
                matches!(result, Err(NoteError::ValidationFailed(_))),
                "Invalid heading level (7) should be rejected, got: {result:?}"
            );
        }

        #[test]
        fn new_returns_error_for_empty_text() {
            // GIVEN: empty heading text
            let level = 1;
            let text = "   ".to_owned();

            // WHEN: creating a new heading
            let result = Heading::new(level, text, 0);

            // THEN: it returns ValidationFailed
            assert!(
                matches!(result, Err(NoteError::ValidationFailed(_))),
                "Empty heading text should be rejected, got: {result:?}"
            );
        }
    }

    mod section {
        use super::*;

        #[test]
        fn section_content_accessor_returns_content() {
            let section = fixtures::section_with_intro();
            assert_eq!(
                section.content(),
                "Body",
                "Section content should be 'Body'"
            );
        }

        #[test]
        fn section_heading_accessor_returns_heading() {
            let section = fixtures::section_with_intro();
            assert!(
                matches!(section.heading(), Some(heading) if heading.text() == "Intro"),
                "Section heading text should be 'Intro'"
            );
        }

        #[test]
        fn section_range_accessor_returns_range() {
            let section = fixtures::section_with_intro();
            assert_eq!(section.range(), 0..4, "Section range should be 0..4");
        }

        #[test]
        fn new_section_sets_heading() {
            let (section, heading, _range) = fixtures::section_with_title();
            assert_eq!(
                section.heading(),
                heading.as_ref(),
                "Section heading should match input"
            );
        }

        #[test]
        fn new_section_sets_content() {
            let (section, _heading, _range) = fixtures::section_with_title();
            assert_eq!(
                section.content(),
                "Section content",
                "Section content should match input"
            );
        }

        #[test]
        fn new_section_sets_range() {
            let (section, _heading, range) = fixtures::section_with_title();
            assert_eq!(
                section.range(),
                range,
                "Section range should match input"
            );
        }
    }
}
