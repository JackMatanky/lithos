//! Note aggregate root and core domain entities.
//!
//! The Note aggregate represents an Obsidian-compatible markdown note and owns
//! all note-local entities such as links, tags, headings, and tasks.
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use rkyv::{Archive, Deserialize, Serialize};

use super::{
    error::NoteError,
    events::NoteEvents,
    frontmatter::Frontmatter,
    link::Link,
    list::List,
    structure::{Heading, Section},
    tag::{ArchivedTag, Tag},
    task::Task,
};

/// Represents an Obsidian-compatible markdown note.
///
/// `Note` is the aggregate root for the note bounded context. It maintains
/// consistency across its sub-entities and stages domain events.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Note {
    /// Unique identity (UUID v7, time-ordered).
    id: NoteId,
    /// Vault-relative path.
    path: NotePath,
    /// All links (wiki-links, markdown links, and embeds).
    links: Vec<Link>,
    /// Hierarchical tags.
    tags: Vec<Tag>,
    /// Markdown headings.
    headings: Vec<Heading>,
    /// Markdown lists.
    lists: Vec<List>,
    /// Task items.
    tasks: Vec<Task>,
    /// Document sections.
    sections: Vec<Section>,
    /// YAML metadata.
    frontmatter: Option<Frontmatter>,
    /// Domain events pending emission (not serialized).
    #[rkyv(with = rkyv::with::Skip)]
    #[serde(skip)]
    pending_events: Vec<NoteEvents>,
}

impl Note {
    /// Creates a new Note with the given ID and path.
    ///
    /// # Errors
    /// Returns [`NoteError::InvalidPath`] if the path validation fails.
    #[inline]
    pub fn new(id: NoteId, path: String) -> Result<Self, NoteError> {
        Ok(Self {
            id,
            path: NotePath::new(path)?,
            links: Vec::new(),
            tags: Vec::new(),
            headings: Vec::new(),
            lists: Vec::new(),
            tasks: Vec::new(),
            sections: Vec::new(),
            frontmatter: None,
            pending_events: Vec::new(),
        })
    }

    /// Returns the note's unique identifier.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> NoteId {
        self.id
    }

    /// Returns the note's vault-relative path.
    #[inline]
    #[must_use]
    pub const fn path(&self) -> &NotePath {
        &self.path
    }

    /// Sets the note's vault-relative path.
    #[inline]
    pub fn set_path(&mut self, path: NotePath) {
        self.path = path;
    }

    /// Returns the note's frontmatter metadata, if any.
    #[inline]
    #[must_use]
    pub const fn frontmatter(&self) -> Option<&Frontmatter> {
        self.frontmatter.as_ref()
    }

    /// Sets the note's frontmatter metadata.
    #[inline]
    pub fn set_frontmatter(&mut self, frontmatter: Option<Frontmatter>) {
        self.frontmatter = frontmatter;
    }

    /// Adds a domain event to the pending events collection.
    #[inline]
    pub fn add_event(&mut self, event: NoteEvents) {
        self.pending_events.push(event);
    }

    /// Returns an iterator over all links in this note.
    #[inline]
    pub fn links(&self) -> impl Iterator<Item = &Link> {
        self.links.iter()
    }

    /// Adds a link to the note.
    #[inline]
    pub fn add_link(&mut self, link: Link) {
        self.links.push(link);
    }

    /// Returns an iterator over all tags in this note.
    #[inline]
    pub fn tags(&self) -> impl Iterator<Item = &Tag> {
        self.tags.iter()
    }

    /// Adds a tag to the note.
    #[inline]
    pub fn add_tag(&mut self, tag: Tag) {
        self.tags.push(tag);
    }

    /// Returns an iterator over all headings in this note.
    #[inline]
    pub fn headings(&self) -> impl Iterator<Item = &Heading> {
        self.headings.iter()
    }

    /// Adds a heading to the note.
    #[inline]
    pub fn add_heading(&mut self, heading: Heading) {
        self.headings.push(heading);
    }

    /// Returns an iterator over all tasks in this note.
    #[inline]
    pub fn tasks(&self) -> impl Iterator<Item = &Task> {
        self.tasks.iter()
    }

    /// Adds a task to the note.
    #[inline]
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// Returns an iterator over all lists in this note.
    #[inline]
    pub fn lists(&self) -> impl Iterator<Item = &List> {
        self.lists.iter()
    }

    /// Adds a list to the note.
    #[inline]
    pub fn add_list(&mut self, list: List) {
        self.lists.push(list);
    }

    /// Returns an iterator over all sections in this note.
    #[inline]
    pub fn sections(&self) -> impl Iterator<Item = &Section> {
        self.sections.iter()
    }

    /// Adds a section to the note.
    #[inline]
    pub fn add_section(&mut self, section: Section) {
        self.sections.push(section);
    }

    /// Returns all embeds in this note.
    #[inline]
    pub fn embeds(&self) -> impl Iterator<Item = &Link> {
        self.links.iter().filter(|l| l.is_embed())
    }

    /// Takes all pending domain events, leaving the collection empty.
    #[inline]
    #[must_use]
    pub fn take_events(&mut self) -> Vec<NoteEvents> {
        std::mem::take(&mut self.pending_events)
    }
}

/// Unique identifier for a Note.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct NoteId(uuid::Uuid);

impl NoteId {
    /// Creates a new random `NoteId` (UUID v7).
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl Default for NoteId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<uuid::Uuid> for NoteId {
    #[inline]
    fn from(uuid: uuid::Uuid) -> Self {
        Self(uuid)
    }
}

impl From<NoteId> for uuid::Uuid {
    #[inline]
    fn from(id: NoteId) -> uuid::Uuid {
        id.0
    }
}

/// Validated vault-relative path for a note.
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
pub struct NotePath(Box<str>);

impl NotePath {
    /// Creates a new `NotePath` with validation.
    ///
    /// # Errors
    /// Returns [`NoteError::InvalidPath`] if the path is invalid.
    #[inline]
    pub fn new(path: String) -> Result<Self, NoteError> {
        // Basic normalization: convert backslashes to forward slashes
        // Avoid allocation if no backslashes
        let normalized = if path.contains('\\') {
            path.replace('\\', "/")
        } else {
            path
        };

        // Use core filesystem validator
        crate::fs::validate_vault_path(&normalized, Some("md"))
            .map_err(|e| NoteError::InvalidPath(e.clone()))?;

        Ok(Self(normalized.into()))
    }

    /// Returns the path as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
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

impl ArchivedNote {
    /// Returns the note's vault-relative path.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &ArchivedNotePath {
        &self.path
    }

    /// Returns the note's tags.
    #[inline]
    #[must_use]
    pub fn tags(&self) -> &rkyv::vec::ArchivedVec<ArchivedTag> {
        &self.tags
    }
}

impl ArchivedNotePath {
    /// Returns the path as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    clippy::arbitrary_source_item_ordering,
    reason = "Tests use assertions in Result-returning functions and group \
              items for readability."
)]
mod tests {
    use super::*;

    mod fixtures {
        use uuid::Uuid;

        use super::super::{Note, NoteId};
        use crate::note::{
            error::NoteError,
            link::{EmbedType, Link, Target},
            types::SourceByteOffset,
        };

        /// Fixed UUID for deterministic tests (valid UUID v7 format).
        pub const TEST_NOTE_ID: Uuid =
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0001);

        pub fn base_note() -> Result<Note, NoteError> {
            Note::new(NoteId::from(TEST_NOTE_ID), "note.md".to_owned())
        }

        pub fn wikilink(raw: &str, pos: u32) -> Result<Link, NoteError> {
            Link::new_wikilink(
                Target::Unresolved {
                    raw: raw.into(),
                },
                None,
                None,
                SourceByteOffset::new(pos),
            )
        }

        pub fn embed(raw: &str, pos: u32) -> Result<Link, NoteError> {
            Link::new_embed(
                Target::Unresolved {
                    raw: raw.into(),
                },
                EmbedType::Image,
                None,
                SourceByteOffset::new(pos),
            )
        }

        pub fn note_with_link_mix() -> Result<Note, NoteError> {
            let mut note = base_note()?;

            note.add_link(wikilink("wiki1.md", 0)?);
            note.add_link(wikilink("wiki2.md", 10)?);
            let md = Link::new_markdown_link(
                Target::External {
                    url: "https://example.com".into(),
                },
                None,
                None,
                SourceByteOffset::new(20),
            )?;
            note.add_link(md);
            note.add_link(embed("img.png", 30)?);

            Ok(note)
        }
    }

    use fixtures::TEST_NOTE_ID;

    use crate::note::{
        list::{List, ListType},
        types::SourceByteOffset,
    };

    mod accessors {
        use super::*;
        use crate::note::structure::HeadingLevel;

        #[test]
        fn tags_update_aggregate_state() -> Result<(), NoteError> {
            let mut note = fixtures::base_note()?;
            let tag = Tag::new("#test")?;
            note.add_tag(tag);

            assert_eq!(note.tags().count(), 1, "Note should have 1 tag");
            Ok(())
        }

        #[test]
        fn headings_update_aggregate_state() -> Result<(), NoteError> {
            let mut note = fixtures::base_note()?;
            let heading = Heading::new(
                HeadingLevel::try_new(1)?,
                "H1",
                SourceByteOffset::new(0),
            )?;
            note.add_heading(heading);

            assert_eq!(
                note.headings().count(),
                1,
                "Note should have 1 heading"
            );
            Ok(())
        }

        #[test]
        fn lists_update_aggregate_state() -> Result<(), NoteError> {
            let mut note = fixtures::base_note()?;
            let list = List::new(ListType::Unordered);
            note.add_list(list);

            assert_eq!(note.lists().count(), 1, "Note should have 1 list");
            Ok(())
        }

        #[test]
        fn embeds_filter_correctly() -> Result<(), NoteError> {
            let note = fixtures::note_with_link_mix()?;
            assert_eq!(note.embeds().count(), 1, "Should only have one embed");
            Ok(())
        }

        #[test]
        fn take_events_clears_pending() -> Result<(), NoteError> {
            let mut note = fixtures::base_note()?;
            note.add_event(NoteEvents::NoteCreated(
                crate::note::events::NoteCreated::new(
                    NoteId::from(TEST_NOTE_ID).into(),
                    "note.md".to_owned(),
                    0,
                ),
            ));

            let events = note.take_events();
            assert_eq!(events.len(), 1);
            assert!(note.take_events().is_empty());
            Ok(())
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn rejects_absolute_path() {
            let result = Note::new(NoteId::new(), "/absolute.md".to_owned());
            result.unwrap_err();
        }

        #[test]
        fn rejects_wrong_extension() {
            let result = Note::new(NoteId::new(), "note.txt".to_owned());
            result.unwrap_err();
        }

        #[test]
        fn accepts_valid_vault_path() -> Result<(), NoteError> {
            let note = Note::new(NoteId::new(), "folder/note.md".to_owned())?;
            assert_eq!(note.path().as_str(), "folder/note.md");
            Ok(())
        }
    }
}
