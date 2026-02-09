//! Note aggregate root and core domain entities.
//!
//! Composes frontmatter, links, tags, headings, tasks, and sections into a
//! unified document model.
//!
//! # Constraints
//! - **Identity**: Uses UUID v7 for stable, time-ordered identity.
//! - **Security**: All file paths must be vault-relative and validated against
//!   traversal.
//! - **Validation**: Uses a three-phase pipeline: Syntactic -> Orchestration ->
//!   Semantic.
//! - **Ownership**: Links are value objects owned by the Note; the parent
//!   relationship is implicit through containment.
#![expect(
    clippy::exhaustive_structs,
    reason = "rkyv generates exhaustive ArchivedNote/ArchivedNotePath despite \
              #[non_exhaustive]"
)]

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
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // For new files (first-time indexing)
/// let new_id = Uuid::now_v7();
/// let note = Note::new(new_id, "projects/example.md".to_string())?;
/// assert_eq!(
///     note.path.as_str(),
///     "projects/example.md",
///     "Note path should match"
/// );
/// # Ok(())
/// # }
/// ```
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
pub struct Note {
    /// UUID v7 identity (time-ordered).
    pub id: Uuid,
    /// Vault-relative path.
    pub path: NotePath,
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
    /// YAML metadata.
    pub frontmatter: Option<Frontmatter>,
    /// Domain events pending emission (not serialized).
    #[rkyv(with = rkyv::with::Skip)]
    #[serde(skip)]
    pub pending_events: Vec<NoteEvents>,
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
    /// # use lithos_core::note::types::SourceByteOffset;
    /// # use uuid::Uuid;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut note = Note::new(Uuid::now_v7(), "note.md".to_string())?;
    /// let embed = Link::new_embed(
    ///     Target::Unresolved {
    ///         raw: "img.png".into(),
    ///     },
    ///     EmbedType::Image,
    ///     None,
    ///     SourceByteOffset::new(0),
    /// )?;
    /// note.links.push(embed);
    /// assert_eq!(note.embeds().count(), 1, "Expected one embed");
    /// # Ok(())
    /// # }
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
    /// # use lithos_core::note::types::SourceByteOffset;
    /// # use uuid::Uuid;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut note = Note::new(Uuid::now_v7(), "note.md".to_string())?;
    /// let link = Link::new_markdown_link(
    ///     Target::External {
    ///         url: "https://example.com".into(),
    ///     },
    ///     None,
    ///     None,
    ///     SourceByteOffset::new(20),
    /// )?;
    /// note.links.push(link);
    /// assert_eq!(note.markdown_links().count(), 1, "Expected one markdown link");
    /// # Ok(())
    /// # }
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let note = Note::new(Uuid::now_v7(), "notes/intro.md".to_string())?;
    /// assert_eq!(note.path.as_str(), "notes/intro.md", "Note path should match");
    /// # Ok(())
    /// # }
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let note_id = Uuid::now_v7();
    /// let note = Note::new(note_id, "test.md".to_string())?;
    /// note.validate()?;
    /// # Ok(())
    /// # }
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
    /// # use lithos_core::note::types::SourceByteOffset;
    /// # use uuid::Uuid;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut note = Note::new(Uuid::now_v7(), "note.md".to_string())?;
    /// let link = Link::new_wikilink(
    ///     Target::Unresolved {
    ///         raw: "target".into(),
    ///     },
    ///     None,
    ///     None,
    ///     SourceByteOffset::new(0),
    /// )?;
    ///
    /// note.links.push(link);
    /// assert_eq!(note.wikilinks().count(), 1, "Expected one wiki link");
    /// # Ok(())
    /// # }
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
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures and submodules for readability."
)]
#[expect(
    clippy::disallowed_methods,
    reason = "Test setup uses expect for deterministic fixtures."
)]
mod tests {
    /// Test fixtures for Note model testing.
    mod fixtures {
        use super::*;

        /// Fixed UUID for deterministic tests (valid UUID v7 format).
        pub const TEST_NOTE_ID: Uuid =
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0001);

        /// Earlier UUID for time-ordering tests (UUID v7 format with earlier
        /// timestamp).
        pub const TEST_NOTE_ID_EARLIER: Uuid =
            Uuid::from_u128(0x0188_0000_0000_0000_8000_0000_0000_0001);

        /// Later UUID for time-ordering tests (UUID v7 format with later
        /// timestamp).
        pub const TEST_NOTE_ID_LATER: Uuid =
            Uuid::from_u128(0x0188_0000_0000_0001_8000_0000_0000_0002);

        pub fn base_note() -> Result<Note, NoteError> {
            Note::new(TEST_NOTE_ID, "note.md".to_owned())
        }

        pub fn wikilink(
            raw: &str,
            pos: impl Into<crate::note::types::SourceByteOffset>,
        ) -> Result<Link, NoteError> {
            Link::new_wikilink(
                Target::Unresolved {
                    raw: raw.into(),
                },
                None,
                None,
                pos.into(),
            )
        }

        pub fn embed(
            raw: &str,
            pos: impl Into<crate::note::types::SourceByteOffset>,
        ) -> Result<Link, NoteError> {
            Link::new_embed(
                Target::Unresolved {
                    raw: raw.into(),
                },
                EmbedType::Image,
                None,
                pos.into(),
            )
        }

        pub fn note_with_link_mix() -> Result<Note, NoteError> {
            let mut note = base_note()?;

            note.links.push(wikilink("wiki1.md", 0u32)?);
            note.links.push(wikilink("wiki2.md", 10u32)?);
            let md = Link::new_markdown_link(
                Target::External {
                    url: "https://example.com".into(),
                },
                None,
                None,
                crate::note::types::SourceByteOffset::new(20),
            )?;
            note.links.push(md);
            note.links.push(embed("img.png", 30u32)?);

            Ok(note)
        }

        // Additional fixtures can be added here as tests expand.
    }

    use fixtures::TEST_NOTE_ID;

    use super::*;
    use crate::note::{
        frontmatter::FieldValue,
        link::{EmbedType, Target},
        task::TaskStatus,
    };

    mod accessors {
        use super::*;

        /// 3.1-UNIT-009: `tags_update_aggregate_state`.
        /// Priority: P1.
        #[test]
        fn tags_update_aggregate_state() {
            let mut note = fixtures::base_note().expect("Valid note fixture");
            let tag = Tag::new("#test").expect("Valid tag expected");
            note.tags.push(tag);

            assert_eq!(note.tags.len(), 1, "Note should have 1 tag");
        }

        /// 3.1-UNIT-009: `headings_update_aggregate_state`.
        /// Priority: P1.
        #[test]
        fn headings_update_aggregate_state() {
            let mut note = fixtures::base_note().expect("Valid note fixture");
            let heading = Heading::new(1, "H1".into(), 0)
                .expect("Valid heading expected");
            note.headings.push(heading);

            assert_eq!(note.headings.len(), 1, "Note should have 1 heading");
        }

        /// 3.1-UNIT-009: `tasks_update_aggregate_state`.
        /// Priority: P1.
        #[test]
        fn tasks_update_aggregate_state() {
            let mut note = fixtures::base_note().expect("Valid note fixture");
            let task = Task::new("Task".into(), TaskStatus::Incomplete, 0)
                .expect("Valid task expected");
            note.tasks.push(task);

            assert_eq!(note.tasks.len(), 1, "Note should have 1 task");
        }

        /// 3.1-UNIT-009: `sections_update_aggregate_state`.
        /// Priority: P1.
        #[test]
        fn sections_update_aggregate_state() {
            let mut note = fixtures::base_note().expect("Valid note fixture");
            note.sections.push(Section::new(None, "Body".into(), 0..4));

            assert_eq!(note.sections.len(), 1, "Note should have 1 section");
        }

        /// 3.1-UNIT-009: `links_update_aggregate_state`.
        /// Priority: P1.
        #[test]
        fn links_update_aggregate_state() {
            let mut note = fixtures::base_note().expect("Valid note fixture");
            note.links
                .push(fixtures::wikilink("link.md", 0).expect("Valid link"));
            note.links.push(fixtures::embed("img.png", 1).expect("Valid link"));

            assert_eq!(
                note.links.len(),
                2,
                "Note should have 2 links (unified links)"
            );
        }

        /// 3.1-UNIT-009: `frontmatter_updates_aggregate_state`.
        /// Priority: P1.
        #[test]
        fn frontmatter_updates_aggregate_state() {
            let mut note = fixtures::base_note().expect("Valid note fixture");
            let fm_fields =
                [("title".to_owned(), FieldValue::String("Title".into()))]
                    .into_iter()
                    .collect();
            let frontmatter = Frontmatter::new(fm_fields)
                .expect("Valid frontmatter expected");
            note.set_frontmatter(Some(frontmatter));

            assert!(
                note.frontmatter().is_some(),
                "Note should have frontmatter set"
            );
        }

        /// 3.1-UNIT-010: `filtered_link_iterators_work`.
        /// Priority: P1.
        #[test]
        fn all_links_iterator_returns_total_count() {
            let note =
                fixtures::note_with_link_mix().expect("Valid note fixture");

            let all_count = note.links.len();
            assert_eq!(all_count, 4, "Note should have 4 total links");
        }

        /// 3.1-UNIT-010: `filtered_link_iterators_work`.
        /// Priority: P1.
        #[test]
        fn wikilinks_iterator_returns_wiki_count() {
            let note =
                fixtures::note_with_link_mix().expect("Valid note fixture");

            let wiki_count = note.wikilinks().count();
            assert_eq!(wiki_count, 2, "Note should have 2 wikilinks");
        }

        /// 3.1-UNIT-010: `filtered_link_iterators_work`.
        /// Priority: P1.
        #[test]
        fn markdown_links_iterator_returns_markdown_count() {
            let note =
                fixtures::note_with_link_mix().expect("Valid note fixture");

            let md_count = note.markdown_links().count();
            assert_eq!(md_count, 1, "Note should have 1 markdown link");
        }

        /// 3.1-UNIT-010: `filtered_link_iterators_work`.
        /// Priority: P1.
        #[test]
        fn embeds_iterator_returns_embed_count() {
            let note =
                fixtures::note_with_link_mix().expect("Valid note fixture");

            let embed_count = note.embeds().count();
            assert_eq!(embed_count, 1, "Note should have 1 embed");
        }
    }

    mod constructor {
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
            let test_id = TEST_NOTE_ID;

            // WHEN: creating a note
            let result = Note::new(test_id, path.to_owned());

            // THEN: the result matches the expected outcome
            match expected {
                Ok(()) => assert!(
                    result.is_ok(),
                    "Expected path '{path}' to be valid"
                ),
                Err(NoteError::InvalidPath(_)) => {
                    assert!(
                        matches!(result, Err(NoteError::InvalidPath(_))),
                        "Expected path '{path}' to be invalid, got: {result:?}"
                    );
                }
                Err(e) => {
                    assert!(
                        matches!(e, NoteError::InvalidPath(_)),
                        "Unexpected error kind in matrix: {e:?}"
                    );
                }
            }
        }

        /// 3.1-UNIT-005: `generates_sequential_uuids`.
        /// Priority: P1.
        #[test]
        fn generates_sequential_uuids() {
            // GIVEN: two UUIDs with different timestamps (v7 format embeds
            // timestamp) Use fixed test constants that represent
            // sequential time order
            use super::fixtures::{TEST_NOTE_ID_EARLIER, TEST_NOTE_ID_LATER};

            // WHEN: creating notes with time-ordered UUIDs
            let note1 = Note::new(TEST_NOTE_ID_EARLIER, "one.md".into())
                .expect("Failed to create note fixture");
            let note2 = Note::new(TEST_NOTE_ID_LATER, "two.md".into())
                .expect("Failed to create note fixture");

            // THEN: later UUIDs sort after earlier ones
            assert!(
                note2.id > note1.id,
                "UUIDv7 should maintain time-based ordering: \
                 {TEST_NOTE_ID_EARLIER} < {TEST_NOTE_ID_LATER}"
            );
        }
    }

    mod validate {
        use super::*;

        /// 3.1-UNIT-006: `succeeds_when_all_entities_are_valid`.
        /// Priority: P0.
        #[test]
        fn succeeds_when_all_entities_are_valid() {
            // GIVEN: a note aggregate with consistent sub-entities
            let note_id = TEST_NOTE_ID;
            let tag = Tag::new("#work").expect("Valid tag expected");
            let heading = Heading::new(1, "Title".into(), 0)
                .expect("Valid heading expected");
            let link = Link::new_wikilink(
                Target::Unresolved {
                    raw: "target.md".into(),
                },
                None,
                None,
                0u32.into(),
            )
            .expect("Valid link expected");

            let note = note_fixture(
                note_id,
                "valid.md",
                vec![tag],
                vec![heading],
                vec![link],
            )
            .expect("Valid note fixture expected");

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

    /// Test fixture: Create a Note with custom fields for validation tests.
    /// Direct struct construction bypasses `Note::new()` validation.
    fn note_fixture(
        id: Uuid,
        path: &str,
        tags: Vec<Tag>,
        headings: Vec<Heading>,
        links: Vec<Link>,
    ) -> Result<Note, NoteError> {
        Ok(Note {
            id,
            path: NotePath::new(path.to_owned())?,
            frontmatter: None,
            links,
            tags,
            headings,
            tasks: vec![],
            sections: vec![],
            pending_events: vec![],
        })
    }
}
