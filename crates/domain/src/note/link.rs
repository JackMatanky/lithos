//! Link subentity for Note aggregate.
//!
//! Represents wiki-links, markdown links, and embeds within notes.
//! Links can reference resolved notes (existing in vault), unresolved notes
//! (not yet created), or external URLs.

use crate::errors::DomainError;

/// Represents different types of embedded content.
///
/// Used within [`Link`] to specify the media type being embedded when
/// `embed_type` is present.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub enum EmbedType {
    /// Embedded audio: `![[audio.mp3]]`.
    Audio,
    /// Embedded image: `![[image.png]]`.
    Image,
    /// Embedded note content: `![[another-note]]`.
    Note,
    /// Embedded PDF: `![[document.pdf]]`.
    Pdf,
    /// Embedded video: `![[video.mp4]]`.
    Video,
}

/// Represents the syntactic style of a link.
///
/// Distinguishes between Wiki-style links (`[[...]]`) and Markdown-style links
/// (`[...](...)`). Both styles can be either regular links or embeds depending
/// on the presence of an exclamation mark prefix (handled by
/// [`Link::is_embed`]).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub enum Style {
    /// Markdown-style link: `[text](url)` or `![text](url)`.
    MdLink,
    /// Wiki-style link: `[[target]]` or `![[target]]`.
    WikiLink,
}

/// Target of a link - may or may not resolve to an existing note.
///
/// This enum models the resolution state of a link target:
/// - [`Target::Resolved`]: Target exists in the vault and has been indexed.
/// - [`Target::Unresolved`]: Target doesn't exist yet (common in Obsidian for
///   "future notes").
/// - [`Target::External`]: Target is an external URL (http/https).
///
/// # Examples
/// ```
/// use lithos_domain::LinkTarget;
/// use uuid::Uuid;
///
/// // A resolved link to an existing note
/// let resolved = LinkTarget::Resolved {
///     id: Uuid::now_v7(),
///     path: "projects/rust.md".into(),
/// };
///
/// // An unresolved link to a note that doesn't exist yet
/// let unresolved = LinkTarget::Unresolved {
///     raw: "Future Project".into(),
/// };
///
/// // An external link
/// let external = LinkTarget::External {
///     url: "https://rust-lang.org".into(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum Target {
    /// External URL (http/https).
    External {
        /// The external URL.
        url: Box<str>,
    },
    /// Resolved: target exists in vault.
    Resolved {
        /// UUID of the target note.
        id: uuid::Uuid,
        /// Vault-relative path to the target.
        path: Box<str>,
    },
    /// Unresolved: target doesn't exist yet.
    Unresolved {
        /// Raw target string from the markdown source.
        raw: Box<str>,
    },
}

impl Target {
    /// Returns `true` if the target is an external URL.
    #[inline]
    #[must_use]
    pub const fn is_external(&self) -> bool {
        matches!(self, Self::External { .. })
    }

    /// Returns `true` if the target is resolved (exists in vault).
    #[inline]
    #[must_use]
    pub const fn is_resolved(&self) -> bool {
        matches!(self, Self::Resolved { .. })
    }

    /// Returns `true` if the target is unresolved (doesn't exist yet).
    #[inline]
    #[must_use]
    pub const fn is_unresolved(&self) -> bool {
        matches!(self, Self::Unresolved { .. })
    }

    /// Returns the path if resolved, or the raw string if unresolved.
    ///
    /// Returns `None` for external URLs.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics preferred for readability"
    )]
    pub fn vault_path(&self) -> Option<&str> {
        match self {
            Self::External {
                ..
            } => None,
            Self::Resolved {
                path,
                ..
            } => Some(path),
            Self::Unresolved {
                raw,
            } => Some(raw),
        }
    }
}

/// Sub-note anchor (heading or block reference).
///
/// Anchors allow linking to specific locations within a note:
/// - [`Anchor::Heading`]: Links to a heading (e.g., `[[note#heading]]`).
/// - [`Anchor::BlockRef`]: Links to a block (e.g., `[[note^block-id]]`).
///
/// # Examples
/// ```
/// use lithos_domain::LinkAnchor;
///
/// let heading = LinkAnchor::Heading("introduction".into());
/// let block = LinkAnchor::BlockRef("abc123".into());
///
/// assert!(heading.is_heading());
/// assert!(block.is_block_ref());
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum Anchor {
    /// Block reference: `^block-id`.
    BlockRef(Box<str>),
    /// Heading anchor: `#heading-text`.
    Heading(Box<str>),
}

impl Anchor {
    /// Returns `true` if this is a block reference.
    #[inline]
    #[must_use]
    pub const fn is_block_ref(&self) -> bool {
        matches!(self, Self::BlockRef(_))
    }

    /// Returns `true` if this is a heading anchor.
    #[inline]
    #[must_use]
    pub const fn is_heading(&self) -> bool {
        matches!(self, Self::Heading(_))
    }

    /// Returns the anchor text (heading text or block ID).
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics preferred for readability"
    )]
    pub fn text(&self) -> &str {
        match self {
            Self::BlockRef(s) | Self::Heading(s) => s,
        }
    }
}

/// Represents a link within a note.
///
/// Links can be wiki-links (Obsidian style) or markdown links.
/// Both styles can be embeds (prefixed with `!`), which is indicated by
/// the presence of `embed_type`.
///
/// All link types support:
/// - [`Target`]: The target (resolved, unresolved, or external).
/// - `alias`: Optional display text.
/// - `position`: Character position in the source document.
///
/// Wiki-links and markdown links additionally support:
/// - [`Anchor`]: Optional heading or block reference.
///
/// # Invariants
/// - Embeds cannot have anchors (enforced by [`Link::validate`]).
/// - External links cannot have block references (only heading anchors).
///
/// # Examples
/// ```
/// use lithos_domain::{EmbedType, Link, LinkAnchor, LinkStyle, LinkTarget};
/// use uuid::Uuid;
///
/// // Wiki-link to an unresolved note with heading anchor
/// let link = Link::new_wikilink(
///     LinkTarget::Unresolved {
///         raw: "Future Note".into(),
///     },
///     Some("my alias".to_string()),
///     Some(LinkAnchor::Heading("section".into())),
///     100,
/// )
/// .unwrap();
/// assert_eq!(link.style(), LinkStyle::WikiLink);
/// assert!(link.target().is_unresolved());
///
/// // Embed an image (Wiki-style)
/// let embed = Link::new_embed(
///     LinkTarget::Unresolved {
///         raw: "diagram.png".into(),
///     },
///     EmbedType::Image,
///     None,
///     200,
/// )
/// .unwrap();
/// assert!(embed.is_embed());
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Link {
    /// Target of the link.
    target: Target,
    /// Optional anchor (heading or block reference).
    anchor: Option<Anchor>,
    /// Character position in the source document.
    position: usize,
    /// Optional display alias.
    alias: Option<Box<str>>,
    /// Syntactic style of the link (Wiki vs Markdown).
    style: Style,
    /// Type of embedded content (if any).
    embed_type: Option<EmbedType>,
}

impl Link {
    /// Returns the optional display alias.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::{Link, LinkTarget};
    ///
    /// let link = Link::new_wikilink(
    ///     LinkTarget::Unresolved {
    ///         raw: "note".into(),
    ///     },
    ///     Some("display text".to_string()),
    ///     None,
    ///     0,
    /// )
    /// .unwrap();
    /// assert_eq!(link.alias(), Some("display text"));
    /// ```
    #[inline]
    #[must_use]
    pub fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    /// Returns the optional anchor (heading or block reference).
    #[inline]
    #[must_use]
    pub fn anchor(&self) -> Option<&Anchor> {
        self.anchor.as_ref()
    }

    /// Returns the type of embedded content, if this is an embed.
    #[inline]
    #[must_use]
    pub const fn embed_type(&self) -> Option<EmbedType> {
        self.embed_type
    }

    /// Returns `true` if this link has an alias.
    #[inline]
    #[must_use]
    pub fn has_alias(&self) -> bool {
        self.alias.is_some()
    }

    /// Returns `true` if this link has an anchor.
    #[inline]
    #[must_use]
    pub fn has_anchor(&self) -> bool {
        self.anchor.is_some()
    }

    /// Returns `true` if this link represents an embedded content reference.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::{EmbedType, Link, LinkTarget};
    ///
    /// let embed = Link::new_embed(
    ///     LinkTarget::Unresolved {
    ///         raw: "img.png".into(),
    ///     },
    ///     EmbedType::Image,
    ///     None,
    ///     0,
    /// )
    /// .unwrap();
    /// assert!(embed.is_embed());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_embed(&self) -> bool {
        self.embed_type.is_some()
    }

    /// Creates a new embedded content reference (Wiki-style by default).
    ///
    /// # Arguments
    /// * `target` - The target of the embed.
    /// * `embed_type` - The type of embedded content.
    /// * `alias` - Optional display alias.
    /// * `position` - Character position in the source document.
    ///
    /// # Errors
    /// Returns [`DomainError::EmptyLinkTarget`] if the target is empty.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::{EmbedType, Link, LinkStyle, LinkTarget};
    ///
    /// let embed = Link::new_embed(
    ///     LinkTarget::Unresolved {
    ///         raw: "diagram.png".into(),
    ///     },
    ///     EmbedType::Image,
    ///     None,
    ///     200,
    /// )
    /// .unwrap();
    /// assert_eq!(embed.style(), LinkStyle::WikiLink);
    /// assert!(embed.is_embed());
    /// ```
    #[inline]
    pub fn new_embed(
        target: Target,
        embed_type: EmbedType,
        alias: Option<String>,
        position: usize,
    ) -> Result<Self, DomainError> {
        Self::validate_target(&target)?;
        Ok(Self {
            alias: alias.map(Into::into),
            anchor: None, // Embeds don't support anchors
            embed_type: Some(embed_type),
            position,
            style: Style::WikiLink, // Default to Wiki-style for new_embed
            target,
        })
    }

    /// Creates a new markdown-style link.
    ///
    /// # Arguments
    /// * `target` - The target of the link.
    /// * `alias` - Optional display text (the `[text]` part).
    /// * `anchor` - Optional heading or block reference.
    /// * `position` - Character position in the source document.
    ///
    /// # Errors
    /// Returns [`DomainError::EmptyLinkTarget`] if the target is empty.
    /// Returns [`DomainError::InvalidLinkConfiguration`] if an external link
    /// has a block reference.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::{Link, LinkAnchor, LinkStyle, LinkTarget};
    ///
    /// let link = Link::new_markdown_link(
    ///     LinkTarget::External {
    ///         url: "https://rust-lang.org".into(),
    ///     },
    ///     Some("Rust".to_string()),
    ///     Some(LinkAnchor::Heading("install".into())),
    ///     75,
    /// )
    /// .unwrap();
    /// assert_eq!(link.style(), LinkStyle::MdLink);
    /// assert_eq!(link.alias(), Some("Rust"));
    /// ```
    #[inline]
    pub fn new_markdown_link(
        target: Target,
        alias: Option<String>,
        anchor: Option<Anchor>,
        position: usize,
    ) -> Result<Self, DomainError> {
        Self::validate_target(&target)?;
        Self::validate_external_anchor(&target, anchor.as_ref())?;
        Ok(Self {
            alias: alias.map(Into::into),
            anchor,
            embed_type: None, // Not an embed
            position,
            style: Style::MdLink,
            target,
        })
    }

    /// Creates a new wiki-link.
    ///
    /// # Arguments
    /// * `target` - The target of the link.
    /// * `alias` - Optional display alias (the `|alias` part).
    /// * `anchor` - Optional heading or block reference.
    /// * `position` - Character position in the source document.
    ///
    /// # Errors
    /// Returns [`DomainError::EmptyLinkTarget`] if the target is empty.
    /// Returns [`DomainError::InvalidLinkConfiguration`] if an external link
    /// has a block reference.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::{Link, LinkAnchor, LinkStyle, LinkTarget};
    /// use uuid::Uuid;
    ///
    /// let link = Link::new_wikilink(
    ///     LinkTarget::Resolved {
    ///         id: Uuid::now_v7(),
    ///         path: "target.md".into(),
    ///     },
    ///     Some("Alias".to_string()),
    ///     Some(LinkAnchor::Heading("intro".into())),
    ///     100,
    /// )
    /// .unwrap();
    /// assert_eq!(link.style(), LinkStyle::WikiLink);
    /// assert_eq!(link.alias(), Some("Alias"));
    /// assert!(link.anchor().is_some());
    /// ```
    #[inline]
    pub fn new_wikilink(
        target: Target,
        alias: Option<String>,
        anchor: Option<Anchor>,
        position: usize,
    ) -> Result<Self, DomainError> {
        Self::validate_target(&target)?;
        Self::validate_external_anchor(&target, anchor.as_ref())?;
        Ok(Self {
            alias: alias.map(Into::into),
            anchor,
            embed_type: None, // Not an embed
            position,
            style: Style::WikiLink,
            target,
        })
    }

    /// Returns the character position in the source document.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }

    /// Returns the style of the link (Wiki vs Markdown).
    #[inline]
    #[must_use]
    pub const fn style(&self) -> Style {
        self.style
    }

    /// Returns the target of the link.
    #[inline]
    #[must_use]
    pub const fn target(&self) -> &Target {
        &self.target
    }

    /// Validates the link's internal consistency.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidLinkConfiguration`] if:
    /// - An embed has an anchor.
    /// - An external link has a block reference.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::{EmbedType, Link, LinkTarget};
    ///
    /// let embed = Link::new_embed(
    ///     LinkTarget::Unresolved {
    ///         raw: "img.png".into(),
    ///     },
    ///     EmbedType::Image,
    ///     None,
    ///     0,
    /// )
    /// .unwrap();
    /// assert!(embed.validate().is_ok());
    /// ```
    #[inline]
    pub fn validate(&self) -> Result<(), DomainError> {
        // Embeds don't support anchors
        if self.is_embed() && self.anchor.is_some() {
            return Err(DomainError::InvalidLinkConfiguration(
                "Embeds cannot have anchors".into(),
            ));
        }

        // External links don't support block refs
        Self::validate_external_anchor(&self.target, self.anchor.as_ref())?;

        Ok(())
    }

    /// Validates that an external target doesn't have a block reference.
    fn validate_external_anchor(
        target: &Target,
        anchor: Option<&Anchor>,
    ) -> Result<(), DomainError> {
        if target.is_external() && matches!(anchor, Some(Anchor::BlockRef(_))) {
            return Err(DomainError::InvalidLinkConfiguration(
                "External links cannot have block references".into(),
            ));
        }
        Ok(())
    }

    /// Validates that the target is not empty.
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics preferred for readability"
    )]
    fn validate_target(target: &Target) -> Result<(), DomainError> {
        let is_empty = match target {
            Target::External {
                url,
            } => url.is_empty(),
            Target::Resolved {
                path,
                ..
            } => path.is_empty(),
            Target::Unresolved {
                raw,
            } => raw.is_empty(),
        };
        if is_empty {
            return Err(DomainError::EmptyLinkTarget);
        }
        Ok(())
    }
}
