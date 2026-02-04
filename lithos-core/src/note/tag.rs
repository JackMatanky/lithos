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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let tag = Tag::new("#work/project/urgent")?;
    /// assert_eq!(tag.full_path.as_str(), "work/project/urgent");
    /// assert_eq!(tag.segments.len(), 3);
    /// assert_eq!(&*tag.segments, &[
    ///     "work".into(),
    ///     "project".into(),
    ///     "urgent".into()
    /// ]);
    ///
    /// let simple_tag = Tag::new("#personal")?;
    /// assert_eq!(simple_tag.full_path.as_str(), "personal");
    /// assert_eq!(&*simple_tag.segments, &["personal".into()]);
    /// # Ok(())
    /// # }
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
#[expect(
    clippy::disallowed_methods,
    reason = "Test setup uses expect for deterministic fixtures."
)]
mod tests {
    use super::*;

    mod constructor {
        use rstest::rstest;

        use super::*;

        #[test]
        fn full_path_returns_expected_value() {
            let tag = Tag::new("#work/project").expect("Tag should parse");
            assert_eq!(
                tag.full_path.as_str(),
                "work/project",
                "Full path should omit leading #"
            );
        }

        #[test]
        fn segments_length_matches_expected() {
            let tag = Tag::new("#work/project").expect("Tag should parse");
            assert_eq!(tag.segments.len(), 2, "Segments length should match");
        }

        #[test]
        fn segments_match_expected_values() {
            let tag = Tag::new("#work/project").expect("Tag should parse");
            assert_eq!(
                &*tag.segments,
                &["work".into(), "project".into()],
                "Segments should match expected values"
            );
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
        ) {
            let tag = Tag::new(input).expect("Tag should parse");
            let actual_segments: Vec<&str> =
                tag.segments.iter().map(AsRef::as_ref).collect();
            assert_eq!(
                actual_segments, expected,
                "Tag segments should match expected for input: {input}"
            );
        }

        #[rstest]
        #[case::missing_hash(
            "invalid",
            NoteError::Tag("Tag must start with #".to_owned())
        )]
        #[case::only_hash(
            "#",
            NoteError::Tag("Tag cannot be empty".to_owned())
        )]
        #[case::empty_segments(
            "#work//urgent",
            NoteError::Tag("Empty tag segment".to_owned())
        )]
        #[case::invalid_chars(
            "#work project",
            NoteError::Tag(
                "Invalid tag segment 'work project': only alphanumeric, \
                  underscore, and hyphen allowed"
                    .to_owned(),
            )
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
