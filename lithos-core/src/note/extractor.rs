//! Block-level artifact extraction component.
//!
//! [`BlockExtractor`] owns the scanner and task spec needed to extract raw
//! artifacts from completed parser blocks. It is invoked by the orchestrator
//! (`MarkdownParser`) after a block's text and scannable ranges have been
//! fully accumulated.

use std::borrow::Cow;

use super::{
    parser::{ArtifactSink, BlockSpan, ContainerKind, LeafKind, TextFragment},
    raw::RawListItemText,
};
use crate::note::{
    error::NoteIngestError,
    position::{SourceByteOffset, SourceByteRange},
    raw::{
        RawFrontmatter, RawHeading, RawLink, RawListItem, RawNote, RawSection,
        RawSectionKind,
    },
    scanner::{NoteScanner, ScannedRawArtifacts},
};

/// Artifact extractor for note markdown blocks.
pub struct BlockExtractor<'source> {
    source: &'source str,
    scanner: NoteScanner,
    out: RawNote<'source>,
}

impl<'source> BlockExtractor<'source> {
    /// Creates a new extractor.
    #[inline]
    #[must_use]
    pub fn new(source: &'source str, scanner: NoteScanner) -> Self {
        Self {
            source,
            scanner,
            out: RawNote::new(
                None,
                Vec::with_capacity(4),
                Vec::with_capacity(8),
                Vec::with_capacity(8),
                Vec::with_capacity(8),
                Vec::with_capacity(8),
                Vec::with_capacity(8),
                Vec::with_capacity(8),
            ),
        }
    }

    /// Consumes the extractor and returns the populated note.
    #[inline]
    #[must_use]
    pub fn finish(mut self) -> RawNote<'source> {
        self.out.sections.sort_by_key(|s| u32::from(s.range.start()));
        self.out
    }

    /// Shared processing for all leaf blocks.
    fn process_leaf(
        &mut self,
        kind: LeafKind,
        span: BlockSpan,
        fragments: &[TextFragment<'source>],
        depth: u32,
    ) -> Result<(), NoteIngestError> {
        let block_range = span.to_source_range()?;
        match kind {
            LeafKind::Heading(payload) => {
                let scanned = self.scan_fragments(fragments)?;
                let text = collect_text(fragments);
                self.out.headings.push(RawHeading::new(
                    payload.to_u8(),
                    Cow::Owned(text.trim().to_owned()),
                    block_range,
                    SourceByteOffset::try_from_usize(span.start)?,
                ));
                self.out.sections.push(RawSection::new(
                    RawSectionKind::Heading,
                    block_range,
                    depth,
                ));
                self.out.tags.extend(scanned.tags);
                self.out.inline_fields.extend(scanned.inline_fields);
                self.out.block_refs.extend(scanned.block_refs);
            }
            LeafKind::Paragraph => {
                let scanned = self.scan_fragments(fragments)?;
                self.out.sections.push(RawSection::new(
                    RawSectionKind::Paragraph,
                    block_range,
                    depth,
                ));
                self.out.tags.extend(scanned.tags);
                self.out.inline_fields.extend(scanned.inline_fields);
                self.out.block_refs.extend(scanned.block_refs);
            }
            LeafKind::ListItem(payload) => {
                let scanned = self.scan_fragments(fragments)?;
                self.out.sections.push(RawSection::new(
                    RawSectionKind::List,
                    block_range,
                    payload.depth.to_u32(),
                ));

                let (raw_text, text_range) =
                    fragments_to_text_and_range(fragments)?;

                let item = RawListItem::new(
                    payload.kind,
                    payload.depth,
                    payload.parent_pos,
                    payload.is_checkbox,
                    RawListItemText::new(Cow::Owned(raw_text), text_range),
                    block_range,
                    scanned.tags,
                    scanned.inline_fields,
                );

                self.out.accept_list_item(item);
                self.out.block_refs.extend(scanned.block_refs);
            }
            LeafKind::Metadata(payload) => {
                let text = collect_text(fragments);
                self.out.sections.push(RawSection::new(
                    RawSectionKind::Frontmatter,
                    block_range,
                    0,
                ));
                self.out.frontmatter = Some(RawFrontmatter::new(
                    payload.kind.into(),
                    text.into(),
                    block_range,
                ));
            }
        }

        Ok(())
    }

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

impl<'source> ArtifactSink<'source> for BlockExtractor<'source> {
    fn on_leaf_complete(
        &mut self,
        kind: LeafKind,
        span: BlockSpan,
        fragments: &[TextFragment<'source>],
        depth: u32,
    ) -> Result<(), NoteIngestError> {
        self.process_leaf(kind, span, fragments, depth)
    }

    fn on_container_complete(
        &mut self,
        kind: ContainerKind,
        span: BlockSpan,
        depth: u32,
    ) -> Result<(), NoteIngestError> {
        let range = span.to_source_range()?;
        match kind {
            ContainerKind::List => {}
            ContainerKind::BlockQuote => self.out.sections.push(
                RawSection::new(RawSectionKind::BlockQuote, range, depth),
            ),
            ContainerKind::CodeBlock => self
                .out
                .sections
                .push(RawSection::new(RawSectionKind::CodeBlock, range, depth)),
        }
        Ok(())
    }

    fn on_link(&mut self, link: RawLink<'source>) {
        self.out.links.push(link);
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
