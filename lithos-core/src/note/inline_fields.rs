//! Inline field value objects and parsing.

use super::{raw::RawInlineField, value::FieldValue};
use crate::note::position::SourceByteOffset;

/// The syntax style used for an inline field.
///
/// This enum acts as the single source of truth for detecting and classifying
/// different inline field syntaxes in markdown. It is not persisted in the
/// domain model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InlineFieldDelimiter {
    /// `[key:: value]`.
    Brackets,
    /// `(key:: value)`.
    Parentheses,
    /// `key:: value`.
    Bare,
    /// `📅 2024-03-18` (Emoji-prefixed).
    Emoji,
}

impl InlineFieldDelimiter {
    /// Returns the character pair for delimited fields.
    #[inline]
    #[must_use]
    pub const fn pair(&self) -> Option<(u8, u8)> {
        match *self {
            Self::Brackets => Some((b'[', b']')),
            Self::Parentheses => Some((b'(', b')')),
            Self::Bare | Self::Emoji => None,
        }
    }
}

/// A normalized identifier for an inline field key.
///
/// This type preserves the original form of the key as it appeared in the
/// source while providing a canonical `kebab-case` version for consistent
/// matching and a `snake_case` version for flexible querying.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct InlineFieldKey {
    raw: Box<str>,
    normalized: Box<str>,
}

impl InlineFieldKey {
    /// Creates a new key from a raw string, computing its normalized form.
    #[inline]
    #[must_use]
    pub fn new(raw: &str) -> Self {
        Self {
            normalized: Self::normalize(raw),
            raw: raw.into(),
        }
    }

    /// Returns the original raw form of the key.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Returns the primary `kebab-case` representation used for canonical
    /// matching.
    #[inline]
    #[must_use]
    pub fn as_kebab(&self) -> &str {
        &self.normalized
    }

    /// Returns the `snake_case` representation of the key.
    #[inline]
    #[must_use]
    pub fn to_snake(&self) -> String {
        self.normalized.replace('-', "_")
    }

    /// Normalizes a key by removing markdown decorators and converting to
    /// `kebab-case`.
    #[inline]
    #[must_use]
    pub fn normalize(key: &str) -> Box<str> {
        let stripped = key
            .chars()
            .filter(|ch| !matches!(ch, '*' | '_' | '~' | '`'))
            .collect::<String>();
        stripped
            .split_whitespace()
            .map(str::to_ascii_lowercase)
            .collect::<Vec<_>>()
            .join("-")
            .into_boxed_str()
    }
}

impl From<&str> for InlineFieldKey {
    #[inline]
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<Box<str>> for InlineFieldKey {
    #[inline]
    fn from(s: Box<str>) -> Self {
        Self::new(&s)
    }
}

impl core::fmt::Display for InlineFieldKey {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.raw)
    }
}

/// Inline field extracted from markdown and converted to domain types.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct InlineField {
    key: InlineFieldKey,
    value: FieldValue,
    position: SourceByteOffset,
}

impl InlineField {
    /// Create a new inline field entry.
    #[inline]
    #[must_use]
    pub fn new(
        key: InlineFieldKey,
        value: FieldValue,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            key,
            value,
            position,
        }
    }

    /// Return the normalized field key identifier.
    #[inline]
    #[must_use]
    pub fn key(&self) -> &InlineFieldKey {
        &self.key
    }

    /// Return the field value.
    #[inline]
    #[must_use]
    pub fn value(&self) -> &FieldValue {
        &self.value
    }

    /// Return the source byte position of the field key.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }
}

/// Specialized scanner for extracting inline fields from markdown text.
#[non_exhaustive]
pub struct InlineFieldScanner;

impl InlineFieldScanner {
    /// Scans text for delimited inline fields like `[key:: value]` or `(key::
    /// value)`.
    ///
    /// The callback receives the key, value, and the start and end offsets of
    /// the *entire* delimited block (including brackets).
    #[inline]
    pub fn scan_delimited<F>(
        text: &str,
        delimiter: InlineFieldDelimiter,
        mut f: F,
    ) where
        F: FnMut(&str, &str, usize, usize),
    {
        let Some((open_delim, close_delim)) = delimiter.pair() else {
            return;
        };
        let bytes = text.as_bytes();
        let mut cursor = 0;
        while let Some(open_rel) = bytes
            .get(cursor..)
            .and_then(|slice| slice.iter().position(|&b| b == open_delim))
        {
            let open = cursor.saturating_add(open_rel);
            let after_open = open.saturating_add(1);
            let Some(close_rel) = bytes
                .get(after_open..)
                .and_then(|slice| slice.iter().position(|&b| b == close_delim))
            else {
                break;
            };
            let close = after_open.saturating_add(close_rel);
            let end = close.saturating_add(1);
            let Some(inner) = text.get(after_open..close) else {
                cursor = end;
                continue;
            };
            if let Some((key, value)) = inner.split_once("::") {
                let key_trimmed = key.trim();
                let value_trimmed = value.trim();
                if !key_trimmed.is_empty() && !value_trimmed.is_empty() {
                    let key_start = key
                        .find(key_trimmed)
                        .unwrap_or(0)
                        .saturating_add(after_open);
                    f(key_trimmed, value_trimmed, key_start, end);
                }
            }
            cursor = end;
        }
    }

    /// Scans text for bare inline fields like `key:: value`.
    ///
    /// To avoid overlapping with delimited fields, provide a list of already
    /// captured spans.
    #[inline]
    pub fn scan_bare<F>(text: &str, bracket_spans: &[(usize, usize)], mut f: F)
    where
        F: FnMut(&str, &str, usize),
    {
        let mut offset = 0usize;
        for line in text.split_inclusive(['\n', '\r']) {
            Self::scan_bare_line(line, offset, bracket_spans, &mut f);
            offset = offset.saturating_add(line.len());
        }
    }

    fn scan_bare_line<F>(
        line: &str,
        offset: usize,
        bracket_spans: &[(usize, usize)],
        f: &mut F,
    ) where
        F: FnMut(&str, &str, usize),
    {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        let Some((key, value)) = trimmed.split_once("::") else {
            return;
        };
        let key_trimmed = key.trim();
        let value_trimmed = value.trim();
        if key_trimmed.is_empty() || value_trimmed.is_empty() {
            return;
        }
        let key_start =
            trimmed.find(key_trimmed).unwrap_or(0).saturating_add(offset);
        let is_bracketed = bracket_spans
            .iter()
            .any(|&(start, end)| key_start >= start && key_start < end);
        if !is_bracketed {
            f(key_trimmed, value_trimmed, key_start);
        }
    }

    /// Scans text for emoji-prefixed fields like `📅 2023-10-27`.
    #[inline]
    pub fn scan_emoji<F>(text: &str, emoji_markers: &[char], mut f: F)
    where
        F: FnMut(&str, &str, usize),
    {
        if emoji_markers.is_empty() {
            return;
        }
        for (idx, ch) in text.char_indices() {
            if !emoji_markers.contains(&ch) {
                continue;
            }
            let value_start = idx.saturating_add(ch.len_utf8());
            let Some(tail) = text.get(value_start..) else {
                continue;
            };
            let Some(value) = tail.split_whitespace().next() else {
                continue;
            };
            if value.is_empty() {
                continue;
            }
            let mut buffer = [0u8; 4];
            let encoded = ch.encode_utf8(&mut buffer);
            f(encoded, value, idx);
        }
    }
}

impl From<RawInlineField> for InlineField {
    #[inline]
    fn from(raw: RawInlineField) -> Self {
        InlineField::new(
            raw.key().into(),
            FieldValue::String(raw.value().into()),
            raw.position(),
        )
    }
}
