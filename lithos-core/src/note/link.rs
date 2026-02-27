//! Link and embed entities for document relationships.
//!
//! Models wiki-links, markdown links, and embedded content references
//! with support for anchors and aliases.

//! Link subentity for Note aggregate.
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use super::{
    aggregate::{NoteId, NotePath},
    error::NoteError,
    types::SourceByteOffset,
};

/// Sub-note anchor (heading or block reference).
///
/// An anchor represents a specific location within a note, allowing links
/// to point to more than just the file level.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::link::Anchor;
/// let heading = Anchor::Heading("Introduction".into());
/// let block = Anchor::BlockRef("abc123".into());
///
/// assert!(heading.is_heading());
/// assert_eq!(block.text(), "abc123");
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
/// Used when a link is prefixed with `!` (e.g., `![[image.png]]`),
/// indicating that the content should be displayed inline.
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
/// Supports multiple syntactic styles (Wiki-links vs Markdown links) and
/// can represent both internal vault references and external URLs.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::{link::{Link, Target}, types::SourceByteOffset};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let target = Target::Unresolved { raw: "Main Page".into() };
/// let link = Link::new_wikilink(target, None, None, SourceByteOffset::new(0))?;
///
/// assert!(!link.is_embed());
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
    target: Target,
    anchor: Option<Anchor>,
    position: SourceByteOffset,
    alias: Option<Box<str>>,
    style: Style,
    embed_type: Option<EmbedType>,
}

/// Represents the syntactic style of a link.
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
/// # Examples
///
/// ```
/// # use lithos_core::note::link::Target;
/// let external = Target::External {
///     url: "https://rust-lang.org".into(),
/// };
/// let unresolved = Target::Unresolved {
///     raw: "New Note".into(),
/// };
///
/// assert!(external.is_external());
/// assert_eq!(unresolved.vault_path(), Some("New Note"));
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
    /// External URL (scheme-based).
    External {
        /// The external URL.
        url: Box<str>,
    },
    /// Resolved: target exists in vault.
    Resolved {
        /// Identifier of the target note.
        id: NoteId,
        /// Vault-relative path to the target.
        path: NotePath,
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
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on &Anchor with non-Copy fields uses match \
                  ergonomics."
    )]
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        match self {
            Self::BlockRef(s) | Self::Heading(s) => s,
        }
    }
}

impl Link {
    /// Returns the optional display alias.
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
    #[inline]
    #[must_use]
    pub const fn is_embed(&self) -> bool {
        self.embed_type.is_some()
    }

    /// Creates a new embedded content reference (wiki embed).
    ///
    /// # Errors
    /// Returns `NoteError::Link` if the target is empty.
    #[inline]
    pub fn new_embed(
        target: Target,
        embed_type: EmbedType,
        alias: Option<&str>,
        anchor: Option<Anchor>,
        position: SourceByteOffset,
    ) -> Result<Self, NoteError> {
        Self::validate_target(&target)?;
        Self::validate_external_anchor(&target, anchor.as_ref())?;
        Ok(Self {
            alias: alias.map(Into::into),
            anchor,
            embed_type: Some(embed_type),
            position,
            style: Style::WikiLink,
            target,
        })
    }

    /// Creates a new embedded content reference from markdown image syntax.
    ///
    /// # Errors
    /// Returns `NoteError::Link` if validation fails.
    #[inline]
    pub fn new_markdown_embed(
        target: Target,
        embed_type: EmbedType,
        alias: Option<&str>,
        position: SourceByteOffset,
    ) -> Result<Self, NoteError> {
        Self::validate_target(&target)?;
        Self::validate_external_anchor(&target, None)?;
        Ok(Self {
            alias: alias.map(Into::into),
            anchor: None,
            embed_type: Some(embed_type),
            position,
            style: Style::MdLink,
            target,
        })
    }

    /// Creates a new markdown-style link.
    ///
    /// # Errors
    /// Returns `NoteError::Link` if validation fails.
    #[inline]
    pub fn new_markdown_link(
        target: Target,
        alias: Option<&str>,
        anchor: Option<Anchor>,
        position: SourceByteOffset,
    ) -> Result<Self, NoteError> {
        Self::validate_target(&target)?;
        Self::validate_external_anchor(&target, anchor.as_ref())?;
        Ok(Self {
            alias: alias.map(Into::into),
            anchor,
            embed_type: None,
            position,
            style: Style::MdLink,
            target,
        })
    }

    /// Creates a new wiki-link.
    ///
    /// # Errors
    /// Returns `NoteError::Link` if validation fails.
    #[inline]
    pub fn new_wikilink(
        target: Target,
        alias: Option<&str>,
        anchor: Option<Anchor>,
        position: SourceByteOffset,
    ) -> Result<Self, NoteError> {
        Self::validate_target(&target)?;
        Self::validate_external_anchor(&target, anchor.as_ref())?;
        Ok(Self {
            alias: alias.map(Into::into),
            anchor,
            embed_type: None,
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
    /// Returns `NoteError::Link` if validation fails.
    #[inline]
    pub fn validate(&self) -> Result<(), NoteError> {
        Self::validate_external_anchor(&self.target, self.anchor.as_ref())?;
        Ok(())
    }

    fn validate_external_anchor(
        target: &Target,
        anchor: Option<&Anchor>,
    ) -> Result<(), NoteError> {
        if target.is_external() && anchor.is_some() {
            return Err(NoteError::Link(
                "External links cannot have anchors".into(),
            ));
        }
        Ok(())
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on &Target with non-Copy fields uses match \
                  ergonomics."
    )]
    fn validate_target(target: &Target) -> Result<(), NoteError> {
        let is_empty = match target {
            Target::External {
                url,
            } => url.is_empty(),
            Target::Resolved {
                path,
                ..
            } => path.as_str().is_empty(),
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
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matching on &Target with non-Copy fields uses match \
                  ergonomics."
    )]
    #[inline]
    #[must_use]
    pub fn vault_path(&self) -> Option<&str> {
        match self {
            Self::External {
                ..
            } => None,
            Self::Resolved {
                path,
                ..
            } => Some(path.as_str()),
            Self::Unresolved {
                raw,
            } => Some(raw.as_ref()),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Tests use assertions in Result-returning functions."
)]
mod tests {
    mod fixtures {
        use super::super::{Anchor, EmbedType, Link, Target};
        use crate::note::{error::NoteError, types::SourceByteOffset};

        pub fn unresolved_target(name: &str) -> Target {
            Target::Unresolved {
                raw: name.into(),
            }
        }

        pub fn wikilink_with_anchor_and_alias() -> Result<Link, NoteError> {
            Link::new_wikilink(
                unresolved_target("Target Note"),
                Some("Display Text"),
                Some(Anchor::Heading("section".into())),
                SourceByteOffset::new(100u32),
            )
        }

        pub fn embed_image() -> Result<Link, NoteError> {
            Link::new_embed(
                unresolved_target("diagram.png"),
                EmbedType::Image,
                None,
                None,
                SourceByteOffset::new(200u32),
            )
        }
    }

    mod constructors {
        use super::{
            super::{Link, Style},
            fixtures::{
                embed_image, unresolved_target, wikilink_with_anchor_and_alias,
            },
        };
        use crate::note::error::NoteError;

        #[test]
        fn wikilink_style_is_wikilink() -> Result<(), NoteError> {
            let link = wikilink_with_anchor_and_alias()?;
            assert_eq!(link.style(), Style::WikiLink);
            Ok(())
        }

        #[test]
        fn wikilink_target_is_unresolved() -> Result<(), NoteError> {
            let link = wikilink_with_anchor_and_alias()?;
            assert!(link.target().is_unresolved());
            Ok(())
        }

        #[test]
        fn embed_reports_is_embed() -> Result<(), NoteError> {
            let embed = embed_image()?;
            assert!(embed.is_embed());
            Ok(())
        }

        #[test]
        fn new_wikilink_rejects_empty_target() {
            let target = unresolved_target("");
            let result = Link::new_wikilink(
                target,
                None,
                None,
                crate::note::types::SourceByteOffset::new(0u32),
            );
            result.unwrap_err();
        }

        #[test]
        fn markdown_link_rejects_external_anchor() {
            let target = super::super::Target::External {
                url: "https://example.com#frag".into(),
            };
            let result = Link::new_markdown_link(
                target,
                None,
                Some(super::super::Anchor::Heading("frag".into())),
                crate::note::types::SourceByteOffset::new(0u32),
            );
            result.unwrap_err();
        }
    }
}
