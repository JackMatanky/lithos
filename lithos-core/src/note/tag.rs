//! Tag subentity for Note aggregate.
//!
//! Represents hierarchical tags used for note organization.
#![allow(
    clippy::exhaustive_structs,
    reason = "rkyv generates Archived types with public fields"
)]

use std::ops::Deref;

use super::error::NoteError;

/// Internal wrapper for the full tag path string.
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
pub struct FullPath(Box<str>);

/// Internal wrapper for tag segments.
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
pub struct Segments(Vec<Box<str>>);

/// Represents a hierarchical tag with segments.
///
/// Tags follow the format `#segment1/segment2/segment3` and are used
/// for organizing and categorizing notes.
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
pub struct Tag {
    /// Full tag path (without leading `#`).
    pub full_path: FullPath,
    /// Individual path segments.
    pub segments: Segments,
}

impl Deref for FullPath {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Segments {
    type Target = [Box<str>];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FullPath {
    /// Returns string reference.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Creates a new `FullPath` from a raw tag string.
    fn new(input: &str) -> Result<Self, NoteError> {
        Self::validate(input)?;
        let tag_path = input.strip_prefix('#').ok_or_else(|| {
            NoteError::Tag("Tag must start with #".to_owned())
        })?;
        Ok(Self(tag_path.into()))
    }

    /// Validates a raw tag string.
    ///
    /// # Errors
    /// Returns [`NoteError::Tag`] if the input doesn't start with `#` or is
    /// empty.
    #[inline]
    pub fn validate(input: &str) -> Result<(), NoteError> {
        let tag_path = input.strip_prefix('#').ok_or_else(|| {
            NoteError::Tag("Tag must start with #".to_owned())
        })?;

        if tag_path.is_empty() {
            return Err(NoteError::Tag("Tag cannot be empty".to_owned()));
        }

        Ok(())
    }
}

impl Segments {
    /// Returns segments as slice.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[Box<str>] {
        &self.0
    }

    /// Creates new `Segments` from a tag path.
    fn new(tag_path: &str) -> Result<Self, NoteError> {
        Self::validate(tag_path)?;
        let segments = tag_path.split('/').map(Into::into).collect();
        Ok(Self(segments))
    }

    /// Validates individual path segments.
    ///
    /// # Errors
    /// Returns [`NoteError::Tag`] if any segment is empty or contains invalid
    /// characters.
    #[inline]
    pub fn validate(tag_path: &str) -> Result<(), NoteError> {
        let segments: Vec<&str> = tag_path.split('/').collect();

        if segments.iter().any(|s| s.is_empty()) {
            return Err(NoteError::Tag("Empty tag segment".to_owned()));
        }

        for segment in &segments {
            if !segment
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                return Err(NoteError::Tag(format!(
                    "Invalid tag segment '{segment}': only alphanumeric, \
                     underscore, and hyphen allowed"
                )));
            }
        }

        Ok(())
    }
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
    /// # Examples
    /// ```
    /// use lithos_core::note::tag::Tag;
    ///
    /// let tag = Tag::new("#work/project/urgent").unwrap();
    /// assert_eq!(tag.full_path.as_str(), "work/project/urgent");
    /// assert_eq!(tag.segments.len(), 3);
    /// assert_eq!(&*tag.segments, &[
    ///     "work".into(),
    ///     "project".into(),
    ///     "urgent".into()
    /// ]);
    ///
    /// let simple_tag = Tag::new("#personal").unwrap();
    /// assert_eq!(simple_tag.full_path.as_str(), "personal");
    /// assert_eq!(&*simple_tag.segments, &["personal".into()]);
    /// ```
    ///
    /// # Errors
    /// Returns [`NoteError::Tag`] if the input doesn't start with `#`, the tag
    /// is empty, contains empty segments, or contains invalid characters.
    #[inline]
    pub fn new(input: &str) -> Result<Self, NoteError> {
        let full_path = FullPath::new(input)?;
        let segments = Segments::new(full_path.as_str())?;

        Ok(Self {
            full_path,
            segments,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod new {
        use rstest::rstest;

        use super::*;

        #[test]
        fn accessors_return_expected_values() {
            let tag_result = Tag::new("#work/project");
            assert!(tag_result.is_ok(), "Tag should parse: {tag_result:?}");
            let Ok(tag) = tag_result else {
                return;
            };
            assert_eq!(tag.full_path.as_str(), "work/project");
            assert_eq!(tag.segments.len(), 2);
            assert_eq!(&*tag.segments, &["work".into(), "project".into()]);
        }

        #[rstest]
        #[case::simple("#personal", Ok(vec!["personal"]))]
        #[case::hierarchical(
            "#work/project/urgent",
            Ok(vec!["work", "project", "urgent"])
        )]
        #[case::missing_hash(
            "invalid",
            Err(NoteError::Tag("Tag must start with #".to_owned()))
        )]
        #[case::only_hash(
            "#",
            Err(NoteError::Tag("Tag cannot be empty".to_owned()))
        )]
        #[case::empty_segments(
            "#work//urgent",
            Err(NoteError::Tag("Empty tag segment".to_owned()))
        )]
        #[case::invalid_chars(
            "#work project",
            Err(NoteError::Tag(
                "Invalid tag segment 'work project': only alphanumeric, \
                  underscore, and hyphen allowed"
                    .to_owned(),
            ))
        )]
        fn tag_parsing_matrix(
            #[case] input: &str,
            #[case] expected: Result<Vec<&str>, NoteError>,
        ) {
            let result = Tag::new(input);

            match expected {
                Ok(segments) => {
                    assert!(result.is_ok(), "Failed for {input}: {result:?}",);
                    let Ok(tag) = result else {
                        return;
                    };
                    let actual_segments: Vec<&str> =
                        tag.segments.iter().map(AsRef::as_ref).collect();
                    assert_eq!(
                        actual_segments, segments,
                        "Tag segments should match expected for input: {input}"
                    );
                }
                Err(e) => {
                    assert!(
                        result.is_err(),
                        "Expected error for {input}, got: {result:?}"
                    );
                    let Err(actual) = result else {
                        return;
                    };
                    assert_eq!(
                        actual, e,
                        "Error should match expected for input: {input}"
                    );
                }
            }
        }
    }

    mod proptests {
        use proptest::{prelude::*, test_runner::TestRunner};

        use super::*;

        #[test]
        fn rejects_invalid_characters_in_segments() {
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
            assert!(
                run_result.is_ok(),
                "Deterministic proptest should not fail: {run_result:?}"
            );
        }

        #[test]
        fn accepts_valid_alphanumeric_tags() {
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
            assert!(
                run_result.is_ok(),
                "Deterministic proptest should not fail: {run_result:?}"
            );
        }
    }
}
