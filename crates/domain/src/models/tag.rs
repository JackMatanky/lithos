//! Tag subentity for Note aggregate.
//!
//! Represents hierarchical tags used for note organization.

use crate::errors::DomainError;

/// Represents a hierarchical tag with segments.
///
/// Tags follow the format `#segment1/segment2/segment3` and are used
/// for organizing and categorizing notes.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Tag {
    /// Full tag path (e.g., "work/project/urgent").
    pub full_path: Box<str>,
    /// Individual path segments.
    pub segments: Vec<Box<str>>,
}

impl Tag {
    /// Parses a tag string into a Tag struct.
    ///
    /// # Tag Format
    /// - Must start with `#`
    /// - Segments separated by `/`
    /// - Segments must match regex `^[a-zA-Z0-9_-]+$`
    /// - No empty segments allowed
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::tag::Tag;
    ///
    /// let tag = Tag::parse("#work/project/urgent").unwrap();
    /// assert_eq!(tag.full_path.as_ref(), "work/project/urgent");
    /// assert_eq!(tag.segments, vec!["work".into(), "project".into(), "urgent".into()]);
    ///
    /// let simple_tag = Tag::parse("#personal").unwrap();
    /// assert_eq!(simple_tag.full_path.as_ref(), "personal");
    /// assert_eq!(simple_tag.segments, vec!["personal".into()]);
    /// ```
    ///
    /// # Errors
    /// - Returns `DomainError::InvalidTag` if the input doesn't start with `#`.
    /// - Returns `DomainError::InvalidTag` if the tag is empty after removing `#`.
    /// - Returns `DomainError::EmptyTagSegment` if any segment is empty.
    /// - Returns `DomainError::InvalidTag` if any segment contains invalid characters.
    #[inline]
    pub fn parse(input: &str) -> Result<Self, DomainError> {
        if !input.starts_with('#') {
            return Err(DomainError::InvalidTag(
                "Tag must start with #".to_owned(),
            ));
        }

        // Remove the # prefix
        #[expect(
            clippy::string_slice,
            reason = "Safe because we check input.starts_with('#') above"
        )]
        let input = &input[1..];

        if input.is_empty() {
            return Err(DomainError::InvalidTag(
                "Tag cannot be empty".to_owned(),
            ));
        }

        // Split by '/' and validate each segment
        let segments: Vec<&str> = input.split('/').collect();

        // Check for empty segments (double slashes)
        if segments.iter().any(|s| s.is_empty()) {
            return Err(DomainError::EmptyTagSegment);
        }

        // Validate each segment
        for segment in &segments {
            if !segment
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                return Err(DomainError::InvalidTag(format!(
                    "Invalid tag segment '{segment}': only alphanumeric, underscore, and hyphen allowed"
                )));
            }
        }

        let full_path = input.to_owned();
        let segments =
            segments.into_iter().map(|s| s.to_owned().into()).collect();

        Ok(Self {
            full_path: full_path.into(),
            segments,
        })
    }
}
