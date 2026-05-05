//! Shared source-related primitives for the Note context.
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
    Default,
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
        let summed =
            base.checked_add(delta).ok_or(StructureError::OffsetOverflow {
                offset: *self,
                delta,
            })?;
        Self::try_from(summed).map_err(|_err| {
            StructureError::OffsetOverflow {
                offset: *self,
                delta,
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

    /// Returns the offset incremented by `rhs`, saturating at `u32::MAX`.
    #[inline]
    #[must_use]
    pub const fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
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
pub struct SourceByteRange(Range<u32>);

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
    pub(crate) const fn new_unchecked(
        start: SourceByteOffset,
        end: SourceByteOffset,
    ) -> Self {
        Self(start.0..end.0)
    }

    /// Returns the start offset (inclusive).
    #[inline]
    #[must_use]
    pub const fn start(&self) -> SourceByteOffset {
        SourceByteOffset(self.0.start)
    }

    /// Returns the end offset (exclusive).
    #[inline]
    #[must_use]
    pub const fn end(&self) -> SourceByteOffset {
        SourceByteOffset(self.0.end)
    }

    /// Returns the length of the range in bytes.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.0.end.saturating_sub(self.0.start)
    }

    /// Returns true if the range is empty.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.start == self.0.end
    }

    /// Returns this range as a standard `Range<usize>` for slicing strings.
    #[inline]
    #[must_use]
    #[expect(clippy::as_conversions, reason = "u32 always fits in usize")]
    pub const fn as_usize_range(&self) -> Range<usize> {
        (self.0.start as usize)..(self.0.end as usize)
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
        line_index: &LineIndex,
    ) -> Result<SourceLocationRange, NoteError> {
        let start = line_index.offset_to_location(source, self.start())?;
        let end = line_index.offset_to_location(source, self.end())?;
        Ok(SourceLocationRange::new_unchecked(start, end))
    }
}

impl TryFrom<Range<usize>> for SourceByteRange {
    type Error = NoteError;

    #[inline]
    fn try_from(range: Range<usize>) -> Result<Self, Self::Error> {
        let start = SourceByteOffset::try_from_usize(range.start)?;
        let end = SourceByteOffset::try_from_usize(range.end)?;
        Self::new(start, end)
    }
}

/// A flat array of valid text byte ranges.
///
/// Serves as the zero-copy scannable index for the artifact discovery phase.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct SourceByteRangeIndex(Vec<SourceByteRange>);

impl SourceByteRangeIndex {
    /// Creates a new, empty index.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Appends a range to the index.
    #[inline]
    pub fn push(&mut self, range: SourceByteRange) {
        self.0.push(range);
    }

    /// Returns true if the index is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of ranges in the index.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the ranges as a slice.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[SourceByteRange] {
        &self.0
    }

    /// Returns an iterator over the ranges.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, SourceByteRange> {
        self.0.iter()
    }
}

impl Default for SourceByteRangeIndex {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'range> IntoIterator for &'range SourceByteRangeIndex {
    type IntoIter = std::slice::Iter<'range, SourceByteRange>;
    type Item = &'range SourceByteRange;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
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
    pub fn try_from_byte_offset(
        source: &str,
        offset: SourceByteOffset,
    ) -> Result<Self, NoteError> {
        let line_index = LineIndex::new(source);
        line_index.offset_to_location(source, offset)
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
        source: &str,
        offset: SourceByteOffset,
        line_index: &LineIndex,
    ) -> Result<Self, NoteError> {
        line_index.offset_to_location(source, offset)
    }

    #[inline]
    fn validate_offset(
        source: &str,
        offset: SourceByteOffset,
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
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct LineIndex(Vec<u32>);

impl LineIndex {
    /// Builds a new line index for the provided source text.
    #[inline]
    #[must_use]
    pub fn new(source: &str) -> Self {
        let mut line_starts = Vec::with_capacity(32);
        line_starts.push(0);
        for (idx, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                let next = idx.saturating_add(1);
                if next <= source.len()
                    && let Ok(v) = u32::try_from(next)
                {
                    line_starts.push(v);
                }
            }
        }
        Self(line_starts)
    }

    /// Converts a byte offset to a line and column location in O(log n) time.
    ///
    /// # Errors
    ///
    /// Returns [`StructureError::OutOfBounds`] if the offset exceeds the source
    /// length or is not on a UTF-8 character boundary.
    #[inline]
    pub fn offset_to_location(
        &self,
        source: &str,
        offset: SourceByteOffset,
    ) -> Result<SourceLocation, NoteError> {
        let raw_offset = SourceLocation::validate_offset(source, offset)?;

        let line_idx = self
            .0
            .partition_point(|&start| {
                usize::try_from(start).is_ok_and(|value| value <= raw_offset)
            })
            .saturating_sub(1);
        let line_start_u32 = self.0.get(line_idx).copied().unwrap_or(0);
        let line_start = usize::try_from(line_start_u32).map_err(|_err| {
            let source_len = SourceByteOffset::try_from(source.len())
                .unwrap_or(SourceByteOffset::new(u32::MAX));
            StructureError::OutOfBounds {
                offset,
                source_len,
            }
        })?;

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
            u32::try_from(line_idx.saturating_add(1)).map_err(|_err| {
                StructureError::InvalidLine {
                    line: u32::try_from(line_idx.saturating_add(1))
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
        let location = SourceLocation::try_from_byte_offset(source, offset)?;
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
        let location = SourceLocation::try_from_byte_offset(source, offset)?;
        assert_eq!(location.line().value(), 1);
        assert_eq!(location.column().value(), 2);

        let newline_offset = SourceByteOffset::try_from("a\u{00E9}\n".len())
            .map_err(|_err| StructureError::OutOfBounds {
                offset: SourceByteOffset::new(u32::MAX),
                source_len: SourceByteOffset::new(u32::MAX),
            })?;
        let location_after_newline =
            SourceLocation::try_from_byte_offset(source, newline_offset)?;
        assert_eq!(location_after_newline.line().value(), 2);
        assert_eq!(location_after_newline.column().value(), 1);
        Ok(())
    }

    #[test]
    fn line_column_rejects_out_of_bounds() {
        let source = "abc";
        let offset = SourceByteOffset::new(10);
        let result = SourceLocation::try_from_byte_offset(source, offset);
        assert!(matches!(result, Err(NoteError::Structure(_))));
    }

    #[test]
    fn line_column_rejects_non_boundary() {
        let source = "a\u{00E9}";
        let offset = SourceByteOffset::new(2);
        let result = SourceLocation::try_from_byte_offset(source, offset);
        assert!(matches!(result, Err(NoteError::Structure(_))));
    }

    #[test]
    fn add_offset_returns_overflow_error() {
        let offset = SourceByteOffset::new(u32::MAX);
        let result = offset.add_offset(1);

        assert!(matches!(
            result,
            Err(NoteError::Structure(StructureError::OffsetOverflow {
                offset: o,
                delta: 1,
            })) if o == SourceByteOffset::new(u32::MAX)
        ));
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
            source, offset, &index,
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
            source, offset, &index,
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
