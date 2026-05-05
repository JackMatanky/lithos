//! Block-level artifact extraction component.
//!
//! [`BlockExtractor`] owns the scanner and task spec needed to extract raw
//! artifacts from completed parser blocks. It is invoked by the orchestrator
//! (`MarkdownParser`) after a block's text and scannable ranges have been
//! fully accumulated.

#![expect(
    clippy::pattern_type_mismatch,
    reason = "Parser pipeline matches borrowed event payloads intentionally"
)]

use std::borrow::Cow;

use super::{
    parser::{
        block::{Block, BlockKind, Closed, ContainerBlockKind, LeafBlockKind},
        structure::{Complete, DocTree, TraversalEvent},
        text::TextSequence,
        types::{
            FrontmatterFormat, InlineDelimiterEnd, InlineDelimiterStart,
            InlineToken, ListKind, ParserEvent, RangedEvent,
        },
    },
    raw::RawListItemText,
};
use crate::note::{
    error::NoteIngestError,
    position::{SourceByteOffset, SourceByteRange},
    raw::{
        RawFrontmatter, RawHeading, RawLink, RawLinkStyle, RawListDepth,
        RawListItem, RawListKind, RawNote, RawSection, RawSectionKind,
        frontmatter::RawFrontmatterFormat,
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

    pub(crate) fn process_doc_tree(
        &mut self,
        tree: &DocTree<'source, Complete>,
    ) -> Result<(), NoteIngestError> {
        let mut list_kinds: Vec<RawListKind> = Vec::with_capacity(4);

        for event in tree.iter_preorder() {
            match event {
                TraversalEvent::Enter(block, depth) => match &block.kind {
                    BlockKind::Leaf(leaf) => {
                        self.process_leaf_block(
                            leaf,
                            block.span.clone(),
                            depth,
                        )?;
                    }
                    BlockKind::Container(container) => {
                        self.process_container_enter(
                            container,
                            block.span.clone(),
                            depth,
                            &mut list_kinds,
                        )?;
                    }
                },
                TraversalEvent::Exit(block, _depth) => {
                    if let BlockKind::Container(ContainerBlockKind::List {
                        ..
                    }) = &block.kind
                    {
                        list_kinds.pop();
                    }
                }
            }
        }
        Ok(())
    }

    fn process_container_enter(
        &mut self,
        container: &ContainerBlockKind<'source>,
        range: SourceByteRange,
        depth: u32,
        list_kinds: &mut Vec<RawListKind>,
    ) -> Result<(), NoteIngestError> {
        match container {
            ContainerBlockKind::List {
                kind,
                ..
            } => {
                list_kinds.push(list_kind_to_raw(*kind));
            }
            ContainerBlockKind::ListItem {
                depth: item_depth,
                parent_pos,
                is_checked,
                children,
            } => {
                let kind = list_kinds
                    .last()
                    .copied()
                    .unwrap_or(RawListKind::Unordered);
                self.process_list_item(
                    kind,
                    *item_depth,
                    *parent_pos,
                    *is_checked,
                    children,
                    range,
                    depth,
                )?;
            }
            ContainerBlockKind::BlockQuote {
                ..
            } => {
                self.out.sections.push(RawSection::new(
                    RawSectionKind::BlockQuote,
                    range,
                    depth,
                ));
            }
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "internal refactor method matching RawListItem requirements"
    )]
    fn process_list_item(
        &mut self,
        list_kind: RawListKind,
        item_depth: u32,
        parent_pos: Option<SourceByteOffset>,
        is_checked: Option<bool>,
        children: &[Block<'source, Closed>],
        range: SourceByteRange,
        section_depth: u32,
    ) -> Result<(), NoteIngestError> {
        let mut text = String::new();
        let mut events = Vec::new();

        collect_text_recursively(children, &mut text, &mut events);
        self.extract_links_from_events(&events);

        let projection = TextSequence::from_events(&events);
        let scanned = self.scan_projection(&projection)?;
        let item_text_range =
            projection.covering_range().unwrap_or(range.clone());

        let item = RawListItem::new(
            list_kind,
            RawListDepth::from(item_depth),
            parent_pos,
            is_checked,
            RawListItemText::new(Cow::Owned(text), item_text_range),
            range.clone(),
            scanned.tags,
            scanned.inline_fields,
        );

        self.out.accept_list_item(item);
        self.out.sections.push(RawSection::new(
            RawSectionKind::List,
            range,
            section_depth,
        ));

        Ok(())
    }

    /// Shared processing for all leaf blocks.
    fn process_leaf_block(
        &mut self,
        leaf: &LeafBlockKind<'source, Closed>,
        range: SourceByteRange,
        depth: u32,
    ) -> Result<(), NoteIngestError> {
        match leaf {
            LeafBlockKind::Heading {
                level,
                events,
            } => {
                self.extract_links_from_events(events);
                let projection = TextSequence::from_events(events);
                let scanned = self.scan_projection(&projection)?;
                let text = projection.as_displayable_text();
                let trimmed = text.trim();
                let heading_text = if trimmed.len() == text.len() {
                    Cow::Owned(text)
                } else {
                    Cow::Owned(trimmed.to_owned())
                };

                self.out.headings.push(RawHeading::new(
                    level.as_u8(),
                    heading_text,
                    range.clone(),
                    range.start(),
                ));
                self.out.sections.push(RawSection::new(
                    RawSectionKind::Heading,
                    range,
                    depth,
                ));
                self.out.tags.extend(scanned.tags);
                self.out.inline_fields.extend(scanned.inline_fields);
                self.out.block_refs.extend(scanned.block_refs);
            }
            LeafBlockKind::Paragraph {
                events,
            } => {
                self.extract_links_from_events(events);
                let projection = TextSequence::from_events(events);
                let scanned = self.scan_projection(&projection)?;
                self.out.sections.push(RawSection::new(
                    RawSectionKind::Paragraph,
                    range,
                    depth,
                ));
                self.out.tags.extend(scanned.tags);
                self.out.inline_fields.extend(scanned.inline_fields);
                self.out.block_refs.extend(scanned.block_refs);
            }
            LeafBlockKind::Frontmatter {
                format,
                text,
            } => {
                self.out.sections.push(RawSection::new(
                    RawSectionKind::Frontmatter,
                    range.clone(),
                    0,
                ));
                self.out.frontmatter = Some(RawFrontmatter::new(
                    match format {
                        FrontmatterFormat::Yaml => RawFrontmatterFormat::Yaml,
                        FrontmatterFormat::Toml => RawFrontmatterFormat::Toml,
                    },
                    text.clone().into(),
                    range,
                ));
            }
            LeafBlockKind::ThematicBreak => {
                self.out.sections.push(RawSection::new(
                    RawSectionKind::ThematicBreak,
                    range,
                    depth,
                ));
            }
            LeafBlockKind::CodeBlock {
                ..
            } => {
                self.out.sections.push(RawSection::new(
                    RawSectionKind::CodeBlock,
                    range,
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

    fn extract_links_from_events(&mut self, events: &[RangedEvent<'source>]) {
        let mut i = 0;
        while let Some(event) = events.get(i) {
            if let ParserEvent::Inline(InlineToken::DelimiterStart(start)) =
                event.event()
                && let Some(mut link) =
                    Self::try_extract_link(start, events, &mut i)
            {
                link.position = event.range().start();
                self.out.links.push(link);
                continue;
            }
            i = i.saturating_add(1);
        }
    }

    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Manual event stream traversal with lookahead"
    )]
    fn try_extract_link(
        start: &InlineDelimiterStart<'source>,
        events: &[RangedEvent<'source>],
        index: &mut usize,
    ) -> Option<RawLink<'source>> {
        let (kind, destination, is_embed) = match start {
            InlineDelimiterStart::Link {
                kind,
                destination,
                ..
            } => (kind, destination, false),
            InlineDelimiterStart::Image {
                kind,
                destination,
                ..
            } => (kind, destination, true),
            InlineDelimiterStart::Emphasis
            | InlineDelimiterStart::Strong
            | InlineDelimiterStart::Strikethrough
            | InlineDelimiterStart::Superscript
            | InlineDelimiterStart::Subscript
            | InlineDelimiterStart::_Marker(_) => return None,
        };

        let mut raw_link = RawLink::new(
            RawLinkStyle::from(*kind),
            is_embed,
            destination.clone(),
            SourceByteOffset::default(),
        );

        let target_end = if is_embed {
            InlineDelimiterEnd::Image
        } else {
            InlineDelimiterEnd::Link
        };

        *index += 1;
        while let Some(inner_event) = events.get(*index) {
            if let ParserEvent::Inline(InlineToken::DelimiterEnd(end)) =
                inner_event.event()
                && *end == target_end
            {
                break;
            }

            // Accumulate display text
            if let ParserEvent::Inline(token) = inner_event.event() {
                match token {
                    InlineToken::Text(t)
                    | InlineToken::InlineCode(t)
                    | InlineToken::Html(t) => {
                        raw_link.text.display.push_str(t);
                    }
                    InlineToken::Math {
                        content,
                        ..
                    } => {
                        raw_link.text.display.push_str(content);
                    }
                    InlineToken::DelimiterStart(_)
                    | InlineToken::DelimiterEnd(_)
                    | InlineToken::LineBreak(_)
                    | InlineToken::FootnoteReference(_) => {}
                }
            }
            *index += 1;
        }

        if *index < events.len() {
            *index += 1;
        }

        Some(raw_link)
    }
}

fn list_kind_to_raw(kind: ListKind) -> RawListKind {
    match kind {
        ListKind::Ordered(start) => RawListKind::Ordered(start),
        ListKind::Unordered => RawListKind::Unordered,
    }
}

fn collect_text_recursively<'source>(
    children: &[Block<'source, Closed>],
    out_text: &mut String,
    out_events: &mut Vec<RangedEvent<'source>>,
) {
    for child in children {
        match &child.kind {
            BlockKind::Leaf(leaf) => match leaf {
                LeafBlockKind::Paragraph {
                    events,
                }
                | LeafBlockKind::Heading {
                    events,
                    ..
                } => {
                    let projection = TextSequence::from_events(events);
                    out_text.push_str(&projection.as_plain_text());
                    out_events.extend(events.iter().cloned());
                }
                LeafBlockKind::CodeBlock {
                    ..
                }
                | LeafBlockKind::Frontmatter {
                    ..
                }
                | LeafBlockKind::ThematicBreak => {}
            },
            BlockKind::Container(container) => match container {
                ContainerBlockKind::BlockQuote {
                    children: inner_children,
                } => {
                    collect_text_recursively(
                        inner_children,
                        out_text,
                        out_events,
                    );
                }
                // DO NOT recurse into nested lists or list items.
                // These belong to their own identity and should not
                // leak their content (tags, checkboxes) into the parent.
                ContainerBlockKind::List {
                    ..
                }
                | ContainerBlockKind::ListItem {
                    ..
                } => {}
            },
        }
    }
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

#[cfg(test)]
mod tests {}
