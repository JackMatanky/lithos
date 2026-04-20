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
    ref_defs: LinkRefResolver,

    // PDA state
    stack: BlockStack<'source>,
    depth: u32,
    link: Option<RawLink<'source>>,
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
        let base = Parser::new_ext(source, Self::extension_options());
        let offset_iter = base.into_offset_iter();
        let ref_defs = LinkRefResolver::new(
            offset_iter
                .reference_definitions()
                .iter()
                .map(|(label, link_def)| {
                    (label.to_owned(), link_def.dest.to_string())
                })
                .collect(),
        );

        let mut parser = Self::new(source, task_spec, ref_defs, sink);

        let normalized = offset_iter.map(|(ev, r)| (normalize_breaks(ev), r));
        for (event, range) in TextMergeWithOffset::new(normalized) {
            parser.step(event, range)?;
        }

        Ok(parser.sink)
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

    fn new(
        _source: &'source str,
        _task_spec: &TaskConfigSpec,
        ref_defs: LinkRefResolver,
        sink: S,
    ) -> Self {
        Self {
            ref_defs,
            stack: BlockStack::new(4, 4),
            depth: 0,
            link: None,
            list_kinds: Vec::with_capacity(4),
            open_items: Vec::with_capacity(8),
            sink,
        }
    }

    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "Unhandled pulldown events are intentionally ignored"
    )]
    fn step(
        &mut self,
        event: Event<'source>,
        range: std::ops::Range<usize>,
    ) -> Result<(), NoteIngestError> {
        match event {
            Event::Start(tag) => self.on_start(tag, range.start)?,
            Event::End(tag) => self.on_end(tag, range.end)?,
            Event::Text(text) => self.on_text(&text, range)?,
            Event::Code(text) => self.on_code(&text),
            Event::TaskListMarker(checked) => self.on_task_marker(checked),
            _ => {}
        }
        Ok(())
    }

    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "Only tags relevant to note extraction are handled"
    )]
    fn on_start(
        &mut self,
        tag: pulldown_cmark::Tag<'source>,
        start: usize,
    ) -> Result<(), NoteIngestError> {
        match tag {
            pulldown_cmark::Tag::MetadataBlock(kind) => {
                self.stack.push_leaf(
                    LeafKind::Metadata(MetadataPayload {
                        kind,
                    }),
                    start,
                );
            }
            pulldown_cmark::Tag::Heading {
                level,
                ..
            } => {
                self.stack.push_leaf(
                    LeafKind::Heading(HeadingPayload {
                        level,
                    }),
                    start,
                );
            }
            pulldown_cmark::Tag::Paragraph => {
                self.stack.push_leaf(LeafKind::Paragraph, start);
            }
            pulldown_cmark::Tag::List(list_start) => {
                let kind = match list_start {
                    Some(n) => RawListKind::Ordered(n),
                    None => RawListKind::Unordered,
                };
                self.list_kinds.push(kind);
                self.depth = self.depth.saturating_add(1);
                self.stack.push_container(ContainerKind::List, start);
            }
            pulldown_cmark::Tag::Item => {
                self.on_list_item_start(start)?;
            }
            pulldown_cmark::Tag::BlockQuote(_) => {
                self.depth = self.depth.saturating_add(1);
                self.stack.push_container(ContainerKind::BlockQuote, start);
            }
            pulldown_cmark::Tag::CodeBlock(_) => {
                self.stack.push_container(ContainerKind::CodeBlock, start);
            }
            pulldown_cmark::Tag::Link {
                link_type,
                dest_url,
                ..
            } => {
                self.open_link(link_type, dest_url, false, start)?;
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
                self.open_link(link_type, dest_url, is_embed, start)?;
            }
            _ => {}
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
        self.stack.push_leaf(
            LeafKind::ListItem(ListItemPayload {
                kind: list_kind,
                depth: list_depth,
                parent_pos,
                is_checkbox: None,
            }),
            start,
        );
        Ok(())
    }

    #[expect(
        clippy::match_same_arms,
        reason = "Symmetric frame finalization keeps tag handling explicit"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "End-tag handling keeps parser state transitions co-located"
    )]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "Unsupported markdown end-tags are intentionally ignored"
    )]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Pattern ergonomics are used for mutable stack frame access"
    )]
    fn on_end(
        &mut self,
        tag: TagEnd,
        byte_end: usize,
    ) -> Result<(), NoteIngestError> {
        match tag {
            TagEnd::Link | TagEnd::Image => self.finalize_link(),
            TagEnd::MetadataBlock(_) => {
                let frame = self.stack.pop()?;
                if let BlockFrame::Leaf {
                    kind,
                    mut span,
                    fragments,
                } = frame
                {
                    span.end = byte_end;
                    self.sink.on_leaf_complete(kind, span, &fragments, 0)?;
                    self.stack.pool_mut().put(fragments);
                }
            }
            TagEnd::Heading(_) => {
                let frame = self.stack.pop()?;
                if let BlockFrame::Leaf {
                    kind,
                    mut span,
                    fragments,
                } = frame
                {
                    span.end = byte_end;
                    self.sink
                        .on_leaf_complete(kind, span, &fragments, self.depth)?;
                    self.stack.pool_mut().put(fragments);
                }
            }
            TagEnd::Paragraph => {
                let mut frame = self.stack.pop()?;
                if let BlockFrame::Leaf {
                    kind,
                    mut span,
                    ref mut fragments,
                } = frame
                {
                    span.end = byte_end;
                    // Propagate fragments to parent list item if applicable.
                    if let Some(BlockFrame::Leaf {
                        kind: LeafKind::ListItem(..),
                        fragments: parent_frags,
                        ..
                    }) = self.stack.last_mut()
                        && parent_frags.is_empty()
                    {
                        parent_frags.extend(fragments.iter().cloned());
                    }
                    self.sink
                        .on_leaf_complete(kind, span, fragments, self.depth)?;
                }
                if let BlockFrame::Leaf {
                    fragments,
                    ..
                } = frame
                {
                    self.stack.pool_mut().put(fragments);
                }
            }

            TagEnd::Item => {
                let frame = self.stack.pop()?;
                if let BlockFrame::Leaf {
                    kind,
                    mut span,
                    fragments,
                } = frame
                {
                    span.end = byte_end;
                    self.sink
                        .on_leaf_complete(kind, span, &fragments, self.depth)?;
                    self.stack.pool_mut().put(fragments);
                }
            }
            TagEnd::List(_) => {
                let frame = self.stack.pop()?;
                if let BlockFrame::Container {
                    kind,
                    mut span,
                } = frame
                {
                    span.end = byte_end;
                    self.depth = self.depth.saturating_sub(1);
                    self.list_kinds.pop();
                    self.sink.on_container_complete(kind, span, self.depth)?;
                }
            }
            TagEnd::BlockQuote(_) => {
                let frame = self.stack.pop()?;
                if let BlockFrame::Container {
                    kind,
                    mut span,
                } = frame
                {
                    span.end = byte_end;
                    self.depth = self.depth.saturating_sub(1);
                    self.sink.on_container_complete(kind, span, self.depth)?;
                }
            }
            TagEnd::CodeBlock => {
                let frame = self.stack.pop()?;
                if let BlockFrame::Container {
                    kind,
                    mut span,
                } = frame
                {
                    span.end = byte_end;
                    self.sink.on_container_complete(kind, span, self.depth)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Pattern ergonomics are used for mutable stack frame access"
    )]
    fn on_text(
        &mut self,
        text: &CowStr<'source>,
        range: std::ops::Range<usize>,
    ) -> Result<(), NoteIngestError> {
        if let Some(BlockFrame::Leaf {
            fragments,
            ..
        }) = self.stack.last_mut()
        {
            let source_range = SourceByteRange::try_from(range)
                .map_err(NoteIngestError::Domain)?;
            fragments.push(TextFragment {
                text: Cow::from(text.clone()),
                range: source_range,
                is_scannable: self.link.is_none(),
            });
        }
        if let Some(ref mut raw) = self.link {
            raw.text.display.push_str(text.as_ref());
        }
        Ok(())
    }

    fn on_code(&mut self, text: &CowStr<'source>) {
        if let Some(ref mut raw) = self.link {
            raw.text.display.push_str(text.as_ref());
        }
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Pattern ergonomics are used for mutable stack frame access"
    )]
    fn on_task_marker(&mut self, checked: bool) {
        if let Some(BlockFrame::Leaf {
            kind: LeafKind::ListItem(payload),
            ..
        }) = self.stack.last_mut()
        {
            payload.is_checkbox = Some(checked);
        }
    }

    fn finalize_link(&mut self) {
        if let Some(raw) = self.link.take() {
            self.sink.on_link(raw);
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
        link_type: pulldown_cmark::LinkType,
        dest_url: CowStr<'source>,
        is_embed: bool,
        start: usize,
    ) -> Result<(), NoteIngestError> {
        let style = RawLinkStyle::from(link_type);
        let target =
            resolve_reference_target(link_type, dest_url, &mut self.ref_defs);
        self.link = Some(RawLink::new(
            style,
            is_embed,
            target,
            SourceByteOffset::try_from_usize(start)?,
        ));
        Ok(())
    }
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
        depth: u32,
    ) -> Result<(), NoteIngestError>;

    fn on_leaf_complete(
        &mut self,
        kind: LeafKind,
        span: BlockSpan,
        fragments: &[TextFragment<'source>],
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

/// A text fragment with its source position.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TextFragment<'source> {
    pub text: Cow<'source, str>,
    pub range: SourceByteRange,
    pub is_scannable: bool,
}

/// The parser's pushdown automaton stack, bundled with its backing fragment
/// pool.
pub(crate) struct BlockStack<'source> {
    frames: Vec<BlockFrame<'source>>,
    pool: FragmentPool<'source>,
}

impl<'source> BlockStack<'source> {
    fn new(frame_cap: usize, prewarm: u8) -> Self {
        let mut pool = FragmentPool::new();
        for _ in 0..prewarm {
            pool.put(Vec::with_capacity(4));
        }
        Self {
            frames: Vec::with_capacity(frame_cap),
            pool,
        }
    }

    fn push_leaf(&mut self, kind: LeafKind, start: usize) {
        self.frames.push(BlockFrame::Leaf {
            kind,
            span: BlockSpan {
                start,
                end: 0,
            },
            fragments: self.pool.take(),
        });
    }

    fn push_container(&mut self, kind: ContainerKind, start: usize) {
        self.frames.push(BlockFrame::Container {
            kind,
            span: BlockSpan {
                start,
                end: 0,
            },
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

    fn pool_mut(&mut self) -> &mut FragmentPool<'source> {
        &mut self.pool
    }
}

/// A block frame on the parser's pushdown stack.
pub(crate) enum BlockFrame<'source> {
    Leaf {
        kind: LeafKind,
        span: BlockSpan,
        fragments: Vec<TextFragment<'source>>,
    },
    Container {
        kind: ContainerKind,
        span: BlockSpan,
    },
}

/// Discriminant for [`LeafKind`] frames.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum LeafKind {
    Metadata(MetadataPayload),
    Heading(HeadingPayload),
    Paragraph,
    ListItem(ListItemPayload),
}

/// Discriminant for [`ContainerKind`] frames.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ContainerKind {
    List,
    BlockQuote,
    CodeBlock,
}

/// Payload for metadata blocks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MetadataPayload {
    pub kind: pulldown_cmark::MetadataBlockKind,
}

/// Payload for heading blocks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HeadingPayload {
    pub level: pulldown_cmark::HeadingLevel,
}

impl HeadingPayload {
    pub(crate) fn to_u8(self) -> u8 {
        match self.level {
            pulldown_cmark::HeadingLevel::H1 => 1,
            pulldown_cmark::HeadingLevel::H2 => 2,
            pulldown_cmark::HeadingLevel::H3 => 3,
            pulldown_cmark::HeadingLevel::H4 => 4,
            pulldown_cmark::HeadingLevel::H5 => 5,
            pulldown_cmark::HeadingLevel::H6 => 6,
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

// ── Fragment pool ────────────────────────────────────────────────────────────

/// A pool of cleared fragment buffers reused across block frames.
pub(crate) struct FragmentPool<'source> {
    pool: Vec<Vec<TextFragment<'source>>>,
}

impl<'source> FragmentPool<'source> {
    /// Creates a new pool with an initial backing capacity.
    pub(crate) fn new() -> Self {
        Self {
            pool: Vec::with_capacity(16),
        }
    }

    /// Removes and returns a cleared fragment buffer from the pool.
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "metrics are user-controlled instrumentation"
    )]
    pub(crate) fn take(&mut self) -> Vec<TextFragment<'source>> {
        FRAGMENT_POOL_METRICS.with(|cell| {
            let mut metrics = cell.borrow_mut();
            metrics.takes += 1;
            metrics.pool_size = self.pool.len();
            metrics.pool_capacity = self.pool.capacity();
        });
        self.pool.pop().unwrap_or_else(|| Vec::with_capacity(4))
    }

    /// Clears `fragments` and returns it to the pool for later reuse.
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "metrics are user-controlled instrumentation"
    )]
    pub(crate) fn put(&mut self, mut fragments: Vec<TextFragment<'source>>) {
        fragments.clear();
        self.pool.push(fragments);
        FRAGMENT_POOL_METRICS.with(|cell| {
            let mut metrics = cell.borrow_mut();
            metrics.puts += 1;
            metrics.pool_size = self.pool.len();
            metrics.pool_capacity = self.pool.capacity();
        });
    }
}

// ── Fragment pool instrumentation ────────────────────────────────────────────

/// Metrics collected during `FragmentPool` operations for benchmarking.
#[derive(Default, Debug, Clone, Copy)]
#[non_exhaustive]
pub struct FragmentPoolMetrics {
    /// Number of times `take()` was called (buffers requested from pool).
    pub takes: usize,
    /// Number of times `put()` was called (buffers returned to pool).
    pub puts: usize,
    /// Current number of buffers held in the pool.
    pub pool_size: usize,
    /// Current capacity of the pool's internal vector.
    pub pool_capacity: usize,
}

thread_local! {
    static FRAGMENT_POOL_METRICS: std::cell::RefCell<FragmentPoolMetrics> =
        std::cell::RefCell::new(FragmentPoolMetrics::default());
}

/// Retrieve the current `FragmentPool` metrics.
#[inline]
#[must_use]
pub fn get_fragment_pool_metrics() -> FragmentPoolMetrics {
    FRAGMENT_POOL_METRICS.with(|cell| *cell.borrow())
}

/// Reset `FragmentPool` metrics to zero.
#[inline]
pub fn reset_fragment_pool_metrics() {
    FRAGMENT_POOL_METRICS
        .with(|cell| *cell.borrow_mut() = FragmentPoolMetrics::default());
}

// ── Shared pub(crate) free functions ─────────────────────────────────────────

/// Maps `SoftBreak`/`HardBreak` to `Text` events before merging.
pub(crate) fn normalize_breaks(event: Event<'_>) -> Event<'_> {
    match event {
        Event::SoftBreak => Event::Text(CowStr::Borrowed(" ")),
        Event::HardBreak => Event::Text(CowStr::Borrowed("\n")),
        other @ (Event::Start(_)
        | Event::End(_)
        | Event::Text(_)
        | Event::Code(_)
        | Event::InlineMath(_)
        | Event::DisplayMath(_)
        | Event::Html(_)
        | Event::InlineHtml(_)
        | Event::FootnoteReference(_)
        | Event::Rule
        | Event::TaskListMarker(_)) => other,
    }
}

// ── Private implementation details ───────────────────────────────────────────

struct LinkRefResolver {
    normalized: std::collections::HashMap<Box<str>, Box<str>>,
}

impl LinkRefResolver {
    #[expect(
        clippy::iter_over_hash_type,
        reason = "Normalization chooses first-seen label order"
    )]
    fn new(raw: std::collections::HashMap<String, String>) -> Self {
        let mut normalized = std::collections::HashMap::new();
        for (label, dest) in raw {
            let key = Self::normalize_label(&label);
            normalized.entry(key).or_insert(dest.into_boxed_str());
        }
        Self {
            normalized,
        }
    }

    fn resolve(&self, label: &str) -> Option<&str> {
        let normalized = Self::normalize_label(label);
        self.normalized.get(normalized.as_ref()).map(AsRef::as_ref)
    }

    fn normalize_label(label: &str) -> Box<str> {
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
}

fn resolve_reference_target<'source>(
    link_type: pulldown_cmark::LinkType,
    dest_url: CowStr<'source>,
    ref_defs: &mut LinkRefResolver,
) -> Cow<'source, str> {
    if is_reference_link_type(link_type)
        && let Some(resolved) = ref_defs.resolve(dest_url.as_ref())
    {
        return Cow::Owned(String::from(resolved));
    }
    Cow::from(dest_url)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::task::TaskConfigSpec;

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
        let task_spec = task_spec_fixture();
        MarkdownParser::parse(markdown, &task_spec).expect("parsing failed")
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
        assert_eq!(field.value, "bare_val");
    }

    #[test]
    fn should_handle_wikilinks() {
        let md = "Check [[target]] and [[target|alias]]";
        let raw = parse_raw(md);
        assert_eq!(raw.links.len(), 2);
        assert_eq!(raw.links.first().unwrap().text.target.as_ref(), "target");
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
        assert_eq!(item.is_checkbox, Some(true));
    }

    #[test]
    fn reference_definitions_first_wins() {
        let md = "[ref]: http://a.example\n[ref]: http://b.example\n\n[ref][]";
        let raw = parse_raw(md);
        let link = raw.links.first().expect("link exists");
        assert_eq!(link.text.target.as_ref(), "http://a.example");
    }

    #[test]
    fn reference_definitions_are_case_insensitive() {
        let md = "[Ref]: http://example.test\n\n[ref][]";
        let raw = parse_raw(md);
        let link = raw.links.first().expect("link exists");
        assert_eq!(link.text.target.as_ref(), "http://example.test");
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
        assert_eq!(link.text.target.as_ref(), "http://example.test");
    }

    #[test]
    fn reference_definitions_unescape_labels() {
        let md = "[Foo\\ Bar]: http://example.test\n\n[foo\\ bar][]";
        let raw = parse_raw(md);
        let link = raw.links.first().expect("link exists");
        assert_eq!(link.text.target.as_ref(), "http://example.test");
    }

    #[test]
    fn reference_definitions_allow_multiline_destination() {
        let md = "[ref]:\n  http://example.test\n\n[ref][]";
        let raw = parse_raw(md);
        let link = raw.links.first().expect("link exists");
        assert_eq!(link.text.target.as_ref(), "http://example.test");
    }

    #[test]
    fn external_scheme_targets_preserve_fragments() {
        let md = "[obsidian](obsidian://open?vault=V#frag)";
        let raw = parse_raw(md);
        let link = raw.links.first().expect("link exists");
        assert_eq!(link.text.target.as_ref(), "obsidian://open?vault=V#frag");
    }

    #[test]
    fn file_scheme_targets_preserve_fragments() {
        let md = "[file](file:///Users/example/test.md#section)";
        let raw = parse_raw(md);
        let link = raw.links.first().expect("link exists");
        assert_eq!(
            link.text.target.as_ref(),
            "file:///Users/example/test.md#section"
        );
    }

    #[test]
    fn s3_scheme_targets_preserve_fragments() {
        let md = "[s3](s3://bucket/key#object)";
        let raw = parse_raw(md);
        let link = raw.links.first().expect("link exists");
        assert_eq!(link.text.target.as_ref(), "s3://bucket/key#object");
    }
}
