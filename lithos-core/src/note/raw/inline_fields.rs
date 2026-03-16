//! Raw inline field extraction helpers.

use crate::note::{
    error::NoteError, parser::ast::Text, position::SourceByteOffset,
};

/// Raw inline field extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawInlineField {
    key: Box<str>,
    value: Box<str>,
    position: SourceByteOffset,
}

impl RawInlineField {
    /// Create a raw inline field entry.
    #[inline]
    #[must_use]
    pub fn new(
        key: Box<str>,
        value: Box<str>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            key,
            value,
            position,
        }
    }

    /// Return the normalized key.
    #[inline]
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Return the raw value string.
    #[inline]
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Return the source byte position of the field key.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }
}

pub(crate) fn scan_inline_fields(
    text: &Text,
    fields: &mut Vec<RawInlineField>,
) -> Result<(), NoteError> {
    let mut combined = String::new();
    let mut segments = Vec::new();

    for node in text.nodes() {
        if node.origin() != crate::note::parser::ast::TextOrigin::Normal {
            continue;
        }
        let start = combined.len();
        combined.push_str(node.content());
        segments.push((start, node.range().start()));
    }

    if combined.is_empty() {
        return Ok(());
    }

    scan_inline_fields_in_text(&combined, &segments, fields)
}

fn scan_inline_fields_in_text(
    text: &str,
    segments: &[(usize, SourceByteOffset)],
    fields: &mut Vec<RawInlineField>,
) -> Result<(), NoteError> {
    let mut bracket_spans = Vec::new();
    scan_inline_fields_delim(
        text,
        b'[',
        b']',
        segments,
        fields,
        &mut bracket_spans,
    )?;
    scan_inline_fields_delim(
        text,
        b'(',
        b')',
        segments,
        fields,
        &mut bracket_spans,
    )?;
    scan_bare_inline_fields(text, segments, fields, &bracket_spans)?;
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "Inline field parsing needs delimiters and position mapping"
)]
fn scan_inline_fields_delim(
    text: &str,
    open_delim: u8,
    close_delim: u8,
    segments: &[(usize, SourceByteOffset)],
    fields: &mut Vec<RawInlineField>,
    spans: &mut Vec<(usize, usize)>,
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
        spans.push((open, close.saturating_add(1)));
        let Some(inner) = text.get(after_open..close) else {
            cursor = close.saturating_add(1);
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
                let position = position_for_offset(segments, key_start)?;
                fields.push(RawInlineField::new(
                    normalize_key(key_trimmed),
                    value_trimmed.into(),
                    position,
                ));
            }
        }
        cursor = close.saturating_add(1);
    }
    Ok(())
}

fn scan_bare_inline_fields(
    text: &str,
    segments: &[(usize, SourceByteOffset)],
    fields: &mut Vec<RawInlineField>,
    bracket_spans: &[(usize, usize)],
) -> Result<(), NoteError> {
    let mut offset = 0usize;
    for line in text.split_inclusive(['\n', '\r']) {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        let Some((key, value)) = trimmed.split_once("::") else {
            offset = offset.saturating_add(line.len());
            continue;
        };
        let key_trimmed = key.trim();
        let value_trimmed = value.trim();
        if key_trimmed.is_empty() || value_trimmed.is_empty() {
            offset = offset.saturating_add(line.len());
            continue;
        }
        let key_start =
            trimmed.find(key_trimmed).unwrap_or(0).saturating_add(offset);
        if bracket_spans
            .iter()
            .any(|&(start, end)| key_start >= start && key_start < end)
        {
            offset = offset.saturating_add(line.len());
            continue;
        }
        let position = position_for_offset(segments, key_start)?;
        fields.push(RawInlineField::new(
            normalize_key(key_trimmed),
            value_trimmed.into(),
            position,
        ));
        offset = offset.saturating_add(line.len());
    }
    Ok(())
}

fn position_for_offset(
    segments: &[(usize, SourceByteOffset)],
    offset: usize,
) -> Result<SourceByteOffset, NoteError> {
    let mut current = None;
    for &(start, position) in segments.iter().rev() {
        if start <= offset {
            current = Some((start, position));
            break;
        }
    }
    let (segment_start, segment_pos) = current
        .ok_or(NoteError::Structure("inline field offset out of range"))?;
    let delta = offset.saturating_sub(segment_start);
    let base = usize::try_from(u32::from(segment_pos)).map_err(|_error| {
        NoteError::Structure("inline field offset out of range")
    })?;
    SourceByteOffset::try_from_usize(base.saturating_add(delta))
}

fn normalize_key(key: &str) -> Box<str> {
    let stripped = key
        .chars()
        .filter(|ch| !matches!(ch, '*' | '_' | '~' | '`'))
        .collect::<String>();
    stripped
        .split_whitespace()
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join("-")
        .into_boxed_str()
}
