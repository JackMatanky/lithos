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
    events::{NoteCreated, NoteEvents},
    frontmatter::Frontmatter,
    link::{Link, LinkType},
    structure::{Heading, Section},
    tag::Tag,
    task::Task,
};
use crate::{errors::DomainError, validation::validate_vault_path};

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
/// # use lithos_domain::Note;
/// # use uuid::Uuid;
/// // For new files (first-time indexing)
/// let new_id = Uuid::now_v7();
/// let note = Note::new(new_id, "projects/example.md".to_string()).unwrap();
/// assert_eq!(note.path(), "projects/example.md");
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Note {
    /// UUID v7 identity (time-ordered).
    id: Uuid,
    /// Vault-relative path.
    path: Box<str>,
    /// YAML metadata.
    frontmatter: Option<Frontmatter>,
    /// All links (wiki-links, markdown links, and embeds).
    links: Vec<Link>,
    /// Hierarchical tags.
    tags: Vec<Tag>,
    /// Markdown headings.
    headings: Vec<Heading>,
    /// Task items.
    tasks: Vec<Task>,
    /// Document sections.
    sections: Vec<Section>,
    /// Domain events pending emission (not serialized).
    #[serde(skip)]
    pending_events: Vec<NoteEvents>,
}

impl Note {
    /// Adds a domain event to the pending events collection.
    #[inline]
    pub fn add_event(&mut self, event: NoteEvents) {
        self.pending_events.push(event);
    }

    /// Adds a heading to the note.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::{Note, Heading};
    /// # use uuid::Uuid;
    /// let mut note = Note::new(Uuid::now_v7(), "note.md".to_string()).unwrap();
    /// note.add_heading(Heading::new(1, "Title".to_string(), 0).unwrap());
    /// assert_eq!(note.headings().len(), 1);
    /// ```
    #[inline]
    pub fn add_heading(&mut self, heading: Heading) {
        self.headings.push(heading);
    }

    /// Adds a link to the note.
    ///
    /// This method accepts any link type (wiki-link, markdown link, or embed).
    /// The link becomes owned by this note aggregate.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::{Note, Link, LinkTarget};
    /// # use uuid::Uuid;
    /// let mut note = Note::new(Uuid::now_v7(), "note.md".to_string()).unwrap();
    /// let link = Link::new_wikilink(
    ///     LinkTarget::Unresolved {
    ///         raw: "target.md".into(),
    ///     },
    ///     None,
    ///     None,
    ///     0,
    /// )
    /// .unwrap();
    /// note.add_link(link);
    /// assert_eq!(note.links().len(), 1);
    /// ```
    #[inline]
    pub fn add_link(&mut self, link: Link) {
        self.links.push(link);
    }

    /// Adds a section to the note.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::{Note, Section};
    /// # use uuid::Uuid;
    /// let mut note = Note::new(Uuid::now_v7(), "note.md".to_string()).unwrap();
    /// note.add_section(Section::new(None, "content".to_string(), 0..7));
    /// assert_eq!(note.sections().len(), 1);
    /// ```
    #[inline]
    pub fn add_section(&mut self, section: Section) {
        self.sections.push(section);
    }

    /// Adds a tag to the note.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::{Note, Tag};
    /// # use uuid::Uuid;
    /// let mut note = Note::new(Uuid::now_v7(), "note.md".to_string()).unwrap();
    /// note.add_tag(Tag::parse("#work").unwrap());
    /// assert_eq!(note.tags().len(), 1);
    /// ```
    #[inline]
    pub fn add_tag(&mut self, tag: Tag) {
        self.tags.push(tag);
    }

    /// Adds a task to the note.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::{Note, Task, TaskStatus};
    /// # use uuid::Uuid;
    /// let mut note = Note::new(Uuid::now_v7(), "note.md".to_string()).unwrap();
    /// let task =
    ///     Task::new("todo".to_string(), TaskStatus::Incomplete, 0).unwrap();
    /// note.add_task(task);
    /// assert_eq!(note.tasks().len(), 1);
    /// ```
    #[inline]
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// Returns all embeds in this note.
    ///
    /// This is a convenience method that filters links by type.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::{Note, Link, LinkTarget, EmbedType};
    /// # use uuid::Uuid;
    /// let mut note = Note::new(Uuid::now_v7(), "note.md".to_string()).unwrap();
    /// let embed = Link::new_embed(
    ///     LinkTarget::Unresolved {
    ///         raw: "img.png".into(),
    ///     },
    ///     EmbedType::Image,
    ///     None,
    ///     0,
    /// )
    /// .unwrap();
    /// note.add_link(embed);
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

    /// Returns the markdown headings in this note.
    #[inline]
    #[must_use]
    pub fn headings(&self) -> &[Heading] {
        &self.headings
    }

    /// Returns the note's unique identifier.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Returns all links in this note (wiki-links, markdown links, and embeds).
    #[inline]
    #[must_use]
    pub fn links(&self) -> &[Link] {
        &self.links
    }

    /// Returns all markdown-style links in this note.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::{Note, Link, LinkTarget};
    /// # use uuid::Uuid;
    /// let mut note = Note::new(Uuid::now_v7(), "note.md".to_string()).unwrap();
    /// let link = Link::new_markdown_link(
    ///     LinkTarget::External {
    ///         url: "https://example.com".into(),
    ///     },
    ///     Some("Example".to_string()),
    ///     None,
    ///     0,
    /// )
    /// .unwrap();
    /// note.add_link(link);
    /// assert_eq!(note.markdown_links().count(), 1);
    /// ```
    #[inline]
    pub fn markdown_links(&self) -> impl Iterator<Item = &Link> {
        self.links.iter().filter(|l| l.link_type() == LinkType::MdLink)
    }

    /// Creates a new note aggregate with the provided UUID and validated path.
    ///
    /// # Errors
    /// Returns `DomainError::EmptyPath` if path is empty.
    /// Returns `DomainError::InvalidPath` if path is absolute, missing `.md`
    /// extension, or contains `..`.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::Note;
    /// # use uuid::Uuid;
    /// let note = Note::new(Uuid::now_v7(), "notes/intro.md".to_string()).unwrap();
    /// assert_eq!(note.path(), "notes/intro.md");
    /// ```
    #[inline]
    pub fn new(id: Uuid, path: String) -> Result<Self, DomainError> {
        validate_vault_path(&path, Some("md"))?;

        let path_for_event = path.clone();
        let path_box: Box<str> = path.into();

        let mut note = Self {
            id,
            path: path_box,
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

    /// Returns the note's vault-relative path.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns a reference to pending domain events without clearing them.
    #[inline]
    #[must_use]
    pub fn pending_events(&self) -> &[NoteEvents] {
        &self.pending_events
    }

    /// Returns the document sections in this note.
    #[inline]
    #[must_use]
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    /// Sets the note's frontmatter.
    #[inline]
    pub fn set_frontmatter(&mut self, frontmatter: Option<Frontmatter>) {
        self.frontmatter = frontmatter;
    }

    /// Returns the hierarchical tags associated with this note.
    #[inline]
    #[must_use]
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    /// Returns all pending domain events and clears the collection.
    #[inline]
    #[must_use]
    pub fn take_events(&mut self) -> Vec<NoteEvents> {
        std::mem::take(&mut self.pending_events)
    }

    /// Returns the task items in this note.
    #[inline]
    #[must_use]
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Validates the note's internal consistency.
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` if cross-entity invariants are
    /// violated (e.g., invalid link configurations).
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::Note;
    /// # use uuid::Uuid;
    /// let note_id = Uuid::now_v7();
    /// let note = Note::new(note_id, "test.md".to_string()).unwrap();
    /// note.validate().unwrap();
    /// ```
    #[inline]
    pub fn validate(&self) -> Result<(), DomainError> {
        for link in &self.links {
            link.validate().map_err(|e| {
                DomainError::ValidationFailed(format!("Invalid link: {e}"))
            })?;
        }

        Ok(())
    }

    /// Returns all wiki-style links in this note.
    ///
    /// # Examples
    /// ```
    /// # use lithos_domain::{Note, Link, LinkTarget};
    /// # use uuid::Uuid;
    /// let mut note = Note::new(Uuid::now_v7(), "note.md".to_string()).unwrap();
    /// let link = Link::new_wikilink(
    ///     LinkTarget::Unresolved {
    ///         raw: "other-note".into(),
    ///     },
    ///     None,
    ///     None,
    ///     0,
    /// )
    /// .unwrap();
    /// note.add_link(link);
    /// assert_eq!(note.wikilinks().count(), 1);
    /// ```
    #[inline]
    pub fn wikilinks(&self) -> impl Iterator<Item = &Link> {
        self.links.iter().filter(|l| l.link_type() == LinkType::WikiLink)
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    clippy::panic,
    reason = "Test module organization and behavior verification patterns"
)]
mod tests {
    // # LINT_DISABLE_REASON: Standard test utilities and behavioral
    // verification patterns.
    use lithos_test_utils::assert_err_kind;

    use super::*;
    use crate::note::{
        frontmatter::FieldValue,
        link::{EmbedType, LinkTarget},
        task::TaskStatus,
    };

    mod new {
        use rstest::rstest;
        use tokio::time::Duration;

        use super::*;

        /// 3.1-UNIT-001: Note Path Validation Matrix.
        /// Priority: P0.
        #[rstest]
        #[case::empty("", Err(DomainError::EmptyPath))]
        #[case::absolute(
            "/absolute/path.md",
            Err(DomainError::InvalidPath("Path must be relative".into()))
        )]
        #[case::traversal(
            "../etc/passwd",
            Err(DomainError::InvalidPath("Path traversal not allowed".into()))
        )]
        #[case::missing_extension(
            "projects/lithos",
            Err(DomainError::InvalidPath("Path must end with .md".into()))
        )]
        #[case::valid("valid.md", Ok(()))]
        #[case::nested_valid("folder/sub/note.md", Ok(()))]
        fn path_validation_matrix(
            #[case] path: &str,
            #[case] expected: Result<(), DomainError>,
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
                Err(DomainError::EmptyPath) => {
                    assert_err_kind!(result, DomainError::EmptyPath);
                }
                Err(DomainError::InvalidPath(_)) => {
                    assert_err_kind!(result, DomainError::InvalidPath(_));
                }
                Err(e) => panic!("Unexpected error kind in matrix: {e:?}"),
            }
        }

        /// 3.1-UNIT-005: `generates_sequential_uuids`.
        /// Priority: P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test fixture creation")]
        fn generates_sequential_uuids() {
            // GIVEN: a paused tokio runtime for deterministic UUID time
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .start_paused(true)
                .build()
                .unwrap();

            // WHEN: creating notes across advanced time
            rt.block_on(async {
                let note1 = Note::new(Uuid::now_v7(), "one.md".into()).unwrap();
                tokio::time::advance(Duration::from_millis(10)).await;
                let note2 = Note::new(Uuid::now_v7(), "two.md".into()).unwrap();

                // THEN: later UUIDs sort after earlier ones
                assert!(note2.id() > note1.id());
            });
        }
    }

    mod validate {
        use super::*;

        /// 3.1-UNIT-006: `succeeds_when_all_entities_are_valid`.
        /// Priority: P0.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn succeeds_when_all_entities_are_valid() {
            // GIVEN: a note aggregate with consistent sub-entities
            let note_id = Uuid::now_v7();
            let note = NoteBuilder::new()
                .id(note_id)
                .path("valid.md".into())
                .tags(vec![Tag::parse("#work").expect("Valid tag")])
                .headings(vec![
                    Heading::new(1, "Title".into(), 0).expect("Valid heading"),
                ])
                .links(vec![
                    Link::new_wikilink(
                        LinkTarget::Unresolved {
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
            result.expect("Validation should pass");
        }
    }

    mod accessors {
        use super::*;

        /// 3.1-UNIT-009: `mutators_update_aggregate_state`.
        /// Priority: P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn mutators_update_aggregate_state() {
            // GIVEN: a basic note
            let note_id = Uuid::now_v7();
            let mut note = Note::new(note_id, "note.md".to_owned()).unwrap();

            // WHEN: adding various sub-entities
            note.add_tag(Tag::parse("#test").unwrap());
            note.add_heading(Heading::new(1, "H1".into(), 0).unwrap());
            note.add_task(
                Task::new("Task".into(), TaskStatus::Incomplete, 0).unwrap(),
            );
            note.add_section(Section::new(None, "Body".into(), 0..4));

            // Add a wikilink
            note.add_link(
                Link::new_wikilink(
                    LinkTarget::Unresolved {
                        raw: "link.md".into(),
                    },
                    None,
                    None,
                    0,
                )
                .unwrap(),
            );

            // Add an embed
            note.add_link(
                Link::new_embed(
                    LinkTarget::Unresolved {
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
            assert_eq!(note.tags().len(), 1);
            assert_eq!(note.headings().len(), 1);
            assert_eq!(note.tasks().len(), 1);
            assert_eq!(note.sections().len(), 1);
            assert_eq!(note.links().len(), 2); // unified links
            assert_eq!(note.wikilinks().count(), 1);
            assert_eq!(note.embeds().count(), 1);
            assert!(note.frontmatter().is_some());
        }

        /// 3.1-UNIT-010: `filtered_link_iterators_work`.
        /// Priority: P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn filtered_link_iterators_work() {
            // GIVEN: a note with various link types (2 wikilinks, 1 markdown, 1
            // embed)
            let mut note =
                Note::new(Uuid::now_v7(), "note.md".to_owned()).unwrap();

            note.add_link(
                Link::new_wikilink(
                    LinkTarget::Unresolved {
                        raw: "wiki1.md".into(),
                    },
                    None,
                    None,
                    0,
                )
                .unwrap(),
            );
            note.add_link(
                Link::new_wikilink(
                    LinkTarget::Unresolved {
                        raw: "wiki2.md".into(),
                    },
                    None,
                    None,
                    10,
                )
                .unwrap(),
            );
            note.add_link(
                Link::new_markdown_link(
                    LinkTarget::External {
                        url: "https://example.com".into(),
                    },
                    None,
                    None,
                    20,
                )
                .unwrap(),
            );
            note.add_link(
                Link::new_embed(
                    LinkTarget::Unresolved {
                        raw: "img.png".into(),
                    },
                    EmbedType::Image,
                    None,
                    30,
                )
                .unwrap(),
            );

            // WHEN: using filtered iterators to query specific link types
            let all_count = note.links().len();
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

    use lithos_test_utils::test_builder;

    test_builder!(NoteBuilder, Note, {
        id: Uuid = Uuid::now_v7(),
        path: Box<str> = "default.md".into(),
        frontmatter: Option<Frontmatter> = None,
        links: Vec<Link> = vec![],
        tags: Vec<Tag> = vec![],
        headings: Vec<Heading> = vec![],
        tasks: Vec<Task> = vec![],
        sections: Vec<Section> = vec![],
        pending_events: Vec<NoteEvents> = vec![],
    });
}

#[cfg(test)]
#[expect(dead_code, reason = "Test fixtures may be used by other crates")]
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
        Tag::parse("#work/project").expect("Valid tag")
    }

    /// Test fixture: Create complete example Note aggregate for testing.
    #[inline]
    #[must_use]
    pub fn example_note() -> Note {
        Note {
            id: TEST_NOTE_ID,
            path: "test/example.md".into(),
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
