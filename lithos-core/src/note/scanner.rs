//! Unified scanning utilities for Note metadata and structure.
//!
//! This module consolidates manual text-scanning logic for Obsidian-style
//! metadata (tags, inline fields, and block references) into a single
//! high-performance state machine.
//!
//! The scanner is designed to be:
//! 1. **Single-Pass**: Each byte of input is touched exactly once.
//! 2. **Zero-Copy**: All artifacts borrow directly from the source text.
//! 3. **Resumable**: Using the [`Cursor`] type, scanning can be paused and
//!    resumed across disjoint text segments while maintaining context.

use crate::note::{
    error::NoteError,
    position::{SourceByteOffset, SourceByteRange},
    raw::{RawBlockRef, RawInlineFieldToken, RawTag},
};

// ── Primary public API types ─────────────────────────────────────────────────

/// A cursor-based scanner for extracting metadata artifacts from markdown.
///
/// `NoteScanner` manages the rules for what constitutes valid metadata
/// (such as allowed emoji markers) while the [`Cursor`] tracks the progress
/// through the text.
#[derive(Debug, Clone)]
pub struct NoteScanner {
    emoji_markers: Box<[char]>,
}

impl NoteScanner {
    /// Create a new scanner with a custom set of emoji markers.
    #[inline]
    #[must_use]
    pub fn new<T: Into<Box<[char]>>>(emoji_markers: T) -> Self {
        Self {
            emoji_markers: emoji_markers.into(),
        }
    }

    /// Scans a contiguous block of text for all metadata artifacts.
    ///
    /// This is a convenience method that creates a temporary cursor and
    /// collects all artifacts into a `Vec`.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if position calculation for an artifact exceeds
    /// supported byte bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::scanner::NoteScanner;
    /// # use lithos_core::note::position::SourceByteOffset;
    /// let scanner = NoteScanner::new(vec!['📅']);
    /// let artifacts = scanner
    ///     .scan_block("#work [due:: tomorrow]", SourceByteOffset::new(0))
    ///     .unwrap();
    /// assert_eq!(artifacts.len(), 2);
    /// ```
    #[inline]
    pub fn scan_block<'source>(
        &self,
        text: &'source str,
        base_offset: SourceByteOffset,
    ) -> Result<Vec<ScannedArtifact<'source>>, NoteError> {
        let mut cursor = Cursor::new(text, base_offset);
        let mut artifacts = Vec::with_capacity(8);
        self.scan_cursor(&mut cursor, &mut artifacts)?;
        Ok(artifacts)
    }

    /// Scans multiple disjoint ranges within the same source text and returns
    /// raw tokens grouped by type.
    ///
    /// This preserves cursor state across ranges to maintain word-boundary
    /// semantics and line-start detection.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if any range offset exceeds supported bounds.
    #[inline]
    pub(crate) fn scan_ranges<'source>(
        &self,
        text: &'source str,
        ranges: &[std::ops::Range<usize>],
        _include_task_marker: bool,
    ) -> Result<ScannedRawArtifacts<'source>, NoteError> {
        let mut artifacts = Vec::with_capacity(8);
        let mut cursor = Cursor::new("", SourceByteOffset::new(0));
        for range in ranges {
            if range.is_empty() {
                continue;
            }
            let Some(segment) = text.get(range.clone()) else {
                continue;
            };
            let base_offset = SourceByteOffset::try_from_usize(range.start)?;
            cursor.reset(segment, base_offset);
            self.scan_cursor(&mut cursor, &mut artifacts)?;
        }
        Ok(ScannedRawArtifacts::from_scanned_artifacts(artifacts))
    }

    /// Continues scanning from a provided [`Cursor`] state.
    ///
    /// This method allows the scanner to process disjoint segments of text
    /// (e.g., text interrupted by links) while maintaining the state required
    /// to correctly identify line-start triggers and alphanumeric boundaries.
    ///
    /// New artifacts are pushed onto the provided `artifacts` vector.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if position calculation for an artifact exceeds
    /// supported byte bounds.
    #[inline]
    pub fn scan_cursor<'source>(
        &self,
        cursor: &mut Cursor<'source>,
        artifacts: &mut Vec<ScannedArtifact<'source>>,
    ) -> Result<(), NoteError> {
        while !cursor.is_eof() {
            match cursor.mode {
                ScanMode::AtLineStart => {
                    Self::handle_line_start(cursor, artifacts)?;
                }
                ScanMode::InBody => {
                    self.handle_body(cursor, artifacts)?;
                }
            }
        }
        Ok(())
    }

    /// Scans for a tag (e.g., `#tag`) at the current cursor position.
    pub(crate) fn scan_tag<'source>(
        cursor: &mut Cursor<'source>,
    ) -> Result<Option<ScannedArtifact<'source>>, NoteError> {
        if !cursor.rest.starts_with('#') {
            return Ok(None);
        }

        let mut len: usize = 1;
        let chars = cursor.rest.get(1..).unwrap_or("").chars();
        let mut has_content = false;

        for c in chars {
            if c.is_alphanumeric() || matches!(c, '_' | '-' | '/') {
                len = len.saturating_add(c.len_utf8());
                has_content = true;
            } else {
                break;
            }
        }

        if has_content {
            let value = cursor.rest.get(0..len).unwrap_or("");
            let range = SourceByteRange::new(
                cursor.offset,
                cursor.offset.add_offset(len)?,
            )?;
            cursor.advance(len)?;
            Ok(Some(ScannedArtifact::Tag(RawTag::new(value.into(), range))))
        } else {
            Ok(None)
        }
    }

    /// Scans for a delimited field (e.g., `[key:: value]` or `(key:: value)`)
    /// at the current cursor position.
    pub(crate) fn scan_delimited_field<'source>(
        cursor: &mut Cursor<'source>,
    ) -> Result<Option<ScannedArtifact<'source>>, NoteError> {
        let opener = cursor.peek_byte();
        let closer = match opener {
            Some(b'[') => b']',
            Some(b'(') => b')',
            _ => return Ok(None),
        };

        let mut len: usize = 1;
        let mut chars = cursor.rest.get(1..).unwrap_or("").chars();
        let mut found_sep = false;
        let mut sep_pos = 0;

        while let Some(c) = chars.next() {
            len = len.saturating_add(c.len_utf8());
            let current_rest = cursor
                .rest
                .get(len.saturating_sub(c.len_utf8())..)
                .unwrap_or("");
            if !found_sep && current_rest.starts_with("::") {
                found_sep = true;
                sep_pos = len.saturating_sub(c.len_utf8());
                len = len.saturating_add(1); // Skip second colon
                chars.next(); // Consume second colon
                continue;
            }

            if u32::from(c) == u32::from(closer) {
                if !found_sep {
                    break;
                }

                let key = cursor.rest.get(1..sep_pos).unwrap_or("").trim();
                let value = cursor
                    .rest
                    .get(sep_pos.saturating_add(2)..len.saturating_sub(1))
                    .unwrap_or("")
                    .trim();

                if key.is_empty() {
                    break;
                }

                let range = SourceByteRange::new(
                    cursor.offset,
                    cursor.offset.add_offset(len)?,
                )?;
                cursor.advance(len)?;
                return Ok(Some(ScannedArtifact::InlineField(
                    RawInlineFieldToken::new(key.into(), value.into(), range),
                )));
            }

            if c == '\n' {
                break;
            }
        }

        Ok(None)
    }

    /// Scans for a bare field (e.g., `key:: value`) at the start of a line.
    pub(crate) fn scan_bare_field<'source>(
        cursor: &mut Cursor<'source>,
    ) -> Result<Option<ScannedArtifact<'source>>, NoteError> {
        let bytes = cursor.rest.as_bytes();
        let mut key_len = 0usize;

        while let Some(&b) = bytes.get(key_len) {
            if b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-') {
                key_len = key_len.saturating_add(1);
            } else {
                break;
            }
        }

        if key_len > 0
            && cursor.rest.get(key_len..).is_some_and(|s| s.starts_with("::"))
        {
            let key = cursor.rest.get(..key_len).unwrap_or("");
            let after_colons =
                cursor.rest.get(key_len.saturating_add(2)..).unwrap_or("");
            let mut val_len = 0usize;
            for ch in after_colons.chars() {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                val_len = val_len.saturating_add(ch.len_utf8());
            }

            let value = after_colons.get(..val_len).unwrap_or("").trim();
            if !value.is_empty() {
                let start = cursor.offset;
                let total_len =
                    key_len.saturating_add(2).saturating_add(val_len);
                let end = cursor.offset.add_offset(total_len)?;
                let range = SourceByteRange::new(start, end)?;
                cursor.advance(total_len)?;
                return Ok(Some(ScannedArtifact::InlineField(
                    RawInlineFieldToken::new(key.into(), value.into(), range),
                )));
            }
        }

        Ok(None)
    }

    /// Scans for an emoji-prefixed field (e.g., `📅 2024-03-20`).
    pub(crate) fn scan_emoji_field<'source>(
        cursor: &mut Cursor<'source>,
    ) -> Result<Option<ScannedArtifact<'source>>, NoteError> {
        let mut chars = cursor.rest.chars();
        let emoji = chars.next().ok_or(NoteError::UnexpectedEof)?;
        let mut len = emoji.len_utf8();

        // Optional whitespace after emoji
        for c in chars {
            if c.is_whitespace() && c != '\n' && c != '\r' {
                len = len.saturating_add(c.len_utf8());
            } else {
                break;
            }
        }

        // Value is the rest of the line
        let value_start = len;
        let mut value_len: usize = 0;
        let value_chars = cursor.rest.get(value_start..).unwrap_or("").chars();
        for c in value_chars {
            if c == '\n' || c == '\r' {
                break;
            }
            value_len = value_len.saturating_add(c.len_utf8());
        }

        if value_len > 0 {
            let key = emoji.to_string();
            let value = cursor
                .rest
                .get(value_start..value_start.saturating_add(value_len))
                .unwrap_or("")
                .trim();
            let range = SourceByteRange::new(
                cursor.offset,
                cursor.offset.add_offset(len.saturating_add(value_len))?,
            )?;
            cursor.advance(len.saturating_add(value_len))?;
            Ok(Some(ScannedArtifact::InlineField(RawInlineFieldToken::new(
                key.into(),
                value.into(),
                range,
            ))))
        } else {
            Ok(None)
        }
    }

    /// Scans for a block reference (e.g., `^ref-id`).
    pub(crate) fn scan_block_ref<'source>(
        cursor: &mut Cursor<'source>,
    ) -> Result<Option<ScannedArtifact<'source>>, NoteError> {
        if !cursor.rest.starts_with('^') {
            return Ok(None);
        }

        let mut len: usize = 1;
        let chars = cursor.rest.get(1..).unwrap_or("").chars();
        let mut has_content = false;

        for c in chars {
            if c.is_alphanumeric() || matches!(c, '-' | '_') {
                len = len.saturating_add(c.len_utf8());
                has_content = true;
            } else {
                break;
            }
        }

        if has_content {
            let remaining = cursor.rest.get(len..).unwrap_or("");
            let mut tail_len = 0usize;
            for ch in remaining.chars() {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                if !ch.is_whitespace() {
                    return Ok(None);
                }
                tail_len = tail_len.saturating_add(ch.len_utf8());
            }

            let id = cursor.rest.get(1..len).unwrap_or("");
            let range = SourceByteRange::new(
                cursor.offset,
                cursor.offset.add_offset(len)?,
            )?;
            cursor.advance(len.saturating_add(tail_len))?;
            Ok(Some(ScannedArtifact::BlockRef(RawBlockRef::new(
                id.into(),
                range.start(),
            ))))
        } else {
            Ok(None)
        }
    }

    fn handle_line_start<'source>(
        cursor: &mut Cursor<'source>,
        artifacts: &mut Vec<ScannedArtifact<'source>>,
    ) -> Result<(), NoteError> {
        cursor.skip_whitespace_on_line()?;

        let Some(first) = cursor.peek_byte() else {
            cursor.mode = ScanMode::InBody;
            return Ok(());
        };

        if first == b'\n' || first == b'\r' {
            cursor.mode = ScanMode::InBody;
            return Ok(());
        }

        Self::run_line_start_rules(cursor, artifacts)?;

        cursor.mode = ScanMode::InBody;
        Ok(())
    }

    fn run_line_start_rules<'source>(
        cursor: &mut Cursor<'source>,
        artifacts: &mut Vec<ScannedArtifact<'source>>,
    ) -> Result<bool, NoteError> {
        // We removed TaskRule as extraction is now a domain concern.
        let bare_field_rule = BareFieldRule;
        if bare_field_rule.try_scan(cursor, artifacts)? {
            return Ok(true);
        }
        Ok(false)
    }

    fn run_body_rules<'source>(
        &self,
        cursor: &mut Cursor<'source>,
        artifacts: &mut Vec<ScannedArtifact<'source>>,
    ) -> Result<bool, NoteError> {
        let tag_rule = TagRule;
        if tag_rule.try_scan(self, cursor, artifacts)? {
            return Ok(true);
        }
        let delimited_field_rule = DelimitedFieldRule;
        if delimited_field_rule.try_scan(self, cursor, artifacts)? {
            return Ok(true);
        }
        let block_ref_rule = BlockRefRule;
        if block_ref_rule.try_scan(self, cursor, artifacts)? {
            return Ok(true);
        }
        let emoji_field_rule = EmojiFieldRule;
        if emoji_field_rule.try_scan(self, cursor, artifacts)? {
            return Ok(true);
        }
        Ok(false)
    }

    fn handle_body<'source>(
        &self,
        cursor: &mut Cursor<'source>,
        artifacts: &mut Vec<ScannedArtifact<'source>>,
    ) -> Result<(), NoteError> {
        match cursor.peek_byte() {
            Some(b'\n' | b'\r') => {
                cursor.advance(1)?;
                cursor.mode = ScanMode::AtLineStart;
                cursor.prev_alnum = false;
                return Ok(());
            }
            None => return Ok(()),
            _ => {}
        }

        if Self::run_body_rules(self, cursor, artifacts)? {
            return Ok(());
        }

        // Advance cursor
        if let Some(c) = cursor.rest.chars().next() {
            cursor.prev_alnum = c.is_alphanumeric();
            cursor.advance(c.len_utf8())?;
        }

        Ok(())
    }
}

// ── Supporting cursor type ───────────────────────────────────────────────────

/// A tracking cursor for the note scanner.
#[derive(Debug, Clone)]
#[non_exhaustive]
#[expect(
    clippy::partial_pub_fields,
    reason = "Cursor encapsulation hides internal scanning modes"
)]
pub struct Cursor<'source> {
    /// The remaining text to be scanned.
    pub rest: &'source str,
    /// The current absolute source offset.
    pub offset: SourceByteOffset,
    /// The current scanning mode.
    mode: ScanMode,
    /// Whether the previous character was alphanumeric (for word boundaries).
    pub prev_alnum: bool,
}

impl<'source> Cursor<'source> {
    /// Create a new cursor starting at `offset`.
    #[inline]
    #[must_use]
    pub const fn new(text: &'source str, offset: SourceByteOffset) -> Self {
        Self {
            rest: text,
            offset,
            mode: ScanMode::AtLineStart,
            prev_alnum: false,
        }
    }

    /// Reset the cursor with new text and offset.
    #[inline]
    pub fn reset(&mut self, text: &'source str, offset: SourceByteOffset) {
        self.rest = text;
        self.offset = offset;
        // Mode and prev_alnum are preserved to maintain context across ranges.
    }

    /// Returns `true` if the cursor has reached the end of the input.
    #[inline]
    #[must_use]
    pub const fn is_eof(&self) -> bool {
        self.rest.is_empty()
    }

    /// Peeks at the next byte without advancing.
    #[inline]
    #[must_use]
    pub fn peek_byte(&self) -> Option<u8> {
        self.rest.as_bytes().first().copied()
    }

    /// Advances the cursor by `n` bytes.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if the offset calculation exceeds supported
    /// bounds.
    #[inline]
    pub fn advance(&mut self, n: usize) -> Result<(), NoteError> {
        if n == 0 {
            return Ok(());
        }
        self.rest = self.rest.get(n..).unwrap_or("");
        self.offset = self.offset.add_offset(n)?;
        Ok(())
    }

    /// Skips horizontal whitespace characters on the current line.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if the offset calculation exceeds supported
    /// bounds.
    #[inline]
    pub fn skip_whitespace_on_line(&mut self) -> Result<(), NoteError> {
        let mut len: usize = 0;
        for c in self.rest.chars() {
            if c.is_whitespace() && c != '\n' && c != '\r' {
                len = len.saturating_add(c.len_utf8());
            } else {
                break;
            }
        }
        self.advance(len)
    }
}

// ── Scanned artifact types ───────────────────────────────────────────────────

/// A single metadata artifact extracted from a scan pass.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ScannedArtifact<'source> {
    /// A hashtag (e.g., `#tag`).
    Tag(RawTag<'source>),
    /// An inline key-value pair (e.g., `[key:: value]`).
    InlineField(RawInlineFieldToken<'source>),
    /// A block reference (e.g., `^ref-id`).
    BlockRef(RawBlockRef<'source>),
}

impl ScannedArtifact<'_> {
    /// Converts the artifact to an owned variant.
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> ScannedArtifact<'static> {
        match self {
            Self::Tag(tag) => ScannedArtifact::Tag(tag.into_owned()),
            Self::InlineField(field) => {
                ScannedArtifact::InlineField(field.into_owned())
            }
            Self::BlockRef(block_ref) => {
                ScannedArtifact::BlockRef(block_ref.into_owned())
            }
        }
    }
}

// ── Internal output type ─────────────────────────────────────────────────────

/// Raw tokens extracted from a single scan pass, grouped by artifact type.
///
/// Produced by [`NoteScanner::scan_ranges`] and consumed by
/// [`BlockExtractor`](crate::note::extractor::BlockExtractor) to populate
/// [`RawNote`](crate::note::raw::RawNote) collections.
#[derive(Debug, Default)]
pub(crate) struct ScannedRawArtifacts<'source> {
    pub tags: Vec<RawTag<'source>>,
    pub inline_fields: Vec<RawInlineFieldToken<'source>>,
    pub block_refs: Vec<RawBlockRef<'source>>,
}

/// Constructor for `ScannedRawArtifacts` from scanned artifacts.
impl<'source> ScannedRawArtifacts<'source> {
    pub(crate) fn from_scanned_artifacts(
        artifacts: Vec<ScannedArtifact<'source>>,
    ) -> Self {
        let capacity = artifacts.len();
        let mut raw = ScannedRawArtifacts {
            tags: Vec::with_capacity(capacity),
            inline_fields: Vec::with_capacity(capacity),
            block_refs: Vec::with_capacity(capacity),
        };
        for artifact in artifacts {
            match artifact {
                ScannedArtifact::Tag(tag) => raw.tags.push(tag),
                ScannedArtifact::InlineField(field) => {
                    raw.inline_fields.push(field);
                }
                ScannedArtifact::BlockRef(block_ref) => {
                    raw.block_refs.push(block_ref);
                }
            }
        }
        raw
    }
}

// ── Private implementation details ───────────────────────────────────────────

/// The positional state of the scanner within a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScanMode {
    /// Scanner is at the start of a line (after newline or at block start).
    /// List markers, tasks, and bare fields are only matched in this mode.
    AtLineStart,
    /// Scanner is within the body of a line.
    InBody,
}

trait BodyRule {
    fn try_scan<'source>(
        &self,
        scanner: &NoteScanner,
        cursor: &mut Cursor<'source>,
        artifacts: &mut Vec<ScannedArtifact<'source>>,
    ) -> Result<bool, NoteError>;
}

struct TagRule;

impl BodyRule for TagRule {
    fn try_scan<'source>(
        &self,
        _scanner: &NoteScanner,
        cursor: &mut Cursor<'source>,
        artifacts: &mut Vec<ScannedArtifact<'source>>,
    ) -> Result<bool, NoteError> {
        if matches!(cursor.peek_byte(), Some(b'#')) && !cursor.prev_alnum {
            if let Some(tag) = NoteScanner::scan_tag(cursor)? {
                artifacts.push(tag);
                return Ok(true);
            }
            cursor.advance(1)?;
            return Ok(true);
        }
        Ok(false)
    }
}

struct DelimitedFieldRule;

impl BodyRule for DelimitedFieldRule {
    fn try_scan<'source>(
        &self,
        _scanner: &NoteScanner,
        cursor: &mut Cursor<'source>,
        artifacts: &mut Vec<ScannedArtifact<'source>>,
    ) -> Result<bool, NoteError> {
        if matches!(cursor.peek_byte(), Some(b'[' | b'(')) {
            if let Some(field) = NoteScanner::scan_delimited_field(cursor)? {
                artifacts.push(field);
                return Ok(true);
            }
            cursor.advance(1)?;
            return Ok(true);
        }
        Ok(false)
    }
}

struct BlockRefRule;

impl BodyRule for BlockRefRule {
    fn try_scan<'source>(
        &self,
        _scanner: &NoteScanner,
        cursor: &mut Cursor<'source>,
        artifacts: &mut Vec<ScannedArtifact<'source>>,
    ) -> Result<bool, NoteError> {
        if matches!(cursor.peek_byte(), Some(b'^')) && !cursor.prev_alnum {
            if let Some(block_ref) = NoteScanner::scan_block_ref(cursor)? {
                artifacts.push(block_ref);
                return Ok(true);
            }
            cursor.advance(1)?;
            return Ok(true);
        }
        Ok(false)
    }
}

struct EmojiFieldRule;

impl BodyRule for EmojiFieldRule {
    fn try_scan<'source>(
        &self,
        scanner: &NoteScanner,
        cursor: &mut Cursor<'source>,
        artifacts: &mut Vec<ScannedArtifact<'source>>,
    ) -> Result<bool, NoteError> {
        let Some(ch) = cursor.rest.chars().next() else {
            return Ok(false);
        };
        if !scanner.emoji_markers.contains(&ch) {
            return Ok(false);
        }
        if let Some(field) = NoteScanner::scan_emoji_field(cursor)? {
            artifacts.push(field);
            return Ok(true);
        }
        cursor.advance(ch.len_utf8())?;
        Ok(true)
    }
}

trait LineStartRule {
    fn try_scan<'source>(
        &self,
        cursor: &mut Cursor<'source>,
        artifacts: &mut Vec<ScannedArtifact<'source>>,
    ) -> Result<bool, NoteError>;
}

struct BareFieldRule;

impl LineStartRule for BareFieldRule {
    fn try_scan<'source>(
        &self,
        cursor: &mut Cursor<'source>,
        artifacts: &mut Vec<ScannedArtifact<'source>>,
    ) -> Result<bool, NoteError> {
        if let Some(field) = NoteScanner::scan_bare_field(cursor)? {
            artifacts.push(field);
            return Ok(true);
        }
        Ok(false)
    }
}
