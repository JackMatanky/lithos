//! Raw link types.

use crate::note::position::SourceByteOffset;

/// Raw link style before validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawLinkStyle {
    Wiki,
    Markdown,
}

/// Raw link extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawLink {
    style: RawLinkStyle,
    is_embed: bool,
    target: Box<str>,
    alias: Option<Box<str>>,
    anchor: Option<Box<str>>,
    position: SourceByteOffset,
}

impl RawLink {
    /// Create a new raw link.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Raw links store full source context"
    )]
    pub fn new(
        style: RawLinkStyle,
        is_embed: bool,
        target: Box<str>,
        alias: Option<Box<str>>,
        anchor: Option<Box<str>>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            style,
            is_embed,
            target,
            alias,
            anchor,
            position,
        }
    }

    /// Return the raw link style.
    #[inline]
    #[must_use]
    pub const fn style(&self) -> RawLinkStyle {
        self.style
    }

    /// Return true if this link is an embed.
    #[inline]
    #[must_use]
    pub const fn is_embed(&self) -> bool {
        self.is_embed
    }

    /// Return the raw target string.
    #[inline]
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Return the alias text, if present.
    #[inline]
    #[must_use]
    pub fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    /// Return the raw anchor text, if present.
    #[inline]
    #[must_use]
    pub fn anchor(&self) -> Option<&str> {
        self.anchor.as_deref()
    }

    /// Return the source byte position.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }
}
