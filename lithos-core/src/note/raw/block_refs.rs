//! Raw block reference extraction helpers.

use crate::note::{error::NoteError, position::SourceByteOffset};

/// Raw block reference token extracted from note text.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawBlockRef {
    id: Box<str>,
    position: SourceByteOffset,
}

impl RawBlockRef {
    /// Create a raw block reference.
    #[inline]
    #[must_use]
    pub fn new(id: Box<str>, position: SourceByteOffset) -> Self {
        Self {
            id,
            position,
        }
    }

    /// Return the raw block reference id.
    #[inline]
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Return the source byte position for the block reference.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }
}

pub(crate) fn collect_block_refs(
    source: &str,
) -> Result<Vec<RawBlockRef>, NoteError> {
    let mut refs = Vec::new();
    let mut offset = 0usize;
    let mut in_code_block = false;
    let mut in_frontmatter = false;
    let mut frontmatter_fence: Option<&'static str> = None;

    for line in source.split_inclusive('\n') {
        let mut trimmed_line = line.trim_end_matches(['\n', '\r']);

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
            if frontmatter_fence.is_some_and(|fence| fence == trimmed_line) {
                in_frontmatter = false;
            }
            offset = offset.saturating_add(line.len());
            continue;
        }

        let trimmed_start = trimmed_line.trim_start();
        if trimmed_start.starts_with("```") || trimmed_start.starts_with("~~~")
        {
            in_code_block = !in_code_block;
            offset = offset.saturating_add(line.len());
            continue;
        }

        if in_code_block {
            offset = offset.saturating_add(line.len());
            continue;
        }

        trimmed_line = trimmed_line.trim_end();
        if let Some(caret_idx) = trimmed_line.rfind('^') {
            let before = trimmed_line.get(..caret_idx).unwrap_or("");
            let after =
                trimmed_line.get(caret_idx.saturating_add(1)..).unwrap_or("");
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
                    offset.saturating_add(caret_idx),
                )?;
                refs.push(RawBlockRef::new(id.into(), position));
            }
        }
        offset = offset.saturating_add(line.len());
    }

    Ok(refs)
}
