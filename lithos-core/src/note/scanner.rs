//! Unified scanning utilities for Note metadata and structure.
//!
//! This module consolidates all manual text-scanning logic (tags, inline
//! fields, block references, and task markers) into a single high-performance
//! state machine. It uses a cursor-based, single-pass approach to identify
//! Obsidian-style metadata that isn't natively handled by standard markdown
//! parsers.
//!
//! The primary entry point is [`NoteScanner`], which processes text blocks
//! and yields zero-copy [`ScannedArtifact`]s.

use crate::note::{error::NoteError, position::SourceByteOffset};

/// A zero-copy artifact extracted from a text block.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScannedArtifact<'source> {
    /// A hashtag (e.g., `#work/project`).
    Tag {
        /// The raw tag text including the `#`.
        text: &'source str,
        /// Source position of the `#`.
        position: SourceByteOffset,
    },
    /// An inline field (e.g., `[key:: value]` or `📅 2024-03-19`).
    InlineField {
        /// The field key.
        key: &'source str,
        /// The field value.
        value: &'source str,
        /// Source position of the key start.
        position: SourceByteOffset,
    },
    /// A block reference (e.g., `^block-id`).
    BlockRef {
        /// The block identifier excluding the `^`.
        id: &'source str,
        /// Source position of the `^`.
        position: SourceByteOffset,
    },
    /// A task marker (e.g., the `x` in `- [x]`).
    TaskMarker {
        /// The character inside the checkbox.
        marker: char,
        /// Source position of the marker character.
        position: SourceByteOffset,
    },
}

/// A cursor-based scanner for extracting metadata artifacts from markdown.
#[derive(Debug, Clone)]
pub struct NoteScanner {
    /// Emoji markers used for colon-less inline fields.
    emoji_markers: Box<[char]>,
}

impl Default for NoteScanner {
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
    /// Create a new scanner with the provided emoji markers.
    #[inline]
    #[must_use]
    pub fn new<T: Into<Box<[char]>>>(emoji_markers: T) -> Self {
        Self {
            emoji_markers: emoji_markers.into(),
        }
    }

    /// Scans a block of text for all metadata artifacts in a single pass.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if position mapping fails.
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

    /// Continues scanning from a provided cursor state.
    ///
    /// This is useful for scanning disjoint text segments while maintaining
    /// line-start and alphanumeric context.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if position mapping fails.
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

/// Internal state for the scanner.
#[derive(Debug, Clone)]
pub struct Cursor<'source> {
    rest: &'source str,
    offset: SourceByteOffset,
    mode: ScanMode,
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

    /// Reset the cursor to point to new text, maintaining line-start and
    /// alphanumeric context.
    #[inline]
    pub fn reset(&mut self, text: &'source str, base_offset: SourceByteOffset) {
        self.rest = text;
        self.offset = base_offset;
    }

    #[inline]
    fn is_eof(&self) -> bool {
        self.rest.is_empty()
    }

    #[inline]
    fn peek_byte(&self) -> Option<u8> {
        self.rest.as_bytes().first().copied()
    }

    #[inline]
    fn advance(&mut self, bytes: usize) -> Result<(), NoteError> {
        self.rest = self.rest.get(bytes..).unwrap_or("");
        self.offset = self.offset.add_offset(bytes)?;
        Ok(())
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScanMode {
    AtLineStart,
    InBody,
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn should_scan_complex_block() {
        let scanner = NoteScanner::new(vec!['\u{1f4c5}', '\u{2705}']);
        let text =
            "- [x] #task [priority:: high] \u{1f4c5} 2023-04-09\nNext line ^id";
        let artifacts =
            scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();

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
                key: "\u{1f4c5}",
                value: "2023-04-09",
                ..
            })
        ));
        assert!(matches!(
            artifacts.get(4),
            Some(&ScannedArtifact::BlockRef {
                id: "id",
                ..
            })
        ));
    }

    #[rstest]
    #[case("- [ ] ", ' ')]
    #[case("* [x] ", 'x')]
    #[case("1. [/] ", '/')]
    #[case("  - [-] ", '-')]
    fn should_scan_task_markers(#[case] text: &str, #[case] expected: char) {
        let scanner = NoteScanner::default();
        let artifacts =
            scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();
        assert!(
            matches!(
                artifacts.first(),
                Some(ScannedArtifact::TaskMarker { marker, .. }) if *marker == expected
            ),
            "Expected TaskMarker with {expected}, got {artifacts:?}"
        );
    }

    #[test]
    fn should_handle_emoji_fields_without_colons() {
        let scanner = NoteScanner::new(vec!['\u{1f4c5}', '\u{2705}']);
        let text = "Completed \u{1f4c5} 2023-04-09 \u{2705} 2023-04-10";
        let artifacts =
            scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();

        assert_eq!(artifacts.len(), 2);
        if let Some(&ScannedArtifact::InlineField {
            key,
            value,
            ..
        }) = artifacts.first()
        {
            assert_eq!(key, "\u{1f4c5}");
            assert_eq!(value, "2023-04-09");
        }
        if let Some(&ScannedArtifact::InlineField {
            key,
            value,
            ..
        }) = artifacts.get(1)
        {
            assert_eq!(key, "\u{2705}");
            assert_eq!(value, "2023-04-10");
        }
    }

    #[test]
    fn should_ignore_tags_preceded_by_alnum() {
        let scanner = NoteScanner::default();
        let text = "word#tag";
        let artifacts =
            scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();
        assert!(artifacts.is_empty());
    }

    #[test]
    fn should_handle_failed_tag_followed_by_field() {
        let scanner = NoteScanner::default();
        let text = "#[key:: val]";
        let artifacts =
            scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();
        assert_eq!(artifacts.len(), 1);
        assert!(matches!(
            artifacts.first(),
            Some(&ScannedArtifact::InlineField {
                key: "key",
                ..
            })
        ));
    }

    #[test]
    fn should_handle_block_ref_at_end_of_line() {
        let scanner = NoteScanner::default();
        let text = "Important point ^my-id\nNext line";
        let artifacts =
            scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();
        assert_eq!(artifacts.len(), 1);
        assert!(
            matches!(artifacts.first(), Some(ScannedArtifact::BlockRef { id, .. }) if *id == "my-id"),
            "Expected BlockRef with id 'my-id', got {artifacts:?}"
        );
    }

    #[test]
    fn should_ignore_block_ref_in_middle_of_text() {
        let scanner = NoteScanner::default();
        let text = "Important ^my-id point";
        let artifacts =
            scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();
        assert!(artifacts.is_empty());
    }

    #[test]
    fn should_scan_bare_fields_at_line_start() {
        let scanner = NoteScanner::default();
        let text = "key:: value\n- List item\nnext_key:: next_value";
        let artifacts =
            scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();
        assert_eq!(artifacts.len(), 2);
        assert!(matches!(
            artifacts.first(),
            Some(&ScannedArtifact::InlineField {
                key: "key",
                value: "value",
                ..
            })
        ));
        assert!(matches!(
            artifacts.get(1),
            Some(&ScannedArtifact::InlineField {
                key: "next_key",
                value: "next_value",
                ..
            })
        ));
    }

    #[test]
    fn should_handle_bare_emoji_field_at_end_of_text() {
        let scanner = NoteScanner::default();
        let text = "Task completed \u{2705} 2023-04-09";
        let artifacts =
            scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();
        assert_eq!(artifacts.len(), 1);
        if let Some(&ScannedArtifact::InlineField {
            key,
            value,
            ..
        }) = artifacts.first()
        {
            assert_eq!(key, "\u{2705}");
            assert_eq!(value, "2023-04-09");
        }
    }
}
