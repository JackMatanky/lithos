//! Heading subentity for Note aggregate.
//!
//! Represents document structure headings within notes.

use crate::errors::DomainError;

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
    /// use lithos_domain::models::heading::Heading;
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
