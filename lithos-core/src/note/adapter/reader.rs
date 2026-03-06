//! Markdown ingestion adapter for note parsing.
//!
//! This adapter keeps file I/O and `pulldown-cmark` details out of the note
//! domain. It reads markdown content with [`crate::fs::FsReader`] and produces
//! domain entities by streaming parser events into extractor state machines.
//! Parsing is deterministic, test-friendly, and centralized in the adapter
//! layer. The `parse_str` entry point is public only for benchmarks; production
//! code should use `parse` to keep file ingestion in one place.

use std::{ops::Range, path::Path, time::SystemTime};

use pulldown_cmark::{
    CowStr, Event, Options, Parser, Tag as CmarkTag, TagEnd,
    utils::TextMergeWithOffset,
};

use crate::{
    config::aggregate::Config,
    fs::FsReader,
    note::{
        error::{NoteError, NoteIngestError},
        frontmatter::Frontmatter,
        link::Link,
        list::List,
        position::{SourceByteOffset, SourceLineIndex, SourceLocation},
        structure::{Heading, Section},
        tag::Tag as NoteTag,
        task::Task,
    },
};

// ----------------------------------------------------------- //
//                    Extraction Protocol                      //
// ----------------------------------------------------------- //

/// Extraction context shared across all extractors.
///
/// Provides global state about the current parsing context that extractors
/// need to make decisions (e.g., whether we're inside a link, code block,
/// etc.).
#[derive(Debug, Default, Clone)]
pub(super) struct ExtractionContext {
    /// Whether the parser is currently inside a link.
    pub inside_link: bool,
    /// Whether the parser is currently inside a code block.
    pub inside_code_block: bool,
    /// Current nesting depth of lists (0 = not in list).
    pub list_depth: usize,
}

/// Extraction state returned after processing an event.
///
/// Indicates whether the extractor should continue processing or has
/// produced an output entity.
#[derive(Debug)]
pub(super) enum ExtractionState<T> {
    /// Continue processing - no entity emitted yet.
    Continue,
    /// Entity extracted and ready to emit.
    Emit(T),
}

/// Extracts typed domain entities from pulldown-cmark event stream.
///
/// Extractors implement a state machine that processes markdown events
/// and emits domain entities when complete patterns are recognized.
///
/// # Type Parameters
///
/// - `Error`: Error type for extraction failures (must convert to `NoteError`)
/// - `Output`: The domain entity type this extractor produces
pub(super) trait Extractor {
    /// Error type for extraction failures.
    type Error: Into<NoteError>;

    /// The domain entity type produced by this extractor.
    type Output;

    /// Finalize extraction and return any buffered entities.
    ///
    /// Called when the event stream ends. Extractors should flush
    /// any incomplete entities or return empty if nothing is buffered.
    fn finish(self) -> Result<Vec<Self::Output>, Self::Error>;

    /// Process a single markdown event.
    ///
    /// Returns `ExtractionState::Continue` to keep processing or
    /// `ExtractionState::Emit` when an entity is ready.
    ///
    /// # Parameters
    ///
    /// - `event`: The pulldown-cmark event being processed
    /// - `text`: Text content (empty for non-text events)
    /// - `range`: Byte range of this event in the source
    /// - `ctx`: Shared extraction context
    fn process(
        &mut self,
        event: &Event<'_>,
        text: pulldown_cmark::CowStr<'_>,
        range: Range<usize>,
        ctx: &ExtractionContext,
    ) -> Result<ExtractionState<Self::Output>, Self::Error>;
}

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
    #[expect(
        clippy::too_many_lines,
        clippy::pattern_type_mismatch,
        reason = "WHAT: long orchestration loop and match ergonomics on \
                  &Event. WHY: extractors reuse the same event references and \
                  the flow is clearer centralized. HOW: keep the loop intact \
                  and match on references without moving events."
    )]
    fn parse_with_timestamps(
        &self,
        markdown: Box<str>,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<ParsedNote, NoteIngestError> {
        let markdown_ref = markdown.as_ref();
        let mut link_ext = super::extract_link::LinkExtractor::new(self.config);
        let mut list_ext = super::extract_list::ListExtractor::new(self.config);
        let mut heading_ext = super::extract_heading::HeadingExtractor::new();
        let mut section_ext =
            super::extract_section::SectionExtractor::new(markdown_ref);
        let mut frontmatter_ext =
            super::extract_frontmatter::FrontmatterExtractor::new();
        let mut tag_ext = super::extract_tag::TagExtractor::new(self.config);
        let line_index = SourceLineIndex::new(markdown_ref);

        let mut links = Vec::new();
        let mut lists = Vec::new();
        let mut tasks = Vec::new();
        let mut headings = Vec::new();
        let mut sections = Vec::new();
        let mut tags = Vec::new();
        let mut frontmatter = None;

        let mut ctx = ExtractionContext::default();
        let mut code_block_depth = 0u32;
        let mut list_depth = 0usize;

        let events =
            Parser::new_ext(markdown_ref, self.options).into_offset_iter();
        let merged = TextMergeWithOffset::new(events);
        for (event, range) in merged {
            Self::update_context(
                &mut ctx,
                &event,
                &mut code_block_depth,
                &mut list_depth,
            );

            let text = match &event {
                Event::Text(text) | Event::Code(text) => text.clone(),
                Event::Start(_)
                | Event::End(_)
                | Event::InlineMath(_)
                | Event::DisplayMath(_)
                | Event::Html(_)
                | Event::InlineHtml(_)
                | Event::FootnoteReference(_)
                | Event::SoftBreak
                | Event::HardBreak
                | Event::Rule
                | Event::TaskListMarker(_) => CowStr::Borrowed(""),
            };

            if let ExtractionState::Emit(output) =
                list_ext.process(&event, text.clone(), range.clone(), &ctx)?
            {
                match output {
                    super::extract_list::ExtractionOutput::List(list) => {
                        lists.push(list);
                    }
                    super::extract_list::ExtractionOutput::Task(task) => {
                        tasks.push(*task);
                    }
                }
            }

            if let ExtractionState::Emit(link) =
                link_ext.process(&event, text.clone(), range.clone(), &ctx)?
            {
                links.push(link);
            }

            if let ExtractionState::Emit(heading) = heading_ext.process(
                &event,
                text.clone(),
                range.clone(),
                &ctx,
            )? {
                headings.push(heading);
            }

            if let ExtractionState::Emit(section) = section_ext.process(
                &event,
                text.clone(),
                range.clone(),
                &ctx,
            )? {
                sections.push(section);
            }

            if let ExtractionState::Emit(fm) = frontmatter_ext.process(
                &event,
                text.clone(),
                range.clone(),
                &ctx,
            )? && frontmatter.is_none()
            {
                tag_ext.set_frontmatter(fm.clone());
                frontmatter = Some(fm);
            }

            if let ExtractionState::Emit(tag) =
                tag_ext.process(&event, text, range, &ctx)?
            {
                tags.push(tag);
            }
        }

        for output in list_ext.finish()? {
            match output {
                super::extract_list::ExtractionOutput::List(list) => {
                    lists.push(list);
                }
                super::extract_list::ExtractionOutput::Task(task) => {
                    tasks.push(*task);
                }
            }
        }
        links.extend(link_ext.finish()?);
        headings.extend(heading_ext.finish()?);
        sections.extend(section_ext.finish()?);
        tags.extend(tag_ext.finish()?);

        Ok(ParsedNote {
            source: markdown,
            lists,
            tasks,
            headings,
            sections,
            links,
            tags,
            frontmatter,
            line_index,
            created_at,
            modified_at,
        })
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &Event preferred for clarity"
    )]
    fn update_context(
        ctx: &mut ExtractionContext,
        event: &Event<'_>,
        code_block_depth: &mut u32,
        list_depth: &mut usize,
    ) {
        match event {
            Event::Start(
                CmarkTag::Link {
                    ..
                }
                | CmarkTag::Image {
                    ..
                },
            ) => {
                ctx.inside_link = true;
            }
            Event::End(TagEnd::Link | TagEnd::Image) => {
                ctx.inside_link = false;
            }
            Event::Start(CmarkTag::CodeBlock(_)) => {
                *code_block_depth = code_block_depth.saturating_add(1);
                ctx.inside_code_block = *code_block_depth > 0;
            }
            Event::End(TagEnd::CodeBlock) => {
                *code_block_depth = code_block_depth.saturating_sub(1);
                ctx.inside_code_block = *code_block_depth > 0;
            }
            Event::Start(CmarkTag::List(_)) => {
                *list_depth = list_depth.saturating_add(1);
                ctx.list_depth = *list_depth;
            }
            Event::End(TagEnd::List(_)) => {
                *list_depth = list_depth.saturating_sub(1);
                ctx.list_depth = *list_depth;
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
///     note::adapter::reader::NoteReader,
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
    tasks: Vec<Task>,
    headings: Vec<Heading>,
    sections: Vec<Section>,
    links: Vec<Link>,
    tags: Vec<NoteTag>,
    frontmatter: Option<Frontmatter>,
    line_index: SourceLineIndex,
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
        &self.line_index
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
        self.line_index.line_column(offset, &self.source)
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
    mod protocol_tests {
        use pulldown_cmark::CowStr;

        use super::*;

        #[test]
        fn extraction_context_defaults() {
            let ctx = ExtractionContext::default();
            assert!(!ctx.inside_link);
            assert!(!ctx.inside_code_block);
            assert_eq!(ctx.list_depth, 0);
        }

        #[test]
        fn extraction_state_is_continue() {
            let state: ExtractionState<String> = ExtractionState::Continue;
            assert!(matches!(state, ExtractionState::Continue));
        }

        #[test]
        fn extraction_state_is_emit() {
            let state = ExtractionState::Emit(String::from("value"));
            assert!(matches!(state, ExtractionState::Emit(_)));
        }

        // Mock extractor for testing protocol
        struct MockExtractor {
            calls: usize,
        }

        impl Extractor for MockExtractor {
            type Error = NoteError;
            type Output = String;

            #[expect(
                clippy::arithmetic_side_effects,
                reason = "Test counter overflow is unrealistic"
            )]
            fn process(
                &mut self,
                _event: &Event<'_>,
                _text: CowStr<'_>,
                _range: Range<usize>,
                _ctx: &ExtractionContext,
            ) -> Result<ExtractionState<String>, NoteError> {
                self.calls += 1;
                if self.calls == 3 {
                    Ok(ExtractionState::Emit(String::from("entity")))
                } else {
                    Ok(ExtractionState::Continue)
                }
            }

            fn finish(self) -> Result<Vec<String>, NoteError> {
                Ok(vec![])
            }
        }

        #[test]
        fn mock_extractor_emits_on_third_call() {
            let mut extractor = MockExtractor {
                calls: 0,
            };
            let ctx = ExtractionContext::default();
            let event = Event::Text(CowStr::Borrowed("test"));

            // First call
            let result1 = extractor
                .process(&event, CowStr::Borrowed("test"), 0..4, &ctx)
                .unwrap();
            assert!(matches!(result1, ExtractionState::Continue));

            // Second call
            let result2 = extractor
                .process(&event, CowStr::Borrowed("test"), 4..8, &ctx)
                .unwrap();
            assert!(matches!(result2, ExtractionState::Continue));

            // Third call - should emit
            let result3 = extractor
                .process(&event, CowStr::Borrowed("test"), 8..12, &ctx)
                .unwrap();
            #[expect(clippy::panic, reason = "Test assertion")]
            let ExtractionState::Emit(value) = result3 else {
                panic!("Expected Emit, got Continue");
            };
            assert_eq!(value, "entity");
        }
    }

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
}
