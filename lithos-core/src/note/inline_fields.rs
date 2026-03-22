//! Inline field value objects and domain conversions.

use super::{raw::RawInlineField, value::FieldValue};
use crate::note::position::SourceByteOffset;

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

    /// Convert from a raw inline field.
    #[inline]
    #[must_use]
    pub fn from_raw(raw: &RawInlineField<'_>) -> Self {
        InlineField::new(
            raw.key.as_ref().into(),
            FieldValue::String(raw.value.as_ref().into()),
            raw.position,
        )
    }
}
