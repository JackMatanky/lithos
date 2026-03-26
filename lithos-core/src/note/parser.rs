//! Markdown parser and extraction.
//!
//! This module provides the primary ingestion engine for Obsidian-compatible
//! markdown files. It uses a single-pass event stream driven by
//! `pulldown-cmark` to extract both structural components (headings, sections,
//! lists) and specialized metadata (tags, inline fields, block references,
//! frontmatter).
//!
//! The main entry point is [`MarkdownParser`].

use std::{sync::Arc, time::SystemTime};

use pulldown_cmark::{
    Event, Options, Parser, TagEnd, utils::TextMergeWithOffset,
};

use crate::{
    config::frontmatter::FrontmatterConfigSpec,
    note::{
        error::{NoteIngestError, NoteParseError, StructureError},
        paths::NotePath,
        position::{SourceByteOffset, SourceByteRange},
        raw::{
            RawBlockRef, RawFieldValue, RawFrontmatter, RawFrontmatterFormat,
            RawHeading, RawInlineField, RawLink, RawLinkStyle, RawList,
            RawListDepth, RawListItem, RawListKind, RawNote, RawReferenceLink,
            RawSection, RawSectionKind, RawTag, RawTaskMarker,
        },
        scanner::{NoteScanner, ScannedArtifact},
    },
};

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
        clippy::too_many_arguments,
        reason = "Parser entrypoint carries required extraction context"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "Event sink matches comprehensive logic"
    )]
    pub fn parse<'source>(
        markdown: &'source str,
        path: NotePath,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
        frontmatter_spec: &Arc<FrontmatterConfigSpec>,
        task_spec: &Arc<crate::config::task::TaskConfigSpec>,
    ) -> Result<RawNote<'source>, NoteIngestError> {
        let emoji_markers = if task_spec.use_emoji {
            task_spec.emoji_markers.to_vec()
        } else {
            Vec::new()
        };
        let scanner = NoteScanner::new(emoji_markers);
        let master_artifacts =
            scanner.scan_block(markdown, SourceByteOffset::new(0))?;
        let mut pool = StringPool::new();

        let _source_bytes = u64::try_from(markdown.len()).map_err(|_err| {
            #[expect(clippy::as_conversions, reason = "u32::MAX fits in usize")]
            NoteParseError::SourceTooLarge {
                size: markdown.len(),
                limit: u32::MAX as usize,
            }
        })?;
        let source_hash =
            blake3::hash(markdown.as_bytes()).to_hex().to_string();

        let mut reference_links = Vec::new();
        let mut block_refs = Vec::new();

        let mut headings = Vec::with_capacity(16);
        let mut sections = Vec::with_capacity(32);
        let mut links = Vec::with_capacity(32);
        let mut tags = Vec::with_capacity(16);
        let mut lists = Vec::with_capacity(8);
        let mut list_items = Vec::with_capacity(32);
        let mut inline_fields = Vec::with_capacity(16);
        let mut frontmatter = None;

        let mut block_stack: Vec<ActiveBlock> = Vec::with_capacity(8);
        let mut list_stack: Vec<RawListKind> = Vec::with_capacity(8);
        let mut list_contexts: Vec<ListContext> = Vec::with_capacity(8);
        let mut open_item_by_depth: Vec<SourceByteOffset> =
            Vec::with_capacity(16);

        let mut depth: u32 = 0;

        let mut current_link: Option<LinkFrame> = None;
        let mut in_metadata: Option<(
            pulldown_cmark::MetadataBlockKind,
            SourceByteOffset,
        )> = None;
        let mut metadata_text = pool.take();

        let parser = Parser::new_ext(markdown, Self::obsidian_options());
        let events = parser.into_offset_iter();
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

            if Self::handle_metadata(
                event.clone(),
                range.clone(),
                &mut in_metadata,
                &mut metadata_text,
                &mut sections,
                frontmatter_spec,
                &mut frontmatter,
            )? {
                continue;
            }

            match event {
                Event::Start(pulldown_cmark::Tag::MetadataBlock(kind)) => {
                    in_metadata = Some((kind, start_pos));
                    metadata_text.clear();
                }
                Event::Start(tag) => {
                    Self::handle_start_tag(
                        tag,
                        start_pos,
                        &mut depth,
                        &mut block_stack,
                        &mut list_stack,
                        &mut list_contexts,
                        &mut current_link,
                        &mut open_item_by_depth,
                        task_spec,
                        &mut pool,
                    );
                }
                Event::End(end_tag) => {
                    Self::handle_end_tag(
                        end_tag,
                        range,
                        &mut depth,
                        &mut block_stack,
                        &mut list_stack,
                        &mut list_contexts,
                        &mut current_link,
                        &mut links,
                        &mut sections,
                        &mut headings,
                        &mut tags,
                        &mut inline_fields,
                        markdown,
                        &mut open_item_by_depth,
                        &mut list_items,
                        &mut lists,
                        &mut block_refs,
                        &master_artifacts,
                        task_spec,
                        &mut pool,
                    )?;
                }
                Event::Text(text) => {
                    Self::handle_text(
                        &text,
                        &range,
                        &mut block_stack,
                        &mut current_link,
                        true,
                    );
                }
                Event::Code(text) => {
                    Self::handle_text(
                        &text,
                        &range,
                        &mut block_stack,
                        &mut current_link,
                        false,
                    );
                }
                Event::SoftBreak | Event::HardBreak => {
                    Self::handle_break(&mut block_stack, &mut current_link);
                }
                Event::TaskListMarker(checked) => {
                    if let Some(block) = block_stack.last_mut() {
                        block.task_marker = Some(checked);
                    }
                }
                Event::InlineMath(_)
                | Event::DisplayMath(_)
                | Event::Html(_)
                | Event::InlineHtml(_)
                | Event::FootnoteReference(_)
                | Event::Rule => {}
            }
        }

        pool.put(metadata_text);

        let reference_defs: Vec<(String, String)> = {
            let parser_for_refs =
                Parser::new_ext(markdown, Self::obsidian_options());
            parser_for_refs
                .reference_definitions()
                .iter()
                .map(|(label, link_def)| {
                    let dest: String = link_def.dest.as_ref().to_owned();
                    (label.to_owned(), dest)
                })
                .collect()
        };
        for (label, dest) in reference_defs {
            reference_links.push(RawReferenceLink::new(
                label.into(),
                dest.into(),
                SourceByteOffset::new(0),
            ));
        }

        sections.sort_by_key(|section| u32::from(section.range.start()));

        Ok(RawNote::new(
            path,
            source_hash.into_boxed_str(),
            u64::try_from(markdown.len()).unwrap_or(0),
            created_at,
            modified_at,
            frontmatter,
            headings,
            sections,
            links,
            tags,
            lists,
            list_items,
            inline_fields,
            reference_links,
            block_refs,
            master_artifacts,
        ))
    }

    /// Returns the pulldown-cmark option set used for Obsidian-compatible
    /// parsing.
    #[inline]
    #[must_use]
    pub const fn obsidian_options() -> Options {
        Options::ENABLE_TASKLISTS
            .union(Options::ENABLE_WIKILINKS)
            .union(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS)
            .union(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS)
            .union(Options::ENABLE_HEADING_ATTRIBUTES)
            .union(Options::ENABLE_TABLES)
            .union(Options::ENABLE_FOOTNOTES)
            .union(Options::ENABLE_STRIKETHROUGH)
            .union(Options::ENABLE_MATH)
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

    #[expect(
        clippy::too_many_arguments,
        reason = "Parser orchestration requires full state context"
    )]
    fn handle_start_tag(
        tag: pulldown_cmark::Tag<'_>,
        start_pos: SourceByteOffset,
        depth: &mut u32,
        block_stack: &mut Vec<ActiveBlock>,
        list_stack: &mut Vec<RawListKind>,
        list_contexts: &mut Vec<ListContext>,
        current_link: &mut Option<LinkFrame>,
        open_item_by_depth: &mut Vec<SourceByteOffset>,
        task_spec: &Arc<crate::config::task::TaskConfigSpec>,
        pool: &mut StringPool,
    ) {
        let kind = match tag {
            pulldown_cmark::Tag::Heading {
                level,
                ..
            } => Some(BlockKind::Heading(Self::heading_level_value(level))),
            pulldown_cmark::Tag::Paragraph => Some(BlockKind::Paragraph),
            pulldown_cmark::Tag::Item => Some(BlockKind::ListItem),
            pulldown_cmark::Tag::List(list_start) => {
                let list_kind = match list_start {
                    Some(start) => RawListKind::Ordered(start),
                    None => RawListKind::Unordered,
                };
                list_stack.push(list_kind);
                list_contexts
                    .push(ListContext::new(list_kind, Arc::clone(task_spec)));
                Some(BlockKind::List)
            }
            pulldown_cmark::Tag::BlockQuote(_) => Some(BlockKind::BlockQuote),
            pulldown_cmark::Tag::CodeBlock(_) => Some(BlockKind::CodeBlock),
            pulldown_cmark::Tag::Link {
                link_type,
                dest_url,
                ..
            } => {
                *current_link = Some(LinkFrame {
                    style: link_type.into(),
                    is_embed: false,
                    target: dest_url.to_string(),
                    start: start_pos,
                    alias: pool.take(),
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
                *current_link = Some(LinkFrame {
                    style: link_type.into(),
                    is_embed,
                    target: dest_url.to_string(),
                    start: start_pos,
                    alias: pool.take(),
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
            let current_depth = *depth;
            if matches!(bkind, BlockKind::List | BlockKind::BlockQuote) {
                *depth = depth.saturating_add(1);
            }
            if matches!(bkind, BlockKind::ListItem) {
                let depth_index = usize::try_from(current_depth).unwrap_or(0);
                if open_item_by_depth.len() <= depth_index {
                    open_item_by_depth.resize(
                        depth_index.saturating_add(1),
                        SourceByteOffset::new(0),
                    );
                }
                if let Some(slot) = open_item_by_depth.get_mut(depth_index) {
                    *slot = start_pos;
                }
                open_item_by_depth.truncate(depth_index.saturating_add(1));
            }
            block_stack.push(ActiveBlock {
                kind: bkind,
                depth: current_depth,
                start_offset: start_pos,
                full_text: pool.take(),
                scannable_ranges: Vec::with_capacity(4),
                task_marker: None,
            });
        }
    }

    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "Handler needs access to full extraction state"
    )]
    fn handle_end_tag<'source>(
        end_tag: pulldown_cmark::TagEnd,
        range: std::ops::Range<usize>,
        depth: &mut u32,
        block_stack: &mut Vec<ActiveBlock>,
        list_stack: &mut Vec<RawListKind>,
        list_contexts: &mut Vec<ListContext>,
        current_link: &mut Option<LinkFrame>,
        links: &mut Vec<RawLink<'source>>,
        sections: &mut Vec<RawSection>,
        headings: &mut Vec<RawHeading<'source>>,
        tags: &mut Vec<RawTag<'source>>,
        inline_fields: &mut Vec<RawInlineField<'source>>,
        _markdown: &'source str,
        open_item_by_depth: &mut [SourceByteOffset],
        list_items: &mut Vec<RawListItem<'source>>,
        lists: &mut Vec<RawList>,
        block_refs: &mut Vec<RawBlockRef<'source>>,
        master_artifacts: &[ScannedArtifact<'source>],
        _task_spec: &Arc<crate::config::task::TaskConfigSpec>,
        pool: &mut StringPool,
    ) -> Result<(), NoteIngestError> {
        match end_tag {
            pulldown_cmark::TagEnd::Link | pulldown_cmark::TagEnd::Image => {
                if let Some(mut link) = current_link.take() {
                    let (target_raw, anchor_raw) =
                        LinkTarget::new(&link.target).split();
                    let alias_raw = if link.alias.is_empty() {
                        None
                    } else {
                        Some(link.alias.trim().to_owned())
                    };

                    links.push(RawLink::new(
                        link.style,
                        link.is_embed,
                        target_raw.to_owned().into(),
                        alias_raw.map(Into::into),
                        anchor_raw.map(|s| s.to_owned().into()),
                        link.start,
                    ));
                    pool.put(std::mem::take(&mut link.target));
                    pool.put(std::mem::take(&mut link.alias));
                }
            }
            pulldown_cmark::TagEnd::Heading(_)
            | pulldown_cmark::TagEnd::Paragraph
            | pulldown_cmark::TagEnd::Item
            | pulldown_cmark::TagEnd::List(_)
            | pulldown_cmark::TagEnd::BlockQuote(_)
            | pulldown_cmark::TagEnd::CodeBlock => {
                if let Some(mut block) = block_stack.pop() {
                    if matches!(
                        block.kind,
                        BlockKind::List | BlockKind::BlockQuote
                    ) {
                        *depth = depth.saturating_sub(1);
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

                    match block.kind {
                        BlockKind::Heading(level) => {
                            headings.push(RawHeading::new(
                                level,
                                block.full_text.trim().to_owned().into(),
                                block_range,
                                block.start_offset,
                            ));
                            sections.push(RawSection::new(
                                RawSectionKind::Heading,
                                block_range,
                                *depth,
                            ));
                            let scan_result = Self::filter_artifacts_by_range(
                                block_range,
                                &block.scannable_ranges,
                                master_artifacts,
                            );
                            tags.extend(scan_result.tags);
                            inline_fields.extend(scan_result.inline_fields);
                            block_refs.extend(scan_result.block_refs);
                        }

                        BlockKind::Paragraph => {
                            Self::finalize_paragraph(
                                &block,
                                block_range,
                                sections,
                                tags,
                                inline_fields,
                                block_stack,
                                block_refs,
                                master_artifacts,
                            );
                        }
                        BlockKind::ListItem => {
                            Self::finalize_list_item(
                                &block,
                                block_range,
                                sections,
                                list_stack,
                                open_item_by_depth,
                                list_items,
                                list_contexts,
                                block_refs,
                                master_artifacts,
                            )?;
                        }
                        BlockKind::List => {
                            list_stack.pop();
                            Self::finalize_list(
                                &block,
                                block_range,
                                list_contexts,
                                lists,
                            );
                        }
                        BlockKind::BlockQuote => {
                            sections.push(RawSection::new(
                                RawSectionKind::BlockQuote,
                                block_range,
                                block.depth,
                            ));
                        }
                        BlockKind::CodeBlock => {
                            sections.push(RawSection::new(
                                RawSectionKind::CodeBlock,
                                block_range,
                                block.depth,
                            ));
                        }
                    }
                    pool.put(std::mem::take(&mut block.full_text));
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

    fn handle_text(
        text: &pulldown_cmark::CowStr<'_>,
        range: &std::ops::Range<usize>,
        block_stack: &mut [ActiveBlock],
        current_link: &mut Option<LinkFrame>,
        is_scannable: bool,
    ) {
        let text_str = text.as_ref();
        if let Some(block) = block_stack.last_mut() {
            block.full_text.push_str(text_str);
            if current_link.is_none() && is_scannable {
                block.scannable_ranges.push(range.clone());
            }
        }
        if let Some(link) = current_link.as_mut() {
            link.alias.push_str(text_str);
        }
    }

    fn handle_break(
        block_stack: &mut [ActiveBlock],
        current_link: &mut Option<LinkFrame>,
    ) {
        let brk = if let Some(block) = block_stack.last() {
            if matches!(block.kind, BlockKind::CodeBlock) {
                "\n"
            } else {
                " "
            }
        } else {
            " "
        };

        if let Some(block) = block_stack.last_mut()
            && !block.full_text.ends_with(' ')
            && !block.full_text.ends_with('\n')
        {
            block.full_text.push_str(brk);
        }
        if let Some(link) = current_link.as_mut()
            && !link.alias.ends_with(' ')
            && !link.alias.ends_with('\n')
        {
            link.alias.push_str(brk);
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Metadata blocks require full state context"
    )]
    fn handle_metadata(
        event: Event<'_>,
        range: std::ops::Range<usize>,
        in_metadata: &mut Option<(
            pulldown_cmark::MetadataBlockKind,
            SourceByteOffset,
        )>,
        metadata_text: &mut String,
        sections: &mut Vec<RawSection>,
        frontmatter_spec: &Arc<FrontmatterConfigSpec>,
        frontmatter: &mut Option<RawFrontmatter<'_>>,
    ) -> Result<bool, NoteIngestError> {
        let Some((kind, start_offset)) = *in_metadata else {
            return Ok(false);
        };
        match event {
            Event::Text(t) | Event::Code(t) => metadata_text.push_str(&t),
            Event::SoftBreak | Event::HardBreak => metadata_text.push('\n'),
            Event::End(TagEnd::MetadataBlock(_)) => {
                in_metadata.take();
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
                sections.push(RawSection::new(
                    RawSectionKind::Frontmatter,
                    block_range,
                    0,
                ));
                *frontmatter = Some(RawFrontmatter::new(
                    Arc::clone(frontmatter_spec),
                    kind.into(),
                    metadata_text.clone().into(),
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
            | Event::TaskListMarker(_) => {}
        }
        Ok(true)
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &ScannedArtifact"
    )]
    fn filter_artifacts_by_range<'source>(
        block_range: SourceByteRange,
        scannable_ranges: &[std::ops::Range<usize>],
        artifacts: &[ScannedArtifact<'source>],
    ) -> ScannedBlock<'source> {
        let mut scan_result = ScannedBlock::default();
        if scannable_ranges.is_empty() {
            return scan_result;
        }

        let start = block_range.start();
        let end = block_range.end();

        // Find the first artifact that could be in range using binary search.
        let first_idx = artifacts.partition_point(|a| a.position() < start);

        let Some(slice) = artifacts.get(first_idx..) else {
            return scan_result;
        };

        for artifact in slice {
            let pos = artifact.position();
            if pos >= end {
                break;
            }

            if !Self::is_scannable_position(pos, scannable_ranges)
                && !artifact.is_marker()
            {
                continue;
            }

            match artifact {
                ScannedArtifact::Tag {
                    text,
                    range,
                } => {
                    scan_result.tags.push(RawTag::new(text.clone(), *range));
                }
                ScannedArtifact::InlineField {
                    key,
                    value,
                    range,
                } => {
                    // Use heuristic parsing to detect types (no spec available
                    // here)
                    let typed_value = RawFieldValue::from_str_with_spec(
                        value.as_ref(),
                        key.as_ref(),
                        None, // No spec available at parse time
                    )
                    .into_owned();
                    scan_result.inline_fields.push(RawInlineField::new(
                        key.clone(),
                        typed_value,
                        *range,
                    ));
                }
                ScannedArtifact::BlockRef {
                    id,
                    ..
                } => {
                    scan_result
                        .block_refs
                        .push(RawBlockRef::new(id.clone(), pos));
                }
                ScannedArtifact::TaskMarker {
                    marker,
                    ..
                } => {
                    scan_result.task_marker =
                        Some(RawTaskMarker::from_char(*marker));
                }
            }
        }

        scan_result
    }

    fn is_scannable_position(
        position: SourceByteOffset,
        scannable_ranges: &[std::ops::Range<usize>],
    ) -> bool {
        let pos = position.as_usize();
        scannable_ranges
            .iter()
            .any(|range| pos >= range.start && pos < range.end)
    }

    #[inline]
    #[expect(
        clippy::too_many_arguments,
        reason = "Finalizing a paragraph requires full state context"
    )]
    fn finalize_paragraph<'source>(
        block: &ActiveBlock,
        block_range: SourceByteRange,
        sections: &mut Vec<RawSection>,
        tags: &mut Vec<RawTag<'source>>,
        inline_fields: &mut Vec<RawInlineField<'source>>,
        block_stack: &mut [ActiveBlock],
        block_refs: &mut Vec<RawBlockRef<'source>>,
        master_artifacts: &[ScannedArtifact<'source>],
    ) {
        sections.push(RawSection::new(
            RawSectionKind::Paragraph,
            block_range,
            block.depth,
        ));
        let scan_result = Self::filter_artifacts_by_range(
            block_range,
            &block.scannable_ranges,
            master_artifacts,
        );
        tags.extend(scan_result.tags);
        inline_fields.extend(scan_result.inline_fields);
        block_refs.extend(scan_result.block_refs);

        if let Some(parent) = block_stack.last_mut()
            && matches!(parent.kind, BlockKind::ListItem)
            && parent.full_text.is_empty()
        {
            parent.full_text.push_str(&block.full_text);
        }
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
            context.task_spec,
            context.item_positions,
        ));
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Finalizing a list item requires full state context"
    )]
    #[expect(
        clippy::ptr_arg,
        reason = "List contexts are a mutable stack in this parser"
    )]
    fn finalize_list_item<'source>(
        block: &ActiveBlock,
        block_range: SourceByteRange,
        sections: &mut Vec<RawSection>,
        list_stack: &[RawListKind],
        open_item_by_depth: &mut [SourceByteOffset],
        list_items: &mut Vec<RawListItem<'source>>,
        list_contexts: &mut Vec<ListContext>,
        block_refs: &mut Vec<RawBlockRef<'source>>,
        master_artifacts: &[ScannedArtifact<'source>],
    ) -> Result<(), NoteIngestError> {
        sections.push(RawSection::new(
            RawSectionKind::List,
            block_range,
            block.depth,
        ));

        let scan_result = Self::filter_artifacts_by_range(
            block_range,
            &block.scannable_ranges,
            master_artifacts,
        );

        let list_kind =
            list_stack.last().copied().unwrap_or(RawListKind::Unordered);
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
            open_item_by_depth.get(depth_index.saturating_sub(1)).copied()
        };

        let task_marker = if block.task_marker.is_some() {
            scan_result.task_marker
        } else {
            None
        };
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

        if let Some(context) = list_contexts.last_mut() {
            context.item_positions.push(block.start_offset);
        }

        list_items.push(RawListItem::new(
            list_kind,
            list_depth,
            raw_text.into(),
            task_marker,
            block_range,
            text_range,
            parent_pos,
            scan_result.tags,
            scan_result.inline_fields,
        ));

        block_refs.extend(scan_result.block_refs);
        Ok(())
    }
}

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
    task_spec: Arc<crate::config::task::TaskConfigSpec>,
    item_positions: Vec<SourceByteOffset>,
}

impl ListContext {
    fn new(
        kind: RawListKind,
        task_spec: Arc<crate::config::task::TaskConfigSpec>,
    ) -> Self {
        Self {
            kind,
            task_spec,
            item_positions: Vec::new(),
        }
    }
}

#[derive(Debug, Default)]
struct ScannedBlock<'source> {
    tags: Vec<RawTag<'source>>,
    inline_fields: Vec<RawInlineField<'source>>,
    block_refs: Vec<RawBlockRef<'source>>,
    task_marker: Option<RawTaskMarker>,
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

struct LinkFrame {
    style: RawLinkStyle,
    is_embed: bool,
    target: String,
    start: SourceByteOffset,
    alias: String,
}

struct LinkTarget<'source>(&'source str);
impl<'source> LinkTarget<'source> {
    fn new(target: &'source str) -> Self {
        Self(target)
    }

    fn split(self) -> (&'source str, Option<&'source str>) {
        if self.is_external() {
            return (self.0, None);
        }
        self.0.split_once('#').map_or((self.0, None), |(p, a)| (p, Some(a)))
    }

    fn is_external(&self) -> bool {
        self.0.starts_with("http://")
            || self.0.starts_with("https://")
            || self.0.starts_with("ftp://")
            || self.0.starts_with("mailto:")
    }
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

    fn take(&mut self) -> String {
        self.pool.pop().unwrap_or_else(|| String::with_capacity(128))
    }

    fn put(&mut self, mut s: String) {
        s.clear();
        self.pool.push(s);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        frontmatter::FrontmatterConfigSpec, task::TaskConfigSpec,
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

    fn frontmatter_spec_fixture() -> FrontmatterConfigSpec {
        FrontmatterConfigSpec::new(
            "title".into(),
            "aliases".into(),
            "tags".into(),
            "file_class".into(),
            "date_created".into(),
            "date_modified".into(),
        )
    }

    fn parse_raw(markdown: &str) -> RawNote<'_> {
        let path = crate::note::paths::NotePath::try_new("test.md")
            .expect("valid test path");
        let frontmatter_spec = Arc::new(frontmatter_spec_fixture());
        let task_spec = Arc::new(task_spec_fixture());
        MarkdownParser::parse(
            markdown,
            path,
            None,
            None,
            &frontmatter_spec,
            &task_spec,
        )
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
}
