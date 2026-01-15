//! Document structure subentities for Note aggregate.
//!
//! Provides heading-based organization and section content management for notes.
//! Headings (H1-H6) mark structural points in the document, while sections group
//! content between headings.

use crate::errors::DomainError;

// ============================================================================
// Heading
// ============================================================================

/// Represents a heading within a note.
///
/// Headings provide document structure and are used to generate
/// table of contents and section organization.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "level is primary data, text and position are secondary"
)]
pub struct Heading {
    /// Heading level (1-6, corresponding to # through ######).
    pub level: u8,
    /// Heading text content.
    pub text: Box<str>,
    /// Character position in the source document.
    pub position: usize,
}

impl Heading {
    /// Creates a new heading with validation.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::structure::Heading;
    ///
    /// let heading = Heading::new(2, "Implementation".to_string(), 10).unwrap();
    /// assert_eq!(heading.level, 2);
    /// assert_eq!(heading.text.as_ref(), "Implementation");
    /// assert_eq!(heading.position, 10);
    /// ```
    ///
    /// # Errors
    /// Returns `DomainError::InvalidHeadingLevel` if `level` is not between 1 and 6.
    #[inline]
    pub fn new(
        level: u8,
        text: String,
        position: usize,
    ) -> Result<Self, DomainError> {
        if !(1..=6).contains(&level) {
            return Err(DomainError::InvalidHeadingLevel(level));
        }

        if text.trim().is_empty() {
            return Err(DomainError::ValidationFailed(
                "Heading text cannot be empty".to_owned(),
            ));
        }

        Ok(Self {
            level,
            text: text.into(),
            position,
        })
    }
}

// ============================================================================
// Section
// ============================================================================

/// Represents a content section within a note.
///
/// Sections organize note content between headings, providing
/// structural organization for large documents.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Section {
    /// Section content text.
    pub content: Box<str>,
    /// Optional heading that starts this section (None for content before first heading).
    pub heading: Option<Heading>,
    /// Character range in the source document.
    pub range: std::ops::Range<usize>,
}

impl Section {
    /// Creates a new section.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::structure::{Section, Heading};
    ///
    /// let range = 10..50;
    /// let section = Section::new(
    ///     Some(Heading::new(1, "Title".to_string(), 10).unwrap()),
    ///     "Content here...".to_string(),
    ///     range.clone()
    /// );
    /// assert_eq!(section.range, range);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    mod heading {
        use super::*;

        #[test]
        fn new_succeeds_for_valid_input() {
            // GIVEN valid heading parameters
            let level = 2;
            let text = "Implementation".to_owned();
            let position = 10;

            // WHEN creating a new heading
            let result = Heading::new(level, text, position).unwrap();

            // THEN it has the correct values
            assert_eq!(result.level, 2);
            assert_eq!(result.text.as_ref(), "Implementation");
            assert_eq!(result.position, 10);
        }

        #[test]
        fn new_returns_error_for_invalid_level() {
            // GIVEN an invalid heading level
            let level = 7;
            let text = "Invalid".to_owned();

            // WHEN creating a new heading
            let result = Heading::new(level, text, 0);

            // THEN it returns InvalidHeadingLevel
            assert!(matches!(result, Err(DomainError::InvalidHeadingLevel(7))));
        }

        #[test]
        fn new_returns_error_for_empty_text() {
            // GIVEN empty heading text
            let level = 1;
            let text = "   ".to_owned();

            // WHEN creating a new heading
            let result = Heading::new(level, text, 0);

            // THEN it returns ValidationFailed
            assert!(matches!(result, Err(DomainError::ValidationFailed(_))));
        }
    }

    mod section {
        use super::*;

        #[test]
        fn new_succeeds_for_valid_input() {
            // GIVEN valid section parameters
            let heading = Some(Heading::new(1, "Title".into(), 0).unwrap());
            let content = "Section content".to_owned();
            let range = 0..15;

            // WHEN creating a new section
            let result = Section::new(heading.clone(), content, range.clone());

            // THEN it has the correct values
            assert_eq!(result.heading, heading);
            assert_eq!(result.content.as_ref(), "Section content");
            assert_eq!(result.range, range);
        }
    }
}
