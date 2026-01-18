//! Link subentity for Note aggregate.
//!
//! Represents wiki-links and references within notes.

use crate::errors::DomainError;

/// Represents different types of links that can appear in notes.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "LinkType is the correct domain name for link types"
)]
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

impl Link {
    /// Returns the optional display alias.
    #[inline]
    #[must_use]
    pub fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    /// Creates a Link instance with the given parameters.
    #[inline]
    fn create_link(
        source_note_id: uuid::Uuid,
        target_path: Box<str>,
        alias: Option<String>,
        link_type: LinkType,
        embed_type: Option<EmbedType>,
        position: usize,
    ) -> Self {
        Self {
            source_note_id,
            target_path,
            alias: alias.map(std::convert::Into::into),
            link_type,
            embed_type,
            position,
        }
    }

    /// Returns the type of embedded content, if applicable.
    #[inline]
    #[must_use]
    pub const fn embed_type(&self) -> Option<EmbedType> {
        self.embed_type
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
    /// use lithos_domain::models::link::{Link, LinkType, EmbedType};
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
        Ok(Self::create_link(
            source_note_id,
            target_path,
            None, // Embeds don't have aliases
            LinkType::Embed,
            Some(embed_type),
            position,
        ))
    }

    /// Creates a new markdown-style link.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::link::{Link, LinkType};
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
        Ok(Self::create_link(
            source_note_id,
            target_path,
            alias,
            LinkType::MdLink,
            None,
            position,
        ))
    }

    /// Creates a new wiki-link.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::link::{Link, LinkType};
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
        Ok(Self::create_link(
            source_note_id,
            target_path,
            alias,
            LinkType::WikiLink,
            None,
            position,
        ))
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
