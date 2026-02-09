//! Link subentity for Note aggregate.
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use super::{error::NoteError, types::SourceByteOffset};

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
    /// Images: `![[image.png]]`.
    Image,
    /// Audio: `![[audio.mp3]]`.
    Audio,
    /// Video: `![[video.mp4]]`.
    Video,
    /// PDF: `![[document.pdf]]`.
    Pdf,
    /// Transcluded note: `![[note]]`.
    Note,
}

/// Syntactic style of a link.
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
    /// Markdown-style: `[alias](target)`.
    MdLink,
    /// Wiki-style: `[[target|alias]]`.
    WikiLink,
}

/// Represents a link to another note or external resource.
///
/// Links are one of the primary ways notes are connected in Obsidian. Lithos
/// supports both Wiki-style and Markdown-style links, including anchors and
/// embeds.
///
/// # Examples
/// ```
/// use lithos_core::note::{
///     link::{Anchor, EmbedType, Link, Style, Target},
///     types::SourceByteOffset,
/// };
/// use uuid::Uuid;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// // Create a standard wiki-link with an alias and anchor
/// let link = Link::new_wikilink(
///     Target::Unresolved {
///         raw: "my note".into(),
///     },
///     Some("my alias".to_string()),
///     Some(Anchor::Heading("section".into())),
///     SourceByteOffset::new(100),
/// )?;
/// assert_eq!(link.style(), Style::WikiLink, "Link style should be WikiLink");
/// assert!(link.target().is_unresolved(), "Link target should be unresolved");
///
/// // Create an image embed
/// let embed = Link::new_embed(
///     Target::Unresolved {
///         raw: "image.png".into(),
///     },
///     EmbedType::Image,
///     None,
///     SourceByteOffset::new(200),
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
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub struct Link {
    /// Link target (internal path, UUID, or external URL).
    target: Target,
    /// Optional anchor (heading or block reference).
    anchor: Option<Anchor>,
    /// Byte offset in the source document.
    position: SourceByteOffset,
    /// Optional display alias.
    alias: Option<Box<str>>,
    /// Syntactic style of the link (Wiki vs Markdown).
    style: Style,
    /// Type of embed, if this is an embed link (`![[...]`).
    embed_type: Option<EmbedType>,
}

/// Link target representation.
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
#[rkyv(compare(PartialEq), derive(Debug))]
#[non_exhaustive]
pub enum Target {
    /// External URL: `[link](https://...)`.
    External {
        /// The external URL string.
        url: Box<str>,
    },
    /// Internal resolved link: `[[note]]`.
    Resolved {
        /// The unique identity of the target note.
        id: uuid::Uuid,
        /// The vault-relative path to the target note.
        path: Box<str>,
    },
    /// Internal unresolved link (broken or not yet indexed).
    Unresolved {
        /// The raw text of the link target.
        raw: Box<str>,
    },
}

impl Anchor {
    /// Returns `true` if the anchor is a block reference.
    #[inline]
    #[must_use]
    pub const fn is_block_ref(&self) -> bool {
        matches!(self, Self::BlockRef(_))
    }

    /// Returns `true` if the anchor is a heading.
    #[inline]
    #[must_use]
    pub const fn is_heading(&self) -> bool {
        matches!(self, Self::Heading(_))
    }

    /// Returns the raw text of the anchor.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on &Anchor enum with non-Copy Box<str> fields \
                  (BlockRef, Heading). Cannot dereference without moving \
                  non-Copy fields. Pattern matching on &self with field \
                  binding is idiomatic for returning borrowed str."
    )]
    pub fn text(&self) -> &str {
        match self {
            Self::BlockRef(text) | Self::Heading(text) => text,
        }
    }
}

impl Link {
    /// Creates an image/media/note embed (`![[...]`).
    ///
    /// # Arguments
    /// * `target` - The target of the embed.
    /// * `embed_type` - The type of content being embedded.
    /// * `alias` - Optional display text (e.g., image alt text).
    /// * `position` - Character position in the source document.
    ///
    /// # Errors
    /// Returns `NoteError::Link` if the target is empty.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::note::{
    ///     link::{EmbedType, Link, Target},
    ///     types::SourceByteOffset,
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let embed = Link::new_embed(
    ///     Target::Unresolved {
    ///         raw: "img.png".into(),
    ///     },
    ///     EmbedType::Image,
    ///     None,
    ///     SourceByteOffset::new(0),
    /// )?;
    /// assert!(embed.is_embed(), "Embed link should be recognized");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    /// # Examples
    /// ```
    /// use lithos_core::note::{
    ///     link::{EmbedType, Link, Style, Target},
    ///     types::SourceByteOffset,
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let embed = Link::new_embed(
    ///     Target::Unresolved {
    ///         raw: "diagram.png".into(),
    ///     },
    ///     EmbedType::Image,
    ///     None,
    ///     SourceByteOffset::new(200),
    /// )?;
    /// assert_eq!(
    ///     embed.style(),
    ///     Style::WikiLink,
    ///     "Embed style should be WikiLink"
    /// );
    ///
    /// assert!(embed.is_embed(), "Link should be an embed");
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_embed(
        target: Target,
        embed_type: EmbedType,
        alias: Option<String>,
        position: SourceByteOffset,
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
    /// use lithos_core::note::{
    ///     link::{Anchor, Link, Style, Target},
    ///     types::SourceByteOffset,
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let link = Link::new_markdown_link(
    ///     Target::External {
    ///         url: "https://rust-lang.org".into(),
    ///     },
    ///     Some("Rust".to_string()),
    ///     Some(Anchor::Heading("install".into())),
    ///     SourceByteOffset::new(75),
    /// )?;
    /// assert_eq!(link.style(), Style::MdLink, "Style should be Markdown");
    /// assert_eq!(link.alias(), Some("Rust"), "Alias should be set");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn new_markdown_link(
        target: Target,
        alias: Option<String>,
        anchor: Option<Anchor>,
        position: SourceByteOffset,
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
    /// use lithos_core::note::{
    ///     link::{Anchor, Link, Style, Target},
    ///     types::SourceByteOffset,
    /// };
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
    ///     SourceByteOffset::new(100),
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
        position: SourceByteOffset,
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

    /// Returns the byte offset in the source document.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
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
    /// use lithos_core::note::{
    ///     link::{EmbedType, Link, Target},
    ///     types::SourceByteOffset,
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let embed = Link::new_embed(
    ///     Target::Unresolved {
    ///         raw: "img.png".into(),
    ///     },
    ///     EmbedType::Image,
    ///     None,
    ///     SourceByteOffset::new(0),
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

    /// Returns `true` if this is an embed link (`![[...]`).
    #[inline]
    #[must_use]
    pub const fn is_embed(&self) -> bool {
        self.embed_type.is_some()
    }

    /// Returns the type of embed, if this is an embed link.
    #[inline]
    #[must_use]
    pub const fn embed_type(&self) -> Option<EmbedType> {
        self.embed_type
    }

    /// Returns the optional display alias.
    #[inline]
    #[must_use]
    pub fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    /// Returns the optional anchor.
    #[inline]
    #[must_use]
    pub const fn anchor(&self) -> Option<&Anchor> {
        self.anchor.as_ref()
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
#[expect(
    clippy::disallowed_methods,
    reason = "Test modules have relaxed unwrap/expect rules"
)]
mod tests {
    mod fixtures {
        use uuid::Uuid;

        use super::super::{Anchor, EmbedType, Link, Target};
        use crate::note::{error::NoteError, types::SourceByteOffset};

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

        pub fn wikilink_with_anchor_and_alias() -> Result<Link, NoteError> {
            Link::new_wikilink(
                unresolved_target("Target Note"),
                Some("Display Text".to_owned()),
                Some(Anchor::Heading("section".into())),
                SourceByteOffset::new(100u32),
            )
        }

        pub fn markdown_link_with_alias() -> Result<Link, NoteError> {
            Link::new_markdown_link(
                external_target("https://example.com"),
                Some("Example".to_owned()),
                None,
                SourceByteOffset::new(50u32),
            )
        }

        pub fn embed_image() -> Result<Link, NoteError> {
            Link::new_embed(
                unresolved_target("diagram.png"),
                EmbedType::Image,
                None,
                SourceByteOffset::new(200u32),
            )
        }

        pub fn resolved_wikilink_with_heading() -> Result<Link, NoteError> {
            Link::new_wikilink(
                resolved_target("note.md"),
                None,
                Some(Anchor::Heading("section".into())),
                SourceByteOffset::new(0u32),
            )
        }

        pub fn markdown_link_with_heading() -> Result<Link, NoteError> {
            Link::new_markdown_link(
                external_target("https://example.com#section"),
                None,
                Some(Anchor::Heading("section".into())),
                SourceByteOffset::new(0u32),
            )
        }

        pub fn embed_with_type(
            embed_type: EmbedType,
        ) -> Result<Link, NoteError> {
            Link::new_embed(
                resolved_target("content"),
                embed_type,
                None,
                SourceByteOffset::new(0u32),
            )
        }

        pub fn fully_populated_wikilink() -> Result<Link, NoteError> {
            Link::new_wikilink(
                resolved_target("target.md"),
                Some("Alias Text".to_owned()),
                Some(Anchor::Heading("section".into())),
                SourceByteOffset::new(42u32),
            )
        }
    }

    mod anchor {
        use super::super::Anchor;

        /// 3.2-UNIT-012: `heading_anchor_is_recognized`.
        /// Priority: P1.
        #[test]
        fn heading_anchor_reports_heading() {
            let anchor = Anchor::Heading("introduction".into());
            assert!(
                anchor.is_heading(),
                "Anchor::Heading should return true for is_heading()"
            );
        }

        /// 3.2-UNIT-012: `heading_anchor_is_recognized`.
        /// Priority: P1.
        #[test]
        fn heading_anchor_is_not_block_ref() {
            let anchor = Anchor::Heading("introduction".into());
            assert!(
                !anchor.is_block_ref(),
                "Anchor::Heading should return false for is_block_ref()"
            );
        }

        /// 3.2-UNIT-012: `heading_anchor_is_recognized`.
        /// Priority: P1.
        #[test]
        fn heading_anchor_returns_text() {
            let anchor = Anchor::Heading("introduction".into());
            assert_eq!(
                anchor.text(),
                "introduction",
                "Anchor::Heading text should match"
            );
        }

        /// 3.2-UNIT-013: `block_ref_anchor_is_recognized`.
        /// Priority: P1.
        #[test]
        fn block_ref_anchor_reports_block_ref() {
            let anchor = Anchor::BlockRef("abc123".into());
            assert!(
                anchor.is_block_ref(),
                "Anchor::BlockRef should return true for is_block_ref()"
            );
        }

        /// 3.2-UNIT-013: `block_ref_anchor_is_recognized`.
        /// Priority: P1.
        #[test]
        fn block_ref_anchor_is_not_heading() {
            let anchor = Anchor::BlockRef("abc123".into());
            assert!(
                !anchor.is_heading(),
                "Anchor::BlockRef should return false for is_heading()"
            );
        }

        /// 3.2-UNIT-013: `block_ref_anchor_is_recognized`.
        /// Priority: P1.
        #[test]
        fn block_ref_anchor_returns_text() {
            let anchor = Anchor::BlockRef("abc123".into());
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
        fn resolved_target_is_resolved() {
            let target = resolved_target("projects/rust.md");
            assert!(
                target.is_resolved(),
                "Resolved target should return true for is_resolved()"
            );
        }

        /// 3.2-UNIT-014: `resolved_target_indicates_resolution_state`.
        /// Priority: P1.
        #[test]
        fn resolved_target_is_not_unresolved() {
            let target = resolved_target("projects/rust.md");
            assert!(
                !target.is_unresolved(),
                "Resolved target should return false for is_unresolved()"
            );
        }

        /// 3.2-UNIT-014: `resolved_target_indicates_resolution_state`.
        /// Priority: P1.
        #[test]
        fn resolved_target_is_not_external() {
            let target = resolved_target("projects/rust.md");
            assert!(
                !target.is_external(),
                "Resolved target should return false for is_external()"
            );
        }

        /// 3.2-UNIT-014: `resolved_target_indicates_resolution_state`.
        /// Priority: P1.
        #[test]
        fn resolved_target_returns_vault_path() {
            let target = resolved_target("projects/rust.md");
            assert_eq!(
                target.vault_path(),
                Some("projects/rust.md"),
                "Resolved target should return path"
            );
        }

        /// 3.2-UNIT-015: `unresolved_target_indicates_resolution_state`.
        /// Priority: P1.
        #[test]
        fn unresolved_target_is_unresolved() {
            let target = unresolved_target("Future Note");
            assert!(
                target.is_unresolved(),
                "Unresolved target should return true for is_unresolved()"
            );
        }

        /// 3.2-UNIT-015: `unresolved_target_indicates_resolution_state`.
        /// Priority: P1.
        #[test]
        fn unresolved_target_is_not_resolved() {
            let target = unresolved_target("Future Note");
            assert!(
                !target.is_resolved(),
                "Unresolved target should return false for is_resolved()"
            );
        }

        /// 3.2-UNIT-015: `unresolved_target_indicates_resolution_state`.
        /// Priority: P1.
        #[test]
        fn unresolved_target_is_not_external() {
            let target = unresolved_target("Future Note");
            assert!(
                !target.is_external(),
                "Unresolved target should return false for is_external()"
            );
        }

        /// 3.2-UNIT-015: `unresolved_target_indicates_resolution_state`.
        /// Priority: P1.
        #[test]
        fn unresolved_target_returns_vault_path() {
            let target = unresolved_target("Future Note");
            assert_eq!(
                target.vault_path(),
                Some("Future Note"),
                "Unresolved target should return raw text as path"
            );
        }

        /// 3.2-UNIT-016: `external_target_indicates_external_state`.
        /// Priority: P1.
        #[test]
        fn external_target_is_external() {
            let target = external_target("https://example.com");
            assert!(
                target.is_external(),
                "External target should return true for is_external()"
            );
        }

        /// 3.2-UNIT-016: `external_target_indicates_external_state`.
        /// Priority: P1.
        #[test]
        fn external_target_is_not_resolved() {
            let target = external_target("https://example.com");
            assert!(
                !target.is_resolved(),
                "External target should return false for is_resolved()"
            );
        }

        /// 3.2-UNIT-016: `external_target_indicates_external_state`.
        /// Priority: P1.
        #[test]
        fn external_target_is_not_unresolved() {
            let target = external_target("https://example.com");
            assert!(
                !target.is_unresolved(),
                "External target should return false for is_unresolved()"
            );
        }

        /// 3.2-UNIT-016: `external_target_indicates_external_state`.
        /// Priority: P1.
        #[test]
        fn external_target_returns_no_vault_path() {
            let target = external_target("https://example.com");
            assert_eq!(
                target.vault_path(),
                None,
                "External target should return None for vault_path()"
            );
        }
    }

    mod constructor {
        use super::{
            super::{EmbedType, Link, Style},
            fixtures::{
                embed_image, external_target, markdown_link_with_alias,
                unresolved_target, wikilink_with_anchor_and_alias,
            },
        };
        use crate::note::{error::NoteError, types::SourceByteOffset};

        /// 3.2-UNIT-017: `new_wikilink_creates_valid_link`.
        /// Priority: P1.
        #[test]
        fn wikilink_style_is_wikilink() {
            let link =
                wikilink_with_anchor_and_alias().expect("valid wikilink setup");
            assert_eq!(
                link.style(),
                Style::WikiLink,
                "Link should be WikiLink style"
            );
        }

        /// 3.2-UNIT-017: `new_wikilink_creates_valid_link`.
        /// Priority: P1.
        #[test]
        fn wikilink_target_is_unresolved() {
            let link =
                wikilink_with_anchor_and_alias().expect("valid wikilink setup");
            assert!(
                link.target().is_unresolved(),
                "Link target should be unresolved"
            );
        }

        /// 3.2-UNIT-017: `new_wikilink_creates_valid_link`.
        /// Priority: P1.
        #[test]
        fn wikilink_alias_matches() {
            let link =
                wikilink_with_anchor_and_alias().expect("valid wikilink setup");
            assert_eq!(
                link.alias(),
                Some("Display Text"),
                "Link alias should match"
            );
        }

        /// 3.2-UNIT-017: `new_wikilink_creates_valid_link`.
        /// Priority: P1.
        #[test]
        fn wikilink_has_anchor() {
            let link =
                wikilink_with_anchor_and_alias().expect("valid wikilink setup");
            assert!(link.anchor().is_some(), "Link should have anchor");
        }

        /// 3.2-UNIT-017: `new_wikilink_creates_valid_link`.
        /// Priority: P1.
        #[test]
        fn wikilink_position_matches() {
            let link =
                wikilink_with_anchor_and_alias().expect("valid wikilink setup");
            assert_eq!(
                link.position(),
                SourceByteOffset::new(100u32),
                "Link position should match"
            );
        }

        /// 3.2-UNIT-017: `new_wikilink_creates_valid_link`.
        /// Priority: P1.
        #[test]
        fn wikilink_is_not_embed() {
            let link =
                wikilink_with_anchor_and_alias().expect("valid wikilink setup");
            assert!(!link.is_embed(), "Wiki-link should not be an embed");
        }

        /// 3.2-UNIT-018: `new_markdown_link_creates_valid_link`.
        /// Priority: P1.
        #[test]
        fn markdown_link_style_is_mdlink() {
            let link =
                markdown_link_with_alias().expect("valid markdown link setup");
            assert_eq!(
                link.style(),
                Style::MdLink,
                "Link should be MdLink style"
            );
        }

        /// 3.2-UNIT-018: `new_markdown_link_creates_valid_link`.
        /// Priority: P1.
        #[test]
        fn markdown_link_target_is_external() {
            let link =
                markdown_link_with_alias().expect("valid markdown link setup");
            assert!(
                link.target().is_external(),
                "Link target should be external"
            );
        }

        /// 3.2-UNIT-018: `new_markdown_link_creates_valid_link`.
        /// Priority: P1.
        #[test]
        fn markdown_link_is_not_embed() {
            let link =
                markdown_link_with_alias().expect("valid markdown link setup");
            assert!(!link.is_embed(), "Markdown link should not be an embed");
        }

        /// 3.2-UNIT-019: `new_embed_creates_valid_embed`.
        /// Priority: P1.
        #[test]
        fn embed_reports_is_embed() {
            let embed = embed_image().expect("valid embed setup");
            assert!(embed.is_embed(), "Link should be an embed");
        }

        /// 3.2-UNIT-019: `new_embed_creates_valid_embed`.
        /// Priority: P1.
        #[test]
        fn embed_type_is_image() {
            let embed = embed_image().expect("valid embed setup");
            assert_eq!(
                embed.embed_type(),
                Some(EmbedType::Image),
                "Embed type should be Image"
            );
        }

        /// 3.2-UNIT-019: `new_embed_creates_valid_embed`.
        /// Priority: P1.
        #[test]
        fn embed_style_is_wikilink() {
            let embed = embed_image().expect("valid embed setup");
            assert_eq!(
                embed.style(),
                Style::WikiLink,
                "Embed should be WikiLink style"
            );
        }

        /// 3.2-UNIT-019: `new_embed_creates_valid_embed`.
        /// Priority: P1.
        #[test]
        fn embed_has_no_anchor() {
            let embed = embed_image().expect("valid embed setup");
            assert!(embed.anchor().is_none(), "Embed should not have anchor");
        }

        /// 3.2-UNIT-020: `new_wikilink_rejects_empty_target`.
        /// Priority: P0.
        #[test]
        fn wikilink_rejects_empty_target() {
            // GIVEN: an empty unresolved target
            let target = unresolved_target("");

            // WHEN: creating a wiki-link
            let result = Link::new_wikilink(
                target,
                None,
                None,
                SourceByteOffset::new(0u32),
            );

            // THEN: it should fail with empty target error
            assert!(
                matches!(&result, Err(NoteError::Link(msg)) if msg.contains("empty")),
                "Empty target should be rejected, got: {result:?}"
            );
        }

        /// 3.2-UNIT-021: `new_markdown_link_rejects_empty_target`.
        /// Priority: P0.
        #[test]
        fn markdown_link_rejects_empty_target() {
            // GIVEN: an empty external target
            let target = external_target("");

            // WHEN: creating a markdown link
            let result = Link::new_markdown_link(
                target,
                None,
                None,
                SourceByteOffset::new(0u32),
            );

            // THEN: it should fail with empty target error
            assert!(
                matches!(&result, Err(NoteError::Link(msg)) if msg.contains("empty")),
                "Empty target should be rejected, got: {result:?}"
            );
        }

        /// 3.2-UNIT-022: `new_embed_rejects_empty_target`.
        /// Priority: P0.
        #[test]
        fn embed_rejects_empty_target() {
            // GIVEN: an empty unresolved target
            let target = unresolved_target("");

            // WHEN: creating an embed
            let result = Link::new_embed(
                target,
                EmbedType::Image,
                None,
                SourceByteOffset::new(0u32),
            );

            // THEN: it should fail with empty target error
            assert!(
                matches!(&result, Err(NoteError::Link(msg)) if msg.contains("empty")),
                "Empty target should be rejected, got: {result:?}"
            );
        }
    }

    mod validation {
        use rstest::rstest;

        use super::{
            super::{Anchor, EmbedType, Link, Style},
            fixtures::{
                embed_with_type, external_target, markdown_link_with_heading,
                resolved_target, resolved_wikilink_with_heading,
            },
        };
        use crate::note::{error::NoteError, types::SourceByteOffset};

        /// 3.2-UNIT-023: `validate_accepts_valid_wikilink`.
        /// Priority: P1.
        #[test]
        fn accepts_valid_wikilink() {
            let link = resolved_wikilink_with_heading().expect("valid setup");
            link.validate().unwrap();
        }

        /// 3.2-UNIT-024: `validate_rejects_embed_with_anchor`.
        /// Priority: P0.
        #[test]
        fn rejects_embed_with_anchor() {
            // GIVEN: an embed with an anchor (invalid combination)
            let embed = Link {
                target: resolved_target("image.png"),
                anchor: Some(Anchor::Heading("invalid".into())),
                position: SourceByteOffset::new(0u32),
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
        fn rejects_external_link_with_block_ref() {
            // GIVEN: an external link with a block reference (invalid)
            // We construct directly to bypass constructor validation
            let link = Link {
                target: external_target("https://example.com"),
                anchor: Some(Anchor::BlockRef("block-id".into())),
                position: SourceByteOffset::new(0u32),
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
        fn accepts_external_link_with_heading() {
            let link = markdown_link_with_heading().expect("valid setup");
            link.validate().unwrap();
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
        fn accepts_resolved_target_with_all_embed_types(
            #[case] embed_type: EmbedType,
        ) {
            let embed = embed_with_type(embed_type).expect("valid setup");
            embed.validate().unwrap();
        }
    }

    mod accessors {
        use rstest::rstest;

        use super::{
            super::{EmbedType, Link},
            fixtures::{fully_populated_wikilink, unresolved_target},
        };
        use crate::note::{error::NoteError, types::SourceByteOffset};

        fn embed_with_type(embed_type: EmbedType) -> Result<Link, NoteError> {
            Link::new_embed(
                unresolved_target("file"),
                embed_type,
                None,
                SourceByteOffset::new(0u32),
            )
        }

        /// 3.2-UNIT-028: `accessors_return_expected_values`.
        /// Priority: P1.
        #[test]
        fn target_returns_resolved() {
            let link = fully_populated_wikilink().expect("valid setup");
            assert!(link.target().is_resolved());
        }

        /// 3.2-UNIT-028: `accessors_return_expected_values`.
        /// Priority: P1.
        #[test]
        fn anchor_returns_some() {
            let link = fully_populated_wikilink().expect("valid setup");
            assert!(link.anchor().is_some());
        }

        /// 3.2-UNIT-028: `accessors_return_expected_values`.
        /// Priority: P1.
        #[test]
        fn alias_returns_alias_text() {
            let link = fully_populated_wikilink().expect("valid setup");
            assert_eq!(link.alias(), Some("Alias Text"));
        }

        /// 3.2-UNIT-028: `accessors_return_expected_values`.
        /// Priority: P1.
        #[test]
        fn position_returns_position() {
            let link = fully_populated_wikilink().expect("valid setup");
            assert_eq!(link.position(), SourceByteOffset::new(42u32));
        }

        /// 3.2-UNIT-028: `accessors_return_expected_values`.
        /// Priority: P1.
        #[test]
        fn is_embed_returns_false_for_wikilink() {
            let link = fully_populated_wikilink().expect("valid setup");
            assert!(!link.is_embed());
        }

        /// 3.2-UNIT-028: `accessors_return_expected_values`.
        /// Priority: P1.
        #[test]
        fn embed_type_returns_none_for_wikilink() {
            let link = fully_populated_wikilink().expect("valid setup");
            assert!(link.embed_type().is_none());
        }

        /// 3.2-UNIT-029: `embed_type_accessor_returns_correct_type`.
        /// Priority: P1.
        #[rstest]
        #[case::image(EmbedType::Image)]
        #[case::audio(EmbedType::Audio)]
        #[case::video(EmbedType::Video)]
        fn embed_type_returns_correct_type(#[case] embed_type: EmbedType) {
            let embed = embed_with_type(embed_type).expect("valid setup");
            assert_eq!(embed.embed_type(), Some(embed_type));
        }
    }
}
