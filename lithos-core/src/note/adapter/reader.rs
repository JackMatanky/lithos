//! Markdown ingestion adapter for note parsing.
//!
//! This adapter keeps file I/O and `pulldown-cmark` details out of the note
//! domain. It reads markdown content with [`crate::fs::FsReader`] and produces
//! domain entities by streaming parser events. The design makes parsing
//! deterministic and test-friendly while keeping storage concerns centralized
//! in the adapter layer.

use std::{
    collections::{HashMap, HashSet},
    ops::Range,
    path::Path,
};

use pulldown_cmark::{
    Event, Options, Parser, Tag as CmarkTag, TagEnd, utils::TextMergeWithOffset,
};

use crate::{
    config::{aggregate::Config, task::StatusSymbol},
    fs::FsReader,
    note::{
        adapter::{tag_scanner::TagScanner, task_parser::TaskParser},
        aggregate::Note,
        error::{FrontmatterParseError, NoteError, TaskError},
        frontmatter::Frontmatter,
        link::{Anchor, EmbedType, Link, Target},
        list::{List, ListDepth, ListItem, ListType},
        structure::{Heading, HeadingLevel, Section},
        tag::Tag as NoteTag,
        task::Task,
        types::{SourceByteOffset, SourceByteRange},
        value::FieldValue,
    },
};

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
/// #     aggregate::Config,
/// #     raw::RawConfig,
/// #     vault::{VaultId, VaultRoot},
/// # };
/// # use lithos_core::fs::FsReader;
/// # use lithos_core::note::adapter::reader::NoteReader;
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
    /// Returns [`NoteError`] when file I/O or parsing fails.
    #[inline]
    pub fn parse(
        &self,
        reader: &FsReader,
        path: &Path,
    ) -> Result<ParseOutcome, NoteError> {
        let markdown = reader
            .read_with::<String, crate::fs::ParseError, _>(
                path,
                |_, content| Ok(content.to_owned()),
            )
            .map_err(|error| NoteError::Storage(format!("{error}").into()))?;
        self.parse_str(&markdown)
    }

    /// Parses markdown into lists, tasks, headings, and links.
    ///
    /// This is crate-visible to support unit tests and in-memory parsing while
    /// keeping the public API focused on file-based ingestion.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when parsing fails.
    #[inline]
    pub(crate) fn parse_str(
        &self,
        markdown: &str,
    ) -> Result<ParseOutcome, NoteError> {
        let mut state = ParseState::new(self.config, markdown);

        let events = Parser::new_ext(markdown, self.options).into_offset_iter();
        let merged = TextMergeWithOffset::new(events);
        for (event, range) in merged {
            state.handle_event(event, range)?;
        }

        state.finish()
    }

    /// Parses markdown and applies extracted elements to a note.
    ///
    /// This is the primary entry point for populating a [`Note`] aggregate
    /// from markdown source. Extracts lists, tasks, headings, links, and
    /// frontmatter.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when file I/O or parsing fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::Path;
    /// # use lithos_core::fs::FsReader;
    /// # use lithos_core::note::{
    /// #     adapter::reader::NoteReader,
    /// #     aggregate::{Note, NoteId},
    /// # };
    /// # use lithos_core::config::{
    /// #     aggregate::Config,
    /// #     raw::RawConfig,
    /// #     vault::{VaultId, VaultRoot},
    /// # };
    /// # let unique = format!(
    /// #     "lithos_note_reader_apply_example_{}",
    /// #     std::process::id()
    /// # );
    /// # let root = std::env::temp_dir().join(unique);
    /// # std::fs::create_dir_all(&root)?;
    /// # std::fs::write(
    /// #     root.join("test.md"),
    /// #     "# Heading\n- [ ] #task Review PR",
    /// # )?;
    /// # let config = Config::build(
    /// #     &RawConfig::default(),
    /// #     VaultId::new(),
    /// #     VaultRoot::try_new(root.clone())?,
    /// # )?;
    /// let reader = FsReader::new(root.as_path());
    /// let note_reader = NoteReader::new(&config);
    /// let mut note = Note::new(NoteId::new(), "test.md")?;
    ///
    /// note_reader.apply(&reader, &mut note, Path::new("test.md"))?;
    /// assert_eq!(note.tasks().count(), 1);
    /// assert_eq!(note.headings().count(), 1);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn apply(
        &self,
        reader: &FsReader,
        note: &mut Note,
        path: &Path,
    ) -> Result<(), NoteError> {
        let parsed = self.parse(reader, path)?;
        parsed.apply_to(note);
        Ok(())
    }

    /// Parses markdown from a string slice and applies extracted elements to a
    /// note.
    ///
    /// This is test-only to keep the public API centered on file ingestion.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when parsing fails.
    #[cfg(test)]
    #[inline]
    pub(crate) fn apply_str(
        &self,
        note: &mut Note,
        markdown: &str,
    ) -> Result<(), NoteError> {
        let parsed = self.parse_str(markdown)?;
        parsed.apply_to(note);
        Ok(())
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
#[derive(Debug)]
#[non_exhaustive]
pub struct ParseOutcome {
    lists: Vec<List>,
    tasks: Vec<Task>,
    headings: Vec<Heading>,
    sections: Vec<Section>,
    links: Vec<Link>,
    tags: Vec<NoteTag>,
    frontmatter: Option<Frontmatter>,
}

impl ParseOutcome {
    /// Lists parsed from the markdown body.
    #[inline]
    #[must_use]
    pub fn lists(&self) -> &[List] {
        &self.lists
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
    fn apply_to(self, note: &mut Note) {
        for list in self.lists {
            note.add_list(list);
        }
        for task in self.tasks {
            note.add_task(task);
        }
        for heading in self.headings {
            note.add_heading(heading);
        }
        for section in self.sections {
            note.add_section(section);
        }
        for link in self.links {
            note.add_link(link);
        }
        for tag in self.tags {
            note.add_tag(tag);
        }
        if let Some(fm) = self.frontmatter {
            note.set_frontmatter(Some(fm));
        }
    }
}

#[derive(Debug)]
struct ParseState<'config, 'source> {
    config: &'config Config,
    task_parser: TaskParser<'config>,
    list_collector: ListCollector,
    tasks: Vec<Task>,
    headings: Vec<Heading>,
    section_collector: SectionCollector<'source>,
    links: Vec<Link>,
    tag_collector: TagCollector,
    frontmatter_collector: FrontmatterCollector,
    item_collector: ItemCollector,
    heading_collector: HeadingCollector,
    link_collector: LinkCollector,
    code_block_depth: u32,
}

impl<'config, 'source> ParseState<'config, 'source> {
    fn new(config: &'config Config, source: &'source str) -> Self {
        Self {
            config,
            task_parser: TaskParser::new(config.task()),
            list_collector: ListCollector::new(),
            tasks: Vec::new(),
            headings: Vec::new(),
            section_collector: SectionCollector::new(source),
            links: Vec::new(),
            tag_collector: TagCollector::new(),
            frontmatter_collector: FrontmatterCollector::new(),
            item_collector: ItemCollector::new(),
            heading_collector: HeadingCollector::new(),
            link_collector: LinkCollector::new(),
            code_block_depth: 0,
        }
    }

    #[tracing::instrument(skip(self, event, range), level = "trace")]
    fn handle_event(
        &mut self,
        event: Event<'_>,
        range: Range<usize>,
    ) -> Result<(), NoteError> {
        self.section_collector.update_last_offset(range.end);
        match event {
            Event::Start(tag) => self.handle_start_tag(tag, range.start)?,
            Event::End(tag_end) => self.handle_end_tag(tag_end)?,
            Event::TaskListMarker(checked) => {
                self.item_collector.set_status(
                    ItemCollector::status_symbol_from_marker(checked)?,
                );
            }
            Event::Text(text) => self.handle_text(&text, false),
            Event::Code(text) => self.handle_text(&text, true),
            Event::SoftBreak | Event::HardBreak => self.handle_break(),
            Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
            | Event::Rule => self
                .section_collector
                .add_rule_section(range.start, range.end)?,
        }

        Ok(())
    }

    fn handle_start_tag(
        &mut self,
        tag: CmarkTag<'_>,
        position: usize,
    ) -> Result<(), NoteError> {
        if Self::is_block_tag(&tag) {
            let is_heading = matches!(tag, CmarkTag::Heading { .. });
            self.section_collector.start_block(position, is_heading)?;
        }
        match tag {
            CmarkTag::List(start) => self.start_list(start)?,
            CmarkTag::Item => self.item_collector.start_item(position)?,
            CmarkTag::Heading {
                level,
                ..
            } => self.heading_collector.start_heading(level, position)?,
            CmarkTag::MetadataBlock(kind) => {
                self.frontmatter_collector.start(kind);
            }
            CmarkTag::Link {
                link_type,
                dest_url,
                ..
            } => self
                .link_collector
                .start_link(link_type, &dest_url, position, false)?,
            CmarkTag::Image {
                link_type,
                dest_url,
                ..
            } => self
                .link_collector
                .start_link(link_type, &dest_url, position, true)?,
            CmarkTag::CodeBlock(_) => {
                self.code_block_depth = self.code_block_depth.saturating_add(1);
            }
            CmarkTag::Paragraph
            | CmarkTag::BlockQuote(_)
            | CmarkTag::HtmlBlock
            | CmarkTag::FootnoteDefinition(_)
            | CmarkTag::DefinitionList
            | CmarkTag::DefinitionListTitle
            | CmarkTag::DefinitionListDefinition
            | CmarkTag::Table(_)
            | CmarkTag::TableHead
            | CmarkTag::TableRow
            | CmarkTag::TableCell
            | CmarkTag::Emphasis
            | CmarkTag::Strong
            | CmarkTag::Strikethrough
            | CmarkTag::Superscript
            | CmarkTag::Subscript => {}
        }
        Ok(())
    }

    fn handle_end_tag(&mut self, tag_end: TagEnd) -> Result<(), NoteError> {
        let mut close_block = false;
        match tag_end {
            TagEnd::List(_) => {
                self.end_list();
                close_block = true;
            }
            TagEnd::Item => self.item_collector.end_item(
                self.list_collector.current_stack_mut(),
                &mut self.tasks,
                self.task_parser,
            )?,
            TagEnd::Heading(_) => {
                self.heading_collector.end_heading(
                    &mut self.headings,
                    &mut self.section_collector,
                )?;
                close_block = true;
            }
            TagEnd::Link | TagEnd::Image => {
                self.link_collector.end_link(&mut self.links)?;
            }
            TagEnd::MetadataBlock(kind) => {
                self.frontmatter_collector.end(kind)?;
            }
            TagEnd::CodeBlock => {
                self.code_block_depth = self.code_block_depth.saturating_sub(1);
                close_block = true;
            }
            TagEnd::Paragraph
            | TagEnd::BlockQuote(_)
            | TagEnd::HtmlBlock
            | TagEnd::FootnoteDefinition
            | TagEnd::DefinitionList
            | TagEnd::Table => {
                close_block = true;
            }
            TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition
            | TagEnd::TableHead
            | TagEnd::TableRow
            | TagEnd::TableCell
            | TagEnd::Emphasis
            | TagEnd::Strong
            | TagEnd::Strikethrough
            | TagEnd::Superscript
            | TagEnd::Subscript => {}
        }
        if close_block {
            self.section_collector.end_block()?;
        }
        Ok(())
    }

    fn handle_text(&mut self, text: &str, is_code: bool) {
        #[expect(
            clippy::else_if_without_else,
            reason = "No work needed when neither condition applies."
        )]
        if self.frontmatter_collector.is_active() {
            self.frontmatter_collector.push_text(text);
        } else if !is_code && self.code_block_depth == 0 {
            self.collect_tags_from_text(text);
        }

        self.link_collector.collect_alias_text(text);

        self.heading_collector.push_text(text);
        self.item_collector.push_text(
            text,
            is_code,
            self.link_collector.has_open_link(),
        );
    }

    fn handle_break(&mut self) {
        if self.frontmatter_collector.is_active() {
            self.frontmatter_collector.push_break();
            return;
        }

        self.link_collector.collect_alias_break();

        self.heading_collector.push_break();
        self.item_collector.push_break(
            self.link_collector.has_open_link(),
            self.code_block_depth == 0,
        );
    }

    fn is_block_tag(tag: &CmarkTag<'_>) -> bool {
        matches!(
            tag,
            CmarkTag::List(_)
                | CmarkTag::Heading { .. }
                | CmarkTag::Paragraph
                | CmarkTag::BlockQuote(_)
                | CmarkTag::CodeBlock(_)
                | CmarkTag::HtmlBlock
                | CmarkTag::FootnoteDefinition(_)
                | CmarkTag::DefinitionList
                | CmarkTag::Table(_)
        )
    }

    fn start_list(&mut self, start: Option<u64>) -> Result<(), NoteError> {
        self.list_collector.start_list(start)?;
        Ok(())
    }

    fn end_list(&mut self) {
        self.list_collector.end_list();
    }

    fn collect_tags_from_text(&mut self, text: &str) {
        self.tag_collector.collect_from_text(text);
    }

    fn finish(mut self) -> Result<ParseOutcome, NoteError> {
        self.section_collector.close()?;
        if let Some(frontmatter) = self.frontmatter_collector.frontmatter() {
            self.tag_collector
                .collect_from_frontmatter(self.config, frontmatter);
        }
        Ok(ParseOutcome {
            lists: self.list_collector.take_lists(),
            tasks: self.tasks,
            headings: self.headings,
            sections: self.section_collector.take_sections(),
            links: self.links,
            tags: self.tag_collector.take_tags(),
            frontmatter: self.frontmatter_collector.take_frontmatter(),
        })
    }
}

#[derive(Debug)]
struct ItemState {
    position: SourceByteOffset,
    text: String,
    tag_scan_text: String,
    status: Option<StatusSymbol>,
}

impl ItemState {
    fn new(position: SourceByteOffset) -> Self {
        Self {
            position,
            text: String::new(),
            tag_scan_text: String::new(),
            status: None,
        }
    }
}

#[derive(Debug, Default)]
struct ItemCollector {
    current: Option<ItemState>,
}

#[derive(Debug, Default)]
struct ListCollector {
    lists: Vec<List>,
    stack: Vec<List>,
}

impl ListCollector {
    fn new() -> Self {
        Self::default()
    }

    fn start_list(&mut self, start: Option<u64>) -> Result<(), NoteError> {
        let depth = ListDepth::try_new(self.stack.len())?;
        let list_type = match start {
            Some(start) => ListType::Ordered {
                start,
            },
            None => ListType::Unordered,
        };
        let list = List::with_depth(list_type, depth);
        self.stack.push(list);
        Ok(())
    }

    fn end_list(&mut self) {
        if let Some(list) = self.stack.pop() {
            self.lists.push(list);
        }
    }

    fn current_stack_mut(&mut self) -> &mut [List] {
        &mut self.stack
    }

    fn take_lists(mut self) -> Vec<List> {
        if !self.stack.is_empty() {
            self.lists.append(&mut self.stack);
        }
        self.lists
    }
}

impl ItemCollector {
    fn new() -> Self {
        Self::default()
    }

    fn start_item(&mut self, start: usize) -> Result<(), NoteError> {
        let position = SourceByteOffset::try_from_usize(start)?;
        self.current = Some(ItemState::new(position));
        Ok(())
    }

    fn end_item(
        &mut self,
        list_stack: &mut [List],
        tasks: &mut Vec<Task>,
        task_parser: TaskParser<'_>,
    ) -> Result<(), NoteError> {
        let Some(item) = self.current.take() else {
            return Ok(());
        };
        let Some(list) = list_stack.last_mut() else {
            return Ok(());
        };

        let raw_text = item.text.trim();
        if let Some(status) = item.status {
            let mut task_id = None;
            let tag_scan_text = item.tag_scan_text.trim();
            let tags = TagScanner::new(tag_scan_text).collect_tags();
            if let Some(task) = task_parser.parse_promoted_checkbox_with_tags(
                raw_text,
                tags,
                status,
                item.position,
            )? {
                task_id = Some(task.id());
                tasks.push(task);
            }

            list.add_item(ListItem::Checkbox {
                text: raw_text.into(),
                status,
                position: item.position,
                task_id,
            });
        } else {
            list.add_item(ListItem::Plain {
                text: raw_text.into(),
                position: item.position,
            });
        }

        Ok(())
    }

    fn push_text(&mut self, text: &str, is_code: bool, has_open_link: bool) {
        if let Some(item) = self.current.as_mut() {
            item.text.push_str(text);
            if !is_code && !has_open_link {
                item.tag_scan_text.push_str(text);
            }
        }
    }

    fn push_break(&mut self, has_open_link: bool, track_tags: bool) {
        if let Some(item) = self.current.as_mut() {
            item.text.push(' ');
            if !has_open_link && track_tags {
                item.tag_scan_text.push(' ');
            }
        }
    }

    fn set_status(&mut self, status: StatusSymbol) {
        if let Some(item) = self.current.as_mut() {
            item.status = Some(status);
        }
    }

    fn status_symbol_from_marker(
        checked: bool,
    ) -> Result<StatusSymbol, NoteError> {
        // pulldown-cmark only exposes a checked boolean, so custom symbols in
        // the source cannot be recovered here.
        let symbol = if checked {
            'x'
        } else {
            ' '
        };
        StatusSymbol::try_new(symbol).map_err(|error| {
            NoteError::Task(TaskError::InvalidStatusSymbol {
                symbol,
                reason: error.to_string().into(),
            })
        })
    }
}

#[derive(Debug)]
struct HeadingState {
    level: HeadingLevel,
    text: String,
    position: SourceByteOffset,
}

#[derive(Debug, Default)]
struct HeadingCollector {
    current: Option<HeadingState>,
}

#[derive(Debug, Default)]
struct FrontmatterCollector {
    kind: Option<pulldown_cmark::MetadataBlockKind>,
    text: String,
    frontmatter: Option<Frontmatter>,
}

impl FrontmatterCollector {
    fn new() -> Self {
        Self::default()
    }

    fn is_active(&self) -> bool {
        self.kind.is_some()
    }

    #[tracing::instrument(skip(self), level = "debug")]
    fn start(&mut self, kind: pulldown_cmark::MetadataBlockKind) {
        self.kind = Some(kind);
        self.text.clear();
    }

    fn push_text(&mut self, text: &str) {
        self.text.push_str(text);
    }

    fn push_break(&mut self) {
        self.text.push('\n');
    }

    #[tracing::instrument(skip(self), level = "debug")]
    fn end(
        &mut self,
        kind: pulldown_cmark::MetadataBlockKind,
    ) -> Result<(), NoteError> {
        if self.kind != Some(kind) {
            self.kind = None;
            self.text.clear();
            return Ok(());
        }
        self.kind = None;

        if self.text.is_empty() {
            return Ok(());
        }

        let fields = match kind {
            pulldown_cmark::MetadataBlockKind::YamlStyle => {
                let yaml_value: serde_yaml::Value =
                    serde_yaml::from_str(&self.text).map_err(|e| {
                        NoteError::Frontmatter(
                            FrontmatterParseError::InvalidYaml {
                                reason: e.to_string().into(),
                            },
                        )
                    })?;
                Self::yaml_to_field_map(&yaml_value)?
            }
            pulldown_cmark::MetadataBlockKind::PlusesStyle => {
                let toml_value: toml::Value = toml::from_str(&self.text)
                    .map_err(|e| {
                        NoteError::Frontmatter(
                            FrontmatterParseError::InvalidToml {
                                reason: e.to_string().into(),
                            },
                        )
                    })?;
                Self::toml_to_field_map(&toml_value)?
            }
        };

        self.frontmatter = Some(Frontmatter::new(fields));
        self.text.clear();

        Ok(())
    }

    fn take_frontmatter(self) -> Option<Frontmatter> {
        self.frontmatter
    }

    fn frontmatter(&self) -> Option<&Frontmatter> {
        self.frontmatter.as_ref()
    }

    fn yaml_to_field_map(
        value: &serde_yaml::Value,
    ) -> Result<HashMap<Box<str>, FieldValue>, NoteError> {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "matching on &Value keeps conversion concise"
        )]
        let serde_yaml::Value::Mapping(map) = value else {
            return Err(NoteError::Frontmatter(
                FrontmatterParseError::NotYamlMapping,
            ));
        };

        let mut fields = HashMap::with_capacity(map.len());

        for (key, value_item) in map {
            let key_str = key.as_str().ok_or(NoteError::Frontmatter(
                FrontmatterParseError::NonStringKey,
            ))?;

            let field_value =
                FieldValue::from_yaml(value_item).map_err(|error| {
                    NoteError::Frontmatter(
                        FrontmatterParseError::InvalidYamlValue {
                            reason: error.to_string().into(),
                        },
                    )
                })?;
            fields.insert(key_str.into(), field_value);
        }

        Ok(fields)
    }

    fn toml_to_field_map(
        value: &toml::Value,
    ) -> Result<HashMap<Box<str>, FieldValue>, NoteError> {
        let table = value.as_table().ok_or(NoteError::Frontmatter(
            FrontmatterParseError::NotTomlTable,
        ))?;

        let mut fields = HashMap::with_capacity(table.len());

        for (key, value_item) in table {
            let field_value = Self::field_value_from_toml(value_item)?;
            fields.insert(key.as_str().into(), field_value);
        }

        Ok(fields)
    }

    fn field_value_from_toml(
        value: &toml::Value,
    ) -> Result<FieldValue, NoteError> {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "matching on &Value keeps conversion concise"
        )]
        match value {
            toml::Value::String(text) => {
                Ok(FieldValue::String(text.clone().into()))
            }
            toml::Value::Integer(number) => {
                const MAX_SAFE_INTEGER: u64 = 0x0020_0000_0000_0000;
                let magnitude = number.unsigned_abs();
                if magnitude > MAX_SAFE_INTEGER {
                    return Err(NoteError::Frontmatter(
                        FrontmatterParseError::InvalidTomlValue {
                            reason: format!(
                                "integer value '{number}' exceeds safe f64 \
                                 range"
                            )
                            .into(),
                        },
                    ));
                }

                #[expect(
                    clippy::as_conversions,
                    reason = "checked MAX_SAFE_INTEGER ensures exact f64"
                )]
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "checked MAX_SAFE_INTEGER ensures exact f64"
                )]
                let parsed = (*number) as f64;
                Ok(FieldValue::Number(parsed))
            }
            toml::Value::Float(value) => Ok(FieldValue::Number(*value)),
            toml::Value::Boolean(value) => Ok(FieldValue::Boolean(*value)),
            toml::Value::Datetime(datetime) => {
                Ok(FieldValue::String(datetime.to_string().into()))
            }
            toml::Value::Array(values) => {
                let mut items = Vec::with_capacity(values.len());
                for item in values {
                    items.push(Self::field_value_from_toml(item)?);
                }
                Ok(FieldValue::Array(items))
            }
            toml::Value::Table(table) => {
                let mut obj = HashMap::with_capacity(table.len());
                for (key, value_item) in table {
                    obj.insert(
                        key.as_str().into(),
                        Self::field_value_from_toml(value_item)?,
                    );
                }
                Ok(FieldValue::Object(obj))
            }
        }
    }
}

impl HeadingCollector {
    fn new() -> Self {
        Self::default()
    }

    #[tracing::instrument(skip(self, level, position), level = "debug")]
    fn start_heading(
        &mut self,
        level: pulldown_cmark::HeadingLevel,
        position: usize,
    ) -> Result<(), NoteError> {
        let level = match level {
            pulldown_cmark::HeadingLevel::H1 => HeadingLevel::try_new(1)?,
            pulldown_cmark::HeadingLevel::H2 => HeadingLevel::try_new(2)?,
            pulldown_cmark::HeadingLevel::H3 => HeadingLevel::try_new(3)?,
            pulldown_cmark::HeadingLevel::H4 => HeadingLevel::try_new(4)?,
            pulldown_cmark::HeadingLevel::H5 => HeadingLevel::try_new(5)?,
            pulldown_cmark::HeadingLevel::H6 => HeadingLevel::try_new(6)?,
        };

        let position = SourceByteOffset::try_from_usize(position)?;

        self.current = Some(HeadingState {
            level,
            text: String::new(),
            position,
        });

        Ok(())
    }

    #[tracing::instrument(skip(self, headings, sections), level = "debug")]
    fn end_heading(
        &mut self,
        headings: &mut Vec<Heading>,
        sections: &mut SectionCollector<'_>,
    ) -> Result<(), NoteError> {
        let Some(heading_state) = self.current.take() else {
            return Ok(());
        };

        let heading = Heading::new(
            heading_state.level,
            heading_state.text,
            heading_state.position,
        )?;

        sections.maybe_assign_heading(&heading);
        headings.push(heading);
        Ok(())
    }

    fn push_text(&mut self, text: &str) {
        if let Some(heading) = self.current.as_mut() {
            heading.text.push_str(text);
        }
    }

    fn push_break(&mut self) {
        if let Some(heading) = self.current.as_mut() {
            heading.text.push(' ');
        }
    }
}

#[derive(Debug)]
struct SectionState {
    start: SourceByteOffset,
    heading: Option<Heading>,
    awaiting_heading: bool,
}

#[derive(Debug)]
struct SectionCollector<'source> {
    source: &'source str,
    block_depth: u32,
    current: Option<SectionState>,
    last_offset: usize,
    sections: Vec<Section>,
}

#[derive(Debug, Default)]
struct TagCollector {
    tags: Vec<NoteTag>,
    tag_set: HashSet<Box<str>>,
}

impl TagCollector {
    fn new() -> Self {
        Self::default()
    }

    fn collect_from_text(&mut self, text: &str) {
        for tag in TagScanner::new(text).collect_tags() {
            self.add_tag(tag);
        }
    }

    fn collect_from_frontmatter(
        &mut self,
        config: &Config,
        frontmatter: &Frontmatter,
    ) {
        let key = config.frontmatter().tags();
        let Some(value) = frontmatter.get(key) else {
            return;
        };

        if let Some(text) = value.as_str() {
            self.collect_from_tokens(text);
            return;
        }

        if let Some(values) = value.as_array() {
            for item in values {
                if let Some(text) = item.as_str() {
                    self.collect_from_tokens(text);
                }
            }
        }
    }

    fn collect_from_tokens(&mut self, text: &str) {
        for token in text.split(|ch: char| ch.is_whitespace() || ch == ',') {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }

            if let Ok(tag) = NoteTag::from_token(token) {
                self.add_tag(tag);
            }
        }
    }

    fn add_tag(&mut self, tag: NoteTag) {
        let key: Box<str> = tag.full_path().into();
        if self.tag_set.insert(key) {
            self.tags.push(tag);
        }
    }

    fn take_tags(self) -> Vec<NoteTag> {
        self.tags
    }
}

impl<'source> SectionCollector<'source> {
    fn new(source: &'source str) -> Self {
        Self {
            source,
            block_depth: 0,
            current: None,
            last_offset: 0,
            sections: Vec::new(),
        }
    }

    fn update_last_offset(&mut self, offset: usize) {
        self.last_offset = offset;
    }

    fn start_block(
        &mut self,
        position: usize,
        is_heading: bool,
    ) -> Result<(), NoteError> {
        if self.block_depth == 0 {
            let start = SourceByteOffset::try_from_usize(position)?;
            self.current = Some(SectionState {
                start,
                heading: None,
                awaiting_heading: is_heading,
            });
        }
        self.block_depth = self.block_depth.saturating_add(1);
        Ok(())
    }

    fn end_block(&mut self) -> Result<(), NoteError> {
        if self.block_depth == 0 {
            return Ok(());
        }
        self.block_depth = self.block_depth.saturating_sub(1);
        if self.block_depth == 0 {
            self.close_current()?;
        }
        Ok(())
    }

    fn add_rule_section(
        &mut self,
        start: usize,
        end: usize,
    ) -> Result<(), NoteError> {
        if self.block_depth != 0 {
            return Ok(());
        }
        let start = SourceByteOffset::try_from_usize(start)?;
        let end = SourceByteOffset::try_from_usize(end)?;
        self.push_section(None, start, end)
    }

    fn maybe_assign_heading(&mut self, heading: &Heading) {
        if let Some(section) = self.current.as_mut()
            && section.awaiting_heading
        {
            section.heading = Some(heading.clone());
            section.awaiting_heading = false;
        }
    }

    fn close(&mut self) -> Result<(), NoteError> {
        if self.current.is_some() {
            self.close_current()?;
        }
        Ok(())
    }

    fn take_sections(self) -> Vec<Section> {
        self.sections
    }

    fn close_current(&mut self) -> Result<(), NoteError> {
        let Some(section) = self.current.take() else {
            return Ok(());
        };
        let end = SourceByteOffset::try_from_usize(self.last_offset)?;
        self.push_section(section.heading, section.start, end)
    }

    fn push_section(
        &mut self,
        heading: Option<Heading>,
        start: SourceByteOffset,
        end: SourceByteOffset,
    ) -> Result<(), NoteError> {
        let range = SourceByteRange::new(start, end)?;
        let start = usize::from(start);
        let end = usize::from(end);
        self.source.get(start..end).ok_or_else(|| {
            NoteError::Structure("section range is not on a boundary".into())
        })?;
        self.sections.push(Section::new(heading, range));
        Ok(())
    }
}

#[derive(Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Parser state flags are naturally independent booleans"
)]
struct LinkState {
    target: Box<str>,
    alias: Option<String>,
    position: SourceByteOffset,
    is_embed: bool,
    is_wikilink: bool,
    is_markdown_image: bool,
    is_external: bool,
    collect_alias: bool,
}

#[derive(Debug, Default)]
struct LinkCollector {
    current: Option<LinkState>,
}

impl LinkCollector {
    fn new() -> Self {
        Self::default()
    }

    #[tracing::instrument(
        skip(self, link_type, dest_url),
        level = "debug",
        fields(dest_url = %dest_url, is_embed)
    )]
    fn start_link(
        &mut self,
        link_type: pulldown_cmark::LinkType,
        dest_url: &pulldown_cmark::CowStr<'_>,
        position: usize,
        is_embed: bool,
    ) -> Result<(), NoteError> {
        use pulldown_cmark::LinkType as PLinkType;

        let position = SourceByteOffset::try_from_usize(position)?;

        match link_type {
            PLinkType::WikiLink {
                has_pothole,
            } => {
                self.current = Some(LinkState {
                    target: dest_url.as_ref().into(),
                    alias: None,
                    position,
                    is_embed,
                    is_wikilink: true,
                    is_markdown_image: false,
                    is_external: false,
                    collect_alias: has_pothole,
                });
            }
            PLinkType::Autolink | PLinkType::Email => {
                let target = dest_url.as_ref();
                self.current = Some(LinkState {
                    target: target.into(),
                    alias: None,
                    position,
                    is_embed,
                    is_wikilink: false,
                    is_markdown_image: is_embed,
                    is_external: true,
                    collect_alias: false,
                });
            }
            PLinkType::Inline
            | PLinkType::Reference
            | PLinkType::ReferenceUnknown
            | PLinkType::Collapsed
            | PLinkType::CollapsedUnknown
            | PLinkType::Shortcut
            | PLinkType::ShortcutUnknown => {
                let target = dest_url.as_ref();
                let is_external = Self::is_external_link(link_type, target);
                self.current = Some(LinkState {
                    target: target.into(),
                    alias: None,
                    position,
                    is_embed,
                    is_wikilink: false,
                    is_markdown_image: is_embed,
                    is_external,
                    collect_alias: true,
                });
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, links), level = "debug")]
    fn end_link(&mut self, links: &mut Vec<Link>) -> Result<(), NoteError> {
        let Some(link_state) = self.current.take() else {
            return Ok(());
        };

        let link = Self::build_link(&link_state)?;
        links.push(link);
        Ok(())
    }

    fn collect_alias_text(&mut self, text: &str) {
        if let Some(link) = self.current.as_mut()
            && link.collect_alias
        {
            let alias = link.alias.get_or_insert_with(String::new);
            alias.push_str(text);
        }
    }

    fn collect_alias_break(&mut self) {
        if let Some(link) = self.current.as_mut()
            && link.collect_alias
        {
            let alias = link.alias.get_or_insert_with(String::new);
            alias.push(' ');
        }
    }

    fn has_open_link(&self) -> bool {
        self.current.is_some()
    }

    fn build_link(state: &LinkState) -> Result<Link, NoteError> {
        let raw_target = state.target.as_ref();
        let (target_str, anchor) = if state.is_external {
            (raw_target, None)
        } else if let Some(pothole_idx) = raw_target.find('#') {
            let (target, anchor_part) = raw_target.split_at(pothole_idx);
            let anchor = if let Some(block_ref) = anchor_part.strip_prefix("#^")
            {
                let block_ref = block_ref.trim();
                if block_ref.is_empty() {
                    None
                } else {
                    Some(Anchor::block_ref(block_ref)?)
                }
            } else {
                let anchor_part = anchor_part.strip_prefix('#').unwrap_or("");
                let anchor_part = anchor_part.trim();
                if anchor_part.is_empty() {
                    None
                } else {
                    Some(Anchor::heading(anchor_part)?)
                }
            };
            (target, anchor)
        } else {
            (raw_target, None)
        };

        let target = if state.is_external {
            Target::External {
                url: raw_target.into(),
            }
        } else {
            Target::Unresolved {
                raw: target_str.into(),
            }
        };

        if state.is_markdown_image {
            let embed_type = Self::determine_embed_type(target_str);
            Link::new_markdown_embed(
                target,
                embed_type,
                state.alias.as_deref(),
                state.position,
            )
        } else if state.is_embed {
            let embed_type = Self::determine_embed_type(target_str);
            Link::new_embed(
                target,
                embed_type,
                state.alias.as_deref(),
                anchor,
                state.position,
            )
        } else if state.is_wikilink {
            Link::new_wikilink(
                target,
                state.alias.as_deref(),
                anchor,
                state.position,
            )
        } else {
            Link::new_markdown_link(
                target,
                state.alias.as_deref(),
                anchor,
                state.position,
            )
        }
    }

    /// Determine embed type from file extension.
    fn determine_embed_type(path: &str) -> EmbedType {
        let Some((_, ext)) = path.rsplit_once('.') else {
            return EmbedType::Note;
        };

        // Use case-insensitive comparison without allocation
        if ext.eq_ignore_ascii_case("png")
            || ext.eq_ignore_ascii_case("jpg")
            || ext.eq_ignore_ascii_case("jpeg")
            || ext.eq_ignore_ascii_case("gif")
            || ext.eq_ignore_ascii_case("svg")
            || ext.eq_ignore_ascii_case("webp")
        {
            return EmbedType::Image;
        }

        if ext.eq_ignore_ascii_case("mp4")
            || ext.eq_ignore_ascii_case("webm")
            || ext.eq_ignore_ascii_case("ogv")
            || ext.eq_ignore_ascii_case("mov")
        {
            return EmbedType::Video;
        }

        if ext.eq_ignore_ascii_case("mp3")
            || ext.eq_ignore_ascii_case("wav")
            || ext.eq_ignore_ascii_case("ogg")
            || ext.eq_ignore_ascii_case("m4a")
        {
            return EmbedType::Audio;
        }

        if ext.eq_ignore_ascii_case("pdf") {
            return EmbedType::Pdf;
        }

        EmbedType::Note
    }

    /// Check if link target should be treated as external.
    fn is_external_link(
        link_type: pulldown_cmark::LinkType,
        target: &str,
    ) -> bool {
        matches!(
            link_type,
            pulldown_cmark::LinkType::Autolink
                | pulldown_cmark::LinkType::Email
        ) || Self::has_scheme(target)
    }

    fn has_scheme(target: &str) -> bool {
        let mut chars = target.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        if !first.is_ascii_alphabetic() {
            return false;
        }
        for ch in chars {
            if ch == ':' {
                return true;
            }
            if !(ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.')) {
                return false;
            }
        }
        false
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
            aggregate::NoteId,
            link::{Anchor, EmbedType, Style, Target},
        },
    };

    fn test_config() -> Config {
        let raw = RawConfig::default();

        Config::build(
            &raw,
            VaultId::new(),
            VaultRoot::try_new(std::path::PathBuf::from("/vault"))
                .expect("vault root"),
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
                    "expected checkbox list item".into(),
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
    fn apply_appends_lists_and_tasks() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "- [ ] #task Review PR\n";

        let mut note = Note::new(NoteId::new(), "notes/test.md")?;

        reader.apply_str(&mut note, markdown)?;

        assert_eq!(note.lists().count(), 1, "note should have 1 list");
        assert_eq!(note.tasks().count(), 1, "note should have 1 task");
        Ok(())
    }

    #[test]
    #[expect(
        clippy::indexing_slicing,
        reason = "Test asserts exact count before indexing"
    )]
    fn apply_appends_headings() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "# Section 1\n\nContent\n\n## Section 2";

        let mut note = Note::new(NoteId::new(), "notes/test.md")?;

        reader.apply_str(&mut note, markdown)?;

        assert_eq!(note.headings().count(), 2, "note should have 2 headings");
        let headings: Vec<_> = note.headings().collect();
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
    fn apply_appends_links() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "[[link1]] and [[link2]]";

        let mut note = Note::new(NoteId::new(), "notes/test.md")?;

        reader.apply_str(&mut note, markdown)?;

        assert_eq!(note.links().count(), 2, "note should have 2 links");
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
    fn apply_sets_frontmatter() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "---
title: My Note
---

# Heading";

        let mut note = Note::new(NoteId::new(), "notes/test.md")?;

        reader.apply_str(&mut note, markdown)?;

        let frontmatter =
            note.frontmatter().expect("note should have frontmatter");
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
}
