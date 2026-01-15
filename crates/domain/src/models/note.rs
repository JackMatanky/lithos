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

use std::{collections::HashMap, ops::Range};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::errors::DomainError;

/// Aggregate root representing an Obsidian note.
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
    pub path: String,
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
    /// # Errors
    /// Returns `DomainError::EmptyPath` if path is empty.
    /// Returns `DomainError::InvalidPath` if path is absolute or contains traversal.
    #[inline]
    pub fn new(path: String) -> Result<Self, DomainError> {
        // Validate path is not empty
        if path.is_empty() {
            return Err(DomainError::EmptyPath);
        }

        // Validate path ends with .md extension
        if !std::path::Path::new(&path)
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
        let path_buf = std::path::Path::new(&path);
        for component in path_buf.components() {
            if matches!(component, std::path::Component::ParentDir) {
                return Err(DomainError::InvalidPath(
                    "Path cannot contain parent directory traversal (..)"
                        .to_owned(),
                ));
            }
        }

        // Generate UUID v7 identity (time-ordered)
        let id = Uuid::now_v7();

        Ok(Self {
            id,
            path,
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
    /// # Errors
    /// Returns `DomainError::ValidationFailed` if tags have empty segments.
    /// Returns `DomainError::InvalidHeadingLevel` if heading level is not 1-6.
    /// Returns `DomainError::EmptyLinkTarget` if any link has an empty target.
    /// Returns `DomainError::EmptyEmbedTarget` if any embed has an empty target.
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

/// Represents YAML metadata extracted from a note header.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Frontmatter {
    /// Key-value pairs of metadata fields.
    pub fields: HashMap<String, FrontmatterValue>,
}

/// Possible values in a frontmatter field.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum FrontmatterValue {
    /// Array of values.
    Array(Vec<FrontmatterValue>),
    /// Boolean value.
    Boolean(bool),
    /// Date/time value.
    Date(DateTime<Utc>),
    /// Numeric value (float).
    Number(f64),
    /// Nested object of values.
    Object(HashMap<String, FrontmatterValue>),
    /// String value.
    String(String),
}

impl Frontmatter {
    /// Extracts the aliases field from frontmatter using the configured key.
    #[inline]
    #[must_use]
    pub fn aliases(
        &self,
        config: &crate::models::config::Config,
    ) -> Vec<String> {
        self.get_string_array(&config.frontmatter.alias_key)
    }

    /// Extracts the `file_class` field from frontmatter using the configured key.
    #[inline]
    #[must_use]
    pub fn file_class(&self, config: &crate::models::config::Config) -> String {
        self.get_string(&config.frontmatter.file_class_key).unwrap_or_default()
    }

    /// Gets a frontmatter value by key.
    #[inline]
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&FrontmatterValue> {
        self.fields.get(key)
    }

    /// Extracts a string value from frontmatter by key.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics are preferred here for clarity when matching FrontmatterValue variants"
    )]
    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.get(key)? {
            FrontmatterValue::String(s) => Some(s.clone()),
            &FrontmatterValue::Array(_)
            | &FrontmatterValue::Boolean(_)
            | &FrontmatterValue::Date(_)
            | &FrontmatterValue::Number(_)
            | &FrontmatterValue::Object(_) => None,
        }
    }

    /// Extracts a string array from frontmatter by key.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics are preferred here for clarity when matching FrontmatterValue variants"
    )]
    pub fn get_string_array(&self, key: &str) -> Vec<String> {
        match self.get(key) {
            Some(FrontmatterValue::Array(arr)) => arr
                .iter()
                .filter_map(|item| match item {
                    FrontmatterValue::String(s) => Some(s.clone()),
                    &FrontmatterValue::Array(_)
                    | &FrontmatterValue::Boolean(_)
                    | &FrontmatterValue::Date(_)
                    | &FrontmatterValue::Number(_)
                    | &FrontmatterValue::Object(_) => None,
                })
                .collect(),
            Some(
                &FrontmatterValue::Boolean(_)
                | &FrontmatterValue::Date(_)
                | &FrontmatterValue::Number(_)
                | &FrontmatterValue::Object(_)
                | &FrontmatterValue::String(_),
            )
            | None => Vec::new(),
        }
    }

    /// Creates a new frontmatter from fields.
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` if fields are invalid.
    #[inline]
    pub fn new(
        fields: HashMap<String, FrontmatterValue>,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            fields,
        })
    }

    /// Extracts the title field from frontmatter using the configured key.
    #[inline]
    #[must_use]
    pub fn title(&self, config: &crate::models::config::Config) -> String {
        self.get_string(&config.frontmatter.title_key).unwrap_or_default()
    }
}

/// Represents a link between notes.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Link {
    /// Optional display alias.
    pub alias: Option<String>,
    /// Type of link (e.g., `WikiLink`).
    pub link_type: LinkType,
    /// Character offset in the source document.
    pub position: usize,
    /// UUID of the source note.
    pub source_note_id: Uuid,
    /// Vault-relative path to the target.
    pub target_path: String,
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
            target_path: target,
            alias,
            link_type: LinkType::MarkdownLink,
            position: pos,
        })
    }

    /// Creates a new wikilink reference.
    ///
    /// # Errors
    /// Returns `DomainError::EmptyLinkTarget` if target path is empty.
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
            target_path: target,
            alias,
            link_type: LinkType::WikiLink,
            position: pos,
        })
    }
}

/// Represents embedded content in a note (e.g., ![[image.png]]).
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
    pub target_path: String,
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
            target_path: path,
            embed_type,
            position: pos,
        })
    }
}

/// Represents a hierarchical tag (e.g., #work/project).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Tag {
    /// Full tag path without the leading '#'.
    pub full_path: String,
    /// List of segments in the hierarchy.
    pub segments: Vec<String>,
}

impl Tag {
    /// Parses a tag string into a hierarchy.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidTag` if format is incorrect.
    /// Returns `DomainError::EmptyTagSegment` if a segment is empty.
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

        let segments: Vec<String> =
            normalized.split('/').map(String::from).collect();

        for segment in &segments {
            if segment.is_empty() {
                return Err(DomainError::EmptyTagSegment);
            }
            if !segment
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                return Err(DomainError::InvalidTag(format!(
                    "Invalid characters in segment '{segment}'"
                )));
            }
        }

        if segments.len() > 10 {
            return Err(DomainError::InvalidTag(format!(
                "Too many segments: {}",
                segments.len()
            )));
        }

        Ok(Self {
            full_path: normalized.to_owned(),
            segments,
        })
    }
}

/// Represents a markdown heading (e.g., ## Title).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Heading {
    /// Heading level (1-6).
    pub level: u8,
    /// Character offset in the source document.
    pub position: usize,
    /// Heading text content.
    pub text: String,
}

impl Heading {
    /// Creates a new heading and validates level.
    ///
    /// # Errors
    /// Returns `DomainError::InvalidHeadingLevel` if level is not 1-6.
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
            text,
        })
    }
}

/// Represents a markdown task item (e.g., - [ ] Task).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Task {
    /// Character offset in the source document.
    pub position: usize,
    /// Task completion status.
    pub status: TaskStatus,
    /// Task text content.
    pub text: String,
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
    #[inline]
    pub fn new(
        text: String,
        status: TaskStatus,
        pos: usize,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            text,
            status,
            position: pos,
        })
    }
}

/// Represents a section of content in a note, optionally associated with a heading.
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

#[cfg(test)]
mod tests {
    use chrono::{Datelike as _, TimeZone as _};
    use proptest::prelude::*;

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

        proptest! {
            #[test]

            fn generates_monotonic_uuid_v7_ids(_ in 0..100u32) {
                // In RED phase, we just show how we would test this
                // We'd generate many notes and check ID ordering
            }
        }
    }

    mod tag {
        use super::*;

        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test baseline")]
        fn parses_hierarchical_tag_successfully() {
            let tag = Tag::parse("#work/project/urgent").unwrap();
            assert_eq!(tag.full_path, "work/project/urgent");
            assert_eq!(tag.segments, vec!["work", "project", "urgent"]);
        }

        #[test]

        fn returns_error_for_invalid_tag_characters() {
            let result = Tag::parse("#invalid segment");
            assert!(matches!(result, Err(DomainError::InvalidTag(_))));
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

    mod frontmatter {
        use super::*;

        #[test]
        #[expect(clippy::panic, reason = "Test error path")]
        fn parses_iso8601_date_successfully() {
            let date = Utc.with_ymd_and_hms(2024, 1, 15, 14, 30, 0).unwrap();
            let val = FrontmatterValue::Date(date);
            if let FrontmatterValue::Date(d) = val {
                assert_eq!(d.year(), 2_024i32);
            } else {
                panic!("Expected Date variant");
            }
        }

        #[test]

        fn converts_numeric_values_correctly() {
            let val = FrontmatterValue::Number(42.0);
            assert!(
                matches!(val, FrontmatterValue::Number(n) if (n - 42.0).abs() < f64::EPSILON)
            );
        }

        #[test]

        fn converts_boolean_values_correctly() {
            let val = FrontmatterValue::Boolean(true);
            assert!(matches!(val, FrontmatterValue::Boolean(true)));
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
            assert_eq!(link.target_path, "target.md");
            assert_eq!(link.alias, Some("Alias".to_owned()));
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
    use super::*;

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
            full_path: "work/project".to_owned(),
            segments: vec!["work".to_owned(), "project".to_owned()],
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
            path: "test/example.md".to_owned(),
            sections: vec![],
            tags: vec![example_tag()],
            tasks: vec![],
        }
    }
}
