//! Note domain entities and business logic.
//!
//! This module defines the Note aggregate root and its associated subentities:
//! Frontmatter, Links, Embeds, Tags, Headings, Tasks, and Sections.
//!
//! # Business Rules
//! - Note IDs use UUID v7 for stable, time-ordered identity.
//! - All file paths must be vault-relative and validated against path traversal.
//! - Tags are hierarchical and follow specific format rules.
//! - Validation follows a three-phase pipeline: Syntactic → Orchestration → Semantic.

use std::ops::Range;

use uuid::Uuid;

use super::frontmatter::Frontmatter;
use crate::errors::DomainError;

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
/// use lithos_domain::models::note::Note;
///
/// let note = Note::new("projects/example.md".to_string()).unwrap();
/// assert_eq!(note.path.as_ref(), "projects/example.md");
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Note {
    /// Embedded files.
    pub embeds: Vec<Embed>,
    /// YAML metadata.
    pub frontmatter: Option<Frontmatter>,
    /// Markdown headings.
    pub headings: Vec<Heading>,
    /// UUID v7 identity (time-ordered).
    pub id: Uuid,
    /// Outgoing links.
    pub links: Vec<Link>,
    /// Vault-relative path.
    pub path: Box<str>,
    /// Document sections.
    pub sections: Vec<Section>,
    /// Hierarchical tags.
    pub tags: Vec<Tag>,
    /// Task items.
    pub tasks: Vec<Task>,
}

impl Note {
    /// Creates a new note aggregate with path validation and identity generation.
    ///
    /// # Invariants
    /// - Generates a new UUID v7 for `id`.
    /// - Validates path according to vault-relative rules.
    ///
    /// # Errors
    /// Returns `DomainError::EmptyPath` if path is empty.
    /// Returns `DomainError::InvalidPath` if path is absolute, missing `.md` extension, or contains `..`.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::note::Note;
    ///
    /// let note = Note::new("vault/notes/project.md".to_string()).unwrap();
    /// assert!(note.id.to_string().starts_with("01"));
    /// ```
    #[inline]
    pub fn new(path: String) -> Result<Self, DomainError> {
        validate_vault_path(&path)?;

        // Generate UUID v7 identity (time-ordered)
        let id = Uuid::now_v7();

        Ok(Self {
            id,
            path: path.into(),
            frontmatter: None,
            links: vec![],
            embeds: vec![],
            tags: vec![],
            headings: vec![],
            tasks: vec![],
            sections: vec![],
        })
    }

    /// Validates the note's internal consistency.
    ///
    /// Performs semantic validation on all subentities.
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` if tags have empty segments.
    /// Returns `DomainError::InvalidHeadingLevel` if heading level is not 1-6.
    /// Returns `DomainError::EmptyLinkTarget` if any link has an empty target.
    /// Returns `DomainError::EmptyEmbedTarget` if any embed has an empty target.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::note::Note;
    ///
    /// let note = Note::new("valid.md".to_string()).unwrap();
    /// assert!(note.validate().is_ok());
    /// ```
    #[inline]
    pub fn validate(&self) -> Result<(), DomainError> {
        for tag in &self.tags {
            if tag.segments.is_empty() {
                return Err(DomainError::ValidationFailed(
                    "Tag has empty segments".to_owned(),
                ));
            }
        }
        for heading in &self.headings {
            if !(1..=6).contains(&heading.level) {
                return Err(DomainError::InvalidHeadingLevel(heading.level));
            }
        }
        for link in &self.links {
            if link.target_path.is_empty() {
                return Err(DomainError::EmptyLinkTarget);
            }
        }
        for embed in &self.embeds {
            if embed.target_path.is_empty() {
                return Err(DomainError::EmptyEmbedTarget);
            }
        }
        Ok(())
    }
}

/// Represents a link between notes.
///
/// # Invariants
/// - `target_path` is non-empty.
/// - `source_note_id` references a valid note.
/// - `position` is a valid offset in the source document.
///
/// # Examples
/// ```
/// use lithos_domain::models::note::{Link, LinkType};
/// use uuid::Uuid;
///
/// let source_id = Uuid::now_v7();
/// let link = Link::new_wikilink(source_id, "target.md".to_string(), None, 100).unwrap();
/// assert_eq!(link.link_type, LinkType::WikiLink);
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Link {
    /// Optional display alias.
    pub alias: Option<Box<str>>,
    /// Type of link (e.g., `WikiLink`).
    pub link_type: LinkType,
    /// Character offset in the source document.
    pub position: usize,
    /// UUID of the source note.
    pub source_note_id: Uuid,
    /// Vault-relative path to the target.
    pub target_path: Box<str>,
}

/// Supported link types.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum LinkType {
    /// `[text](url)` standard markdown link.
    MarkdownLink,
    /// `[[target]]` or `[[target|alias]]`.
    WikiLink,
}

impl Link {
    /// Creates a new markdown-style link reference.
    ///
    /// # Errors
    /// Returns `DomainError::EmptyLinkTarget` if target path is empty.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::note::{Link, LinkType};
    /// use uuid::Uuid;
    ///
    /// let source_id = Uuid::now_v7();
    /// let link = Link::new_markdown_link(source_id, "doc.html".to_string(), Some("Link".to_string()), 75).unwrap();
    /// assert_eq!(link.link_type, LinkType::MarkdownLink);
    /// ```
    #[inline]
    pub fn new_markdown_link(
        source_id: Uuid,
        target: String,
        alias: Option<String>,
        pos: usize,
    ) -> Result<Self, DomainError> {
        if target.is_empty() {
            return Err(DomainError::EmptyLinkTarget);
        }
        Ok(Self {
            source_note_id: source_id,
            target_path: target.into(),
            alias: alias.map(std::convert::Into::into),
            link_type: LinkType::MarkdownLink,
            position: pos,
        })
    }

    /// Creates a new wikilink reference.
    ///
    /// # Errors
    /// Returns `DomainError::EmptyLinkTarget` if target path is empty.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::note::Link;
    /// use uuid::Uuid;
    ///
    /// let source_id = Uuid::now_v7();
    /// let link = Link::new_wikilink(source_id, "page.md".to_string(), Some("Display".to_string()), 50).unwrap();
    /// assert_eq!(link.target_path.as_ref(), "page.md");
    /// ```
    #[inline]
    pub fn new_wikilink(
        source_id: Uuid,
        target: String,
        alias: Option<String>,
        pos: usize,
    ) -> Result<Self, DomainError> {
        if target.is_empty() {
            return Err(DomainError::EmptyLinkTarget);
        }
        Ok(Self {
            source_note_id: source_id,
            target_path: target.into(),
            alias: alias.map(std::convert::Into::into),
            link_type: LinkType::WikiLink,
            position: pos,
        })
    }
}

/// Represents embedded content in a note (e.g., ![[image.png]]).
///
/// # Invariants
/// - `target_path` is non-empty.
/// - `source_note_id` references a valid note.
///
/// # Examples
/// ```
/// use lithos_domain::models::note::{Embed, EmbedType};
/// use uuid::Uuid;
///
/// let source_id = Uuid::now_v7();
/// let embed = Embed::new(source_id, "diagram.png".to_string(), EmbedType::Image, 200).unwrap();
/// assert_eq!(embed.embed_type, EmbedType::Image);
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Embed {
    /// Type of embedded content.
    pub embed_type: EmbedType,
    /// Character offset in the source document.
    pub position: usize,
    /// UUID of the note containing this embed.
    pub source_note_id: Uuid,
    /// Vault-relative path to the embedded file.
    pub target_path: Box<str>,
}

/// Supported embed types.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum EmbedType {
    /// Audio file.
    Audio,
    /// Image file.
    Image,
    /// Another markdown note.
    Note,
    /// PDF document.
    Pdf,
    /// Video file.
    Video,
}

impl Embed {
    /// Creates a new embed reference.
    ///
    /// # Errors
    /// Returns `DomainError::EmptyEmbedTarget` if path is empty.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::note::{Embed, EmbedType};
    /// use uuid::Uuid;
    ///
    /// let source_id = Uuid::now_v7();
    /// let embed = Embed::new(source_id, "audio.mp3".to_string(), EmbedType::Audio, 150).unwrap();
    /// assert_eq!(embed.target_path.as_ref(), "audio.mp3");
    /// ```
    #[inline]
    pub fn new(
        source_id: Uuid,
        path: String,
        embed_type: EmbedType,
        pos: usize,
    ) -> Result<Self, DomainError> {
        if path.is_empty() {
            return Err(DomainError::EmptyEmbedTarget);
        }
        Ok(Self {
            source_note_id: source_id,
            target_path: path.into(),
            embed_type,
            position: pos,
        })
    }
}

/// Represents a hierarchical tag (e.g., #work/project).
///
/// # Invariants
/// - `full_path` does not include the leading `#`.
/// - `segments` contains 1-10 valid segments.
/// - Each segment matches `^[a-zA-Z0-9_-]+$`.
///
/// # Examples
/// ```
/// use lithos_domain::models::note::Tag;
///
/// let tag = Tag::parse("#work/project").unwrap();
/// assert_eq!(tag.full_path.as_ref(), "work/project");
/// assert_eq!(tag.segments, vec!["work".into(), "project".into()]);
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Tag {
    /// Full tag path without the leading '#'.
    pub full_path: Box<str>,
    /// List of segments in the hierarchy.
    pub segments: Vec<Box<str>>,
}

impl Tag {
    /// Parses a tag string into a hierarchy.
    ///
    /// Accepts tags with or without leading `#`.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidTag` if format is incorrect or too many segments.
    /// Returns `DomainError::EmptyTagSegment` if a segment is empty.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::note::Tag;
    ///
    /// let tag = Tag::parse("#personal").unwrap();
    /// assert_eq!(tag.segments.len(), 1);
    ///
    /// let hierarchical = Tag::parse("work/deep/nested").unwrap();
    /// assert_eq!(hierarchical.segments.len(), 3);
    /// ```
    #[inline]
    pub fn parse(input: &str) -> Result<Self, DomainError> {
        let normalized = input.strip_prefix('#').unwrap_or(input);

        if normalized.is_empty() {
            return Err(DomainError::InvalidTag(
                "Tag cannot be empty".to_owned(),
            ));
        }

        if normalized.starts_with('/') || normalized.ends_with('/') {
            return Err(DomainError::InvalidTag(
                "Tag cannot have leading or trailing slashes".to_owned(),
            ));
        }

        let segments: Vec<Box<str>> =
            normalized.split('/').map(std::convert::Into::into).collect();

        for segment in &segments {
            if segment.is_empty() {
                return Err(DomainError::EmptyTagSegment);
            }
            validate_tag_segment(segment)?;
        }

        if segments.len() > 10 {
            return Err(DomainError::InvalidTag(format!(
                "Too many segments: {}",
                segments.len()
            )));
        }

        Ok(Self {
            full_path: normalized.into(),
            segments,
        })
    }
}

/// Represents a markdown heading (e.g., ## Title).
///
/// # Invariants
/// - `level` is between 1 and 6 inclusive.
/// - `text` is non-empty.
///
/// # Examples
/// ```
/// use lithos_domain::models::note::Heading;
///
/// let heading = Heading::new(2, "Implementation".to_string(), 10).unwrap();
/// assert_eq!(heading.level, 2);
/// assert_eq!(heading.text.as_ref(), "Implementation");
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Heading {
    /// Heading level (1-6).
    pub level: u8,
    /// Character offset in the source document.
    pub position: usize,
    /// Heading text content.
    pub text: Box<str>,
}

impl Heading {
    /// Creates a new heading and validates level.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidHeadingLevel` if level is not 1-6.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::note::Heading;
    ///
    /// let h1 = Heading::new(1, "Title".to_string(), 0).unwrap();
    /// let h6 = Heading::new(6, "Subsection".to_string(), 100).unwrap();
    /// assert!(Heading::new(0, "Invalid".to_string(), 0).is_err());
    /// ```
    #[inline]
    pub fn new(
        level: u8,
        text: String,
        position: usize,
    ) -> Result<Self, DomainError> {
        if !(1..=6).contains(&level) {
            return Err(DomainError::InvalidHeadingLevel(level));
        }
        Ok(Self {
            level,
            position,
            text: text.into(),
        })
    }
}

/// Represents a markdown task item (e.g., - [ ] Task).
///
/// # Invariants
/// - `text` is the task description.
///
/// # Examples
/// ```
/// use lithos_domain::models::note::{Task, TaskStatus};
///
/// let task = Task::new("Buy milk".to_string(), TaskStatus::Incomplete, 50).unwrap();
/// assert_eq!(task.text.as_ref(), "Buy milk");
/// assert_eq!(task.status, TaskStatus::Incomplete);
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Task {
    /// Character offset in the source document.
    pub position: usize,
    /// Task completion status.
    pub status: TaskStatus,
    /// Task text content.
    pub text: Box<str>,
}

/// Supported task statuses.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum TaskStatus {
    /// `- [-]`.
    Cancelled,
    /// `- [x]`.
    Complete,
    /// `- [ ]`.
    Incomplete,
}

impl Task {
    /// Creates a new task.
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` if task data is invalid.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::note::{Task, TaskStatus};
    ///
    /// let complete = Task::new("Completed task".to_string(), TaskStatus::Complete, 25).unwrap();
    /// let cancelled = Task::new("Cancelled".to_string(), TaskStatus::Cancelled, 75).unwrap();
    /// ```
    #[inline]
    pub fn new(
        text: String,
        status: TaskStatus,
        pos: usize,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            text: text.into(),
            status,
            position: pos,
        })
    }
}

/// Represents a section of content in a note, optionally associated with a heading.
///
/// # Invariants
/// - `range` is a valid range in the document.
/// - If `heading` is Some, it starts the section.
///
/// # Examples
/// ```
/// use lithos_domain::models::note::Section;
/// use std::ops::Range;
///
/// let section = Section::new(None, "Content without heading".to_string(), 0..100);
/// assert!(section.heading.is_none());
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Section {
    /// Raw content of the section.
    pub content: String,
    /// The heading that starts this section.
    pub heading: Option<Heading>,
    /// Range of character offsets in the document.
    pub range: Range<usize>,
}

impl Section {
    /// Creates a new section.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::note::Section;
    /// use std::ops::Range;
    ///
    /// let section = Section::new(None, "Body content".to_string(), 10..200);
    /// assert_eq!(section.range, 10..200);
    /// ```
    #[inline]
    #[must_use]
    pub fn new(
        heading: Option<Heading>,
        content: String,
        range: Range<usize>,
    ) -> Self {
        // RED PHASE: Just returns something for now
        Self {
            content,
            heading,
            range,
        }
    }
}

/// Validates a vault-relative path for use in notes.
///
/// # Errors
/// Returns `DomainError::EmptyPath` if path is empty.
/// Returns `DomainError::InvalidPath` if path is absolute, missing .md extension, or contains traversal.
#[inline]
fn validate_vault_path(path: &str) -> Result<(), DomainError> {
    // Validate path is not empty
    if path.is_empty() {
        return Err(DomainError::EmptyPath);
    }

    // Validate path ends with .md extension
    if !std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        return Err(DomainError::InvalidPath(
            "Path must end with .md extension".to_owned(),
        ));
    }

    // Validate path is vault-relative (not absolute)
    if path.starts_with('/') || path.contains(':') {
        return Err(DomainError::InvalidPath(
            "Path must be vault-relative, not absolute".to_owned(),
        ));
    }

    // Validate path does not contain traversal sequences
    let path_buf = std::path::Path::new(path);
    for component in path_buf.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(DomainError::InvalidPath(
                "Path cannot contain parent directory traversal (..)"
                    .to_owned(),
            ));
        }
    }

    Ok(())
}

/// Validates a tag segment for allowed characters.
///
/// # Errors
/// Returns `DomainError::InvalidTag` if segment contains invalid characters.
#[inline]
fn validate_tag_segment(segment: &str) -> Result<(), DomainError> {
    if !segment
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(DomainError::InvalidTag(format!(
            "Invalid characters in segment '{segment}'"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod note {
        use super::*;

        #[test]
        fn rejects_empty_path() {
            let result = Note::new(String::new());
            assert!(matches!(result, Err(DomainError::EmptyPath)));
        }

        #[test]
        fn rejects_absolute_path() {
            let result = Note::new("/absolute/path.md".to_owned());
            assert!(matches!(result, Err(DomainError::InvalidPath(_))));
        }

        #[test]
        fn rejects_path_traversal() {
            let result = Note::new("../etc/passwd".to_owned());
            assert!(matches!(result, Err(DomainError::InvalidPath(_))));
        }

        #[test]
        fn rejects_path_missing_md_extension() {
            let result = Note::new("projects/lithos".to_owned());
            assert!(matches!(result, Err(DomainError::InvalidPath(_))));
        }
    }

    mod tag {
        use super::*;

        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn parses_hierarchical_tag_successfully() {
            let tag = Tag::parse("#work/project/urgent").expect("Valid tag");
            assert_eq!(tag.full_path.as_ref(), "work/project/urgent");
            assert_eq!(
                tag.segments,
                vec!["work".into(), "project".into(), "urgent".into()]
            );
        }

        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn parses_simple_tag_successfully() {
            let tag = Tag::parse("#personal").expect("Valid tag");
            assert_eq!(tag.full_path.as_ref(), "personal");
            assert_eq!(tag.segments, vec!["personal".into()]);
        }

        #[test]
        fn returns_error_for_empty_tag_segments() {
            let result = Tag::parse("#project//urgent");
            assert!(matches!(result, Err(DomainError::EmptyTagSegment)));
        }

        #[test]
        fn returns_error_for_leading_or_trailing_slashes() {
            let result = Tag::parse("#/leading");
            assert!(matches!(result, Err(DomainError::InvalidTag(_))));

            let result = Tag::parse("#trailing/");
            assert!(matches!(result, Err(DomainError::InvalidTag(_))));
        }
    }

    mod link {
        use super::*;

        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test baseline")]
        fn parses_wikilink_with_alias_successfully() {
            let source_id = Uuid::now_v7();
            let link = Link::new_wikilink(
                source_id,
                "target.md".to_owned(),
                Some("Alias".to_owned()),
                100,
            )
            .unwrap();
            assert_eq!(link.target_path.as_ref(), "target.md");
            assert_eq!(link.alias, Some("Alias".into()));
            assert_eq!(link.link_type, LinkType::WikiLink);
        }

        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test baseline")]
        fn tracks_link_position_in_document() {
            let source_id = Uuid::now_v7();
            let link = Link::new_wikilink(
                source_id,
                "target.md".to_owned(),
                None,
                500,
            )
            .unwrap();
            assert_eq!(link.position, 500);
        }
    }

    mod embed {
        use super::*;

        #[test]
        fn validates_embed_target_is_not_empty() {
            let source_id = Uuid::now_v7();
            let result =
                Embed::new(source_id, String::new(), EmbedType::Image, 0);
            assert!(matches!(result, Err(DomainError::EmptyEmbedTarget)));
        }
    }

    mod heading {
        use super::*;

        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test baseline")]
        fn validates_heading_levels_1_to_6() {
            for level in 1..=6 {
                let heading =
                    Heading::new(level, "Title".to_owned(), 0).unwrap();
                assert_eq!(heading.level, level);
            }
        }

        #[test]
        fn returns_error_for_invalid_heading_level_0() {
            let result = Heading::new(0, "Title".to_owned(), 0);
            assert!(matches!(result, Err(DomainError::InvalidHeadingLevel(0))));
        }

        #[test]
        fn returns_error_for_invalid_heading_level_7() {
            let result = Heading::new(7, "Title".to_owned(), 0);
            assert!(matches!(result, Err(DomainError::InvalidHeadingLevel(7))));
        }
    }

    mod task {
        use super::*;

        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test baseline")]
        fn parses_all_task_status_variants() {
            let statuses = vec![
                TaskStatus::Incomplete,
                TaskStatus::Complete,
                TaskStatus::Cancelled,
            ];
            for status in statuses {
                let task = Task::new("Buy milk".to_owned(), status.clone(), 0)
                    .unwrap();
                assert_eq!(task.status, status);
            }
        }
    }

    mod section {
        use super::*;

        #[test]
        fn calculates_content_range_correctly() {
            let range = 10..50;
            let section =
                Section::new(None, "Content".to_owned(), range.clone());
            assert_eq!(section.range, range);
        }
    }
}

/// Test fixtures for deterministic note data.
#[cfg(test)]
pub mod fixtures {
    use std::collections::HashMap;

    use super::*;
    use crate::models::frontmatter::FrontmatterValue;

    /// Fixed UUID for deterministic tests (valid UUID v7 format).
    /// Uses timestamp 2024-01-01 00:00:00 UTC for consistency.
    pub const TEST_NOTE_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0001);

    /// Creates an example frontmatter for testing.
    #[inline]
    #[must_use]
    pub fn example_frontmatter() -> Frontmatter {
        let mut fields = HashMap::new();
        fields.insert(
            "title".to_owned(),
            FrontmatterValue::String("Test Note".to_owned()),
        );
        Frontmatter {
            fields,
        }
    }

    /// Creates an example tag for testing.
    #[inline]
    #[must_use]
    pub fn example_tag() -> Tag {
        Tag {
            full_path: "work/project".into(),
            segments: vec!["work".into(), "project".into()],
        }
    }

    /// Creates an example note for testing.
    #[inline]
    #[must_use]
    pub fn example_note() -> Note {
        Note {
            embeds: vec![],
            frontmatter: Some(example_frontmatter()),
            headings: vec![],
            id: TEST_NOTE_ID,
            links: vec![],
            path: "test/example.md".into(),
            sections: vec![],
            tags: vec![example_tag()],
            tasks: vec![],
        }
    }
}
