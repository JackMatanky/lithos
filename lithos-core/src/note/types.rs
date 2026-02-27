//! Shared source-related primitives for the Note context.

//! Shared domain types for the Note context.
#![allow(
    clippy::exhaustive_structs,
    reason = "rkyv generates exhaustive archived types"
)]

use rkyv::{Archive, Deserialize, Serialize};

use super::error::NoteError;

/// A byte offset into the original UTF-8 source of a note.
///
/// Offset values are zero-indexed and represent the number of bytes from
/// the start of the file.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::types::SourceByteOffset;
/// let offset = SourceByteOffset::new(1024);
/// assert_eq!(u32::from(offset), 1024);
/// ```
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
///
/// Defines a contiguous span of text within a note, typically representing
/// an entity's location.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::types::{SourceByteOffset, SourceByteRange};
/// let start = SourceByteOffset::new(0);
/// let end = SourceByteOffset::new(10);
/// let range = SourceByteRange::new(start, end)?;
///
/// assert_eq!(range.len(), 10);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
    start: SourceByteOffset,
    /// End of the range (exclusive).
    end: SourceByteOffset,
}

impl SourceByteRange {
    /// Creates a new range from start and end offsets.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Structure`] if `start` is greater than `end`.
    #[inline]
    pub fn new(
        start: SourceByteOffset,
        end: SourceByteOffset,
    ) -> Result<Self, NoteError> {
        if start > end {
            return Err(NoteError::Structure(
                "source range start must be <= end".into(),
            ));
        }
        Ok(Self {
            start,
            end,
        })
    }

    /// Returns the start offset (inclusive).
    #[inline]
    #[must_use]
    pub const fn start(&self) -> SourceByteOffset {
        self.start
    }

    /// Returns the end offset (exclusive).
    #[inline]
    #[must_use]
    pub const fn end(&self) -> SourceByteOffset {
        self.end
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
        self.start.0 == self.end.0
    }
}
