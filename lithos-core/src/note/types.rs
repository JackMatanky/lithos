//! Shared domain types for the Note context.
#![allow(
    clippy::exhaustive_structs,
    reason = "rkyv generates exhaustive archived types"
)]

use rkyv::{Archive, Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a Note.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct NoteId(Uuid);

impl NoteId {
    /// Creates a new random `NoteId` (UUID v7).
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for NoteId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for NoteId {
    #[inline]
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<NoteId> for Uuid {
    #[inline]
    fn from(id: NoteId) -> Self {
        id.0
    }
}

/// A byte offset into the original UTF-8 source of a note.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct SourceByteOffset(u32);

impl SourceByteOffset {
    /// Creates a new `SourceByteOffset`.
    #[inline]
    #[must_use]
    pub const fn new(offset: u32) -> Self {
        Self(offset)
    }
}

impl From<u32> for SourceByteOffset {
    #[inline]
    fn from(offset: u32) -> Self {
        Self(offset)
    }
}

impl From<SourceByteOffset> for u32 {
    #[inline]
    fn from(offset: SourceByteOffset) -> Self {
        offset.0
    }
}

impl TryFrom<usize> for SourceByteOffset {
    type Error = std::num::TryFromIntError;

    #[inline]
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        u32::try_from(value).map(Self)
    }
}

impl From<SourceByteOffset> for usize {
    #[inline]
    fn from(offset: SourceByteOffset) -> Self {
        // Safe because u32 always fits in usize on Lithos supported platforms
        #[expect(
            clippy::as_conversions,
            reason = "Safe conversion from u32 to usize"
        )]
        {
            offset.0 as usize
        }
    }
}

/// A byte range in the original UTF-8 source.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct SourceByteRange {
    /// Start of the range (inclusive).
    pub start: SourceByteOffset,
    /// End of the range (exclusive).
    pub end: SourceByteOffset,
}

impl SourceByteRange {
    /// Creates a new range from start and end offsets.
    #[inline]
    #[must_use]
    pub const fn new(start: SourceByteOffset, end: SourceByteOffset) -> Self {
        Self {
            start,
            end,
        }
    }

    /// Returns the length of the range in bytes.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.end.0.saturating_sub(self.start.0)
    }

    /// Returns true if the range is empty.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start.0 >= self.end.0
    }
}

/// Heading level (1-6).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct HeadingLevel(u8);

impl HeadingLevel {
    /// Creates a new `HeadingLevel`, validating it is between 1 and 6.
    ///
    /// # Errors
    /// Returns an error if the level is not in the range 1..=6.
    #[inline]
    pub fn try_new(level: u8) -> Result<Self, crate::note::error::NoteError> {
        if (1..=6).contains(&level) {
            Ok(Self(level))
        } else {
            Err(crate::note::error::NoteError::Structure(format!(
                "Invalid heading level: {level}. Must be between 1 and 6."
            )))
        }
    }

    /// Returns the raw level value.
    #[inline]
    #[must_use]
    pub const fn as_u8(&self) -> u8 {
        self.0
    }
}
