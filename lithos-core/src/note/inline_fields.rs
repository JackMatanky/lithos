//! Inline field value objects and parsing.

use super::{
    raw::{RawInlineField, RawTask},
    value::FieldValue,
};
use crate::note::position::SourceByteOffset;

/// A pair of raw strings representing an extracted inline field before
/// validation.
pub type InlineKeyValuePair = (Box<str>, Box<str>);

/// A normalized identifier for an inline field key.
///
/// This type ensures that keys are stored in a canonical kebab-case format
/// while providing utilities for `snake_case` conversion to support flexible
/// querying.
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
pub struct InlineFieldKey(Box<str>);

impl InlineFieldKey {
    /// Creates a new normalized key from a raw string.
    ///
    /// Strips markdown decorators (`*`, `_`, etc.) and converts
    /// whitespace-separated tokens into kebab-case.
    #[inline]
    #[must_use]
    pub fn new(raw: &str) -> Self {
        Self(InlineFieldScanner::normalize_key(raw))
    }

    /// Returns the primary kebab-case representation.
    #[inline]
    #[must_use]
    pub fn as_kebab(&self) -> &str {
        &self.0
    }

    /// Returns the `snake_case` representation of the key.
    #[inline]
    #[must_use]
    pub fn to_snake(&self) -> String {
        self.0.replace('-', "_")
    }

    /// Returns the raw internal storage.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
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
        f.write_str(&self.0)
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

    /// Return the normalized field key.
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

/// Collection of raw inline field tokens extracted from a text block (e.g., a
/// task).
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct InlineFieldCollection {
    fields: Vec<InlineKeyValuePair>,
}

impl InlineFieldCollection {
    /// Create a new field collection.
    #[inline]
    #[must_use]
    pub fn new(fields: Vec<InlineKeyValuePair>) -> Self {
        Self {
            fields,
        }
    }

    /// Parse inline fields and emoji dates from text.
    #[inline]
    #[must_use]
    pub fn parse(text: &str, emoji_markers: &[char]) -> Self {
        let mut fields = Vec::new();
        InlineFieldScanner::scan_delimited(text, b'[', b']', |k, v, _, _| {
            fields.push((k.into(), v.into()));
        });
        InlineFieldScanner::scan_delimited(text, b'(', b')', |k, v, _, _| {
            fields.push((k.into(), v.into()));
        });

        InlineFieldScanner::scan_emoji(text, emoji_markers, |k, v, _| {
            fields.push((k.into(), v.into()));
        });

        Self::new(fields)
    }

    /// Return parsed inline field tokens.
    #[inline]
    #[must_use]
    pub fn inline_fields(&self) -> &[InlineKeyValuePair] {
        &self.fields
    }
}

/// Specialized scanner for extracting inline fields from markdown text.
#[non_exhaustive]
pub struct InlineFieldScanner;

impl InlineFieldScanner {
    /// Normalizes an inline field key by removing markdown decorators and
    /// converting to kebab-case.
    #[inline]
    #[must_use]
    pub fn normalize_key(key: &str) -> Box<str> {
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

    /// Scans text for delimited inline fields like `[key:: value]` or `(key::
    /// value)`.
    ///
    /// The callback receives the key, value, and the start and end offsets of
    /// the *entire* delimited block (including brackets).
    #[inline]
    pub fn scan_delimited<F>(
        text: &str,
        open_delim: u8,
        close_delim: u8,
        mut f: F,
    ) where
        F: FnMut(&str, &str, usize, usize),
    {
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

impl From<&RawTask> for InlineFieldCollection {
    #[inline]
    fn from(raw: &RawTask) -> Self {
        Self::new(raw.inline_fields().to_vec())
    }
}
