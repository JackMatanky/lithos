//! Link subentity for Note aggregate.
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use super::error::NoteError;

/// Sub-note anchor (heading or block reference).
///
/// Anchors allow linking to specific locations within a note:
/// - [`Anchor::Heading`]: Links to a heading (e.g., `[[note#heading]]`).
/// - [`Anchor::BlockRef`]: Links to a block (e.g., `[[note^block-id]]`).
///
/// # Examples
/// ```
/// use lithos_core::note::link::Anchor;
///
/// let heading = Anchor::Heading("introduction".into());
/// let block = Anchor::BlockRef("abc123".into());
///
/// assert!(heading.is_heading(), "Heading anchor should be recognized");
/// assert!(
///     block.is_block_ref(),
///     "Block reference anchor should be recognized"
/// );
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum Anchor {
    /// Block reference: `^block-id`.
    BlockRef(Box<str>),
    /// Heading anchor: `#heading-text`.
    Heading(Box<str>),
}

/// Represents different types of embedded content.
///
/// Used within [`Link`] to specify the media type being embedded when
/// `embed_type` is present.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
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

/// Represents a link within a note.
///
/// Links can be wiki-links (Obsidian style) or markdown links.
/// Both styles can be embeds (prefixed with `!`), which is indicated by
/// the presence of `embed_type`.
///
/// All link types support:
/// - [`Target`]: The target (resolved, unresolved, or external).
/// - `alias`: Optional display alias.
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
/// use lithos_core::note::link::{Anchor, EmbedType, Link, Style, Target};
/// use uuid::Uuid;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// // Wiki-link to an unresolved note with heading anchor
/// let link = Link::new_wikilink(
///     Target::Unresolved {
///         raw: "Future Note".into(),
///     },
///     Some("my alias".to_string()),
///     Some(Anchor::Heading("section".into())),
///     100,
/// )?;
/// assert_eq!(link.style(), Style::WikiLink, "Link style should be WikiLink");
/// assert!(link.target().is_unresolved(), "Link target should be unresolved");
///
/// // Embed an image (Wiki-style)
/// let embed = Link::new_embed(
///     Target::Unresolved {
///         raw: "diagram.png".into(),
///     },
///     EmbedType::Image,
///     None,
///     200,
/// )?;
/// assert!(embed.is_embed(), "Link should be an embed");
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
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

/// Represents the syntactic style of a link.
///
/// Distinguishes between Wiki-style links (`[[...]]`) and Markdown-style links
/// (`[...](...)`). Both styles can be either regular links or embeds depending
/// on the presence of an exclamation mark prefix (handled by
/// [`Link::is_embed`]).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
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
/// use lithos_core::note::link::Target;
/// use uuid::Uuid;
///
/// // A resolved link to an existing note
/// let resolved = Target::Resolved {
///     id: Uuid::now_v7(),
///     path: "projects/rust.md".into(),
/// };
///
/// // An unresolved link to a note that doesn't exist yet
/// let unresolved = Target::Unresolved {
///     raw: "Future Project".into(),
/// };
///
/// // An external link
/// let external = Target::External {
///     url: "https://rust-lang.org".into(),
/// };
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
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
        reason = "Matching on &Anchor enum with non-Copy Box<str> fields. \
                  Cannot dereference without moving. Pattern binding in match \
                  arms is idiomatic for returning &str."
    )]
    pub fn text(&self) -> &str {
        match self {
            Self::BlockRef(s) | Self::Heading(s) => s,
        }
    }
}

impl Link {
    /// Returns the optional display alias.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::note::link::{Link, Target};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let link = Link::new_wikilink(
    ///     Target::Unresolved {
    ///         raw: "note".into(),
    ///     },
    ///     Some("display text".to_string()),
    ///     None,
    ///     0,
    /// )?;
    /// assert_eq!(link.alias(), Some("display text"));
    /// # Ok(())
    /// # }
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
    /// use lithos_core::note::link::{EmbedType, Link, Target};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let embed = Link::new_embed(
    ///     Target::Unresolved {
    ///         raw: "img.png".into(),
    ///     },
    ///     EmbedType::Image,
    ///     None,
    ///     0,
    /// )?;
    /// assert!(embed.is_embed(), "Embed link should be recognized");
    /// # Ok(())
    /// # }
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
    /// Returns `NoteError::Link` if the target is empty.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::note::link::{EmbedType, Link, Style, Target};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let embed = Link::new_embed(
    ///     Target::Unresolved {
    ///         raw: "diagram.png".into(),
    ///     },
    ///     EmbedType::Image,
    ///     None,
    ///     200,
    /// )?;
    /// assert_eq!(
    ///     embed.style(),
    ///     Style::WikiLink,
    ///     "Embed style should be WikiLink"
    /// );
    /// assert!(embed.is_embed(), "Link should be an embed");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn new_embed(
        target: Target,
        embed_type: EmbedType,
        alias: Option<String>,
        position: usize,
    ) -> Result<Self, NoteError> {
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
    /// Returns `NoteError::Link` if the target is empty.
    /// Returns `NoteError::Link` if an external link has a block reference.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::note::link::{Anchor, Link, Style, Target};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let link = Link::new_markdown_link(
    ///     Target::External {
    ///         url: "https://rust-lang.org".into(),
    ///     },
    ///     Some("Rust".to_string()),
    ///     Some(Anchor::Heading("install".into())),
    ///     75,
    /// )?;
    /// assert_eq!(link.style(), Style::MdLink);
    /// assert_eq!(link.alias(), Some("Rust"));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn new_markdown_link(
        target: Target,
        alias: Option<String>,
        anchor: Option<Anchor>,
        position: usize,
    ) -> Result<Self, NoteError> {
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
    /// Returns `NoteError::Link` if the target is empty.
    /// Returns `NoteError::Link` if an external link has a block reference.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::note::link::{Anchor, Link, Style, Target};
    /// use uuid::Uuid;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let link = Link::new_wikilink(
    ///     Target::Resolved {
    ///         id: Uuid::now_v7(),
    ///         path: "target.md".into(),
    ///     },
    ///     Some("Alias".to_string()),
    ///     Some(Anchor::Heading("intro".into())),
    ///     100,
    /// )?;
    /// assert_eq!(link.style(), Style::WikiLink, "Link style should be WikiLink");
    /// assert_eq!(link.alias(), Some("Alias"), "Link alias should match");
    /// assert!(link.anchor().is_some(), "Link anchor should be present");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn new_wikilink(
        target: Target,
        alias: Option<String>,
        anchor: Option<Anchor>,
        position: usize,
    ) -> Result<Self, NoteError> {
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
    /// Returns `NoteError::Link` if:
    /// - An embed has an anchor.
    /// - An external link has a block reference.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::note::link::{EmbedType, Link, Target};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let embed = Link::new_embed(
    ///     Target::Unresolved {
    ///         raw: "img.png".into(),
    ///     },
    ///     EmbedType::Image,
    ///     None,
    ///     0,
    /// )?;
    /// assert!(embed.validate().is_ok(), "Valid embed should pass validation");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn validate(&self) -> Result<(), NoteError> {
        // Embeds don't support anchors
        if self.is_embed() && self.anchor.is_some() {
            return Err(NoteError::Link("Embeds cannot have anchors".into()));
        }

        // External links don't support block refs
        Self::validate_external_anchor(&self.target, self.anchor.as_ref())?;

        Ok(())
    }

    /// Validates that an external target doesn't have a block reference.
    fn validate_external_anchor(
        target: &Target,
        anchor: Option<&Anchor>,
    ) -> Result<(), NoteError> {
        if target.is_external() && matches!(anchor, Some(Anchor::BlockRef(_))) {
            return Err(NoteError::Link(
                "External links cannot have block references".into(),
            ));
        }
        Ok(())
    }

    /// Validates that the target is not empty.
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on &Target enum with non-Copy Box<str> fields \
                  (path, raw, url). Cannot dereference without moving \
                  non-Copy fields. Pattern matching on target reference with \
                  field binding is idiomatic for validation."
    )]
    fn validate_target(target: &Target) -> Result<(), NoteError> {
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
            return Err(NoteError::Link("Link target cannot be empty".into()));
        }
        Ok(())
    }
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
        reason = "Matching on &Target enum with non-Copy Box<str> fields \
                  (path, raw, url). Cannot dereference without moving \
                  non-Copy fields. Pattern matching on &self with field \
                  binding is idiomatic for returning borrowed str."
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

#[cfg(test)]
mod tests {
    /// Test fixtures for Link testing.
    mod fixtures {
        use uuid::Uuid;

        use super::super::Target;

        const TEST_RESOLVED_ID: Uuid =
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0A01);

        /// Creates a valid unresolved target for testing.
        pub fn unresolved_target(name: &str) -> Target {
            Target::Unresolved {
                raw: name.into(),
            }
        }

        /// Creates a valid resolved target for testing.
        pub fn resolved_target(path: &str) -> Target {
            Target::Resolved {
                id: TEST_RESOLVED_ID,
                path: path.into(),
            }
        }

        /// Creates a valid external target for testing.
        pub fn external_target(url: &str) -> Target {
            Target::External {
                url: url.into(),
            }
        }
    }

    mod anchor {
        use super::super::Anchor;

        /// 3.2-UNIT-012: `heading_anchor_is_recognized`.
        /// Priority: P1.
        #[test]
        fn heading_anchor_is_recognized() {
            // GIVEN: a heading anchor
            let anchor = Anchor::Heading("introduction".into());

            // THEN: it should be identified as a heading
            assert!(
                anchor.is_heading(),
                "Anchor::Heading should return true for is_heading()"
            );
            assert!(
                !anchor.is_block_ref(),
                "Anchor::Heading should return false for is_block_ref()"
            );
            assert_eq!(
                anchor.text(),
                "introduction",
                "Anchor::Heading text should match"
            );
        }

        /// 3.2-UNIT-013: `block_ref_anchor_is_recognized`.
        /// Priority: P1.
        #[test]
        fn block_ref_anchor_is_recognized() {
            // GIVEN: a block reference anchor
            let anchor = Anchor::BlockRef("abc123".into());

            // THEN: it should be identified as a block reference
            assert!(
                anchor.is_block_ref(),
                "Anchor::BlockRef should return true for is_block_ref()"
            );
            assert!(
                !anchor.is_heading(),
                "Anchor::BlockRef should return false for is_heading()"
            );
            assert_eq!(
                anchor.text(),
                "abc123",
                "Anchor::BlockRef text should match"
            );
        }
    }

    mod target {
        use super::fixtures::{
            external_target, resolved_target, unresolved_target,
        };

        /// 3.2-UNIT-014: `resolved_target_indicates_resolution_state`.
        /// Priority: P1.
        #[test]
        fn resolved_target_indicates_resolution_state() {
            // GIVEN: a resolved target
            let target = resolved_target("projects/rust.md");

            // THEN: it should indicate resolved state
            assert!(
                target.is_resolved(),
                "Resolved target should return true for is_resolved()"
            );
            assert!(
                !target.is_unresolved(),
                "Resolved target should return false for is_unresolved()"
            );
            assert!(
                !target.is_external(),
                "Resolved target should return false for is_external()"
            );
            assert_eq!(
                target.vault_path(),
                Some("projects/rust.md"),
                "Resolved target should return path"
            );
        }

        /// 3.2-UNIT-015: `unresolved_target_indicates_resolution_state`.
        /// Priority: P1.
        #[test]
        fn unresolved_target_indicates_resolution_state() {
            // GIVEN: an unresolved target
            let target = unresolved_target("Future Note");

            // THEN: it should indicate unresolved state
            assert!(
                target.is_unresolved(),
                "Unresolved target should return true for is_unresolved()"
            );
            assert!(
                !target.is_resolved(),
                "Unresolved target should return false for is_resolved()"
            );
            assert!(
                !target.is_external(),
                "Unresolved target should return false for is_external()"
            );
            assert_eq!(
                target.vault_path(),
                Some("Future Note"),
                "Unresolved target should return raw text as path"
            );
        }

        /// 3.2-UNIT-016: `external_target_indicates_external_state`.
        /// Priority: P1.
        #[test]
        fn external_target_indicates_external_state() {
            // GIVEN: an external target
            let target = external_target("https://example.com");

            // THEN: it should indicate external state
            assert!(
                target.is_external(),
                "External target should return true for is_external()"
            );
            assert!(
                !target.is_resolved(),
                "External target should return false for is_resolved()"
            );
            assert!(
                !target.is_unresolved(),
                "External target should return false for is_unresolved()"
            );
            assert_eq!(
                target.vault_path(),
                None,
                "External target should return None for vault_path()"
            );
        }
    }

    mod constructors {
        use super::{
            super::{Anchor, EmbedType, Link, NoteError, Style},
            fixtures::{external_target, unresolved_target},
        };

        /// 3.2-UNIT-017: `new_wikilink_creates_valid_link`.
        /// Priority: P1.
        #[test]
        fn new_wikilink_creates_valid_link() {
            // GIVEN: an unresolved target with anchor and alias
            let target = unresolved_target("Target Note");
            let anchor = Some(Anchor::Heading("section".into()));
            let alias = Some("Display Text".to_owned());

            // WHEN: creating a wiki-link
            let link_result =
                Link::new_wikilink(target, alias.clone(), anchor, 100);
            assert!(
                link_result.is_ok(),
                "Expected valid wikilink, got: {link_result:?}"
            );
            let Ok(link) = link_result else {
                return;
            };

            // THEN: it should have correct properties
            assert_eq!(
                link.style(),
                Style::WikiLink,
                "Link should be WikiLink style"
            );
            assert!(
                link.target().is_unresolved(),
                "Link target should be unresolved"
            );
            assert_eq!(
                link.alias(),
                Some("Display Text"),
                "Link alias should match"
            );
            assert!(link.anchor().is_some(), "Link should have anchor");
            assert_eq!(link.position(), 100, "Link position should match");
            assert!(!link.is_embed(), "Wiki-link should not be an embed");
        }

        /// 3.2-UNIT-018: `new_markdown_link_creates_valid_link`.
        /// Priority: P1.
        #[test]
        fn new_markdown_link_creates_valid_link() {
            // GIVEN: an external URL target
            let target = external_target("https://example.com");

            // WHEN: creating a markdown link
            let link_result = Link::new_markdown_link(
                target,
                Some("Example".to_owned()),
                None,
                50,
            );
            assert!(
                link_result.is_ok(),
                "Expected valid markdown link, got: {link_result:?}"
            );
            let Ok(link) = link_result else {
                return;
            };

            // THEN: it should have correct properties
            assert_eq!(
                link.style(),
                Style::MdLink,
                "Link should be MdLink style"
            );
            assert!(
                link.target().is_external(),
                "Link target should be external"
            );
            assert!(!link.is_embed(), "Markdown link should not be an embed");
        }

        /// 3.2-UNIT-019: `new_embed_creates_valid_embed`.
        /// Priority: P1.
        #[test]
        fn new_embed_creates_valid_embed() {
            // GIVEN: an image target
            let target = unresolved_target("diagram.png");

            // WHEN: creating an embed
            let embed_result =
                Link::new_embed(target, EmbedType::Image, None, 200);
            assert!(
                embed_result.is_ok(),
                "Expected valid embed, got: {embed_result:?}"
            );
            let Ok(embed) = embed_result else {
                return;
            };

            // THEN: it should be a valid embed
            assert!(embed.is_embed(), "Link should be an embed");
            assert_eq!(
                embed.embed_type(),
                Some(EmbedType::Image),
                "Embed type should be Image"
            );
            assert_eq!(
                embed.style(),
                Style::WikiLink,
                "Embed should be WikiLink style"
            );
            assert!(embed.anchor().is_none(), "Embed should not have anchor");
        }

        /// 3.2-UNIT-020: `new_wikilink_rejects_empty_target`.
        /// Priority: P0.
        #[test]
        fn new_wikilink_rejects_empty_target() {
            // GIVEN: an empty unresolved target
            let target = unresolved_target("");

            // WHEN: creating a wiki-link
            let result = Link::new_wikilink(target, None, None, 0);

            // THEN: it should fail with empty target error
            assert!(
                matches!(&result, Err(NoteError::Link(msg)) if msg.contains("empty")),
                "Empty target should be rejected, got: {result:?}"
            );
        }

        /// 3.2-UNIT-021: `new_markdown_link_rejects_empty_target`.
        /// Priority: P0.
        #[test]
        fn new_markdown_link_rejects_empty_target() {
            // GIVEN: an empty external target
            let target = external_target("");

            // WHEN: creating a markdown link
            let result = Link::new_markdown_link(target, None, None, 0);

            // THEN: it should fail with empty target error
            assert!(
                matches!(&result, Err(NoteError::Link(msg)) if msg.contains("empty")),
                "Empty target should be rejected, got: {result:?}"
            );
        }

        /// 3.2-UNIT-022: `new_embed_rejects_empty_target`.
        /// Priority: P0.
        #[test]
        fn new_embed_rejects_empty_target() {
            // GIVEN: an empty unresolved target
            let target = unresolved_target("");

            // WHEN: creating an embed
            let result = Link::new_embed(target, EmbedType::Image, None, 0);

            // THEN: it should fail with empty target error
            assert!(
                matches!(&result, Err(NoteError::Link(msg)) if msg.contains("empty")),
                "Empty target should be rejected, got: {result:?}"
            );
        }
    }

    mod validators {
        use rstest::rstest;

        use super::{
            super::{Anchor, EmbedType, Link, NoteError, Style},
            fixtures::{external_target, resolved_target},
        };

        /// 3.2-UNIT-023: `validate_accepts_valid_wikilink`.
        /// Priority: P1.
        #[test]
        fn validate_accepts_valid_wikilink() {
            // GIVEN: a valid wiki-link with heading anchor
            let link_result = Link::new_wikilink(
                resolved_target("note.md"),
                None,
                Some(Anchor::Heading("section".into())),
                0,
            );
            assert!(
                link_result.is_ok(),
                "Expected valid wiki-link, got: {link_result:?}"
            );
            let Ok(link) = link_result else {
                return;
            };

            // WHEN: validating the link
            let result = link.validate();

            // THEN: it should pass validation
            assert!(
                result.is_ok(),
                "Valid wiki-link should pass validation, got: {result:?}"
            );
        }

        /// 3.2-UNIT-024: `validate_rejects_embed_with_anchor`.
        /// Priority: P0.
        #[test]
        fn validate_rejects_embed_with_anchor() {
            // GIVEN: an embed with an anchor (invalid combination)
            let embed = Link {
                target: resolved_target("image.png"),
                anchor: Some(Anchor::Heading("invalid".into())),
                position: 0,
                alias: None,
                style: Style::WikiLink,
                embed_type: Some(EmbedType::Image),
            };

            // WHEN: validating the embed
            let result = embed.validate();

            // THEN: it should fail with anchor error
            assert!(
                matches!(
                    &result,
                    Err(NoteError::Link(msg)) if msg.contains("anchor")
                ),
                "Embed with anchor should be rejected, got: {result:?}"
            );
        }

        /// 3.2-UNIT-025: `validate_rejects_external_link_with_block_ref`.
        /// Priority: P0.
        #[test]
        fn validate_rejects_external_link_with_block_ref() {
            // GIVEN: an external link with a block reference (invalid)
            // We construct directly to bypass constructor validation
            let link = Link {
                target: external_target("https://example.com"),
                anchor: Some(Anchor::BlockRef("block-id".into())),
                position: 0,
                alias: None,
                style: Style::MdLink,
                embed_type: None,
            };

            // WHEN: validating the link
            let result = link.validate();

            // THEN: it should fail with block ref error
            assert!(
                matches!(
                    &result,
                    Err(NoteError::Link(msg)) if msg.contains("block reference")
                ),
                "External link with block ref should be rejected, got: \
                 {result:?}"
            );
        }

        /// 3.2-UNIT-026: `validate_accepts_external_link_with_heading`.
        /// Priority: P1.
        #[test]
        fn validate_accepts_external_link_with_heading() {
            // GIVEN: an external link with a heading anchor (valid)
            let link_result = Link::new_markdown_link(
                external_target("https://example.com#section"),
                None,
                Some(Anchor::Heading("section".into())),
                0,
            );
            assert!(
                link_result.is_ok(),
                "Expected valid external markdown link, got: {link_result:?}"
            );
            let Ok(link) = link_result else {
                return;
            };

            // WHEN: validating the link
            let result = link.validate();

            // THEN: it should pass validation
            assert!(
                result.is_ok(),
                "External link with heading should be valid, got: {result:?}"
            );
        }

        /// 3.2-UNIT-027:
        /// `validate_accepts_resolved_target_with_all_embed_types`.
        /// Priority: P1.
        #[rstest]
        #[case::image(EmbedType::Image)]
        #[case::audio(EmbedType::Audio)]
        #[case::video(EmbedType::Video)]
        #[case::pdf(EmbedType::Pdf)]
        #[case::note(EmbedType::Note)]
        fn validate_accepts_resolved_target_with_all_embed_types(
            #[case] embed_type: EmbedType,
        ) {
            // GIVEN: a valid embed with different types
            let embed_result = Link::new_embed(
                resolved_target("content"),
                embed_type,
                None,
                0,
            );
            assert!(
                embed_result.is_ok(),
                "Expected valid embed, got: {embed_result:?}"
            );
            let Ok(embed) = embed_result else {
                return;
            };

            // WHEN: validating the embed
            let result = embed.validate();

            // THEN: it should pass validation
            assert!(
                result.is_ok(),
                "Valid embed with type {embed_type:?} should pass validation, \
                 got: {result:?}"
            );
        }
    }

    mod accessors {
        use rstest::rstest;

        use super::{
            super::{Anchor, EmbedType, Link},
            fixtures::{resolved_target, unresolved_target},
        };

        /// 3.2-UNIT-028: `accessors_return_expected_values`.
        /// Priority: P1.
        #[test]
        fn accessors_return_expected_values() {
            // GIVEN: a fully populated wiki-link
            let target = resolved_target("target.md");
            let anchor = Some(Anchor::Heading("section".into()));
            let alias = Some("Alias Text".to_owned());
            let link_result =
                Link::new_wikilink(target, alias.clone(), anchor.clone(), 42);
            assert!(
                link_result.is_ok(),
                "Expected valid link, got: {link_result:?}"
            );
            let Ok(link) = link_result else {
                return;
            };

            // THEN: all accessors should return expected values
            assert!(
                link.target().is_resolved(),
                "target() should return resolved target"
            );
            assert!(
                link.anchor().is_some(),
                "anchor() should return Some for link with anchor"
            );
            assert_eq!(
                link.alias(),
                Some("Alias Text"),
                "alias() should return the alias text"
            );
            assert_eq!(link.position(), 42, "position() should return 42");
            assert!(!link.is_embed(), "is_embed() should return false");
            assert!(
                link.embed_type().is_none(),
                "embed_type() should return None"
            );
        }

        /// 3.2-UNIT-029: `embed_type_accessor_returns_correct_type`.
        /// Priority: P1.
        #[rstest]
        #[case::image(EmbedType::Image)]
        #[case::audio(EmbedType::Audio)]
        #[case::video(EmbedType::Video)]
        fn embed_type_accessor_returns_correct_type(
            #[case] embed_type: EmbedType,
        ) {
            // GIVEN: an embed with a specific type
            let embed_result =
                Link::new_embed(unresolved_target("file"), embed_type, None, 0);
            assert!(
                embed_result.is_ok(),
                "Expected valid embed, got: {embed_result:?}"
            );
            let Ok(embed) = embed_result else {
                return;
            };

            // THEN: embed_type() should return the correct type
            assert_eq!(
                embed.embed_type(),
                Some(embed_type),
                "embed_type() should return {embed_type:?}"
            );
        }
    }
}
