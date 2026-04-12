//! Shared source-related primitives for the Note context.
//!
//! Shared domain types for the Note context.
#![allow(
    clippy::exhaustive_structs,
    reason = "rkyv generates exhaustive archived types"
)]

use std::ops::Range;

use rkyv::{Archive, Deserialize, Serialize};

use super::error::{NoteError, StructureError};

/// A byte offset into the original UTF-8 source of a note.
///
/// Offset values are zero-indexed and represent the number of bytes from
/// the start of the file.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::position::SourceByteOffset;
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
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
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

    /// Returns the offset as a `usize`.
    #[inline]
    #[must_use]
    #[expect(clippy::as_conversions, reason = "u32 always fits in usize")]
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }

    /// Adds a relative offset to this byte offset.
    ///
    /// # Errors
    ///
    /// Returns [`StructureError::OffsetOverflow`] if the resulting offset
    /// cannot fit in `u32`.
    #[inline]
    pub fn add_offset(&self, delta: usize) -> Result<Self, NoteError> {
        let base: usize = (*self).into();
        Self::try_from(base.saturating_add(delta)).map_err(|_err| {
            StructureError::OutOfBounds {
                offset: Self::new(u32::MAX),
                source_len: Self::new(u32::MAX),
            }
            .into()
        })
    }

    /// Tries to create from a `usize`, returning a structured error on
    /// overflow.
    ///
    /// # Errors
    ///
    /// Returns [`StructureError::OutOfBounds`] if `value` exceeds `u32::MAX`.
    #[inline]
    pub fn try_from_usize(value: usize) -> Result<Self, StructureError> {
        Self::try_from(value).map_err(|_err| StructureError::OutOfBounds {
            offset: Self::new(u32::MAX),
            source_len: Self::new(u32::MAX),
        })
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

#[expect(clippy::as_conversions, reason = "u32 always fits in usize")]
impl From<SourceByteOffset> for usize {
    #[inline]
    fn from(offset: SourceByteOffset) -> Self {
        offset.0 as usize
    }
}

impl TryFrom<usize> for SourceByteOffset {
    type Error = std::num::TryFromIntError;

    #[inline]
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        u32::try_from(value).map(Self)
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
/// # use lithos_core::note::position::{SourceByteOffset, SourceByteRange};
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
    Hash,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
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
    /// Returns [`StructureError::InvalidRange`] if `start` is greater than
    /// `end`.
    #[inline]
    pub fn new(
        start: SourceByteOffset,
        end: SourceByteOffset,
    ) -> Result<Self, NoteError> {
        if start > end {
            return Err(StructureError::InvalidRange {
                start,
                end,
            }
            .into());
        }
        Ok(Self::new_unchecked(start, end))
    }

    #[inline]
    const fn new_unchecked(
        start: SourceByteOffset,
        end: SourceByteOffset,
    ) -> Self {
        Self {
            start,
            end,
        }
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

    /// Converts this byte range into a location range using cached line starts.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if the offsets are out of bounds or
    /// not on UTF-8 character boundaries.
    #[inline]
    pub fn to_location_range(
        &self,
        source: &str,
        line_starts: &[usize],
    ) -> Result<SourceLocationRange, NoteError> {
        let start = SourceLocation::try_from_byte_offset_with_index(
            self.start,
            source,
            line_starts,
        )?;
        let end = SourceLocation::try_from_byte_offset_with_index(
            self.end,
            source,
            line_starts,
        )?;
        Ok(SourceLocationRange::new_unchecked(start, end))
    }
}

impl TryFrom<Range<usize>> for SourceByteRange {
    type Error = NoteError;

    #[inline]
    fn try_from(range: Range<usize>) -> Result<Self, Self::Error> {
        let out_of_bounds =
            |_: std::num::TryFromIntError| StructureError::OutOfBounds {
                offset: SourceByteOffset::new(u32::MAX),
                source_len: SourceByteOffset::new(u32::MAX),
            };
        let start =
            SourceByteOffset::try_from(range.start).map_err(out_of_bounds)?;
        let end =
            SourceByteOffset::try_from(range.end).map_err(out_of_bounds)?;
        Self::new(start, end)
    }
}

/// 1-based line/column position derived from a byte offset.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct SourceLocation {
    offset: SourceByteOffset,
    line: SourceLine,
    column: SourceColumn,
}

impl SourceLocation {
    /// Creates a new source location from offset and line/column values.
    #[must_use]
    #[inline]
    pub const fn new(
        offset: SourceByteOffset,
        line: SourceLine,
        column: SourceColumn,
    ) -> Self {
        Self {
            offset,
            line,
            column,
        }
    }

    /// Returns the byte offset for this location.
    #[inline]
    #[must_use]
    pub const fn offset(&self) -> SourceByteOffset {
        self.offset
    }

    /// Returns the 1-based line number for this location.
    #[inline]
    #[must_use]
    pub const fn line(&self) -> SourceLine {
        self.line
    }

    /// Returns the 1-based column number for this location.
    #[inline]
    #[must_use]
    pub const fn column(&self) -> SourceColumn {
        self.column
    }

    /// Builds a source location from a byte offset and source text.
    ///
    /// Line and column numbers are 1-based. Column counts Unicode scalar
    /// values, not bytes.
    ///
    /// # Errors
    ///
    /// Returns [`StructureError::OutOfBounds`] if the offset exceeds the source
    /// length or is not on a UTF-8 character boundary.
    #[inline]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Line/column counters are bounded by source length"
    )]
    pub fn try_from_byte_offset(
        offset: SourceByteOffset,
        source: &str,
    ) -> Result<Self, NoteError> {
        let raw_offset = Self::validate_offset(offset, source)?;

        let mut line = 1u32;
        let mut column = 1u32;
        for (idx, ch) in source.char_indices() {
            if idx >= raw_offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        Ok(Self::new(
            offset,
            SourceLine::try_new(line)?,
            SourceColumn::try_new(column)?,
        ))
    }

    /// Builds a source location from a byte offset using cached line starts.
    ///
    /// Line and column numbers are 1-based. Column counts Unicode scalar
    /// values, not bytes.
    ///
    /// # Errors
    ///
    /// Returns [`StructureError::OutOfBounds`] if the offset exceeds the source
    /// length or is not on a UTF-8 character boundary.
    #[inline]
    pub fn try_from_byte_offset_with_index(
        offset: SourceByteOffset,
        source: &str,
        line_starts: &[usize],
    ) -> Result<Self, NoteError> {
        let raw_offset = Self::validate_offset(offset, source)?;

        let line_index = line_starts
            .partition_point(|&start| start <= raw_offset)
            .saturating_sub(1);
        let line_start = line_starts.get(line_index).copied().unwrap_or(0);
        let slice = source.get(line_start..raw_offset).ok_or_else(|| {
            let source_len = SourceByteOffset::try_from(source.len())
                .unwrap_or(SourceByteOffset::new(u32::MAX));
            StructureError::OutOfBounds {
                offset,
                source_len,
            }
        })?;
        let column_count = slice.chars().count().saturating_add(1);
        let line =
            u32::try_from(line_index.saturating_add(1)).map_err(|_err| {
                StructureError::InvalidLine {
                    line: u32::try_from(line_index.saturating_add(1))
                        .unwrap_or(u32::MAX),
                }
            })?;
        let column = u32::try_from(column_count).map_err(|_err| {
            StructureError::InvalidColumn {
                column: u32::try_from(column_count).unwrap_or(u32::MAX),
            }
        })?;

        Ok(SourceLocation::new(
            offset,
            SourceLine::try_new(line)?,
            SourceColumn::try_new(column)?,
        ))
    }

    #[inline]
    fn validate_offset(
        offset: SourceByteOffset,
        source: &str,
    ) -> Result<usize, NoteError> {
        let raw_offset: usize = offset.into();

        if raw_offset > source.len() {
            let source_len = SourceByteOffset::try_from(source.len())
                .unwrap_or(SourceByteOffset::new(u32::MAX));
            return Err(StructureError::OutOfBounds {
                offset,
                source_len,
            }
            .into());
        }
        if !source.is_char_boundary(raw_offset) {
            let source_len = SourceByteOffset::try_from(source.len())
                .unwrap_or(SourceByteOffset::new(u32::MAX));
            return Err(StructureError::OutOfBounds {
                offset,
                source_len,
            }
            .into());
        }
        Ok(raw_offset)
    }
}

/// Start/end locations for a source range.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct SourceLocationRange {
    start: SourceLocation,
    end: SourceLocation,
}

impl SourceLocationRange {
    /// Creates a new range from start and end locations.
    ///
    /// # Errors
    ///
    /// Returns [`StructureError::InvalidRange`] if `start` is after `end`.
    #[inline]
    pub fn new(
        start: SourceLocation,
        end: SourceLocation,
    ) -> Result<Self, NoteError> {
        if start.offset() > end.offset() {
            return Err(StructureError::InvalidRange {
                start: start.offset(),
                end: end.offset(),
            }
            .into());
        }
        Ok(Self::new_unchecked(start, end))
    }

    #[inline]
    const fn new_unchecked(start: SourceLocation, end: SourceLocation) -> Self {
        Self {
            start,
            end,
        }
    }

    /// Returns the start location (inclusive).
    #[inline]
    #[must_use]
    pub const fn start(&self) -> SourceLocation {
        self.start
    }

    /// Returns the end location (exclusive).
    #[inline]
    #[must_use]
    pub const fn end(&self) -> SourceLocation {
        self.end
    }

    /// Returns the byte range for this location range.
    #[inline]
    #[must_use]
    pub const fn byte_range(&self) -> SourceByteRange {
        SourceByteRange::new_unchecked(self.start.offset(), self.end.offset())
    }
}

/// 1-based line number derived from a byte offset.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct SourceLine(u32);

impl SourceLine {
    /// Creates a new 1-based line number.
    ///
    /// # Errors
    ///
    /// Returns [`StructureError::InvalidLine`] if the value is zero.
    #[inline]
    pub fn try_new(value: u32) -> Result<Self, NoteError> {
        if value == 0 {
            return Err(StructureError::InvalidLine {
                line: 0,
            }
            .into());
        }
        Ok(Self(value))
    }

    /// Returns the 1-based line number.
    #[inline]
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// 1-based column number derived from a byte offset.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct SourceColumn(u32);

impl SourceColumn {
    /// Creates a new 1-based column number.
    ///
    /// # Errors
    ///
    /// Returns [`StructureError::InvalidColumn`] if the value is zero.
    #[inline]
    pub fn try_new(value: u32) -> Result<Self, NoteError> {
        if value == 0 {
            return Err(StructureError::InvalidColumn {
                column: 0,
            }
            .into());
        }
        Ok(Self(value))
    }

    /// Returns the 1-based column number.
    #[inline]
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// Precomputed line start offsets for fast line/column lookups.
#[derive(Debug, Clone)]
#[cfg(test)]
pub(crate) struct LineIndex {
    line_starts: Vec<usize>,
}

#[cfg(test)]
impl LineIndex {
    /// Builds a new line index for the provided source text.
    #[inline]
    #[must_use]
    pub(crate) fn new(source: &str) -> Self {
        let mut line_starts = Vec::with_capacity(32);
        line_starts.push(0);
        for (idx, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                let next = idx.saturating_add(1);
                if next <= source.len() {
                    line_starts.push(next);
                }
            }
        }
        Self {
            line_starts,
        }
    }

    #[inline]
    #[must_use]
    pub(crate) fn as_slice(&self) -> &[usize] {
        &self.line_starts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn line_column_starts_at_one() -> Result<(), NoteError> {
        let source = "abc";
        let offset = SourceByteOffset::new(0);
        let location = SourceLocation::try_from_byte_offset(offset, source)?;
        assert_eq!(location.line().value(), 1);
        assert_eq!(location.column().value(), 1);
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn line_column_tracks_unicode_and_newlines() -> Result<(), NoteError> {
        let source = "a\u{00E9}\nb";
        let offset = SourceByteOffset::try_from("a".len()).map_err(|_err| {
            StructureError::OutOfBounds {
                offset: SourceByteOffset::new(u32::MAX),
                source_len: SourceByteOffset::new(u32::MAX),
            }
        })?;
        let location = SourceLocation::try_from_byte_offset(offset, source)?;
        assert_eq!(location.line().value(), 1);
        assert_eq!(location.column().value(), 2);

        let newline_offset = SourceByteOffset::try_from("a\u{00E9}\n".len())
            .map_err(|_err| StructureError::OutOfBounds {
                offset: SourceByteOffset::new(u32::MAX),
                source_len: SourceByteOffset::new(u32::MAX),
            })?;
        let location_after_newline =
            SourceLocation::try_from_byte_offset(newline_offset, source)?;
        assert_eq!(location_after_newline.line().value(), 2);
        assert_eq!(location_after_newline.column().value(), 1);
        Ok(())
    }

    #[test]
    fn line_column_rejects_out_of_bounds() {
        let source = "abc";
        let offset = SourceByteOffset::new(10);
        let result = SourceLocation::try_from_byte_offset(offset, source);
        assert!(matches!(result, Err(NoteError::Structure(_))));
    }

    #[test]
    fn line_column_rejects_non_boundary() {
        let source = "a\u{00E9}";
        let offset = SourceByteOffset::new(2);
        let result = SourceLocation::try_from_byte_offset(offset, source);
        assert!(matches!(result, Err(NoteError::Structure(_))));
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn line_index_matches_offset_lookup() -> Result<(), NoteError> {
        let source = "first\nsecond\nthird";
        let index = LineIndex::new(source);
        let offset = SourceByteOffset::try_from("first\nsecond".len())
            .map_err(|_err| StructureError::OutOfBounds {
                offset: SourceByteOffset::new(u32::MAX),
                source_len: SourceByteOffset::new(u32::MAX),
            })?;
        let location = SourceLocation::try_from_byte_offset_with_index(
            offset,
            source,
            index.as_slice(),
        )?;
        assert_eq!(location.line().value(), 2);
        assert_eq!(location.column().value(), 7);
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn line_index_handles_crlf() -> Result<(), NoteError> {
        let source = "first\r\nsecond";
        let index = LineIndex::new(source);
        let offset =
            SourceByteOffset::try_from("first\r\n".len()).map_err(|_err| {
                StructureError::OutOfBounds {
                    offset: SourceByteOffset::new(u32::MAX),
                    source_len: SourceByteOffset::new(u32::MAX),
                }
            })?;
        let location = SourceLocation::try_from_byte_offset_with_index(
            offset,
            source,
            index.as_slice(),
        )?;
        assert_eq!(location.line().value(), 2);
        assert_eq!(location.column().value(), 1);
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn location_range_accepts_ordered_offsets() -> Result<(), NoteError> {
        let start = SourceLocation::new(
            SourceByteOffset::new(0),
            SourceLine::try_new(1)?,
            SourceColumn::try_new(1)?,
        );
        let end = SourceLocation::new(
            SourceByteOffset::new(5),
            SourceLine::try_new(1)?,
            SourceColumn::try_new(6)?,
        );
        let range = SourceLocationRange::new(start, end)?;
        let byte_range = range.byte_range();
        assert_eq!(byte_range.len(), 5);
        Ok(())
    }

    #[test]
    fn location_range_rejects_inverted_offsets() {
        let start = SourceLocation::new(
            SourceByteOffset::new(5),
            SourceLine::try_new(1).expect("valid line"),
            SourceColumn::try_new(6).expect("valid column"),
        );
        let end = SourceLocation::new(
            SourceByteOffset::new(0),
            SourceLine::try_new(1).expect("valid line"),
            SourceColumn::try_new(1).expect("valid column"),
        );
        let result = SourceLocationRange::new(start, end);
        assert!(matches!(result, Err(NoteError::Structure(_))));
    }
}
