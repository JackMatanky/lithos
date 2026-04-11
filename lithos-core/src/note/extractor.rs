//! Block-level artifact extraction component.
//!
//! [`BlockExtractor`] owns the scanner and task spec needed to extract raw
//! artifacts from completed parser blocks. It is invoked by the orchestrator
//! (`MarkdownParser`) after a block's text and scannable ranges have been
//! fully accumulated.
//!
//! **Scope rule:** if finalizing a block type requires calling the scanner,
//! the method belongs here. If not, it stays as a few lines in `on_end`.

use std::borrow::Cow;

use super::{
    parser::{
        Block, BlockKind, StringPool, block_ref_tail_range, depth_raw_to_u32,
    },
    raw::inline_field::field_token_to_raw,
};
use crate::{
    config::task::TaskConfigSpec,
    note::{
        error::NoteIngestError,
        position::{SourceByteOffset, SourceByteRange},
        raw::{
            RawHeading, RawListItem, RawNote, RawSection, RawSectionKind,
            RawTaskStatusSymbol,
        },
        scanner::{NoteScanner, ScannedRawArtifacts},
    },
};

/// Extracts raw artifacts from completed parser blocks.
///
/// Receives a [`Block`] with accumulated text and scannable ranges, runs
/// [`NoteScanner`] on the appropriate ranges, and writes the resulting raw
/// artifacts into [`RawNote`].
///
/// Orchestrator-level concerns (stack mutations, list context updates, text
/// propagation to parent blocks) remain the caller's responsibility.
pub(crate) struct BlockExtractor<'source, 'cfg> {
    source: &'source str,
    scanner: NoteScanner,
    task_spec: &'cfg TaskConfigSpec,
}

impl<'source, 'cfg> BlockExtractor<'source, 'cfg> {
    /// Creates a new `BlockExtractor` bound to `source` and `task_spec`.
    pub(crate) fn new(
        source: &'source str,
        scanner: NoteScanner,
        task_spec: &'cfg TaskConfigSpec,
    ) -> Self {
        Self {
            source,
            scanner,
            task_spec,
        }
    }

    // -----------------------------------------------------------------------
    // Public: one method per scan-based block type
    // -----------------------------------------------------------------------

    /// Finalises a heading block by recording the heading and its section.
    ///
    /// Scans the block's scannable ranges for inline artifacts, appends the
    /// extracted [`RawHeading`] and a [`RawSectionKind::Heading`] section to
    /// `out`, routes any scan artifacts to the appropriate `out` collections,
    /// and returns the block text to `pool`.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if block scanning or range construction
    /// fails.
    #[expect(
        clippy::too_many_arguments,
        reason = "Block finalization requires block, range, depth, output, \
                  and pool — no natural grouping avoids this without \
                  introducing a context struct just for these two methods"
    )]
    pub(crate) fn finalize_heading(
        &self,
        block: Block<'source>,
        block_range: SourceByteRange,
        depth: u32,
        out: &mut RawNote<'source>,
        pool: &mut StringPool,
    ) -> Result<(), NoteIngestError> {
        let BlockKind::Heading(level) = block.kind else {
            return Ok(());
        };
        let scan = self.scan_block(&block, block_range)?;
        out.headings.push(RawHeading::new(
            level,
            Cow::Owned(block.text.trim().to_owned()),
            block_range,
            block.start,
        ));
        out.sections.push(RawSection::new(
            RawSectionKind::Heading,
            block_range,
            depth,
        ));
        self.extend_output(scan, out);
        pool.put(block.text);
        Ok(())
    }

    /// Finalises a paragraph block by recording its section.
    ///
    /// Scans the block's scannable ranges for inline artifacts, appends a
    /// [`RawSectionKind::Paragraph`] section to `out`, routes any scan
    /// artifacts, and returns the block text to `pool`.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if block scanning or range construction
    /// fails.
    #[expect(
        clippy::too_many_arguments,
        reason = "Block finalization requires block, range, depth, output, \
                  and pool — no natural grouping avoids this without \
                  introducing a context struct just for these two methods"
    )]
    pub(crate) fn finalize_paragraph(
        &self,
        block: Block<'source>,
        block_range: SourceByteRange,
        depth: u32,
        out: &mut RawNote<'source>,
        pool: &mut StringPool,
    ) -> Result<(), NoteIngestError> {
        let scan = self.scan_block(&block, block_range)?;
        out.sections.push(RawSection::new(
            RawSectionKind::Paragraph,
            block_range,
            depth,
        ));
        self.extend_output(scan, out);
        pool.put(block.text);
        Ok(())
    }

    /// Finalises a list item block by building and appending a [`RawListItem`].
    ///
    /// Scans the block for inline artifacts and the optional task marker,
    /// computes the trimmed text range, applies task-spec date typing to inline
    /// fields, and delegates to [`RawNote::accept_list_item`] to enforce the
    /// dual-write invariant. Returns the block text to `pool`.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if block scanning, task-marker scanning, or
    /// text-range computation fails.
    pub(crate) fn finalize_list_item(
        &self,
        block: Block<'source>,
        block_range: SourceByteRange,
        out: &mut RawNote<'source>,
        pool: &mut StringPool,
    ) -> Result<(), NoteIngestError> {
        let BlockKind::ListItem {
            list_kind,
            list_depth,
            parent_pos,
        } = block.kind
        else {
            return Ok(());
        };

        let scan = self.scan_block(&block, block_range)?;
        out.sections.push(RawSection::new(
            RawSectionKind::List,
            block_range,
            depth_raw_to_u32(list_depth),
        ));

        let task_marker = if block.task_checked.is_some() {
            self.scan_task_marker_first_line(block_range)?
        } else {
            None
        };

        // Destructure scan before consuming its parts separately.
        let ScannedRawArtifacts {
            tags,
            inline_fields,
            block_refs,
            ..
        } = scan;
        // block_refs route directly; tags/fields go through accept_list_item
        out.block_refs.extend(block_refs);

        let (raw_text, text_range) =
            Self::compute_item_text(&block, block_range)?;

        let inline_fields = inline_fields
            .into_iter()
            .map(|t| field_token_to_raw(t, self.task_spec))
            .collect();

        let item = RawListItem::new(
            list_kind,
            list_depth,
            Cow::Owned(raw_text),
            block.task_checked,
            task_marker,
            block_range,
            text_range,
            parent_pos,
            tags,
            inline_fields,
        );
        out.accept_list_item(item);
        pool.put(block.text);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Routes scan artifacts into the output collections.
    ///
    /// Not used for list items — those go through [`RawNote::accept_list_item`]
    /// to enforce the dual-write invariant.
    fn extend_output(
        &self,
        scan: ScannedRawArtifacts<'source>,
        out: &mut RawNote<'source>,
    ) {
        out.tags.extend(scan.tags);
        out.inline_fields.extend(
            scan.inline_fields
                .into_iter()
                .map(|t| field_token_to_raw(t, self.task_spec)),
        );
        out.block_refs.extend(scan.block_refs);
    }

    /// Scans a block's scannable ranges for metadata artifacts.
    ///
    /// Includes the block-ref tail fallback when no refs are found in the
    /// primary scan and the last scannable range ends at the block boundary.
    fn scan_block(
        &self,
        block: &Block<'source>,
        block_range: SourceByteRange,
    ) -> Result<ScannedRawArtifacts<'source>, NoteIngestError> {
        let mut raw = self
            .scanner
            .scan_ranges_raw(self.source, &block.scannable, false)
            .map_err(NoteIngestError::Domain)?;

        if raw.block_refs.is_empty() {
            let last_end = block.scannable.last().map(|r| r.end);
            if last_end == Some(block_range.end().as_usize())
                && let Some(tail) =
                    block_ref_tail_range(self.source, block_range)
            {
                let tail_raw = self
                    .scanner
                    .scan_ranges_raw(
                        self.source,
                        std::slice::from_ref(&tail),
                        false,
                    )
                    .map_err(NoteIngestError::Domain)?;
                raw.block_refs.extend(tail_raw.block_refs);
            }
        }
        Ok(raw)
    }

    /// Delegates to [`NoteScanner::scan_task_marker_first_line`].
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if offset calculation exceeds supported
    /// bounds.
    fn scan_task_marker_first_line(
        &self,
        block_range: SourceByteRange,
    ) -> Result<Option<RawTaskStatusSymbol>, NoteIngestError> {
        NoteScanner::scan_task_marker_first_line(self.source, block_range)
            .map_err(NoteIngestError::Domain)
    }

    /// Computes trimmed text content and its source-mapped byte range for a
    /// list item.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if byte offset construction fails.
    fn compute_item_text(
        block: &Block<'source>,
        block_range: SourceByteRange,
    ) -> Result<(String, SourceByteRange), NoteIngestError> {
        let raw_text = block.text.trim().to_owned();
        let text_range = if raw_text.is_empty() {
            SourceByteRange::new(block_range.start(), block_range.start())
                .map_err(NoteIngestError::Domain)?
        } else {
            let leading_trim =
                block.text.len().saturating_sub(block.text.trim_start().len());
            let base_start = block
                .scannable
                .first()
                .and_then(|r| SourceByteOffset::try_from(r.start).ok())
                .unwrap_or(block_range.start());
            let text_start = base_start
                .add_offset(leading_trim)
                .map_err(NoteIngestError::Domain)?;
            let text_end = text_start
                .add_offset(raw_text.len())
                .map_err(NoteIngestError::Domain)?;
            SourceByteRange::new(text_start, text_end)
                .map_err(NoteIngestError::Domain)?
        };
        Ok((raw_text, text_range))
    }
}
