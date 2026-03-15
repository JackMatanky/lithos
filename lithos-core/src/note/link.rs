//! Link and embed entities for document relationships.
//!
//! Models wiki-links, markdown links, and embedded content references
//! with support for anchors and aliases.
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]
#![expect(dead_code, reason = "Legacy link builders retained for future use")]

use super::{
    aggregate::NoteId,
    error::{LinkError, NoteError},
    heading::HeadingText,
    paths::NotePath,
    position::SourceByteOffset,
    structure::BlockRefId,
};

/// Represents a link within a note.
///
/// Supports multiple syntactic styles (Wiki-links vs Markdown links) and
/// can represent both internal vault references and external URLs.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::{link::{Link, Target}, position::SourceByteOffset};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let target = Target::Unresolved { raw: "Main Page".into() };
/// let link = Link::try_new_wikilink(
///     target,
///     None,
///     None,
///     SourceByteOffset::new(0),
/// )?;
///
/// assert!(!link.is_embed());
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Link {
    target: Target,
    anchor: Option<Anchor>,
    position: SourceByteOffset,
    alias: Option<LinkAlias>,
    style: Style,
    embed_type: Option<EmbedType>,
}

/// Link discovered in frontmatter values.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct FrontmatterLink {
    key: Box<str>,
    target: Target,
    anchor: Option<Anchor>,
    alias: Option<Box<str>>,
    embed_type: Option<EmbedType>,
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
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
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

/// Sub-note anchor (heading or block reference).
///
/// An anchor represents a specific location within a note, allowing links
/// to point to more than just the file level.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::link::Anchor;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let heading = Anchor::heading("Introduction")?;
/// let block = Anchor::block_ref("abc123")?;
///
/// assert!(heading.is_heading());
/// assert_eq!(block.text(), "abc123");
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum Anchor {
    /// Block reference: `^block-id`.
    BlockRef(BlockRefId),
    /// Heading anchor: `#heading-text`.
    Heading(HeadingText),
}

/// Represents the syntactic style of a link.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
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

/// Whether a link is an embed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EmbedState {
    /// Standard link.
    Link,
    /// Embedded content.
    Embed,
}

/// Controls alias text collection behavior during parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AliasMode {
    /// Ignore alias text.
    Ignore,
    /// Collect alias text.
    Collect,
}

/// Builder for accumulating link data during parsing.
#[derive(Debug)]
pub(crate) struct LinkBuilder {
    target: Box<str>,
    alias: Option<String>,
    position: SourceByteOffset,
    style: Style,
    embed: EmbedState,
    alias_mode: AliasMode,
}

impl LinkBuilder {
    #[inline]
    pub(crate) fn new(
        target: &str,
        position: SourceByteOffset,
        style: Style,
        embed: EmbedState,
        alias_mode: AliasMode,
    ) -> Self {
        Self {
            target: target.into(),
            alias: None,
            position,
            style,
            embed,
            alias_mode,
        }
    }

    #[inline]
    pub(crate) fn add_alias_text(&mut self, text: &str) {
        if self.alias_mode == AliasMode::Collect {
            self.alias.get_or_insert_with(String::new).push_str(text);
        }
    }

    pub(crate) fn build(self) -> Result<Link, NoteError> {
        let raw_target = self.target.as_ref();
        let is_external = is_external_target(raw_target);

        let (target_path, anchor) = if is_external {
            (raw_target, None)
        } else {
            split_target_and_anchor(raw_target)?
        };

        let target = if is_external {
            Target::External {
                url: target_path.into(),
            }
        } else {
            Target::Unresolved {
                raw: target_path.into(),
            }
        };

        match (self.embed, self.style) {
            (EmbedState::Embed, Style::WikiLink) => {
                let embed_type = EmbedType::from_extension(target_path);
                Link::try_new_embed(
                    target,
                    embed_type,
                    self.alias.as_deref(),
                    anchor,
                    self.position,
                )
            }
            (EmbedState::Embed, Style::MdLink) => {
                let embed_type = EmbedType::from_extension(target_path);
                Link::try_new_markdown_embed(
                    target,
                    embed_type,
                    self.alias.as_deref(),
                    self.position,
                )
            }
            (EmbedState::Link, Style::WikiLink) => Link::try_new_wikilink(
                target,
                self.alias.as_deref(),
                anchor,
                self.position,
            ),
            (EmbedState::Link, Style::MdLink) => Link::try_new_markdown_link(
                target,
                self.alias.as_deref(),
                anchor,
                self.position,
            ),
        }
    }
}

impl FrontmatterLink {
    /// Creates a frontmatter link entry.
    #[inline]
    #[must_use]
    pub fn new(
        key: Box<str>,
        target: Target,
        anchor: Option<Anchor>,
        alias: Option<Box<str>>,
        embed_type: Option<EmbedType>,
    ) -> Self {
        Self {
            key,
            target,
            anchor,
            alias,
            embed_type,
        }
    }

    /// Returns the frontmatter key that contained this link.
    #[inline]
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the target of the link.
    #[inline]
    #[must_use]
    pub const fn target(&self) -> &Target {
        &self.target
    }

    /// Returns the optional anchor.
    #[inline]
    #[must_use]
    pub fn anchor(&self) -> Option<&Anchor> {
        self.anchor.as_ref()
    }

    /// Returns the optional alias text.
    #[inline]
    #[must_use]
    pub fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    /// Returns the optional embed type.
    #[inline]
    #[must_use]
    pub const fn embed_type(&self) -> Option<EmbedType> {
        self.embed_type
    }

    /// Returns true if this frontmatter link is an embed.
    #[inline]
    #[must_use]
    pub const fn is_embed(&self) -> bool {
        self.embed_type.is_some()
    }
}

/// Parse a frontmatter value into a frontmatter link, if it matches wiki
/// syntax.
pub(crate) fn parse_frontmatter_link(
    key: &str,
    value: &str,
) -> Result<Option<FrontmatterLink>, NoteError> {
    let trimmed = value.trim();
    let (embed, inner) = if let Some(rest) =
        trimmed.strip_prefix("![[").and_then(|rest| rest.strip_suffix("]]"))
    {
        (true, rest)
    } else if let Some(rest) =
        trimmed.strip_prefix("[[").and_then(|rest| rest.strip_suffix("]]"))
    {
        (false, rest)
    } else {
        return Ok(None);
    };

    let (target_text, alias) =
        if let Some((left, right)) = inner.split_once('|') {
            (left.trim(), Some(right.trim()))
        } else {
            (inner.trim(), None)
        };

    if target_text.is_empty() {
        return Ok(None);
    }

    let (target_path, anchor) = split_target_and_anchor(target_text)?;
    let target = if is_external_target(target_path) {
        Target::External {
            url: target_path.into(),
        }
    } else {
        Target::Unresolved {
            raw: target_path.into(),
        }
    };
    let embed_type = embed.then(|| EmbedType::from_extension(target_path));

    Ok(Some(FrontmatterLink::new(
        key.into(),
        target,
        anchor,
        alias.filter(|text| !text.is_empty()).map(Into::into),
        embed_type,
    )))
}

impl EmbedType {
    /// Determine embed type from file extension.
    ///
    /// Uses case-insensitive matching without allocation.
    /// Defaults to [`EmbedType::Note`] for unknown extensions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::link::EmbedType;
    /// assert_eq!(EmbedType::from_extension("image.png"), EmbedType::Image);
    /// assert_eq!(EmbedType::from_extension("video.mp4"), EmbedType::Video);
    /// assert_eq!(EmbedType::from_extension("audio.mp3"), EmbedType::Audio);
    /// assert_eq!(EmbedType::from_extension("doc.pdf"), EmbedType::Pdf);
    /// assert_eq!(EmbedType::from_extension("note.md"), EmbedType::Note);
    /// ```
    #[inline]
    #[must_use]
    pub fn from_extension(path: &str) -> Self {
        let Some((_, ext)) = path.rsplit_once('.') else {
            return Self::Note;
        };

        if matches_any_ignore_case(ext, &[
            "png", "jpg", "jpeg", "gif", "svg", "webp",
        ]) {
            return Self::Image;
        }
        if matches_any_ignore_case(ext, &["mp4", "webm", "ogv", "mov"]) {
            return Self::Video;
        }
        if matches_any_ignore_case(ext, &["mp3", "wav", "ogg", "m4a"]) {
            return Self::Audio;
        }
        if ext.eq_ignore_ascii_case("pdf") {
            return Self::Pdf;
        }
        Self::Note
    }
}

/// Helper to check if a string matches any candidate (case-insensitive).
#[inline]
fn matches_any_ignore_case(s: &str, candidates: &[&str]) -> bool {
    candidates.iter().any(|c| s.eq_ignore_ascii_case(c))
}

/// Validated link alias text.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct LinkAlias(Box<str>);

impl LinkAlias {
    /// Creates a validated link alias.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Link`] if the alias is empty.
    #[inline]
    pub fn try_new(value: &str) -> Result<Self, NoteError> {
        let text = value.trim();
        if text.is_empty() {
            return Err(NoteError::Link(LinkError::EmptyAlias));
        }
        Ok(Self(text.into()))
    }

    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Anchor {
    /// Creates a validated block reference anchor.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Link`] if the block reference is empty.
    #[inline]
    pub fn block_ref(value: &str) -> Result<Self, NoteError> {
        Ok(Self::BlockRef(BlockRefId::try_new(value)?))
    }

    /// Creates a validated heading anchor.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Link`] if the heading text is empty.
    #[inline]
    pub fn heading(value: &str) -> Result<Self, NoteError> {
        Ok(Self::Heading(HeadingText::try_new_anchor(value)?))
    }

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
        reason = "Match ergonomics on &self"
    )]
    pub fn text(&self) -> &str {
        match self {
            Self::BlockRef(s) => s.as_str(),
            Self::Heading(s) => s.as_str(),
        }
    }
}

/// Split an internal link target into path and optional anchor.
///
/// # Errors
///
/// Returns [`NoteError::Link`] if the anchor is malformed.
pub(crate) fn split_target_and_anchor(
    target: &str,
) -> Result<(&str, Option<Anchor>), NoteError> {
    let Some((path, anchor_text)) = target.split_once('#') else {
        return Ok((target, None));
    };

    if let Some(block_ref) = anchor_text.strip_prefix('^') {
        Ok((path, Some(Anchor::block_ref(block_ref)?)))
    } else {
        Ok((path, Some(Anchor::heading(anchor_text)?)))
    }
}

pub(crate) fn is_external_target(target: &str) -> bool {
    target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("ftp://")
        || target.starts_with("mailto:")
}

impl Link {
    /// Returns the optional display alias.
    #[inline]
    #[must_use]
    pub fn alias(&self) -> Option<&str> {
        self.alias.as_ref().map(LinkAlias::as_str)
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
    pub fn try_new_embed(
        target: Target,
        embed_type: EmbedType,
        alias: Option<&str>,
        anchor: Option<Anchor>,
        position: SourceByteOffset,
    ) -> Result<Self, NoteError> {
        Self::validate_target(&target)?;
        Self::validate_external_anchor(&target, anchor.as_ref())?;
        let alias = Self::parse_alias(alias)?;
        Ok(Self {
            alias,
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
    pub fn try_new_markdown_embed(
        target: Target,
        embed_type: EmbedType,
        alias: Option<&str>,
        position: SourceByteOffset,
    ) -> Result<Self, NoteError> {
        Self::validate_target(&target)?;
        Self::validate_external_anchor(&target, None)?;
        let alias = Self::parse_alias(alias)?;
        Ok(Self {
            alias,
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
    pub fn try_new_markdown_link(
        target: Target,
        alias: Option<&str>,
        anchor: Option<Anchor>,
        position: SourceByteOffset,
    ) -> Result<Self, NoteError> {
        Self::validate_target(&target)?;
        Self::validate_external_anchor(&target, anchor.as_ref())?;
        let alias = Self::parse_alias(alias)?;
        Ok(Self {
            alias,
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
    pub fn try_new_wikilink(
        target: Target,
        alias: Option<&str>,
        anchor: Option<Anchor>,
        position: SourceByteOffset,
    ) -> Result<Self, NoteError> {
        Self::validate_target(&target)?;
        Self::validate_external_anchor(&target, anchor.as_ref())?;
        let alias = Self::parse_alias(alias)?;
        Ok(Self {
            alias,
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
            return Err(NoteError::Link(LinkError::ExternalAnchor));
        }
        Ok(())
    }

    fn parse_alias(
        alias: Option<&str>,
    ) -> Result<Option<LinkAlias>, NoteError> {
        alias.map(LinkAlias::try_new).transpose()
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &Target"
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
            return Err(NoteError::Link(LinkError::EmptyTarget));
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
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &Target"
    )]
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
        use crate::note::{error::NoteError, position::SourceByteOffset};

        pub fn unresolved_target(name: &str) -> Target {
            Target::Unresolved {
                raw: name.into(),
            }
        }

        pub fn wikilink_with_anchor_and_alias() -> Result<Link, NoteError> {
            Link::try_new_wikilink(
                unresolved_target("Target Note"),
                Some("Display Text"),
                Some(Anchor::heading("section")?),
                SourceByteOffset::new(100u32),
            )
        }

        pub fn embed_image() -> Result<Link, NoteError> {
            Link::try_new_embed(
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
            let result = Link::try_new_wikilink(
                target,
                None,
                None,
                crate::note::position::SourceByteOffset::new(0u32),
            );
            result.unwrap_err();
        }

        #[test]
        fn markdown_link_rejects_external_anchor() -> Result<(), NoteError> {
            let target = super::super::Target::External {
                url: "https://example.com#frag".into(),
            };
            let result = Link::try_new_markdown_link(
                target,
                None,
                Some(super::super::Anchor::heading("frag")?),
                crate::note::position::SourceByteOffset::new(0u32),
            );
            result.unwrap_err();
            Ok(())
        }
    }

    mod embed_type_detection {
        use super::super::EmbedType;

        #[test]
        fn detects_image_extensions() {
            assert_eq!(
                EmbedType::from_extension("image.png"),
                EmbedType::Image
            );
            assert_eq!(
                EmbedType::from_extension("photo.jpg"),
                EmbedType::Image
            );
            assert_eq!(EmbedType::from_extension("pic.jpeg"), EmbedType::Image);
            assert_eq!(EmbedType::from_extension("icon.gif"), EmbedType::Image);
            assert_eq!(EmbedType::from_extension("logo.svg"), EmbedType::Image);
            assert_eq!(
                EmbedType::from_extension("hero.webp"),
                EmbedType::Image
            );
        }

        #[test]
        fn detects_video_extensions() {
            assert_eq!(
                EmbedType::from_extension("video.mp4"),
                EmbedType::Video
            );
            assert_eq!(
                EmbedType::from_extension("clip.webm"),
                EmbedType::Video
            );
            assert_eq!(
                EmbedType::from_extension("movie.ogv"),
                EmbedType::Video
            );
            assert_eq!(
                EmbedType::from_extension("recording.mov"),
                EmbedType::Video
            );
        }

        #[test]
        fn detects_audio_extensions() {
            assert_eq!(EmbedType::from_extension("song.mp3"), EmbedType::Audio);
            assert_eq!(
                EmbedType::from_extension("sound.wav"),
                EmbedType::Audio
            );
            assert_eq!(
                EmbedType::from_extension("audio.ogg"),
                EmbedType::Audio
            );
            assert_eq!(
                EmbedType::from_extension("track.m4a"),
                EmbedType::Audio
            );
        }

        #[test]
        fn detects_pdf_extension() {
            assert_eq!(EmbedType::from_extension("doc.pdf"), EmbedType::Pdf);
            assert_eq!(EmbedType::from_extension("Doc.PDF"), EmbedType::Pdf);
        }

        #[test]
        fn case_insensitive_matching() {
            assert_eq!(
                EmbedType::from_extension("IMAGE.PNG"),
                EmbedType::Image
            );
            assert_eq!(
                EmbedType::from_extension("Video.MP4"),
                EmbedType::Video
            );
            assert_eq!(
                EmbedType::from_extension("Audio.MP3"),
                EmbedType::Audio
            );
        }

        #[test]
        fn no_extension_defaults_to_note() {
            assert_eq!(EmbedType::from_extension("filename"), EmbedType::Note);
        }

        #[test]
        fn unknown_extension_defaults_to_note() {
            assert_eq!(EmbedType::from_extension("file.txt"), EmbedType::Note);
            assert_eq!(EmbedType::from_extension("doc.docx"), EmbedType::Note);
        }
    }
}
