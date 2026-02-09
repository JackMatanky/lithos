//! Tag domain entity for Note aggregate.

#![expect(
    clippy::exhaustive_structs,
    reason = "rkyv generates exhaustive ArchivedTag despite #[non_exhaustive]"
)]

use super::error::NoteError;

/// Validated note tag (e.g., `#work/project`).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Tag {
    /// Full path of the tag without leading `#` (e.g., `work/project`).
    pub full_path: FullPath,
    /// Individual segments of the tag (e.g., `["work", "project"]`).
    pub segments: Segments,
}

/// Tag full path representation.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct FullPath(Box<str>);

/// Tag segments representation.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct Segments(Box<[Box<str>]>);

impl FullPath {
    fn new(input: &str) -> Result<Self, NoteError> {
        let full_path = input.strip_prefix('#').ok_or_else(|| {
            NoteError::Tag("Tag must start with #".to_owned())
        })?;

        if full_path.is_empty() {
            return Err(NoteError::Tag("Tag cannot be empty".to_owned()));
        }
        Ok(Self(full_path.into()))
    }

    /// Return the full path as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Segments {
    fn new(full_path: &str) -> Result<Self, NoteError> {
        let segments: Vec<Box<str>> = full_path
            .split('/')
            .map(|s| {
                if s.is_empty() {
                    return Err(NoteError::Tag("Empty tag segment".to_owned()));
                }
                if !s
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                {
                    return Err(NoteError::Tag(format!(
                        "Invalid tag segment '{s}': only alphanumeric, \
                         underscore, and hyphen allowed"
                    )));
                }
                Ok(s.into())
            })
            .collect::<Result<_, _>>()?;

        Ok(Self(segments.into_boxed_slice()))
    }

    /// Return an iterator over segments.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Box<str>> {
        self.0.iter()
    }

    /// Returns the number of segments.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if there are no segments.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::ops::Deref for Segments {
    type Target = [Box<str>];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Tag {
    /// Create a new validated tag.
    ///
    /// Tags must start with `#` and contain one or more segments separated by
    /// `/`. Each segment must be alphanumeric, underscores, or hyphens.
    ///
    /// # Examples
    /// ```
    /// # use lithos_core::note::tag::Tag;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let tag = Tag::new("#work/project")?;
    /// assert_eq!(tag.full_path.as_str(), "work/project");
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
    reason = "Test modules have relaxed unwrap/expect rules"
)]
mod tests {
    use super::*;

    mod constructor {
        use rstest::rstest;

        use super::super::*;

        fn tag_with_project_path() -> Tag {
            Tag::new("#work/project").expect("valid setup")
        }

        #[test]
        fn full_path_returns_expected_value() {
            let tag = tag_with_project_path();
            assert_eq!(
                tag.full_path.as_str(),
                "work/project",
                "Full path should omit leading #"
            );
        }

        #[test]
        fn segments_length_matches_expected() {
            let tag = tag_with_project_path();
            assert_eq!(tag.segments.len(), 2, "Segments length should match");
        }

        #[test]
        fn segments_match_expected_values() {
            let tag = tag_with_project_path();
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
            let tag = Tag::new(input).expect("valid input");
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
