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

    /// Creates a new `SourceByteOffset` from a `usize` offset.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Structure`] if the offset cannot fit in `u32`.
    #[inline]
    pub fn try_from_usize(offset: usize) -> Result<Self, NoteError> {
        Self::try_from(offset).map_err(|error| {
            NoteError::Structure(
                format!("source offset out of range: {error}").into(),
            )
        })
    }

    /// Converts this byte offset into a line/column pair for the given source.
    ///
    /// Line and column numbers are 1-based. Column counts Unicode scalar
    /// values, not bytes.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Structure`] if the offset exceeds the source
    /// length or is not on a UTF-8 character boundary.
    #[inline]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Line/column counters are bounded by source length"
    )]
    pub fn line_column(
        self,
        source: &str,
    ) -> Result<SourceLineColumn, NoteError> {
        let offset = usize::from(self);
        if offset > source.len() {
            return Err(NoteError::Structure(
                "source offset exceeds input length".into(),
            ));
        }
        if !source.is_char_boundary(offset) {
            return Err(NoteError::Structure(
                "source offset is not on a character boundary".into(),
            ));
        }

        let mut line = 1u32;
        let mut column = 1u32;
        for (idx, ch) in source.char_indices() {
            if idx >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        Ok(SourceLineColumn {
            line,
            column,
        })
    }
}

/// 1-based line/column position derived from a byte offset.
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
pub struct SourceLineColumn {
    line: u32,
    column: u32,
}

impl SourceLineColumn {
    /// Returns the 1-based line number.
    #[inline]
    #[must_use]
    pub const fn line(&self) -> u32 {
        self.line
    }

    /// Returns the 1-based column number.
    #[inline]
    #[must_use]
    pub const fn column(&self) -> u32 {
        self.column
    }
}

/// Precomputed line start offsets for fast line/column lookups.
#[derive(Debug, Clone)]
pub struct SourceLineIndex {
    line_starts: Vec<usize>,
}

impl SourceLineIndex {
    /// Builds a new line index for the provided source text.
    #[inline]
    #[must_use]
    pub fn new(source: &str) -> Self {
        let mut line_starts = Vec::with_capacity(32);
        line_starts.push(0);
        for (idx, ch) in source.char_indices() {
            if ch == '\n' {
                let next = idx.saturating_add(ch.len_utf8());
                if next <= source.len() {
                    line_starts.push(next);
                }
            }
        }
        Self {
            line_starts,
        }
    }

    /// Converts a byte offset into a line/column using the cached index.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Structure`] if the offset is out of bounds or not
    /// on a UTF-8 character boundary.
    #[inline]
    pub fn line_column(
        &self,
        offset: SourceByteOffset,
        source: &str,
    ) -> Result<SourceLineColumn, NoteError> {
        let offset = usize::from(offset);
        if offset > source.len() {
            return Err(NoteError::Structure(
                "source offset exceeds input length".into(),
            ));
        }
        if !source.is_char_boundary(offset) {
            return Err(NoteError::Structure(
                "source offset is not on a character boundary".into(),
            ));
        }

        let line_index = self
            .line_starts
            .partition_point(|&start| start <= offset)
            .saturating_sub(1);
        let line_start = self.line_starts.get(line_index).copied().unwrap_or(0);
        let slice = source.get(line_start..offset).ok_or_else(|| {
            NoteError::Structure("source offset is not on a boundary".into())
        })?;
        let column_count = slice.chars().count().saturating_add(1);
        let line =
            u32::try_from(line_index.saturating_add(1)).map_err(|_error| {
                NoteError::Structure("line index out of range".into())
            })?;
        let column = u32::try_from(column_count).map_err(|_error| {
            NoteError::Structure("column index out of range".into())
        })?;

        Ok(SourceLineColumn {
            line,
            column,
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
        let line_column = offset.line_column(source)?;
        assert_eq!(line_column.line(), 1);
        assert_eq!(line_column.column(), 1);
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn line_column_tracks_unicode_and_newlines() -> Result<(), NoteError> {
        let source = "a\u{00E9}\nb";
        let offset = SourceByteOffset::try_from("a".len())
            .map_err(|error| NoteError::Structure(error.to_string().into()))?;
        let line_column = offset.line_column(source)?;
        assert_eq!(line_column.line(), 1);
        assert_eq!(line_column.column(), 2);

        let newline_offset = SourceByteOffset::try_from("a\u{00E9}\n".len())
            .map_err(|error| NoteError::Structure(error.to_string().into()))?;
        let line_column_after_newline = newline_offset.line_column(source)?;
        assert_eq!(line_column_after_newline.line(), 2);
        assert_eq!(line_column_after_newline.column(), 1);
        Ok(())
    }

    #[test]
    fn line_column_rejects_out_of_bounds() {
        let source = "abc";
        let offset = SourceByteOffset::new(10);
        let result = offset.line_column(source);
        assert!(matches!(result, Err(NoteError::Structure(_))));
    }

    #[test]
    fn line_column_rejects_non_boundary() {
        let source = "a\u{00E9}";
        let offset = SourceByteOffset::new(2);
        let result = offset.line_column(source);
        assert!(matches!(result, Err(NoteError::Structure(_))));
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn line_index_matches_offset_lookup() -> Result<(), NoteError> {
        let source = "first\nsecond\nthird";
        let index = SourceLineIndex::new(source);
        let offset = SourceByteOffset::try_from("first\nsecond".len())
            .map_err(|error| NoteError::Structure(error.to_string().into()))?;
        let line_column = index.line_column(offset, source)?;
        assert_eq!(line_column.line(), 2);
        assert_eq!(line_column.column(), 7);
        Ok(())
    }
}
