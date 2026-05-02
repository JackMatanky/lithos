//! Markdown parsing pipeline and orchestration.
//!
//! This module provides the primary ingestion engine for Obsidian-compatible
//! markdown files. It uses a single-pass event stream driven by
//! `pulldown-cmark` to extract both structural components (headings, sections,
//! lists) and specialized metadata (tags, inline fields, block references,
//! frontmatter).
//!
//! # Pipeline Architecture
//!
//! The parsing process follows a strict multi-stage, event-driven pipeline
//! designed for modularity, zero-cost abstractions, and strong separation of
//! concerns. The phases are:
//!
//! 1. **Adapter Layer** ([`stream::MarkdownEventStream`]): Wraps the core
//!    `pulldown-cmark` parser. Normalizes soft and hard line breaks and merges
//!    adjacent text nodes according to configurable policies.
//! 2. **Structure Builder**: Tracks block depth and nesting. Emits completed
//!    leaf and container blocks without allocating text fragments for container
//!    blocks.
//! 3. **Lexical Metadata Scanner**: Scans leaf text segments using explicit
//!    rules to identify artifacts like tags, inline fields, and block
//!    references.
//! 4. **Semantic Validator**: Enforces schema rules and standardizes metadata
//!    tokens.
//! 5. **Artifact Assembler**: Collects structural blocks and validated metadata
//!    into the final domain [`RawNote`](crate::note::raw::RawNote), applying
//!    routing policies (e.g., whether list-item tags are global).
//!
//! # Core Invariants
//!
//! - **Parser isolates grammar**: The parser does not depend on or import
//!   domain types (e.g. `RawTag` or `RawNote`).
//! - **Semantic rules are explicit**: Rules (like line break handling) are
//!   passed as configuration data, not hidden in branching code.
//! - **Memory efficiency**: No large AST allocations. Uses zero-cost iterator
//!   adapters internally and only clones when extracting reference definitions.
//!
//! # Examples
//!
//! ```ignore
//! // Conceptual usage of the parsing pipeline (Adapter Layer)
//! use lithos_core::note::parser::{
//!     config::EventStreamConfig,
//!     stream::MarkdownEventStream
//! };
//!
//! let source = "# Hello\n[link]: /url";
//! let config = EventStreamConfig::default();
//! let stream = MarkdownEventStream::new(source, config);
//!
//! // Extract reference definitions cleanly:
//! assert_eq!(stream.references().resolve("link"), Some("/url"));
//!
//! // Iterate over normalized events:
//! for event in stream {
//!     // ...
//! }
//! ```
//!
//! The current entry point is [`MarkdownParser`], which composes parser,
//! structure, and scanning responsibilities while this module evolves.

#![expect(
    clippy::pattern_type_mismatch,
    reason = "Parser pipeline matches borrowed event payloads intentionally"
)]

/// Block-domain types for parser AST.
pub(crate) mod block;
/// Configuration types for the event stream.
pub(crate) mod config;
/// Cached parsing context for markdown documents.
pub(crate) mod context;
/// Extracted reference link definitions.
pub(crate) mod references;
/// Event stream processing and normalization.
pub(crate) mod stream;
/// Block structure and AST types for markdown documents.
pub(crate) mod structure;
/// Derived inline text projection types.
pub(crate) mod text;
/// Parser-owned neutral event and payload types.
pub(crate) mod types;

#[cfg(test)]
#[path = "context_integration_test.rs"]
mod context_integration_test;

use std::borrow::Cow;

use pulldown_cmark::Options;
use text::{TextContext, TextSequence};
use types::{
    BlockEnd, BlockStart, FrontmatterFormat, InlineDelimiterEnd,
    InlineDelimiterStart, InlineToken, LinkKind, ParserEvent, RangedEvent,
};

use crate::{
    config::task::TaskConfigSpec,
    note::{
        error::{NoteIngestError, NoteParseError},
        extractor::BlockExtractor,
        position::{SourceByteOffset, SourceByteRange},
        raw::{RawLink, RawLinkStyle, RawListDepth, RawListKind, RawNote},
        scanner::NoteScanner,
    },
};

// ── Primary public API ───────────────────────────────────────────────────────

/// Markdown parser for extracting note facts and structure.
#[expect(
    private_bounds,
    reason = "ArtifactSink is internal while parser facade stays public"
)]
pub struct MarkdownParser<'source, S>
where
    S: ArtifactSink<'source>,
{
    // Source and configuration
    ref_defs: references::ReferenceDefinitions,

    // PDA state
    stack: BlockStack<'source>,
    depth: u32,
    link: Option<ActiveLink<'source>>,
    list_kinds: Vec<RawListKind>,
    open_items: Vec<usize>,

    // Components
    sink: S,
}

#[expect(
    private_bounds,
    reason = "Generic sink plumbing is internal implementation detail"
)]
impl<'source, S> MarkdownParser<'source, S>
where
    S: ArtifactSink<'source>,
{
    /// Parses markdown into a minimal AST and extracts raw note artifacts.
    #[inline]
    pub(crate) fn parse_with_sink(
        source: &'source str,
        task_spec: &TaskConfigSpec,
        sink: S,
    ) -> Result<S, NoteIngestError> {
        let stream_config = config::EventStreamConfig::default();
        let (stream, ref_defs) =
            stream::MarkdownEventStream::new(source, stream_config);

        let mut parser = Self::new(source, task_spec, ref_defs, sink);

        for event in stream {
            let event = event?;
            parser.step_spanned(&event)?;
        }

        Ok(parser.sink)
    }

    fn step_spanned(
        &mut self,
        event: &RangedEvent<'source>,
    ) -> Result<(), NoteIngestError> {
        let range = event.range();
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Pattern matching borrowed parser event variants"
        )]
        match event.event() {
            ParserEvent::BlockStart(block) => {
                self.on_block_start(block, range.start().as_usize())?;
            }
            ParserEvent::BlockEnd(block) => {
                self.on_block_end(*block, range.end().as_usize())?;
            }
            ParserEvent::Inline(inline) => {
                self.on_inline_event(inline, range)?;
            }
            ParserEvent::TaskListMarker(checked) => {
                self.on_task_marker(*checked);
            }
            ParserEvent::ThematicBreak => {
                self.sink.on_leaf_complete(
                    LeafKind::ThematicBreak,
                    BlockSpan {
                        start: range.start().as_usize(),
                        end: range.end().as_usize(),
                    },
                    &[],
                    self.depth,
                )?;
            }
        }
        Ok(())
    }

    fn on_inline_event(
        &mut self,
        inline: &InlineToken<'source>,
        range: SourceByteRange,
    ) -> Result<(), NoteIngestError> {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Pattern matching borrowed inline parser variants"
        )]
        match inline {
            InlineToken::DelimiterStart(InlineDelimiterStart::Link {
                kind,
                destination,
                ..
            }) => {
                self.open_link(
                    *kind,
                    destination.clone(),
                    false,
                    range.start().as_usize(),
                )?;
            }
            InlineToken::DelimiterStart(InlineDelimiterStart::Image {
                kind,
                destination,
                ..
            }) => {
                let is_embed = matches!(kind, LinkKind::WikiLink { .. });
                self.open_link(
                    *kind,
                    destination.clone(),
                    is_embed,
                    range.start().as_usize(),
                )?;
            }
            InlineToken::DelimiterEnd(
                InlineDelimiterEnd::Link | InlineDelimiterEnd::Image,
            ) => {
                self.finalize_link();
            }
            InlineToken::Text(_)
            | InlineToken::InlineCode(_)
            | InlineToken::DelimiterStart(_)
            | InlineToken::DelimiterEnd(_)
            | InlineToken::Html(_)
            | InlineToken::LineBreak(_)
            | InlineToken::Math {
                ..
            }
            | InlineToken::FootnoteReference(_) => {}
        }

        self.record_link_event(inline, range);
        self.record_inline_event(inline, range);

        Ok(())
    }

    /// Returns the pulldown-cmark option set used for Obsidian-compatible
    /// parsing.
    #[inline]
    #[must_use]
    pub fn extension_options() -> Options {
        config::EventStreamConfig::default_options()
    }

    fn new(
        _source: &'source str,
        _task_spec: &TaskConfigSpec,
        ref_defs: references::ReferenceDefinitions,
        sink: S,
    ) -> Self {
        Self {
            ref_defs,
            stack: BlockStack::new(4),
            depth: 0,
            link: None,
            list_kinds: Vec::with_capacity(4),
            open_items: Vec::with_capacity(8),
            sink,
        }
    }

    fn on_block_start(
        &mut self,
        block: &BlockStart<'source>,
        start: usize,
    ) -> Result<(), NoteIngestError> {
        match block {
            BlockStart::Frontmatter {
                format,
            } => {
                self.stack.push_leaf(
                    LeafKind::Metadata(MetadataPayload {
                        format: *format,
                    }),
                    start,
                );
            }
            BlockStart::Heading {
                level,
            } => {
                self.stack.push_leaf(
                    LeafKind::Heading(HeadingPayload {
                        level: *level,
                    }),
                    start,
                );
            }
            BlockStart::Paragraph => {
                self.stack.push_leaf(LeafKind::Paragraph, start);
            }
            BlockStart::List {
                kind,
            } => {
                let raw_kind = match kind {
                    types::ListKind::Ordered(n) => RawListKind::Ordered(*n),
                    types::ListKind::Unordered => RawListKind::Unordered,
                };
                self.list_kinds.push(raw_kind);
                self.depth = self.depth.saturating_add(1);
                self.stack.push_container(ContainerKind::List, start);
            }
            BlockStart::ListItem => self.on_list_item_start(start)?,
            BlockStart::BlockQuote => {
                self.depth = self.depth.saturating_add(1);
                self.stack.push_container(ContainerKind::BlockQuote, start);
            }
            BlockStart::CodeBlock {
                ..
            } => {
                self.stack.push_container(ContainerKind::CodeBlock, start);
            }
        }
        Ok(())
    }

    fn on_list_item_start(
        &mut self,
        start: usize,
    ) -> Result<(), NoteIngestError> {
        let list_kind =
            self.list_kinds.last().copied().unwrap_or(RawListKind::Unordered);
        let list_depth = RawListDepth::from(self.depth.saturating_sub(1));
        let depth_index =
            usize::try_from(self.depth).unwrap_or(0).saturating_sub(1);
        let parent_pos = if self.depth <= 1 {
            None
        } else {
            self.open_items
                .get(depth_index.saturating_sub(1))
                .copied()
                .map(SourceByteOffset::try_from_usize)
                .transpose()?
        };
        self.record_open_item(start, depth_index);
        self.stack.push_container(
            ContainerKind::ListItem(ListItemPayload {
                kind: list_kind,
                depth: list_depth,
                parent_pos,
                is_checkbox: None,
            }),
            start,
        );
        Ok(())
    }

    fn on_block_end(
        &mut self,
        block: BlockEnd,
        byte_end: usize,
    ) -> Result<(), NoteIngestError> {
        match block {
            BlockEnd::Frontmatter => {
                self.finalize_leaf_frame(byte_end, 0)?;
            }
            BlockEnd::Heading => {
                self.finalize_leaf_frame(byte_end, self.depth)?;
            }
            BlockEnd::ListItem | BlockEnd::CodeBlock => {
                self.finalize_container_frame(byte_end, self.depth)?;
            }
            BlockEnd::Paragraph => {
                let mut frame = self.stack.pop()?;
                if let BlockFrame::Leaf {
                    kind,
                    mut span,
                    ref mut events,
                } = frame
                {
                    span.end = byte_end;
                    // Propagate paragraph events to parent container if
                    // applicable.
                    if let Some(BlockFrame::Container {
                        events: parent_events,
                        ..
                    }) = self.stack.last_mut()
                    {
                        parent_events.extend(events.iter().cloned());
                    }
                    self.sink
                        .on_leaf_complete(kind, span, events, self.depth)?;
                }
            }

            BlockEnd::List => {
                self.depth = self.depth.saturating_sub(1);
                self.list_kinds.pop();
                self.finalize_container_frame(byte_end, self.depth)?;
            }
            BlockEnd::BlockQuote => {
                self.depth = self.depth.saturating_sub(1);
                self.finalize_container_frame(byte_end, self.depth)?;
            }
        }
        Ok(())
    }

    fn finalize_leaf_frame(
        &mut self,
        byte_end: usize,
        depth: u32,
    ) -> Result<(), NoteIngestError> {
        let frame = self.stack.pop()?;
        match frame {
            BlockFrame::Leaf {
                kind,
                mut span,
                events,
            } => {
                span.end = byte_end;
                self.sink.on_leaf_complete(kind, span, &events, depth)?;
            }
            BlockFrame::Container {
                span,
                ..
            } => {
                return Err(frame_role_mismatch_error(
                    "leaf",
                    "container",
                    span,
                    byte_end,
                    self.stack.frames.len(),
                ));
            }
        }
        Ok(())
    }

    fn finalize_container_frame(
        &mut self,
        byte_end: usize,
        depth: u32,
    ) -> Result<(), NoteIngestError> {
        let frame = self.stack.pop()?;
        match frame {
            BlockFrame::Container {
                kind,
                mut span,
                events,
            } => {
                span.end = byte_end;
                self.sink.on_container_complete(kind, span, &events, depth)?;
            }
            BlockFrame::Leaf {
                span,
                ..
            } => {
                return Err(frame_role_mismatch_error(
                    "container",
                    "leaf",
                    span,
                    byte_end,
                    self.stack.frames.len(),
                ));
            }
        }
        Ok(())
    }

    fn record_inline_event(
        &mut self,
        inline: &InlineToken<'source>,
        range: SourceByteRange,
    ) {
        if let Some(frame) = self.stack.last_mut() {
            let events = match frame {
                BlockFrame::Leaf {
                    events,
                    ..
                }
                | BlockFrame::Container {
                    events,
                    ..
                } => events,
            };
            events.push(RangedEvent::new(
                ParserEvent::Inline(inline.clone()),
                range,
            ));
        }
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Pattern ergonomics are used for mutable stack frame access"
    )]
    fn on_task_marker(&mut self, checked: bool) {
        if let Some(BlockFrame::Container {
            kind: ContainerKind::ListItem(payload),
            ..
        }) = self.stack.last_mut()
        {
            payload.is_checkbox = Some(checked);
        }
    }

    fn finalize_link(&mut self) {
        if let Some(mut active) = self.link.take() {
            active.raw.text.display = link_display_from_events(&active.events);
            self.sink.on_link(active.raw);
        }
    }

    fn record_link_event(
        &mut self,
        inline: &InlineToken<'source>,
        range: SourceByteRange,
    ) {
        if let Some(active) = self.link.as_mut() {
            active.events.push(RangedEvent::new(
                ParserEvent::Inline(inline.clone()),
                range,
            ));
        }
    }

    fn record_open_item(&mut self, start: usize, depth_index: usize) {
        if self.open_items.len() <= depth_index {
            self.open_items.resize(depth_index.saturating_add(1), 0);
        }
        if let Some(slot) = self.open_items.get_mut(depth_index) {
            *slot = start;
        }
        self.open_items.truncate(depth_index.saturating_add(1));
    }

    fn open_link(
        &mut self,
        link_type: LinkKind,
        dest_url: Cow<'source, str>,
        is_embed: bool,
        start: usize,
    ) -> Result<(), NoteIngestError> {
        let style = RawLinkStyle::from(link_type);
        let target =
            resolve_reference_target(link_type, dest_url, &self.ref_defs);
        self.link = Some(ActiveLink {
            raw: RawLink::new(
                style,
                is_embed,
                target,
                SourceByteOffset::try_from_usize(start)?,
            ),
            events: Vec::with_capacity(8),
        });
        Ok(())
    }
}

struct ActiveLink<'source> {
    raw: RawLink<'source>,
    events: Vec<RangedEvent<'source>>,
}

fn frame_role_mismatch_error(
    expected: &'static str,
    found: &'static str,
    opened: BlockSpan,
    closed_at: usize,
    depth: usize,
) -> NoteIngestError {
    let start_range =
        SourceByteRange::try_from(opened.start..opened.start).ok();
    let Ok(end_range) = SourceByteRange::try_from(closed_at..closed_at)
        .or_else(|_| SourceByteRange::try_from(opened.start..opened.start))
    else {
        return NoteParseError::InvalidTopology {
            code: "parser.stack.range_construction",
            detail: "failed to build range for frame role mismatch diagnostic"
                .into(),
            range: None,
        }
        .into();
    };
    NoteParseError::EventStackMismatch {
        expected,
        found,
        depth,
        start_range,
        end_range,
    }
    .into()
}

fn link_display_from_events(events: &[RangedEvent<'_>]) -> String {
    TextSequence::from_events(events)
        .nodes()
        .iter()
        .filter(|node| {
            matches!(
                node.context(),
                TextContext::LinkLabel | TextContext::Normal
            ) && node.is_displayable()
        })
        .map(text::TextNode::text)
        .collect()
}

impl<'source> MarkdownParser<'source, BlockExtractor<'source>> {
    /// Parses markdown into raw note artifacts.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if markdown parsing or source position
    /// mapping fails.
    #[inline]
    pub fn parse(
        source: &'source str,
        task_spec: &TaskConfigSpec,
    ) -> Result<RawNote<'source>, NoteIngestError> {
        let emoji_markers = if task_spec.use_emoji {
            task_spec.emoji_markers.clone()
        } else {
            Box::new([])
        };
        let scanner = NoteScanner::new(emoji_markers);
        let sink = BlockExtractor::new(source, scanner);
        Self::parse_with_sink(source, task_spec, sink)
            .map(BlockExtractor::finish)
    }
}

// ── PDA stack types ──────────────────────────────────────────────────────────

/// The parser's pushdown automaton sink.
pub(crate) trait ArtifactSink<'source> {
    fn on_container_complete(
        &mut self,
        kind: ContainerKind,
        span: BlockSpan,
        events: &[RangedEvent<'source>],
        depth: u32,
    ) -> Result<(), NoteIngestError>;

    fn on_leaf_complete(
        &mut self,
        kind: LeafKind,
        span: BlockSpan,
        events: &[RangedEvent<'source>],
        depth: u32,
    ) -> Result<(), NoteIngestError>;

    fn on_link(&mut self, link: RawLink<'source>);
}

/// Source span for a block element, capturing both start and end positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockSpan {
    pub start: usize,
    pub end: usize,
}

impl BlockSpan {
    /// Converts this span to a [`SourceByteRange`].
    pub(crate) fn to_source_range(
        self,
    ) -> Result<SourceByteRange, NoteIngestError> {
        SourceByteRange::try_from(self.start..self.end)
            .map_err(NoteIngestError::Domain)
    }
}

/// The parser's pushdown automaton stack.
pub(crate) struct BlockStack<'source> {
    frames: Vec<BlockFrame<'source>>,
}

impl<'source> BlockStack<'source> {
    fn new(frame_cap: usize) -> Self {
        Self {
            frames: Vec::with_capacity(frame_cap),
        }
    }

    fn push_leaf(&mut self, kind: LeafKind, start: usize) {
        self.frames.push(BlockFrame::Leaf {
            kind,
            span: BlockSpan {
                start,
                end: 0,
            },
            events: Vec::with_capacity(8),
        });
    }

    fn push_container(&mut self, kind: ContainerKind, start: usize) {
        self.frames.push(BlockFrame::Container {
            kind,
            span: BlockSpan {
                start,
                end: 0,
            },
            events: Vec::with_capacity(8),
        });
    }

    fn pop(&mut self) -> Result<BlockFrame<'source>, NoteIngestError> {
        self.frames.pop().ok_or_else(|| {
            NoteParseError::Markdown {
                line: 0,
                column: 0,
                reason: "block stack underflow: mismatched Start/End events"
                    .into(),
            }
            .into()
        })
    }

    fn last_mut(&mut self) -> Option<&mut BlockFrame<'source>> {
        self.frames.last_mut()
    }
}

/// A block frame on the parser's pushdown stack.
pub(crate) enum BlockFrame<'source> {
    Leaf {
        kind: LeafKind,
        span: BlockSpan,
        events: Vec<RangedEvent<'source>>,
    },
    Container {
        kind: ContainerKind,
        span: BlockSpan,
        events: Vec<RangedEvent<'source>>,
    },
}

/// Discriminant for [`LeafKind`] frames.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum LeafKind {
    Metadata(MetadataPayload),
    Heading(HeadingPayload),
    Paragraph,
    ThematicBreak,
}

/// Discriminant for [`ContainerKind`] frames.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ContainerKind {
    List,
    ListItem(ListItemPayload),
    BlockQuote,
    CodeBlock,
}

/// Payload for metadata blocks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MetadataPayload {
    pub format: FrontmatterFormat,
}

/// Payload for heading blocks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HeadingPayload {
    pub level: types::HeadingLevel,
}

impl HeadingPayload {
    pub(crate) fn to_u8(self) -> u8 {
        match self.level {
            types::HeadingLevel::H1 => 1,
            types::HeadingLevel::H2 => 2,
            types::HeadingLevel::H3 => 3,
            types::HeadingLevel::H4 => 4,
            types::HeadingLevel::H5 => 5,
            types::HeadingLevel::H6 => 6,
        }
    }
}

/// Payload for list item blocks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ListItemPayload {
    pub kind: RawListKind,
    pub depth: RawListDepth,
    pub parent_pos: Option<SourceByteOffset>,
    pub is_checkbox: Option<bool>,
}

// ── Private implementation details ───────────────────────────────────────────

fn resolve_reference_target<'source>(
    link_type: LinkKind,
    dest_url: Cow<'source, str>,
    ref_defs: &references::ReferenceDefinitions,
) -> Cow<'source, str> {
    if is_reference_link_type(link_type)
        && let Some(resolved) = ref_defs.resolve(dest_url.as_ref())
    {
        return Cow::Owned(String::from(resolved));
    }
    dest_url
}

#[inline]
fn is_reference_link_type(link_type: LinkKind) -> bool {
    matches!(
        link_type,
        LinkKind::Reference
            | LinkKind::ReferenceUnknown
            | LinkKind::Collapsed
            | LinkKind::CollapsedUnknown
            | LinkKind::Shortcut
            | LinkKind::ShortcutUnknown
    )
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    clippy::panic,
    reason = "Tests prioritize readability with assertions"
)]
mod tests {
    use super::*;
    use crate::{config::task::TaskConfigSpec, note::error::NoteError};

    #[derive(Default)]
    struct NoopSink;

    impl<'source> ArtifactSink<'source> for NoopSink {
        fn on_container_complete(
            &mut self,
            _kind: ContainerKind,
            _span: BlockSpan,
            _events: &[RangedEvent<'source>],
            _depth: u32,
        ) -> Result<(), NoteIngestError> {
            Ok(())
        }

        fn on_leaf_complete(
            &mut self,
            _kind: LeafKind,
            _span: BlockSpan,
            _events: &[RangedEvent<'source>],
            _depth: u32,
        ) -> Result<(), NoteIngestError> {
            Ok(())
        }

        fn on_link(&mut self, _link: RawLink<'source>) {}
    }

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

    fn parse_raw(markdown: &str) -> Result<RawNote<'_>, NoteIngestError> {
        let task_spec = task_spec_fixture();
        MarkdownParser::parse(markdown, &task_spec)
    }

    #[test]
    fn should_extract_block_ref_from_paragraph_tail()
    -> Result<(), NoteIngestError> {
        let md = "Paragraph text ^my-id";
        let raw = parse_raw(md)?;
        assert_eq!(raw.block_refs.len(), 1);
        assert_eq!(
            raw.block_refs
                .first()
                .ok_or(NoteError::Internal("missing block ref".into()))?
                .id,
            "my-id"
        );
        Ok(())
    }

    #[test]
    fn should_capture_yaml_at_start() -> Result<(), NoteIngestError> {
        let md = "---\ntags: [a]\n---\nContent";
        let raw = parse_raw(md)?;
        let fm = raw
            .frontmatter
            .as_ref()
            .ok_or(NoteError::Internal("frontmatter missing".into()))?;
        assert_eq!(fm.text, "tags: [a]\n");
        Ok(())
    }

    #[test]
    fn should_capture_tags_inside_heading() -> Result<(), NoteIngestError> {
        let md = "## Heading #tag";
        let raw = parse_raw(md)?;
        assert!(raw.tags.iter().any(|t| t.value == "#tag"));
        Ok(())
    }

    #[test]
    fn should_ignore_tags_inside_links() -> Result<(), NoteIngestError> {
        let md = "See [[target|#tag]] and [link #tag](http://example.test)";
        let raw = parse_raw(md)?;
        assert!(raw.tags.is_empty());
        Ok(())
    }

    #[test]
    fn should_ignore_block_refs_inside_links() -> Result<(), NoteIngestError> {
        let md = "See [link ^ref](http://example.test)";
        let raw = parse_raw(md)?;
        assert!(raw.block_refs.is_empty());
        Ok(())
    }

    #[test]
    fn should_detect_tag_after_link_label_gap() -> Result<(), NoteIngestError> {
        let md = "See [label](http://example.test) #tag";
        let raw = parse_raw(md)?;
        assert!(raw.tags.iter().any(|tag| tag.value == "#tag"));
        Ok(())
    }

    #[test]
    fn should_detect_block_ref_after_link_label_gap()
    -> Result<(), NoteIngestError> {
        let md = "See [label](http://example.test) ^my-id";
        let raw = parse_raw(md)?;
        assert!(raw.block_refs.iter().any(|block_ref| block_ref.id == "my-id"));
        Ok(())
    }

    #[test]
    fn should_not_scan_code_or_math_for_tags() -> Result<(), NoteIngestError> {
        let md = "`#hidden` $#hidden_math$ #visible";
        let raw = parse_raw(md)?;
        assert_eq!(raw.tags.len(), 1);
        assert_eq!(
            raw.tags.first().map(|tag| tag.value.as_ref()),
            Some("#visible")
        );
        Ok(())
    }

    #[test]
    fn finalize_leaf_frame_rejects_container_topology_mismatch() {
        let mut parser = MarkdownParser::new(
            "",
            &task_spec_fixture(),
            references::ReferenceDefinitions::new(
                std::collections::HashMap::new(),
            ),
            NoopSink,
        );
        parser.stack.push_container(ContainerKind::List, 0);

        let result = parser.finalize_leaf_frame(1, 0);
        assert!(matches!(
            result,
            Err(NoteIngestError::Parse(NoteParseError::EventStackMismatch {
                expected: "leaf",
                found: "container",
                ..
            }))
        ));
    }

    #[test]
    fn finalize_container_frame_rejects_leaf_topology_mismatch() {
        let mut parser = MarkdownParser::new(
            "",
            &task_spec_fixture(),
            references::ReferenceDefinitions::new(
                std::collections::HashMap::new(),
            ),
            NoopSink,
        );
        parser.stack.push_container(ContainerKind::List, 0);
        parser.stack.push_leaf(LeafKind::Paragraph, 0);

        let result = parser.finalize_container_frame(1, 0);
        assert!(matches!(
            result,
            Err(NoteIngestError::Parse(NoteParseError::EventStackMismatch {
                expected: "container",
                found: "leaf",
                depth: 1,
                ..
            }))
        ));
    }

    #[test]
    fn should_extract_bare_fields() -> Result<(), NoteIngestError> {
        let md = "bare_key:: bare_val";
        let raw = parse_raw(md)?;
        let field = raw
            .inline_fields
            .first()
            .ok_or(NoteError::Internal("field missing".into()))?;
        assert_eq!(field.key, "bare_key");
        assert_eq!(field.value, "bare_val");
        Ok(())
    }

    #[test]
    fn should_handle_wikilinks() -> Result<(), NoteIngestError> {
        let md = "Check [[target]] and [[target|alias]]";
        let raw = parse_raw(md)?;
        assert_eq!(raw.links.len(), 2);
        assert_eq!(
            raw.links
                .first()
                .ok_or(NoteError::Internal("link missing".into()))?
                .text
                .target
                .as_ref(),
            "target"
        );
        Ok(())
    }

    #[test]
    fn should_track_list_nesting() -> Result<(), NoteIngestError> {
        let md = "- Parent\n  - Child";
        let raw = parse_raw(md)?;
        assert_eq!(raw.list_items.len(), 2);
        let mut sorted = raw.list_items.clone();
        sorted.sort_by_key(|i| i.range.start().as_usize());
        let child = sorted
            .get(1)
            .ok_or(NoteError::Internal("Child list item missing".into()))?;
        assert!(matches!(child.depth, RawListDepth::Nested(1)));
        Ok(())
    }

    #[test]
    fn should_capture_checkbox_state_and_marker() -> Result<(), NoteIngestError>
    {
        let md = "- [x] Done";
        let raw = parse_raw(md)?;
        let item = raw
            .list_items
            .first()
            .ok_or(NoteError::Internal("list item missing".into()))?;
        assert_eq!(item.is_checkbox, Some(true));
        Ok(())
    }

    #[test]
    fn should_extract_thematic_break() -> Result<(), NoteIngestError> {
        let md = "Paragraph\n\n---\n\nAnother paragraph";
        let raw = parse_raw(md)?;
        assert_eq!(raw.sections.len(), 3);
        assert!(matches!(
            raw.sections.get(1).map(|s| s.kind),
            Some(crate::note::raw::RawSectionKind::ThematicBreak)
        ));
        Ok(())
    }

    #[test]
    fn should_report_event_stack_mismatch_on_finalization() {
        let mut parser = MarkdownParser::new(
            "",
            &task_spec_fixture(),
            references::ReferenceDefinitions::new(
                std::collections::HashMap::new(),
            ),
            NoopSink,
        );

        // Push a container but try to finalize a leaf
        parser.stack.push_container(ContainerKind::List, 0);
        let result = parser.finalize_leaf_frame(1, 0);

        match result {
            Err(NoteIngestError::Parse(
                NoteParseError::EventStackMismatch {
                    expected,
                    found,
                    ..
                },
            )) => {
                assert_eq!(expected, "leaf");
                assert_eq!(found, "container");
            }
            _ => panic!("Expected EventStackMismatch error, got {result:?}"),
        }
    }

    #[test]
    fn reference_definitions_first_wins() -> Result<(), NoteIngestError> {
        let md = "[ref]: http://a.example\n[ref]: http://b.example\n\n[ref][]";
        let raw = parse_raw(md)?;
        let link = raw
            .links
            .first()
            .ok_or(NoteError::Internal("link missing".into()))?;
        assert_eq!(link.text.target.as_ref(), "http://a.example");
        Ok(())
    }

    #[test]
    fn reference_definitions_are_case_insensitive()
    -> Result<(), NoteIngestError> {
        let md = "[Ref]: http://example.test\n\n[ref][]";
        let raw = parse_raw(md)?;
        let link = raw
            .links
            .first()
            .ok_or(NoteError::Internal("link missing".into()))?;
        assert_eq!(link.text.target.as_ref(), "http://example.test");
        Ok(())
    }

    #[test]
    fn reference_definitions_in_frontmatter_are_ignored()
    -> Result<(), NoteIngestError> {
        let md = "---\n[ref]: http://frontmatter.test\n---\n\n[ref][]";
        let raw = parse_raw(md)?;
        assert!(raw.links.is_empty());
        Ok(())
    }

    #[test]
    fn reference_definitions_in_fenced_code_are_ignored()
    -> Result<(), NoteIngestError> {
        let md = "```\n[ref]: http://code.test\n```\n\n[ref][]";
        let raw = parse_raw(md)?;
        assert!(raw.links.is_empty());
        Ok(())
    }

    #[test]
    fn reference_definitions_normalize_whitespace()
    -> Result<(), NoteIngestError> {
        let md = "[Foo   Bar]: http://example.test\n\n[foo bar][]";
        let raw = parse_raw(md)?;
        let link = raw
            .links
            .first()
            .ok_or(NoteError::Internal("link missing".into()))?;
        assert_eq!(link.text.target.as_ref(), "http://example.test");
        Ok(())
    }

    #[test]
    fn reference_definitions_unescape_labels() -> Result<(), NoteIngestError> {
        let md = "[Foo\\ Bar]: http://example.test\n\n[foo\\ bar][]";
        let raw = parse_raw(md)?;
        let link = raw
            .links
            .first()
            .ok_or(NoteError::Internal("link missing".into()))?;
        assert_eq!(link.text.target.as_ref(), "http://example.test");
        Ok(())
    }

    #[test]
    fn reference_definitions_allow_multiline_destination()
    -> Result<(), NoteIngestError> {
        let md = "[ref]:\n  http://example.test\n\n[ref][]";
        let raw = parse_raw(md)?;
        let link = raw
            .links
            .first()
            .ok_or(NoteError::Internal("link missing".into()))?;
        assert_eq!(link.text.target.as_ref(), "http://example.test");
        Ok(())
    }

    #[test]
    fn external_scheme_targets_preserve_fragments()
    -> Result<(), NoteIngestError> {
        let md = "[obsidian](obsidian://open?vault=V#frag)";
        let raw = parse_raw(md)?;
        let link = raw
            .links
            .first()
            .ok_or(NoteError::Internal("link missing".into()))?;
        assert_eq!(link.text.target.as_ref(), "obsidian://open?vault=V#frag");
        Ok(())
    }

    #[test]
    fn file_scheme_targets_preserve_fragments() -> Result<(), NoteIngestError> {
        let md = "[file](file:///Users/example/test.md#section)";
        let raw = parse_raw(md)?;
        let link = raw
            .links
            .first()
            .ok_or(NoteError::Internal("link missing".into()))?;
        assert_eq!(
            link.text.target.as_ref(),
            "file:///Users/example/test.md#section"
        );
        Ok(())
    }

    #[test]
    fn s3_scheme_targets_preserve_fragments() -> Result<(), NoteIngestError> {
        let md = "[s3](s3://bucket/key#object)";
        let raw = parse_raw(md)?;
        let link = raw
            .links
            .first()
            .ok_or(NoteError::Internal("link missing".into()))?;
        assert_eq!(link.text.target.as_ref(), "s3://bucket/key#object");
        Ok(())
    }
}
