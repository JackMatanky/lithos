//! Markdown parser and extraction.
//!
//! This module provides the primary ingestion engine for Obsidian-compatible
//! markdown files. It uses a single-pass event stream driven by
//! `pulldown-cmark` to extract both structural components (headings, sections,
//! lists) and specialized metadata (tags, inline fields, block references,
//! frontmatter).
//!
//! The main entry point is [`MarkdownParser`].

use std::borrow::Cow;

use pulldown_cmark::{
    CowStr, Event, Options, Parser, TagEnd, utils::TextMergeWithOffset,
};

use crate::{
    config::task::TaskConfigSpec,
    note::{
        error::{NoteIngestError, NoteParseError, StructureError},
        inline_fields::InlineFieldKey,
        paths::NotePath,
        position::{SourceByteOffset, SourceByteRange},
        raw::{
            RawBlockRef, RawFrontmatter, RawFrontmatterFormat, RawHeading,
            RawInlineField, RawInlineFieldToken, RawLink, RawLinkStyle,
            RawList, RawListDepth, RawListItem, RawListKind, RawNote,
            RawSection, RawSectionKind, RawTag, RawTaskStatusSymbol,
        },
        scanner::NoteScanner,
    },
};

type ScannedArtifacts<'source> = (
    Vec<RawTag<'source>>,
    Vec<RawInlineField<'source>>,
    Vec<RawBlockRef<'source>>,
    Option<RawTaskStatusSymbol>,
);

type ReferenceMap = std::collections::HashMap<Box<str>, Box<str>>;

/// Markdown parser for extracting note facts and structure.
#[non_exhaustive]
pub struct MarkdownParser;

impl MarkdownParser {
    /// Parses markdown into a minimal AST and extracts raw note artifacts.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if:
    /// - The source length exceeds the representable range of
    ///   [`SourceByteOffset`].
    /// - Structural extraction fails due to internal parser inconsistencies.
    /// - Metadata extraction (tags, fields) encounters invalid position
    ///   mapping.
    #[inline]
    #[expect(
        clippy::too_many_lines,
        reason = "Event sink matches comprehensive logic"
    )]
    pub fn parse<'source>(
        markdown: &'source str,
        path: NotePath,
        task_spec: &crate::config::task::TaskConfigSpec,
    ) -> Result<RawNote<'source>, NoteIngestError> {
        let emoji_markers = if task_spec.use_emoji {
            task_spec.emoji_markers.clone()
        } else {
            Box::new([])
        };
        let scanner = NoteScanner::new(emoji_markers);
        let mut pool = StringPool::new();

        let _source_bytes: u64 =
            u64::try_from(markdown.len()).map_err(|_err| {
                #[expect(
                    clippy::as_conversions,
                    reason = "u32::MAX fits in usize"
                )]
                NoteParseError::SourceTooLarge {
                    size: markdown.len(),
                    limit: u32::MAX as usize,
                }
            })?;

        let parser = Parser::new_ext(markdown, Self::extension_options());
        let offset_iter = parser.into_offset_iter();

        // Collect reference definitions before consuming the iterator
        let ref_defs: std::collections::HashMap<Box<str>, Box<str>> =
            offset_iter
                .reference_definitions()
                .iter()
                .map(|(label, link_def)| {
                    (
                        normalize_reference_label(label),
                        link_def.dest.as_ref().into(),
                    )
                })
                .collect();

        let mut state = ParserState {
            block: BlockState {
                stack: Vec::with_capacity(4),
                depth: 0,
            },
            list: ListState {
                stack: Vec::with_capacity(4),
                contexts: Vec::with_capacity(4),
                open_item_by_depth: Vec::with_capacity(8),
            },
            link: LinkState {
                current: None,
                ref_defs,
            },
            meta: MetaState {
                in_metadata: None,
                buffer: pool.take(),
            },
            out: OutputState {
                headings: Vec::with_capacity(4),
                sections: Vec::with_capacity(8),
                links: Vec::with_capacity(8),
                tags: Vec::with_capacity(8),
                lists: Vec::with_capacity(4),
                list_items: Vec::with_capacity(8),
                inline_fields: Vec::with_capacity(8),
                block_refs: Vec::with_capacity(8),
                frontmatter: None,
            },
            pool,
            task_spec,
        };

        let events = offset_iter.map(|(event, range)| {
            let normalized = match event {
                Event::SoftBreak => Event::Text(CowStr::Borrowed(" ")),
                Event::HardBreak => Event::Text(CowStr::Borrowed("\n")),
                Event::Start(_)
                | Event::End(_)
                | Event::Text(_)
                | Event::Code(_)
                | Event::InlineMath(_)
                | Event::DisplayMath(_)
                | Event::Html(_)
                | Event::InlineHtml(_)
                | Event::FootnoteReference(_)
                | Event::Rule
                | Event::TaskListMarker(_) => event,
            };
            (normalized, range)
        });
        let iter = TextMergeWithOffset::new(events);

        for (event, range) in iter {
            let start_pos =
                SourceByteOffset::try_from(range.start).map_err(|_err| {
                    #[expect(
                        clippy::as_conversions,
                        reason = "u32::MAX fits in usize"
                    )]
                    NoteParseError::SourceTooLarge {
                        size: range.start,
                        limit: u32::MAX as usize,
                    }
                })?;

            if MetadataHandler::on_event(&event, &range, &mut state)? {
                continue;
            }

            match event {
                Event::Start(pulldown_cmark::Tag::MetadataBlock(kind)) => {
                    state.meta.in_metadata = Some((kind, start_pos));
                    state.meta.buffer.clear();
                }
                Event::Start(tag) => {
                    StartTagHandler::on_start(tag, start_pos, &mut state);
                }
                Event::End(end_tag) => {
                    EndTagHandler::on_end(
                        end_tag, range, markdown, &scanner, &mut state,
                    )?;
                }
                Event::Text(text) => {
                    TextHandler::on_scannable_text(&text, &range, &mut state);
                }
                Event::Code(text) => {
                    TextHandler::on_unscannable_text(&text, &range, &mut state);
                }
                Event::TaskListMarker(checked) => {
                    if let Some(block) = state.block.stack.last_mut() {
                        block.task_marker = Some(checked);
                    }
                }
                Event::InlineMath(_)
                | Event::DisplayMath(_)
                | Event::Html(_)
                | Event::InlineHtml(_)
                | Event::FootnoteReference(_)
                | Event::Rule
                | Event::SoftBreak
                | Event::HardBreak => {}
            }
        }

        state.pool.put(state.meta.buffer);

        state
            .out
            .sections
            .sort_by_key(|section| u32::from(section.range.start()));

        Ok(RawNote::new(
            path,
            state.out.frontmatter,
            state.out.headings,
            state.out.sections,
            state.out.links,
            state.out.tags,
            state.out.lists,
            state.out.list_items,
            state.out.inline_fields,
            state.out.block_refs,
        ))
    }

    /// Returns the pulldown-cmark option set used for Obsidian-compatible
    /// parsing.
    #[inline]
    #[must_use]
    pub const fn extension_options() -> Options {
        Options::ENABLE_TASKLISTS
            .union(Options::ENABLE_WIKILINKS)
            .union(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS)
            .union(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS)
            .union(Options::ENABLE_STRIKETHROUGH)
    }

    #[inline]
    fn heading_level_value(level: pulldown_cmark::HeadingLevel) -> u8 {
        match level {
            pulldown_cmark::HeadingLevel::H1 => 1,
            pulldown_cmark::HeadingLevel::H2 => 2,
            pulldown_cmark::HeadingLevel::H3 => 3,
            pulldown_cmark::HeadingLevel::H4 => 4,
            pulldown_cmark::HeadingLevel::H5 => 5,
            pulldown_cmark::HeadingLevel::H6 => 6,
        }
    }

    fn scan_block_artifacts<'source>(
        scan: &ScanContext<'source, '_, '_>,
        scannable_ranges: &[std::ops::Range<usize>],
        include_task_marker: bool,
        block_range: SourceByteRange,
    ) -> Result<ScannedArtifacts<'source>, NoteIngestError> {
        let raw_tokens = scan
            .scanner
            .scan_ranges_raw(scan.source, scannable_ranges, include_task_marker)
            .map_err(NoteIngestError::Domain)?;
        let tags = raw_tokens.tags;
        let inline_fields = raw_tokens
            .inline_fields
            .into_iter()
            .map(|token| Self::inline_field_from_token(token, scan.task_spec))
            .collect();
        let mut block_refs = raw_tokens.block_refs;
        if block_refs.is_empty() {
            let tail_scannable = scannable_ranges
                .last()
                .is_some_and(|range| range.end == block_range.end().as_usize());
            if tail_scannable
                && let Some(tail_range) =
                    Self::block_ref_tail_range(scan.source, block_range)
            {
                let tail_tokens = scan
                    .scanner
                    .scan_ranges_raw(
                        scan.source,
                        std::slice::from_ref(&tail_range),
                        false,
                    )
                    .map_err(NoteIngestError::Domain)?;
                block_refs.extend(tail_tokens.block_refs);
            }
        }
        Ok((tags, inline_fields, block_refs, raw_tokens.task_marker))
    }

    fn inline_field_from_token<'source>(
        token: RawInlineFieldToken<'source>,
        task_spec: &TaskConfigSpec,
    ) -> RawInlineField<'source> {
        let RawInlineFieldToken {
            key,
            value,
            range,
        } = token;
        let normalized = InlineFieldKey::normalize(key.as_ref());
        let date_spec = task_spec
            .temporal_specs
            .get(normalized.as_ref())
            .map(|entry| entry.1.as_ref());
        let typed_value = match value {
            Cow::Borrowed(text) => {
                crate::note::raw::RawFieldValue::from_str_with_spec(
                    text,
                    key.as_ref(),
                    date_spec,
                )
            }
            Cow::Owned(text) => {
                crate::note::raw::RawFieldValue::from_str_with_spec(
                    &text,
                    key.as_ref(),
                    date_spec,
                )
                .into_owned()
            }
        };
        RawInlineField::new(key, typed_value, range)
    }

    #[inline]
    fn block_ref_tail_range(
        source: &str,
        block_range: SourceByteRange,
    ) -> Option<std::ops::Range<usize>> {
        let start = block_range.start().as_usize();
        let end = block_range.end().as_usize();
        if end <= start {
            return None;
        }
        let block_len = end.saturating_sub(start);
        let tail_len = block_len.min(512usize);
        let mut tail_start = end.saturating_sub(tail_len);
        if tail_start < start {
            tail_start = start;
        }
        while tail_start < end && !source.is_char_boundary(tail_start) {
            tail_start = tail_start.saturating_add(1);
        }
        let slice = source.get(tail_start..end)?;
        if !slice.contains('^') {
            return None;
        }
        Some(tail_start..end)
    }

    #[inline]
    fn finalize_paragraph<'source>(
        block: &ActiveBlock,
        block_range: SourceByteRange,
        scan: ScanContext<'source, '_, '_>,
        state: &mut ParserState<'source, '_>,
    ) -> Result<(), NoteIngestError> {
        state.out.sections.push(RawSection::new(
            RawSectionKind::Paragraph,
            block_range,
            block.depth,
        ));
        let (scan_tags, scan_fields, scan_refs, _marker) =
            Self::scan_block_artifacts(
                &scan,
                &block.scannable_ranges,
                false,
                block_range,
            )?;
        state.out.tags.extend(scan_tags);
        state.out.inline_fields.extend(scan_fields);
        state.out.block_refs.extend(scan_refs);

        if let Some(parent) = state.block.stack.last_mut()
            && matches!(parent.kind, BlockKind::ListItem)
            && parent.full_text.is_empty()
        {
            parent.full_text.push_str(&block.full_text);
        }
        Ok(())
    }

    fn finalize_list(
        block: &ActiveBlock,
        block_range: SourceByteRange,
        list_contexts: &mut Vec<ListContext>,
        lists: &mut Vec<RawList>,
    ) {
        let Some(context) = list_contexts.pop() else {
            return;
        };
        let list_depth = if block.depth <= 1 {
            RawListDepth::Root
        } else {
            RawListDepth::Nested(
                u8::try_from(block.depth.saturating_sub(1)).unwrap_or(u8::MAX),
            )
        };
        lists.push(RawList::new(
            context.kind,
            list_depth,
            block_range,
            context.item_positions,
        ));
    }

    #[expect(
        clippy::too_many_lines,
        reason = "List item finalization is complex by nature; contexts are a \
                  mutable stack in this parser"
    )]
    fn finalize_list_item<'source>(
        block: &ActiveBlock,
        block_range: SourceByteRange,
        scan: ScanContext<'source, '_, '_>,
        state: &mut ParserState<'source, '_>,
    ) -> Result<(), NoteIngestError> {
        state.out.sections.push(RawSection::new(
            RawSectionKind::List,
            block_range,
            block.depth,
        ));
        let is_checked = block.task_marker;
        let (scan_tags, scan_fields, scan_refs, _) =
            Self::scan_block_artifacts(
                &scan,
                &block.scannable_ranges,
                false,
                block_range,
            )?;

        state.out.block_refs.extend(scan_refs);

        // Only scan first line for task marker (checkboxes always at start)
        let task_marker = if is_checked.is_some() {
            let block_start = block_range.start().as_usize();
            let block_slice = scan.source.get(block_start..).unwrap_or("");
            let first_line_len =
                block_slice.find('\n').unwrap_or(block_slice.len()).min(80); // Cap at 80 chars (checkboxes are always near start)

            let first_line_end = SourceByteOffset::try_from(
                block_start.saturating_add(first_line_len),
            )
            .map_err(|_err| {
                #[expect(
                    clippy::as_conversions,
                    reason = "u32::MAX fits in usize"
                )]
                NoteIngestError::Domain(
                    crate::note::error::StructureError::OutOfBounds {
                        offset: block_start.saturating_add(first_line_len),
                        source_len: u32::MAX as usize,
                    }
                    .into(),
                )
            })?;

            let prefix_range =
                SourceByteRange::new(block_range.start(), first_line_end)
                    .map_err(NoteIngestError::Domain)?;

            scan.scanner
                .scan_task_marker(scan.source, prefix_range)
                .map_err(NoteIngestError::Domain)?
        } else {
            None
        };

        let list_kind =
            state.list.stack.last().copied().unwrap_or(RawListKind::Unordered);
        let list_depth = if block.depth <= 1 {
            RawListDepth::Root
        } else {
            RawListDepth::Nested(
                u8::try_from(block.depth.saturating_sub(1)).unwrap_or(u8::MAX),
            )
        };
        let depth_index = usize::try_from(block.depth).unwrap_or(0);
        let parent_pos = if block.depth <= 1 {
            None
        } else {
            state
                .list
                .open_item_by_depth
                .get(depth_index.saturating_sub(1))
                .copied()
        };

        if is_checked.is_some() && task_marker.is_none() {
            tracing::warn!(
                range_start = u32::from(block_range.start()),
                range_end = u32::from(block_range.end()),
                checked = is_checked,
                "Checkbox detected without task marker; treating as scan \
                 failure"
            );
        }
        let raw_text = block.full_text.trim().to_owned();
        let text_range = if raw_text.is_empty() {
            SourceByteRange::new(block_range.start(), block_range.start())
                .map_err(NoteIngestError::Domain)?
        } else {
            let leading_trim = block
                .full_text
                .len()
                .saturating_sub(block.full_text.trim_start().len());
            let base_start = block
                .scannable_ranges
                .first()
                .and_then(|range| SourceByteOffset::try_from(range.start).ok())
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

        if let Some(context_list) = state.list.contexts.last_mut() {
            context_list.item_positions.push(block.start_offset);
        }

        let raw_list_item = RawListItem::new(
            list_kind,
            list_depth,
            raw_text.into(),
            is_checked,
            task_marker,
            block_range,
            text_range,
            parent_pos,
            scan_tags,
            scan_fields,
        );
        state.out.tags.extend(raw_list_item.tags.clone());
        state.out.inline_fields.extend(raw_list_item.inline_fields.clone());
        state.out.list_items.push(raw_list_item);

        Ok(())
    }
}

impl MetadataHandler {
    fn on_event(
        event: &Event<'_>,
        range: &std::ops::Range<usize>,
        state: &mut ParserState<'_, '_>,
    ) -> Result<bool, NoteIngestError> {
        let Some((kind, start_offset)) = state.meta.in_metadata else {
            return Ok(false);
        };
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Match ergonomics on &Event keep metadata handling \
                      concise"
        )]
        match event {
            Event::Text(t) | Event::Code(t) => state.meta.buffer.push_str(t),
            Event::End(TagEnd::MetadataBlock(_)) => {
                state.meta.in_metadata = None;
                let end_pos =
                    SourceByteOffset::try_from(range.end).map_err(|_err| {
                        #[expect(
                            clippy::as_conversions,
                            reason = "u32::MAX fits in usize"
                        )]
                        NoteIngestError::Domain(
                            StructureError::OutOfBounds {
                                offset: range.end,
                                source_len: u32::MAX as usize,
                            }
                            .into(),
                        )
                    })?;
                let block_range = SourceByteRange::new(start_offset, end_pos)
                    .map_err(NoteIngestError::Domain)?;
                state.out.sections.push(RawSection::new(
                    RawSectionKind::Frontmatter,
                    block_range,
                    0,
                ));
                state.out.frontmatter = Some(RawFrontmatter::new(
                    kind.into(),
                    std::mem::take(&mut state.meta.buffer).into(),
                    block_range,
                ));
            }
            Event::Start(_)
            | Event::End(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
            | Event::Rule
            | Event::TaskListMarker(_)
            | Event::SoftBreak
            | Event::HardBreak => {}
        }
        Ok(true)
    }
}

impl StartTagHandler {
    #[expect(
        clippy::too_many_lines,
        reason = "Start tag handler mirrors pulldown-cmark tag variants"
    )]
    fn on_start<'source>(
        tag: pulldown_cmark::Tag<'source>,
        start_pos: SourceByteOffset,
        state: &mut ParserState<'source, '_>,
    ) {
        let kind = match tag {
            pulldown_cmark::Tag::Heading {
                level,
                ..
            } => Some(BlockKind::Heading(MarkdownParser::heading_level_value(
                level,
            ))),
            pulldown_cmark::Tag::Paragraph => Some(BlockKind::Paragraph),
            pulldown_cmark::Tag::Item => Some(BlockKind::ListItem),
            pulldown_cmark::Tag::List(list_start) => {
                let list_kind = match list_start {
                    Some(start) => RawListKind::Ordered(start),
                    None => RawListKind::Unordered,
                };
                state.list.stack.push(list_kind);
                state.list.contexts.push(ListContext::new(list_kind));
                Some(BlockKind::List)
            }
            pulldown_cmark::Tag::BlockQuote(_) => Some(BlockKind::BlockQuote),
            pulldown_cmark::Tag::CodeBlock(_) => Some(BlockKind::CodeBlock),
            pulldown_cmark::Tag::Link {
                link_type,
                dest_url,
                ..
            } => {
                let target = resolve_reference_target(
                    link_type,
                    dest_url,
                    &state.link.ref_defs,
                );
                state.link.current = Some(LinkFrame {
                    style: link_type.into(),
                    is_embed: false,
                    target,
                    start: start_pos,
                    alias: state.pool.take(),
                });
                None
            }
            pulldown_cmark::Tag::Image {
                link_type,
                dest_url,
                ..
            } => {
                let is_embed = matches!(
                    link_type,
                    pulldown_cmark::LinkType::WikiLink { .. }
                );
                let target = resolve_reference_target(
                    link_type,
                    dest_url,
                    &state.link.ref_defs,
                );
                state.link.current = Some(LinkFrame {
                    style: link_type.into(),
                    is_embed,
                    target,
                    start: start_pos,
                    alias: state.pool.take(),
                });
                None
            }
            pulldown_cmark::Tag::HtmlBlock
            | pulldown_cmark::Tag::FootnoteDefinition(_)
            | pulldown_cmark::Tag::DefinitionList
            | pulldown_cmark::Tag::DefinitionListTitle
            | pulldown_cmark::Tag::DefinitionListDefinition
            | pulldown_cmark::Tag::Table(_)
            | pulldown_cmark::Tag::TableHead
            | pulldown_cmark::Tag::TableRow
            | pulldown_cmark::Tag::TableCell
            | pulldown_cmark::Tag::Emphasis
            | pulldown_cmark::Tag::Strong
            | pulldown_cmark::Tag::Strikethrough
            | pulldown_cmark::Tag::Superscript
            | pulldown_cmark::Tag::Subscript
            | pulldown_cmark::Tag::MetadataBlock(_) => None,
        };

        if let Some(bkind) = kind {
            let current_depth = state.block.depth;
            if matches!(bkind, BlockKind::List | BlockKind::BlockQuote) {
                state.block.depth = state.block.depth.saturating_add(1);
            }
            if matches!(bkind, BlockKind::ListItem) {
                let depth_index = usize::try_from(current_depth).unwrap_or(0);
                if state.list.open_item_by_depth.len() <= depth_index {
                    state.list.open_item_by_depth.resize(
                        depth_index.saturating_add(1),
                        SourceByteOffset::new(0),
                    );
                }
                if let Some(slot) =
                    state.list.open_item_by_depth.get_mut(depth_index)
                {
                    *slot = start_pos;
                }
                state
                    .list
                    .open_item_by_depth
                    .truncate(depth_index.saturating_add(1));
            }
            state.block.stack.push(ActiveBlock {
                kind: bkind,
                depth: current_depth,
                start_offset: start_pos,
                full_text: state.pool.take(),
                scannable_ranges: Vec::with_capacity(4),
                task_marker: None,
            });
        }
    }
}

impl EndTagHandler {
    #[expect(
        clippy::too_many_lines,
        reason = "End tag handler finalizes multiple block kinds"
    )]
    fn on_end<'source>(
        end_tag: pulldown_cmark::TagEnd,
        range: std::ops::Range<usize>,
        markdown: &'source str,
        scanner: &NoteScanner,
        state: &mut ParserState<'source, '_>,
    ) -> Result<(), NoteIngestError> {
        match end_tag {
            pulldown_cmark::TagEnd::Link | pulldown_cmark::TagEnd::Image => {
                if let Some(mut link) = state.link.current.take() {
                    let (target_raw, anchor_raw) =
                        LinkTarget::new(link.target).split();
                    let alias_raw = trim_to_opt(&link.alias);

                    state.out.links.push(RawLink::new(
                        link.style,
                        link.is_embed,
                        target_raw,
                        alias_raw.map(Into::into),
                        anchor_raw,
                        link.start,
                    ));
                    state.pool.put(std::mem::take(&mut link.alias));
                }
            }
            pulldown_cmark::TagEnd::Heading(_)
            | pulldown_cmark::TagEnd::Paragraph
            | pulldown_cmark::TagEnd::Item
            | pulldown_cmark::TagEnd::List(_)
            | pulldown_cmark::TagEnd::BlockQuote(_)
            | pulldown_cmark::TagEnd::CodeBlock => {
                if let Some(mut block) = state.block.stack.pop() {
                    if matches!(
                        block.kind,
                        BlockKind::List | BlockKind::BlockQuote
                    ) {
                        state.block.depth = state.block.depth.saturating_sub(1);
                    }

                    let end_pos = SourceByteOffset::try_from(range.end)
                        .map_err(|_err| {
                            #[expect(
                                clippy::as_conversions,
                                reason = "u32::MAX fits in usize"
                            )]
                            NoteIngestError::Domain(
                                StructureError::OutOfBounds {
                                    offset: range.end,
                                    source_len: u32::MAX as usize,
                                }
                                .into(),
                            )
                        })?;
                    let block_range =
                        SourceByteRange::new(block.start_offset, end_pos)
                            .map_err(NoteIngestError::Domain)?;

                    let scan = ScanContext {
                        scanner,
                        source: markdown,
                        task_spec: state.task_spec,
                    };

                    match block.kind {
                        BlockKind::Heading(level) => {
                            state.out.headings.push(RawHeading::new(
                                level,
                                block.full_text.trim().to_owned().into(),
                                block_range,
                                block.start_offset,
                            ));
                            state.out.sections.push(RawSection::new(
                                RawSectionKind::Heading,
                                block_range,
                                state.block.depth,
                            ));
                            let (scan_tags, scan_fields, scan_refs, _marker) =
                                MarkdownParser::scan_block_artifacts(
                                    &scan,
                                    &block.scannable_ranges,
                                    false,
                                    block_range,
                                )?;
                            state.out.tags.extend(scan_tags);
                            state.out.inline_fields.extend(scan_fields);
                            state.out.block_refs.extend(scan_refs);
                        }
                        BlockKind::Paragraph => {
                            MarkdownParser::finalize_paragraph(
                                &block,
                                block_range,
                                scan,
                                state,
                            )?;
                        }
                        BlockKind::ListItem => {
                            MarkdownParser::finalize_list_item(
                                &block,
                                block_range,
                                scan,
                                state,
                            )?;
                        }
                        BlockKind::List => {
                            state.list.stack.pop();
                            MarkdownParser::finalize_list(
                                &block,
                                block_range,
                                &mut state.list.contexts,
                                &mut state.out.lists,
                            );
                        }
                        BlockKind::BlockQuote => {
                            state.out.sections.push(RawSection::new(
                                RawSectionKind::BlockQuote,
                                block_range,
                                block.depth,
                            ));
                        }
                        BlockKind::CodeBlock => {
                            state.out.sections.push(RawSection::new(
                                RawSectionKind::CodeBlock,
                                block_range,
                                block.depth,
                            ));
                        }
                    }
                    state.pool.put(std::mem::take(&mut block.full_text));
                }
            }
            pulldown_cmark::TagEnd::HtmlBlock
            | pulldown_cmark::TagEnd::FootnoteDefinition
            | pulldown_cmark::TagEnd::DefinitionList
            | pulldown_cmark::TagEnd::DefinitionListTitle
            | pulldown_cmark::TagEnd::DefinitionListDefinition
            | pulldown_cmark::TagEnd::Table
            | pulldown_cmark::TagEnd::TableHead
            | pulldown_cmark::TagEnd::TableRow
            | pulldown_cmark::TagEnd::TableCell
            | pulldown_cmark::TagEnd::Emphasis
            | pulldown_cmark::TagEnd::Strong
            | pulldown_cmark::TagEnd::Strikethrough
            | pulldown_cmark::TagEnd::Superscript
            | pulldown_cmark::TagEnd::Subscript
            | TagEnd::MetadataBlock(_) => {}
        }
        Ok(())
    }
}

impl TextHandler {
    fn on_scannable_text(
        text: &pulldown_cmark::CowStr<'_>,
        range: &std::ops::Range<usize>,
        state: &mut ParserState<'_, '_>,
    ) {
        let text_str = text.as_ref();
        if let Some(block) = state.block.stack.last_mut() {
            block.full_text.push_str(text_str);
            if state.link.current.is_none() {
                block.scannable_ranges.push(range.clone());
            }
        }
        if let Some(link) = state.link.current.as_mut() {
            link.alias.push_str(text_str);
        }
    }

    fn on_unscannable_text(
        text: &pulldown_cmark::CowStr<'_>,
        _range: &std::ops::Range<usize>,
        state: &mut ParserState<'_, '_>,
    ) {
        let text_str = text.as_ref();
        if let Some(block) = state.block.stack.last_mut() {
            block.full_text.push_str(text_str);
        }
        if let Some(link) = state.link.current.as_mut() {
            link.alias.push_str(text_str);
        }
    }
}

struct ParserState<'source, 'spec> {
    block: BlockState,
    list: ListState,
    link: LinkState<'source>,
    meta: MetaState,
    out: OutputState<'source>,
    pool: StringPool,
    task_spec: &'spec TaskConfigSpec,
}

struct BlockState {
    stack: Vec<ActiveBlock>,
    depth: u32,
}

struct ListState {
    stack: Vec<RawListKind>,
    contexts: Vec<ListContext>,
    open_item_by_depth: Vec<SourceByteOffset>,
}

struct LinkState<'source> {
    current: Option<LinkFrame<'source>>,
    ref_defs: ReferenceMap,
}

struct MetaState {
    in_metadata: Option<(pulldown_cmark::MetadataBlockKind, SourceByteOffset)>,
    buffer: String,
}

struct OutputState<'source> {
    headings: Vec<RawHeading<'source>>,
    sections: Vec<RawSection>,
    links: Vec<RawLink<'source>>,
    tags: Vec<RawTag<'source>>,
    lists: Vec<RawList>,
    list_items: Vec<RawListItem<'source>>,
    inline_fields: Vec<RawInlineField<'source>>,
    block_refs: Vec<RawBlockRef<'source>>,
    frontmatter: Option<RawFrontmatter<'source>>,
}

#[derive(Clone, Copy)]
struct ScanContext<'source, 'spec, 'scan> {
    scanner: &'scan NoteScanner,
    source: &'source str,
    task_spec: &'spec TaskConfigSpec,
}

struct MetadataHandler;
struct StartTagHandler;
struct EndTagHandler;
struct TextHandler;

struct ActiveBlock {
    kind: BlockKind,
    depth: u32,
    start_offset: SourceByteOffset,
    full_text: String,
    scannable_ranges: Vec<std::ops::Range<usize>>,
    task_marker: Option<bool>,
}

struct ListContext {
    kind: RawListKind,
    item_positions: Vec<SourceByteOffset>,
}

impl ListContext {
    fn new(kind: RawListKind) -> Self {
        Self {
            kind,
            item_positions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum BlockKind {
    Heading(u8),
    Paragraph,
    ListItem,
    List,
    BlockQuote,
    CodeBlock,
}

fn trim_to_opt(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

struct LinkFrame<'source> {
    style: RawLinkStyle,
    is_embed: bool,
    target: Cow<'source, str>,
    start: SourceByteOffset,
    alias: String,
}

struct LinkTarget<'source>(Cow<'source, str>);
impl<'source> LinkTarget<'source> {
    fn new(target: Cow<'source, str>) -> Self {
        Self(target)
    }

    fn split(self) -> (Cow<'source, str>, Option<Cow<'source, str>>) {
        if self.is_external() {
            return (self.0, None);
        }
        match self.0 {
            Cow::Borrowed(text) => text
                .split_once('#')
                .map_or((Cow::Borrowed(text), None), |(p, a)| {
                    (Cow::Borrowed(p), Some(Cow::Borrowed(a)))
                }),
            Cow::Owned(mut text) => {
                if let Some(pos) = text.find('#') {
                    #[expect(
                        clippy::arithmetic_side_effects,
                        reason = "pos is from find(), always < text.len()"
                    )]
                    let anchor = text.split_off(pos + 1);
                    text.truncate(pos);
                    (Cow::Owned(text), Some(Cow::Owned(anchor)))
                } else {
                    (Cow::Owned(text), None)
                }
            }
        }
    }

    fn is_external(&self) -> bool {
        crate::note::link::Target::is_external_target(self.0.as_ref())
    }
}

/// Metrics collected during `StringPool` operations for benchmarking.
#[derive(Default, Debug, Clone, Copy)]
#[non_exhaustive]
pub struct StringPoolMetrics {
    /// Number of times `take()` was called (strings requested from pool).
    pub takes: usize,
    /// Number of times `put()` was called (strings returned to pool).
    pub puts: usize,
    /// Current number of strings held in the pool.
    pub pool_size: usize,
    /// Current capacity of the pool's internal vector.
    pub pool_capacity: usize,
}

thread_local! {
    static STRING_POOL_METRICS: std::cell::RefCell<StringPoolMetrics> =
        std::cell::RefCell::new(StringPoolMetrics::default());
}

/// Retrieve the current `StringPool` metrics.
///
/// These metrics are accumulated during parsing operations.
#[inline]
#[must_use]
pub fn get_string_pool_metrics() -> StringPoolMetrics {
    STRING_POOL_METRICS.with(|cell| *cell.borrow())
}

/// Reset `StringPool` metrics to zero.
///
/// Call this before a benchmark iteration to start fresh metrics collection.
#[inline]
pub fn reset_string_pool_metrics() {
    STRING_POOL_METRICS
        .with(|cell| *cell.borrow_mut() = StringPoolMetrics::default());
}

struct StringPool {
    pool: Vec<String>,
}
impl StringPool {
    fn new() -> Self {
        Self {
            pool: Vec::with_capacity(16),
        }
    }

    #[expect(
        clippy::arithmetic_side_effects,
        reason = "metrics are user-controlled instrumentation"
    )]
    fn take(&mut self) -> String {
        STRING_POOL_METRICS.with(|cell| {
            let mut metrics = cell.borrow_mut();
            metrics.takes += 1;
            metrics.pool_size = self.pool.len();
            metrics.pool_capacity = self.pool.capacity();
        });
        self.pool.pop().unwrap_or_else(|| String::with_capacity(128))
    }

    #[expect(
        clippy::arithmetic_side_effects,
        reason = "metrics are user-controlled instrumentation"
    )]
    fn put(&mut self, mut s: String) {
        s.clear();
        self.pool.push(s);
        STRING_POOL_METRICS.with(|cell| {
            let mut metrics = cell.borrow_mut();
            metrics.puts += 1;
            metrics.pool_size = self.pool.len();
            metrics.pool_capacity = self.pool.capacity();
        });
    }
}

impl From<pulldown_cmark::MetadataBlockKind> for RawFrontmatterFormat {
    #[inline]
    fn from(kind: pulldown_cmark::MetadataBlockKind) -> Self {
        match kind {
            pulldown_cmark::MetadataBlockKind::YamlStyle => Self::Yaml,
            pulldown_cmark::MetadataBlockKind::PlusesStyle => Self::Toml,
        }
    }
}

impl From<pulldown_cmark::LinkType> for RawLinkStyle {
    #[inline]
    fn from(kind: pulldown_cmark::LinkType) -> Self {
        match kind {
            pulldown_cmark::LinkType::WikiLink {
                ..
            } => Self::Wiki,
            pulldown_cmark::LinkType::Inline
            | pulldown_cmark::LinkType::Reference
            | pulldown_cmark::LinkType::ReferenceUnknown
            | pulldown_cmark::LinkType::Collapsed
            | pulldown_cmark::LinkType::CollapsedUnknown
            | pulldown_cmark::LinkType::Shortcut
            | pulldown_cmark::LinkType::ShortcutUnknown
            | pulldown_cmark::LinkType::Autolink
            | pulldown_cmark::LinkType::Email => Self::Markdown,
        }
    }
}

#[inline]
fn is_reference_link_type(link_type: pulldown_cmark::LinkType) -> bool {
    matches!(
        link_type,
        pulldown_cmark::LinkType::Reference
            | pulldown_cmark::LinkType::ReferenceUnknown
            | pulldown_cmark::LinkType::Collapsed
            | pulldown_cmark::LinkType::CollapsedUnknown
            | pulldown_cmark::LinkType::Shortcut
            | pulldown_cmark::LinkType::ShortcutUnknown
    )
}

fn resolve_reference_target<'source>(
    link_type: pulldown_cmark::LinkType,
    dest_url: CowStr<'source>,
    reference_map: &ReferenceMap,
) -> Cow<'source, str> {
    if is_reference_link_type(link_type) {
        let normalized = normalize_reference_label(dest_url.as_ref());
        if let Some(resolved) = reference_map.get(normalized.as_ref()) {
            return Cow::Owned(String::from(resolved.as_ref()));
        }
    }
    Cow::from(dest_url)
}

fn normalize_reference_label(label: &str) -> Box<str> {
    let needs_lowercase = label.chars().any(|c| c.is_ascii_uppercase());
    let needs_whitespace_fix = label.starts_with([' ', '\t'])
        || label.ends_with([' ', '\t'])
        || label.contains("  ")
        || label.contains('\\');

    if !needs_lowercase && !needs_whitespace_fix {
        return label.to_owned().into_boxed_str();
    }

    let mut normalized = String::with_capacity(label.len());
    let mut last_was_space = false;
    let mut chars = label.chars().peekable();

    while let Some(ch) = chars.next() {
        let ch = if ch == '\\' {
            chars.next().unwrap_or('\\')
        } else {
            ch
        };

        if ch.is_whitespace() {
            if normalized.is_empty() || last_was_space {
                continue;
            }
            normalized.push(' ');
            last_was_space = true;
            continue;
        }

        normalized.push(ch.to_ascii_lowercase());
        last_was_space = false;
    }

    if last_was_space {
        normalized.pop();
    }

    normalized.into_boxed_str()
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;
    use crate::{
        config::task::TaskConfigSpec,
        note::raw::{RawFieldValue, RawTaskMarker},
    };

    fn task_spec_fixture() -> TaskConfigSpec {
        TaskConfigSpec {
            enabled: true,
            use_emoji: true,
            emoji_markers: vec![
                '\u{1f4c5}', // 📅
                '\u{2705}',  // ✅
                '\u{23f0}',  // ⏰
                '\u{1f6eb}', // 🛫
                '\u{23f3}',  // ⏳
            ]
            .into(),
            promotion_tags: vec!["task".into()].into(),
            status_mappings: std::collections::HashMap::new(),
            temporal_specs: std::collections::HashMap::new(),
            field_specs: std::collections::HashMap::new(),
        }
    }

    fn parse_raw(markdown: &str) -> RawNote<'_> {
        let path = crate::note::paths::NotePath::try_new("test.md")
            .expect("valid test path");
        let task_spec = task_spec_fixture();
        MarkdownParser::parse(markdown, path, &task_spec)
            .expect("parsing failed")
    }

    #[test]
    fn should_extract_block_ref_from_paragraph_tail() {
        let md = "Paragraph text ^my-id";
        let raw = parse_raw(md);
        assert_eq!(raw.block_refs.len(), 1);
        assert_eq!(raw.block_refs.first().unwrap().id, "my-id");
    }

    #[test]
    fn should_capture_yaml_at_start() {
        let md = "---\ntags: [a]\n---\nContent";
        let raw = parse_raw(md);
        let fm = raw.frontmatter.as_ref().expect("frontmatter missing");
        assert_eq!(fm.text, "tags: [a]\n");
    }

    #[test]
    fn should_capture_tags_inside_heading() {
        let md = "## Heading #tag";
        let raw = parse_raw(md);
        assert!(raw.tags.iter().any(|t| t.value == "#tag"));
    }

    #[test]
    fn should_ignore_tags_inside_links() {
        let md = "See [[target|#tag]] and [link #tag](http://example.test)";
        let raw = parse_raw(md);
        assert!(raw.tags.is_empty());
    }

    #[test]
    fn should_ignore_block_refs_inside_links() {
        let md = "See [link ^ref](http://example.test)";
        let raw = parse_raw(md);
        assert!(raw.block_refs.is_empty());
    }

    #[test]
    fn should_extract_bare_fields() {
        let md = "bare_key:: bare_val";
        let raw = parse_raw(md);
        let field = raw.inline_fields.first().expect("field exists");
        assert_eq!(field.key, "bare_key");
        assert_eq!(field.value, RawFieldValue::String("bare_val".into()));
    }

    #[test]
    fn should_handle_wikilinks() {
        let md = "Check [[target]] and [[target|alias]]";
        let raw = parse_raw(md);
        assert_eq!(raw.links.len(), 2);
        assert_eq!(raw.links.first().unwrap().target.as_ref(), "target");
    }

    #[test]
    fn should_track_list_nesting() {
        let md = "- Parent\n  - Child";
        let raw = parse_raw(md);
        assert_eq!(raw.list_items.len(), 2);
        let mut sorted = raw.list_items.clone();
        sorted.sort_by_key(|i| i.range.start().as_usize());
        let child = sorted.get(1).expect("Child list item must exist");
        assert!(matches!(child.depth, RawListDepth::Nested(1)));
    }

    #[test]
    fn should_capture_checkbox_state_and_marker() {
        let md = "- [x] Done";
        let raw = parse_raw(md);
        let item = raw.list_items.first().expect("list item exists");
        assert_eq!(item.is_checked, Some(true));
        assert!(matches!(
            item.task_marker.map(|s| s.marker),
            Some(RawTaskMarker::Checked('x'))
        ));
    }

    #[test]
    fn reference_definitions_first_wins() {
        let md = "[ref]: http://a.example\n[ref]: http://b.example\n\n[ref][]";
        let raw = parse_raw(md);
        let link = raw.links.first().expect("link exists");
        assert_eq!(link.target.as_ref(), "http://a.example");
    }

    #[test]
    fn reference_definitions_are_case_insensitive() {
        let md = "[Ref]: http://example.test\n\n[ref][]";
        let raw = parse_raw(md);
        let link = raw.links.first().expect("link exists");
        assert_eq!(link.target.as_ref(), "http://example.test");
    }

    #[test]
    fn reference_definitions_in_frontmatter_are_ignored() {
        let md = "---\n[ref]: http://frontmatter.test\n---\n\n[ref][]";
        let raw = parse_raw(md);
        assert!(raw.links.is_empty());
    }

    #[test]
    fn reference_definitions_in_fenced_code_are_ignored() {
        let md = "```\n[ref]: http://code.test\n```\n\n[ref][]";
        let raw = parse_raw(md);
        assert!(raw.links.is_empty());
    }

    #[test]
    fn reference_definitions_normalize_whitespace() {
        let md = "[Foo   Bar]: http://example.test\n\n[foo bar][]";
        let raw = parse_raw(md);
        let link = raw.links.first().expect("link exists");
        assert_eq!(link.target.as_ref(), "http://example.test");
    }

    #[test]
    fn reference_definitions_unescape_labels() {
        let md = "[Foo\\ Bar]: http://example.test\n\n[foo\\ bar][]";
        let raw = parse_raw(md);
        let link = raw.links.first().expect("link exists");
        assert_eq!(link.target.as_ref(), "http://example.test");
    }

    #[test]
    fn reference_definitions_allow_multiline_destination() {
        let md = "[ref]:\n  http://example.test\n\n[ref][]";
        let raw = parse_raw(md);
        let link = raw.links.first().expect("link exists");
        assert_eq!(link.target.as_ref(), "http://example.test");
    }

    #[test]
    fn external_scheme_targets_preserve_fragments() {
        let md = "[obsidian](obsidian://open?vault=V#frag)";
        let raw = parse_raw(md);
        let link = raw.links.first().expect("link exists");
        assert_eq!(link.target.as_ref(), "obsidian://open?vault=V#frag");
        assert!(link.anchor.is_none());
    }

    #[test]
    fn file_scheme_targets_preserve_fragments() {
        let md = "[file](file:///Users/example/test.md#section)";
        let raw = parse_raw(md);
        let link = raw.links.first().expect("link exists");
        assert_eq!(
            link.target.as_ref(),
            "file:///Users/example/test.md#section"
        );
        assert!(link.anchor.is_none());
    }

    #[test]
    fn s3_scheme_targets_preserve_fragments() {
        let md = "[s3](s3://bucket/key#object)";
        let raw = parse_raw(md);
        let link = raw.links.first().expect("link exists");
        assert_eq!(link.target.as_ref(), "s3://bucket/key#object");
        assert!(link.anchor.is_none());
    }

    #[test]
    fn link_target_split_with_owned_string_and_anchor() {
        let owned = Cow::Owned(String::from("note#heading"));
        let target = LinkTarget::new(owned);
        let (path, anchor) = target.split();

        assert_eq!(path.as_ref(), "note");
        assert_eq!(
            anchor.as_ref().map(std::convert::AsRef::as_ref),
            Some("heading")
        );
    }

    #[test]
    fn link_target_split_with_owned_string_no_anchor() {
        let owned = Cow::Owned(String::from("note"));
        let target = LinkTarget::new(owned);
        let (path, anchor) = target.split();

        assert_eq!(path.as_ref(), "note");
        assert!(anchor.is_none());
    }

    #[test]
    fn link_target_split_with_owned_string_multiple_hashes() {
        let owned = Cow::Owned(String::from("note#a#b#c"));
        let target = LinkTarget::new(owned);
        let (path, anchor) = target.split();

        assert_eq!(path.as_ref(), "note");
        assert_eq!(
            anchor.as_ref().map(std::convert::AsRef::as_ref),
            Some("a#b#c")
        );
    }
}
