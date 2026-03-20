//! Unified scanning utilities for Note metadata and structure.
//!
//! This module consolidates all manual text-scanning logic (tags, inline
//! fields, block references) into a single boundary, reducing redundant
//! passes over markdown content and ensuring heuristic consistency.
//!
//! The primary entry point is [`NoteScanner`], which handles scanning blocks
//! for multiple types of artifacts. It also provides specialized scanners
//! like [`TaskMarkerScanner`] for low-level parsing of task-specific syntax.

use std::{iter::Peekable, str::CharIndices};

use crate::note::{
    error::NoteError,
    position::SourceByteOffset,
    raw::{RawBlockRef, RawInlineField, RawTag, RawTaskMarker},
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

    /// Scans a block of text for tags and inline fields in a single pass.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if position mapping fails.
    #[inline]
    pub fn scan_block(
        &self,
        text: &str,
        base_offset: SourceByteOffset,
    ) -> Result<Vec<ScanArtifact>, NoteError> {
        let mut artifacts = Vec::new();
        let mut chars = text.char_indices().peekable();
        let mut line_start = true;
        let mut prev_alnum = false;

        while let Some(&(_idx, ch)) = chars.peek() {
            if ch == '\n' || ch == '\r' {
                line_start = true;
                prev_alnum = false;
                chars.next();
                continue;
            }

            let triggered = if ch == '#' && !prev_alnum {
                if let Some(tag) =
                    self.scan_tag_inner(text, &mut chars, base_offset)?
                {
                    artifacts.push(ScanArtifact::Tag(tag));
                    true
                } else {
                    false
                }
            } else if ch == '[' || ch == '(' {
                if let Some(field) = self.scan_delimited_field_inner(
                    text,
                    &mut chars,
                    base_offset,
                )? {
                    artifacts.push(ScanArtifact::InlineField(field));
                    true
                } else {
                    false
                }
            } else if self.emoji_markers.contains(&ch) {
                if let Some(field) =
                    self.scan_emoji_field_inner(text, &mut chars, base_offset)?
                {
                    artifacts.push(ScanArtifact::InlineField(field));
                    true
                } else {
                    false
                }
            } else if line_start
                && let Some(field) =
                    self.scan_bare_field_inner(text, &mut chars, base_offset)?
            {
                artifacts.push(ScanArtifact::InlineField(field));
                true
            } else {
                false
            };

            if triggered {
                line_start = false;
                prev_alnum = true;
            } else if let Some((_, consumed_ch)) = chars.next() {
                if !consumed_ch.is_whitespace() {
                    line_start = false;
                }
                prev_alnum = consumed_ch.is_alphanumeric();
            } else {
                // End of stream
                break;
            }
        }

        Ok(artifacts)
    }

    /// Internal handler for tag scanning.
    ///
    /// Consumes the `#` and all valid tag characters ONLY if a valid tag is
    /// found.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if position mapping fails.
    #[inline]
    pub fn scan_tag_inner(
        &self,
        text: &str,
        chars: &mut Peekable<CharIndices<'_>>,
        base_offset: SourceByteOffset,
    ) -> Result<Option<RawTag>, NoteError> {
        let mut lookahead = chars.clone();
        let Some((start_idx, _hash)) = lookahead.next() else {
            return Ok(None);
        };

        let mut end_idx = start_idx.saturating_add(1);
        let mut consumed_count = 1usize;

        while let Some(&(next_idx, next_ch)) = lookahead.peek() {
            if next_ch.is_alphanumeric() || matches!(next_ch, '_' | '-' | '/') {
                lookahead.next();
                consumed_count = consumed_count.saturating_add(1);
                end_idx = next_idx.saturating_add(next_ch.len_utf8());
            } else {
                break;
            }
        }

        if let Some(raw) = text.get(start_idx..end_idx)
            && raw.len() > 1
        {
            for _ in 0..consumed_count {
                chars.next();
            }
            let position = base_offset.add_offset(start_idx)?;
            Ok(Some(RawTag::new(raw.into(), position)))
        } else {
            Ok(None)
        }
    }

    /// Internal handler for delimited inline fields `[key:: value]` or `(key::
    /// value)`.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if position mapping fails.
    #[inline]
    #[expect(clippy::excessive_nesting, reason = "Parser state-machine depth")]
    pub fn scan_delimited_field_inner(
        &self,
        text: &str,
        chars: &mut Peekable<CharIndices<'_>>,
        base_offset: SourceByteOffset,
    ) -> Result<Option<RawInlineField>, NoteError> {
        let mut lookahead = chars.clone();
        let Some((start_idx, open_delim)) = lookahead.next() else {
            return Ok(None);
        };
        let close_delim = if open_delim == '[' {
            ']'
        } else {
            ')'
        };

        let mut inner_text = String::with_capacity(32);
        let mut consumed_count = 1usize;

        for (idx, ch) in lookahead {
            consumed_count = consumed_count.saturating_add(1);
            if ch == close_delim {
                if let Some((key, value)) = inner_text.split_once("::") {
                    let key_trimmed = key.trim();
                    let value_trimmed = value.trim();
                    if !key_trimmed.is_empty() && !value_trimmed.is_empty() {
                        for _ in 0..consumed_count {
                            chars.next();
                        }
                        let key_start_rel = text
                            .get(start_idx..idx)
                            .and_then(|s| s.find(key_trimmed))
                            .unwrap_or(1);
                        let position = base_offset.add_offset(
                            start_idx.saturating_add(key_start_rel),
                        )?;
                        return Ok(Some(RawInlineField::new(
                            key_trimmed.into(),
                            value_trimmed.into(),
                            position,
                        )));
                    }
                }
                break;
            }
            if ch == '\n' || ch == '\r' {
                break;
            }
            inner_text.push(ch);
        }

        Ok(None)
    }

    /// Internal handler for emoji-prefixed fields.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if position mapping fails.
    #[inline]
    pub fn scan_emoji_field_inner(
        &self,
        _text: &str,
        chars: &mut Peekable<CharIndices<'_>>,
        base_offset: SourceByteOffset,
    ) -> Result<Option<RawInlineField>, NoteError> {
        let mut lookahead = chars.clone();
        let Some((idx, ch)) = lookahead.next() else {
            return Ok(None);
        };
        let mut consumed_count = 1usize;

        while let Some(&(_, next_ch)) = lookahead.peek()
            && next_ch.is_whitespace()
            && next_ch != '\n'
            && next_ch != '\r'
        {
            lookahead.next();
            consumed_count = consumed_count.saturating_add(1);
        }

        let mut value = String::with_capacity(16);
        while let Some(&(_, next_ch)) = lookahead.peek()
            && !next_ch.is_whitespace()
        {
            value.push(next_ch);
            lookahead.next();
            consumed_count = consumed_count.saturating_add(1);
        }

        if value.is_empty() {
            Ok(None)
        } else {
            for _ in 0..consumed_count {
                chars.next();
            }
            let mut buffer = [0u8; 4];
            let key = ch.encode_utf8(&mut buffer);
            let position = base_offset.add_offset(idx)?;
            Ok(Some(RawInlineField::new(key.into(), value.into(), position)))
        }
    }

    /// Internal handler for bare fields `key:: value`.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if position mapping fails.
    #[inline]
    #[expect(clippy::excessive_nesting, reason = "Parser state-machine depth")]
    pub fn scan_bare_field_inner(
        &self,
        _text: &str,
        chars: &mut Peekable<CharIndices<'_>>,
        base_offset: SourceByteOffset,
    ) -> Result<Option<RawInlineField>, NoteError> {
        let mut key = String::with_capacity(16);
        let mut consumed_count = 0usize;
        let mut lookahead = chars.clone();
        let mut first_idx = None;

        while let Some((idx, ch)) = lookahead.next() {
            if first_idx.is_none() {
                first_idx = Some(idx);
            }
            consumed_count = consumed_count.saturating_add(1);
            if ch == ':' {
                let mut peek_next = lookahead.clone();
                if let Some((_, ':')) = peek_next.next() {
                    lookahead.next();
                    consumed_count = consumed_count.saturating_add(1);
                    let key_trimmed = key.trim();
                    if !key_trimmed.is_empty()
                        && key_trimmed.chars().all(|c| {
                            c.is_ascii_alphanumeric() || c == '_' || c == '-'
                        })
                    {
                        let mut value = String::with_capacity(16);
                        for (_, vch) in lookahead.by_ref() {
                            consumed_count = consumed_count.saturating_add(1);
                            if vch == '\n' || vch == '\r' {
                                break;
                            }
                            value.push(vch);
                        }
                        let value_trimmed = value.trim();
                        if !value_trimmed.is_empty() {
                            for _ in 0..consumed_count {
                                chars.next();
                            }
                            let position = base_offset
                                .add_offset(first_idx.unwrap_or(idx))?;
                            return Ok(Some(RawInlineField::new(
                                key_trimmed.into(),
                                value_trimmed.into(),
                                position,
                            )));
                        }
                    }
                    break;
                }
            }
            if ch == '\n'
                || ch == '\r'
                || (ch.is_whitespace() && !key.is_empty())
            {
                break;
            }
            key.push(ch);
        }

        Ok(None)
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

    /// Converts a raw marker character into a [`RawTaskMarker`].
    ///
    /// Maps space to [`RawTaskMarker::Unchecked`], 'x'/'X' to
    /// [`RawTaskMarker::Checked`], and all other characters to
    /// [`RawTaskMarker::Other`].
    #[inline]
    #[must_use]
    pub fn raw_task_marker_from_char(marker: char) -> RawTaskMarker {
        match marker {
            ' ' => RawTaskMarker::Unchecked(marker),
            'x' | 'X' => RawTaskMarker::Checked(marker),
            _ => RawTaskMarker::Other(marker),
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
    use crate::note::raw::RawTaskMarker;

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

            #[test]
            #[expect(clippy::panic, reason = "Test assertion")]
            #[expect(
                clippy::pattern_type_mismatch,
                reason = "Slice pattern match"
            )]
            fn should_not_skip_triggers_after_failed_tag() {
                let scanner = NoteScanner::default();
                let text = "#[field:: value]";
                let artifacts =
                    scanner.scan_block(text, SourceByteOffset::new(0)).unwrap();

                assert_eq!(artifacts.len(), 1);
                match &*artifacts {
                    [ScanArtifact::InlineField(f)] => {
                        assert_eq!(f.key(), "field");
                        assert_eq!(f.value(), "value");
                    }
                    _ => panic!("Expected exactly one inline field"),
                }
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
        fn should_convert_marker_to_raw_task_marker() {
            assert!(matches!(
                TaskMarkerScanner::raw_task_marker_from_char(' '),
                RawTaskMarker::Unchecked(' ')
            ));
            assert!(matches!(
                TaskMarkerScanner::raw_task_marker_from_char('x'),
                RawTaskMarker::Checked('x')
            ));
            assert!(matches!(
                TaskMarkerScanner::raw_task_marker_from_char('X'),
                RawTaskMarker::Checked('X')
            ));
            assert!(matches!(
                TaskMarkerScanner::raw_task_marker_from_char('!'),
                RawTaskMarker::Other('!')
            ));
        }
    }
}
