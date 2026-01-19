//! Note bounded context aggregate root.
//!
//! This module defines the Note aggregate root that composes subentities
//! from other modules: Frontmatter, Links, Embeds, Tags, Headings, Tasks, and Sections.
//!
//! # Business Rules
//! - Note IDs use UUID v7 for stable, time-ordered identity.
//! - All file paths must be vault-relative and validated against path traversal.
//! - Validation follows a three-phase pipeline: Syntactic → Orchestration → Semantic.

use uuid::Uuid;

use super::{
    events::{NoteCreated, NoteEvents},
    frontmatter::Frontmatter,
    link::Link,
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
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Logical grouping preferred over alphabetical for domain models"
)]
pub struct Note {
    /// UUID v7 identity (time-ordered).
    id: Uuid,
    /// Vault-relative path.
    path: Box<str>,
    /// YAML metadata.
    frontmatter: Option<Frontmatter>,
    /// Outgoing links.
    links: Vec<Link>,
    /// Embedded files.
    embeds: Vec<Link>,
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
    /// Adds an embed to the note, ensuring aggregate consistency.
    #[inline]
    pub fn add_embed(&mut self, mut embed: Link) {
        embed.set_source_note_id(self.id);
        self.embeds.push(embed);
    }

    /// Adds a domain event to the pending events collection.
    #[inline]
    fn add_event(&mut self, event: NoteEvents) {
        self.pending_events.push(event);
    }

    /// Adds a heading to the note.
    #[inline]
    pub fn add_heading(&mut self, heading: Heading) {
        self.headings.push(heading);
    }

    /// Adds a link to the note, ensuring aggregate consistency.
    #[inline]
    pub fn add_link(&mut self, mut link: Link) {
        link.set_source_note_id(self.id);
        self.links.push(link);
    }

    /// Adds a section to the note.
    #[inline]
    pub fn add_section(&mut self, section: Section) {
        self.sections.push(section);
    }

    /// Adds a tag to the note.
    #[inline]
    pub fn add_tag(&mut self, tag: Tag) {
        self.tags.push(tag);
    }

    /// Adds a task to the note.
    #[inline]
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// Returns the embedded files in this note.
    #[inline]
    #[must_use]
    pub fn embeds(&self) -> &[Link] {
        &self.embeds
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

    /// Returns the outgoing links from this note.
    #[inline]
    #[must_use]
    pub fn links(&self) -> &[Link] {
        &self.links
    }

    /// Creates a new note aggregate with the provided UUID and validated path.
    ///
    /// # Errors
    /// Returns `DomainError::EmptyPath` if path is empty.
    /// Returns `DomainError::InvalidPath` if path is absolute, missing `.md` extension, or contains `..`.
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

        let path_box: Box<str> = path.into();
        let path_for_event = path_box.to_string();

        let mut note = Self {
            id,
            path: path_box,
            frontmatter: None,
            links: vec![],
            embeds: vec![],
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
    /// Returns `DomainError::ValidationFailed` if cross-entity invariants are violated.
    #[inline]
    pub fn validate(&self) -> Result<(), DomainError> {
        for link in &self.links {
            if link.source_note_id() != self.id {
                return Err(DomainError::ValidationFailed(
                    "Link source note ID mismatch".to_owned(),
                ));
            }
        }

        for embed in &self.embeds {
            if embed.source_note_id() != self.id {
                return Err(DomainError::ValidationFailed(
                    "Embed source note ID mismatch".to_owned(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod new {
        use tokio::time::Duration;

        use super::*;

        #[test]
        fn returns_error_when_path_is_empty() {
            let test_id = Uuid::now_v7();
            let result = Note::new(test_id, String::new());
            assert!(matches!(result, Err(DomainError::EmptyPath)));
        }

        #[test]
        fn returns_error_when_path_is_absolute() {
            let test_id = Uuid::now_v7();
            let result = Note::new(test_id, "/absolute/path.md".to_owned());
            assert!(matches!(result, Err(DomainError::InvalidPath(_))));
        }

        #[test]
        fn returns_error_when_path_contains_traversal() {
            let test_id = Uuid::now_v7();
            let result = Note::new(test_id, "../etc/passwd".to_owned());
            assert!(matches!(result, Err(DomainError::InvalidPath(_))));
        }

        #[test]
        fn returns_error_when_path_missing_md_extension() {
            let test_id = Uuid::now_v7();
            let result = Note::new(test_id, "projects/lithos".to_owned());
            assert!(matches!(result, Err(DomainError::InvalidPath(_))));
        }

        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test fixture creation")]
        fn generates_sequential_uuids() {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .start_paused(true)
                .build()
                .unwrap();

            rt.block_on(async {
                let note1 = Note::new(Uuid::now_v7(), "one.md".into()).unwrap();
                tokio::time::advance(Duration::from_millis(10)).await;
                let note2 = Note::new(Uuid::now_v7(), "two.md".into()).unwrap();
                assert!(note2.id() > note1.id());
            });
        }
    }

    mod validate {
        use super::*;

        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn succeeds_when_all_entities_are_valid() {
            let note_id = Uuid::now_v7();
            let note = NoteBuilder::new()
                .id(note_id)
                .path("valid.md".into())
                .tags(vec![Tag::parse("#work").expect("Valid tag")])
                .headings(vec![
                    Heading::new(1, "Title".into(), 0).expect("Valid heading"),
                ])
                .links(vec![
                    Link::new_wikilink(note_id, "target.md".into(), None, 0)
                        .expect("Valid target"),
                ])
                .build();

            note.validate().unwrap();
        }

        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn returns_error_when_link_source_note_id_mismatch() {
            let note_id = Uuid::now_v7();
            let other_id = Uuid::now_v7();
            let note = NoteBuilder::new()
                .id(note_id)
                .links(vec![
                    Link::new_wikilink(other_id, "target.md".into(), None, 0)
                        .expect("Valid target"),
                ])
                .build();

            assert!(matches!(
                note.validate(),
                Err(DomainError::ValidationFailed(_))
            ));
        }
    }

    use lithos_test_utils::test_builder;

    test_builder!(NoteBuilder, Note, {
        id: Uuid = Uuid::now_v7(),
        path: Box<str> = "default.md".into(),
        frontmatter: Option<Frontmatter> = None,
        links: Vec<Link> = vec![],
        embeds: Vec<Link> = vec![],
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
    /// Panics if the hardcoded date string is invalid or frontmatter construction fails.
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
            embeds: vec![],
            tags: vec![example_tag()],
            headings: vec![],
            tasks: vec![],
            sections: vec![],
            pending_events: vec![],
        }
    }
}
