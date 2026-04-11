use std::borrow::Cow;

use pulldown_cmark::LinkType;

use crate::note::position::SourceByteOffset;

/// Raw link extracted from markdown.
///
/// Holds the target exactly as it appears in the source — no anchor is split
/// out at this layer. Call [`RawLink::split_target`] during conversion to the
/// domain type when the `(path, anchor)` decomposition is needed.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawLink<'source> {
    pub style: RawLinkStyle,
    pub is_embed: bool,
    pub target: Cow<'source, str>,
    pub text: RawLinkText,
    pub position: SourceByteOffset,
}

impl<'source> RawLink<'source> {
    /// Creates a new raw link with empty display text.
    #[inline]
    #[must_use]
    pub const fn new(
        style: RawLinkStyle,
        is_embed: bool,
        target: Cow<'source, str>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            style,
            is_embed,
            target,
            text: RawLinkText::new(),
            position,
        }
    }

    /// Splits the target at the first `#` for internal links, returning the
    /// modified link alongside the extracted anchor fragment.
    ///
    /// Pass `is_external = true` to leave the target unchanged and return
    /// `None` for the anchor, preserving any `#` fragment in the URL.
    #[inline]
    #[must_use]
    pub(crate) fn split_target(
        self,
        is_external: bool,
    ) -> (Self, Option<Cow<'source, str>>) {
        let (target, anchor) =
            RawLinkTarget::new(self.target).split(is_external);
        (
            Self {
                target,
                ..self
            },
            anchor,
        )
    }
}

impl RawLink<'_> {
    /// Converts this raw link into an owned variant.
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawLink<'static> {
        RawLink {
            style: self.style,
            is_embed: self.is_embed,
            target: Cow::Owned(self.target.into_owned()),
            text: self.text,
            position: self.position,
        }
    }
}

/// The display text of a link, with lazy alias derivation.
///
/// `title` holds the raw accumulated text from parser events. `alias` is only
/// populated after [`RawLinkText::split`] is called — it is `Some` when the
/// link syntax included an explicit alias separator (`has_alias = true`), and
/// `None` when the display text just mirrors the target.
#[derive(Debug, Clone, PartialEq)]
pub struct RawLinkText {
    /// Raw accumulated display text.
    pub title: String,
    /// Alias text, present only when an explicit separator was in the source.
    pub alias: Option<String>,
}

impl RawLinkText {
    pub(crate) const fn new() -> Self {
        Self {
            title: String::new(),
            alias: None,
        }
    }

    /// Derives the alias from the accumulated title.
    ///
    /// If `has_alias` is `true`, `alias` is set to the trimmed `title` (or
    /// `None` if the title is blank after trimming). If `has_alias` is `false`,
    /// `alias` remains `None` — the display text mirrors the target and is not
    /// a real alias.
    #[must_use]
    pub(crate) fn split(mut self, has_alias: bool) -> Self {
        if has_alias {
            let trimmed = self.title.trim();
            self.alias = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_owned())
            };
        }
        self
    }
}

/// Wraps a raw link target string and splits it at the first `#` separator.
///
/// For internal links, the text before `#` is the target path and the text
/// after is the anchor fragment. External links are left unchanged; the
/// caller is responsible for passing the correct `is_external` flag.
pub(crate) struct RawLinkTarget<'source>(Cow<'source, str>);

impl<'source> RawLinkTarget<'source> {
    pub(crate) fn new(target: Cow<'source, str>) -> Self {
        Self(target)
    }

    /// Splits the target into `(path, anchor)`.
    ///
    /// Returns `(self, None)` when `is_external` is `true`. For internal
    /// targets, splits on the first `#`; returns `None` anchor if no `#` is
    /// present.
    pub(crate) fn split(
        self,
        is_external: bool,
    ) -> (Cow<'source, str>, Option<Cow<'source, str>>) {
        if is_external {
            return (self.0, None);
        }
        match self.0 {
            Cow::Borrowed(text) => text
                .split_once('#')
                .map_or((Cow::Borrowed(text), None), |(p, a)| {
                    (Cow::Borrowed(p), Some(Cow::Borrowed(a)))
                }),
            Cow::Owned(mut text) => {
                if let Some(pos) = text.find('#') {
                    #[expect(
                        clippy::arithmetic_side_effects,
                        reason = "pos is from find(), always < text.len()"
                    )]
                    let anchor = text.split_off(pos + 1);
                    text.truncate(pos);
                    (Cow::Owned(text), Some(Cow::Owned(anchor)))
                } else {
                    (Cow::Owned(text), None)
                }
            }
        }
    }
}

/// Raw link style before validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawLinkStyle {
    Wiki,
    Markdown,
}

/// Converts a pulldown-cmark [`LinkType`] to a [`RawLinkStyle`].
impl From<LinkType> for RawLinkStyle {
    #[inline]
    fn from(kind: LinkType) -> Self {
        match kind {
            LinkType::WikiLink {
                ..
            } => Self::Wiki,
            LinkType::Inline
            | LinkType::Reference
            | LinkType::ReferenceUnknown
            | LinkType::Collapsed
            | LinkType::CollapsedUnknown
            | LinkType::Shortcut
            | LinkType::ShortcutUnknown
            | LinkType::Autolink
            | LinkType::Email => Self::Markdown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_with_anchor() {
        let target =
            RawLinkTarget::new(Cow::Owned(String::from("note#heading")));
        let (path, anchor) = target.split(false);

        assert_eq!(path.as_ref(), "note");
        assert_eq!(
            anchor.as_ref().map(std::convert::AsRef::as_ref),
            Some("heading")
        );
    }

    #[test]
    fn split_without_anchor() {
        let target = RawLinkTarget::new(Cow::Owned(String::from("note")));
        let (path, anchor) = target.split(false);

        assert_eq!(path.as_ref(), "note");
        assert!(anchor.is_none());
    }

    #[test]
    fn split_multiple_hashes() {
        let target = RawLinkTarget::new(Cow::Owned(String::from("note#a#b#c")));
        let (path, anchor) = target.split(false);

        assert_eq!(path.as_ref(), "note");
        assert_eq!(
            anchor.as_ref().map(std::convert::AsRef::as_ref),
            Some("a#b#c")
        );
    }

    #[test]
    fn split_external_preserves_fragment() {
        let target = RawLinkTarget::new(Cow::Owned(String::from(
            "https://example.com/page#section",
        )));
        let (path, anchor) = target.split(true);

        assert_eq!(path.as_ref(), "https://example.com/page#section");
        assert!(anchor.is_none());
    }

    #[test]
    fn text_split_with_alias() {
        let text = RawLinkText {
            title: String::from("  My Alias  "),
            alias: None,
        };
        let result = text.split(true);
        assert_eq!(result.alias.as_deref(), Some("My Alias"));
        assert_eq!(result.title, "  My Alias  ");
    }

    #[test]
    fn text_split_without_alias() {
        let text = RawLinkText {
            title: String::from("Target Note"),
            alias: None,
        };
        let result = text.split(false);
        assert!(result.alias.is_none());
    }

    #[test]
    fn text_split_blank_alias_returns_none() {
        let text = RawLinkText {
            title: String::from("   "),
            alias: None,
        };
        let result = text.split(true);
        assert!(result.alias.is_none());
    }
}
