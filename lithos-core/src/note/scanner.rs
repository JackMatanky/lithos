//! Unified scanning utilities for Note metadata and structure.
//!
//! This module consolidates all manual text-scanning logic (tags, inline
//! fields, block references) into a single boundary, reducing redundant
//! passes over markdown content and ensuring heuristic consistency.

use crate::note::{
    error::{NoteError, NoteIngestError},
    position::SourceByteOffset,
    raw::{RawBlockRef, RawInlineField, RawReferenceLink, RawTag, RawTaskKind},
};

/// A unified result from the scanning process.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ScanArtifact {
    /// A hashtag (e.g., `#work/project`).
    Tag(RawTag),
    /// An inline field (e.g., `[key:: value]`).
    InlineField(RawInlineField),
    /// A block reference (e.g., `^block-id`).
    BlockRef(RawBlockRef),
    /// A reference link definition (e.g., `[label]: target`).
    ReferenceLink(RawReferenceLink),
}

/// The syntax style used for an inline field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InlineFieldDelimiter {
    /// `[key:: value]`.
    Brackets,
    /// `(key:: value)`.
    Parentheses,
    /// `key:: value`.
    Bare,
    /// `📅 2024-03-18` (Emoji-prefixed).
    Emoji,
}

impl InlineFieldDelimiter {
    /// Returns the character pair for delimited fields.
    #[inline]
    #[must_use]
    pub const fn pair(&self) -> Option<(u8, u8)> {
        match *self {
            Self::Brackets => Some((b'[', b']')),
            Self::Parentheses => Some((b'(', b')')),
            Self::Bare | Self::Emoji => None,
        }
    }
}

/// Specialized scanner for extracting metadata artifacts from markdown.
#[derive(Debug, Clone, Default)]
pub struct NoteScanner {
    /// Emoji markers used for date/status fields.
    emoji_markers: Box<[char]>,
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

    /// Performs an optimized binary search to map a local text offset to a
    /// source byte position.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Structure`] if the offset is out of range.
    #[inline]
    pub fn map_position(
        segments: &[(usize, SourceByteOffset)],
        offset: usize,
    ) -> Result<SourceByteOffset, NoteError> {
        if segments.is_empty() {
            return SourceByteOffset::try_from_usize(offset)
                .map_err(|_error| NoteError::Structure("offset out of range"));
        }

        // Use binary search instead of linear scan for O(log n) performance
        let idx = segments
            .binary_search_by_key(&offset, |&(start, _)| start)
            .unwrap_or_else(|i| i.saturating_sub(1));

        let &(segment_start, segment_pos) = segments
            .get(idx)
            .ok_or(NoteError::Structure("inline field offset out of range"))?;

        let delta = offset.saturating_sub(segment_start);
        let base =
            usize::try_from(u32::from(segment_pos)).map_err(|_error| {
                NoteError::Structure("inline field offset out of range")
            })?;

        SourceByteOffset::try_from_usize(base.saturating_add(delta))
    }

    /// Scans a block of text for tags and inline fields.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if position mapping fails.
    #[inline]
    pub fn scan_block(
        &self,
        text: &str,
        segments: &[(usize, SourceByteOffset)],
    ) -> Result<Vec<ScanArtifact>, NoteError> {
        let mut artifacts = Vec::new();

        // 1. Scan for tags
        Self::scan_tags(text, segments, &mut artifacts)?;

        // 2. Scan for inline fields (Delimited and Bare)
        self.scan_inline_fields(text, segments, &mut artifacts)?;

        Ok(artifacts)
    }

    fn scan_tags(
        text: &str,
        segments: &[(usize, SourceByteOffset)],
        out: &mut Vec<ScanArtifact>,
    ) -> Result<(), NoteError> {
        let mut chars = text.char_indices().peekable();
        let mut prev_is_alnum = false;

        while let Some((start_idx, ch)) = chars.next() {
            if ch != '#' || prev_is_alnum {
                prev_is_alnum = ch.is_alphanumeric();
                continue;
            }

            let Some(mut end_idx) = start_idx.checked_add(ch.len_utf8()) else {
                prev_is_alnum = ch.is_alphanumeric();
                continue;
            };

            while let Some(&(next_idx, next_ch)) = chars.peek() {
                if !(next_ch.is_alphanumeric()
                    || matches!(next_ch, '_' | '-' | '/'))
                {
                    break;
                }
                chars.next();
                let Some(updated) = next_idx.checked_add(next_ch.len_utf8())
                else {
                    break;
                };
                end_idx = updated;
            }

            let Some(raw) = text.get(start_idx..end_idx) else {
                prev_is_alnum = ch.is_alphanumeric();
                continue;
            };

            if raw.len() > 1 {
                let position = Self::map_position(segments, start_idx)?;
                out.push(ScanArtifact::Tag(RawTag::new(raw.into(), position)));
            }

            prev_is_alnum =
                raw.chars().last().is_some_and(char::is_alphanumeric);
        }
        Ok(())
    }

    fn scan_inline_fields(
        &self,
        text: &str,
        segments: &[(usize, SourceByteOffset)],
        out: &mut Vec<ScanArtifact>,
    ) -> Result<(), NoteError> {
        if !text.contains("::") && self.emoji_markers.is_empty() {
            return Ok(());
        }

        let mut bracket_spans = Vec::new();

        // 1. Delimited (Brackets)
        Self::scan_delimited_fields(
            text,
            b'[',
            b']',
            segments,
            &mut bracket_spans,
            out,
        )?;

        // 2. Delimited (Parentheses)
        Self::scan_delimited_fields(
            text,
            b'(',
            b')',
            segments,
            &mut bracket_spans,
            out,
        )?;

        // 3. Emoji fields
        self.scan_emoji_fields(text, segments, out)?;

        // 4. Bare fields (on each line, if not inside brackets)
        let mut offset = 0usize;
        for line in text.split_inclusive(['\n', '\r']) {
            Self::scan_bare_fields(
                line,
                offset,
                &bracket_spans,
                segments,
                out,
            )?;
            offset = offset.saturating_add(line.len());
        }

        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Internal multi-pass scanning logic"
    )]
    fn scan_delimited_fields(
        text: &str,
        open_delim: u8,
        close_delim: u8,
        segments: &[(usize, SourceByteOffset)],
        bracket_spans: &mut Vec<(usize, usize)>,
        out: &mut Vec<ScanArtifact>,
    ) -> Result<(), NoteError> {
        let bytes = text.as_bytes();
        let mut cursor = 0;
        while let Some(open_rel) = bytes
            .get(cursor..)
            .and_then(|slice| slice.iter().position(|&b| b == open_delim))
        {
            let open = cursor.saturating_add(open_rel);
            let after_open = open.saturating_add(1);
            let Some(close_rel) = bytes
                .get(after_open..)
                .and_then(|slice| slice.iter().position(|&b| b == close_delim))
            else {
                break;
            };
            let close = after_open.saturating_add(close_rel);
            let end = close.saturating_add(1);
            let Some(inner) = text.get(after_open..close) else {
                cursor = end;
                continue;
            };

            if let Some((key, value)) = inner.split_once("::") {
                let key_trimmed = key.trim();
                let value_trimmed = value.trim();
                if !key_trimmed.is_empty() && !value_trimmed.is_empty() {
                    let key_start = key
                        .find(key_trimmed)
                        .unwrap_or(0)
                        .saturating_add(after_open);
                    bracket_spans.push((open, end));
                    let position = Self::map_position(segments, key_start)?;
                    out.push(ScanArtifact::InlineField(RawInlineField::new(
                        key_trimmed.into(),
                        value_trimmed.into(),
                        position,
                    )));
                }
            }
            cursor = end;
        }
        Ok(())
    }

    fn scan_emoji_fields(
        &self,
        text: &str,
        segments: &[(usize, SourceByteOffset)],
        out: &mut Vec<ScanArtifact>,
    ) -> Result<(), NoteError> {
        if self.emoji_markers.is_empty() {
            return Ok(());
        }
        for (idx, ch) in text.char_indices() {
            if !self.emoji_markers.contains(&ch) {
                continue;
            }
            let value_start = idx.saturating_add(ch.len_utf8());
            let Some(tail) = text.get(value_start..) else {
                continue;
            };
            let Some(value) = tail.split_whitespace().next() else {
                continue;
            };
            if value.is_empty() {
                continue;
            }
            let mut buffer = [0u8; 4];
            let key = ch.encode_utf8(&mut buffer);
            let position = Self::map_position(segments, idx)?;
            out.push(ScanArtifact::InlineField(RawInlineField::new(
                key.into(),
                value.into(),
                position,
            )));
        }
        Ok(())
    }

    fn scan_bare_fields(
        line: &str,
        line_offset: usize,
        bracket_spans: &[(usize, usize)],
        segments: &[(usize, SourceByteOffset)],
        out: &mut Vec<ScanArtifact>,
    ) -> Result<(), NoteError> {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        let Some((key, value)) = trimmed.split_once("::") else {
            return Ok(());
        };
        let key_trimmed = key.trim();
        let value_trimmed = value.trim();
        if key_trimmed.is_empty() || value_trimmed.is_empty() {
            return Ok(());
        }

        // Bare fields must have "sane" keys to avoid false positives with
        // markdown syntax (like [ or #) or long sentences.
        // We reject spaces in bare keys to keep it unambiguous.
        if !key_trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Ok(());
        }

        let key_start =
            trimmed.find(key_trimmed).unwrap_or(0).saturating_add(line_offset);
        let is_bracketed = bracket_spans
            .iter()
            .any(|&(start, end)| key_start >= start && key_start < end);

        if !is_bracketed {
            let position = Self::map_position(segments, key_start)?;
            out.push(ScanArtifact::InlineField(RawInlineField::new(
                key_trimmed.into(),
                value_trimmed.into(),
                position,
            )));
        }
        Ok(())
    }

    /// Scans the entire document for line-based artifacts (`BlockRefs` and
    /// Reference Links).
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if offset calculation fails.
    #[inline]
    pub fn scan_document(
        &self,
        markdown: &str,
    ) -> Result<Vec<ScanArtifact>, NoteIngestError> {
        let mut artifacts = Vec::new();
        let mut offset = 0usize;
        let mut in_code_block = false;
        let mut in_frontmatter = false;
        let mut frontmatter_fence: Option<&'static str> = None;

        for line in markdown.split_inclusive('\n') {
            let trimmed_line = line.trim_end_matches(['\n', '\r']);

            // 1. Handle Frontmatter boundary
            if offset == 0 {
                if trimmed_line == "---" {
                    in_frontmatter = true;
                    frontmatter_fence = Some("---");
                    offset = offset.saturating_add(line.len());
                    continue;
                }
                if trimmed_line == "+++" {
                    in_frontmatter = true;
                    frontmatter_fence = Some("+++");
                    offset = offset.saturating_add(line.len());
                    continue;
                }
            }

            if in_frontmatter {
                if frontmatter_fence.is_some_and(|fence| fence == trimmed_line)
                {
                    in_frontmatter = false;
                }
                offset = offset.saturating_add(line.len());
                continue;
            }

            // 2. Handle Code Block boundary
            let trimmed_start = trimmed_line.trim_start();
            if trimmed_start.starts_with("```")
                || trimmed_start.starts_with("~~~")
            {
                in_code_block = !in_code_block;
                offset = offset.saturating_add(line.len());
                continue;
            }

            if in_code_block {
                offset = offset.saturating_add(line.len());
                continue;
            }

            // 3. Scan for Reference Link Definitions (starts with `[`)
            if let Some(ref_link) =
                Self::scan_ref_link_definition(line, offset)?
            {
                artifacts.push(ScanArtifact::ReferenceLink(ref_link));
            }

            // 4. Scan for Block References (ends with ` ^id`)
            if let Some(block_ref) = Self::scan_block_ref(trimmed_line, offset)?
            {
                artifacts.push(ScanArtifact::BlockRef(block_ref));
            }

            offset = offset.saturating_add(line.len());
        }

        Ok(artifacts)
    }

    fn scan_ref_link_definition(
        line: &str,
        line_offset: usize,
    ) -> Result<Option<RawReferenceLink>, NoteIngestError> {
        let trimmed_line = line.trim_end_matches(['\n', '\r']);
        let leading =
            trimmed_line.chars().take_while(|ch| ch.is_whitespace()).count();
        let content = trimmed_line.get(leading..).unwrap_or("");

        if !content.starts_with('[') {
            return Ok(None);
        }

        let Some(close) = content.find("]:") else {
            return Ok(None);
        };

        let label = content.get(1..close).unwrap_or("");
        let after_colon = close.saturating_add(2);
        let mut rest = content.get(after_colon..).unwrap_or("");
        if let Some(stripped) = rest.strip_prefix(' ') {
            rest = stripped;
        }
        let dest = rest.trim_start();

        if label.trim().is_empty() || dest.is_empty() {
            return Ok(None);
        }

        let target = if let Some(stripped) = dest.strip_prefix('<')
            && let Some(end) = stripped.find('>')
        {
            stripped.get(..end).unwrap_or("")
        } else {
            dest.split_whitespace().next().unwrap_or("")
        };

        if target.is_empty() {
            return Ok(None);
        }

        let normalized = label
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_ascii_lowercase();

        let position = SourceByteOffset::try_from_usize(
            line_offset.saturating_add(leading),
        )
        .map_err(|_error| {
            NoteIngestError::Source("reference link offset out of range".into())
        })?;

        Ok(Some(RawReferenceLink::new(
            normalized.into_boxed_str(),
            target.into(),
            position,
        )))
    }

    fn scan_block_ref(
        trimmed_line: &str,
        line_offset: usize,
    ) -> Result<Option<RawBlockRef>, NoteError> {
        let line = trimmed_line.trim_end();
        if let Some(caret_idx) = line.rfind('^') {
            let before = line.get(..caret_idx).unwrap_or("");
            let after = line.get(caret_idx.saturating_add(1)..).unwrap_or("");
            let id = after.trim();

            let valid = !id.is_empty()
                && id.chars().all(|ch| {
                    ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'
                });

            if valid
                && (before.is_empty()
                    || before.chars().last().is_some_and(char::is_whitespace))
            {
                let position = SourceByteOffset::try_from_usize(
                    line_offset.saturating_add(caret_idx),
                )?;
                return Ok(Some(RawBlockRef::new(id.into(), position)));
            }
        }
        Ok(None)
    }
}

/// Helper for scanning task markers from source.
pub struct TaskMarkerScanner<'source> {
    chars: std::iter::Peekable<std::str::Chars<'source>>,
}

impl<'source> TaskMarkerScanner<'source> {
    /// Create a new scanner for a single line of text.
    #[inline]
    #[must_use]
    pub fn new(line: &'source str) -> Self {
        Self {
            chars: line.chars().peekable(),
        }
    }

    /// Scans for a task marker (e.g., `- [ ]`).
    #[inline]
    pub fn scan(&mut self) -> Option<char> {
        self.skip_whitespace();
        self.consume_list_marker()?;
        self.skip_whitespace();
        self.parse_checkbox_marker()
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.chars.peek(), Some(ch) if ch.is_whitespace()) {
            self.chars.next();
        }
    }

    fn consume_list_marker(&mut self) -> Option<()> {
        let first = self.chars.peek().copied()?;
        if matches!(first, '-' | '*' | '+') {
            self.chars.next();
            return Some(());
        }
        if !first.is_ascii_digit() {
            return None;
        }
        while matches!(self.chars.peek(), Some(ch) if ch.is_ascii_digit()) {
            self.chars.next();
        }
        match self.chars.peek().copied()? {
            '.' | ')' => {
                self.chars.next();
                Some(())
            }
            _ => None,
        }
    }

    fn parse_checkbox_marker(&mut self) -> Option<char> {
        if self.chars.next()? != '[' {
            return None;
        }
        let marker = self.chars.next()?;
        if self.chars.next()? != ']' {
            return None;
        }
        Some(marker)
    }

    /// Converts a raw marker character into a [`RawTaskKind`].
    #[inline]
    #[must_use]
    pub fn raw_task_kind_from_marker(marker: char) -> RawTaskKind {
        match marker {
            ' ' => RawTaskKind::Unchecked(marker),
            'x' | 'X' => RawTaskKind::Checked(marker),
            _ => RawTaskKind::Other(marker),
        }
    }

    /// Helper to find a task marker in source at a given position.
    #[inline]
    #[must_use]
    pub fn find_in_source(
        source: &'source str,
        position: SourceByteOffset,
    ) -> Option<char> {
        let start = usize::try_from(u32::from(position)).ok()?;
        let tail = source.get(start..)?;
        let line = tail.split(['\n', '\r']).next().unwrap_or(tail);
        Self::new(line).scan()
    }
}
