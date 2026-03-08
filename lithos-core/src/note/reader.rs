//! Markdown ingestion adapter for note parsing.
//!
//! This adapter keeps file I/O and `pulldown-cmark` details out of the note
//! domain. It reads markdown content with [`crate::fs::FsReader`] and produces
//! domain entities by walking the pulldown-cmark event stream once. Parsing is
//! deterministic, test-friendly, and centralized in the adapter layer. The
//! `parse_str` entry point is public only for benchmarks; production code
//! should use `parse` to keep file ingestion in one place.

use std::{cell::OnceCell, path::Path, time::SystemTime};

use pulldown_cmark::{
    Event, MetadataBlockKind, Options, Parser, Tag as CmarkTag, TagEnd,
    utils::TextMergeWithOffset,
};

use crate::{
    config::{aggregate::Config, task::StatusSymbol},
    fs::FsReader,
    note::{
        error::{NoteError, NoteIngestError},
        frontmatter::{Frontmatter, FrontmatterFormat},
        heading::{Heading, HeadingBuilder, HeadingLevel},
        link::{
            AliasMode, EmbedState, FrontmatterLink, Link, LinkBuilder, Style,
        },
        list::{List, ListDepth, ListItem, ListItemEntry, ListType},
        position::{
            SourceByteOffset, SourceByteRange, SourceLineIndex, SourceLocation,
        },
        structure::{BlockRef, Section, SectionKind},
        tag::{Tag as NoteTag, scan_tags},
        task::{Task, TaskBuilder},
    },
};

// ----------------------------------------------------------- //
//                      Markdown Reader                        //
// ----------------------------------------------------------- //

#[derive(Debug, Default)]
struct InlineText {
    buffer: String,
}

impl InlineText {
    fn new() -> Self {
        Self::default()
    }

    fn push_text(&mut self, text: &str) {
        self.buffer.push_str(text);
    }

    fn push_break(&mut self) {
        if !self.buffer.ends_with(' ') {
            self.buffer.push(' ');
        }
    }

    fn finish(self) -> String {
        self.buffer
    }
}

#[derive(Debug)]
struct ListItemBuilder {
    position: SourceByteOffset,
    depth: ListDepth,
    text: InlineText,
    is_checkbox: bool,
    status_symbol: Option<char>,
}

impl ListItemBuilder {
    fn new(position: SourceByteOffset, depth: ListDepth) -> Self {
        Self {
            position,
            depth,
            text: InlineText::new(),
            is_checkbox: false,
            status_symbol: None,
        }
    }

    fn mark_as_checkbox(&mut self, checked: bool) {
        self.is_checkbox = true;
        self.status_symbol = Some(if checked {
            'x'
        } else {
            ' '
        });
    }

    fn add_text(&mut self, text: &str) {
        self.text.push_text(text);
    }

    fn add_break(&mut self) {
        self.text.push_break();
    }
}

/// Markdown reader for extracting note structural elements.
///
/// `NoteReader` uses `pulldown-cmark` to traverse a markdown document and
/// extract structural elements such as headings, lists, tasks, and links.
/// It is bound to a specific [`Config`] which defines the rules for task
/// promotion and metadata parsing.
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # use lithos_core::config::{
/// #     aggregate::{Config, Version},
/// #     raw::RawConfig,
/// #     vault::{VaultId, VaultRoot},
/// # };
/// # use lithos_core::fs::FsReader;
/// # use lithos_core::note::reader::NoteReader;
/// # let unique = format!(
/// #     "lithos_note_reader_example_{}",
/// #     std::process::id()
/// # );
/// # let root = std::env::temp_dir().join(unique);
/// # std::fs::create_dir_all(&root)?;
/// # std::fs::write(
/// #     root.join("note.md"),
/// #     "# Heading\n- [ ] #task Review PR",
/// # )?;
/// # let config = Config::build(
/// #     &RawConfig::default(),
/// #     VaultId::new(),
/// #     VaultRoot::try_new(root.clone())?,
/// #     Version::initial(),
/// # )?;
/// let reader = FsReader::new(root.as_path());
/// let note_reader = NoteReader::new(&config);
///
/// let parsed = note_reader.parse(&reader, Path::new("note.md"))?;
///
/// assert_eq!(parsed.tasks().len(), 1);
/// assert_eq!(parsed.headings().len(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct NoteReader<'config> {
    config: &'config Config,
    options: Options,
}

impl<'config> NoteReader<'config> {
    /// Creates a new [`NoteReader`] bound to the provided configuration.
    ///
    /// The reader stores the markdown options once so repeated parses do not
    /// rebuild the flag set, keeping parsing overhead minimal.
    #[inline]
    #[must_use]
    pub const fn new(config: &'config Config) -> Self {
        Self {
            config,
            options: obsidian_options(),
        }
    }

    /// Parse a markdown file into lists, tasks, headings, links, and
    /// frontmatter.
    ///
    /// This is the adapter's primary parsing entry point. It keeps file I/O
    /// and parsing together while leaving the note domain independent of the
    /// filesystem and markdown crate choice.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] when file I/O or parsing fails.
    #[inline]
    pub fn parse(
        &self,
        reader: &FsReader,
        path: &Path,
    ) -> Result<ParsedNote, NoteIngestError> {
        let markdown = reader
            .read_with::<String, crate::fs::ParseError, _>(
                path,
                |_, content| Ok(content.into()),
            )
            .map_err(|error| {
                NoteIngestError::Source(format!("{error}").into())
            })?;
        let markdown = markdown.into_boxed_str();
        let metadata = match reader.metadata(path) {
            Ok(meta) => Some(meta),
            Err(error) => {
                tracing::debug!(
                    path = %path.display(),
                    error = %error,
                    "Failed to read note metadata"
                );
                None
            }
        };
        let modified_at = extract_timestamp(
            path,
            metadata.as_ref(),
            std::fs::Metadata::modified,
            "modified",
        );
        let created_at = extract_timestamp(
            path,
            metadata.as_ref(),
            std::fs::Metadata::created,
            "created",
        );

        self.parse_with_timestamps(markdown, created_at, modified_at)
    }

    /// Parses markdown into lists, tasks, headings, and links.
    ///
    /// **Internal API**: This is public solely for benchmarking.
    /// Do not depend on it in production code - use `parse` instead.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] when parsing fails.
    #[inline]
    #[doc(hidden)]
    pub fn parse_str(
        &self,
        markdown: &str,
    ) -> Result<ParsedNote, NoteIngestError> {
        self.parse_with_timestamps(markdown.into(), None, None)
    }

    /// Parses owned markdown content with explicit timestamps.
    ///
    /// Intended for application services that already loaded the file content
    /// and want to avoid re-reading the filesystem.
    ///
    /// # Errors
    /// Returns [`NoteIngestError`] when parsing fails.
    #[inline]
    pub fn parse_content(
        &self,
        markdown: Box<str>,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<ParsedNote, NoteIngestError> {
        self.parse_with_timestamps(markdown, created_at, modified_at)
    }

    #[inline]
    #[expect(clippy::too_many_lines, reason = "Main parsing loop is long")]
    #[expect(
        clippy::cognitive_complexity,
        reason = "Parsing loop handles many markdown event variants"
    )]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Matches borrow pulldown-cmark events for efficiency"
    )]
    fn parse_with_timestamps(
        &self,
        markdown: Box<str>,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<ParsedNote, NoteIngestError> {
        let markdown_ref = markdown.as_ref();
        let line_index = OnceCell::new();

        let mut links = Vec::new();
        let mut frontmatter_links = Vec::new();
        let mut lists = Vec::new();
        let mut list_items = Vec::new();
        let mut tasks = Vec::new();
        let mut headings = Vec::new();
        let mut sections = Vec::new();
        let mut tags = Vec::new();
        let mut frontmatter = None;

        let mut list_stack: Vec<List> = Vec::new();
        let mut current_item: Option<ListItemBuilder> = None;
        let mut item_stack: Vec<ListItemBuilder> = Vec::new();
        let mut current_link: Option<LinkBuilder> = None;
        let mut current_heading: Option<HeadingBuilder> = None;
        let mut frontmatter_kind: Option<MetadataBlockKind> = None;
        let mut frontmatter_text = String::new();
        let mut inside_link = false;
        let mut code_block_depth = 0u32;
        let mut section_depth = 0u32;
        let mut current_section: Option<(SectionKind, SourceByteOffset)> = None;
        let mut open_item_by_depth: Vec<SourceByteOffset> = Vec::new();

        let events =
            Parser::new_ext(markdown_ref, self.options).into_offset_iter();
        let merged = TextMergeWithOffset::new(events);
        for (event, range) in merged {
            if let Event::Start(tag) = &event {
                handle_section_start(
                    tag,
                    range.start,
                    &mut section_depth,
                    &mut current_section,
                )?;
            }
            match &event {
                Event::Start(
                    CmarkTag::Link {
                        ..
                    }
                    | CmarkTag::Image {
                        ..
                    },
                ) => {
                    inside_link = true;
                }
                Event::End(TagEnd::Link | TagEnd::Image) => {
                    inside_link = false;
                }
                Event::Start(CmarkTag::CodeBlock(_)) => {
                    code_block_depth = code_block_depth.saturating_add(1);
                }
                Event::End(TagEnd::CodeBlock) => {
                    code_block_depth = code_block_depth.saturating_sub(1);
                }
                Event::Start(_)
                | Event::End(_)
                | Event::Text(_)
                | Event::Code(_)
                | Event::InlineMath(_)
                | Event::DisplayMath(_)
                | Event::Html(_)
                | Event::InlineHtml(_)
                | Event::FootnoteReference(_)
                | Event::SoftBreak
                | Event::HardBreak
                | Event::Rule
                | Event::TaskListMarker(_) => {}
            }

            let inside_code_block = code_block_depth > 0;

            match &event {
                Event::Start(CmarkTag::MetadataBlock(kind)) => {
                    frontmatter_kind = Some(*kind);
                    frontmatter_text.clear();
                }
                Event::End(TagEnd::MetadataBlock(kind)) => {
                    if frontmatter_kind == Some(*kind) && frontmatter.is_none()
                    {
                        frontmatter =
                            parse_frontmatter_block(*kind, &frontmatter_text)?;
                    }
                    frontmatter_kind = None;
                    frontmatter_text.clear();
                    close_section(
                        &mut sections,
                        &mut current_section,
                        &mut section_depth,
                        range,
                        None,
                    )?;
                }
                Event::Start(CmarkTag::Heading {
                    level,
                    ..
                }) => {
                    let position =
                        SourceByteOffset::try_from_usize(range.start)?;
                    let level = match level {
                        pulldown_cmark::HeadingLevel::H1 => {
                            HeadingLevel::try_new(1)?
                        }
                        pulldown_cmark::HeadingLevel::H2 => {
                            HeadingLevel::try_new(2)?
                        }
                        pulldown_cmark::HeadingLevel::H3 => {
                            HeadingLevel::try_new(3)?
                        }
                        pulldown_cmark::HeadingLevel::H4 => {
                            HeadingLevel::try_new(4)?
                        }
                        pulldown_cmark::HeadingLevel::H5 => {
                            HeadingLevel::try_new(5)?
                        }
                        pulldown_cmark::HeadingLevel::H6 => {
                            HeadingLevel::try_new(6)?
                        }
                    };
                    current_heading =
                        Some(HeadingBuilder::new(level, position));
                }
                Event::End(TagEnd::Heading(_)) => {
                    if let Some(builder) = current_heading.take() {
                        let heading = builder.build()?;
                        headings.push(heading.clone());
                        close_section(
                            &mut sections,
                            &mut current_section,
                            &mut section_depth,
                            range,
                            Some(heading),
                        )?;
                    } else {
                        close_section(
                            &mut sections,
                            &mut current_section,
                            &mut section_depth,
                            range,
                            None,
                        )?;
                    }
                }
                Event::Start(CmarkTag::Link {
                    link_type,
                    dest_url,
                    ..
                }) => {
                    let position =
                        SourceByteOffset::try_from_usize(range.start)?;
                    current_link = Some(start_link_builder(
                        *link_type, dest_url, position, false,
                    ));
                }
                Event::Start(CmarkTag::Image {
                    link_type,
                    dest_url,
                    ..
                }) => {
                    let position =
                        SourceByteOffset::try_from_usize(range.start)?;
                    current_link = Some(start_link_builder(
                        *link_type, dest_url, position, true,
                    ));
                }
                Event::End(TagEnd::Link | TagEnd::Image) => {
                    if let Some(builder) = current_link.take() {
                        links.push(builder.build()?);
                    }
                }
                Event::End(
                    TagEnd::Paragraph
                    | TagEnd::CodeBlock
                    | TagEnd::BlockQuote(_)
                    | TagEnd::Table,
                ) => {
                    close_section(
                        &mut sections,
                        &mut current_section,
                        &mut section_depth,
                        range,
                        None,
                    )?;
                }
                Event::Start(CmarkTag::List(start)) => {
                    let depth = ListDepth::try_new(list_stack.len())?;
                    let list_type = match *start {
                        Some(start_num) => ListType::Ordered {
                            start: start_num,
                        },
                        None => ListType::Unordered,
                    };
                    list_stack.push(List::with_depth(list_type, depth));
                }
                Event::End(TagEnd::List(_)) => {
                    if let Some(list) = list_stack.pop() {
                        lists.push(list);
                    }
                    close_section(
                        &mut sections,
                        &mut current_section,
                        &mut section_depth,
                        range,
                        None,
                    )?;
                }
                Event::Start(CmarkTag::Item) => {
                    let position =
                        SourceByteOffset::try_from_usize(range.start)?;
                    let depth = list_stack
                        .last()
                        .map_or_else(ListDepth::root, List::depth);
                    if let Some(active_item) = current_item.take() {
                        item_stack.push(active_item);
                    }
                    let depth_index = usize::from(depth.as_u8());
                    if open_item_by_depth.len() <= depth_index {
                        open_item_by_depth
                            .resize(depth_index.saturating_add(1), position);
                    }
                    if let Some(slot) = open_item_by_depth.get_mut(depth_index)
                    {
                        *slot = position;
                    }
                    open_item_by_depth.truncate(depth_index.saturating_add(1));
                    current_item = Some(ListItemBuilder::new(position, depth));
                }
                Event::End(TagEnd::Item) => {
                    if let Some(item) = current_item.take() {
                        let position = item.position;
                        let depth = item.depth;
                        let is_checkbox = item.is_checkbox;
                        let status_symbol = item.status_symbol;
                        let raw_text = item.text.finish();
                        let (status, promoted_task) = promote_task_from_item(
                            is_checkbox,
                            status_symbol,
                            position,
                            &raw_text,
                            self.config,
                        )?;
                        let promoted_task_id =
                            promoted_task.as_ref().map(Task::id);

                        add_list_item(
                            &mut list_stack,
                            &raw_text,
                            position,
                            status,
                            promoted_task_id,
                        );

                        tasks.extend(promoted_task.into_iter());

                        let parent =
                            parent_for_depth(depth, &open_item_by_depth);
                        let record = ListItemRecord {
                            position,
                            depth,
                            parent,
                            status,
                            task_id: promoted_task_id,
                        };
                        record_list_item(&mut list_items, &record);
                    }
                    current_item = item_stack.pop();
                }
                Event::TaskListMarker(checked) => {
                    if let Some(item) = current_item.as_mut() {
                        item.mark_as_checkbox(*checked);
                    }
                }
                Event::Text(text) => {
                    if frontmatter_kind.is_some() {
                        frontmatter_text.push_str(text);
                    }
                    if let Some(builder) = current_heading.as_mut() {
                        builder.push_text(text);
                    }
                    if let Some(builder) = current_link.as_mut() {
                        builder.add_alias_text(text);
                    }
                    if let Some(item) = current_item.as_mut() {
                        item.add_text(text);
                    }
                    collect_tags(
                        text,
                        inside_code_block,
                        inside_link,
                        &mut tags,
                    );
                }
                Event::Code(text) => {
                    if let Some(builder) = current_heading.as_mut() {
                        builder.push_text(text);
                    }
                    if let Some(builder) = current_link.as_mut() {
                        builder.add_alias_text(text);
                    }
                    if let Some(item) = current_item.as_mut() {
                        item.add_text(text);
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    if frontmatter_kind.is_some() {
                        frontmatter_text.push('\n');
                    }
                    if let Some(builder) = current_heading.as_mut() {
                        builder.push_break();
                    }
                    if let Some(builder) = current_link.as_mut() {
                        builder.add_alias_text(" ");
                    }
                    if let Some(item) = current_item.as_mut() {
                        item.add_break();
                    }
                }
                Event::Start(_)
                | Event::End(_)
                | Event::InlineMath(_)
                | Event::DisplayMath(_)
                | Event::Html(_)
                | Event::InlineHtml(_)
                | Event::FootnoteReference(_)
                | Event::Rule => {}
            }
        }

        lists.extend(list_stack);
        if let Some(frontmatter) = frontmatter.as_ref() {
            collect_frontmatter_tags(frontmatter, self.config, &mut tags);
            collect_frontmatter_links(frontmatter, &mut frontmatter_links);
        }

        let block_refs = collect_block_refs(markdown_ref)?;
        list_items.sort_by_key(ListItemEntry::position);

        Ok(ParsedNote {
            source: markdown,
            lists,
            list_items,
            tasks,
            headings,
            sections,
            links,
            frontmatter_links,
            block_refs,
            tags,
            frontmatter,
            line_index,
            created_at,
            modified_at,
        })
    }
}

fn start_link_builder(
    link_type: pulldown_cmark::LinkType,
    dest_url: &pulldown_cmark::CowStr<'_>,
    position: SourceByteOffset,
    is_embed: bool,
) -> LinkBuilder {
    let embed = if is_embed {
        EmbedState::Embed
    } else {
        EmbedState::Link
    };
    match link_type {
        pulldown_cmark::LinkType::WikiLink {
            has_pothole,
        } => {
            let alias_mode = if has_pothole {
                AliasMode::Collect
            } else {
                AliasMode::Ignore
            };
            LinkBuilder::new(
                dest_url.as_ref(),
                position,
                Style::WikiLink,
                embed,
                alias_mode,
            )
        }
        pulldown_cmark::LinkType::Autolink
        | pulldown_cmark::LinkType::Email => LinkBuilder::new(
            dest_url.as_ref(),
            position,
            Style::MdLink,
            embed,
            AliasMode::Ignore,
        ),
        pulldown_cmark::LinkType::Inline
        | pulldown_cmark::LinkType::Reference
        | pulldown_cmark::LinkType::ReferenceUnknown
        | pulldown_cmark::LinkType::Collapsed
        | pulldown_cmark::LinkType::CollapsedUnknown
        | pulldown_cmark::LinkType::Shortcut
        | pulldown_cmark::LinkType::ShortcutUnknown => LinkBuilder::new(
            dest_url.as_ref(),
            position,
            Style::MdLink,
            embed,
            AliasMode::Collect,
        ),
    }
}

fn add_tag(tags: &mut Vec<NoteTag>, tag: NoteTag) {
    if !tags.iter().any(|existing| existing.full_path() == tag.full_path()) {
        tags.push(tag);
    }
}

fn collect_frontmatter_tags(
    frontmatter: &Frontmatter,
    config: &Config,
    tags: &mut Vec<NoteTag>,
) {
    let key = config.frontmatter().tags();
    let Some(value) = frontmatter.get(key) else {
        return;
    };

    let mut collect_tokens = |text: &str| {
        for token in text.split(|ch: char| ch.is_whitespace() || ch == ',') {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }
            if let Ok(tag) = NoteTag::try_from_token(token) {
                add_tag(tags, tag);
            }
        }
    };

    if let Some(text) = value.as_str() {
        collect_tokens(text);
        return;
    }

    if let Some(values) = value.as_array() {
        for item in values {
            if let Some(text) = item.as_str() {
                collect_tokens(text);
            }
        }
    }
}

fn collect_frontmatter_links(
    frontmatter: &Frontmatter,
    links: &mut Vec<FrontmatterLink>,
) {
    for (key, value) in frontmatter.fields() {
        collect_frontmatter_links_for_value(key, value, links);
    }
}

fn collect_frontmatter_links_for_value(
    key: &str,
    value: &crate::note::value::FieldValue,
    links: &mut Vec<FrontmatterLink>,
) {
    if let Some(text) = value.as_str() {
        if let Ok(Some(link)) =
            crate::note::link::parse_frontmatter_link(key, text)
        {
            links.push(link);
        }
        return;
    }

    if let Some(values) = value.as_array() {
        for item in values {
            if let Some(text) = array_as_wikilink(item)
                && let Ok(Some(link)) =
                    crate::note::link::parse_frontmatter_link(key, &text)
            {
                links.push(link);
                continue;
            }
            collect_frontmatter_links_for_value(key, item, links);
        }
        return;
    }

    if let Some(values) = value.object_fields() {
        for (child_key, child_value) in values {
            let child_key_str: &str = child_key;
            let mut combined = String::with_capacity(
                key.len().saturating_add(child_key_str.len()).saturating_add(1),
            );
            combined.push_str(key);
            combined.push('.');
            combined.push_str(child_key_str);
            collect_frontmatter_links_for_value(&combined, child_value, links);
        }
    }
}

fn array_as_wikilink(value: &crate::note::value::FieldValue) -> Option<String> {
    let outer = value.as_array()?;
    if outer.len() != 1 {
        return None;
    }
    if let Some(text) =
        outer.first().and_then(crate::note::value::FieldValue::as_str)
    {
        return Some(wrap_wikilink_text(text));
    }
    let inner = outer.first()?.as_array()?;
    if inner.len() != 1 {
        return None;
    }
    let text =
        inner.first().and_then(crate::note::value::FieldValue::as_str)?;
    Some(wrap_wikilink_text(text))
}

fn wrap_wikilink_text(text: &str) -> String {
    let mut combined = String::with_capacity(text.len().saturating_add(4));
    combined.push_str("[[");
    combined.push_str(text);
    combined.push_str("]]");
    combined
}

fn handle_section_start(
    tag: &CmarkTag<'_>,
    start: usize,
    section_depth: &mut u32,
    current_section: &mut Option<(SectionKind, SourceByteOffset)>,
) -> Result<(), NoteError> {
    let Some(kind) = section_kind_for_tag(tag) else {
        return Ok(());
    };
    if *section_depth == 0 {
        let start = SourceByteOffset::try_from_usize(start)?;
        *current_section = Some((kind, start));
    }
    *section_depth = section_depth.saturating_add(1);
    Ok(())
}

fn parse_frontmatter_block(
    kind: MetadataBlockKind,
    text: &str,
) -> Result<Option<Frontmatter>, NoteError> {
    if text.is_empty() {
        return Ok(None);
    }
    let format = match kind {
        MetadataBlockKind::YamlStyle => FrontmatterFormat::Yaml,
        MetadataBlockKind::PlusesStyle => FrontmatterFormat::Toml,
    };
    let parsed =
        Frontmatter::parse(format, text).map_err(NoteError::Frontmatter)?;
    Ok(Some(parsed))
}

type TaskPromotion = (Option<StatusSymbol>, Option<Task>);

fn promote_task_from_item(
    is_checkbox: bool,
    status_symbol: Option<char>,
    position: SourceByteOffset,
    raw_text: &str,
    config: &Config,
) -> Result<TaskPromotion, NoteError> {
    if !is_checkbox {
        return Ok((None, None));
    }

    let symbol = status_symbol.unwrap_or(' ');
    let checkbox_status = StatusSymbol::try_new(symbol).map_err(|_error| {
        NoteError::Task(crate::note::error::TaskError::InvalidStatusSymbol {
            symbol,
            reason: "status symbol must be a single ASCII character",
        })
    })?;

    if config.task().tags().is_empty() {
        return Ok((Some(checkbox_status), None));
    }

    let tags_for_task = scan_tags(raw_text);
    let builder = TaskBuilder::new(config.task());
    let promoted = builder.promote_from_checkbox(
        raw_text,
        tags_for_task,
        checkbox_status,
        position,
    )?;
    Ok((Some(checkbox_status), promoted))
}

fn add_list_item(
    list_stack: &mut [List],
    raw_text: &str,
    position: SourceByteOffset,
    status: Option<StatusSymbol>,
    task_id: Option<crate::note::task::TaskId>,
) {
    let Some(list) = list_stack.last_mut() else {
        return;
    };
    if let Some(checkbox_status) = status {
        list.add_item(ListItem::Checkbox {
            text: raw_text.trim().into(),
            status: checkbox_status,
            position,
            task_id,
        });
    } else {
        list.add_item(ListItem::Plain {
            text: raw_text.trim().into(),
            position,
        });
    }
}

struct ListItemRecord {
    position: SourceByteOffset,
    depth: ListDepth,
    parent: Option<SourceByteOffset>,
    status: Option<StatusSymbol>,
    task_id: Option<crate::note::task::TaskId>,
}

fn record_list_item(
    list_items: &mut Vec<ListItemEntry>,
    record: &ListItemRecord,
) {
    list_items.push(ListItemEntry::new(
        record.position,
        record.depth,
        record.parent,
        record.status,
        record.task_id,
    ));
}

fn parent_for_depth(
    depth: ListDepth,
    open_item_by_depth: &[SourceByteOffset],
) -> Option<SourceByteOffset> {
    let depth_index = usize::from(depth.as_u8());
    if depth_index == 0 {
        return None;
    }
    open_item_by_depth.get(depth_index.saturating_sub(1)).copied()
}

fn collect_tags(
    text: &str,
    inside_code_block: bool,
    inside_link: bool,
    tags: &mut Vec<NoteTag>,
) {
    if inside_code_block || inside_link {
        return;
    }
    for tag in scan_tags(text) {
        add_tag(tags, tag);
    }
}

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Match ergonomics on borrowed pulldown-cmark tags"
)]
fn section_kind_for_tag(tag: &CmarkTag<'_>) -> Option<SectionKind> {
    match tag {
        CmarkTag::Paragraph => Some(SectionKind::Paragraph),
        CmarkTag::Heading {
            ..
        } => Some(SectionKind::Heading),
        CmarkTag::List(_) => Some(SectionKind::List),
        CmarkTag::CodeBlock(_) => Some(SectionKind::Code),
        CmarkTag::BlockQuote(kind) => Some(if kind.is_some() {
            SectionKind::Callout
        } else {
            SectionKind::BlockQuote
        }),
        CmarkTag::Table(_) => Some(SectionKind::Table),
        CmarkTag::MetadataBlock(_) => Some(SectionKind::Frontmatter),
        CmarkTag::HtmlBlock
        | CmarkTag::Item
        | CmarkTag::FootnoteDefinition(_)
        | CmarkTag::DefinitionList
        | CmarkTag::DefinitionListTitle
        | CmarkTag::DefinitionListDefinition
        | CmarkTag::TableHead
        | CmarkTag::TableRow
        | CmarkTag::TableCell
        | CmarkTag::Emphasis
        | CmarkTag::Strong
        | CmarkTag::Strikethrough
        | CmarkTag::Superscript
        | CmarkTag::Subscript
        | CmarkTag::Link {
            ..
        }
        | CmarkTag::Image {
            ..
        } => None,
    }
}

fn close_section(
    sections: &mut Vec<Section>,
    current_section: &mut Option<(SectionKind, SourceByteOffset)>,
    section_depth: &mut u32,
    event_range: std::ops::Range<usize>,
    heading: Option<Heading>,
) -> Result<(), NoteError> {
    if *section_depth == 0 {
        return Ok(());
    }
    *section_depth = section_depth.saturating_sub(1);
    if *section_depth > 0 {
        return Ok(());
    }
    let Some((kind, start)) = current_section.take() else {
        return Ok(());
    };
    let end = SourceByteOffset::try_from_usize(event_range.end)?;
    let source_range = SourceByteRange::new(start, end)?;
    sections.push(Section::new(kind, heading, source_range));
    Ok(())
}

fn collect_block_refs(source: &str) -> Result<Vec<BlockRef>, NoteError> {
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
                let block_id = crate::note::structure::BlockRefId::try_new(id)?;
                refs.push(BlockRef::new(block_id, position));
            }
        }

        offset = offset.saturating_add(line.len());
    }

    Ok(refs)
}

/// Build the pulldown-cmark option set used for Obsidian-compatible parsing.
///
/// This centralizes feature toggles so adapters and tests share identical
/// parsing behavior.
///
/// Enables:
/// - `WikiLinks`: `[[link]]`, `[[link|alias]]`, `![[embed]]`
/// - Frontmatter: YAML metadata blocks
/// - Tables: GFM tables
/// - Footnotes: Markdown footnotes
/// - Math: Inline `$...$` and display `$$...$$`
/// - Strikethrough: `~~text~~`
/// - Heading Attributes: `# Title {#id .class}`
/// - Task Lists: `- [ ] task`
const fn obsidian_options() -> Options {
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

/// Results of parsing a markdown note into structured elements.
///
/// # Examples
///
/// ```
/// use lithos_core::{
///     config::{
///         aggregate::Config,
///         raw::RawConfig,
///         vault::{VaultId, VaultRoot},
///     },
///     note::reader::NoteReader,
/// };
///
/// let root = std::env::temp_dir()
///     .join(format!("lithos_parse_outcome_{}", std::process::id()));
/// std::fs::create_dir_all(&root)?;
/// let config = Config::build(
///     &RawConfig::default(),
///     VaultId::new(),
///     VaultRoot::try_new(root.clone())?,
///     lithos_core::config::aggregate::Version::initial(),
/// )?;
/// let reader = NoteReader::new(&config);
/// let outcome = reader.parse_str("# Heading\n- [ ] #task Review PR")?;
/// assert_eq!(outcome.headings().len(), 1);
/// assert_eq!(outcome.tasks().len(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub struct ParsedNote {
    source: Box<str>,
    lists: Vec<List>,
    list_items: Vec<ListItemEntry>,
    tasks: Vec<Task>,
    headings: Vec<Heading>,
    sections: Vec<Section>,
    links: Vec<Link>,
    frontmatter_links: Vec<FrontmatterLink>,
    block_refs: Vec<BlockRef>,
    tags: Vec<NoteTag>,
    frontmatter: Option<Frontmatter>,
    line_index: OnceCell<SourceLineIndex>,
    created_at: Option<SystemTime>,
    modified_at: Option<SystemTime>,
}

impl ParsedNote {
    /// Lists parsed from the markdown body.
    #[inline]
    #[must_use]
    pub fn lists(&self) -> &[List] {
        &self.lists
    }

    /// List item metadata entries parsed from the markdown body.
    #[inline]
    #[must_use]
    pub fn list_items(&self) -> &[ListItemEntry] {
        &self.list_items
    }

    /// Raw markdown source used for parsing.
    #[inline]
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Tasks parsed from task list items.
    #[inline]
    #[must_use]
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Headings parsed from the markdown body.
    #[inline]
    #[must_use]
    pub fn headings(&self) -> &[Heading] {
        &self.headings
    }

    /// Sections parsed from the markdown body.
    #[inline]
    #[must_use]
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    /// Links parsed from the markdown body.
    #[inline]
    #[must_use]
    pub fn links(&self) -> &[Link] {
        &self.links
    }

    /// Frontmatter links parsed from metadata values.
    #[inline]
    #[must_use]
    pub fn frontmatter_links(&self) -> &[FrontmatterLink] {
        &self.frontmatter_links
    }

    /// Block reference identifiers parsed from the markdown body.
    #[inline]
    #[must_use]
    pub fn block_refs(&self) -> &[BlockRef] {
        &self.block_refs
    }

    /// Tags parsed from the markdown body.
    #[inline]
    #[must_use]
    pub fn tags(&self) -> &[NoteTag] {
        &self.tags
    }

    /// Frontmatter parsed from the metadata block, if present.
    #[inline]
    #[must_use]
    pub fn frontmatter(&self) -> Option<&Frontmatter> {
        self.frontmatter.as_ref()
    }

    /// Line index for converting byte offsets into line/column positions.
    #[inline]
    #[must_use]
    pub fn line_index(&self) -> &SourceLineIndex {
        self.line_index.get_or_init(|| SourceLineIndex::new(&self.source))
    }

    /// Converts a byte offset into a line/column location.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if the offset is out of bounds or invalid.
    #[inline]
    pub fn location_for_offset(
        &self,
        offset: SourceByteOffset,
    ) -> Result<SourceLocation, NoteError> {
        self.line_index().line_column(offset, &self.source)
    }

    /// Filesystem created timestamp at ingestion time, if available.
    #[inline]
    #[must_use]
    pub const fn created_at(&self) -> Option<SystemTime> {
        self.created_at
    }

    /// Filesystem modified timestamp at ingestion time, if available.
    #[inline]
    #[must_use]
    pub const fn modified_at(&self) -> Option<SystemTime> {
        self.modified_at
    }
}

fn extract_timestamp(
    path: &Path,
    metadata: Option<&std::fs::Metadata>,
    time_fn: fn(&std::fs::Metadata) -> std::io::Result<std::time::SystemTime>,
    time_type: &str,
) -> Option<SystemTime> {
    let meta = metadata?;

    match time_fn(meta) {
        Ok(time) => Some(time),
        Err(error) => {
            tracing::debug!(
                path = %path.display(),
                error = %error,
                time_type,
                "Failed to read timestamp from metadata"
            );
            None
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Tests use assertions in Result-returning functions."
)]
mod tests {
    use super::*;
    use crate::{
        config::{
            aggregate::Config,
            raw::RawConfig,
            vault::{VaultId, VaultRoot},
        },
        note::{
            link::{Anchor, EmbedType, Style, Target},
            list::{ListDepth, ListItem, ListType},
            position::SourceByteOffset,
            value::FieldValue,
        },
    };

    /// Test alias for historical parse outcome name.
    type ParseOutcome = ParsedNote;

    fn test_config() -> Config {
        let raw = RawConfig::default();

        Config::build(
            &raw,
            VaultId::new(),
            VaultRoot::try_new(std::path::PathBuf::from("/vault"))
                .expect("vault root"),
            crate::config::aggregate::Version::initial(),
        )
        .expect("failed to build test config")
    }

    #[test]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Test matches &ListItem using match ergonomics."
    )]
    fn parses_checkbox_list_and_promotes_tasks() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "- [ ] #task Review PR [priority:: 1]\n- [x] Buy milk\n";

        let ParseOutcome {
            lists,
            tasks,
            ..
        } = reader.parse_str(markdown)?;
        assert_eq!(lists.len(), 1, "expected one list");
        assert_eq!(tasks.len(), 1, "expected one promoted task");

        let list = lists.first().expect("list should exist");
        assert!(matches!(list.list_type(), ListType::Unordered));
        assert_eq!(list.items().count(), 2, "expected two list items");

        let first_item = list.items().next().expect("first item");
        let (task_id, status) = match first_item {
            ListItem::Checkbox {
                task_id,
                status,
                ..
            } => (task_id, status),
            ListItem::Plain {
                ..
            } => {
                return Err(NoteError::Structure(
                    "expected checkbox list item",
                ));
            }
        };
        assert_eq!(status.value(), ' ', "expected unchecked status");
        assert!(task_id.is_some(), "expected promoted task id");

        let task = tasks.first().expect("task should exist");
        assert_eq!(task_id.as_ref(), Some(&task.id()));
        Ok(())
    }

    #[test]
    fn captures_list_depths() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "1. First\n   - [ ] #task Nested\n";

        let ParseOutcome {
            lists,
            ..
        } = reader.parse_str(markdown)?;
        assert_eq!(lists.len(), 2, "expected two lists");

        let ordered = lists
            .iter()
            .find(|list| matches!(list.list_type(), ListType::Ordered { .. }))
            .expect("ordered list should exist");
        assert_eq!(
            ordered.depth(),
            ListDepth::root(),
            "ordered list should be top-level"
        );

        let unordered = lists
            .iter()
            .find(|list| matches!(list.list_type(), ListType::Unordered))
            .expect("unordered list should exist");
        assert_eq!(
            unordered.depth(),
            ListDepth::try_new(1)?,
            "nested list should have depth 1"
        );
        Ok(())
    }

    #[test]
    fn parses_lists_and_tasks() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "- [ ] #task Review PR\n";

        let parsed = reader.parse_str(markdown)?;

        assert_eq!(parsed.lists().len(), 1, "note should have 1 list");
        assert_eq!(parsed.tasks().len(), 1, "note should have 1 task");
        Ok(())
    }

    #[test]
    fn parsed_note_exposes_line_index() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "first\nsecond";

        let parsed = reader.parse_str(markdown)?;
        let location = parsed
            .line_index()
            .line_column(SourceByteOffset::new(0), markdown)?;

        assert_eq!(location.line().value(), 1);
        assert_eq!(location.column().value(), 1);
        Ok(())
    }

    #[test]
    #[expect(
        clippy::indexing_slicing,
        reason = "Test asserts exact count before indexing"
    )]
    fn parses_headings() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "# Section 1\n\nContent\n\n## Section 2";

        let parsed = reader.parse_str(markdown)?;

        assert_eq!(parsed.headings().len(), 2, "note should have 2 headings");
        let headings: Vec<_> = parsed.headings().iter().collect();
        assert_eq!(headings[0].text(), "Section 1");
        assert_eq!(headings[1].text(), "Section 2");
        Ok(())
    }

    #[test]
    #[expect(
        clippy::indexing_slicing,
        reason = "Test asserts exact count before indexing"
    )]
    fn parses_wikilink_simple() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "[[target note]]";

        let ParseOutcome {
            links,
            ..
        } = reader.parse_str(markdown)?;
        assert_eq!(links.len(), 1, "expected one link");
        assert!(matches!(links[0].style(), Style::WikiLink));
        assert_eq!(links[0].target().vault_path(), Some("target note"));
        assert!(!links[0].is_embed());

        Ok(())
    }

    #[test]
    #[expect(
        clippy::indexing_slicing,
        reason = "Test asserts exact count before indexing"
    )]
    fn parses_wikilink_with_alias() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "[[target|display text]]";

        let ParseOutcome {
            links,
            ..
        } = reader.parse_str(markdown)?;
        assert_eq!(links.len(), 1, "expected one link");
        assert!(matches!(links[0].style(), Style::WikiLink));
        assert_eq!(links[0].target().vault_path(), Some("target"));
        assert_eq!(links[0].alias(), Some("display text"));

        Ok(())
    }

    #[test]
    #[expect(
        clippy::indexing_slicing,
        reason = "Test asserts exact count before indexing"
    )]
    fn parses_wikilink_with_heading_anchor() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "[[note#Section Title]]";

        let ParseOutcome {
            links,
            ..
        } = reader.parse_str(markdown)?;
        assert_eq!(links.len(), 1, "expected one link");
        assert_eq!(links[0].target().vault_path(), Some("note"));
        assert!(matches!(
            links[0].anchor(),
            Some(Anchor::Heading(text)) if text.as_str() == "Section Title"
        ));

        Ok(())
    }

    #[test]
    #[expect(
        clippy::indexing_slicing,
        reason = "Test asserts exact count before indexing"
    )]
    fn parses_wikilink_with_blockref_anchor() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "[[note#^block123]]";

        let ParseOutcome {
            links,
            ..
        } = reader.parse_str(markdown)?;
        assert_eq!(links.len(), 1, "expected one link");
        assert_eq!(links[0].target().vault_path(), Some("note"));
        assert!(matches!(
            links[0].anchor(),
            Some(Anchor::BlockRef(text)) if text.as_str() == "block123"
        ));

        Ok(())
    }

    #[test]
    #[expect(
        clippy::indexing_slicing,
        reason = "Test asserts exact count before indexing"
    )]
    fn parses_embed_image() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "![[image.png]]";

        let ParseOutcome {
            links,
            ..
        } = reader.parse_str(markdown)?;
        assert_eq!(links.len(), 1, "expected one link");
        assert!(links[0].is_embed());
        assert!(matches!(links[0].embed_type(), Some(EmbedType::Image)));
        assert_eq!(links[0].target().vault_path(), Some("image.png"));

        Ok(())
    }

    #[test]
    #[expect(
        clippy::indexing_slicing,
        reason = "Test asserts exact count before indexing"
    )]
    fn parses_embed_video() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "![[video.mp4]]";

        let ParseOutcome {
            links,
            ..
        } = reader.parse_str(markdown)?;
        assert_eq!(links.len(), 1, "expected one link");
        assert!(links[0].is_embed());
        assert!(matches!(links[0].embed_type(), Some(EmbedType::Video)));

        Ok(())
    }

    #[test]
    #[expect(
        clippy::indexing_slicing,
        reason = "Test asserts exact count before indexing"
    )]
    fn parses_standard_markdown_link() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "[link text](target.md)";

        let ParseOutcome {
            links,
            ..
        } = reader.parse_str(markdown)?;
        assert_eq!(links.len(), 1, "expected one link");
        assert!(matches!(links[0].style(), Style::MdLink));
        assert_eq!(links[0].target().vault_path(), Some("target.md"));

        Ok(())
    }

    #[test]
    #[expect(
        clippy::indexing_slicing,
        reason = "Test asserts exact count before indexing"
    )]
    fn parses_external_url_link() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "[example](https://example.com)";

        let ParseOutcome {
            links,
            ..
        } = reader.parse_str(markdown)?;
        assert_eq!(links.len(), 1, "expected one link");
        assert!(matches!(links[0].target(), Target::External { .. }));

        Ok(())
    }

    #[test]
    fn parses_links() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "[[link1]] and [[link2]]";

        let parsed = reader.parse_str(markdown)?;

        assert_eq!(parsed.links().len(), 2, "note should have 2 links");
        Ok(())
    }

    #[test]
    fn parses_frontmatter() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "---
title: Test Note
tags:
  - rust
  - markdown
priority: 1
---

# Content";

        let ParseOutcome {
            frontmatter,
            ..
        } = reader.parse_str(markdown)?;
        let fm = frontmatter.expect("should have frontmatter");

        assert_eq!(
            fm.get_raw("title").and_then(FieldValue::as_str),
            Some("Test Note")
        );
        assert_eq!(
            fm.get_raw("priority").and_then(FieldValue::as_number),
            Some(1.0f64)
        );

        Ok(())
    }

    #[test]
    fn frontmatter_tags_merge_into_note_tags() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "---
tags:
  - alpha
  - beta
---
";

        let ParseOutcome {
            tags,
            ..
        } = reader.parse_str(markdown)?;
        assert_eq!(tags.len(), 2);
        assert!(tags.iter().any(|tag| tag.full_path() == "alpha"));
        assert!(tags.iter().any(|tag| tag.full_path() == "beta"));
        Ok(())
    }

    #[test]
    fn parses_frontmatter_with_nested_objects() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "---
metadata:
  author: John Doe
  version: 2
---

Content";

        let ParseOutcome {
            frontmatter,
            ..
        } = reader.parse_str(markdown)?;
        let fm = frontmatter.expect("should have frontmatter");

        // Check nested object access
        let metadata = fm.get_raw("metadata").expect("should have metadata");
        assert!(metadata.object_fields().is_some());

        Ok(())
    }

    #[test]
    fn no_frontmatter_when_missing() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "# Just a heading\n\nSome content";

        let ParseOutcome {
            frontmatter,
            ..
        } = reader.parse_str(markdown)?;
        assert!(frontmatter.is_none(), "should not have frontmatter");

        Ok(())
    }

    #[test]
    fn parses_frontmatter_into_outcome() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "---
title: My Note
---

# Heading";

        let parsed = reader.parse_str(markdown)?;
        let frontmatter =
            parsed.frontmatter().expect("note should have frontmatter");
        assert_eq!(
            frontmatter.get_raw("title").and_then(FieldValue::as_str),
            Some("My Note")
        );
        Ok(())
    }

    #[test]
    fn code_blocks_do_not_produce_tasks() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "```rust
// - [ ] #task This is in a code block
```

- [ ] #task This is outside code block";

        let ParseOutcome {
            tasks,
            ..
        } = reader.parse_str(markdown)?;

        // Should only find the task outside the code block
        assert_eq!(tasks.len(), 1, "should only find task outside code block");
        let task = tasks.first().expect("should have at least one task");
        assert!(
            task.text().contains("outside code block"),
            "task should be the one outside code block"
        );

        Ok(())
    }

    #[test]
    fn code_blocks_do_not_produce_headings() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "```markdown
# Heading in code block
```

# Real heading";

        let ParseOutcome {
            headings,
            ..
        } = reader.parse_str(markdown)?;

        // Should only find the heading outside the code block
        assert_eq!(
            headings.len(),
            1,
            "should only find heading outside code block"
        );
        let heading =
            headings.first().expect("should have at least one heading");
        assert_eq!(heading.text(), "Real heading");

        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn heading_includes_markdown_link_text() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "# Heading with [link](target.md)";

        let ParseOutcome {
            headings,
            links,
            ..
        } = reader.parse_str(markdown)?;
        assert_eq!(headings.len(), 1);
        let heading = headings.first().expect("heading should exist");
        assert_eq!(heading.text(), "Heading with link");
        assert_eq!(links.len(), 1);
        let link = links.first().expect("link should exist");
        assert_eq!(link.alias(), Some("link"));
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn list_item_includes_markdown_link_text() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "- Item with [link](target.md)";

        let ParseOutcome {
            lists,
            ..
        } = reader.parse_str(markdown)?;
        let list = lists.first().expect("list should exist");
        let item = list.items().next().expect("item should exist");
        assert_eq!(item.text(), "Item with link");
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn link_alias_preserves_break_whitespace() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "[line1\nline2](target.md)";

        let ParseOutcome {
            links,
            ..
        } = reader.parse_str(markdown)?;
        let link = links.first().expect("link should exist");
        assert_eq!(link.alias(), Some("line1 line2"));
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn markdown_image_captures_alt_text() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "![Alt text](image.png)";

        let ParseOutcome {
            links,
            ..
        } = reader.parse_str(markdown)?;
        assert_eq!(links.len(), 1);
        let link = links.first().expect("link should exist");
        assert_eq!(link.alias(), Some("Alt text"));
        assert!(link.is_embed());
        assert!(matches!(link.embed_type(), Some(EmbedType::Image)));
        assert!(matches!(link.style(), Style::MdLink));
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn wiki_embed_preserves_anchor() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "![[note#Section]]";

        let ParseOutcome {
            links,
            ..
        } = reader.parse_str(markdown)?;
        assert_eq!(links.len(), 1);
        let link = links.first().expect("link should exist");
        assert!(link.is_embed());
        assert!(matches!(link.style(), Style::WikiLink));
        assert!(matches!(link.embed_type(), Some(EmbedType::Note)));
        assert!(matches!(
            link.anchor(),
            Some(Anchor::Heading(text)) if text.as_str() == "Section"
        ));
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn external_links_retain_fragments() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "[ext](https://example.com#frag)";

        let ParseOutcome {
            links,
            ..
        } = reader.parse_str(markdown)?;
        let link = links.first().expect("link should exist");
        assert!(link.anchor().is_none());
        assert!(matches!(
            link.target(),
            Target::External { url } if url.as_ref() == "https://example.com#frag"
        ));
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn non_http_schemes_are_external() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "[mail](mailto:team@example.com)";

        let ParseOutcome {
            links,
            ..
        } = reader.parse_str(markdown)?;
        let link = links.first().expect("link should exist");
        assert!(matches!(
            link.target(),
            Target::External { url } if url.as_ref() == "mailto:team@example.com"
        ));
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Assertions are used to fail tests"
    )]
    fn frontmatter_preserves_block_newlines() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "---
desc: |
  line1
  line2
---
";

        let ParseOutcome {
            frontmatter,
            ..
        } = reader.parse_str(markdown)?;
        let fm = frontmatter.expect("frontmatter should exist");
        assert_eq!(
            fm.get_raw("desc").and_then(FieldValue::as_str),
            Some("line1\nline2\n")
        );
        Ok(())
    }

    #[test]
    fn parses_frontmatter_links() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "---
related: [[Note]]
linklist:
  - [[Note2|Alias]]
obj:
  ref: ![[image.png]]
---
";

        let parsed = reader.parse_str(markdown)?;
        let links = parsed.frontmatter_links();
        assert_eq!(links.len(), 3);
        assert!(links.iter().any(|link| {
            link.key() == "related"
                && matches!(
                    link.target(),
                    Target::Unresolved { raw } if raw.as_ref() == "Note"
                )
        }));
        assert!(links.iter().any(|link| {
            link.key() == "linklist" && link.alias() == Some("Alias")
        }));
        assert!(
            links.iter().any(|link| link.key() == "obj.ref" && link.is_embed())
        );
        Ok(())
    }

    #[test]
    fn parses_block_refs() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "First line ^block1
```md
code ^block2
```
- Item ^block3
";

        let parsed = reader.parse_str(markdown)?;
        let block_refs = parsed.block_refs();
        assert_eq!(block_refs.len(), 2);
        assert!(block_refs.iter().any(|block| block.id().as_str() == "block1"));
        assert!(block_refs.iter().any(|block| block.id().as_str() == "block3"));
        Ok(())
    }

    #[test]
    fn list_items_track_parents() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "- Parent
  - Child
- Sibling
";

        let parsed = reader.parse_str(markdown)?;
        let items = parsed.list_items();
        assert_eq!(items.len(), 3);
        let first = items.first().expect("first item");
        let second = items.get(1).expect("second item");
        let third = items.get(2).expect("third item");
        assert_eq!(first.parent(), None);
        assert_eq!(second.depth(), ListDepth::try_new(1)?);
        assert_eq!(second.parent(), Some(first.position()));
        assert_eq!(third.parent(), None);
        Ok(())
    }
}
