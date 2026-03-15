//! Raw tag extraction helpers.

use crate::note::{error::NoteError, position::SourceByteOffset};

/// Raw tag token extracted from text.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawTag {
    value: Box<str>,
    position: SourceByteOffset,
}

impl RawTag {
    /// Create a raw tag token.
    #[inline]
    #[must_use]
    pub fn new(value: Box<str>, position: SourceByteOffset) -> Self {
        Self {
            value,
            position,
        }
    }

    /// Return the raw tag token value.
    #[inline]
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Return the source byte position of the tag token.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }
}

pub(crate) fn scan_raw_tags(
    text: &str,
    base_offset: SourceByteOffset,
) -> Result<Vec<RawTag>, NoteError> {
    let mut tags = Vec::new();
    let mut chars = text.char_indices().peekable();
    let mut prev_is_alnum = false;
    let base = usize::try_from(u32::from(base_offset))
        .map_err(|_error| NoteError::Structure("tag offset out of range"))?;

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
            let Some(updated) = next_idx.checked_add(next_ch.len_utf8()) else {
                break;
            };
            end_idx = updated;
        }

        let Some(raw) = text.get(start_idx..end_idx) else {
            prev_is_alnum = ch.is_alphanumeric();
            continue;
        };

        if raw.len() > 1 {
            let offset = base.saturating_add(start_idx);
            let position = SourceByteOffset::try_from_usize(offset)?;
            tags.push(RawTag::new(raw.into(), position));
        }

        prev_is_alnum = raw.chars().last().is_some_and(char::is_alphanumeric);
    }

    Ok(tags)
}
