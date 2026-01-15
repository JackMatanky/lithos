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
    frontmatter::Frontmatter,
    link::Link,
    structure::{Heading, Section},
    tag::Tag,
    task::Task,
};
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
/// use uuid::Uuid;
///
/// // For new files (first-time indexing)
/// let new_id = Uuid::now_v7();
/// let note = Note::new(new_id, "projects/example.md".to_string()).unwrap();
/// assert_eq!(note.path.as_ref(), "projects/example.md");
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "id and path are primary identifiers, should be first"
)]
pub struct Note {
    /// UUID v7 identity (time-ordered).
    pub id: Uuid,
    /// Vault-relative path.
    pub path: Box<str>,
    /// YAML metadata.
    pub frontmatter: Option<Frontmatter>,
    /// Outgoing links.
    pub links: Vec<Link>,
    /// Embedded files.
    pub embeds: Vec<Link>,
    /// Hierarchical tags.
    pub tags: Vec<Tag>,
    /// Markdown headings.
    pub headings: Vec<Heading>,
    /// Task items.
    pub tasks: Vec<Task>,
    /// Document sections.
    pub sections: Vec<Section>,
}

impl Note {
    /// Creates a new note aggregate with the provided UUID and validated path.
    ///
    /// # UUID Source
    /// The UUID should be obtained from the repository layer via:
    /// - `NoteRepository::get_or_create_note_id()` for indexed files (preserves existing identity)
    /// - `Uuid::now_v7()` for brand-new notes (first-time indexing)
    ///
    /// This design ensures UUID stability across file renames and system restarts,
    /// as UUIDs are persisted in the Redb cache and retrieved via path lookups.
    ///
    /// # Invariants
    /// - Uses the provided UUID v7 for `id` (time-ordered identity).
    /// - Validates path according to vault-relative rules.
    /// - Emits `NoteCreated` domain event.
    ///
    /// # Errors
    /// Returns `DomainError::EmptyPath` if path is empty.
    /// Returns `DomainError::InvalidPath` if path is absolute, missing `.md` extension, or contains `..`.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::note::Note;
    /// use uuid::Uuid;
    ///
    /// // For new files (first-time indexing)
    /// let new_id = Uuid::now_v7();
    /// let note = Note::new(new_id, "vault/notes/project.md".to_string()).unwrap();
    /// assert!(note.id.to_string().starts_with("01"));
    ///
    /// // For existing files (rename detection preserves UUID)
    /// let existing_id = Uuid::parse_str("01936b2e-8f4a-7890-abcd-ef1234567890").unwrap();
    /// let renamed_note = Note::new(existing_id, "archive/old-project.md".to_string()).unwrap();
    /// assert_eq!(renamed_note.id, existing_id);
    /// ```
    #[inline]
    pub fn new(id: Uuid, path: String) -> Result<Self, DomainError> {
        validate_vault_path(&path)?;

        // Emit domain event for note creation
        // Note: Event emission would typically be handled by the application layer
        // This is a placeholder for the event emission infrastructure
        // TODO: Integrate with event bus in application layer (Epic 7)

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
    /// # Invariants
    /// - Emits `NoteFrontmatterValidated` domain event upon successful validation.
    ///
    /// # Errors
    /// Returns `DomainError::ValidationFailed` if tags have empty segments.
    /// Returns `DomainError::InvalidHeadingLevel` if heading level is not 1-6.
    /// Returns `DomainError::EmptyLinkTarget` if any link or embed has an empty target.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::note::Note;
    /// use uuid::Uuid;
    ///
    /// let test_id = Uuid::now_v7();
    /// let note = Note::new(test_id, "valid.md".to_string()).unwrap();
    /// assert!(note.validate().is_ok());
    /// ```
    #[inline]
    pub fn validate(&self) -> Result<(), DomainError> {
        self.validate_tags()?;
        self.validate_headings()?;
        self.validate_links()?;
        self.validate_embeds()?;

        // Validation successful - NoteFrontmatterValidated event would be emitted here
        // Note: Event emission is handled by the application layer (Epic 7)
        // This is a placeholder for the event emission infrastructure
        // TODO: Integrate with event bus in application layer

        Ok(())
    }

    /// Validates all embeds in the note.
    #[inline]
    fn validate_embeds(&self) -> Result<(), DomainError> {
        for embed in &self.embeds {
            if embed.target_path.is_empty() {
                return Err(DomainError::EmptyLinkTarget);
            }
        }
        Ok(())
    }

    /// Validates all headings in the note.
    #[inline]
    fn validate_headings(&self) -> Result<(), DomainError> {
        for heading in &self.headings {
            if !(1..=6).contains(&heading.level) {
                return Err(DomainError::InvalidHeadingLevel(heading.level));
            }
        }
        Ok(())
    }

    /// Validates all links in the note.
    #[inline]
    fn validate_links(&self) -> Result<(), DomainError> {
        for link in &self.links {
            if link.target_path.is_empty() {
                return Err(DomainError::EmptyLinkTarget);
            }
        }
        Ok(())
    }

    /// Validates all tags in the note.
    #[inline]
    fn validate_tags(&self) -> Result<(), DomainError> {
        for tag in &self.tags {
            if tag.segments.is_empty() {
                return Err(DomainError::ValidationFailed(
                    "Tag has empty segments".to_owned(),
                ));
            }
        }
        Ok(())
    }
}

/// Validates a vault-relative path according to business rules.
///
/// # Rules
/// - Must not be empty
/// - Must not start with `/` (absolute path)
/// - Must not contain `..` (path traversal)
/// - Must end with `.md` extension
///
/// # Errors
/// Returns `DomainError::EmptyPath` if path is empty.
/// Returns `DomainError::InvalidPath` for violations.
fn validate_vault_path(path: &str) -> Result<(), DomainError> {
    validate_path_not_empty(path)?;
    validate_path_is_relative(path)?;
    validate_path_no_traversal(path)?;
    validate_path_has_md_extension(path)?;
    Ok(())
}

/// Validates that a path is not empty.
///
/// # Errors
/// Returns `DomainError::EmptyPath` if the path is empty.
#[inline]
fn validate_path_not_empty(path: &str) -> Result<(), DomainError> {
    if path.is_empty() {
        return Err(DomainError::EmptyPath);
    }
    Ok(())
}

/// Validates that a path is relative (not absolute).
///
/// Checks for both Unix-style (`/path`) and Windows-style (`C:/path`) absolute paths.
///
/// # Errors
/// Returns `DomainError::InvalidPath` if the path is absolute.
#[inline]
fn validate_path_is_relative(path: &str) -> Result<(), DomainError> {
    // Check for Unix-style absolute path
    if path.starts_with('/') {
        return Err(DomainError::InvalidPath(
            "Path must be relative".to_owned(),
        ));
    }

    // Check for Windows-style absolute paths (drive letter followed by colon and slash)
    if is_windows_absolute_path(path) {
        return Err(DomainError::InvalidPath(
            "Path must be relative (Windows absolute paths not allowed)"
                .to_owned(),
        ));
    }

    Ok(())
}

/// Checks if a path is a Windows-style absolute path (e.g., `C:/path`).
#[inline]
fn is_windows_absolute_path(path: &str) -> bool {
    path.len() >= 3
        && path.chars().nth(1) == Some(':')
        && path.chars().nth(2) == Some('/')
        && path.chars().next().is_some_and(char::is_alphabetic)
}

/// Validates that a path does not contain path traversal sequences.
///
/// # Errors
/// Returns `DomainError::InvalidPath` if the path contains `..`.
#[inline]
fn validate_path_no_traversal(path: &str) -> Result<(), DomainError> {
    if path.contains("..") {
        return Err(DomainError::InvalidPath(
            "Path traversal not allowed".to_owned(),
        ));
    }
    Ok(())
}

/// Validates that a path has a `.md` extension.
///
/// # Errors
/// Returns `DomainError::InvalidPath` if the path does not end with `.md`.
#[inline]
fn validate_path_has_md_extension(path: &str) -> Result<(), DomainError> {
    if !std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        return Err(DomainError::InvalidPath(
            "Path must end with .md".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use lithos_test_utils::test_builder;

    use super::*;

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
    });

    mod new {
        use lithos_test_utils::time_test;
        use tokio::time::Duration;

        use super::*;

        /// 3.2-UNIT-001: Note Creation - Empty Path.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn returns_error_when_path_is_empty() {
            // GIVEN a test UUID and empty path string
            let test_id =
                Uuid::parse_str("01936b2e-8f4a-7890-abcd-ef1234567890")
                    .unwrap();
            let path = String::new();

            // WHEN a new Note is constructed
            let result = Note::new(test_id, path);

            // THEN it returns an EmptyPath error
            assert!(matches!(result, Err(DomainError::EmptyPath)));
        }

        /// 3.2-UNIT-002: Note Creation - Absolute Path.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn returns_error_when_path_is_absolute() {
            // GIVEN a test UUID and absolute path string
            let test_id =
                Uuid::parse_str("01936b2e-8f4a-7890-abcd-ef1234567891")
                    .unwrap();
            let path = "/absolute/path.md".to_owned();

            // WHEN a new Note is constructed
            let result = Note::new(test_id, path);

            // THEN it returns an InvalidPath error
            assert!(matches!(result, Err(DomainError::InvalidPath(_))));
        }

        /// 3.2-UNIT-003: Note Creation - Path Traversal.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn returns_error_when_path_contains_traversal() {
            // GIVEN a test UUID and path string with traversal components
            let test_id =
                Uuid::parse_str("01936b2e-8f4a-7890-abcd-ef1234567892")
                    .unwrap();
            let path = "../etc/passwd".to_owned();

            // WHEN a new Note is constructed
            let result = Note::new(test_id, path);

            // THEN it returns an InvalidPath error
            assert!(matches!(result, Err(DomainError::InvalidPath(_))));
        }

        /// 3.2-UNIT-004: Note Creation - Missing Extension.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn returns_error_when_path_missing_md_extension() {
            // GIVEN a test UUID and path string without .md extension
            let test_id =
                Uuid::parse_str("01936b2e-8f4a-7890-abcd-ef1234567893")
                    .unwrap();
            let path = "projects/lithos".to_owned();

            // WHEN a new Note is constructed
            let result = Note::new(test_id, path);

            // THEN it returns an InvalidPath error
            assert!(matches!(result, Err(DomainError::InvalidPath(_))));
        }

        /// 3.2-UNIT-034: Note Path Validation - Absolute Paths (Windows).
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn returns_error_for_windows_absolute_paths() {
            // GIVEN a test UUID and path with a colon (Windows absolute)
            let test_id =
                Uuid::parse_str("01936b2e-8f4a-7890-abcd-ef1234567898")
                    .unwrap();
            let path = "C:/path.md".to_owned();

            // WHEN constructed
            let result = Note::new(test_id, path);

            // THEN it returns InvalidPath
            assert!(matches!(result, Err(DomainError::InvalidPath(_))));
        }

        /// 3.2-UNIT-035: Note Creation - UUID v7 Sequence.
        /// P1.
        time_test!(
            async fn generates_sequential_uuids() {
                // GIVEN a note created at T0
                let note1 = Note::new(Uuid::now_v7(), "one.md".into()).unwrap();

                // WHEN advancing time and creating a second note
                tokio::time::advance(Duration::from_millis(10)).await;
                let note2 = Note::new(Uuid::now_v7(), "two.md".into()).unwrap();

                // THEN the second ID is strictly greater than the first
                assert!(
                    note2.id > note1.id,
                    "UUID v7 must be chronologically sortable"
                );
            }
        );
    }

    mod validate {
        use super::*;
        use crate::{EmbedType, LinkType};

        /// 3.2-UNIT-022: Note Validation - Success.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn succeeds_when_all_entities_are_valid() {
            // GIVEN a note with valid sub-entities using NoteBuilder
            let note = NoteBuilder::new()
                .path("valid.md".into())
                .tags(vec![Tag::parse("#work").expect("Valid tag")])
                .headings(vec![
                    Heading::new(1, "Title".into(), 0).expect("Valid heading"),
                ])
                .links(vec![
                    Link::new_wikilink(
                        Uuid::now_v7(),
                        "target.md".into(),
                        None,
                        0,
                    )
                    .expect("Valid target"),
                ])
                .build();

            // WHEN validation is performed
            let result = note.validate();

            // THEN it succeeds
            assert!(
                result.is_ok(),
                "Expected valid note, got: {:?}",
                result.err()
            );
        }

        /// 3.2-UNIT-023: Note Validation - Invalid Heading.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn returns_error_when_heading_level_is_invalid() {
            // GIVEN a note with an invalid heading (manually constructed)
            let note = NoteBuilder::new()
                .path("valid.md".into())
                .headings(vec![Heading {
                    level: 0,
                    text: "Invalid".into(),
                    position: 0,
                }])
                .build();

            // WHEN the note is validated
            let result = note.validate();

            // THEN it returns an InvalidHeadingLevel error
            assert!(matches!(result, Err(DomainError::InvalidHeadingLevel(0))));
        }

        /// 3.2-UNIT-032: Note Validation - Empty Link Target.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn returns_error_when_link_target_is_empty() {
            // GIVEN a note with an empty link target
            let note = NoteBuilder::new()
                .links(vec![Link {
                    source_note_id: Uuid::now_v7(),
                    target_path: "".into(),
                    alias: None,
                    link_type: LinkType::WikiLink,
                    embed_type: None,
                    position: 0,
                }])
                .build();

            // WHEN validated
            let result = note.validate();

            // THEN it returns EmptyLinkTarget
            assert!(matches!(result, Err(DomainError::EmptyLinkTarget)));
        }

        /// 3.2-UNIT-033: Note Validation - Empty Embed Target.
        /// P1.
        #[test]
        #[expect(clippy::disallowed_methods, reason = "Test setup")]
        fn returns_error_when_embed_target_is_empty() {
            // GIVEN a note with an empty embed target
            let note = NoteBuilder::new()
                .embeds(vec![Link {
                    source_note_id: Uuid::now_v7(),
                    target_path: "".into(),
                    alias: None,
                    link_type: LinkType::Embed,
                    embed_type: Some(EmbedType::Image),
                    position: 0,
                }])
                .build();

            // WHEN validated
            let result = note.validate();

            // THEN it returns EmptyLinkTarget
            assert!(matches!(result, Err(DomainError::EmptyLinkTarget)));
        }
    }

    #[cfg(test)]
    mod proptests {
        use proptest::prelude::*;

        use super::*;

        proptest! {
            /// 3.2-PROP-001: Path Traversal Security Fuzzing.
            #[test]
            fn rejects_path_traversal(
                s in r#".*\.\..*"# // Generates strings containing ".."
            ) {
                let result = validate_vault_path(&s);
                prop_assert!(
                    result.is_err(),
                    "Path traversal '..' should always be rejected: {}",
                    s
                );
            }

            /// 3.2-PROP-002: Extension Enforcement Fuzzing.
            #[test]
            fn enforces_md_extension(
                s in r#"[a-zA-Z0-9/_-]{1,50}"# // Generates paths without extensions
            ) {
                // Ensure the string doesn't accidentally end with .md or .MD
                prop_assume!(!s.to_lowercase().ends_with(".md"));
                let result = validate_vault_path(&s);
                prop_assert!(
                    result.is_err(),
                    "Paths without .md extension must be rejected: {}",
                    s
                );
            }

            /// 3.2-PROP-003: Absolute Path Fuzzing.
            #[test]
            fn rejects_absolute_paths(
                s in r#"/.*"# // Generates paths starting with /
            ) {
                let result = validate_vault_path(&s);
                prop_assert!(
                    result.is_err(),
                    "Absolute paths must be rejected: {}",
                    s
                );
            }
        }
    }
}

/// Test fixtures for deterministic note data.
#[cfg(test)]
pub mod fixtures {
    use std::collections::HashMap;

    use super::*;
    use crate::models::frontmatter::FieldValue;

    /// Fixed UUID for deterministic tests (valid UUID v7 format).
    /// Uses timestamp 2024-01-01 00:00:00 UTC for consistency.
    pub const TEST_NOTE_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0001);

    /// Creates an example frontmatter for testing.
    ///
    /// # Panics
    /// Panics if the hardcoded date string is invalid (should never happen).
    #[inline]
    #[must_use]
    #[expect(
        clippy::disallowed_methods,
        reason = "Test fixture - unwrap/expect acceptable in test code"
    )]
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

    /// Creates an example tag for testing.
    ///
    /// # Panics
    /// Panics if the hardcoded tag string is invalid (should never happen).
    #[inline]
    #[must_use]
    #[expect(
        clippy::disallowed_methods,
        reason = "Test fixture - unwrap/expect acceptable in test code"
    )]
    pub fn example_tag() -> Tag {
        Tag::parse("work/project").expect("Valid tag")
    }

    /// Creates an example note for testing.
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
        }
    }
}
