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
        position::{SourceByteOffset, SourceByteRange},
        raw::{
            RawFrontmatter, RawFrontmatterFormat, RawLink, RawLinkStyle,
            RawList, RawListDepth, RawListKind, RawNote, RawSection,
            RawSectionKind,
        },
        scanner::NoteScanner,
    },
};

// ── Primary public API ───────────────────────────────────────────────────────

/// Markdown parser for extracting note facts and structure.
///
/// This is a stateful pushdown automaton. Callers use only the static
/// [`MarkdownParser::parse`] entry point; the struct itself is an
/// internal implementation detail.
pub struct MarkdownParser<'source, 'cfg> {
    // Source and configuration
    ref_defs: LinkRefResolver,
    pool: StringPool,

    // PDA state
    stack: Vec<Block<'source>>,
    depth: u32,
    link: Option<LinkFrame<'source>>,
    list_kinds: Vec<RawListKind>,
    list_ctxs: Vec<ListCtx>,
    open_items: Vec<SourceByteOffset>,

    // Components
    extractor: crate::note::extractor::BlockExtractor<'source, 'cfg>,
    out: RawNote<'source>,
}

impl<'source, 'cfg> MarkdownParser<'source, 'cfg> {
    /// Parses markdown into a minimal AST and extracts raw note artifacts.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if:
    /// - Structural extraction fails due to internal parser inconsistencies.
    /// - Metadata extraction (tags, fields) encounters invalid position
    ///   mapping.
    #[inline]
    pub fn parse(
        source: &'source str,
        task_spec: &'cfg TaskConfigSpec,
    ) -> Result<RawNote<'source>, NoteIngestError> {
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

        let mut parser = Self::new(source, task_spec, ref_defs);

        let normalized = offset_iter.map(|(ev, r)| (normalize_breaks(ev), r));
        for (event, range) in TextMergeWithOffset::new(normalized) {
            parser.step(event, range)?;
        }

        parser.out.sections.sort_by_key(|s| u32::from(s.range.start()));
        Ok(parser.out)
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
        source: &'source str,
        task_spec: &'cfg TaskConfigSpec,
        ref_defs: LinkRefResolver,
    ) -> Self {
        let emoji_markers = if task_spec.use_emoji {
            task_spec.emoji_markers.clone()
        } else {
            Box::new([])
        };
        let scanner = NoteScanner::new(emoji_markers);
        let extractor = crate::note::extractor::BlockExtractor::new(
            source, scanner, task_spec,
        );
        let mut pool = StringPool::new();
        let out = RawNote::new(
            None,
            Vec::with_capacity(4),
            Vec::with_capacity(8),
            Vec::with_capacity(8),
            Vec::with_capacity(8),
            Vec::with_capacity(4),
            Vec::with_capacity(8),
            Vec::with_capacity(8),
            Vec::with_capacity(8),
        );
        // Pre-warm the pool so early `take()` calls hit the fast path.
        for _ in 0u8..4u8 {
            pool.put(String::with_capacity(128));
        }
        Self {
            ref_defs,
            pool,
            stack: Vec::with_capacity(4),
            depth: 0,
            link: None,
            list_kinds: Vec::with_capacity(4),
            list_ctxs: Vec::with_capacity(4),
            open_items: Vec::with_capacity(8),
            extractor,
            out,
        }
    }

    fn step(
        &mut self,
        event: Event<'source>,
        range: std::ops::Range<usize>,
    ) -> Result<(), NoteIngestError> {
        match event {
            Event::Start(tag) => self.on_start(tag, range.start)?,
            Event::End(tag) => self.on_end(tag, range.end)?,
            Event::Text(text) => self.on_text(&text, range),
            Event::Code(text) => self.on_code(&text),
            Event::TaskListMarker(checked) => self.on_task_marker(checked),
            Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
            | Event::SoftBreak
            | Event::HardBreak
            | Event::Rule => {}
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Event handlers
    // -----------------------------------------------------------------------

    fn on_start(
        &mut self,
        tag: pulldown_cmark::Tag<'source>,
        byte_start: usize,
    ) -> Result<(), NoteIngestError> {
        let start = to_offset(byte_start)?;
        match tag {
            pulldown_cmark::Tag::MetadataBlock(kind) => {
                self.push_block(BlockKind::Metadata(kind), start);
            }
            pulldown_cmark::Tag::Heading {
                level,
                ..
            } => {
                self.push_block(BlockKind::Heading(level_to_u8(level)), start);
            }
            pulldown_cmark::Tag::Paragraph => {
                self.push_block(BlockKind::Paragraph, start);
            }
            pulldown_cmark::Tag::List(list_start) => {
                let kind = match list_start {
                    Some(n) => RawListKind::Ordered(n),
                    None => RawListKind::Unordered,
                };
                self.list_kinds.push(kind);
                self.list_ctxs.push(ListCtx {
                    item_positions: Vec::new(),
                });
                self.depth = self.depth.saturating_add(1);
                self.push_block(BlockKind::List, start);
            }
            pulldown_cmark::Tag::Item => {
                let list_kind = self
                    .list_kinds
                    .last()
                    .copied()
                    .unwrap_or(RawListKind::Unordered);
                let list_depth =
                    RawListDepth::from(self.depth.saturating_sub(1));
                let depth_index =
                    usize::try_from(self.depth).unwrap_or(0).saturating_sub(1);
                let parent_pos = if self.depth <= 1 {
                    None
                } else {
                    self.open_items.get(depth_index.saturating_sub(1)).copied()
                };
                self.record_open_item(start, depth_index);
                self.push_block(
                    BlockKind::ListItem {
                        list_kind,
                        list_depth,
                        parent_pos,
                    },
                    start,
                );
            }
            pulldown_cmark::Tag::BlockQuote(_) => {
                self.depth = self.depth.saturating_add(1);
                self.push_block(BlockKind::BlockQuote, start);
            }
            pulldown_cmark::Tag::CodeBlock(_) => {
                self.push_block(BlockKind::CodeBlock, start);
            }
            pulldown_cmark::Tag::Link {
                link_type,
                dest_url,
                ..
            } => {
                self.open_link(link_type, dest_url, false, start);
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
                self.open_link(link_type, dest_url, is_embed, start);
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
            | pulldown_cmark::Tag::Subscript => {}
        }
        Ok(())
    }

    fn on_end(
        &mut self,
        tag: TagEnd,
        byte_end: usize,
    ) -> Result<(), NoteIngestError> {
        match tag {
            TagEnd::Link | TagEnd::Image => self.finalize_link(),
            TagEnd::MetadataBlock(_) => self.finalize_metadata(byte_end)?,
            TagEnd::Heading(_) => {
                let block = pop_block(&mut self.stack)?;
                let range = block.range_to(byte_end)?;
                self.extractor.finalize_heading(
                    block,
                    range,
                    self.depth,
                    &mut self.out,
                    &mut self.pool,
                )?;
            }
            TagEnd::Paragraph => {
                let block = pop_block(&mut self.stack)?;
                // Propagate text to parent list item before moving block.
                if let Some(parent) = self.stack.last_mut()
                    && matches!(parent.kind, BlockKind::ListItem { .. })
                    && parent.text.is_empty()
                {
                    parent.text.push_str(block.text.trim());
                }
                let range = block.range_to(byte_end)?;
                self.extractor.finalize_paragraph(
                    block,
                    range,
                    self.depth,
                    &mut self.out,
                    &mut self.pool,
                )?;
            }
            TagEnd::Item => {
                let block = pop_block(&mut self.stack)?;
                let item_start = block.start;
                let range = block.range_to(byte_end)?;
                self.extractor.finalize_list_item(
                    block,
                    range,
                    &mut self.out,
                    &mut self.pool,
                )?;
                if let Some(ctx) = self.list_ctxs.last_mut() {
                    ctx.item_positions.push(item_start);
                }
            }
            TagEnd::List(_) => {
                let block = pop_block(&mut self.stack)?;
                let range = block.range_to(byte_end)?;
                self.depth = self.depth.saturating_sub(1);
                if let (Some(ctx), Some(kind)) =
                    (self.list_ctxs.pop(), self.list_kinds.pop())
                {
                    self.out.lists.push(RawList::new(
                        kind,
                        RawListDepth::from(self.depth),
                        range,
                        ctx.item_positions,
                    ));
                }
                self.pool.put(block.text);
            }
            TagEnd::BlockQuote(_) => {
                let block = pop_block(&mut self.stack)?;
                let range = block.range_to(byte_end)?;
                self.depth = self.depth.saturating_sub(1);
                self.out.sections.push(RawSection::new(
                    RawSectionKind::BlockQuote,
                    range,
                    self.depth,
                ));
                self.pool.put(block.text);
            }
            TagEnd::CodeBlock => {
                let block = pop_block(&mut self.stack)?;
                let range = block.range_to(byte_end)?;
                self.out.sections.push(RawSection::new(
                    RawSectionKind::CodeBlock,
                    range,
                    self.depth,
                ));
                self.pool.put(block.text);
            }
            TagEnd::HtmlBlock
            | TagEnd::FootnoteDefinition
            | TagEnd::DefinitionList
            | TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition
            | TagEnd::Table
            | TagEnd::TableHead
            | TagEnd::TableRow
            | TagEnd::TableCell
            | TagEnd::Emphasis
            | TagEnd::Strong
            | TagEnd::Strikethrough
            | TagEnd::Superscript
            | TagEnd::Subscript => {}
        }
        Ok(())
    }

    fn on_text(
        &mut self,
        text: &CowStr<'source>,
        range: std::ops::Range<usize>,
    ) {
        if let Some(block) = self.stack.last_mut() {
            block.text.push_str(text.as_ref());
            if self.link.is_none() {
                block.scannable.push(range);
            }
        }
        if let Some(link) = self.link.as_mut() {
            link.alias.push_str(text.as_ref());
        }
    }

    fn on_code(&mut self, text: &CowStr<'source>) {
        // Inline code contributes to full text but NOT to scannable ranges.
        if let Some(block) = self.stack.last_mut() {
            block.text.push_str(text.as_ref());
        }
        if let Some(link) = self.link.as_mut() {
            link.alias.push_str(text.as_ref());
        }
    }

    fn on_task_marker(&mut self, checked: bool) {
        if let Some(block) = self.stack.last_mut() {
            block.task_checked = Some(checked);
        }
    }

    fn finalize_link(&mut self) {
        if let Some(mut link) = self.link.take() {
            let (target_raw, anchor_raw) = LinkTarget::new(link.target).split();
            let alias_raw = trim_to_opt(&link.alias);
            self.out.links.push(RawLink::new(
                link.style,
                link.is_embed,
                target_raw,
                alias_raw.map(Into::into),
                anchor_raw,
                link.start,
            ));
            self.pool.put(std::mem::take(&mut link.alias));
        }
    }

    fn finalize_metadata(
        &mut self,
        byte_end: usize,
    ) -> Result<(), NoteIngestError> {
        let block = pop_block(&mut self.stack)?;
        let end = to_offset(byte_end)?;
        let block_range = SourceByteRange::new(block.start, end)
            .map_err(NoteIngestError::Domain)?;
        let BlockKind::Metadata(kind) = block.kind else {
            // Return text to pool if kind doesn't match (shouldn't happen).
            self.pool.put(block.text);
            return Ok(());
        };
        self.out.sections.push(RawSection::new(
            RawSectionKind::Frontmatter,
            block_range,
            0,
        ));
        // block.text moves into RawFrontmatter; does not return to pool.
        self.out.frontmatter = Some(RawFrontmatter::new(
            kind.into(),
            block.text.into(),
            block_range,
        ));
        Ok(())
    }

    // Private push helpers
    fn push_block(&mut self, kind: BlockKind, start: SourceByteOffset) {
        self.stack.push(Block {
            kind,
            start,
            text: self.pool.take(),
            scannable: Vec::with_capacity(4),
            task_checked: None,
            _marker: std::marker::PhantomData,
        });
    }

    fn record_open_item(
        &mut self,
        start: SourceByteOffset,
        depth_index: usize,
    ) {
        if self.open_items.len() <= depth_index {
            self.open_items.resize(
                depth_index.saturating_add(1),
                SourceByteOffset::new(0),
            );
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
        start: SourceByteOffset,
    ) {
        let target =
            resolve_reference_target(link_type, dest_url, &mut self.ref_defs);
        self.link = Some(LinkFrame {
            style: link_type.into(),
            is_embed,
            target,
            start,
            alias: self.pool.take(),
        });
    }
}

// ── String pool ──────────────────────────────────────────────────────────────

/// A pool of cleared `String` buffers reused across block frames.
///
/// Instead of allocating a fresh `String` for each block frame, the parser
/// returns used strings to this pool via [`put`](Self::put) and retrieves them
/// via [`take`](Self::take). This eliminates per-block heap allocation in the
/// common case once the pool has been pre-warmed.
pub(crate) struct StringPool {
    pool: Vec<String>,
}

impl StringPool {
    /// Creates a new pool with an initial backing capacity.
    pub(crate) fn new() -> Self {
        Self {
            pool: Vec::with_capacity(16),
        }
    }

    /// Removes and returns a cleared string from the pool.
    ///
    /// Returns a freshly allocated `String` with 128-byte capacity when the
    /// pool is empty.
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "metrics are user-controlled instrumentation"
    )]
    pub(crate) fn take(&mut self) -> String {
        STRING_POOL_METRICS.with(|cell| {
            let mut metrics = cell.borrow_mut();
            metrics.takes += 1;
            metrics.pool_size = self.pool.len();
            metrics.pool_capacity = self.pool.capacity();
        });
        self.pool.pop().unwrap_or_else(|| String::with_capacity(128))
    }

    /// Clears `s` and returns it to the pool for later reuse.
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "metrics are user-controlled instrumentation"
    )]
    pub(crate) fn put(&mut self, mut s: String) {
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

// ── String pool instrumentation ──────────────────────────────────────────────

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

// ── PDA stack types ──────────────────────────────────────────────────────────

/// A block frame on the parser's pushdown stack.
///
/// Each `Block` corresponds to one open markdown container or leaf element
/// (heading, paragraph, list item, etc.). Text and scannable byte ranges
/// accumulate here until the matching `End` event, at which point the block is
/// popped and passed to [`BlockExtractor`] for artifact extraction.
///
/// [`BlockExtractor`]: crate::note::extractor::BlockExtractor
pub(crate) struct Block<'source> {
    pub kind: BlockKind,
    pub start: SourceByteOffset,
    /// Pool-backed text accumulator.
    pub text: String,
    /// Non-link text ranges within the source, used for scanning.
    pub scannable: Vec<std::ops::Range<usize>>,
    pub task_checked: Option<bool>,
    pub _marker: std::marker::PhantomData<&'source str>,
}

impl Block<'_> {
    /// Computes the [`SourceByteRange`] from this block's start to `byte_end`.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if `byte_end` exceeds the supported offset
    /// range or if the resulting range is invalid.
    pub(crate) fn range_to(
        &self,
        byte_end: usize,
    ) -> Result<SourceByteRange, NoteIngestError> {
        let end = to_offset(byte_end)?;
        SourceByteRange::new(self.start, end).map_err(NoteIngestError::Domain)
    }
}

/// Discriminant for [`Block`] stack frames, carrying per-kind metadata.
///
/// Determines which finalisation path in
/// [`BlockExtractor`](crate::note::extractor::BlockExtractor) or the `on_end`
/// handler is taken when the matching `End` event is received.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BlockKind {
    /// YAML or TOML frontmatter block.
    Metadata(pulldown_cmark::MetadataBlockKind),
    /// ATX or setext heading at the given level (1–6).
    Heading(u8),
    /// Bare paragraph block.
    Paragraph,
    /// A single list item with its enclosing list context.
    ListItem {
        list_kind: RawListKind,
        list_depth: RawListDepth,
        /// Source position of the parent item, if nested.
        parent_pos: Option<SourceByteOffset>,
    },
    /// Ordered or unordered list container.
    List,
    /// Block quote container.
    BlockQuote,
    /// Fenced or indented code block.
    CodeBlock,
}

/// List context for the new [`MarkdownParser`] stateful struct.
///
/// Tracks item start positions for the current open list, used to populate
/// [`RawList::item_positions`] when the list is finalised.
pub(crate) struct ListCtx {
    pub item_positions: Vec<SourceByteOffset>,
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

/// Pops the top block from `stack`, returning an error on underflow.
///
/// Underflow indicates mismatched Start/End events, which pulldown-cmark
/// guarantees do not occur for well-formed input.
pub(crate) fn pop_block<'source>(
    stack: &mut Vec<Block<'source>>,
) -> Result<Block<'source>, NoteIngestError> {
    stack.pop().ok_or_else(|| {
        NoteParseError::Markdown {
            line: 0,
            column: 0,
            reason: "block stack underflow: mismatched Start/End events".into(),
        }
        .into()
    })
}

/// Converts a byte offset to [`SourceByteOffset`], mapping overflow to a
/// [`NoteIngestError`].
pub(crate) fn to_offset(
    byte: usize,
) -> Result<SourceByteOffset, NoteIngestError> {
    SourceByteOffset::try_from(byte).map_err(|_err| {
        #[expect(clippy::as_conversions, reason = "u32::MAX fits in usize")]
        NoteIngestError::Domain(
            crate::note::error::StructureError::OutOfBounds {
                offset: byte,
                source_len: u32::MAX as usize,
            }
            .into(),
        )
    })
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

/// Converts a pulldown-cmark metadata block kind to a [`RawFrontmatterFormat`].
impl From<pulldown_cmark::MetadataBlockKind> for RawFrontmatterFormat {
    #[inline]
    fn from(kind: pulldown_cmark::MetadataBlockKind) -> Self {
        match kind {
            pulldown_cmark::MetadataBlockKind::YamlStyle => Self::Yaml,
            pulldown_cmark::MetadataBlockKind::PlusesStyle => Self::Toml,
        }
    }
}

/// Converts a pulldown-cmark link type to a [`RawLinkStyle`].
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

fn level_to_u8(level: pulldown_cmark::HeadingLevel) -> u8 {
    match level {
        pulldown_cmark::HeadingLevel::H1 => 1,
        pulldown_cmark::HeadingLevel::H2 => 2,
        pulldown_cmark::HeadingLevel::H3 => 3,
        pulldown_cmark::HeadingLevel::H4 => 4,
        pulldown_cmark::HeadingLevel::H5 => 5,
        pulldown_cmark::HeadingLevel::H6 => 6,
    }
}

fn trim_to_opt(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
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
    ref_defs: &mut LinkRefResolver,
) -> Cow<'source, str> {
    if is_reference_link_type(link_type)
        && let Some(resolved) = ref_defs.resolve(dest_url.as_ref())
    {
        return Cow::Owned(String::from(resolved));
    }
    Cow::from(dest_url)
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
