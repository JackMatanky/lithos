//! Note-specific domain types and newtypes.
#![expect(
    clippy::exhaustive_structs,
    reason = "rkyv generates exhaustive ArchivedSourceByteOffset despite \
              #[non_exhaustive]"
)]

use rkyv::{
    Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize,
};
use serde::{Deserialize, Serialize};

/// A byte offset into a source document.
///
/// Represented as a `u32` to optimize storage (supporting files up to 4GB).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Serialize,
    Deserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct SourceByteOffset {
    /// The raw byte offset.
    pub value: u32,
}

impl From<u32> for SourceByteOffset {
    #[inline]
    fn from(value: u32) -> Self {
        Self {
            value,
        }
    }
}

impl From<SourceByteOffset> for u32 {
    #[inline]
    fn from(offset: SourceByteOffset) -> Self {
        offset.value
    }
}

impl From<SourceByteOffset> for usize {
    #[inline]
    #[expect(
        clippy::expect_used,
        reason = "u32 always fits in usize on all supported platforms"
    )]
    #[expect(
        clippy::disallowed_methods,
        reason = "u32 always fits in usize on all supported platforms"
    )]
    fn from(offset: SourceByteOffset) -> Self {
        usize::try_from(offset.value).expect("u32 should fit in usize")
    }
}

impl TryFrom<usize> for SourceByteOffset {
    type Error = std::num::TryFromIntError;

    #[inline]
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        u32::try_from(value).map(|value| Self {
            value,
        })
    }
}

impl SourceByteOffset {
    /// Creates a new `SourceByteOffset`.
    #[inline]
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self {
            value,
        }
    }

    /// Returns the raw `u32` value.
    #[inline]
    #[must_use]
    pub const fn value(self) -> u32 {
        self.value
    }
}
