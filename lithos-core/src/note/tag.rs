//! Hierarchical tag entity for note organization.
//!
//! Supports multi-segment tags (e.g., `#work/project`) with validated
//! path segments and efficient string representations.

//! Tag subentity for Note aggregate.
//!
//! Represents hierarchical tags used for note organization.
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use std::ops::Deref;

use rkyv::{Archive, Deserialize, Serialize};

use super::error::{NoteError, TagError};

/// Represents a hierarchical tag with segments.
///
/// Tags follow the format `#segment1/segment2/segment3` and are used
/// for organizing and categorizing notes.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::tag::Tag;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let tag = Tag::new("#work/project/urgent")?;
/// assert_eq!(tag.full_path(), "work/project/urgent");
/// assert_eq!(tag.segments().count(), 3);
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Tag {
    /// Full tag path (without leading `#`).
    full_path: TagPath,
    /// Individual path segments.
    segments: Segments,
}

impl Tag {
    /// Creates a new `Tag` from a raw tag string.
    ///
    /// # Tag Format
    /// - Must start with `#`
    /// - Segments separated by `/`
    /// - Segments must match alphanumeric, underscore, or hyphen
    /// - No empty segments allowed
    ///
    /// # Errors
    /// Returns [`NoteError::Tag`] if validation fails.
    #[inline]
    pub fn new(input: &str) -> Result<Self, NoteError> {
        let tag_path_str = input
            .strip_prefix('#')
            .ok_or(NoteError::Tag(TagError::MissingHash))?;

        let (full_path, segments) = parse_tag_path(tag_path_str)?;

        Ok(Self {
            full_path,
            segments,
        })
    }

    /// Returns the full tag path (without leading `#`).
    #[inline]
    #[must_use]
    pub fn full_path(&self) -> &str {
        &self.full_path.0
    }

    /// Returns the individual segments of the tag.
    #[inline]
    #[must_use]
    pub fn segments(&self) -> TagSegments<'_> {
        TagSegments {
            inner: self.segments.0.iter(),
        }
    }
}

/// Internal wrapper for the full tag path string (without leading `#`).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
struct TagPath(Box<str>);

impl Deref for TagPath {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Internal wrapper for tag segments.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
struct Segments(Vec<Box<str>>);

/// Borrowed iterator over tag segments.
pub struct TagSegments<'tag> {
    inner: std::slice::Iter<'tag, Box<str>>,
}

#[expect(
    clippy::missing_trait_methods,
    reason = "TagSegments relies on default iterator methods."
)]
impl<'tag> Iterator for TagSegments<'tag> {
    type Item = &'tag str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Box::as_ref)
    }
}

impl Deref for Segments {
    type Target = [Box<str>];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn parse_tag_path(path: &str) -> Result<(TagPath, Segments), NoteError> {
    if path.is_empty() {
        return Err(NoteError::Tag(TagError::EmptyTag));
    }

    let segments_count = path.split('/').count();
    let mut segments = Vec::with_capacity(segments_count);
    for segment in path.split('/') {
        if segment.is_empty() {
            return Err(NoteError::Tag(TagError::EmptySegment));
        }

        if !segment.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(NoteError::Tag(TagError::InvalidSegment {
                segment: segment.into(),
            }));
        }
        segments.push(segment.into());
    }

    Ok((TagPath(path.into()), Segments(segments)))
}

impl ArchivedTag {
    /// Returns the full tag path.
    #[inline]
    #[must_use]
    pub fn full_path(&self) -> &str {
        self.full_path.as_str()
    }
}

impl ArchivedTagPath {
    /// Returns the tag path as a string slice.
    #[inline]
    #[must_use]
    fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Tests use assertions in Result-returning functions."
)]
mod tests {
    use super::*;

    mod constructor {
        use rstest::rstest;

        use super::*;

        fn tag_with_project_path() -> Result<Tag, NoteError> {
            Tag::new("#work/project")
        }

        #[test]
        fn full_path_returns_expected_value() -> Result<(), NoteError> {
            let tag = tag_with_project_path()?;
            assert_eq!(
                tag.full_path(),
                "work/project",
                "Full path should omit leading #"
            );
            Ok(())
        }

        #[test]
        fn segments_length_matches_expected() -> Result<(), NoteError> {
            let tag = tag_with_project_path()?;
            assert_eq!(
                tag.segments().count(),
                2,
                "Segments length should match"
            );
            Ok(())
        }

        #[test]
        fn segments_match_expected_values() -> Result<(), NoteError> {
            let tag = tag_with_project_path()?;
            let segments: Vec<&str> = tag.segments().collect();
            assert_eq!(
                segments,
                vec!["work", "project"],
                "Segments should match expected values"
            );
            Ok(())
        }

        #[rstest]
        #[case::simple("#personal", vec!["personal"])]
        #[case::hierarchical(
            "#work/project/urgent",
            vec!["work", "project", "urgent"]
        )]
        fn tag_parsing_accepts_valid_inputs(
            #[case] input: &str,
            #[case] expected: Vec<&str>,
        ) -> Result<(), NoteError> {
            let tag = Tag::new(input)?;
            let actual_segments: Vec<&str> = tag.segments().collect();
            assert_eq!(
                actual_segments, expected,
                "Tag segments should match expected for input: {input}"
            );
            Ok(())
        }

        #[rstest]
        #[case::missing_hash("invalid", NoteError::Tag(TagError::MissingHash))]
        #[case::only_hash("#", NoteError::Tag(TagError::EmptyTag))]
        #[case::empty_segments(
            "#work//urgent",
            NoteError::Tag(TagError::EmptySegment)
        )]
        #[case::invalid_chars(
            "#work project",
            NoteError::Tag(TagError::InvalidSegment {
                segment: "work project".into(),
            })
        )]
        fn tag_parsing_rejects_invalid_inputs(
            #[case] input: &str,
            #[case] expected: NoteError,
        ) {
            let result = Tag::new(input);
            assert_eq!(
                result,
                Err(expected),
                "Expected error for {input}, got: {result:?}"
            );
        }
    }

    mod proptests {
        use proptest::{prelude::*, test_runner::TestRunner};

        use super::*;

        #[test]
        fn rejects_invalid_characters_in_segments() -> Result<(), String> {
            let mut runner = TestRunner::deterministic();
            let strategy =
                "#[a-zA-Z0-9_-]*/[ !@#$%^&*()]+/[a-zA-Z0-9_-]*".prop_map(|s| s);

            let run_result = runner.run(&strategy, |s| {
                let result = Tag::new(&s);
                prop_assert!(
                    result.is_err(),
                    "Tag with invalid characters '{s}' should be rejected"
                );
                Ok(())
            });
            run_result.map_err(|e| {
                format!("Deterministic proptest should not fail: {e:?}")
            })?;

            Ok(())
        }

        #[test]
        fn accepts_valid_alphanumeric_tags() -> Result<(), String> {
            let mut runner = TestRunner::deterministic();
            let strategy = "#[a-zA-Z0-9_-]+(/[a-zA-Z0-9_-]+)*".prop_map(|s| s);

            let run_result = runner.run(&strategy, |s| {
                let result = Tag::new(&s);
                prop_assert!(
                    result.is_ok(),
                    "Valid tag '{s}' should be accepted"
                );
                Ok(())
            });
            run_result.map_err(|e| {
                format!("Deterministic proptest should not fail: {e:?}")
            })?;

            Ok(())
        }
    }
}
