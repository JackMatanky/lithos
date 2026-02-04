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
    ///
    /// let heading = Heading::new(2, "Implementation".to_string(), 10).unwrap();
    /// assert_eq!(heading.level(), 2);
    /// assert_eq!(heading.text(), "Implementation");
    /// assert_eq!(heading.position(), 10);
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
    ///
    /// let range = 10..50;
    /// let section = Section::new(
    ///     Some(Heading::new(1, "Title".to_string(), 10).unwrap()),
    ///     "Content here...".to_string(),
    ///     range.clone(),
    /// );
    /// assert_eq!(section.range(), range);
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

    mod heading {
        use super::*;

        #[test]
        fn accessors_return_expected_values() -> Result<(), String> {
            // GIVEN: a heading
            let heading =
                Heading::new(3, "Summary".to_owned(), 22).map_err(|e| {
                    format!("Failed to create heading fixture: {e}")
                })?;

            // THEN: accessors expose fields
            if heading.level() != 3 {
                return Err(format!(
                    "Heading level should be 3, got {}",
                    heading.level()
                ));
            }
            if heading.text() != "Summary" {
                return Err(format!(
                    "Heading text should be 'Summary', got '{}'",
                    heading.text()
                ));
            }
            if heading.position() != 22 {
                return Err(format!(
                    "Heading position should be 22, got {}",
                    heading.position()
                ));
            }

            Ok(())
        }

        #[test]
        fn new_succeeds_for_valid_input() -> Result<(), String> {
            // GIVEN: valid heading parameters
            let level = 2;
            let text = "Implementation".to_owned();
            let position = 10;

            // WHEN: creating a new heading
            let result = Heading::new(level, text, position)
                .map_err(|e| format!("Valid heading should be created: {e}"))?;

            // THEN: it has the correct values
            if result.level() != 2 {
                return Err(format!(
                    "Heading level should be 2, got {}",
                    result.level()
                ));
            }
            if result.text() != "Implementation" {
                return Err(format!(
                    "Heading text should be 'Implementation', got '{}'",
                    result.text()
                ));
            }
            if result.position() != 10 {
                return Err(format!(
                    "Heading position should be 10, got {}",
                    result.position()
                ));
            }

            Ok(())
        }

        #[test]
        fn new_returns_error_for_invalid_level() -> Result<(), String> {
            // GIVEN: an invalid heading level
            let level = 7;
            let text = "Invalid".to_owned();

            // WHEN: creating a new heading
            let result = Heading::new(level, text, 0);

            // THEN: it returns InvalidHeadingLevel
            if !matches!(result, Err(NoteError::ValidationFailed(_))) {
                return Err(format!(
                    "Invalid heading level (7) should be rejected, got: \
                     {result:?}"
                ));
            }

            Ok(())
        }

        #[test]
        fn new_returns_error_for_empty_text() -> Result<(), String> {
            // GIVEN: empty heading text
            let level = 1;
            let text = "   ".to_owned();

            // WHEN: creating a new heading
            let result = Heading::new(level, text, 0);

            // THEN: it returns ValidationFailed
            if !matches!(result, Err(NoteError::ValidationFailed(_))) {
                return Err(format!(
                    "Empty heading text should be rejected, got: {result:?}"
                ));
            }

            Ok(())
        }
    }

    mod section {
        use super::*;

        #[test]
        fn accessors_return_expected_values() -> Result<(), String> {
            // GIVEN: a section with heading
            let heading =
                Heading::new(1, "Intro".to_owned(), 0).map_err(|e| {
                    format!("Failed to create heading fixture: {e}")
                })?;
            let section =
                Section::new(Some(heading.clone()), "Body".to_owned(), 0..4);

            // THEN: accessors return expected values
            if section.content() != "Body" {
                return Err(format!(
                    "Section content should be 'Body', got '{}'",
                    section.content()
                ));
            }
            let Some(section_heading) = section.heading() else {
                return Err("Section heading should be present".to_owned());
            };
            if section_heading.text() != "Intro" {
                return Err(format!(
                    "Section heading text should be 'Intro', got '{}'",
                    section_heading.text()
                ));
            }
            if section.range() != (0..4) {
                return Err(format!(
                    "Section range should be 0..4, got {:?}",
                    section.range()
                ));
            }

            Ok(())
        }

        #[test]
        fn new_succeeds_for_valid_input() -> Result<(), String> {
            // GIVEN: valid section parameters
            let heading =
                Some(Heading::new(1, "Title".into(), 0).map_err(|e| {
                    format!("Failed to create heading fixture: {e}")
                })?);
            let content = "Section content".to_owned();
            let range = 0..15;

            // WHEN: creating a new section
            let result = Section::new(heading.clone(), content, range.clone());

            // THEN: it has the correct values
            if result.heading() != heading.as_ref() {
                return Err("Section heading should match input".to_owned());
            }
            if result.content() != "Section content" {
                return Err(format!(
                    "Section content should match input, got '{}'",
                    result.content()
                ));
            }
            if result.range() != range {
                return Err(format!(
                    "Section range should match input, got {:?}",
                    result.range()
                ));
            }

            Ok(())
        }
    }
}
