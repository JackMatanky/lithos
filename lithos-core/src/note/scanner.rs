//! Unified scanning utilities for Note metadata and structure.
//!
//! This module consolidates all manual text-scanning logic (tags, inline
//! fields, block references) into a single boundary, reducing redundant
//! passes over markdown content and ensuring heuristic consistency.
//!
//! The primary entry point is [`NoteScanner`], which handles scanning blocks
//! for multiple types of artifacts. It also provides specialized scanners
//! like [`TaskMarkerScanner`] for low-level parsing of task-specific syntax.

use crate::note::{
    error::NoteError,
    position::SourceByteOffset,
    raw::{RawBlockRef, RawInlineField, RawTag, RawTaskKind},
};

/// A unified result from the scanning process.
///
/// This enum represents the different types of metadata artifacts that can be
/// extracted from a text block.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ScanArtifact {
    /// A hashtag (e.g., `#work/project`).
    Tag(RawTag),
    /// An inline field (e.g., `[key:: value]`).
    InlineField(RawInlineField),
    /// A block reference (e.g., `^block-id`).
    BlockRef(RawBlockRef),
}

/// Specialized scanner for extracting metadata artifacts from markdown.
///
/// `NoteScanner` is designed to be used within the ingestion pipeline to
/// identify Obsidian-style metadata that isn't natively handled by standard
/// markdown parsers.
#[derive(Debug, Clone, Default)]
pub struct NoteScanner {
    /// Emoji markers used for date/status fields.
    emoji_markers: Box<[char]>,
}

impl NoteScanner {
    /// Create a new scanner with the provided emoji markers.
    ///
    /// Emoji markers allow for compact inline fields like `📅 2024-03-19`.
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

    /// Scans a block of text for tags and inline fields.
    ///
    /// This method performs multiple passes over the text to identify:
    /// 1. Hierarchical tags (e.g., `#work/project`)
    /// 2. Delimited inline fields (e.g., `[key:: value]` or `(key:: value)`)
    /// 3. Emoji-prefixed fields (if configured)
    /// 4. Bare inline fields (e.g., `key:: value` at the start of a line)
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::scanner::{NoteScanner, ScanArtifact};
    /// # use lithos_core::note::position::SourceByteOffset;
    /// let scanner = NoteScanner::default();
    /// let artifacts = scanner
    ///     .scan_block("Check #tag [key:: value]", SourceByteOffset::new(0))
    ///     .unwrap();
    ///
    /// assert_eq!(artifacts.len(), 2);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if position mapping fails due to overflow or
    /// invalid UTF-8 boundaries.
    #[inline]
    pub fn scan_block(
        &self,
        text: &str,
        base_offset: SourceByteOffset,
    ) -> Result<Vec<ScanArtifact>, NoteError> {
        let mut artifacts = Vec::new();

        // 1. Scan for tags
        Self::scan_tags(text, base_offset, &mut artifacts)?;

        // 2. Scan for inline fields (Delimited and Bare)
        self.scan_inline_fields(text, base_offset, &mut artifacts)?;

        Ok(artifacts)
    }

    fn scan_tags(
        text: &str,
        base_offset: SourceByteOffset,
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
                let position = base_offset.add_offset(start_idx)?;
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
        base_offset: SourceByteOffset,
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
            base_offset,
            &mut bracket_spans,
            out,
        )?;

        // 2. Delimited (Parentheses)
        Self::scan_delimited_fields(
            text,
            b'(',
            b')',
            base_offset,
            &mut bracket_spans,
            out,
        )?;

        // 3. Emoji fields
        self.scan_emoji_fields(text, base_offset, out)?;

        // 4. Bare fields (on each line, if not inside brackets)
        let mut offset = 0usize;
        for line in text.split_inclusive(['\n', '\r']) {
            Self::scan_bare_fields(
                line,
                offset,
                &bracket_spans,
                base_offset,
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
        base_offset: SourceByteOffset,
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
                    let position = base_offset.add_offset(key_start)?;
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
        base_offset: SourceByteOffset,
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
            let position = base_offset.add_offset(idx)?;
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
        base_offset: SourceByteOffset,
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
            let position = base_offset.add_offset(key_start)?;
            out.push(ScanArtifact::InlineField(RawInlineField::new(
                key_trimmed.into(),
                value_trimmed.into(),
                position,
            )));
        }
        Ok(())
    }

    /// Scans the tail of a text block for an Obsidian-style block reference.
    ///
    /// Block references are identifiers at the very end of a block (like
    /// paragraphs or list items) that allow linking directly to that content.
    /// They follow the pattern ` ^block-id`, where `block-id` is
    /// alphanumeric and preceded by a space and a caret.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::scanner::NoteScanner;
    /// # use lithos_core::note::position::SourceByteOffset;
    /// let scanner = NoteScanner::default();
    /// let text = "Important point ^my-id";
    ///
    /// let block_ref = scanner
    ///     .scan_tail_for_block_ref(text, SourceByteOffset::new(0))
    ///     .unwrap()
    ///     .unwrap();
    ///
    /// assert_eq!(block_ref.id(), "my-id");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if position calculation for the caret exceeds
    /// byte bounds.
    #[inline]
    pub fn scan_tail_for_block_ref(
        &self,
        text: &str,
        base_offset: SourceByteOffset,
    ) -> Result<Option<RawBlockRef>, NoteError> {
        let line = text.trim_end();
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
                let position = base_offset.add_offset(caret_idx)?;
                return Ok(Some(RawBlockRef::new(id.into(), position)));
            }
        }
        Ok(None)
    }
}

/// Helper for scanning task markers from markdown source.
///
/// Task markers like `- [ ]` or `1. [x]` are often handled at the block level
/// by markdown parsers, but the specific marker character (e.g., '/', '!', '>')
/// is needed for custom status tracking. This scanner provides precise
/// extraction of these markers.
pub struct TaskMarkerScanner<'source> {
    chars: std::iter::Peekable<std::str::Chars<'source>>,
}

impl<'source> TaskMarkerScanner<'source> {
    /// Create a new scanner for a single line of text or a block.
    #[inline]
    #[must_use]
    pub fn new(line: &'source str) -> Self {
        Self {
            chars: line.chars().peekable(),
        }
    }

    /// Scans for the next task marker in the current source.
    ///
    /// This method skips leading whitespace and identifies list markers
    /// (e.g., `-`, `*`, `+`, `1.`) followed by a checkbox `[ ]`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::scanner::TaskMarkerScanner;
    /// let mut scanner = TaskMarkerScanner::new("- [x] My task");
    /// assert_eq!(scanner.scan(), Some('x'));
    ///
    /// let mut scanner = TaskMarkerScanner::new("  1. [/] Ongoing");
    /// assert_eq!(scanner.scan(), Some('/'));
    /// ```
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
    ///
    /// Maps space to [`RawTaskKind::Unchecked`], 'x'/'X' to
    /// [`RawTaskKind::Checked`], and all other characters to
    /// [`RawTaskKind::Other`].
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
    ///
    /// This is useful when the ingestion process identifies a block as a task
    /// and needs to extract the precise marker character from the original
    /// source.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::scanner::TaskMarkerScanner;
    /// # use lithos_core::note::position::SourceByteOffset;
    /// let source = "  - [!] Alert";
    /// let marker =
    ///     TaskMarkerScanner::find_in_source(source, SourceByteOffset::new(0));
    ///
    /// assert_eq!(marker, Some('!'));
    /// ```
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
    use rstest::rstest;

    use super::*;
    use crate::note::raw::RawTaskKind;

    mod scan_block {
        use super::*;

        mod tags {
            use super::*;

            #[test]
            fn should_extract_simple_and_hierarchical_tags() {
                let scanner = NoteScanner::default();
                let text = "Text with #tag and #nested/tag.";
                let artifacts = scanner
                    .scan_block(text, SourceByteOffset::new(100))
                    .unwrap();

                let tags: Vec<_> = artifacts
                    .into_iter()
                    .filter_map(|a| match a {
                        ScanArtifact::Tag(t) => Some((
                            t.value().to_owned(),
                            u32::from(t.position()),
                        )),
                        ScanArtifact::InlineField(_)
                        | ScanArtifact::BlockRef(_) => None,
                    })
                    .collect();

                assert_eq!(tags, vec![
                    ("#tag".to_owned(), 110),
                    ("#nested/tag".to_owned(), 119)
                ]);
            }

            #[test]
            fn should_ignore_tags_preceded_by_alphanumeric() {
                let scanner = NoteScanner::default();
                let text = "word#tag";
                let artifacts =
                    scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();

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
                let artifacts =
                    scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();

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
                let artifacts = scanner
                    .scan_block(text, SourceByteOffset::new(10))
                    .unwrap();

                let fields: Vec<_> = artifacts
                    .into_iter()
                    .filter_map(|a| match a {
                        ScanArtifact::InlineField(f) => Some((
                            f.key().to_owned(),
                            f.value().to_owned(),
                            u32::from(f.position()),
                        )),
                        ScanArtifact::Tag(_) | ScanArtifact::BlockRef(_) => {
                            None
                        }
                    })
                    .collect();

                assert_eq!(fields, vec![
                    ("key1".to_owned(), "val1".to_owned(), 11),
                    ("key2".to_owned(), "val2".to_owned(), 29)
                ]);
            }

            #[test]
            fn should_extract_bare_fields() {
                let scanner = NoteScanner::default();
                let text = "bare_key:: bare_val\nAnother line";
                let artifacts =
                    scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();

                let fields: Vec<_> = artifacts
                    .into_iter()
                    .filter_map(|a| match a {
                        ScanArtifact::InlineField(f) => {
                            Some((f.key().to_owned(), f.value().to_owned()))
                        }
                        ScanArtifact::Tag(_) | ScanArtifact::BlockRef(_) => {
                            None
                        }
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
                let text = "[key:: nested:: field]";
                let artifacts =
                    scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();

                let fields: Vec<_> = artifacts
                    .into_iter()
                    .filter_map(|a| match a {
                        ScanArtifact::InlineField(f) => {
                            Some((f.key().to_owned(), f.value().to_owned()))
                        }
                        ScanArtifact::Tag(_) | ScanArtifact::BlockRef(_) => {
                            None
                        }
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
                let artifacts =
                    scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();

                let fields: Vec<_> = artifacts
                    .into_iter()
                    .filter_map(|a| match a {
                        ScanArtifact::InlineField(f) => {
                            Some((f.key().to_owned(), f.value().to_owned()))
                        }
                        ScanArtifact::Tag(_) | ScanArtifact::BlockRef(_) => {
                            None
                        }
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
                let artifacts =
                    scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();

                let field_count = artifacts
                    .iter()
                    .filter(|a| matches!(a, ScanArtifact::InlineField(_)))
                    .count();
                assert_eq!(field_count, 0, "Should ignore bare field: {text}");
            }
        }
    }

    mod scan_tail_for_block_ref {
        use super::*;

        #[test]
        fn should_extract_block_ref_from_tail() {
            let scanner = NoteScanner::default();
            let text = "Paragraph with a block ref ^my-id";
            let block_ref = scanner
                .scan_tail_for_block_ref(text, SourceByteOffset::new(50))
                .unwrap()
                .expect("block ref should be found");

            assert_eq!(block_ref.id(), "my-id");
            assert_eq!(u32::from(block_ref.position()), 50 + 27);
        }

        #[test]
        fn should_ignore_invalid_block_refs() {
            let scanner = NoteScanner::default();
            let text = "No space^id";
            let block_ref = scanner
                .scan_tail_for_block_ref(text, SourceByteOffset::new(0))
                .unwrap();
            assert!(block_ref.is_none());

            let text2 = " ^ invalid id";
            let block_ref2 = scanner
                .scan_tail_for_block_ref(text2, SourceByteOffset::new(0))
                .unwrap();
            assert!(block_ref2.is_none());
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
