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
        let tag_path = extract_tag_path(input)?;
        let segments = split_tag_segments(tag_path)?;
        validate_tag_segments(&segments)?;

        Ok(Self {
            full_path: tag_path.into(),
            segments: segments.into_iter().map(Into::into).collect(),
        })
    }
}

/// Extracts the tag path by removing the `#` prefix and validating format.
///
/// # Errors
/// - Returns `DomainError::InvalidTag` if input doesn't start with `#`.
/// - Returns `DomainError::InvalidTag` if tag is empty after removing `#`.
#[inline]
fn extract_tag_path(input: &str) -> Result<&str, DomainError> {
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
    let tag_path = &input[1..];

    if tag_path.is_empty() {
        return Err(DomainError::InvalidTag("Tag cannot be empty".to_owned()));
    }

    Ok(tag_path)
}

/// Splits a tag path into segments and validates no empty segments exist.
///
/// # Errors
/// - Returns `DomainError::EmptyTagSegment` if any segment is empty (double slashes).
#[inline]
fn split_tag_segments(tag_path: &str) -> Result<Vec<&str>, DomainError> {
    let segments: Vec<&str> = tag_path.split('/').collect();

    // Check for empty segments (double slashes like "work//urgent")
    if segments.iter().any(|s| s.is_empty()) {
        return Err(DomainError::EmptyTagSegment);
    }

    Ok(segments)
}

/// Validates that all tag segments contain only allowed characters.
///
/// Allowed characters: alphanumeric, underscore (`_`), hyphen (`-`).
///
/// # Errors
/// - Returns `DomainError::InvalidTag` if any segment contains invalid characters.
#[inline]
fn validate_tag_segments(segments: &[&str]) -> Result<(), DomainError> {
    for segment in segments {
        if !is_valid_tag_segment(segment) {
            return Err(DomainError::InvalidTag(format!(
                "Invalid tag segment '{segment}': only alphanumeric, underscore, and hyphen allowed"
            )));
        }
    }
    Ok(())
}

/// Checks if a tag segment contains only valid characters.
///
/// Valid characters: alphanumeric, underscore (`_`), hyphen (`-`).
#[inline]
fn is_valid_tag_segment(segment: &str) -> bool {
    segment.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}
