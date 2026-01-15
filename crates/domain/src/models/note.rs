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

        /// 3.2-UNIT-001: Note Creation - Empty Path.
        /// P1.
        #[test]
        fn new_note_returns_error_when_path_is_empty() {
            // GIVEN an empty path string
            let path = String::new();

            // WHEN a new Note is constructed
            let result = Note::new(path);

            // THEN it returns an EmptyPath error
            assert!(matches!(result, Err(DomainError::EmptyPath)));
        }

        /// 3.2-UNIT-002: Note Creation - Absolute Path.
        /// P1.
        #[test]
        fn new_note_returns_error_when_path_is_absolute() {
            // GIVEN an absolute path string
            let path = "/absolute/path.md".to_owned();

            // WHEN a new Note is constructed
            let result = Note::new(path);

            // THEN it returns an InvalidPath error
            assert!(matches!(result, Err(DomainError::InvalidPath(_))));
        }

        /// 3.2-UNIT-003: Note Creation - Path Traversal.
        /// P1.
        #[test]
        fn new_note_returns_error_when_path_contains_traversal() {
            // GIVEN a path string with traversal components
            let path = "../etc/passwd".to_owned();

            // WHEN a new Note is constructed
            let result = Note::new(path);

            // THEN it returns an InvalidPath error
            assert!(matches!(result, Err(DomainError::InvalidPath(_))));
        }

        /// 3.2-UNIT-004: Note Creation - Missing Extension.
        /// P1.
        #[test]
        fn new_note_returns_error_when_path_missing_md_extension() {
            // GIVEN a path string without .md extension
            let path = "projects/lithos".to_owned();

            // WHEN a new Note is constructed
            let result = Note::new(path);

            // THEN it returns an InvalidPath error
            assert!(matches!(result, Err(DomainError::InvalidPath(_))));
        }

        /// 3.2-UNIT-022: Note Validation - Success.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn validate_note_succeeds_when_all_entities_are_valid() {
            // GIVEN a note with valid sub-entities
            let mut note =
                Note::new("valid.md".to_owned()).expect("Valid path");
            note.tags.push(Tag::parse("work").expect("Valid tag"));
            note.headings
                .push(Heading::new(1, "Title".into(), 0).expect("Valid level"));
            note.links.push(
                Link::new_wikilink(note.id, "target.md".into(), None, 0)
                    .expect("Valid target"),
            );
            note.embeds.push(
                Embed::new(note.id, "img.png".into(), EmbedType::Image, 0)
                    .expect("Valid target"),
            );

            // WHEN the note is validated
            // THEN it returns Ok
            note.validate().expect("Validation failed");
        }

        /// 3.2-UNIT-023: Note Validation - Invalid Heading.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn validate_note_returns_error_when_heading_level_is_invalid() {
            // GIVEN a note with an invalid heading (manually constructed)
            let mut note =
                Note::new("valid.md".to_owned()).expect("Valid path");
            note.headings.push(Heading {
                level: 0,
                text: "Invalid".into(),
                position: 0,
            });

            // WHEN the note is validated
            let result = note.validate();

            // THEN it returns an InvalidHeadingLevel error
            assert!(matches!(result, Err(DomainError::InvalidHeadingLevel(0))));
        }

        /// 3.2-UNIT-032: Note Validation - Empty Link Target.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn validate_note_returns_error_when_link_target_is_empty() {
            // GIVEN a note with an empty link target
            let mut note = Note::new("valid.md".into()).expect("Valid path");
            note.links.push(Link {
                alias: None,
                link_type: LinkType::WikiLink,
                position: 0,
                source_note_id: note.id,
                target_path: "".into(),
            });

            // WHEN validated
            // THEN it returns EmptyLinkTarget
            assert!(matches!(
                note.validate(),
                Err(DomainError::EmptyLinkTarget)
            ));
        }

        /// 3.2-UNIT-033: Note Validation - Empty Embed Target.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn validate_note_returns_error_when_embed_target_is_empty() {
            // GIVEN a note with an empty embed target
            let mut note = Note::new("valid.md".into()).expect("Valid path");
            note.embeds.push(Embed {
                embed_type: EmbedType::Image,
                position: 0,
                source_note_id: note.id,
                target_path: "".into(),
            });

            // WHEN validated
            // THEN it returns EmptyEmbedTarget
            assert!(matches!(
                note.validate(),
                Err(DomainError::EmptyEmbedTarget)
            ));
        }

        /// 3.2-UNIT-034: Note Path Validation - Absolute Paths.
        /// P1.
        #[test]
        fn new_note_returns_error_when_path_has_colon() {
            // GIVEN a path with a colon (Windows absolute)
            let path = "C:/path.md".to_owned();

            // WHEN constructed
            // THEN it returns InvalidPath
            assert!(matches!(
                Note::new(path),
                Err(DomainError::InvalidPath(_))
            ));
        }
    }

    mod tag {
        use proptest::prelude::*;

        use super::*;

        /// 3.2-UNIT-005: Tag Parsing - Hierarchical.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn parse_tag_succeeds_when_tag_is_hierarchical() {
            // GIVEN a valid hierarchical tag string
            let input = "#work/project/urgent";

            // WHEN the tag is parsed
            let tag = Tag::parse(input).expect("Valid tag");

            // THEN it correctly extracts segments and full path
            assert_eq!(tag.full_path.as_ref(), "work/project/urgent");
            assert_eq!(
                tag.segments,
                vec!["work".into(), "project".into(), "urgent".into()]
            );
        }

        /// 3.2-UNIT-006: Tag Parsing - Simple.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn parse_tag_succeeds_when_tag_is_simple() {
            // GIVEN a valid simple tag string
            let input = "#personal";

            // WHEN the tag is parsed
            let tag = Tag::parse(input).expect("Valid tag");

            // THEN it correctly extracts the single segment
            assert_eq!(tag.full_path.as_ref(), "personal");
            assert_eq!(tag.segments, vec!["personal".into()]);
        }

        /// 3.2-UNIT-007: Tag Parsing - Empty Segments.
        /// P1.
        #[test]
        fn parse_tag_returns_error_when_segments_are_empty() {
            // GIVEN a tag string with empty segments
            let input = "#project//urgent";

            // WHEN the tag is parsed
            let result = Tag::parse(input);

            // THEN it returns an EmptyTagSegment error
            assert!(matches!(result, Err(DomainError::EmptyTagSegment)));
        }

        /// 3.2-UNIT-008: Tag Parsing - Surrounding Slashes.
        /// P1.
        #[test]
        fn parse_tag_returns_error_when_tag_has_surrounding_slashes() {
            // GIVEN tags with leading or trailing slashes
            let leading = "#/leading";
            let trailing = "#trailing/";

            // WHEN the tags are parsed
            let res_leading = Tag::parse(leading);
            let res_trailing = Tag::parse(trailing);

            // THEN both return InvalidTag errors
            assert!(matches!(res_leading, Err(DomainError::InvalidTag(_))));
            assert!(matches!(res_trailing, Err(DomainError::InvalidTag(_))));
        }

        /// 3.2-UNIT-024: Tag Parsing - Too Many Segments.
        /// P1.
        #[test]
        fn parse_tag_returns_error_when_too_many_segments() {
            // GIVEN a tag with 11 segments
            let input = "1/2/3/4/5/6/7/8/9/10/11";

            // WHEN the tag is parsed
            let result = Tag::parse(input);

            // THEN it returns an InvalidTag error
            assert!(matches!(result, Err(DomainError::InvalidTag(_))));
        }

        /// 3.2-UNIT-035: Tag Segment Validation - Invalid Characters.
        /// P1.
        #[test]
        fn parse_tag_returns_error_when_segment_contains_invalid_chars() {
            // GIVEN a tag with invalid character
            let input = "invalid!";

            // WHEN parsed
            // THEN it returns InvalidTag
            assert!(matches!(
                Tag::parse(input),
                Err(DomainError::InvalidTag(_))
            ));
        }

        proptest! {
            /// 3.2-PROP-001: Tag Property - Hierarchical Segments.
            /// P2.
            #[test]
            #[expect(clippy::disallowed_methods, clippy::indexing_slicing, reason = "Test logic")]
            fn property_tag_parsing_preserves_hierarchy(
                s in prop::collection::vec("[a-zA-Z0-9_-]+", 1..10)
            ) {
                let input = s.join("/");
                let tag = Tag::parse(&input).expect("Valid tag");
                assert_eq!(tag.segments.len(), s.len());
                for (i, segment) in tag.segments.iter().enumerate() {
                    assert_eq!(segment.as_ref(), &s[i]);
                }
            }
        }
    }

    mod link {
        use uuid::Uuid;

        use super::*;

        /// 3.2-UNIT-009: Link Creation - `WikiLink` with Alias.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test baseline")]
        fn new_wikilink_succeeds_when_alias_is_provided() {
            // GIVEN link parameters including an alias
            let source_id = Uuid::now_v7();
            let target = "target.md".to_owned();
            let alias = Some("Alias".to_owned());
            let pos = 100;

            // WHEN a new wikilink is constructed
            let link =
                Link::new_wikilink(source_id, target, alias, pos).unwrap();

            // THEN it maintains all provided attributes
            assert_eq!(link.target_path.as_ref(), "target.md");
            assert_eq!(link.alias, Some("Alias".into()));
            assert_eq!(link.link_type, LinkType::WikiLink);
        }

        /// 3.2-UNIT-010: Link Creation - Position Tracking.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test baseline")]
        fn new_wikilink_maintains_position_offset() {
            // GIVEN a character offset
            let source_id = Uuid::now_v7();
            let pos = 500;

            // WHEN a new wikilink is constructed
            let link = Link::new_wikilink(
                source_id,
                "target.md".to_owned(),
                None,
                pos,
            )
            .unwrap();

            // THEN the position is correctly stored
            assert_eq!(link.position, 500);
        }

        /// 3.2-UNIT-025: Link Creation - Markdown Link.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn new_markdown_link_succeeds_with_valid_data() {
            // GIVEN markdown link parameters
            let source_id = Uuid::now_v7();
            let target = "https://example.com".to_owned();
            let alias = Some("Example".to_owned());

            // WHEN a new markdown link is constructed
            let link = Link::new_markdown_link(source_id, target, alias, 0)
                .expect("Valid target");

            // THEN it is correctly typed as MarkdownLink
            assert_eq!(link.link_type, LinkType::MarkdownLink);
        }
    }

    mod embed {
        use uuid::Uuid;

        use super::*;

        /// 3.2-UNIT-011: Embed Creation - Target Validation.
        /// P1.
        #[test]
        fn new_embed_returns_error_when_target_is_empty() {
            // GIVEN an empty target path for an embed
            let source_id = Uuid::now_v7();
            let path = String::new();

            // WHEN a new Embed is constructed
            let result = Embed::new(source_id, path, EmbedType::Image, 0);

            // THEN it returns an EmptyEmbedTarget error
            assert!(matches!(result, Err(DomainError::EmptyEmbedTarget)));
        }
    }

    mod heading {
        use super::*;

        /// 3.2-UNIT-012: Heading Creation - Valid Levels.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test baseline")]
        fn new_heading_succeeds_when_level_is_valid() {
            // GIVEN valid heading levels 1 through 6
            for level in 1..=6 {
                // WHEN a new heading is constructed
                let heading =
                    Heading::new(level, "Title".to_owned(), 0).unwrap();

                // THEN the level is correctly assigned
                assert_eq!(heading.level, level);
            }
        }

        /// 3.2-UNIT-013: Heading Creation - Level Zero.
        /// P1.
        #[test]
        fn new_heading_returns_error_when_level_is_zero() {
            // GIVEN an invalid heading level of 0
            let level = 0;

            // WHEN a new heading is constructed
            let result = Heading::new(level, "Title".to_owned(), 0);

            // THEN it returns an InvalidHeadingLevel error
            assert!(matches!(result, Err(DomainError::InvalidHeadingLevel(0))));
        }

        /// 3.2-UNIT-014: Heading Creation - Level Too High.
        /// P1.
        #[test]
        fn new_heading_returns_error_when_level_is_too_high() {
            // GIVEN an invalid heading level of 7
            let level = 7;

            // WHEN a new heading is constructed
            let result = Heading::new(level, "Title".to_owned(), 0);

            // THEN it returns an InvalidHeadingLevel error
            assert!(matches!(result, Err(DomainError::InvalidHeadingLevel(7))));
        }
    }

    mod task {
        use super::*;

        /// 3.2-UNIT-015: Task Creation - Status Variants.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test baseline")]
        fn new_task_succeeds_for_all_status_variants() {
            // GIVEN all possible task status variants
            let statuses = vec![
                TaskStatus::Incomplete,
                TaskStatus::Complete,
                TaskStatus::Cancelled,
            ];

            for status in statuses {
                // WHEN a new task is constructed
                let task = Task::new("Buy milk".to_owned(), status.clone(), 0)
                    .unwrap();

                // THEN the status is correctly assigned
                assert_eq!(task.status, status);
            }
        }
    }

    mod section {
        use super::*;

        /// 3.2-UNIT-016: Section Creation - Content Range.
        /// P1.
        #[test]
        fn new_section_maintains_content_range() {
            // GIVEN a character range
            let range = 10..50;

            // WHEN a new section is constructed
            let section =
                Section::new(None, "Content".to_owned(), range.clone());

            // THEN the range is correctly preserved
            assert_eq!(section.range, range);
        }
    }
}

/// Test fixtures for deterministic note data.
#[cfg(test)]
pub mod fixtures {
    use std::collections::HashMap;

    use uuid::Uuid;

    use super::*;
    use crate::models::frontmatter::FieldValue;

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
            FieldValue::String("Test Note".to_owned()),
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
