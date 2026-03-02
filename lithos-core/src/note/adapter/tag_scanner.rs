//! Tag scanning helpers for markdown ingestion.

use crate::note::tag::Tag;

/// Scans raw text for Obsidian-style tags.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TagScanner<'text> {
    text: &'text str,
}

impl<'text> TagScanner<'text> {
    #[inline]
    pub(crate) const fn new(text: &'text str) -> Self {
        Self {
            text,
        }
    }

    #[inline]
    pub(crate) fn collect_tags(self) -> Vec<Tag> {
        let mut tags = Vec::new();
        let mut chars = self.text.chars().peekable();
        let mut prev_is_alnum = false;

        while let Some(ch) = chars.next() {
            if ch != '#' || prev_is_alnum {
                prev_is_alnum = ch.is_alphanumeric();
                continue;
            }

            let mut raw = String::with_capacity(16);
            raw.push('#');
            while let Some(&next) = chars.peek() {
                if !(next.is_alphanumeric() || matches!(next, '_' | '-' | '/'))
                {
                    break;
                }
                raw.push(next);
                chars.next();
            }

            if raw.len() > 1
                && let Ok(tag) = Tag::from_token(&raw)
            {
                tags.push(tag);
            }

            prev_is_alnum =
                raw.chars().last().is_some_and(char::is_alphanumeric);
        }

        tags
    }
}
