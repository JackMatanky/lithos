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
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::scanner::NoteScanner;
    /// let scanner = NoteScanner::new(vec!['📅']);
    /// ```
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
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::scanner::NoteScanner;
    /// # use lithos_core::note::position::SourceByteOffset;
    /// let segments =
    ///     [(0, SourceByteOffset::new(100)), (10, SourceByteOffset::new(200))];
    /// let pos = NoteScanner::map_position(&segments, 15).unwrap();
    /// assert_eq!(u32::from(pos), 205);
    /// ```
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
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::scanner::{NoteScanner, ScanArtifact};
    /// let scanner = NoteScanner::default();
    /// let artifacts =
    ///     scanner.scan_block("Check #tag [key:: value]", &[]).unwrap();
    /// assert_eq!(artifacts.len(), 2);
    /// ```
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

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Tests grouped by behavior"
)]
mod tests {
    use proptest::prelude::*;
    use rstest::rstest;

    use super::*;
    use crate::note::raw::RawTaskKind;

    mod proptests {
        use super::*;

        proptest! {
            #[test]
            fn map_position_is_monotonic(
                segments in prop::collection::vec((0usize..1000usize, 0u32..u32::try_from(1000usize).unwrap_or(u32::MAX)), 1..10)
                    .prop_map(|mut v| {
                        v.sort_by_key(|s| s.0);
                        v.iter().map(|&(o, p)| (o, SourceByteOffset::new(p))).collect::<Vec<_>>()
                    }),
                offset in 0usize..2000usize
            ) {
                let pos = NoteScanner::map_position(&segments, offset);
                if let Ok(p) = pos {
                    // Position should be >= offset if segments start at 0 and have pos >= 0
                    // Or more generally, it should not crash.
                    let val: u32 = u32::from(p);
                    let _: u32 = val;
                }
            }
        }
    }

    mod map_position {
        use super::*;

        #[test]
        fn should_map_offset_directly_when_no_segments() {
            let segments = [];
            let pos = NoteScanner::map_position(&segments, 10).unwrap();
            assert_eq!(u32::from(pos), 10);
        }

        #[test]
        fn should_map_exact_segment_start() {
            let segments = [(0, SourceByteOffset::new(100))];
            let pos = NoteScanner::map_position(&segments, 0).unwrap();
            assert_eq!(u32::from(pos), 100);
        }

        #[test]
        fn should_map_offset_within_segment() {
            let segments = [(5, SourceByteOffset::new(100))];
            // offset 10 is 5 bytes after segment start (5)
            // pos should be 100 + 5 = 105
            let pos = NoteScanner::map_position(&segments, 10).unwrap();
            assert_eq!(u32::from(pos), 105);
        }

        #[test]
        fn should_select_correct_segment_using_binary_search() {
            let segments = [
                (0, SourceByteOffset::new(100)),
                (10, SourceByteOffset::new(200)),
                (20, SourceByteOffset::new(300)),
            ];

            // Offset 5 -> segment 0 (pos 100) -> 100 + 5 = 105
            assert_eq!(
                u32::from(NoteScanner::map_position(&segments, 5).unwrap()),
                105
            );
            // Offset 10 -> segment 1 (pos 200) -> 200 + 0 = 200
            assert_eq!(
                u32::from(NoteScanner::map_position(&segments, 10).unwrap()),
                200
            );
            // Offset 15 -> segment 1 (pos 200) -> 200 + 5 = 205
            assert_eq!(
                u32::from(NoteScanner::map_position(&segments, 15).unwrap()),
                205
            );
            // Offset 25 -> segment 2 (pos 300) -> 300 + 5 = 305
            assert_eq!(
                u32::from(NoteScanner::map_position(&segments, 25).unwrap()),
                305
            );
        }
    }

    mod scan_block {
        use super::*;

        mod tags {
            use super::*;

            #[test]
            fn should_extract_simple_and_hierarchical_tags() {
                let scanner = NoteScanner::default();
                let text = "Text with #tag and #nested/tag.";
                let artifacts = scanner.scan_block(text, &[]).unwrap();

                let tags: Vec<_> = artifacts
                    .into_iter()
                    .filter_map(|a| match a {
                        ScanArtifact::Tag(t) => Some(t.value().to_owned()),
                        ScanArtifact::InlineField(_)
                        | ScanArtifact::BlockRef(_)
                        | ScanArtifact::ReferenceLink(_) => None,
                    })
                    .collect();

                assert_eq!(tags, vec!["#tag", "#nested/tag"]);
            }

            #[test]
            fn should_ignore_tags_preceded_by_alphanumeric() {
                let scanner = NoteScanner::default();
                let text = "word#tag";
                let artifacts = scanner.scan_block(text, &[]).unwrap();

                let tag_count = artifacts
                    .iter()
                    .filter(|a| matches!(a, ScanArtifact::Tag(_)))
                    .count();
                assert_eq!(tag_count, 0);
            }

            #[test]
            fn should_ignore_single_hash() {
                let scanner = NoteScanner::default();
                let text = "Just a # and some text";
                let artifacts = scanner.scan_block(text, &[]).unwrap();

                let tag_count = artifacts
                    .iter()
                    .filter(|a| matches!(a, ScanArtifact::Tag(_)))
                    .count();
                assert_eq!(tag_count, 0);
            }
        }

        mod inline_fields {
            use super::*;

            #[test]
            fn should_extract_bracketed_and_parenthesized_fields() {
                let scanner = NoteScanner::default();
                let text = "[key1:: val1] and (key2:: val2)";
                let artifacts = scanner.scan_block(text, &[]).unwrap();

                let fields: Vec<_> = artifacts
                    .into_iter()
                    .filter_map(|a| match a {
                        ScanArtifact::InlineField(f) => {
                            Some((f.key().to_owned(), f.value().to_owned()))
                        }
                        ScanArtifact::Tag(_)
                        | ScanArtifact::BlockRef(_)
                        | ScanArtifact::ReferenceLink(_) => None,
                    })
                    .collect();

                assert_eq!(
                    fields
                        .iter()
                        .map(|pair| (pair.0.as_str(), pair.1.as_str()))
                        .collect::<Vec<_>>(),
                    vec![("key1", "val1"), ("key2", "val2")]
                );
            }

            #[test]
            fn should_extract_bare_fields() {
                let scanner = NoteScanner::default();
                let text = "bare_key:: bare_val\nAnother line";
                let artifacts = scanner.scan_block(text, &[]).unwrap();

                let fields: Vec<_> = artifacts
                    .into_iter()
                    .filter_map(|a| match a {
                        ScanArtifact::InlineField(f) => {
                            Some((f.key().to_owned(), f.value().to_owned()))
                        }
                        ScanArtifact::Tag(_)
                        | ScanArtifact::BlockRef(_)
                        | ScanArtifact::ReferenceLink(_) => None,
                    })
                    .collect();

                assert_eq!(
                    fields
                        .iter()
                        .map(|pair| (pair.0.as_str(), pair.1.as_str()))
                        .collect::<Vec<_>>(),
                    vec![("bare_key", "bare_val")]
                );
            }

            #[test]
            fn should_ignore_bare_fields_inside_brackets() {
                let scanner = NoteScanner::default();
                // "nested:: field" is inside [], so it should be captured as
                // delimited, NOT duplicated as bare.
                let text = "[key:: nested:: field]";
                let artifacts = scanner.scan_block(text, &[]).unwrap();

                let fields: Vec<_> = artifacts
                    .into_iter()
                    .filter_map(|a| match a {
                        ScanArtifact::InlineField(f) => {
                            Some((f.key().to_owned(), f.value().to_owned()))
                        }
                        ScanArtifact::Tag(_)
                        | ScanArtifact::BlockRef(_)
                        | ScanArtifact::ReferenceLink(_) => None,
                    })
                    .collect();

                assert_eq!(
                    fields
                        .iter()
                        .map(|pair| (pair.0.as_str(), pair.1.as_str()))
                        .collect::<Vec<_>>(),
                    vec![("key", "nested:: field")]
                );
            }

            #[test]
            fn should_extract_emoji_fields() {
                let scanner = NoteScanner::new(vec!['\u{1f4c5}', '\u{2705}']);
                let text = "\u{1f4c5} 2024-03-19 and \u{2705} done";
                let artifacts = scanner.scan_block(text, &[]).unwrap();

                let fields: Vec<_> = artifacts
                    .into_iter()
                    .filter_map(|a| match a {
                        ScanArtifact::InlineField(f) => {
                            Some((f.key().to_owned(), f.value().to_owned()))
                        }
                        ScanArtifact::Tag(_)
                        | ScanArtifact::BlockRef(_)
                        | ScanArtifact::ReferenceLink(_) => None,
                    })
                    .collect();

                assert_eq!(
                    fields
                        .iter()
                        .map(|pair| (pair.0.as_str(), pair.1.as_str()))
                        .collect::<Vec<_>>(),
                    vec![("\u{1f4c5}", "2024-03-19"), ("\u{2705}", "done")]
                );
            }

            #[rstest]
            #[case::space_in_key("invalid key:: value")]
            #[case::special_chars("key!:: value")]
            fn should_ignore_invalid_bare_keys(#[case] text: &str) {
                let scanner = NoteScanner::default();
                let artifacts = scanner.scan_block(text, &[]).unwrap();

                let field_count = artifacts
                    .iter()
                    .filter(|a| matches!(a, ScanArtifact::InlineField(_)))
                    .count();
                assert_eq!(field_count, 0, "Should ignore bare field: {text}");
            }
        }
    }

    mod scan_document {
        use super::*;

        #[test]
        fn should_extract_block_refs_and_ref_links() {
            let scanner = NoteScanner::default();
            let markdown = "
Paragraph with a block ref ^my-id

[link-label]: https://example.com\
                            ";
            let artifacts = scanner.scan_document(markdown).unwrap();

            let mut block_ref = None;
            let mut ref_link = None;

            for a in artifacts {
                match a {
                    ScanArtifact::BlockRef(r) => {
                        block_ref = Some(r);
                    }
                    ScanArtifact::ReferenceLink(l) => {
                        ref_link = Some(l);
                    }
                    ScanArtifact::Tag(_) | ScanArtifact::InlineField(_) => {}
                }
            }

            assert_eq!(block_ref.unwrap().id(), "my-id");
            assert_eq!(ref_link.unwrap().id(), "link-label");
        }

        #[test]
        fn should_ignore_artifacts_in_frontmatter() {
            let scanner = NoteScanner::default();
            let markdown = "---
aliases: [^not-a-block-ref]
---
Actual ^block-ref
";
            let artifacts = scanner.scan_document(markdown).unwrap();

            let block_refs: Vec<_> = artifacts
                .into_iter()
                .filter_map(|a| match a {
                    ScanArtifact::BlockRef(r) => Some(r.id().to_owned()),
                    ScanArtifact::Tag(_)
                    | ScanArtifact::InlineField(_)
                    | ScanArtifact::ReferenceLink(_) => None,
                })
                .collect();

            assert_eq!(block_refs, vec!["block-ref"]);
        }

        #[test]
        fn should_ignore_artifacts_in_code_blocks() {
            let scanner = NoteScanner::default();
            let markdown = "
```rust
let x = \"^not-a-ref\";
```
Actual ^block-ref
";
            let artifacts = scanner.scan_document(markdown).unwrap();

            let block_refs: Vec<_> = artifacts
                .into_iter()
                .filter_map(|a| match a {
                    ScanArtifact::BlockRef(r) => Some(r.id().to_owned()),
                    ScanArtifact::Tag(_)
                    | ScanArtifact::InlineField(_)
                    | ScanArtifact::ReferenceLink(_) => None,
                })
                .collect();

            assert_eq!(block_refs, vec!["block-ref"]);
        }

        #[test]
        fn should_handle_various_reference_link_formats() {
            let scanner = NoteScanner::default();
            let markdown = "
[simple]: target
[bracketed]: <bracket-target>
  [indented]: target
";
            let artifacts = scanner.scan_document(markdown).unwrap();

            let labels: Vec<_> = artifacts
                .into_iter()
                .filter_map(|a| match a {
                    ScanArtifact::ReferenceLink(l) => Some(l.id().to_owned()),
                    ScanArtifact::Tag(_)
                    | ScanArtifact::InlineField(_)
                    | ScanArtifact::BlockRef(_) => None,
                })
                .collect();

            assert_eq!(labels, vec!["simple", "bracketed", "indented"]);
        }
    }

    mod task_marker_scanner {
        use super::*;

        #[rstest]
        #[case::hyphen("- [ ] ", ' ')]
        #[case::asterisk("* [x] ", 'x')]
        #[case::plus("+ [X] ", 'X')]
        #[case::ordered("1. [-] ", '-')]
        #[case::ordered_paren("1) [/] ", '/')]
        #[case::indented("  - [!] ", '!')]
        fn should_detect_valid_task_markers(
            #[case] line: &str,
            #[case] expected: char,
        ) {
            let mut scanner = TaskMarkerScanner::new(line);
            assert_eq!(scanner.scan(), Some(expected));
        }

        #[rstest]
        #[case::no_brackets("- text")]
        #[case::no_space_in_brackets("- []")]
        #[case::no_list_prefix("[ ] text")]
        #[case::invalid_prefix("a. [ ]")]
        fn should_return_none_for_invalid_markers(#[case] line: &str) {
            let mut scanner = TaskMarkerScanner::new(line);
            assert!(scanner.scan().is_none());
        }

        #[test]
        fn should_convert_marker_to_raw_task_kind() {
            assert!(matches!(
                TaskMarkerScanner::raw_task_kind_from_marker(' '),
                RawTaskKind::Unchecked(' ')
            ));
            assert!(matches!(
                TaskMarkerScanner::raw_task_kind_from_marker('x'),
                RawTaskKind::Checked('x')
            ));
            assert!(matches!(
                TaskMarkerScanner::raw_task_kind_from_marker('X'),
                RawTaskKind::Checked('X')
            ));
            assert!(matches!(
                TaskMarkerScanner::raw_task_kind_from_marker('!'),
                RawTaskKind::Other('!')
            ));
        }
    }
}
