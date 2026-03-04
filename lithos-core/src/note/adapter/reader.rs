//! Markdown ingestion adapter for note parsing.
//!
//! This adapter keeps file I/O and `pulldown-cmark` details out of the note
//! domain. It reads markdown content with [`crate::fs::FsReader`] and produces
//! domain entities by streaming parser events. The design makes parsing
//! deterministic and test-friendly while keeping storage concerns centralized
//! in the adapter layer.

use std::{ops::Range, path::Path};

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use crate::{
    config::{aggregate::Config, task::StatusSymbol},
    fs::FsReader,
    note::{
        aggregate::Note,
        error::NoteError,
        frontmatter::Frontmatter,
        link::{Anchor, EmbedType, Link, Target},
        list::{List, ListItem, ListType},
        structure::{Heading, HeadingLevel},
        task::Task,
        types::SourceByteOffset,
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
/// let (lists, tasks, headings, links, _frontmatter) =
///     note_reader.parse(&reader, Path::new("note.md"))?;
///
/// assert_eq!(tasks.len(), 1);
/// assert_eq!(headings.len(), 1);
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
            .map_err(|error| NoteError::Storage(format!("{error}")))?;
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
        let mut state = ParseState::new(self.config);

        for (event, range) in
            Parser::new_ext(markdown, self.options).into_offset_iter()
        {
            state.handle_event(event, range)?;
        }

        Ok(state.finish())
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
        Self::apply_parts(note, parsed);
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
        Self::apply_parts(note, parsed);
        Ok(())
    }

    #[inline]
    fn apply_parts(note: &mut Note, parsed: ParseOutcome) {
        let (lists, tasks, headings, links, frontmatter) = parsed;
        for list in lists {
            note.add_list(list);
        }
        for task in tasks {
            note.add_task(task);
        }
        for heading in headings {
            note.add_heading(heading);
        }
        for link in links {
            note.add_link(link);
        }
        if let Some(fm) = frontmatter {
            note.set_frontmatter(Some(fm));
        }
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
        .union(Options::ENABLE_HEADING_ATTRIBUTES)
        .union(Options::ENABLE_TABLES)
        .union(Options::ENABLE_FOOTNOTES)
        .union(Options::ENABLE_STRIKETHROUGH)
        .union(Options::ENABLE_MATH)
}

type ParseOutcome =
    (Vec<List>, Vec<Task>, Vec<Heading>, Vec<Link>, Option<Frontmatter>);

#[derive(Debug)]
struct ParseState<'config> {
    config: &'config Config,
    lists: Vec<List>,
    tasks: Vec<Task>,
    headings: Vec<Heading>,
    links: Vec<Link>,
    frontmatter: Option<Frontmatter>,
    metadata_text: String,
    in_metadata_block: bool,
    list_stack: Vec<List>,
    current_item: Option<ItemState>,
    current_heading: Option<HeadingState>,
    current_link: Option<LinkState>,
}

impl<'config> ParseState<'config> {
    fn new(config: &'config Config) -> Self {
        Self {
            config,
            lists: Vec::new(),
            tasks: Vec::new(),
            headings: Vec::new(),
            links: Vec::new(),
            frontmatter: None,
            metadata_text: String::new(),
            in_metadata_block: false,
            list_stack: Vec::with_capacity(4),
            current_item: None,
            current_heading: None,
            current_link: None,
        }
    }

    #[tracing::instrument(skip(self, event, range), level = "trace")]
    fn handle_event(
        &mut self,
        event: Event<'_>,
        range: Range<usize>,
    ) -> Result<(), NoteError> {
        match event {
            Event::Start(Tag::List(start)) => self.start_list(start)?,
            Event::End(TagEnd::List(_)) => self.end_list(),
            Event::Start(Tag::Item) => self.start_item(range.start)?,
            Event::End(TagEnd::Item) => self.end_item()?,
            Event::Start(Tag::Heading {
                level,
                ..
            }) => self.start_heading(level, range.start)?,
            Event::End(TagEnd::Heading(_)) => self.end_heading()?,
            Event::Start(Tag::MetadataBlock(
                pulldown_cmark::MetadataBlockKind::YamlStyle,
            )) => {
                self.start_metadata_block();
            }
            Event::End(TagEnd::MetadataBlock(
                pulldown_cmark::MetadataBlockKind::YamlStyle,
            )) => {
                self.end_metadata_block()?;
            }
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                ..
            }) => self.start_link(link_type, &dest_url, range.start, false)?,
            Event::End(TagEnd::Link | TagEnd::Image) => {
                self.end_link()?;
            }
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                ..
            }) => self.start_link(link_type, &dest_url, range.start, true)?,
            Event::TaskListMarker(checked) => {
                if let Some(item) = self.current_item.as_mut() {
                    item.status = Some(status_symbol_from_marker(checked)?);
                }
            }
            Event::Text(text) | Event::Code(text) => {
                if self.in_metadata_block {
                    self.metadata_text.push_str(&text);
                } else if let Some(link) = self.current_link.as_mut() {
                    // For wikilinks with alias, this is the alias text
                    if link.is_wikilink_with_alias {
                        link.alias = Some(text.to_string());
                    }
                } else if let Some(heading) = self.current_heading.as_mut() {
                    heading.text.push_str(&text);
                } else if let Some(item) = self.current_item.as_mut() {
                    item.text.push_str(&text);
                } else {
                    // Text not in a heading, link, or list item - ignore for
                    // now
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(heading) = self.current_heading.as_mut() {
                    heading.text.push(' ');
                } else if let Some(item) = self.current_item.as_mut() {
                    item.text.push(' ');
                } else {
                    // Break not in a heading or list item - ignore for now
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

        Ok(())
    }

    fn start_list(&mut self, start: Option<u64>) -> Result<(), NoteError> {
        let depth = parse_depth(self.list_stack.len())?;
        let list_type = match start {
            Some(start) => ListType::Ordered {
                start,
            },
            None => ListType::Unordered,
        };
        let list = List::with_depth(list_type, depth);
        self.list_stack.push(list);
        Ok(())
    }

    fn end_list(&mut self) {
        if let Some(list) = self.list_stack.pop() {
            self.lists.push(list);
        }
    }

    fn start_item(&mut self, start: usize) -> Result<(), NoteError> {
        let position = parse_offset(start)?;
        self.current_item = Some(ItemState::new(position));
        Ok(())
    }

    fn end_item(&mut self) -> Result<(), NoteError> {
        let Some(item) = self.current_item.take() else {
            return Ok(());
        };
        let Some(list) = self.list_stack.last_mut() else {
            return Ok(());
        };

        let raw_text = item.text.trim();
        if let Some(status) = item.status {
            let mut task_id = None;
            if Task::should_promote(raw_text, self.config.task()) {
                let task = Task::from_checkbox(
                    raw_text,
                    status,
                    item.position,
                    self.config.task(),
                )?;
                task_id = Some(task.id());
                self.tasks.push(task);
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

    #[expect(
        clippy::type_complexity,
        reason = "Parse result is naturally a 5-tuple"
    )]
    fn finish(
        mut self,
    ) -> (Vec<List>, Vec<Task>, Vec<Heading>, Vec<Link>, Option<Frontmatter>)
    {
        if !self.list_stack.is_empty() {
            self.lists.append(&mut self.list_stack);
        }
        (self.lists, self.tasks, self.headings, self.links, self.frontmatter)
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

        let position = parse_offset(position)?;

        self.current_heading = Some(HeadingState {
            level,
            text: String::new(),
            position,
        });

        Ok(())
    }

    #[tracing::instrument(skip(self), level = "debug")]
    fn end_heading(&mut self) -> Result<(), NoteError> {
        let Some(heading_state) = self.current_heading.take() else {
            return Ok(());
        };

        let heading = Heading::new(
            heading_state.level,
            heading_state.text,
            heading_state.position,
        )?;

        self.headings.push(heading);
        Ok(())
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

        let position = parse_offset(position)?;

        match link_type {
            PLinkType::WikiLink {
                has_pothole,
            } => {
                // WikiLink: [[target]] or [[target|alias]]
                // dest_url contains: "target" or "target#heading" or
                // "target#^blockref"
                self.current_link = Some(LinkState {
                    target: dest_url.as_ref().into(),
                    alias: None,
                    position,
                    is_embed,
                    is_wikilink: true,
                    is_wikilink_with_alias: has_pothole,
                });
            }
            PLinkType::Inline
            | PLinkType::Reference
            | PLinkType::ReferenceUnknown
            | PLinkType::Collapsed
            | PLinkType::CollapsedUnknown
            | PLinkType::Shortcut
            | PLinkType::ShortcutUnknown
            | PLinkType::Autolink
            | PLinkType::Email => {
                // Standard markdown link: [text](url)
                self.current_link = Some(LinkState {
                    target: dest_url.as_ref().into(),
                    alias: None,
                    position,
                    is_embed,
                    is_wikilink: false,
                    is_wikilink_with_alias: false,
                });
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self), level = "debug")]
    fn end_link(&mut self) -> Result<(), NoteError> {
        let Some(link_state) = self.current_link.take() else {
            return Ok(());
        };

        let link = Self::build_link(&link_state)?;
        self.links.push(link);
        Ok(())
    }

    fn build_link(state: &LinkState) -> Result<Link, NoteError> {
        let raw_target = state.target.as_ref();
        let anchor_info = if let Some(pothole_idx) = raw_target.find('#') {
            let (target, anchor_part) = raw_target.split_at(pothole_idx);
            let anchor = if let Some(block_ref) = anchor_part.strip_prefix("#^")
            {
                Some(Anchor::BlockRef(block_ref.into()))
            } else {
                let anchor_part = anchor_part.strip_prefix('#').unwrap_or("");
                Some(Anchor::Heading(anchor_part.into()))
            };
            (target, anchor)
        } else {
            (raw_target, None)
        };

        let (target_str, anchor) = anchor_info;

        if state.is_embed {
            // ![[embed]] syntax
            let embed_type = determine_embed_type(target_str);
            Link::new_embed(
                Target::Unresolved {
                    raw: target_str.into(),
                },
                embed_type,
                state.alias.as_deref(),
                state.position,
            )
        } else if state.is_wikilink {
            // [[link]] syntax
            Link::new_wikilink(
                Target::Unresolved {
                    raw: target_str.into(),
                },
                state.alias.as_deref(),
                anchor,
                state.position,
            )
        } else {
            // Standard markdown link
            let target = if is_external_url(target_str) {
                Target::External {
                    url: target_str.into(),
                }
            } else {
                Target::Unresolved {
                    raw: target_str.into(),
                }
            };
            Link::new_markdown_link(
                target,
                state.alias.as_deref(),
                anchor,
                state.position,
            )
        }
    }

    #[tracing::instrument(skip(self), level = "debug")]
    fn start_metadata_block(&mut self) {
        self.in_metadata_block = true;
        self.metadata_text.clear();
    }

    #[tracing::instrument(skip(self), level = "debug")]
    fn end_metadata_block(&mut self) -> Result<(), NoteError> {
        self.in_metadata_block = false;

        if self.metadata_text.is_empty() {
            return Ok(());
        }

        // Parse YAML using serde_yaml
        let yaml_value: serde_yaml::Value =
            serde_yaml::from_str(&self.metadata_text).map_err(|e| {
                NoteError::Frontmatter(format!("invalid YAML: {e}"))
            })?;

        // Convert to our FieldValue type
        let fields = yaml_to_field_map(&yaml_value)?;

        self.frontmatter = Some(Frontmatter::new(fields)?);
        self.metadata_text.clear();

        Ok(())
    }
}

/// Convert `serde_yaml` Value to our `FieldValue` map.
///
/// Uses `FieldValue::from_yaml()` for the conversion logic, which is
/// centralized in the value module.
#[expect(
    clippy::pattern_type_mismatch,
    reason = "matching on &Value is clearer than *value for YAML"
)]
fn yaml_to_field_map(
    yaml: &serde_yaml::Value,
) -> Result<std::collections::HashMap<Box<str>, FieldValue>, NoteError> {
    let serde_yaml::Value::Mapping(map) = yaml else {
        return Err(NoteError::Frontmatter(
            "frontmatter must be a YAML mapping".into(),
        ));
    };

    let mut fields = std::collections::HashMap::with_capacity(map.len());

    for (key, value) in map {
        let key_str = key
            .as_str()
            .ok_or_else(|| NoteError::Frontmatter("non-string key".into()))?;

        let field_value =
            FieldValue::from_yaml(value).map_err(NoteError::Frontmatter)?;
        fields.insert(key_str.into(), field_value);
    }

    Ok(fields)
}

#[derive(Debug)]
struct ItemState {
    position: SourceByteOffset,
    text: String,
    status: Option<StatusSymbol>,
}

impl ItemState {
    fn new(position: SourceByteOffset) -> Self {
        Self {
            position,
            text: String::new(),
            status: None,
        }
    }
}

#[derive(Debug)]
struct HeadingState {
    level: HeadingLevel,
    text: String,
    position: SourceByteOffset,
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
    is_wikilink_with_alias: bool,
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

/// Check if URL is external (http/https).
fn is_external_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

fn parse_offset(offset: usize) -> Result<SourceByteOffset, NoteError> {
    SourceByteOffset::try_from(offset).map_err(|error| {
        NoteError::Structure(format!("source offset out of range: {error}"))
    })
}

fn parse_depth(depth: usize) -> Result<u8, NoteError> {
    u8::try_from(depth).map_err(|error| {
        NoteError::Structure(format!("list depth out of range: {error}"))
    })
}

fn status_symbol_from_marker(checked: bool) -> Result<StatusSymbol, NoteError> {
    let symbol = if checked {
        'x'
    } else {
        ' '
    };
    StatusSymbol::try_new(symbol).map_err(|error| {
        NoteError::Task(format!("invalid status symbol '{symbol}': {error}"))
    })
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

        let (lists, tasks, _headings, _links, _frontmatter) =
            reader.parse_str(markdown)?;
        assert_eq!(lists.len(), 1, "expected one list");
        assert_eq!(tasks.len(), 1, "expected one promoted task");

        let list = lists.first().expect("list should exist");
        assert!(matches!(list.list_type(), ListType::Unordered));
        assert_eq!(list.items().len(), 2, "expected two list items");

        let first_item = list.items().first().expect("first item");
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
                    "expected checkbox list item".to_owned(),
                ));
            }
        };
        assert_eq!(status.value(), ' ', "expected unchecked status");
        assert!(task_id.is_some(), "expected promoted task id");

        let task = tasks.first().expect("task should exist");
        assert_eq!(task_id, &Some(task.id()));
        Ok(())
    }

    #[test]
    fn captures_list_depths() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "1. First\n   - [ ] #task Nested\n";

        let (lists, _tasks, _headings, _links, _frontmatter) =
            reader.parse_str(markdown)?;
        assert_eq!(lists.len(), 2, "expected two lists");

        let ordered = lists
            .iter()
            .find(|list| matches!(list.list_type(), ListType::Ordered { .. }))
            .expect("ordered list should exist");
        assert_eq!(ordered.depth(), 0, "ordered list should be top-level");

        let unordered = lists
            .iter()
            .find(|list| matches!(list.list_type(), ListType::Unordered))
            .expect("unordered list should exist");
        assert_eq!(unordered.depth(), 1, "nested list should have depth 1");
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

        let (_lists, _tasks, _headings, links, _frontmatter) =
            reader.parse_str(markdown)?;
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

        let (_lists, _tasks, _headings, links, _frontmatter) =
            reader.parse_str(markdown)?;
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

        let (_lists, _tasks, _headings, links, _frontmatter) =
            reader.parse_str(markdown)?;
        assert_eq!(links.len(), 1, "expected one link");
        assert_eq!(links[0].target().vault_path(), Some("note"));
        assert!(matches!(
            links[0].anchor(),
            Some(Anchor::Heading(text)) if text.as_ref() == "Section Title"
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

        let (_lists, _tasks, _headings, links, _frontmatter) =
            reader.parse_str(markdown)?;
        assert_eq!(links.len(), 1, "expected one link");
        assert_eq!(links[0].target().vault_path(), Some("note"));
        assert!(matches!(
            links[0].anchor(),
            Some(Anchor::BlockRef(text)) if text.as_ref() == "block123"
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

        let (_lists, _tasks, _headings, links, _frontmatter) =
            reader.parse_str(markdown)?;
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

        let (_lists, _tasks, _headings, links, _frontmatter) =
            reader.parse_str(markdown)?;
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

        let (_lists, _tasks, _headings, links, _frontmatter) =
            reader.parse_str(markdown)?;
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

        let (_lists, _tasks, _headings, links, _frontmatter) =
            reader.parse_str(markdown)?;
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

        let (_lists, _tasks, _headings, _links, frontmatter) =
            reader.parse_str(markdown)?;
        let fm = frontmatter.expect("should have frontmatter");

        assert_eq!(
            fm.get("title").and_then(FieldValue::as_str),
            Some("Test Note")
        );
        assert_eq!(
            fm.get("priority").and_then(FieldValue::as_number),
            Some(1.0f64)
        );

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

        let (_lists, _tasks, _headings, _links, frontmatter) =
            reader.parse_str(markdown)?;
        let fm = frontmatter.expect("should have frontmatter");

        // Check nested object access
        let metadata = fm.get("metadata").expect("should have metadata");
        assert!(metadata.as_object().is_some());

        Ok(())
    }

    #[test]
    fn no_frontmatter_when_missing() -> Result<(), NoteError> {
        let config = test_config();
        let reader = NoteReader::new(&config);
        let markdown = "# Just a heading\n\nSome content";

        let (_lists, _tasks, _headings, _links, frontmatter) =
            reader.parse_str(markdown)?;
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
            frontmatter.get("title").and_then(FieldValue::as_str),
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

        let (_lists, tasks, _headings, _links, _frontmatter) =
            reader.parse_str(markdown)?;

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

        let (_lists, _tasks, headings, _links, _frontmatter) =
            reader.parse_str(markdown)?;

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
}
