//! Link subentity for Note aggregate.
//!
//! Represents wiki-links and references within notes.

use crate::errors::DomainError;

/// Represents different types of links that can appear in notes.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
    clippy::arbitrary_source_item_ordering,
    reason = "source_note_id must be first for logical grouping"
)]
pub struct Link {
    /// UUID of the note containing this link.
    pub source_note_id: uuid::Uuid,
    /// Path to the target note/file (vault-relative).
    pub target_path: Box<str>,
    /// Optional display alias (for [[target|alias]] or [text](url) syntax).
    pub alias: Option<Box<str>>,
    /// Type of link.
    pub link_type: LinkType,
    /// Type of embedded content (only present for Embed links).
    pub embed_type: Option<EmbedType>,
    /// Character position in the source document.
    pub position: usize,
}

#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "new_wikilink is more commonly used than new_markdown_link"
)]
impl Link {
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
    /// assert_eq!(link.target_path.as_ref(), "target.md");
    /// assert_eq!(link.alias, Some("Alias".into()));
    /// assert_eq!(link.link_type, LinkType::WikiLink);
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
        if target_path.is_empty() {
            return Err(DomainError::EmptyLinkTarget);
        }

        Ok(Self {
            source_note_id,
            target_path: target_path.into(),
            alias: alias.map(std::convert::Into::into),
            link_type: LinkType::WikiLink,
            embed_type: None,
            position,
        })
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
    /// assert_eq!(link.target_path.as_ref(), "doc.html");
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
        if target_path.is_empty() {
            return Err(DomainError::EmptyLinkTarget);
        }

        Ok(Self {
            source_note_id,
            target_path: target_path.into(),
            alias: alias.map(std::convert::Into::into),
            link_type: LinkType::MdLink,
            embed_type: None,
            position,
        })
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
    /// assert_eq!(embed.target_path.as_ref(), "diagram.png");
    /// assert_eq!(embed.link_type, LinkType::Embed);
    /// assert_eq!(embed.embed_type, Some(EmbedType::Image));
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
        if target_path.is_empty() {
            return Err(DomainError::EmptyLinkTarget);
        }

        Ok(Self {
            source_note_id,
            target_path: target_path.into(),
            alias: None, // Embeds don't have aliases
            link_type: LinkType::Embed,
            embed_type: Some(embed_type),
            position,
        })
    }

    /// Returns true if this link represents an embedded content reference.
    #[inline]
    #[must_use]
    pub fn is_embed(&self) -> bool {
        self.link_type == LinkType::Embed
    }
}
