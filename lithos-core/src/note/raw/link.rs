use std::borrow::Cow;

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
pub struct RawLink<'source> {
    pub style: RawLinkStyle,
    pub is_embed: bool,
    pub target: Cow<'source, str>,
    pub alias: Option<Cow<'source, str>>,
    pub anchor: Option<Cow<'source, str>>,
    pub position: SourceByteOffset,
}

impl<'source> RawLink<'source> {
    /// Create a new raw link.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Raw links store full source context"
    )]
    pub const fn new(
        style: RawLinkStyle,
        is_embed: bool,
        target: Cow<'source, str>,
        alias: Option<Cow<'source, str>>,
        anchor: Option<Cow<'source, str>>,
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
}

impl RawLink<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawLink<'static> {
        RawLink {
            style: self.style,
            is_embed: self.is_embed,
            target: Cow::Owned(self.target.into_owned()),
            alias: self.alias.map(|alias| Cow::Owned(alias.into_owned())),
            anchor: self.anchor.map(|anchor| Cow::Owned(anchor.into_owned())),
            position: self.position,
        }
    }
}
