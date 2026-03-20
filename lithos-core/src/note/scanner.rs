//! Unified scanning utilities for Note metadata and structure.
//!
//! This module consolidates manual text-scanning logic for Obsidian-style
//! metadata (tags, inline fields, block references, and task markers) into
//! a single high-performance state machine.
//!
//! The scanner is designed to be:
//! 1. **Single-Pass**: Each byte of input is touched exactly once.
//! 2. **Zero-Copy**: All artifacts borrow directly from the source text.
//! 3. **Resumable**: Using the [`Cursor`] type, scanning can be paused and
//!    resumed across disjoint text segments while maintaining context.

use crate::note::{error::NoteError, position::SourceByteOffset};

/// A zero-copy artifact extracted from a text block.
///
/// This enum represents the various types of metadata that can be identified
/// within a markdown block. All variants carry a lifetime linked to the
/// original source string.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScannedArtifact<'source> {
    /// A hashtag (e.g., `#work/project`).
    ///
    /// Extracted when a `#` is encountered at a word boundary, followed by
    /// valid tag characters.
    Tag {
        /// The raw tag text including the leading `#`.
        text: &'source str,
        /// The absolute source position of the `#` character.
        position: SourceByteOffset,
    },
    /// An inline field (e.g., `[key:: value]` or `📅 2024-03-19`).
    ///
    /// Supports both the standard Obsidian `key:: value` syntax (bracketed or
    /// bare at line start) and the colon-less emoji field syntax.
    InlineField {
        /// The field key (e.g., "priority" or "📅").
        key: &'source str,
        /// The field value, trimmed of leading/trailing whitespace.
        value: &'source str,
        /// The absolute source position where the key starts.
        position: SourceByteOffset,
    },
    /// A block reference (e.g., `^block-id`).
    ///
    /// Extracted when a `^` is found at the very end of a block (followed only
    /// by optional whitespace).
    BlockRef {
        /// The unique block identifier, excluding the leading `^`.
        id: &'source str,
        /// The absolute source position of the `^` character.
        position: SourceByteOffset,
    },
    /// A task marker (e.g., the `x` in `- [x]`).
    ///
    /// Extracted from list items identifying the precise character used within
    /// the checkbox.
    TaskMarker {
        /// The character inside the checkbox.
        marker: char,
        /// The absolute source position of the marker character.
        position: SourceByteOffset,
    },
}

/// A cursor-based scanner for extracting metadata artifacts from markdown.
///
/// `NoteScanner` manages the rules for what constitutes valid metadata
/// (such as allowed emoji markers) while the [`Cursor`] tracks the progress
/// through the text.
#[derive(Debug, Clone)]
pub struct NoteScanner {
    /// Emoji markers used for identifying colon-less inline fields.
    emoji_markers: Box<[char]>,
}

impl Default for NoteScanner {
    /// Creates a `NoteScanner` with standard Obsidian emoji markers.
    ///
    /// Default markers include: 📅 (due), ✅ (done), ⏰ (reminder), 🛫 (start),
    /// and ⏳ (scheduled).
    #[inline]
    fn default() -> Self {
        Self {
            emoji_markers: vec![
                '\u{1f4c5}', // 📅
                '\u{2705}',  // ✅
                '\u{23f0}',  // ⏰
                '\u{1f6eb}', // 🛫
                '\u{23f3}',  // ⏳
            ]
            .into(),
        }
    }
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
    /// let scanner = NoteScanner::default();
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
        let mut artifacts = Vec::new();
        self.scan_cursor(&mut cursor, &mut artifacts)?;
        Ok(artifacts)
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

    /// Handles triggers that only occur at the beginning of a line (tasks and
    /// bare fields).
    fn handle_line_start<'source>(
        cursor: &mut Cursor<'source>,
        artifacts: &mut Vec<ScannedArtifact<'source>>,
    ) -> Result<(), NoteError> {
        cursor.skip_whitespace_on_line();

        let Some(first) = cursor.peek_byte() else {
            cursor.mode = ScanMode::InBody;
            return Ok(());
        };

        if first == b'\n' || first == b'\r' {
            cursor.mode = ScanMode::InBody;
            return Ok(());
        }

        // Try to match list prefix: -, *, +, or 1.
        if let Some(prefix_len) = Self::match_list_prefix(cursor) {
            cursor.advance(prefix_len)?;
            cursor.skip_whitespace_on_line();

            // Try to match checkbox: [x]
            if cursor.rest.starts_with('[')
                && let Some(marker_char) = cursor.rest.chars().nth(1)
                && cursor.rest.get(2..3) == Some("]")
            {
                let marker_pos = cursor.offset.add_offset(1)?;
                artifacts.push(ScannedArtifact::TaskMarker {
                    marker: marker_char,
                    position: marker_pos,
                });
                cursor.advance(3)?;
            }
        } else if let Some(field) = Self::scan_bare_field(cursor)? {
            artifacts.push(field);
        } else {
            // No trigger at line start
        }

        cursor.mode = ScanMode::InBody;
        Ok(())
    }

    /// Matches common list item prefixes including ordered list numbers.
    fn match_list_prefix(cursor: &Cursor<'_>) -> Option<usize> {
        let first = cursor.peek_byte()?;
        if matches!(first, b'-' | b'*' | b'+') {
            return Some(1);
        }
        if first.is_ascii_digit() {
            let bytes = cursor.rest.as_bytes();
            let mut idx = 0usize;
            while let Some(&b) = bytes.get(idx)
                && b.is_ascii_digit()
            {
                idx = idx.saturating_add(1);
            }
            if let Some(&b) = bytes.get(idx)
                && matches!(b, b'.' | b')')
            {
                return Some(idx.saturating_add(1));
            }
        }
        None
    }

    /// Main dispatch loop for artifacts that can occur anywhere in a block.
    fn handle_body<'source>(
        &self,
        cursor: &mut Cursor<'source>,
        artifacts: &mut Vec<ScannedArtifact<'source>>,
    ) -> Result<(), NoteError> {
        match cursor.peek_byte() {
            Some(b'#') if !cursor.prev_alnum => {
                if let Some(tag) = Self::scan_tag(cursor)? {
                    artifacts.push(tag);
                } else {
                    cursor.advance(1)?;
                }
            }
            Some(b'[' | b'(') => {
                if let Some(field) = Self::scan_delimited_field(cursor)? {
                    artifacts.push(field);
                } else {
                    cursor.advance(1)?;
                }
            }
            Some(b'^') if !cursor.prev_alnum => {
                if let Some(block_ref) = Self::scan_block_ref(cursor)? {
                    artifacts.push(block_ref);
                } else {
                    cursor.advance(1)?;
                }
            }
            Some(b'\n' | b'\r') => {
                cursor.advance(1)?;
                cursor.mode = ScanMode::AtLineStart;
                cursor.prev_alnum = false;
            }
            Some(b) if b < 128 => {
                cursor.prev_alnum = b.is_ascii_alphanumeric();
                cursor.advance(1)?;
            }
            Some(_) => {
                // Multi-byte Unicode character
                let ch = cursor.rest.chars().next().unwrap_or('\0');
                if self.emoji_markers.contains(&ch) {
                    if let Some(field) = Self::scan_emoji_field(cursor)? {
                        artifacts.push(field);
                    } else {
                        cursor.advance(ch.len_utf8())?;
                    }
                } else {
                    cursor.prev_alnum = ch.is_alphanumeric();
                    cursor.advance(ch.len_utf8())?;
                }
            }
            None => {}
        }
        Ok(())
    }

    /// Scans for a tag artifact starting at the current cursor position.
    fn scan_tag<'source>(
        cursor: &mut Cursor<'source>,
    ) -> Result<Option<ScannedArtifact<'source>>, NoteError> {
        let bytes = cursor.rest.as_bytes();
        let mut idx = 1usize; // skip '#'
        while let Some(&b) = bytes.get(idx) {
            if b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'/') {
                idx = idx.saturating_add(1);
            } else if b >= 128 {
                // Potential Unicode alphanumeric in tag
                if let Some(ch) =
                    cursor.rest.get(idx..).and_then(|s| s.chars().next())
                    && ch.is_alphanumeric()
                {
                    idx = idx.saturating_add(ch.len_utf8());
                    continue;
                }
                break;
            } else {
                break;
            }
        }

        if idx > 1 {
            let text = cursor.rest.get(..idx).unwrap_or("");
            let position = cursor.offset;
            cursor.advance(idx)?;
            Ok(Some(ScannedArtifact::Tag {
                text,
                position,
            }))
        } else {
            Ok(None)
        }
    }

    /// Scans for a bracketed or parenthesized inline field.
    fn scan_delimited_field<'source>(
        cursor: &mut Cursor<'source>,
    ) -> Result<Option<ScannedArtifact<'source>>, NoteError> {
        let open = cursor.peek_byte().unwrap_or(b'[');
        let close = if open == b'[' {
            b']'
        } else {
            b')'
        };

        if let Some(close_idx) = cursor.rest.find(char::from(close)) {
            let inner = cursor.rest.get(1..close_idx).unwrap_or("");
            if let Some((key, value)) = inner.split_once("::") {
                let key_trimmed = key.trim();
                let value_trimmed = value.trim();
                if !key_trimmed.is_empty() && !value_trimmed.is_empty() {
                    let key_start_offset =
                        inner.find(key_trimmed).unwrap_or(0).saturating_add(1);
                    let position =
                        cursor.offset.add_offset(key_start_offset)?;
                    let artifact = ScannedArtifact::InlineField {
                        key: key_trimmed,
                        value: value_trimmed,
                        position,
                    };
                    cursor.advance(close_idx.saturating_add(1))?;
                    return Ok(Some(artifact));
                }
            }
        }
        Ok(None)
    }

    /// Scans for a colon-less emoji field.
    fn scan_emoji_field<'source>(
        cursor: &mut Cursor<'source>,
    ) -> Result<Option<ScannedArtifact<'source>>, NoteError> {
        let Some(emoji_ch) = cursor.rest.chars().next() else {
            return Ok(None);
        };
        let emoji_len = emoji_ch.len_utf8();
        let position = cursor.offset;

        let Some(mut after_emoji) = cursor.rest.get(emoji_len..) else {
            return Ok(None);
        };
        let mut consumed = emoji_len;

        // Skip leading whitespace after emoji
        while let Some(ch) = after_emoji.chars().next()
            && ch.is_whitespace()
            && ch != '\n'
            && ch != '\r'
        {
            let len = ch.len_utf8();
            after_emoji = after_emoji.get(len..).unwrap_or("");
            consumed = consumed.saturating_add(len);
        }

        // Capture until next whitespace or end of line/block
        let mut val_len = 0usize;
        for ch in after_emoji.chars() {
            if ch.is_whitespace() {
                break;
            }
            val_len = val_len.saturating_add(ch.len_utf8());
        }

        if val_len > 0 {
            let key = cursor.rest.get(..emoji_len).unwrap_or("");
            let value = after_emoji.get(..val_len).unwrap_or("");
            cursor.advance(consumed.saturating_add(val_len))?;
            Ok(Some(ScannedArtifact::InlineField {
                key,
                value,
                position,
            }))
        } else {
            Ok(None)
        }
    }

    /// Scans for a bare `key:: value` pair at line start.
    fn scan_bare_field<'source>(
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
                let artifact = ScannedArtifact::InlineField {
                    key,
                    value,
                    position: cursor.offset,
                };
                cursor.advance(
                    key_len.saturating_add(2).saturating_add(val_len),
                )?;
                return Ok(Some(artifact));
            }
        }

        Ok(None)
    }

    /// Scans for an Obsidian block reference at the current position.
    fn scan_block_ref<'source>(
        cursor: &mut Cursor<'source>,
    ) -> Result<Option<ScannedArtifact<'source>>, NoteError> {
        let bytes = cursor.rest.as_bytes();
        let mut len = 1usize; // skip '^'
        while let Some(&b) = bytes.get(len) {
            if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_') {
                len = len.saturating_add(1);
            } else {
                break;
            }
        }

        if len > 1 {
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
            let position = cursor.offset;
            cursor.advance(len.saturating_add(tail_len))?;
            Ok(Some(ScannedArtifact::BlockRef {
                id,
                position,
            }))
        } else {
            Ok(None)
        }
    }
}

/// Internal state for the metadata scanner.
///
/// `Cursor` tracks the remaining text segment being scanned, the current
/// absolute byte offset, and contextual flags like whether the scanner is
/// currently at the start of a line.
#[derive(Debug, Clone)]
pub struct Cursor<'source> {
    /// The remaining slice of text to be processed.
    rest: &'source str,
    /// The absolute source byte offset of the `rest` slice.
    offset: SourceByteOffset,
    /// The current state of the scanner (`LineStart` vs Body).
    mode: ScanMode,
    /// True if the previous character was alphanumeric.
    prev_alnum: bool,
}

impl<'source> Cursor<'source> {
    /// Create a new cursor at the start of a text block.
    #[inline]
    #[must_use]
    pub fn new(text: &'source str, base_offset: SourceByteOffset) -> Self {
        Self {
            rest: text,
            offset: base_offset,
            mode: ScanMode::AtLineStart,
            prev_alnum: false,
        }
    }

    /// Reset the cursor to point to new text, maintaining contextual flags.
    ///
    /// This allows resuming a scan across disjoint source segments (e.g., when
    /// standard markdown artifacts like links are skipped).
    #[inline]
    pub fn reset(&mut self, text: &'source str, base_offset: SourceByteOffset) {
        self.rest = text;
        self.offset = base_offset;
    }

    /// Returns `true` if the cursor has reached the end of the input text.
    #[inline]
    #[must_use]
    pub fn is_eof(&self) -> bool {
        self.rest.is_empty()
    }

    /// Peeks at the first byte of the remaining text without consuming it.
    #[inline]
    #[must_use]
    fn peek_byte(&self) -> Option<u8> {
        self.rest.as_bytes().first().copied()
    }

    /// Advances the cursor by the specified number of bytes.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if the resulting offset exceeds supported bounds.
    #[inline]
    fn advance(&mut self, bytes: usize) -> Result<(), NoteError> {
        self.rest = self.rest.get(bytes..).unwrap_or("");
        self.offset = self.offset.add_offset(bytes)?;
        Ok(())
    }

    /// Consumes whitespace characters from the current line.
    fn skip_whitespace_on_line(&mut self) {
        let bytes = self.rest.as_bytes();
        let mut idx = 0usize;
        while let Some(&b) = bytes.get(idx) {
            if b == b' ' || b == b'\t' {
                idx = idx.saturating_add(1);
            } else {
                break;
            }
        }
        if idx > 0 {
            let _result = self.advance(idx);
        }
    }
}

/// The positional state of the scanner within a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScanMode {
    /// Scanner is at the start of a line (after newline or at block start).
    /// List markers, tasks, and bare fields are only matched in this mode.
    AtLineStart,
    /// Scanner is within the body of a line.
    InBody,
}

#[cfg(test)]
mod tests {
    mod block_ref {
        use rstest::rstest;

        use super::super::*;

        #[rstest]
        #[case::at_end("Block text ^id", "id")]
        #[case::with_whitespace("Block text ^id  ", "id")]
        #[case::with_underscore("Block text ^id_123", "id_123")]
        #[case::with_hyphen("Block text ^id-123", "id-123")]
        fn should_extract_valid_block_refs(
            #[case] text: &str,
            #[case] expected_id: &str,
        ) {
            let scanner = NoteScanner::default();
            let artifacts =
                scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();
            assert_eq!(artifacts.len(), 1);
            let first = artifacts.first().expect("Should have one artifact");
            assert!(
                matches!(first, ScannedArtifact::BlockRef { id, .. } if id == &expected_id),
                "Expected BlockRef with id '{expected_id}', got {first:?}"
            );
        }

        #[rstest]
        #[case::mid_block("Text ^id summary")]
        #[case::no_space("Text^id")]
        #[case::preceded_by_alnum("word^id")]
        fn should_ignore_invalid_block_refs(#[case] text: &str) {
            let scanner = NoteScanner::default();
            let artifacts =
                scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();
            assert!(artifacts.is_empty(), "Failed for: {text}");
        }
    }

    mod constructor {
        use super::super::*;

        #[test]
        fn should_create_with_default_emojis() {
            let scanner = NoteScanner::default();
            assert!(!scanner.emoji_markers.is_empty());
            assert!(scanner.emoji_markers.contains(&'\u{1f4c5}'));
        }

        #[test]
        fn should_create_with_custom_emojis() {
            let scanner = NoteScanner::new(vec!['\u{2b50}']);
            assert_eq!(scanner.emoji_markers.len(), 1);
            assert!(scanner.emoji_markers.contains(&'\u{2b50}'));
        }
    }

    mod cursor {
        use super::super::*;

        #[test]
        fn should_handle_unicode_advancement() {
            let text = "\u{1f980}#rust";
            let mut cursor = Cursor::new(text, SourceByteOffset::new(0));
            let scanner = NoteScanner::default();
            let mut artifacts = Vec::new();

            scanner.scan_cursor(&mut cursor, &mut artifacts).unwrap();
            assert_eq!(artifacts.len(), 1);
            let first = artifacts.first().expect("Should have one artifact");
            assert!(
                matches!(first, ScannedArtifact::Tag { text: t, position, .. } if t == &"#rust" && u32::from(*position) == 4),
                "Expected Tag #rust at position 4, got {first:?}"
            );
        }

        #[test]
        fn should_handle_line_start_mode_transitions() {
            let text = "Line 1\n#tag";
            let mut cursor = Cursor::new(text, SourceByteOffset::new(0));
            assert_eq!(cursor.mode, ScanMode::AtLineStart);

            let scanner = NoteScanner::default();
            let mut artifacts = Vec::new();
            scanner.scan_cursor(&mut cursor, &mut artifacts).unwrap();

            assert_eq!(artifacts.len(), 1);
            assert!(matches!(
                artifacts.first(),
                Some(ScannedArtifact::Tag { .. })
            ));
        }
    }

    mod inline_field {
        use rstest::rstest;

        use super::super::*;

        #[rstest]
        #[case::bracketed("[key:: val]", "key", "val")]
        #[case::parenthesized("(key:: val)", "key", "val")]
        #[case::nested_syntax("[key:: nested::val]", "key", "nested::val")]
        #[case::no_spaces("[key::val]", "key", "val")]
        fn should_extract_delimited_fields(
            #[case] text: &str,
            #[case] key: &str,
            #[case] val: &str,
        ) {
            let scanner = NoteScanner::default();
            let artifacts =
                scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();
            assert_eq!(artifacts.len(), 1);
            let first = artifacts.first().expect("Should have one artifact");
            assert!(
                matches!(first, ScannedArtifact::InlineField { key: k, value: v, .. } if k == &key && v == &val),
                "Expected InlineField '{key}: {val}', got {first:?}"
            );
        }

        #[test]
        fn should_extract_emoji_field() {
            let scanner = NoteScanner::default();
            let artifacts = scanner
                .scan_block("\u{1f4c5} 2024-03-20", SourceByteOffset::new(0))
                .unwrap();
            assert_eq!(artifacts.len(), 1);
            let first = artifacts.first().expect("Should have one artifact");
            assert!(
                matches!(first, ScannedArtifact::InlineField { key, value, .. } if key == &"\u{1f4c5}" && value == &"2024-03-20"),
                "Expected InlineField with emoji, got {first:?}"
            );
        }

        #[test]
        fn should_extract_bare_field_at_line_start() {
            let scanner = NoteScanner::default();
            let text = "bare_key:: bare_value";
            let artifacts =
                scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();
            assert_eq!(artifacts.len(), 1);
            let first = artifacts.first().expect("Should have one artifact");
            assert!(
                matches!(first, ScannedArtifact::InlineField { key, value, .. } if key == &"bare_key" && value == &"bare_value"),
                "Expected bare InlineField, got {first:?}"
            );
        }

        #[test]
        fn should_ignore_bare_field_in_body() {
            let scanner = NoteScanner::default();
            let text = "Not a bare_key:: bare_value";
            let artifacts =
                scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();
            assert!(artifacts.is_empty());
        }
    }

    mod scan_block {
        use super::super::*;

        #[test]
        fn should_extract_all_artifact_types() {
            let scanner = NoteScanner::default();
            let text = "- [x] #task [priority:: high] \u{1f4c5} \
                        2024-03-20\nSummary ^ref-id";
            let artifacts =
                scanner.scan_block(text, SourceByteOffset::new(100)).unwrap();

            assert_eq!(artifacts.len(), 5);
            assert!(matches!(
                artifacts.first(),
                Some(&ScannedArtifact::TaskMarker {
                    marker: 'x',
                    ..
                })
            ));
            assert!(matches!(
                artifacts.get(1),
                Some(&ScannedArtifact::Tag {
                    text: "#task",
                    ..
                })
            ));
            assert!(matches!(
                artifacts.get(2),
                Some(&ScannedArtifact::InlineField {
                    key: "priority",
                    value: "high",
                    ..
                })
            ));
            assert!(matches!(
                artifacts.get(3),
                Some(&ScannedArtifact::InlineField {
                    key: "📅",
                    value: "2024-03-20",
                    ..
                })
            ));
            assert!(matches!(
                artifacts.get(4),
                Some(&ScannedArtifact::BlockRef {
                    id: "ref-id",
                    ..
                })
            ));
        }

        #[test]
        fn should_handle_empty_input() {
            let scanner = NoteScanner::default();
            let artifacts =
                scanner.scan_block("", SourceByteOffset::new(0)).unwrap();
            assert!(artifacts.is_empty());
        }
    }

    mod scan_cursor {
        use super::super::*;

        #[test]
        fn should_maintain_context_across_segments() {
            let scanner = NoteScanner::default();
            let mut cursor =
                Cursor::new("Segment 1 #tag1 ", SourceByteOffset::new(0));
            let mut artifacts = Vec::new();

            scanner.scan_cursor(&mut cursor, &mut artifacts).unwrap();
            assert_eq!(artifacts.len(), 1);
            assert!(matches!(
                artifacts.first(),
                Some(&ScannedArtifact::Tag {
                    text: "#tag1",
                    ..
                })
            ));

            // Resume with a new segment starting immediately after an
            // alphanumeric char
            cursor.reset("#not-a-tag", SourceByteOffset::new(16));
            cursor.prev_alnum = true; // Simulating segment boundary

            scanner.scan_cursor(&mut cursor, &mut artifacts).unwrap();
            assert_eq!(
                artifacts.len(),
                1,
                "Should not identify #not-a-tag when preceded by alnum"
            );
        }
    }

    mod tag {
        use rstest::rstest;

        use super::super::*;

        #[rstest]
        #[case::simple("#tag", 1)]
        #[case::hierarchical("#work/project", 1)]
        #[case::with_underscore("#my_tag", 1)]
        #[case::with_hyphen("#tag-123", 1)]
        #[case::unicode("#\u{03b1}\u{03b2}\u{03b3}", 1)]
        #[case::multiple("#one #two", 2)]
        fn should_extract_valid_tags(#[case] text: &str, #[case] count: usize) {
            let scanner = NoteScanner::default();
            let artifacts =
                scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();
            assert_eq!(artifacts.len(), count);
            for a in artifacts {
                assert!(matches!(a, ScannedArtifact::Tag { .. }));
            }
        }

        #[rstest]
        #[case::preceded_by_alnum("word#tag")]
        #[case::single_hash("#")]
        #[case::empty_hash("# ")]
        fn should_ignore_invalid_tags(#[case] text: &str) {
            let scanner = NoteScanner::default();
            let artifacts =
                scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();
            assert!(artifacts.is_empty(), "Failed for: {text}");
        }

        #[test]
        fn should_track_absolute_position() {
            let scanner = NoteScanner::default();
            let artifacts = scanner
                .scan_block("  #tag", SourceByteOffset::new(10))
                .unwrap();
            let first = artifacts.first().expect("Should have one artifact");
            assert!(
                matches!(first, ScannedArtifact::Tag { position, .. } if u32::from(*position) == 12),
                "Expected Tag at position 12, got {first:?}"
            );
        }
    }

    mod task_marker {
        use rstest::rstest;

        use super::super::*;

        #[rstest]
        #[case::hyphen("- [ ] ", ' ')]
        #[case::asterisk("* [x] ", 'x')]
        #[case::plus("+ [X] ", 'X')]
        #[case::ordered("1. [/] ", '/')]
        #[case::ordered_paren("1) [!] ", '!')]
        #[case::indented("  - [-] ", '-')]
        fn should_extract_task_markers(
            #[case] text: &str,
            #[case] expected: char,
        ) {
            let scanner = NoteScanner::default();
            let artifacts =
                scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();
            assert!(
                matches!(
                    artifacts.first(),
                    Some(&ScannedArtifact::TaskMarker { marker, .. }) if marker == expected
                ),
                "Failed for: {text}"
            );
        }

        #[rstest]
        #[case::no_list_prefix("[x]")]
        #[case::missing_bracket("- x]")]
        #[case::malformed_checkbox("- []")]
        fn should_ignore_malformed_tasks(#[case] text: &str) {
            let scanner = NoteScanner::default();
            let artifacts =
                scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();
            let has_task = artifacts
                .iter()
                .any(|a| matches!(a, ScannedArtifact::TaskMarker { .. }));
            assert!(!has_task, "Should not have task for: {text}");
        }
    }
}
