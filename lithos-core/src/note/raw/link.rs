use std::borrow::Cow;

use pulldown_cmark::LinkType;

use crate::note::position::SourceByteOffset;

/// Raw link extracted from markdown.
///
/// Holds the target exactly as it appears in the source — no anchor is split
/// out at this layer. Call [`RawLinkText::split_target`] during conversion to
/// the domain type when the `(path, anchor)` decomposition is needed.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawLink<'source> {
    pub style: RawLinkStyle,
    pub is_embed: bool,
    pub text: RawLinkText<'source>,
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
            text: RawLinkText::new(target),
            position,
        }
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
            text: self.text.into_owned(),
            position: self.position,
        }
    }
}

/// Raw link style before validation.
///
/// For wiki-links, `has_alias` reflects whether a `|` separator was present
/// in the source (`[[target|alias]]` vs `[[target]]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawLinkStyle {
    /// Wiki-link style with explicit alias flag.
    Wiki {
        has_alias: bool,
    },
    /// Standard markdown link or image style.
    Markdown,
}

/// Converts a pulldown-cmark [`LinkType`] to a [`RawLinkStyle`].
impl From<LinkType> for RawLinkStyle {
    #[inline]
    fn from(kind: LinkType) -> Self {
        match kind {
            LinkType::WikiLink {
                has_pothole,
            } => Self::Wiki {
                has_alias: has_pothole,
            },
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

/// The full textual content of a raw link.
///
/// Holds the unsplit target string (as given by the parser) alongside the
/// display text accumulated from events. Provides methods to extract the
/// anchor fragment and alias during conversion to the domain type rather than
/// at parse time.
#[derive(Debug, Clone, PartialEq)]
pub struct RawLinkText<'source> {
    /// Raw target as it appears in the source, including any `#anchor`
    /// fragment.
    pub target: Cow<'source, str>,
    /// Accumulated display text from parser events.
    pub display: String,
}

impl<'source> RawLinkText<'source> {
    pub(crate) const fn new(target: Cow<'source, str>) -> Self {
        Self {
            target,
            display: String::new(),
        }
    }

    /// Returns the alias for this link given its style, or `None` if no
    /// explicit alias is present.
    ///
    /// For wiki-links, `has_alias = true` means the display text is a real
    /// alias; `has_alias = false` means it mirrors the target and should be
    /// ignored. For markdown links, display text is always treated as the
    /// alias.
    pub fn alias(&self, style: RawLinkStyle) -> Option<String> {
        let has_alias = match style {
            RawLinkStyle::Wiki {
                has_alias,
            } => has_alias,
            RawLinkStyle::Markdown => true,
        };
        if has_alias {
            let trimmed = self.display.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_owned())
            }
        } else {
            None
        }
    }

    /// Splits the target at the first `#` for internal links.
    ///
    /// Pass `is_external = true` to leave the target unchanged and return
    /// `None` for the anchor, preserving any `#` fragment in the URL.
    pub(crate) fn split_target(
        self,
        is_external: bool,
    ) -> (Cow<'source, str>, Option<Cow<'source, str>>) {
        if is_external {
            return (self.target, None);
        }
        match self.target {
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

impl RawLinkText<'_> {
    /// Converts this text into an owned variant.
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawLinkText<'static> {
        RawLinkText {
            target: Cow::Owned(self.target.into_owned()),
            display: self.display,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_target_with_anchor() {
        let text = RawLinkText::new(Cow::Owned(String::from("note#heading")));
        let (path, anchor) = text.split_target(false);

        assert_eq!(path.as_ref(), "note");
        assert_eq!(
            anchor.as_ref().map(std::convert::AsRef::as_ref),
            Some("heading")
        );
    }

    #[test]
    fn split_target_without_anchor() {
        let text = RawLinkText::new(Cow::Owned(String::from("note")));
        let (path, anchor) = text.split_target(false);

        assert_eq!(path.as_ref(), "note");
        assert!(anchor.is_none());
    }

    #[test]
    fn split_target_multiple_hashes() {
        let text = RawLinkText::new(Cow::Owned(String::from("note#a#b#c")));
        let (path, anchor) = text.split_target(false);

        assert_eq!(path.as_ref(), "note");
        assert_eq!(
            anchor.as_ref().map(std::convert::AsRef::as_ref),
            Some("a#b#c")
        );
    }

    #[test]
    fn split_target_external_preserves_fragment() {
        let text = RawLinkText::new(Cow::Owned(String::from(
            "https://example.com/page#section",
        )));
        let (path, anchor) = text.split_target(true);

        assert_eq!(path.as_ref(), "https://example.com/page#section");
        assert!(anchor.is_none());
    }

    #[test]
    fn alias_wiki_with_pothole() {
        let text = RawLinkText {
            target: Cow::Borrowed("note"),
            display: String::from("  My Alias  "),
        };
        let style = RawLinkStyle::Wiki {
            has_alias: true,
        };
        assert_eq!(text.alias(style).as_deref(), Some("My Alias"));
    }

    #[test]
    fn alias_wiki_without_pothole() {
        let text = RawLinkText {
            target: Cow::Borrowed("note"),
            display: String::from("note"),
        };
        let style = RawLinkStyle::Wiki {
            has_alias: false,
        };
        assert!(text.alias(style).is_none());
    }

    #[test]
    fn alias_markdown_uses_display() {
        let text = RawLinkText {
            target: Cow::Borrowed("https://example.com"),
            display: String::from("Example"),
        };
        assert_eq!(
            text.alias(RawLinkStyle::Markdown).as_deref(),
            Some("Example")
        );
    }

    #[test]
    fn alias_blank_display_returns_none() {
        let text = RawLinkText {
            target: Cow::Borrowed("note"),
            display: String::from("   "),
        };
        let style = RawLinkStyle::Wiki {
            has_alias: true,
        };
        assert!(text.alias(style).is_none());
    }
}
