//! Section subentity for Note aggregate.
//!
//! Represents content sections organized by headings within notes.

use crate::models::heading::Heading;

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
    /// use lithos_domain::models::section::Section;
    /// use lithos_domain::models::heading::Heading;
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
