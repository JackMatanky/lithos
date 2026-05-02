//! Block-level artifact extraction component.
//!
//! [`BlockExtractor`] owns the scanner and task spec needed to extract raw
//! artifacts from completed parser blocks. It is invoked by the orchestrator
//! (`MarkdownParser`) after a block's text and scannable ranges have been
//! fully accumulated.

use std::borrow::Cow;

use super::{
    parser::{
        ArtifactSink, BlockSpan, ContainerKind, LeafKind,
        text::TextSequence,
        types::{FrontmatterFormat, RangedEvent},
    },
    raw::RawListItemText,
};
use crate::note::{
    error::NoteIngestError,
    position::{SourceByteOffset, SourceByteRange},
    raw::{
        RawFrontmatter, RawHeading, RawLink, RawListItem, RawNote, RawSection,
        RawSectionKind, frontmatter::RawFrontmatterFormat,
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
        events: &[RangedEvent<'source>],
        depth: u32,
    ) -> Result<(), NoteIngestError> {
        let projection = TextSequence::from_events(events);
        let block_range = span.to_source_range()?;
        match kind {
            LeafKind::Heading(payload) => {
                let scanned = self.scan_projection(&projection)?;
                let text = projection.as_displayable_text();
                let trimmed = text.trim();
                let heading_text = if trimmed.len() == text.len() {
                    Cow::Owned(text)
                } else {
                    Cow::Owned(trimmed.to_owned())
                };

                self.out.headings.push(RawHeading::new(
                    payload.to_u8(),
                    heading_text,
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
                let scanned = self.scan_projection(&projection)?;
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
                let scanned = self.scan_projection(&projection)?;
                self.out.sections.push(RawSection::new(
                    RawSectionKind::List,
                    block_range,
                    payload.depth.to_u32(),
                ));

                let (raw_text, text_range) =
                    projection_text_and_range(&projection, block_range)?;

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
                let text = projection.as_plain_text();
                self.out.sections.push(RawSection::new(
                    RawSectionKind::Frontmatter,
                    block_range,
                    0,
                ));
                self.out.frontmatter = Some(RawFrontmatter::new(
                    match payload.format {
                        FrontmatterFormat::Yaml => RawFrontmatterFormat::Yaml,
                        FrontmatterFormat::Toml => RawFrontmatterFormat::Toml,
                    },
                    text.into(),
                    block_range,
                ));
            }
            LeafKind::ThematicBreak => {
                self.out.sections.push(RawSection::new(
                    RawSectionKind::ThematicBreak,
                    block_range,
                    depth,
                ));
            }
        }

        Ok(())
    }

    fn scan_projection(
        &self,
        projection: &TextSequence,
    ) -> Result<ScannedRawArtifacts<'source>, NoteIngestError> {
        let scannable_ranges = scannable_ranges_from_projection(projection);

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
        events: &[RangedEvent<'source>],
        depth: u32,
    ) -> Result<(), NoteIngestError> {
        self.process_leaf(kind, span, events, depth)
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

/// Extracts text and source range from projected text nodes without any
/// trimming. The raw layer preserves source content as-is; trimming is a domain
/// concern.
fn projection_text_and_range(
    projection: &TextSequence,
    block_range: SourceByteRange,
) -> Result<(String, SourceByteRange), NoteIngestError> {
    let text = projection.as_displayable_text();
    let range = match projection.covering_range() {
        Some(range) => range,
        None => SourceByteRange::new(block_range.start(), block_range.start())
            .map_err(NoteIngestError::Domain)?,
    };
    Ok((text, range))
}

fn scannable_ranges_from_projection(
    projection: &TextSequence,
) -> Vec<std::ops::Range<usize>> {
    projection
        .nodes()
        .iter()
        .filter(|node| node.is_scannable())
        .map(|node| {
            node.range().start().as_usize()..node.range().end().as_usize()
        })
        .collect()
}
