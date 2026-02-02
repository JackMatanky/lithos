//! Note bounded context aggregate root.
//!
//! This module defines the Note aggregate root that composes subentities
//! from other modules: Frontmatter, Links, Tags, Heading, Task, and Section.
//!
//! # Business Rules
//! - Note IDs use UUID v7 for stable, time-ordered identity.
//! - All file paths must be vault-relative and validated against path
//!   traversal.
//! - Validation follows a three-phase pipeline: Syntactic -> Orchestration ->
//!   Semantic.
//! - Links are value objects owned by the Note aggregate; the parent
//!   relationship is implicit through containment.

use uuid::Uuid;

use super::{
    error::NoteError,
    events::{NoteCreated, NoteEvents},
    frontmatter::Frontmatter,
    link::{Link, Style},
    structure::{Heading, Section},
    tag::Tag,
    task::Task,
};
use crate::fs::validate_vault_path;

/// Aggregate root representing an Obsidian note.
///
/// # Invariants
/// - `id` is always a valid UUID v7.
/// - `path` is vault-relative, non-empty, ends with `.md`, no traversal.
/// - All subentities are consistent (e.g., link targets non-empty).
/// - Entities are immutable after construction.
///
/// # Link Ownership
/// Links are value objects owned by this aggregate. The parent relationship
/// is implicit through containment - there is no `source_note_id` stored in
/// each link. This follows DDD aggregate patterns where ownership is
/// structural, not duplicated data.
///
/// # Examples
/// ```
/// # use lithos_core::note::aggregate::Note;
/// # use uuid::Uuid;
/// // For new files (first-time indexing)
/// let new_id = Uuid::now_v7();
/// let note = Note::new(new_id, "projects/example.md".to_string()).unwrap();
/// assert_eq!(note.path.as_str(), "projects/example.md");
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[expect(
    clippy::partial_pub_fields,
    reason = "Aggregate root requires mixed visibility for domain events"
)]
#[non_exhaustive]
pub struct Note {
    /// UUID v7 identity (time-ordered).
    pub id: Uuid,
    /// Vault-relative path.
    pub path: NotePath,
    /// YAML metadata.
    pub frontmatter: Option<Frontmatter>,
    /// All links (wiki-links, markdown links, and embeds).
    pub links: Vec<Link>,
    /// Hierarchical tags.
    pub tags: Vec<Tag>,
    /// Markdown headings.
    pub headings: Vec<Heading>,
    /// Task items.
    pub tasks: Vec<Task>,
    /// Document sections.
    pub sections: Vec<Section>,
    /// Domain events pending emission (not serialized).
    #[serde(skip)]
    pending_events: Vec<NoteEvents>,
}

/// Validated vault-relative path for a note.
///
/// Enforces invariants:
/// - Must be relative to vault root
/// - Must end with `.md` extension
/// - Must not contain path traversal segments (`..`)
/// - Must not be empty
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
pub struct NotePath(Box<str>);

impl AsRef<str> for NotePath {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for NotePath {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for NotePath {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<NotePath> for String {
    #[inline]
    fn from(path: NotePath) -> Self {
        path.0.into()
    }
}

impl TryFrom<&str> for NotePath {
    type Error = NoteError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl TryFrom<String> for NotePath {
    type Error = NoteError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Note {
    /// Adds a domain event to the pending events collection.
    #[inline]
    pub fn add_event(&mut self, event: NoteEvents) {
        self.pending_events.push(event);
    }

    /// Returns all embeds in this note.
    ///
    /// This is a convenience method that filters links by type.
    ///
    /// # Examples
    /// ```
    /// # use lithos_core::note::aggregate::Note;
    /// # use lithos_core::note::link::{Link, Target, EmbedType};
    /// # use uuid::Uuid;
    /// let mut note = Note::new(Uuid::now_v7(), "note.md".to_string()).unwrap();
    /// let embed = Link::new_embed(
    ///     Target::Unresolved {
    ///         raw: "img.png".into(),
    ///     },
    ///     EmbedType::Image,
    ///     None,
    ///     0,
    /// )
    /// .unwrap();
    /// note.links.push(embed);
    /// assert_eq!(note.embeds().count(), 1);
    /// ```
    #[inline]
    pub fn embeds(&self) -> impl Iterator<Item = &Link> {
        self.links.iter().filter(|l| l.is_embed())
    }

    /// Returns a reference to the note's frontmatter, if present.
    #[inline]
    #[must_use]
    pub const fn frontmatter(&self) -> Option<&Frontmatter> {
        self.frontmatter.as_ref()
    }

    /// Returns all markdown-style links in this note (excluding embeds).
    ///
    /// # Examples
    /// ```
    /// # use lithos_core::note::aggregate::Note;
    /// # use lithos_core::note::link::{Link, Target};
    /// # use uuid::Uuid;
    /// let mut note = Note::new(Uuid::now_v7(), "note.md".to_string()).unwrap();
    /// let link = Link::new_markdown_link(
    ///     Target::External {
    ///         url: "https://example.com".into(),
    ///     },
    ///     None,
    ///     None,
    ///     20,
    /// )
    /// .unwrap();
    /// note.links.push(link);
    /// assert_eq!(note.markdown_links().count(), 1);
    /// ```
    #[inline]
    pub fn markdown_links(&self) -> impl Iterator<Item = &Link> {
        self.links
            .iter()
            .filter(|l| l.style() == Style::MdLink && !l.is_embed())
    }

    /// Creates a new note aggregate with the provided UUID and validated path.
    ///
    /// # Errors
    /// Returns `NoteError::InvalidPath` if path is empty, absolute, missing
    /// `.md` extension, or contains `..`.
    ///
    /// # Examples
    /// ```
    /// # use lithos_core::note::aggregate::Note;
    /// # use uuid::Uuid;
    /// let note = Note::new(Uuid::now_v7(), "notes/intro.md".to_string()).unwrap();
    /// assert_eq!(note.path.as_str(), "notes/intro.md");
    /// ```
    #[inline]
    pub fn new(id: Uuid, path: String) -> Result<Self, NoteError> {
        let path_for_event = path.clone();
        let note_path = NotePath::new(path)?;

        let mut note = Self {
            id,
            path: note_path,
            frontmatter: None,
            links: vec![],
            tags: vec![],
            headings: vec![],
            tasks: vec![],
            sections: vec![],
            pending_events: vec![],
        };

        note.add_event(NoteEvents::NoteCreated(NoteCreated::new(
            id,
            path_for_event,
            chrono::Utc::now().timestamp(),
        )));

        Ok(note)
    }

    /// Returns a reference to pending domain events without clearing them.
    #[inline]
    #[must_use]
    pub fn pending_events(&self) -> &[NoteEvents] {
        &self.pending_events
    }

    /// Sets the note's frontmatter.
    #[inline]
    pub fn set_frontmatter(&mut self, frontmatter: Option<Frontmatter>) {
        self.frontmatter = frontmatter;
    }

    /// Returns all pending domain events and clears the collection.
    #[inline]
    #[must_use]
    pub fn take_events(&mut self) -> Vec<NoteEvents> {
        std::mem::take(&mut self.pending_events)
    }

    /// Validates the note's internal consistency.
    ///
    /// # Errors
    /// Returns `NoteError::ValidationFailed` if cross-entity invariants are
    /// violated (e.g., invalid link configurations).
    ///
    /// # Examples
    /// ```
    /// # use lithos_core::note::aggregate::Note;
    /// # use uuid::Uuid;
    /// let note_id = Uuid::now_v7();
    /// let note = Note::new(note_id, "test.md".to_string()).unwrap();
    /// note.validate().unwrap();
    /// ```
    #[inline]
    pub fn validate(&self) -> Result<(), NoteError> {
        for link in &self.links {
            link.validate().map_err(|e| {
                NoteError::ValidationFailed(format!("Invalid link: {e}"))
            })?;
        }

        Ok(())
    }

    /// Returns all wiki-style links in this note (excluding embeds).
    ///
    /// # Examples
    /// ```
    /// # use lithos_core::note::aggregate::Note;
    /// # use lithos_core::note::link::{Link, Target};
    /// # use uuid::Uuid;
    /// let mut note = Note::new(Uuid::now_v7(), "note.md".to_string()).unwrap();
    /// let link = Link::new_wikilink(
    ///     Target::Unresolved {
    ///         raw: "other-note".into(),
    ///     },
    ///     None,
    ///     None,
    ///     0,
    /// )
    /// .unwrap();
    /// note.links.push(link);
    /// assert_eq!(note.wikilinks().count(), 1);
    /// ```
    #[inline]
    pub fn wikilinks(&self) -> impl Iterator<Item = &Link> {
        self.links
            .iter()
            .filter(|l| l.style() == Style::WikiLink && !l.is_embed())
    }
}

impl NotePath {
    /// Returns the path as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Creates a new `NotePath` with validation.
    ///
    /// # Errors
    /// Returns `NoteError::InvalidPath` if validation fails.
    #[inline]
    pub fn new(path: String) -> Result<Self, NoteError> {
        Self::validate(&path)?;
        Ok(Self(path.into()))
    }

    /// Validates a vault-relative path.
    ///
    /// # Errors
    /// Returns `NoteError::InvalidPath` if validation fails.
    #[inline]
    pub fn validate(path: &str) -> Result<(), NoteError> {
        validate_vault_path(path, Some("md")).map_err(NoteError::InvalidPath)
    }
}

#[cfg(test)]
/// Test fixtures for Note model testing.
pub mod fixtures {
    use std::collections::HashMap;

    use super::*;
    use crate::note::frontmatter::FieldValue;

    /// Fixed UUID for deterministic tests (valid UUID v7 format).
    pub const TEST_NOTE_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0001);

    /// Test fixture: Create example frontmatter with realistic field values.
    ///
    /// # Panics
    /// Panics if the hardcoded date string is invalid or frontmatter
    /// construction fails.
    #[expect(
        clippy::disallowed_methods,
        reason = "Test fixture - unwrap/expect acceptable in test code"
    )]
    #[inline]
    #[must_use]
    pub fn example_frontmatter() -> Frontmatter {
        let mut fields = HashMap::new();
        fields.insert(
            "title".to_owned(),
            FieldValue::String("Test Note".to_owned()),
        );
        fields.insert(
            "created".to_owned(),
            FieldValue::Date(
                chrono::DateTime::parse_from_rfc3339("2024-01-15T14:30:00Z")
                    .unwrap()
                    .into(),
            ),
        );
        Frontmatter::new(fields).expect("Valid frontmatter")
    }

    /// Test fixture: Create example hierarchical tag for testing.
    ///
    /// # Panics
    /// Panics if the hardcoded tag string is invalid.
    #[expect(
        clippy::disallowed_methods,
        reason = "Test fixture - unwrap/expect acceptable in test code"
    )]
    #[inline]
    #[must_use]
    pub fn example_tag() -> Tag {
        Tag::new("#work/project").expect("Valid tag")
    }

    /// Test fixture: Create complete example Note aggregate for testing.
    ///
    /// # Panics
    /// Panics if the hardcoded path is invalid.
    #[expect(
        clippy::disallowed_methods,
        reason = "Test fixture - unwrap/expect acceptable in test code"
    )]
    #[inline]
    #[must_use]
    pub fn example_note() -> Note {
        Note {
            id: TEST_NOTE_ID,
            path: NotePath::new("test/example.md".to_owned())
                .expect("Valid path"),
            frontmatter: Some(example_frontmatter()),
            links: vec![],
            tags: vec![example_tag()],
            headings: vec![],
            tasks: vec![],
            sections: vec![],
            pending_events: vec![],
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    clippy::panic,
    clippy::disallowed_methods,
    reason = "Test module organization and behavior verification patterns; \
              unwrap/expect acceptable in tests."
)]
mod tests {
    // # LINT_DISABLE_REASON: Standard test utilities and behavioral
    // verification patterns.
    use lithos_test_utils::assert_err_kind;

    use super::*;
    use crate::note::{
        frontmatter::FieldValue,
        link::{EmbedType, Target},
        task::TaskStatus,
    };

    mod accessors {
        use super::*;

        /// 3.1-UNIT-009: `mutators_update_aggregate_state`.
        /// Priority: P1.
        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses Result::unwrap() for ergonomic addition of \
                      sub-entities during state transition testing. \
                      Acceptable in test-only code paths."
        )]
        fn mutators_update_aggregate_state() {
            // GIVEN: a basic note
            let note_id = Uuid::now_v7();
            let mut note = Note::new(note_id, "note.md".to_owned()).unwrap();

            // WHEN: adding various sub-entities
            note.tags.push(Tag::new("#test").unwrap());
            note.headings.push(Heading::new(1, "H1".into(), 0).unwrap());
            note.tasks.push(
                Task::new("Task".into(), TaskStatus::Incomplete, 0).unwrap(),
            );
            note.sections.push(Section::new(None, "Body".into(), 0..4));

            // Add a wikilink
            note.links.push(
                Link::new_wikilink(
                    Target::Unresolved {
                        raw: "link.md".into(),
                    },
                    None,
                    None,
                    0,
                )
                .unwrap(),
            );

            // Add an embed
            note.links.push(
                Link::new_embed(
                    Target::Unresolved {
                        raw: "img.png".into(),
                    },
                    EmbedType::Image,
                    None,
                    0,
                )
                .unwrap(),
            );

            let fm_fields =
                [("title".to_owned(), FieldValue::String("Title".into()))]
                    .into_iter()
                    .collect();
            note.set_frontmatter(Some(Frontmatter::new(fm_fields).unwrap()));

            // THEN: the aggregate state is updated correctly
            assert_eq!(note.tags.len(), 1);
            assert_eq!(note.headings.len(), 1);
            assert_eq!(note.tasks.len(), 1);
            assert_eq!(note.sections.len(), 1);
            assert_eq!(note.links.len(), 2); // unified links
            assert_eq!(note.wikilinks().count(), 1);
            assert_eq!(note.embeds().count(), 1);
            assert!(note.frontmatter().is_some());
        }

        /// 3.1-UNIT-010: `filtered_link_iterators_work`.
        /// Priority: P1.
        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses Result::unwrap() for populating Note with \
                      diverse link variants for iterator testing. Failures \
                      represent environment or test data issues."
        )]
        fn filtered_link_iterators_work() {
            // GIVEN: a note with various link types (2 wikilinks, 1 markdown, 1
            // embed)
            let mut note =
                Note::new(Uuid::now_v7(), "note.md".to_owned()).unwrap();

            note.links.push(
                Link::new_wikilink(
                    Target::Unresolved {
                        raw: "wiki1.md".into(),
                    },
                    None,
                    None,
                    0,
                )
                .unwrap(),
            );
            note.links.push(
                Link::new_wikilink(
                    Target::Unresolved {
                        raw: "wiki2.md".into(),
                    },
                    None,
                    None,
                    10,
                )
                .unwrap(),
            );
            note.links.push(
                Link::new_markdown_link(
                    Target::External {
                        url: "https://example.com".into(),
                    },
                    None,
                    None,
                    20,
                )
                .unwrap(),
            );
            note.links.push(
                Link::new_embed(
                    Target::Unresolved {
                        raw: "img.png".into(),
                    },
                    EmbedType::Image,
                    None,
                    30,
                )
                .unwrap(),
            );

            // WHEN: using filtered iterators to query specific link types
            let all_count = note.links.len();
            let wiki_count = note.wikilinks().count();
            let md_count = note.markdown_links().count();
            let embed_count = note.embeds().count();

            // THEN: each iterator returns the correct count for its type
            assert_eq!(all_count, 4);
            assert_eq!(wiki_count, 2);
            assert_eq!(md_count, 1);
            assert_eq!(embed_count, 1);
        }
    }

    mod new {
        use rstest::rstest;

        use super::*;

        /// 3.1-UNIT-001: Note Path Validation Matrix.
        /// Priority: P0.
        #[rstest]
        #[case::empty("", Err(NoteError::InvalidPath("Path cannot be empty".into())))]
        #[case::absolute(
            "/absolute/path.md",
            Err(NoteError::InvalidPath("Path must be relative".into()))
        )]
        #[case::traversal(
            "../etc/passwd",
            Err(NoteError::InvalidPath("Path traversal not allowed".into()))
        )]
        #[case::missing_extension(
            "projects/lithos",
            Err(NoteError::InvalidPath("Path must end with .md".into()))
        )]
        #[case::valid("valid.md", Ok(()))]
        #[case::nested_valid("folder/sub/note.md", Ok(()))]
        fn path_validation_matrix(
            #[case] path: &str,
            #[case] expected: Result<(), NoteError>,
        ) {
            // GIVEN: a vault path from the validation matrix
            let test_id = Uuid::now_v7();

            // WHEN: creating a note
            let result = Note::new(test_id, path.to_owned());

            // THEN: the result matches the expected outcome
            match expected {
                Ok(()) => assert!(
                    result.is_ok(),
                    "Expected path '{path}' to be valid"
                ),
                Err(NoteError::InvalidPath(_)) => {
                    assert_err_kind!(result, NoteError::InvalidPath(_));
                }
                Err(e) => panic!("Unexpected error kind in matrix: {e:?}"),
            }
        }

        /// 3.1-UNIT-005: `generates_sequential_uuids`.
        /// Priority: P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test fixture creation")]
        fn generates_sequential_uuids() {
            // WHEN: creating notes sequentially
            let note1 = Note::new(Uuid::now_v7(), "one.md".into()).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
            let note2 = Note::new(Uuid::now_v7(), "two.md".into()).unwrap();

            // THEN: later UUIDs sort after earlier ones
            assert!(note2.id > note1.id);
        }
    }

    mod validate {
        use super::*;

        /// 3.1-UNIT-006: `succeeds_when_all_entities_are_valid`.
        /// Priority: P0.
        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses Result::expect() during arrangement of \
                      complex Note aggregate. Failures here indicate logic \
                      errors in test data rather than code under test."
        )]
        fn succeeds_when_all_entities_are_valid() {
            // GIVEN: a note aggregate with consistent sub-entities
            let note_id = Uuid::now_v7();
            let note = NoteBuilder::new()
                .id(note_id)
                .path(NotePath::new("valid.md".to_owned()).expect("Valid path"))
                .tags(vec![Tag::new("#work").expect("Valid tag")])
                .headings(vec![
                    Heading::new(1, "Title".into(), 0).expect("Valid heading"),
                ])
                .links(vec![
                    Link::new_wikilink(
                        Target::Unresolved {
                            raw: "target.md".into(),
                        },
                        None,
                        None,
                        0,
                    )
                    .expect("Valid link"),
                ])
                .build();

            // WHEN: validating the aggregate
            let result = note.validate();

            // THEN: validation succeeds
            assert!(
                result.is_ok(),
                "Validation should pass, but failed with: {:?}",
                result.err()
            );
        }
    }

    use lithos_test_utils::test_builder;

    test_builder!(NoteBuilder, Note, {
        id: Uuid = Uuid::now_v7(),
        path: NotePath = NotePath::new("default.md".to_owned()).expect("Valid default path"),
        frontmatter: Option<Frontmatter> = None,
        links: Vec<Link> = vec![],
        tags: Vec<Tag> = vec![],
        headings: Vec<Heading> = vec![],
        tasks: Vec<Task> = vec![],
        sections: Vec<Section> = vec![],
        pending_events: Vec<NoteEvents> = vec![],
    });
}
