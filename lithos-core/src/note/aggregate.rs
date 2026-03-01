//! Note aggregate root and primary domain entity.
//!
//! This module defines the [`crate::note::aggregate::Note`] aggregate, which
//! serves as the central coordination point for all note-related data including
//! links, tags, and tasks.

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

use std::fmt;

use rkyv::{Archive, Deserialize, Serialize};
use uuid::Uuid;

use super::{
    error::{NoteError, NoteMetadataError},
    events::NoteEvents,
    frontmatter::Frontmatter,
    link::Link,
    list::List,
    structure::{Heading, Section},
    tag::Tag,
    task::Task,
};

/// Represents an Obsidian-compatible markdown note.
///
/// `Note` is the aggregate root for the note bounded context. It maintains
/// consistency across its sub-entities (links, tags, tasks, etc.) and stages
/// domain events for persistence.
///
/// This struct uses zero-copy serialization via `rkyv` and is designed for
/// high-performance indexing and retrieval.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::aggregate::{Note, NoteId};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let id = NoteId::new();
/// let path = "inbox/meeting-notes.md";
///
/// let note = Note::new(id, path)?;
/// assert_eq!(note.path().as_str(), "inbox/meeting-notes.md");
/// # Ok(())
/// # }
/// ```
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
    /// Parsed content and indexable structures.
    content: NoteContent,
    /// YAML metadata.
    frontmatter: Option<Frontmatter>,
    /// Domain events pending emission (not serialized).
    #[rkyv(with = rkyv::with::Skip)]
    #[serde(skip)]
    pending_events: Vec<NoteEvents>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
struct NoteContent {
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
}

impl NoteContent {
    fn new() -> Self {
        Self::default()
    }
}

impl Note {
    /// Creates a new [`Note`] with the given ID and path.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::InvalidPath`] if the path validation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::aggregate::{Note, NoteId};
    /// let id = NoteId::new();
    /// let note = Note::new(id, "test.md").unwrap();
    /// ```
    #[inline]
    pub fn new(id: NoteId, path: &str) -> Result<Self, NoteError> {
        Ok(Self {
            id,
            path: NotePath::try_from(path)?,
            content: NoteContent::new(),
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
        self.content.links.iter()
    }

    /// Adds a link to the note.
    #[inline]
    pub fn add_link(&mut self, link: Link) {
        self.content.links.push(link);
    }

    /// Returns an iterator over all tags in this note.
    #[inline]
    pub fn tags(&self) -> impl Iterator<Item = &Tag> {
        self.content.tags.iter()
    }

    /// Adds a tag to the note.
    #[inline]
    pub fn add_tag(&mut self, tag: Tag) {
        self.content.tags.push(tag);
    }

    /// Returns an iterator over all headings in this note.
    #[inline]
    pub fn headings(&self) -> impl Iterator<Item = &Heading> {
        self.content.headings.iter()
    }

    /// Adds a heading to the note.
    #[inline]
    pub fn add_heading(&mut self, heading: Heading) {
        self.content.headings.push(heading);
    }

    /// Returns an iterator over all tasks in this note.
    #[inline]
    pub fn tasks(&self) -> impl Iterator<Item = &Task> {
        self.content.tasks.iter()
    }

    /// Adds a task to the note.
    #[inline]
    pub fn add_task(&mut self, task: Task) {
        self.content.tasks.push(task);
    }

    /// Returns an iterator over all lists in this note.
    #[inline]
    pub fn lists(&self) -> impl Iterator<Item = &List> {
        self.content.lists.iter()
    }

    /// Adds a list to the note.
    #[inline]
    pub fn add_list(&mut self, list: List) {
        self.content.lists.push(list);
    }

    /// Returns an iterator over all sections in this note.
    #[inline]
    pub fn sections(&self) -> impl Iterator<Item = &Section> {
        self.content.sections.iter()
    }

    /// Adds a section to the note.
    #[inline]
    pub fn add_section(&mut self, section: Section) {
        self.content.sections.push(section);
    }

    /// Returns all embeds in this note.
    #[inline]
    pub fn embeds(&self) -> impl Iterator<Item = &Link> {
        self.content.links.iter().filter(|l| l.is_embed())
    }

    /// Takes all pending domain events, leaving the collection empty.
    #[inline]
    #[must_use]
    pub fn take_events(&mut self) -> Vec<NoteEvents> {
        std::mem::take(&mut self.pending_events)
    }
}

/// Unique identifier for a Note.
///
/// Uses UUID v7 for time-ordered, sortable identities that are well-suited
/// for database primary keys.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::aggregate::NoteId;
/// let id = NoteId::new();
/// println!("Created note with ID: {:?}", id);
/// ```
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
    /// Creates a new random note identifier (UUID v7).
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Parses a note identifier from a string.
    ///
    /// # Errors
    /// Returns [`uuid::Error`] if the string is not a valid UUID.
    #[inline]
    pub fn parse(id: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(id)?))
    }
}

impl fmt::Display for NoteId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
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
///
/// Ensures the path follows Obsidian conventions (e.g., forward slashes,
/// `.md` extension) and prevents directory traversal attacks.
///
/// # Errors
///
/// Returns [`NoteError::InvalidPath`] if:
/// - The path is absolute (starts with `/`).
/// - The extension is not `.md`.
/// - The path contains invalid characters.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::aggregate::NotePath;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let path = NotePath::new("daily/2024-01-01.md")?;
/// assert_eq!(path.as_str(), "daily/2024-01-01.md");
/// # Ok(())
/// # }
/// ```
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
    /// Creates a new [`NotePath`] with validation.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::InvalidPath`] if the path is invalid.
    #[inline]
    pub fn new(path: &str) -> Result<Self, NoteError> {
        let normalized = normalize_note_path(path);
        validate_note_path(normalized.as_ref())?;
        Ok(Self(normalized.into()))
    }

    /// Returns the path as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NotePath {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Validated alias name for a note.
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
pub struct AliasName(Box<str>);

impl AliasName {
    /// Creates a validated alias name.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Metadata`] if the alias is empty.
    #[inline]
    pub fn try_new(value: &str) -> Result<Self, NoteError> {
        if value.trim().is_empty() {
            return Err(NoteError::Metadata(NoteMetadataError::AliasEmpty));
        }
        Ok(Self(value.trim().into()))
    }

    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AliasName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Validated file class name for a note.
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
pub struct FileClassName(Box<str>);

impl FileClassName {
    /// Creates a validated file class name.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Metadata`] if the class is empty.
    #[inline]
    pub fn try_new(value: &str) -> Result<Self, NoteError> {
        if value.trim().is_empty() {
            return Err(NoteError::Metadata(NoteMetadataError::FileClassEmpty));
        }
        Ok(Self(value.trim().into()))
    }

    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FileClassName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Validated folder path within the vault.
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
pub struct FolderPath(Box<str>);

impl FolderPath {
    /// Creates a validated folder path.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Metadata`] if the folder is empty.
    #[inline]
    pub fn try_new(value: &str) -> Result<Self, NoteError> {
        if value.trim().is_empty() {
            return Err(NoteError::Metadata(NoteMetadataError::FolderEmpty));
        }
        Ok(Self(value.trim().into()))
    }

    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FolderPath {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for NotePath {
    type Error = NoteError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for NotePath {
    type Error = NoteError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

fn has_drive_or_unc_prefix(path: &str) -> bool {
    let mut chars = path.chars();
    let first = chars.next();
    let second = chars.next();
    let Some(first) = first else {
        return false;
    };
    if let Some(second) = second {
        if first.is_ascii_alphabetic() && second == ':' {
            return true;
        }
        if first == '/' && second == '/' {
            return true;
        }
    }
    false
}

fn has_curdir_segment(path: &str) -> bool {
    if path == "." || path.starts_with("./") || path.ends_with("/.") {
        return true;
    }

    let bytes = path.as_bytes();
    bytes.windows(3).any(|window| window == b"/./")
}

fn normalize_note_path(path: &str) -> std::borrow::Cow<'_, str> {
    if path.contains('\\') {
        let mut owned = String::with_capacity(path.len());
        for ch in path.chars() {
            if ch == '\\' {
                owned.push('/');
            } else {
                owned.push(ch);
            }
        }
        std::borrow::Cow::Owned(owned)
    } else {
        std::borrow::Cow::Borrowed(path)
    }
}

fn validate_note_path(path: &str) -> Result<(), NoteError> {
    if path.is_empty() {
        return Err(NoteError::InvalidPath("path cannot be empty".into()));
    }

    if has_curdir_segment(path) {
        return Err(NoteError::InvalidPath(
            "path must not include '.' components".into(),
        ));
    }

    if has_drive_or_unc_prefix(path) {
        return Err(NoteError::InvalidPath(
            "windows-style prefixes are not allowed".into(),
        ));
    }

    let normalized_path = std::path::Path::new(path);
    if normalized_path.is_absolute() {
        return Err(NoteError::InvalidPath("path must be relative".into()));
    }

    for component in normalized_path.components() {
        match component {
            std::path::Component::ParentDir => {
                return Err(NoteError::InvalidPath(
                    "path traversal not allowed".into(),
                ));
            }
            std::path::Component::CurDir => {
                return Err(NoteError::InvalidPath(
                    "path must not include '.' components".into(),
                ));
            }
            std::path::Component::Normal(segment) => {
                if segment.to_str().is_some_and(|segment| segment == ".") {
                    return Err(NoteError::InvalidPath(
                        "path must not include '.' components".into(),
                    ));
                }
                if segment
                    .to_str()
                    .is_some_and(|segment| segment.starts_with('.'))
                {
                    return Err(NoteError::InvalidPath(
                        "hidden path components not allowed".into(),
                    ));
                }
            }
            std::path::Component::Prefix(_) | std::path::Component::RootDir => {
                return Err(NoteError::InvalidPath(
                    "path must be relative".into(),
                ));
            }
        }
    }

    if !normalized_path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        return Err(NoteError::InvalidPath(
            "path must have .md extension".into(),
        ));
    }

    Ok(())
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
            Note::new(NoteId::from(TEST_NOTE_ID), "note.md")
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
                    "note.md",
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
            let result = Note::new(NoteId::new(), "/absolute.md");
            result.unwrap_err();
        }

        #[test]
        fn rejects_curdir_components() {
            let result = NotePath::new("folder/./note.md");
            result.unwrap_err();
        }

        #[test]
        fn rejects_wrong_extension() {
            let result = Note::new(NoteId::new(), "note.txt");
            result.unwrap_err();
        }

        #[test]
        fn rejects_windows_drive_prefix() {
            let result = NotePath::new("C:notes/note.md");
            result.unwrap_err();
        }

        #[test]
        fn rejects_unc_prefix() {
            let result = NotePath::new("//server/share/note.md");
            result.unwrap_err();
        }

        #[test]
        fn accepts_valid_vault_path() -> Result<(), NoteError> {
            let note = Note::new(NoteId::new(), "folder/note.md")?;
            assert_eq!(note.path().as_str(), "folder/note.md");
            Ok(())
        }
    }
}
