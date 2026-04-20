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
    parser::{Block, BlockKind, FragmentPool, TextFragment},
    raw::RawListItemText,
};
use crate::note::{
    error::NoteIngestError,
    position::{SourceByteOffset, SourceByteRange},
    raw::{RawHeading, RawListItem, RawNote, RawSection, RawSectionKind},
    scanner::{NoteScanner, ScannedRawArtifacts},
};

/// Artifact extractor for note markdown blocks.
///
/// Bundles a [`NoteScanner`] with the original source text to provide
/// high-level block finalization methods. This avoids passing the scanner and
/// source to every parser event handler.
pub(crate) struct BlockExtractor<'source> {
    source: &'source str,
    scanner: NoteScanner,
}

impl<'source> BlockExtractor<'source> {
    /// Creates a new extractor.
    #[inline]
    #[must_use]
    pub fn new(source: &'source str, scanner: NoteScanner) -> Self {
        Self {
            source,
            scanner,
        }
    }

    /// Finalises a heading block by recording the heading and its section.
    ///
    /// Scans the block's fragments for inline artifacts, appends the extracted
    /// [`RawHeading`] and a [`RawSectionKind::Heading`] section to `out`,
    /// routes any scan artifacts to the appropriate `out` collections, and
    /// returns the block fragments to `pool`.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if block scanning or range construction
    /// fails.
    pub(crate) fn finalize_heading(
        &self,
        block: Block<'source>,
        depth: u32,
        out: &mut RawNote<'source>,
        pool: &mut FragmentPool<'source>,
    ) -> Result<(), NoteIngestError> {
        let BlockKind::Heading(payload) = block.kind else {
            return Ok(());
        };
        let block_range = block.span.to_source_range()?;
        let text = collect_text(&block.fragments);
        let scan = self.scan_fragments(&block.fragments)?;
        out.headings.push(RawHeading::new(
            payload.to_u8(),
            Cow::Owned(text.trim().to_owned()),
            block_range,
            SourceByteOffset::try_from_usize(block.span.start)?,
        ));
        out.sections.push(RawSection::new(
            RawSectionKind::Heading,
            block_range,
            depth,
        ));
        Self::extend_output(scan, out);
        pool.put(block.fragments);
        Ok(())
    }

    /// Finalises a paragraph block by recording its section.
    ///
    /// Scans the block's fragments for inline artifacts, appends a
    /// [`RawSectionKind::Paragraph`] section to `out`, routes any scan
    /// artifacts, and returns the block fragments to `pool`.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if block scanning or range construction
    /// fails.
    pub(crate) fn finalize_paragraph(
        &self,
        block: Block<'source>,
        depth: u32,
        out: &mut RawNote<'source>,
        pool: &mut FragmentPool<'source>,
    ) -> Result<(), NoteIngestError> {
        let block_range = block.span.to_source_range()?;
        let scan = self.scan_fragments(&block.fragments)?;
        out.sections.push(RawSection::new(
            RawSectionKind::Paragraph,
            block_range,
            depth,
        ));
        Self::extend_output(scan, out);
        pool.put(block.fragments);
        Ok(())
    }

    /// Finalises a list item block by building and appending a [`RawListItem`].
    ///
    /// Scans the block for inline artifacts, computes the text range from
    /// fragments, and delegates to [`RawNote::accept_list_item`] to enforce the
    /// dual-write invariant. Returns the block fragments to `pool`.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if range construction fails.
    pub(crate) fn finalize_list_item(
        &self,
        block: Block<'source>,
        out: &mut RawNote<'source>,
        pool: &mut FragmentPool<'source>,
    ) -> Result<(), NoteIngestError> {
        let BlockKind::ListItem(payload) = block.kind else {
            return Ok(());
        };

        let block_range = block.span.to_source_range()?;
        let scan = self.scan_fragments(&block.fragments)?;
        out.sections.push(RawSection::new(
            RawSectionKind::List,
            block_range,
            payload.depth.to_u32(),
        ));

        // Destructure scan before consuming its parts separately.
        let ScannedRawArtifacts {
            tags,
            inline_fields,
            block_refs,
        } = scan;
        // block_refs route directly; tags/fields go through accept_list_item
        out.block_refs.extend(block_refs);

        let (raw_text, text_range) =
            fragments_to_text_and_range(&block.fragments)?;

        let item = RawListItem::new(
            payload.kind,
            payload.depth,
            payload.parent_pos,
            payload.is_checkbox,
            RawListItemText::new(Cow::Owned(raw_text), text_range),
            block_range,
            tags,
            inline_fields,
        );
        out.accept_list_item(item);
        pool.put(block.fragments);
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
        scan: ScannedRawArtifacts<'source>,
        out: &mut RawNote<'source>,
    ) {
        out.tags.extend(scan.tags);
        out.inline_fields.extend(scan.inline_fields);
        out.block_refs.extend(scan.block_refs);
    }

    /// Scans fragments for metadata artifacts.
    ///
    /// Extracts scannable ranges from fragments and delegates to the scanner.
    fn scan_fragments(
        &self,
        fragments: &[TextFragment<'source>],
    ) -> Result<ScannedRawArtifacts<'source>, NoteIngestError> {
        let scannable_ranges: Vec<std::ops::Range<usize>> = fragments
            .iter()
            .filter(|f| f.is_scannable)
            .map(|f| f.range.start().as_usize()..f.range.end().as_usize())
            .collect();

        self.scanner
            .scan_ranges(self.source, &scannable_ranges, false)
            .map_err(NoteIngestError::Domain)
    }
}

// ── Free functions ───────────────────────────────────────────────────────────

/// Collects text from fragments into a single string.
fn collect_text(fragments: &[TextFragment<'_>]) -> String {
    fragments.iter().map(|f| f.text.as_ref()).collect()
}

/// Extracts text and source range from fragments without any trimming.
/// The raw layer preserves source content as-is; trimming is a domain concern.
///
/// # Errors
///
/// Returns [`NoteIngestError`] if range construction fails.
fn fragments_to_text_and_range(
    fragments: &[TextFragment<'_>],
) -> Result<(String, SourceByteRange), NoteIngestError> {
    if fragments.is_empty() {
        let zero = SourceByteOffset::new(0);
        return Ok((
            String::new(),
            SourceByteRange::new(zero, zero)
                .map_err(NoteIngestError::Domain)?,
        ));
    }

    let text = collect_text(fragments);
    let start = fragments
        .first()
        .map_or_else(|| SourceByteOffset::new(0), |f| f.range.start());
    let end = fragments.last().map_or(start, |f| f.range.end());
    let range =
        SourceByteRange::new(start, end).map_err(NoteIngestError::Domain)?;

    Ok((text, range))
}
