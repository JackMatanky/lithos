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
