//! Markdown ingestion adapter for note parsing.
//!
//! This adapter keeps file I/O and `pulldown-cmark` details out of the note
//! domain. It reads markdown content with [`crate::fs::FsReader`] and produces
//! domain entities by walking the pulldown-cmark event stream once. Parsing is
//! deterministic, test-friendly, and centralized in the adapter layer. The
//! `parse_str` entry point is public only for benchmarks; production code
//! should use `parse` to keep file ingestion in one place.

mod frontmatter;
mod links;
mod lists;
mod sections;
mod state;
mod tags;

use std::{cell::OnceCell, path::Path, time::SystemTime};

use pulldown_cmark::{
    Event, MetadataBlockKind, Options, Parser, Tag as CmarkTag, TagEnd,
    utils::TextMergeWithOffset,
};

use self::{
    frontmatter::{collect_frontmatter_links, collect_frontmatter_tags},
    links::start_link_builder,
    lists::{
        add_list_item, parent_for_depth, promote_task_from_item,
        record_list_item,
    },
    sections::{
        close_section, collect_block_refs, handle_section_start,
        parse_frontmatter_block,
    },
    state::{ListItemBuilder, ListItemRecord},
    tags::collect_tags,
};
use crate::{
    config::aggregate::Config,
    fs::FsReader,
    note::{
        error::{NoteError, NoteIngestError},
        frontmatter::Frontmatter,
        heading::{Heading, HeadingBuilder, HeadingLevel},
        link::{FrontmatterLink, Link, LinkBuilder},
        list::{List, ListDepth, ListItemEntry, ListType},
        position::{LineIndex, SourceByteOffset, SourceLocation},
        structure::{BlockRef, Section, SectionKind},
        tag::Tag as NoteTag,
        task::Task,
    },
};

// ----------------------------------------------------------- //
//                      Markdown Reader                        //
// ----------------------------------------------------------- //

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
        let modified_at = reader.modified_at(path);
        let created_at = reader.created_at(path);

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
                        let position = item.position();
                        let depth = item.depth();
                        let is_checkbox = item.is_checkbox();
                        let status_symbol = item.status_symbol();
                        let raw_text = item.into_text();
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
                        let record = ListItemRecord::new(
                            position,
                            depth,
                            parent,
                            status,
                            promoted_task_id,
                        );
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
    line_index: OnceCell<LineIndex>,
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

    #[inline]
    fn line_index(&self) -> &LineIndex {
        self.line_index.get_or_init(|| LineIndex::new(&self.source))
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
        SourceLocation::try_from_byte_offset_with_index(
            offset,
            &self.source,
            self.line_index().as_slice(),
        )
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
    fn parsed_note_locations_match_direct_lookup() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "first\nsecond";

        let parsed = reader.parse_str(markdown)?;
        let location = parsed.location_for_offset(SourceByteOffset::new(0))?;
        let direct = SourceLocation::try_from_byte_offset(
            SourceByteOffset::new(0),
            markdown,
        )?;

        assert_eq!(location, direct);
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
