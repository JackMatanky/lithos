//! Inline field value objects and domain conversions.

#![expect(
    clippy::pattern_type_mismatch,
    reason = "Pattern matching style is clear in context"
)]

use super::{raw::RawInlineField, value::FieldValue};
use crate::note::position::{SourceByteOffset, SourceByteRange};

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
        let mut normalized = String::with_capacity(key.len());
        let mut needs_dash = false;

        for ch in key.chars() {
            if matches!(ch, '*' | '_' | '~' | '`') {
                continue;
            }
            if ch.is_whitespace() {
                if !normalized.is_empty() {
                    needs_dash = true;
                }
                continue;
            }
            if needs_dash {
                normalized.push('-');
                needs_dash = false;
            }
            normalized.push(ch.to_ascii_lowercase());
        }

        if normalized.ends_with('-') {
            while normalized.ends_with('-') {
                normalized.pop();
            }
        }

        normalized.into_boxed_str()
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
    range: SourceByteRange,
}

impl InlineField {
    /// Create a new inline field entry.
    #[inline]
    #[must_use]
    pub fn new(
        key: InlineFieldKey,
        value: FieldValue,
        range: SourceByteRange,
    ) -> Self {
        Self {
            key,
            value,
            range,
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
        self.range.start()
    }

    /// Return the source byte range of the field.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }

    /// Convert from a raw inline field.
    #[inline]
    #[must_use]
    pub fn from_raw(raw: &RawInlineField<'_>) -> Self {
        use crate::note::raw::RawFieldValue;

        let value = match &raw.value {
            RawFieldValue::String(s) => FieldValue::String(s.as_ref().into()),
            RawFieldValue::Number(n) => FieldValue::Number(*n),
            RawFieldValue::Date(d) => FieldValue::Date((*d).into()),
            RawFieldValue::DateTime(dt) => FieldValue::DateTime((*dt).into()),
            RawFieldValue::Time(t) => FieldValue::Time((*t).into()),
            RawFieldValue::Boolean(b) => FieldValue::Boolean(*b),
            RawFieldValue::Array(values) => FieldValue::Array(
                values
                    .iter()
                    .cloned()
                    .map(FieldValue::from)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            RawFieldValue::Object(values) => FieldValue::Object(Box::new(
                values
                    .iter()
                    .map(|(key, value)| {
                        (key.clone(), FieldValue::from(value.clone()))
                    })
                    .collect(),
            )),
            RawFieldValue::Null => FieldValue::Null,
        };

        InlineField::new(raw.key.as_ref().into(), value, raw.range)
    }
}
