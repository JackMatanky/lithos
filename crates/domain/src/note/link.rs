//! Link subentity for Note aggregate.
//!
//! Represents wiki-links and references within notes.

use crate::errors::DomainError;

/// Represents different types of links that can appear in notes.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub enum LinkType {
    /// Embedded content: ![[target]].
    Embed,
    /// Markdown-style link: [text](url).
    MdLink,
    /// Wiki-style link: [[target]] or [[target|alias]].
    WikiLink,
}

/// Represents different types of embedded content.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub enum EmbedType {
    /// Embedded audio: ![[audio.mp3]].
    Audio,
    /// Embedded image: ![[image.png]].
    Image,
    /// Embedded note content: ![[another-note]].
    Note,
    /// Embedded PDF: ![[document.pdf]].
    Pdf,
    /// Embedded video: ![[video.mp4]].
    Video,
}

/// Represents a link within a note.
///
/// Links can be wiki-links (Obsidian style), markdown links, or embeds.
/// Each link belongs to a specific note and has positioning information.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "pub(crate) used for internal builders and tests"
)]
#[expect(
    clippy::struct_field_names,
    reason = "link_type is the correct domain name"
)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Logical grouping preferred over alphabetical for domain models"
)]
pub struct Link {
    /// UUID of the note containing this link.
    pub(crate) source_note_id: uuid::Uuid,
    /// Path to the target note/file (vault-relative).
    pub(crate) target_path: Box<str>,
    /// Optional display alias (for [[target|alias]] or [text](url) syntax).
    pub(crate) alias: Option<Box<str>>,
    /// Type of link.
    pub(crate) link_type: LinkType,
    /// Type of embedded content (only present for Embed links).
    pub(crate) embed_type: Option<EmbedType>,
    /// Character position in the source document.
    pub(crate) position: usize,
}

/// Parameters for creating a Link.
#[derive(Debug)]
pub(crate) struct LinkParams {
    /// Optional display alias.
    pub alias: Option<String>,
    /// Type of embedded content.
    pub embed_type: Option<EmbedType>,
    /// Type of link.
    pub link_type: LinkType,
    /// Character position in source.
    pub position: usize,
    /// UUID of the note containing this link.
    pub source_note_id: uuid::Uuid,
    /// Path to the target note/file (vault-relative).
    pub target_path: Box<str>,
}

impl Link {
    /// Returns the optional display alias.
    #[inline]
    #[must_use]
    pub fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    /// Creates a Link instance with the given parameters.
    #[inline]
    fn create_link(params: LinkParams) -> Self {
        Self {
            source_note_id: params.source_note_id,
            target_path: params.target_path,
            alias: params.alias.map(std::convert::Into::into),
            link_type: params.link_type,
            embed_type: params.embed_type,
            position: params.position,
        }
    }

    /// Returns the type of embedded content, if applicable.
    #[inline]
    #[must_use]
    pub const fn embed_type(&self) -> Option<EmbedType> {
        self.embed_type
    }

    /// Returns true if this link stores an alias.
    #[inline]
    #[must_use]
    pub fn has_alias(&self) -> bool {
        self.alias.is_some()
    }

    /// Returns true if this link represents an embedded content reference.
    #[inline]
    #[must_use]
    pub fn is_embed(&self) -> bool {
        self.link_type == LinkType::Embed
    }

    /// Returns the type of link.
    #[inline]
    #[must_use]
    pub const fn link_type(&self) -> &LinkType {
        &self.link_type
    }

    /// Creates a new embedded content reference.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::{Link, LinkType, EmbedType};
    /// use uuid::Uuid;
    ///
    /// let source_id = Uuid::now_v7();
    /// let embed = Link::new_embed(
    ///     source_id,
    ///     "diagram.png".to_string(),
    ///     EmbedType::Image,
    ///     200
    /// ).unwrap();
    /// assert_eq!(embed.target_path(), "diagram.png");
    /// assert_eq!(embed.link_type(), &LinkType::Embed);
    /// assert_eq!(embed.embed_type(), Some(EmbedType::Image));
    /// ```
    ///
    /// # Errors
    /// Returns `DomainError::EmptyLinkTarget` if `target_path` is empty.
    #[inline]
    pub fn new_embed(
        source_note_id: uuid::Uuid,
        target_path: String,
        embed_type: EmbedType,
        position: usize,
    ) -> Result<Self, DomainError> {
        let target_path = Self::validate_path(target_path)?;
        Ok(Self::create_link(LinkParams {
            alias: None,
            embed_type: Some(embed_type),
            link_type: LinkType::Embed,
            position,
            source_note_id,
            target_path,
        }))
    }

    /// Creates a new markdown-style link.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::{Link, LinkType};
    /// use uuid::Uuid;
    ///
    /// let source_id = Uuid::now_v7();
    /// let link = Link::new_markdown_link(
    ///     source_id,
    ///     "doc.html".to_string(),
    ///     Some("Link".to_string()),
    ///     75
    /// ).unwrap();
    /// assert_eq!(link.target_path(), "doc.html");
    /// ```
    ///
    /// # Errors
    /// Returns `DomainError::EmptyLinkTarget` if `target_path` is empty.
    #[inline]
    pub fn new_markdown_link(
        source_note_id: uuid::Uuid,
        target_path: String,
        alias: Option<String>,
        position: usize,
    ) -> Result<Self, DomainError> {
        let target_path = Self::validate_path(target_path)?;
        Ok(Self::create_link(LinkParams {
            alias,
            embed_type: None,
            link_type: LinkType::MdLink,
            position,
            source_note_id,
            target_path,
        }))
    }

    /// Creates a new wiki-link.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::{Link, LinkType};
    /// use uuid::Uuid;
    ///
    /// let source_id = Uuid::now_v7();
    /// let link = Link::new_wikilink(
    ///     source_id,
    ///     "target.md".to_string(),
    ///     Some("Alias".to_string()),
    ///     100
    /// ).unwrap();
    /// assert_eq!(link.target_path(), "target.md");
    /// assert_eq!(link.alias(), Some("Alias"));
    /// assert_eq!(link.link_type(), &LinkType::WikiLink);
    /// ```
    ///
    /// # Errors
    /// Returns `DomainError::EmptyLinkTarget` if `target_path` is empty.
    #[inline]
    pub fn new_wikilink(
        source_note_id: uuid::Uuid,
        target_path: String,
        alias: Option<String>,
        position: usize,
    ) -> Result<Self, DomainError> {
        let target_path = Self::validate_path(target_path)?;
        Ok(Self::create_link(LinkParams {
            alias,
            embed_type: None,
            link_type: LinkType::WikiLink,
            position,
            source_note_id,
            target_path,
        }))
    }

    /// Returns the character position in the source document.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }

    /// Sets the source note ID.
    ///
    /// This is used by the Note aggregate to enforce consistency.
    #[inline]
    pub(crate) fn set_source_note_id(&mut self, id: uuid::Uuid) {
        self.source_note_id = id;
    }

    /// Returns the UUID of the note containing this link.
    #[inline]
    #[must_use]
    pub const fn source_note_id(&self) -> uuid::Uuid {
        self.source_note_id
    }

    /// Returns the target path of the link.
    #[inline]
    #[must_use]
    pub fn target_path(&self) -> &str {
        &self.target_path
    }

    /// Validates and converts a path string.
    #[inline]
    fn validate_path(path: String) -> Result<Box<str>, DomainError> {
        if path.is_empty() {
            return Err(DomainError::EmptyLinkTarget);
        }
        Ok(path.into())
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Unit tests use unwrap for readability"
)]
mod tests {
    use super::*;

    #[test]
    fn link_accessors_return_expected_values() {
        // GIVEN a markdown link
        let source_id = uuid::Uuid::now_v7();
        let link = Link::new_markdown_link(
            source_id,
            "doc.md".to_owned(),
            Some("Docs".to_owned()),
            42,
        )
        .unwrap();

        // THEN accessors expose the expected data
        assert_eq!(link.source_note_id(), source_id);
        assert_eq!(link.target_path(), "doc.md");
        assert_eq!(link.alias(), Some("Docs"));
        assert!(link.has_alias());
        assert_eq!(link.link_type(), &LinkType::MdLink);
        assert_eq!(link.embed_type(), None);
        assert_eq!(link.position(), 42);
        assert!(!link.is_embed());
    }

    #[test]
    fn embed_accessors_expose_embed_metadata() {
        // GIVEN an embed link
        let source_id = uuid::Uuid::now_v7();
        let embed = Link::new_embed(
            source_id,
            "image.png".to_owned(),
            EmbedType::Image,
            12,
        )
        .unwrap();

        // THEN embed accessors reflect embed-specific state
        assert_eq!(embed.link_type(), &LinkType::Embed);
        assert_eq!(embed.embed_type(), Some(EmbedType::Image));
        assert!(embed.is_embed());
    }

    #[test]
    fn rejects_empty_link_targets() {
        // GIVEN: an empty target path
        let source_id = uuid::Uuid::now_v7();

        // WHEN: creating links or embeds with the empty target
        let embed =
            Link::new_embed(source_id, String::new(), EmbedType::Note, 1);
        let markdown =
            Link::new_markdown_link(source_id, String::new(), None, 2);
        let wiki = Link::new_wikilink(source_id, String::new(), None, 3);

        // THEN: all constructors return an EmptyLinkTarget error
        assert!(matches!(embed, Err(DomainError::EmptyLinkTarget)));
        assert!(matches!(markdown, Err(DomainError::EmptyLinkTarget)));
        assert!(matches!(wiki, Err(DomainError::EmptyLinkTarget)));
    }
}
