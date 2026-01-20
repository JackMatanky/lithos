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
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "pub(crate) used for internal builders and tests"
)]
pub struct Tag {
    /// Full tag path (e.g., "work/project/urgent").
    pub(crate) full_path: Box<str>,
    /// Individual path segments.
    pub(crate) segments: Vec<Box<str>>,
}

impl Tag {
    /// Returns the full tag path without the leading `#`.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.full_path
    }

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
    /// use lithos_domain::Tag;
    ///
    /// let tag = Tag::parse("#work/project/urgent").unwrap();
    /// assert_eq!(tag.as_str(), "work/project/urgent");
    /// assert_eq!(tag.segments(), &[ "work".into(), "project".into(), "urgent".into() ]);
    ///
    /// let simple_tag = Tag::parse("#personal").unwrap();
    /// assert_eq!(simple_tag.as_str(), "personal");
    /// assert_eq!(simple_tag.segments(), &["personal".into()]);
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

    /// Returns the individual path segments.
    #[inline]
    #[must_use]
    pub fn segments(&self) -> &[Box<str>] {
        &self.segments
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

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Unit tests use unwrap for readability"
)]
mod tests {
    use super::*;

    mod parse {
        use rstest::rstest;

        use super::*;

        #[test]
        fn accessors_return_expected_values() {
            // GIVEN: a parsed tag
            let tag = Tag::parse("#work/project").unwrap();

            // THEN: accessors expose path and segments
            assert_eq!(tag.as_str(), "work/project");
            assert_eq!(tag.segments(), &["work".into(), "project".into()]);
        }

        /// 3.2-UNIT-001: Tag Parsing Matrix.
        /// Priority: P0.
        #[rstest]
        #[case::simple("#personal", Ok(vec!["personal"]))]
        #[case::hierarchical("#work/project/urgent", Ok(vec!["work", "project", "urgent"]))]
        #[case::missing_hash(
            "invalid",
            Err(DomainError::InvalidTag("Tag must start with #".to_owned()))
        )]
        #[case::only_hash("#", Err(DomainError::InvalidTag("Tag cannot be empty".to_owned())))]
        #[case::empty_segments(
            "#work//urgent",
            Err(DomainError::EmptyTagSegment)
        )]
        #[case::invalid_chars(
            "#work project",
            Err(DomainError::InvalidTag("Invalid tag segment 'work project': only alphanumeric, underscore, and hyphen allowed".to_owned()))
        )]
        fn tag_parsing_matrix(
            #[case] input: &str,
            #[case] expected: Result<Vec<&str>, DomainError>,
        ) {
            // GIVEN: a tag string from the parsing matrix
            // WHEN: parsing the tag
            let result = Tag::parse(input);

            // THEN: the result matches the expected outcome
            match expected {
                Ok(segments) => {
                    let tag = result.unwrap();
                    let actual_segments: Vec<&str> =
                        tag.segments().iter().map(AsRef::as_ref).collect();
                    assert_eq!(actual_segments, segments);
                }
                Err(e) => {
                    let actual = result.unwrap_err();
                    assert_eq!(
                        std::mem::discriminant(&actual),
                        std::mem::discriminant(&e),
                        "Tag '{input}' produced wrong error variant"
                    );
                }
            }
        }
    }

    #[cfg(test)]
    mod proptests {
        use proptest::prelude::*;

        use super::*;

        proptest! {
            /// 3.2-PROP-004: Tag Segment Validation Fuzzing.
            #[test]
            fn rejects_invalid_characters_in_segments(
                s in "#[a-zA-Z0-9_-]*/[ !@#$%^&*()]+/[a-zA-Z0-9_-]*"
            ) {
                let result = Tag::parse(&s);
                prop_assert!(
                    result.is_err(),
                    "Tags with special characters in segments should be rejected: {}",
                    s
                );
            }

            /// 3.2-PROP-005: Valid Tag Fuzzing.
            #[test]
            fn accepts_valid_alphanumeric_tags(
                s in "#[a-zA-Z0-9_-]+(/[a-zA-Z0-9_-]+)*"
            ) {
                let result = Tag::parse(&s);
                prop_assert!(
                    result.is_ok(),
                    "Valid alphanumeric tags should be accepted: {}",
                    s
                );
            }
        }
    }
}
