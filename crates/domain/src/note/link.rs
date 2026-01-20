//! Link subentity for Note aggregate.
//!
//! Represents wiki-links, markdown links, and embeds within notes.
//! Links can reference resolved notes (existing in vault), unresolved notes
//! (not yet created), or external URLs.

use crate::errors::DomainError;

/// Represents different types of embedded content.
///
/// Used within [`LinkType::Embed`] to specify the media type being embedded.
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

/// Represents different types of links that can appear in notes.
///
/// All link types support aliases and can target resolved notes, unresolved
/// notes, or external URLs. Wiki-links and markdown links also support
/// anchors (heading or block references).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub enum LinkType {
    /// Embedded content: `![[target]]`, `![[target|alias]]`.
    Embed(EmbedType),
    /// Markdown-style link: `[text](url)`, `[text](url#heading)`.
    MdLink,
    /// Wiki-style link: `[[target]]`, `[[target|alias]]`, `[[target#heading]]`.
    WikiLink,
}

/// Target of a link - may or may not resolve to an existing note.
///
/// This enum models the resolution state of a link target:
/// - [`LinkTarget::Resolved`]: Target exists in the vault and has been indexed.
/// - [`LinkTarget::Unresolved`]: Target doesn't exist yet (common in Obsidian
///   for "future notes").
/// - [`LinkTarget::External`]: Target is an external URL (http/https).
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
pub enum LinkTarget {
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

impl LinkTarget {
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
/// - [`LinkAnchor::Heading`]: Links to a heading (e.g., `[[note#heading]]`).
/// - [`LinkAnchor::BlockRef`]: Links to a block (e.g., `[[note^block-id]]`).
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
pub enum LinkAnchor {
    /// Block reference: `^block-id`.
    BlockRef(Box<str>),
    /// Heading anchor: `#heading-text`.
    Heading(Box<str>),
}

impl LinkAnchor {
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
/// Links can be wiki-links (Obsidian style), markdown links, or embeds.
/// All link types support:
/// - [`LinkTarget`]: The target (resolved, unresolved, or external).
/// - `alias`: Optional display text.
/// - `position`: Character position in the source document.
///
/// Wiki-links and markdown links additionally support:
/// - [`LinkAnchor`]: Optional heading or block reference.
///
/// # Invariants
/// - Embeds cannot have anchors (enforced by [`Link::validate`]).
/// - External links cannot have block references (only heading anchors).
///
/// # Examples
/// ```
/// use lithos_domain::{EmbedType, Link, LinkAnchor, LinkTarget, LinkType};
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
/// assert_eq!(link.link_type(), LinkType::WikiLink);
/// assert!(link.target().is_unresolved());
///
/// // Embed an image
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
#[expect(
    clippy::struct_field_names,
    reason = "link_type is the correct domain name for this field"
)]
pub struct Link {
    /// Optional display alias.
    alias: Option<Box<str>>,
    /// Optional anchor (heading or block reference).
    anchor: Option<LinkAnchor>,
    /// Type of link (wiki, markdown, or embed).
    link_type: LinkType,
    /// Character position in the source document.
    position: usize,
    /// Target of the link.
    target: LinkTarget,
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
    pub fn anchor(&self) -> Option<&LinkAnchor> {
        self.anchor.as_ref()
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
        matches!(self.link_type, LinkType::Embed(_))
    }

    /// Returns the type of link.
    #[inline]
    #[must_use]
    pub const fn link_type(&self) -> LinkType {
        self.link_type
    }

    /// Creates a new embedded content reference.
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
    /// use lithos_domain::{EmbedType, Link, LinkTarget, LinkType};
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
    /// assert_eq!(embed.link_type(), LinkType::Embed(EmbedType::Image));
    /// assert!(embed.is_embed());
    /// ```
    #[inline]
    pub fn new_embed(
        target: LinkTarget,
        embed_type: EmbedType,
        alias: Option<String>,
        position: usize,
    ) -> Result<Self, DomainError> {
        Self::validate_target(&target)?;
        Ok(Self {
            alias: alias.map(Into::into),
            anchor: None, // Embeds don't support anchors
            link_type: LinkType::Embed(embed_type),
            position,
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
    /// use lithos_domain::{Link, LinkAnchor, LinkTarget, LinkType};
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
    /// assert_eq!(link.link_type(), LinkType::MdLink);
    /// assert_eq!(link.alias(), Some("Rust"));
    /// ```
    #[inline]
    pub fn new_markdown_link(
        target: LinkTarget,
        alias: Option<String>,
        anchor: Option<LinkAnchor>,
        position: usize,
    ) -> Result<Self, DomainError> {
        Self::validate_target(&target)?;
        Self::validate_external_anchor(&target, anchor.as_ref())?;
        Ok(Self {
            alias: alias.map(Into::into),
            anchor,
            link_type: LinkType::MdLink,
            position,
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
    /// use lithos_domain::{Link, LinkAnchor, LinkTarget, LinkType};
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
    /// assert_eq!(link.link_type(), LinkType::WikiLink);
    /// assert_eq!(link.alias(), Some("Alias"));
    /// assert!(link.anchor().is_some());
    /// ```
    #[inline]
    pub fn new_wikilink(
        target: LinkTarget,
        alias: Option<String>,
        anchor: Option<LinkAnchor>,
        position: usize,
    ) -> Result<Self, DomainError> {
        Self::validate_target(&target)?;
        Self::validate_external_anchor(&target, anchor.as_ref())?;
        Ok(Self {
            alias: alias.map(Into::into),
            anchor,
            link_type: LinkType::WikiLink,
            position,
            target,
        })
    }

    /// Returns the character position in the source document.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }

    /// Returns the target of the link.
    #[inline]
    #[must_use]
    pub const fn target(&self) -> &LinkTarget {
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
        target: &LinkTarget,
        anchor: Option<&LinkAnchor>,
    ) -> Result<(), DomainError> {
        if target.is_external()
            && matches!(anchor, Some(LinkAnchor::BlockRef(_)))
        {
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
    fn validate_target(target: &LinkTarget) -> Result<(), DomainError> {
        let is_empty = match target {
            LinkTarget::External {
                url,
            } => url.is_empty(),
            LinkTarget::Resolved {
                path,
                ..
            } => path.is_empty(),
            LinkTarget::Unresolved {
                raw,
            } => raw.is_empty(),
        };
        if is_empty {
            return Err(DomainError::EmptyLinkTarget);
        }
        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    clippy::str_to_string,
    reason = "Test code uses expect/unwrap for clarity and to_string for \
              convenience"
)]
mod tests {
    use super::*;

    mod link_target {
        use super::*;

        #[test]
        fn resolved_target_exposes_correct_properties() {
            // GIVEN: a resolved link target pointing to an existing note
            let target = LinkTarget::Resolved {
                id: uuid::Uuid::now_v7(),
                path: "notes/test.md".into(),
            };

            // WHEN: checking the target's properties
            // THEN: it identifies as resolved and provides the vault path
            assert!(target.is_resolved());
            assert!(!target.is_unresolved());
            assert!(!target.is_external());
            assert_eq!(target.vault_path(), Some("notes/test.md"));
        }

        #[test]
        fn unresolved_target_exposes_correct_properties() {
            // GIVEN: an unresolved link target (note doesn't exist yet)
            let target = LinkTarget::Unresolved {
                raw: "Future Note".into(),
            };

            // WHEN: checking the target's properties
            // THEN: it identifies as unresolved and provides the raw string
            assert!(!target.is_resolved());
            assert!(target.is_unresolved());
            assert!(!target.is_external());
            assert_eq!(target.vault_path(), Some("Future Note"));
        }

        #[test]
        fn external_target_exposes_correct_properties() {
            // GIVEN: an external link target (URL)
            let target = LinkTarget::External {
                url: "https://example.com".into(),
            };

            // WHEN: checking the target's properties
            // THEN: it identifies as external and has no vault path
            assert!(!target.is_resolved());
            assert!(!target.is_unresolved());
            assert!(target.is_external());
            assert_eq!(target.vault_path(), None);
        }
    }

    mod link_anchor {
        use super::*;

        #[test]
        fn heading_anchor_exposes_correct_properties() {
            // GIVEN: a heading anchor
            let anchor = LinkAnchor::Heading("introduction".into());

            // WHEN: checking the anchor's properties
            // THEN: it identifies as heading and provides the text
            assert!(anchor.is_heading());
            assert!(!anchor.is_block_ref());
            assert_eq!(anchor.text(), "introduction");
        }

        #[test]
        fn block_ref_anchor_exposes_correct_properties() {
            // GIVEN: a block reference anchor
            let anchor = LinkAnchor::BlockRef("abc123".into());

            // WHEN: checking the anchor's properties
            // THEN: it identifies as block ref and provides the text
            assert!(!anchor.is_heading());
            assert!(anchor.is_block_ref());
            assert_eq!(anchor.text(), "abc123");
        }
    }

    mod wikilink {
        use super::*;

        #[test]
        fn creates_basic_wikilink_with_minimal_parameters() {
            // GIVEN: an unresolved target and position
            let target = LinkTarget::Unresolved {
                raw: "target note".into(),
            };

            // WHEN: creating a basic wikilink without alias or anchor
            let link =
                Link::new_wikilink(target, None, None, 42).expect("Valid link");

            // THEN: the link has correct type and all accessors work
            assert_eq!(link.link_type(), LinkType::WikiLink);
            assert!(link.target().is_unresolved());
            assert_eq!(link.alias(), None);
            assert!(link.anchor().is_none());
            assert_eq!(link.position(), 42);
            assert!(!link.is_embed());
        }

        #[test]
        fn creates_wikilink_with_alias_and_heading_anchor() {
            // GIVEN: a resolved target, alias, and heading anchor
            let target = LinkTarget::Resolved {
                id: uuid::Uuid::now_v7(),
                path: "notes/test.md".into(),
            };
            let anchor = LinkAnchor::Heading("section".into());

            // WHEN: creating a wikilink with all optional parameters
            let link = Link::new_wikilink(
                target,
                Some("Display".to_string()),
                Some(anchor),
                100,
            )
            .expect("Valid link");

            // THEN: the link includes the alias and anchor
            assert_eq!(link.alias(), Some("Display"));
            assert!(link.has_alias());
            assert!(link.has_anchor());
            assert!(link.anchor().expect("has anchor").is_heading());
        }

        #[test]
        fn creates_wikilink_with_block_reference_anchor() {
            // GIVEN: an unresolved target and block reference anchor
            let target = LinkTarget::Unresolved {
                raw: "note".into(),
            };
            let anchor = LinkAnchor::BlockRef("block-id".into());

            // WHEN: creating a wikilink with block reference
            let link = Link::new_wikilink(target, None, Some(anchor), 0)
                .expect("Valid link");

            // THEN: the link has a block reference anchor
            assert!(link.anchor().expect("has anchor").is_block_ref());
        }

        #[test]
        fn rejects_wikilink_with_empty_target() {
            // GIVEN: an empty target string
            let target = LinkTarget::Unresolved {
                raw: "".into(),
            };

            // WHEN: attempting to create a wikilink
            let result = Link::new_wikilink(target, None, None, 0);

            // THEN: creation fails with EmptyLinkTarget error
            assert!(matches!(result, Err(DomainError::EmptyLinkTarget)));
        }
    }

    mod markdown_link {
        use super::*;

        #[test]
        fn creates_basic_markdown_link_to_external_url() {
            // GIVEN: an external URL target and display text
            let target = LinkTarget::External {
                url: "https://rust-lang.org".into(),
            };

            // WHEN: creating a markdown link
            let link = Link::new_markdown_link(
                target,
                Some("Rust".to_string()),
                None,
                0,
            )
            .expect("Valid link");

            // THEN: the link has correct type and properties
            assert_eq!(link.link_type(), LinkType::MdLink);
            assert!(link.target().is_external());
            assert_eq!(link.alias(), Some("Rust"));
        }

        #[test]
        fn creates_markdown_link_with_heading_anchor() {
            // GIVEN: an external URL and heading anchor
            let target = LinkTarget::External {
                url: "https://example.com".into(),
            };
            let anchor = LinkAnchor::Heading("section".into());

            // WHEN: creating a markdown link with anchor
            let link = Link::new_markdown_link(target, None, Some(anchor), 0)
                .expect("Valid link");

            // THEN: the link includes the heading anchor
            assert!(link.anchor().expect("has anchor").is_heading());
        }

        #[test]
        fn rejects_external_link_with_block_reference() {
            // GIVEN: an external URL and block reference anchor
            let target = LinkTarget::External {
                url: "https://example.com".into(),
            };
            let anchor = LinkAnchor::BlockRef("block".into());

            // WHEN: attempting to create a markdown link
            let result = Link::new_markdown_link(target, None, Some(anchor), 0);

            // THEN: creation fails because external links can't have block refs
            assert!(matches!(
                result,
                Err(DomainError::InvalidLinkConfiguration(_))
            ));
        }

        #[test]
        fn rejects_markdown_link_with_empty_url() {
            // GIVEN: an empty URL
            let target = LinkTarget::External {
                url: "".into(),
            };

            // WHEN: attempting to create a markdown link
            let result = Link::new_markdown_link(target, None, None, 0);

            // THEN: creation fails with EmptyLinkTarget error
            assert!(matches!(result, Err(DomainError::EmptyLinkTarget)));
        }
    }

    mod embed {
        use super::*;

        #[test]
        fn creates_image_embed_with_correct_type() {
            // GIVEN: an unresolved target for an image file
            let target = LinkTarget::Unresolved {
                raw: "diagram.png".into(),
            };

            // WHEN: creating an image embed
            let embed = Link::new_embed(target, EmbedType::Image, None, 200)
                .expect("Valid embed");

            // THEN: the embed has correct type and no anchor (embeds don't
            // support anchors)
            assert_eq!(embed.link_type(), LinkType::Embed(EmbedType::Image));
            assert!(embed.is_embed());
            assert!(embed.anchor().is_none());
        }

        #[test]
        fn creates_embed_with_display_alias() {
            // GIVEN: an unresolved target and alias (caption)
            let target = LinkTarget::Unresolved {
                raw: "image.png".into(),
            };

            // WHEN: creating an embed with alias
            let embed = Link::new_embed(
                target,
                EmbedType::Image,
                Some("Caption".to_string()),
                0,
            )
            .expect("Valid embed");

            // THEN: the embed includes the alias
            assert_eq!(embed.alias(), Some("Caption"));
        }

        #[test]
        fn creates_note_embed_with_resolved_target() {
            // GIVEN: a resolved target pointing to another note
            let target = LinkTarget::Resolved {
                id: uuid::Uuid::now_v7(),
                path: "other-note.md".into(),
            };

            // WHEN: creating a note embed
            let embed = Link::new_embed(target, EmbedType::Note, None, 0)
                .expect("Valid embed");

            // THEN: the embed has Note type
            assert_eq!(embed.link_type(), LinkType::Embed(EmbedType::Note));
        }

        #[test]
        fn rejects_embed_with_empty_target() {
            // GIVEN: an empty target string
            let target = LinkTarget::Unresolved {
                raw: "".into(),
            };

            // WHEN: attempting to create an embed
            let result = Link::new_embed(target, EmbedType::Image, None, 0);

            // THEN: creation fails with EmptyLinkTarget error
            assert!(matches!(result, Err(DomainError::EmptyLinkTarget)));
        }
    }

    mod validation {
        use super::*;

        #[test]
        fn valid_wikilink_passes_validation() {
            // GIVEN: a properly constructed wikilink
            let link = Link::new_wikilink(
                LinkTarget::Unresolved {
                    raw: "note".into(),
                },
                None,
                None,
                0,
            )
            .expect("Valid link");

            // WHEN: validating the link
            // THEN: validation passes
            link.validate().expect("Validation should pass");
        }

        #[test]
        fn valid_embed_passes_validation() {
            // GIVEN: a properly constructed embed
            let embed = Link::new_embed(
                LinkTarget::Unresolved {
                    raw: "img.png".into(),
                },
                EmbedType::Image,
                None,
                0,
            )
            .expect("Valid embed");

            // WHEN: validating the embed
            // THEN: validation passes
            embed.validate().expect("Validation should pass");
        }
    }
}
