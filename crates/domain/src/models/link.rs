//! Link subentity for Note aggregate.
//!
//! Represents wiki-links and references within notes.

use crate::errors::DomainError;

/// Represents different types of links and embeds that can appear in notes.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "LinkType is the correct domain name for link types"
)]
pub enum LinkType {
    /// Embedded audio: ![[audio.mp3]].
    EmbedAudio,
    /// Embedded image: ![[image.png]].
    EmbedImage,
    /// Embedded note content: ![[another-note]].
    EmbedNote,
    /// Embedded PDF: ![[document.pdf]].
    EmbedPdf,
    /// Embedded video: ![[video.mp4]].
    EmbedVideo,
    /// Wiki-style link: [[target]] or [[target|alias]].
    WikiLink,
}

/// Represents a link within a note.
///
/// Links can be wiki-links (Obsidian style) or other reference types.
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
    /// Optional display alias (for [[target|alias]] syntax).
    pub alias: Option<Box<str>>,
    /// Type of link.
    pub link_type: LinkType,
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
            link_type: LinkType::WikiLink, // For now, treat as wikilink
            position,
        })
    }

    /// Creates a new embedded content reference.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::models::link::{Link, LinkType};
    /// use uuid::Uuid;
    ///
    /// let source_id = Uuid::now_v7();
    /// let embed = Link::new_embed(
    ///     source_id,
    ///     "diagram.png".to_string(),
    ///     LinkType::EmbedImage,
    ///     200
    /// ).unwrap();
    /// assert_eq!(embed.target_path.as_ref(), "diagram.png");
    /// assert_eq!(embed.link_type, LinkType::EmbedImage);
    /// ```
    ///
    /// # Errors
    /// Returns `DomainError::EmptyLinkTarget` if `target_path` is empty.
    #[inline]
    pub fn new_embed(
        source_note_id: uuid::Uuid,
        target_path: String,
        link_type: LinkType,
        position: usize,
    ) -> Result<Self, DomainError> {
        if target_path.is_empty() {
            return Err(DomainError::EmptyLinkTarget);
        }

        // Validate that link_type is an embed type
        if !matches!(
            link_type,
            LinkType::EmbedAudio
                | LinkType::EmbedImage
                | LinkType::EmbedNote
                | LinkType::EmbedPdf
                | LinkType::EmbedVideo
        ) {
            return Err(DomainError::InvalidLinkType);
        }

        Ok(Self {
            source_note_id,
            target_path: target_path.into(),
            alias: None, // Embeds don't have aliases
            link_type,
            position,
        })
    }

    /// Returns true if this link represents an embedded content reference.
    #[inline]
    #[must_use]
    pub fn is_embed(&self) -> bool {
        matches!(
            self.link_type,
            LinkType::EmbedAudio
                | LinkType::EmbedImage
                | LinkType::EmbedNote
                | LinkType::EmbedPdf
                | LinkType::EmbedVideo
        )
    }
}
