//! Markdown parser and extraction.
use std::time::SystemTime;

use pulldown_cmark::{
    Event, LinkType, Options, Parser, TagEnd, utils::TextMergeWithOffset,
};

use crate::note::{
    error::NoteIngestError,
    paths::NotePath,
    position::{SourceByteOffset, SourceByteRange},
    raw::{
        RawFrontmatter, RawFrontmatterFormat, RawHeading, RawInlineField,
        RawLink, RawLinkStyle, RawListDepth, RawListItem, RawListType, RawNote,
        RawReferenceLink, RawSection, RawSectionKind, RawTag, RawTask,
    },
    scanner::{NoteScanner, ScanArtifact, TaskMarkerScanner},
};

/// Markdown parser for extracting note facts.
#[non_exhaustive]
pub struct MarkdownParser;

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

impl MarkdownParser {
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
        reason = "Handler needs access to full extraction state"
    )]
    fn handle_start_tag(
        tag: pulldown_cmark::Tag<'_>,
        start_pos: SourceByteOffset,
        depth: &mut u32,
        block_stack: &mut Vec<ActiveBlock>,
        list_stack: &mut Vec<RawListType>,
        current_link: &mut Option<LinkFrame>,
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
                let list_type = match list_start {
                    Some(start) => RawListType::Ordered {
                        start,
                    },
                    None => RawListType::Unordered,
                };
                list_stack.push(list_type);
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
                let is_embed = matches!(link_type, LinkType::WikiLink { .. });
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
            if matches!(
                bkind,
                BlockKind::List | BlockKind::ListItem | BlockKind::BlockQuote
            ) {
                *depth = depth.saturating_add(1);
            }
            block_stack.push(ActiveBlock {
                kind: bkind,
                depth: current_depth,
                start_offset: start_pos,
                full_text: pool.take(),
                scannable_text: pool.take(),
                task_marker: None,
            });
        }
    }

    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "Handler needs access to full extraction state"
    )]
    fn handle_end_tag(
        end_tag: pulldown_cmark::TagEnd,
        range: std::ops::Range<usize>,
        depth: &mut u32,
        block_stack: &mut Vec<ActiveBlock>,
        list_stack: &mut Vec<RawListType>,
        current_link: &mut Option<LinkFrame>,
        links: &mut Vec<RawLink>,
        sections: &mut Vec<RawSection>,
        headings: &mut Vec<RawHeading>,
        tags: &mut Vec<RawTag>,
        inline_fields: &mut Vec<RawInlineField>,
        markdown: &str,
        open_item_by_depth: &mut Vec<SourceByteOffset>,
        list_items: &mut Vec<RawListItem>,
        tasks: &mut Vec<RawTask>,
        block_refs: &mut Vec<crate::note::raw::RawBlockRef>,
        scanner: &NoteScanner,
        pool: &mut StringPool,
    ) -> Result<(), NoteIngestError> {
        match end_tag {
            pulldown_cmark::TagEnd::Link | pulldown_cmark::TagEnd::Image => {
                if let Some(mut link) = current_link.take() {
                    let alias = if link.alias.trim().is_empty() {
                        None
                    } else {
                        Some(link.alias.trim().into())
                    };
                    let (target_raw, anchor) =
                        LinkTarget::new(&link.target).split();
                    links.push(RawLink::new(
                        link.style,
                        link.is_embed,
                        target_raw.into(),
                        alias,
                        anchor.map(Into::into),
                        link.start,
                    ));
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
                        BlockKind::List
                            | BlockKind::ListItem
                            | BlockKind::BlockQuote
                    ) {
                        *depth = depth.saturating_sub(1);
                    }

                    let end_pos = SourceByteOffset::try_from_usize(range.end)
                        .map_err(NoteIngestError::Domain)?;
                    let block_range =
                        SourceByteRange::new(block.start_offset, end_pos)
                            .map_err(NoteIngestError::Domain)?;

                    match block.kind {
                        BlockKind::Heading(level) => {
                            sections.push(RawSection::new(
                                RawSectionKind::Heading,
                                block_range,
                                block.depth,
                            ));
                            headings.push(RawHeading::new(
                                level,
                                block.full_text.trim().into(),
                                block_range,
                                block.start_offset,
                            ));

                            Self::collect_block_artifacts(
                                &block,
                                scanner,
                                tags,
                                inline_fields,
                            )?;
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
                                scanner,
                            )?;
                        }
                        BlockKind::ListItem => {
                            Self::finalize_list_item(
                                &block,
                                markdown,
                                block_range,
                                sections,
                                tags,
                                inline_fields,
                                list_stack,
                                open_item_by_depth,
                                list_items,
                                tasks,
                                block_refs,
                                scanner,
                            )?;
                        }
                        BlockKind::List => {
                            list_stack.pop();
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
                    pool.put(std::mem::take(&mut block.scannable_text));
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
            | pulldown_cmark::TagEnd::MetadataBlock(_) => {}
        }
        Ok(())
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "CowStr is passed by value for consistency with cmark events"
    )]
    fn handle_text(
        text: pulldown_cmark::CowStr<'_>,
        block_stack: &mut [ActiveBlock],
        current_link: &mut Option<LinkFrame>,
    ) {
        let text_str = text.as_ref();
        if let Some(block) = block_stack.last_mut() {
            block.full_text.push_str(text_str);
            if current_link.is_none() {
                block.scannable_text.push_str(text_str);
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

        if let Some(block) = block_stack.last_mut() {
            if !block.full_text.ends_with(' ')
                && !block.full_text.ends_with('\n')
            {
                block.full_text.push_str(brk);
            }
            if current_link.is_none()
                && !block.scannable_text.ends_with(' ')
                && !block.scannable_text.ends_with('\n')
            {
                block.scannable_text.push_str(brk);
            }
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
        clippy::wildcard_enum_match_arm,
        reason = "Handler needs access to full extraction state"
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
        frontmatter: &mut Option<RawFrontmatter>,
    ) -> Result<bool, NoteIngestError> {
        let Some((kind, start)) = *in_metadata else {
            return Ok(false);
        };

        match event {
            Event::Text(t) | Event::Code(t) => {
                metadata_text.push_str(&t);
            }
            Event::SoftBreak | Event::HardBreak => {
                metadata_text.push('\n');
            }
            Event::End(TagEnd::MetadataBlock(_)) => {
                in_metadata.take();

                let end_pos = SourceByteOffset::try_from_usize(range.end)
                    .map_err(NoteIngestError::Domain)?;
                let block_range = SourceByteRange::new(start, end_pos)
                    .map_err(NoteIngestError::Domain)?;
                sections.push(RawSection::new(
                    RawSectionKind::Frontmatter,
                    block_range,
                    0,
                ));
                *frontmatter = Some(RawFrontmatter::new(
                    kind.into(),
                    metadata_text.clone().into_boxed_str(),
                    block_range,
                ));
            }
            _ => {}
        }
        Ok(true)
    }

    fn collect_block_artifacts(
        block: &ActiveBlock,
        scanner: &NoteScanner,
        tags: &mut Vec<RawTag>,
        inline_fields: &mut Vec<RawInlineField>,
    ) -> Result<(), NoteIngestError> {
        let artifacts = scanner
            .scan_block(&block.scannable_text, block.start_offset)
            .map_err(NoteIngestError::Domain)?;

        for artifact in artifacts {
            match artifact {
                ScanArtifact::Tag(tag) => tags.push(tag),
                ScanArtifact::InlineField(field) => inline_fields.push(field),
                ScanArtifact::BlockRef(_) => {}
            }
        }
        Ok(())
    }

    #[inline]
    #[expect(
        clippy::too_many_arguments,
        reason = "Helper function needs access to extraction state"
    )]
    fn finalize_paragraph(
        block: &ActiveBlock,
        block_range: SourceByteRange,
        sections: &mut Vec<RawSection>,
        tags: &mut Vec<RawTag>,
        inline_fields: &mut Vec<RawInlineField>,
        block_stack: &mut [ActiveBlock],
        block_refs: &mut Vec<crate::note::raw::RawBlockRef>,
        scanner: &NoteScanner,
    ) -> Result<(), NoteIngestError> {
        sections.push(RawSection::new(
            RawSectionKind::Paragraph,
            block_range,
            block.depth,
        ));

        Self::collect_block_artifacts(block, scanner, tags, inline_fields)?;

        if let Some(block_ref) = scanner
            .scan_tail_for_block_ref(&block.full_text, block.start_offset)
            .map_err(NoteIngestError::Domain)?
        {
            block_refs.push(block_ref);
        }

        if let Some(parent) = block_stack.last_mut()
            && matches!(parent.kind, BlockKind::ListItem)
            && parent.full_text.is_empty()
        {
            parent.full_text.push_str(&block.full_text);
        }

        Ok(())
    }

    #[inline]
    #[expect(
        clippy::too_many_arguments,
        reason = "Helper function needs access to extraction state"
    )]
    fn finalize_list_item(
        block: &ActiveBlock,
        markdown: &str,
        block_range: SourceByteRange,
        sections: &mut Vec<RawSection>,
        tags: &mut Vec<RawTag>,
        inline_fields: &mut Vec<RawInlineField>,
        list_stack: &[RawListType],
        open_item_by_depth: &mut Vec<SourceByteOffset>,
        list_items: &mut Vec<RawListItem>,
        tasks: &mut Vec<RawTask>,
        block_refs: &mut Vec<crate::note::raw::RawBlockRef>,
        scanner: &NoteScanner,
    ) -> Result<(), NoteIngestError> {
        sections.push(RawSection::new(
            RawSectionKind::List,
            block_range,
            block.depth,
        ));

        Self::collect_block_artifacts(block, scanner, tags, inline_fields)?;

        if let Some(block_ref) = scanner
            .scan_tail_for_block_ref(&block.full_text, block.start_offset)
            .map_err(NoteIngestError::Domain)?
        {
            block_refs.push(block_ref);
        }

        let list_type =
            list_stack.last().copied().unwrap_or(RawListType::Unordered);
        let list_depth = if block.depth == 0 {
            RawListDepth::Root
        } else {
            RawListDepth::Nested(u8::try_from(block.depth).unwrap_or(u8::MAX))
        };

        let depth_index = usize::try_from(block.depth).unwrap_or(0);
        if open_item_by_depth.len() <= depth_index {
            open_item_by_depth
                .resize(depth_index.saturating_add(1), block.start_offset);
        }
        if let Some(slot) = open_item_by_depth.get_mut(depth_index) {
            *slot = block.start_offset;
        }
        open_item_by_depth.truncate(depth_index.saturating_add(1));

        let parent_pos = if block.depth == 0 {
            None
        } else {
            open_item_by_depth.get(depth_index.saturating_sub(1)).copied()
        };

        let task_kind = match block.task_marker {
            Some(checked) => {
                let fallback = if checked {
                    'x'
                } else {
                    ' '
                };
                let marker = TaskMarkerScanner::find_in_source(
                    markdown,
                    block.start_offset,
                )
                .unwrap_or(fallback);
                Some(TaskMarkerScanner::raw_task_kind_from_marker(marker))
            }
            None => None,
        };

        let raw_text: Box<str> = block.full_text.trim().into();

        if let Some(tk) = task_kind {
            list_items.push(RawListItem::new(
                list_type,
                list_depth,
                raw_text.clone(),
                Some(tk),
                block_range,
                parent_pos,
            ));

            let mut task_tags = Vec::new();
            let mut task_fields = Vec::new();

            let task_artifacts = scanner
                .scan_block(&block.full_text, block.start_offset)
                .map_err(NoteIngestError::Domain)?;

            for artifact in task_artifacts {
                match artifact {
                    ScanArtifact::Tag(tag) => {
                        task_tags.push(tag.value().into());
                    }
                    ScanArtifact::InlineField(field) => task_fields.push(field),
                    ScanArtifact::BlockRef(_) => {}
                }
            }

            tasks.push(RawTask::new(
                tk,
                raw_text,
                task_tags,
                task_fields,
                block_range,
            ));
        } else {
            list_items.push(RawListItem::new(
                list_type,
                list_depth,
                raw_text,
                None,
                block_range,
                parent_pos,
            ));
        }

        Ok(())
    }

    /// Parses markdown into a minimal AST and extracts raw note artifacts.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if byte ranges cannot be represented or
    /// extraction fails.
    #[inline]
    #[expect(
        clippy::too_many_lines,
        reason = "Event sink matches comprehensive logic"
    )]
    pub fn parse(
        markdown: &str,
        path: NotePath,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<RawNote, NoteIngestError> {
        let scanner = NoteScanner::default();
        let mut pool = StringPool::new();

        let source_bytes = u64::try_from(markdown.len()).map_err(|_error| {
            NoteIngestError::Source("source length out of range".into())
        })?;
        let source_hash = blake3::hash(markdown.as_bytes())
            .to_hex()
            .to_string()
            .into_boxed_str();

        let mut reference_links = Vec::new();
        let mut block_refs = Vec::new();

        let mut headings = Vec::new();
        let mut sections = Vec::new();
        let mut links = Vec::new();
        let mut tags = Vec::new();
        let mut list_items = Vec::new();
        let mut tasks = Vec::new();
        let mut inline_fields = Vec::new();
        let mut frontmatter = None;

        let mut block_stack: Vec<ActiveBlock> = Vec::with_capacity(8);
        let mut list_stack: Vec<RawListType> = Vec::with_capacity(8);
        let mut open_item_by_depth: Vec<SourceByteOffset> =
            Vec::with_capacity(8);

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
            let start_pos = SourceByteOffset::try_from_usize(range.start)
                .map_err(NoteIngestError::Domain)?;

            if Self::handle_metadata(
                event.clone(),
                range.clone(),
                &mut in_metadata,
                &mut metadata_text,
                &mut sections,
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
                        &mut current_link,
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
                        &mut current_link,
                        &mut links,
                        &mut sections,
                        &mut headings,
                        &mut tags,
                        &mut inline_fields,
                        markdown,
                        &mut open_item_by_depth,
                        &mut list_items,
                        &mut tasks,
                        &mut block_refs,
                        &scanner,
                        &mut pool,
                    )?;
                }
                Event::Text(text) | Event::Code(text) => {
                    Self::handle_text(
                        text,
                        &mut block_stack,
                        &mut current_link,
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

        // Re-create the parser to access reference definitions (v0.13 API)
        // Since we already consumed the first parser via into_offset_iter()
        let parser_for_refs =
            Parser::new_ext(markdown, Self::obsidian_options());
        for (label, link_def) in parser_for_refs.reference_definitions().iter()
        {
            reference_links.push(RawReferenceLink::new(
                label.to_owned().into_boxed_str(),
                link_def.dest.to_string().into_boxed_str(),
                SourceByteOffset::new(0), // RefDefs don't track original definition offset yet in cmark
            ));
        }

        sections.sort_by_key(|section| u32::from(section.range().start()));

        Ok(RawNote::new(
            path,
            source_hash,
            source_bytes,
            created_at,
            modified_at,
            frontmatter,
            headings,
            sections,
            links,
            tags,
            list_items,
            tasks,
            inline_fields,
            reference_links,
            block_refs,
        ))
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

struct ActiveBlock {
    kind: BlockKind,
    depth: u32,
    start_offset: SourceByteOffset,
    full_text: String,
    scannable_text: String,
    task_marker: Option<bool>,
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
        let Some((path, anchor_text)) = self.0.split_once('#') else {
            return (self.0, None);
        };
        (path, Some(anchor_text))
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

#[cfg(test)]
#[expect(clippy::panic_in_result_fn, reason = "Tests use assertions")]
mod tests {
    use super::*;
    use crate::note::paths::NotePath;

    #[test]
    fn extract_markdown_collects_task_tokens() -> Result<(), NoteIngestError> {
        let markdown = "- [ ] #task Review PR [priority:: 1]";
        let path = NotePath::try_new("notes/task.md")?;
        let raw = MarkdownParser::parse(markdown, path, None, None)?;

        assert_eq!(raw.tasks().len(), 1);
        let task = raw.tasks().first().expect("task should exist");
        assert_eq!(task.task_kind().marker(), ' ');
        assert!(task.tags().iter().any(|tag| tag.as_ref() == "#task"));
        assert!(
            task.inline_fields()
                .iter()
                .any(|pair| pair.key() == "priority" && pair.value() == "1")
        );
        Ok(())
    }

    #[test]
    fn extract_markdown_preserves_task_marker_case()
    -> Result<(), NoteIngestError> {
        use crate::note::raw::RawTaskKind;
        let markdown = "- [X] #task Done";
        let path = NotePath::try_new("notes/task.md")?;
        let raw = MarkdownParser::parse(markdown, path, None, None)?;

        let task = raw.tasks().first().expect("task should exist");
        assert!(matches!(task.task_kind(), RawTaskKind::Checked('X')));
        Ok(())
    }
}
