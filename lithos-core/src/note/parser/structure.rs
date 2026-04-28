//! Block structure and AST types for markdown documents.
//!
//! This module provides the core AST data structures that represent the
//! hierarchical structure of a parsed markdown document. The AST follows
//! CommonMark semantics, distinguishing between **leaf blocks**
//! (content-bearing) and **container blocks** (structure-bearing).
//!
//! # Design Philosophy
//!
//! - **Minimal AST**: Only structure and content, no metadata extraction
//! - **CommonMark-aligned**: Block types map directly to CommonMark spec
//! - **Zero-copy where possible**: Events borrow from source via lifetimes
//! - **Explicit nesting**: Container blocks have `children` fields
//!
//! # Examples
//!
//! ```rust,ignore
//! use lithos_core::note::parser::{ParserContext, structure::Block};
//!
//! let source = "# Heading\n\nParagraph text";
//! let ctx = ParserContext::new(source, config)?;
//!
//! // (DocStructure building shown in Phase 3)
//! ```

use pulldown_cmark::{CowStr, Event, MetadataBlockKind, Tag, TagEnd};

use super::{context::ParserContext, stream::EventWithRange};
use crate::note::{
    error::{NoteIngestError, NoteParseError},
    position::SourceByteRange,
};

/// A complete block in the markdown document tree.
///
/// Each block represents a single structural element from the markdown source,
/// such as a paragraph, heading, list, or code block. Blocks form a tree
/// structure through the `children` fields in container block variants.
///
/// # Lifecycle
///
/// Blocks are created during AST building (Phase 3) by finalizing
/// [`ProcessingBlock`] instances. Once created, blocks are immutable and can be
/// safely borrowed across multiple pipeline stages.
///
/// # Examples
///
/// ```rust,ignore
/// // A simple paragraph block
/// let block = Block {
///     kind: BlockKind::Paragraph {
///         events: vec![EventWithRange::new(Event::Text("Hello".into()), span)],
///     },
///     span: SourceByteRange::new(start, end)?,
/// };
///
/// // Extract text using helper method
/// assert_eq!(block.text(), Some("Hello".to_string()));
/// ```
#[derive(Debug, Clone, PartialEq)]
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "module-private struct with deliberate pub(crate) fields"
)]
pub(crate) struct Block<'source> {
    /// The type and content of this block.
    pub(crate) kind: BlockKind<'source>,
    /// The complete source byte range (both start and end known).
    pub(crate) span: SourceByteRange,
}

impl<'source> Block<'source> {
    /// Extract plain text from inline events (lazy evaluation).
    ///
    /// Returns `Some(String)` for leaf blocks that contain inline content
    /// (Paragraph, Heading). Returns `None` for container blocks, code blocks,
    /// and other non-scannable blocks.
    ///
    /// # Performance
    ///
    /// This method allocates a new `String` on each call. For repeated access,
    /// cache the result.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let paragraph = Block {
    ///     kind: BlockKind::Paragraph {
    ///         events: vec![
    ///             EventWithRange::new(Event::Text("Hello ".into()), span1),
    ///             EventWithRange::new(Event::Text("world".into()), span2),
    ///         ],
    ///     },
    ///     span,
    /// };
    ///
    /// assert_eq!(paragraph.text(), Some("Hello world".to_string()));
    /// ```
    #[must_use]
    pub(crate) fn text(&self) -> Option<String> {
        match &self.kind {
            BlockKind::Paragraph {
                events,
            }
            | BlockKind::Heading {
                events,
                ..
            } => Some(Self::events_to_text(events)),
            _ => None,
        }
    }

    /// Returns true if this block should be scanned for metadata.
    ///
    /// Code blocks and frontmatter return false (we don't scan code or
    /// frontmatter content for tags/fields). All other blocks return true.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let paragraph = Block { kind: BlockKind::Paragraph { .. }, .. };
    /// assert!(paragraph.is_scannable());
    ///
    /// let code_block = Block { kind: BlockKind::CodeBlock { .. }, .. };
    /// assert!(!code_block.is_scannable());
    /// ```
    #[must_use]
    #[inline]
    pub(crate) fn is_scannable(&self) -> bool {
        !matches!(
            self.kind,
            BlockKind::CodeBlock { .. } | BlockKind::Frontmatter { .. }
        )
    }

    /// Helper: Extract text from a sequence of events.
    fn events_to_text(events: &[EventWithRange<'source>]) -> String {
        events
            .iter()
            .filter_map(|e| {
                #[expect(
                    clippy::pattern_type_mismatch,
                    reason = "Matching on borrowed enum variant from accessor"
                )]
                match e.event() {
                    Event::Text(s) => Some(s.as_ref()),
                    _ => None,
                }
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

/// The type and content of a markdown block.
///
/// This enum distinguishes between **leaf blocks** (which contain inline
/// content like text and code spans) and **container blocks** (which contain
/// other blocks). The structure closely mirrors the `CommonMark` specification.
///
/// # Leaf vs Container
///
/// - **Leaf blocks**: Store `events: Vec<EventWithRange>` (inline content)
/// - **Container blocks**: Store `children: Vec<Block>` (nested blocks)
///
/// # Examples
///
/// ```rust,ignore
/// // Leaf block
/// let paragraph = BlockKind::Paragraph {
///     events: vec![EventWithRange::new(Event::Text("text".into()), span)],
/// };
///
/// // Container block
/// let list = BlockKind::List {
///     kind: ListKind::Unordered,
///     children: vec![
///         Block { kind: BlockKind::ListItem { .. }, .. },
///     ],
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub(crate) enum BlockKind<'source> {
    // ═══════════════════════════════════════════════════════════
    // LEAF BLOCKS (Content-bearing)
    // ═══════════════════════════════════════════════════════════
    /// Paragraph block containing inline content.
    ///
    /// Paragraphs are the most common block type, containing text, links,
    /// emphasis, code spans, and other inline elements.
    Paragraph {
        events: Vec<EventWithRange<'source>>,
    },

    /// Heading block (H1-H6) with inline content.
    ///
    /// Headings provide document structure and are often used to extract
    /// a table of contents.
    Heading {
        level: HeadingLevel,
        events: Vec<EventWithRange<'source>>,
    },

    /// Fenced or indented code block.
    ///
    /// Code blocks are **not scannable** (we don't extract metadata from code).
    /// The text is flattened from events during AST building.
    CodeBlock {
        language: Option<CowStr<'source>>,
        text: String,
    },

    /// YAML or Pluses-delimited frontmatter block.
    ///
    /// Frontmatter is **not scannable** for inline metadata (it's structured
    /// data). Can only appear at the start of a document.
    Frontmatter {
        format: MetadataBlockKind,
        text: String,
    },

    /// Thematic break (horizontal rule).
    ///
    /// Represented in markdown as `---`, `***`, or `___`.
    ThematicBreak,

    // ═══════════════════════════════════════════════════════════
    // CONTAINER BLOCKS (Structure-bearing)
    // ═══════════════════════════════════════════════════════════
    /// Blockquote containing nested blocks.
    ///
    /// Blockquotes can contain any other blocks, including other blockquotes,
    /// lists, paragraphs, etc.
    BlockQuote {
        children: Vec<Block<'source>>,
    },

    /// Ordered or unordered list containing list items.
    ///
    /// The `kind` field distinguishes between ordered (`1. item`) and
    /// unordered (`- item`) lists.
    List {
        kind: ListKind,
        children: Vec<Block<'source>>,
    },

    /// Individual list item (can contain paragraphs, sublists, etc.).
    ///
    /// List items are **container blocks** per `CommonMark` spec—they can hold
    /// multiple paragraphs, code blocks, nested lists, etc.
    ///
    /// # Depth Tracking
    ///
    /// - `depth`: Nesting level (0 = root, 1 = first level nested, etc.)
    /// - `parent_span`: Byte range of the parent list item (for nested items)
    ///
    /// # Task Lists
    ///
    /// - `is_checkbox == Some(true)`: Checked task `- [x] Done`
    /// - `is_checkbox == Some(false)`: Unchecked task `- [ ] Todo`
    /// - `is_checkbox == None`: Regular list item `- Item`
    ListItem {
        /// Nesting depth (0 = root, 1 = first level, etc.)
        depth: u32,
        /// Span of parent list item (if nested).
        parent_span: Option<SourceByteRange>,
        /// Checkbox state (Some(true) = checked, Some(false) = unchecked, None
        /// = not a task).
        is_checkbox: Option<bool>,
        /// Child blocks (paragraphs, code, sublists, etc.)
        children: Vec<Block<'source>>,
    },
}

/// Heading level (H1 through H6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl From<pulldown_cmark::HeadingLevel> for HeadingLevel {
    fn from(level: pulldown_cmark::HeadingLevel) -> Self {
        match level {
            pulldown_cmark::HeadingLevel::H1 => Self::H1,
            pulldown_cmark::HeadingLevel::H2 => Self::H2,
            pulldown_cmark::HeadingLevel::H3 => Self::H3,
            pulldown_cmark::HeadingLevel::H4 => Self::H4,
            pulldown_cmark::HeadingLevel::H5 => Self::H5,
            pulldown_cmark::HeadingLevel::H6 => Self::H6,
        }
    }
}

impl HeadingLevel {
    /// Convert to numeric level (1-6).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert_eq!(HeadingLevel::H1.as_u8(), 1);
    /// assert_eq!(HeadingLevel::H6.as_u8(), 6);
    /// ```
    #[must_use]
    #[inline]
    pub(crate) const fn as_u8(self) -> u8 {
        match self {
            Self::H1 => 1,
            Self::H2 => 2,
            Self::H3 => 3,
            Self::H4 => 4,
            Self::H5 => 5,
            Self::H6 => 6,
        }
    }
}

/// List type (ordered or unordered).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ListKind {
    /// Unordered list (`-`, `*`, or `+` markers).
    Unordered,
    /// Ordered list (`1.`, `2.`, etc. markers).
    Ordered {
        /// Starting number (usually 1, but can be any positive integer).
        start: u64,
    },
}

// ═══════════════════════════════════════════════════════════════════════════
// DOC STRUCTURE - THE DOCUMENT AST
// ═══════════════════════════════════════════════════════════════════════════

/// The complete document structure (AST) for a markdown document.
///
/// This is the primary output of the parsing layer. It represents the
/// hierarchical structure of a markdown document as a tree of [`Block`]
/// instances.
///
/// # Lifecycle
///
/// 1. **Creation**: `DocStructure::from_context(ctx)` builds the AST from
///    cached events
/// 2. **Traversal**: Consumers walk the tree to extract metadata (Phase 4)
/// 3. **Immutability**: Once created, the structure is read-only
///
/// # Examples
///
/// ```rust,ignore
/// use lithos_core::note::parser::{ParserContext, structure::DocStructure};
///
/// let source = "# Heading\n\nParagraph";
/// let ctx = ParserContext::new(source, config)?;
/// let structure = DocStructure::from_context(&ctx)?;
///
/// // Access root-level blocks
/// for block in structure.blocks() {
///     // ... process block
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DocStructure<'source> {
    /// Root-level blocks in the document.
    blocks: Vec<Block<'source>>,
}

impl<'source> DocStructure<'source> {
    /// Build the document structure from a parser context.
    ///
    /// This method transforms the flat event stream from `ParserContext` into
    /// a hierarchical tree of blocks by processing each event and maintaining
    /// a stack of open blocks.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if:
    /// - The event stream contains mismatched Start/End tags (stack underflow)
    /// - Source byte offsets cannot be converted to [`SourceByteRange`]
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let source = "# Heading\n\nParagraph";
    /// let ctx = ParserContext::new(source, config)?;
    /// let structure = DocStructure::from_context(&ctx)?;
    ///
    /// assert_eq!(structure.blocks().len(), 2); // Heading + Paragraph
    /// ```
    pub(crate) fn from_context(
        ctx: &ParserContext<'source>,
    ) -> Result<Self, NoteIngestError> {
        let mut builder = StructureBuilder::new();

        for spanned_event in ctx.events() {
            builder.process_event(spanned_event)?;
        }

        Ok(Self {
            blocks: builder.finalize()?,
        })
    }

    /// Returns a borrowed slice of the root-level blocks.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let structure = DocStructure::from_context(&ctx)?;
    /// for block in structure.blocks() {
    ///     match &block.kind {
    ///         BlockKind::Heading { level, .. } => {
    ///             println!("Found heading at level {}", level.as_u8());
    ///         }
    ///         _ => {}
    ///     }
    /// }
    /// ```
    #[must_use]
    #[inline]
    pub(crate) fn blocks(&self) -> &[Block<'source>] {
        &self.blocks
    }

    /// Traverse the document AST using a visitor.
    ///
    /// This method performs a **pre-order depth-first traversal** of the block
    /// tree, calling the appropriate `visit_*` method on the visitor for each
    /// block. Container blocks (lists, blockquotes) are visited before their
    /// children, and depth tracking is handled automatically.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// struct BlockCounter {
    ///     count: usize,
    /// }
    ///
    /// impl<'source> BlockVisitor<'source> for BlockCounter {
    ///     fn visit_paragraph(&mut self, _block: &Block, _depth: u32) {
    ///         self.count += 1;
    ///     }
    ///     fn visit_heading(&mut self, _block: &Block, _level: HeadingLevel, _depth: u32) {
    ///         self.count += 1;
    ///     }
    ///     // ... implement other visit methods
    /// }
    ///
    /// let mut counter = BlockCounter { count: 0 };
    /// structure.walk(&mut counter);
    /// println!("Found {} blocks", counter.count);
    /// ```
    pub(crate) fn walk<V>(&self, visitor: &mut V)
    where
        V: super::visitor::BlockVisitor<'source>,
    {
        for block in &self.blocks {
            Self::walk_block(block, visitor, 0);
        }
    }

    /// Recursively walk a single block and its children.
    fn walk_block<V>(block: &Block<'source>, visitor: &mut V, depth: u32)
    where
        V: super::visitor::BlockVisitor<'source>,
    {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Pattern matching on referenced enum with exhaustive \
                      variants"
        )]
        match &block.kind {
            BlockKind::Paragraph {
                ..
            } => {
                visitor.visit_paragraph(block, depth);
            }
            BlockKind::Heading {
                level,
                ..
            } => {
                visitor.visit_heading(block, *level, depth);
            }
            BlockKind::CodeBlock {
                language,
                ..
            } => {
                visitor.visit_code_block(block, language.as_ref(), depth);
            }
            BlockKind::Frontmatter {
                format,
                ..
            } => {
                visitor.visit_frontmatter(block, *format, depth);
            }
            BlockKind::ThematicBreak => {
                visitor.visit_thematic_break(block, depth);
            }
            BlockKind::BlockQuote {
                children,
            } => {
                visitor.visit_blockquote(block, depth);
                // Visit children with incremented depth
                for child in children {
                    Self::walk_block(child, visitor, depth.saturating_add(1));
                }
            }
            BlockKind::List {
                kind,
                children,
            } => {
                visitor.visit_list(block, *kind, depth);
                // Visit list items (children are ListItem blocks)
                for child in children {
                    Self::walk_block(child, visitor, depth);
                }
            }
            BlockKind::ListItem {
                is_checkbox,
                children,
                ..
            } => {
                visitor.visit_list_item(block, *is_checkbox, depth);
                // Visit item contents with incremented depth
                for child in children {
                    Self::walk_block(child, visitor, depth.saturating_add(1));
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// STRUCTURE BUILDER - INTERNAL AST CONSTRUCTION
// ═══════════════════════════════════════════════════════════════════════════

/// Internal builder for constructing the document AST.
///
/// This type is private to the `structure` module and is never exposed outside.
/// It maintains a stack of `ProcessingBlock` instances and processes events
/// one at a time to build the hierarchical structure.
///
/// # Algorithm
///
/// The builder uses a **stack-based algorithm** to process the flat event
/// stream:
///
/// 1. **Start events**: Push new `ProcessingBlock` onto `stack`
/// 2. **Inline events** (Text, Code, etc.): Accumulate in current block
/// 3. **End events**: Pop block, finalize, push to parent or `root_blocks`
/// 4. **Task markers**: Update current list item's checkbox state
/// 5. **Rules**: Create thematic break block immediately
///
/// ## Depth Tracking
///
/// - `depth` increments when entering containers (`BlockQuote`, List)
/// - `depth` decrements when exiting containers
/// - Pre-computed during building for efficient access later
///
/// ## List Item Parent Tracking
///
/// - `list_item_parents[depth]` stores the span of the most recent list item at
///   that depth level
/// - Used to populate `ListItem::parent_span` for nested items
///
/// # Invariants
///
/// - `stack` contains only incomplete blocks (no `completed_block` field)
/// - `root_blocks` contains only finalized blocks from depth 0
/// - Stack must be empty after `finalize()` (all blocks closed)
struct StructureBuilder<'source> {
    /// Stack of blocks currently being built.
    stack: Vec<ProcessingBlock<'source>>,
    /// Completed root-level blocks.
    root_blocks: Vec<Block<'source>>,
    /// Current nesting depth (for lists and blockquotes).
    depth: u32,
    /// List item parent tracking (indexed by depth).
    list_item_parents: Vec<SourceByteRange>,
}

impl<'source> StructureBuilder<'source> {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            root_blocks: Vec::new(),
            depth: 0,
            list_item_parents: Vec::new(),
        }
    }

    fn process_event(
        &mut self,
        spanned_event: &EventWithRange<'source>,
    ) -> Result<(), NoteIngestError> {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Matching on borrowed enum variant from accessor"
        )]
        match spanned_event.event() {
            Event::Start(tag) => self.on_start(tag, spanned_event.range())?,
            Event::End(tag_end) => {
                self.on_end(tag_end, spanned_event.range())?;
            }
            Event::Text(_)
            | Event::Code(_)
            | Event::Html(_)
            | Event::InlineHtml(_) => {
                self.on_inline_event(spanned_event);
            }
            Event::TaskListMarker(checked) => {
                self.on_task_marker(*checked);
            }
            Event::Rule => {
                // Thematic break (horizontal rule) - standalone block
                let block = Block {
                    kind: BlockKind::ThematicBreak,
                    span: spanned_event.range(),
                };
                if let Some(parent) = self.stack.last_mut() {
                    parent.push_child(block);
                } else {
                    self.root_blocks.push(block);
                }
            }
            _ => {
                // Ignore other events (SoftBreak, HardBreak already normalized)
            }
        }
        Ok(())
    }

    fn on_start(
        &mut self,
        tag: &Tag<'source>,
        span: SourceByteRange,
    ) -> Result<(), NoteIngestError> {
        match tag {
            Tag::Paragraph => {
                self.stack.push(ProcessingBlock::new_leaf(
                    ProcessingBlockKind::Paragraph,
                    span.start().as_usize(),
                    self.depth,
                ));
            }
            Tag::Heading {
                level,
                ..
            } => {
                self.stack.push(ProcessingBlock::new_leaf(
                    ProcessingBlockKind::Heading {
                        level: (*level).into(),
                    },
                    span.start().as_usize(),
                    self.depth,
                ));
            }
            Tag::BlockQuote(_) => {
                self.stack.push(ProcessingBlock::new_container(
                    ProcessingBlockKind::BlockQuote,
                    span.start().as_usize(),
                    self.depth,
                ));
                self.depth = self.depth.saturating_add(1);
            }
            Tag::CodeBlock(code_kind) => {
                let language = match code_kind {
                    pulldown_cmark::CodeBlockKind::Indented => None,
                    pulldown_cmark::CodeBlockKind::Fenced(info) => {
                        if info.is_empty() {
                            None
                        } else {
                            Some(info.to_string())
                        }
                    }
                };
                self.stack.push(ProcessingBlock::new_leaf(
                    ProcessingBlockKind::CodeBlock {
                        language,
                    },
                    span.start().as_usize(),
                    self.depth,
                ));
            }
            Tag::List(start) => {
                let kind = match start {
                    Some(n) => ListKind::Ordered {
                        start: *n,
                    },
                    None => ListKind::Unordered,
                };
                self.stack.push(ProcessingBlock::new_container(
                    ProcessingBlockKind::List {
                        kind,
                    },
                    span.start().as_usize(),
                    self.depth,
                ));
                self.depth = self.depth.saturating_add(1);
            }
            Tag::Item => {
                let parent_span = if self.depth > 1 {
                    let parent_depth_index = usize::try_from(self.depth)
                        .unwrap_or(0)
                        .saturating_sub(2);
                    self.list_item_parents.get(parent_depth_index).copied()
                } else {
                    None
                };

                self.stack.push(ProcessingBlock::new_container(
                    ProcessingBlockKind::ListItem {
                        is_checkbox: None,
                        parent_span,
                    },
                    span.start().as_usize(),
                    self.depth,
                ));
            }
            Tag::MetadataBlock(kind) => {
                self.stack.push(ProcessingBlock::new_leaf(
                    ProcessingBlockKind::Frontmatter {
                        format: *kind,
                    },
                    span.start().as_usize(),
                    self.depth,
                ));
            }
            _ => {
                // Other tags (Link, Image, Emphasis, etc.) are inline, not
                // blocks
            }
        }
        Ok(())
    }

    fn on_end(
        &mut self,
        tag_end: &TagEnd,
        span: SourceByteRange,
    ) -> Result<(), NoteIngestError> {
        match tag_end {
            TagEnd::Paragraph
            | TagEnd::Heading(_)
            | TagEnd::CodeBlock
            | TagEnd::MetadataBlock(_) => {
                let processing = self.stack.pop().ok_or_else(|| {
                    NoteParseError::Markdown {
                        line: 0,
                        column: 0,
                        reason: "stack underflow: End tag without matching \
                                 Start"
                            .into(),
                    }
                })?;

                let block = processing.finalize(span.end().as_usize())?;

                // Append to parent or root
                if let Some(parent) = self.stack.last_mut() {
                    parent.push_child(block);
                } else {
                    // This is a root block - we'll collect it in finalize()
                    // For now, push it back as a completed block
                    self.root_blocks.push(block);
                }
            }
            TagEnd::BlockQuote(_) => {
                let processing = self.stack.pop().ok_or_else(|| {
                    NoteParseError::Markdown {
                        line: 0,
                        column: 0,
                        reason: "stack underflow: End BlockQuote without Start"
                            .into(),
                    }
                })?;

                self.depth = self.depth.saturating_sub(1);
                let block = processing.finalize(span.end().as_usize())?;

                if let Some(parent) = self.stack.last_mut() {
                    parent.push_child(block);
                } else {
                    self.root_blocks.push(block);
                }
            }
            TagEnd::List(_) => {
                let processing = self.stack.pop().ok_or_else(|| {
                    NoteParseError::Markdown {
                        line: 0,
                        column: 0,
                        reason: "stack underflow: End List without Start"
                            .into(),
                    }
                })?;

                self.depth = self.depth.saturating_sub(1);
                let block = processing.finalize(span.end().as_usize())?;

                if let Some(parent) = self.stack.last_mut() {
                    parent.push_child(block);
                } else {
                    self.root_blocks.push(block);
                }
            }
            TagEnd::Item => {
                let processing = self.stack.pop().ok_or_else(|| {
                    NoteParseError::Markdown {
                        line: 0,
                        column: 0,
                        reason: "stack underflow: End Item without Start"
                            .into(),
                    }
                })?;

                let block = processing.finalize(span.end().as_usize())?;

                // Track this list item as a potential parent
                let depth_index =
                    usize::try_from(self.depth).unwrap_or(0).saturating_sub(1);
                if self.list_item_parents.len() <= depth_index {
                    self.list_item_parents
                        .resize(depth_index.saturating_add(1), span);
                }
                if let Some(slot) = self.list_item_parents.get_mut(depth_index)
                {
                    *slot = block.span;
                }

                if let Some(parent) = self.stack.last_mut() {
                    parent.push_child(block);
                } else {
                    self.root_blocks.push(block);
                }
            }
            _ => {
                // Other end tags (Link, Image, Emphasis) are inline
            }
        }
        Ok(())
    }

    fn on_inline_event(&mut self, event: &EventWithRange<'source>) {
        if let Some(current) = self.stack.last_mut() {
            current.push_event(event.clone());
        }
    }

    fn on_task_marker(&mut self, checked: bool) {
        if let Some(current) = self.stack.last_mut() {
            current.set_task_marker(checked);
        }
    }

    fn finalize(self) -> Result<Vec<Block<'source>>, NoteIngestError> {
        // Check that all blocks were closed
        if !self.stack.is_empty() {
            return Err(NoteParseError::Markdown {
                line: 0,
                column: 0,
                reason: "unclosed blocks at end of document".into(),
            }
            .into());
        }
        Ok(self.root_blocks)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PROCESSING BLOCK - TEMPORARY BUILDER STATE
// ═══════════════════════════════════════════════════════════════════════════

/// Temporary state for a block being constructed.
///
/// This type only exists during AST building and is never exposed outside
/// the `StructureBuilder`. It accumulates events and child blocks until
/// the closing tag arrives, then finalizes into a complete [`Block`].
struct ProcessingBlock<'source> {
    kind: ProcessingBlockKind,
    start: usize,
    events: Vec<EventWithRange<'source>>,
    children: Vec<Block<'source>>,
    depth: u32,
}

impl<'source> ProcessingBlock<'source> {
    fn new_leaf(kind: ProcessingBlockKind, start: usize, depth: u32) -> Self {
        Self {
            kind,
            start,
            events: Vec::new(),
            children: Vec::new(),
            depth,
        }
    }

    fn new_container(
        kind: ProcessingBlockKind,
        start: usize,
        depth: u32,
    ) -> Self {
        Self {
            kind,
            start,
            events: Vec::new(),
            children: Vec::new(),
            depth,
        }
    }

    fn push_event(&mut self, event: EventWithRange<'source>) {
        self.events.push(event);
    }

    fn push_child(&mut self, child: Block<'source>) {
        self.children.push(child);
    }

    fn set_task_marker(&mut self, checked: bool) {
        if let ProcessingBlockKind::ListItem {
            is_checkbox,
            ..
        } = &mut self.kind
        {
            *is_checkbox = Some(checked);
        }
    }

    fn finalize(self, end: usize) -> Result<Block<'source>, NoteIngestError> {
        let span = SourceByteRange::try_from(self.start..end)
            .map_err(NoteIngestError::Domain)?;

        let kind = match self.kind {
            ProcessingBlockKind::Paragraph => BlockKind::Paragraph {
                events: self.events,
            },
            ProcessingBlockKind::Heading {
                level,
            } => BlockKind::Heading {
                level,
                events: self.events,
            },
            ProcessingBlockKind::CodeBlock {
                language,
            } => {
                let text = Self::flatten_text(&self.events);
                BlockKind::CodeBlock {
                    language: language
                        .map(|s| CowStr::Boxed(s.into_boxed_str())),
                    text,
                }
            }
            ProcessingBlockKind::Frontmatter {
                format,
            } => {
                let text = Self::flatten_text(&self.events);
                BlockKind::Frontmatter {
                    format,
                    text,
                }
            }
            ProcessingBlockKind::BlockQuote => BlockKind::BlockQuote {
                children: self.children,
            },
            ProcessingBlockKind::List {
                kind,
            } => BlockKind::List {
                kind,
                children: self.children,
            },
            ProcessingBlockKind::ListItem {
                is_checkbox,
                parent_span,
            } => BlockKind::ListItem {
                depth: self.depth.saturating_sub(1),
                parent_span,
                is_checkbox,
                children: self.children,
            },
        };

        Ok(Block {
            kind,
            span,
        })
    }

    fn flatten_text(events: &[EventWithRange<'source>]) -> String {
        events
            .iter()
            .filter_map(|e| {
                #[expect(
                    clippy::pattern_type_mismatch,
                    reason = "Matching on borrowed enum variant from accessor"
                )]
                match e.event() {
                    Event::Text(s) => Some(s.as_ref()),
                    _ => None,
                }
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

/// The type of block being built (parallel to `BlockKind` but incomplete).
///
/// This enum mirrors [`BlockKind`] but represents blocks that are still being
/// constructed. The key differences:
///
/// - Uses `String` instead of `CowStr` (converted during finalization)
/// - No `children` field (stored separately in `ProcessingBlock`)
/// - No `ThematicBreak` variant (created directly, not built over time)
///
/// When a block is finalized, this type is converted to the corresponding
/// [`BlockKind`] variant.
enum ProcessingBlockKind {
    Paragraph,
    Heading {
        level: HeadingLevel,
    },
    CodeBlock {
        language: Option<String>,
    },
    Frontmatter {
        format: MetadataBlockKind,
    },
    BlockQuote,
    List {
        kind: ListKind,
    },
    ListItem {
        is_checkbox: Option<bool>,
        parent_span: Option<SourceByteRange>,
    },
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module keeps imports and nested suites grouped for \
              readability"
)]
mod tests {
    use super::*;
    use crate::note::position::SourceByteOffset;

    fn span(start: usize, end: usize) -> SourceByteRange {
        SourceByteRange::new(
            SourceByteOffset::new(
                start.try_into().expect("start should fit in u32"),
            ),
            SourceByteOffset::new(
                end.try_into().expect("end should fit in u32"),
            ),
        )
        .expect("test span should be valid")
    }

    fn text_event(text: &str, start: usize, end: usize) -> EventWithRange<'_> {
        EventWithRange::new(
            Event::Text(CowStr::Borrowed(text)),
            span(start, end),
        )
    }

    mod block_text {
        use super::*;

        #[test]
        fn extracts_text_from_paragraph() {
            let block = Block {
                kind: BlockKind::Paragraph {
                    events: vec![
                        text_event("Hello ", 0, 6),
                        text_event("world", 6, 11),
                    ],
                },
                span: span(0, 11),
            };

            assert_eq!(
                block.text(),
                Some("Hello world".to_owned()),
                "paragraph should extract concatenated text from events"
            );
        }

        #[test]
        fn extracts_text_from_heading() {
            let block = Block {
                kind: BlockKind::Heading {
                    level: HeadingLevel::H1,
                    events: vec![text_event("Title", 2, 7)],
                },
                span: span(0, 7),
            };

            assert_eq!(
                block.text(),
                Some("Title".to_owned()),
                "heading should extract text from events"
            );
        }

        #[test]
        fn returns_none_for_code_block() {
            let block = Block {
                kind: BlockKind::CodeBlock {
                    language: Some(CowStr::Borrowed("rust")),
                    text: "fn main() {}".to_owned(),
                },
                span: span(0, 20),
            };

            assert_eq!(
                block.text(),
                None,
                "code block should not provide text extraction"
            );
        }

        #[test]
        fn returns_none_for_container_blocks() {
            let block = Block {
                kind: BlockKind::BlockQuote {
                    children: vec![],
                },
                span: span(0, 10),
            };

            assert_eq!(
                block.text(),
                None,
                "container blocks should not provide text extraction"
            );
        }

        #[test]
        fn filters_non_text_events() {
            let block = Block {
                kind: BlockKind::Paragraph {
                    events: vec![
                        text_event("Before ", 0, 7),
                        EventWithRange::new(
                            Event::Code(CowStr::Borrowed("code")),
                            span(7, 13),
                        ),
                        text_event(" After", 13, 19),
                    ],
                },
                span: span(0, 19),
            };

            // Code events are filtered out by events_to_text
            assert_eq!(
                block.text(),
                Some("Before  After".to_owned()),
                "should filter non-text events"
            );
        }
    }

    mod block_is_scannable {
        use super::*;

        #[test]
        fn paragraph_is_scannable() {
            let block = Block {
                kind: BlockKind::Paragraph {
                    events: vec![],
                },
                span: span(0, 5),
            };

            assert!(
                block.is_scannable(),
                "paragraphs should be scannable for metadata"
            );
        }

        #[test]
        fn heading_is_scannable() {
            let block = Block {
                kind: BlockKind::Heading {
                    level: HeadingLevel::H2,
                    events: vec![],
                },
                span: span(0, 5),
            };

            assert!(
                block.is_scannable(),
                "headings should be scannable for metadata"
            );
        }

        #[test]
        fn code_block_is_not_scannable() {
            let block = Block {
                kind: BlockKind::CodeBlock {
                    language: None,
                    text: String::new(),
                },
                span: span(0, 5),
            };

            assert!(
                !block.is_scannable(),
                "code blocks should not be scanned for metadata"
            );
        }

        #[test]
        fn frontmatter_is_not_scannable() {
            let block = Block {
                kind: BlockKind::Frontmatter {
                    format: MetadataBlockKind::YamlStyle,
                    text: String::new(),
                },
                span: span(0, 5),
            };

            assert!(
                !block.is_scannable(),
                "frontmatter should not be scanned for inline metadata"
            );
        }

        #[test]
        fn list_is_scannable() {
            let block = Block {
                kind: BlockKind::List {
                    kind: ListKind::Unordered,
                    children: vec![],
                },
                span: span(0, 5),
            };

            assert!(
                block.is_scannable(),
                "lists should be scannable (children will be scanned)"
            );
        }
    }

    mod heading_level {
        use super::*;

        #[test]
        fn as_u8_returns_numeric_level() {
            assert_eq!(HeadingLevel::H1.as_u8(), 1);
            assert_eq!(HeadingLevel::H2.as_u8(), 2);
            assert_eq!(HeadingLevel::H3.as_u8(), 3);
            assert_eq!(HeadingLevel::H4.as_u8(), 4);
            assert_eq!(HeadingLevel::H5.as_u8(), 5);
            assert_eq!(HeadingLevel::H6.as_u8(), 6);
        }

        #[test]
        fn from_pulldown_cmark_heading_level() {
            assert_eq!(
                HeadingLevel::from(pulldown_cmark::HeadingLevel::H1),
                HeadingLevel::H1
            );
            assert_eq!(
                HeadingLevel::from(pulldown_cmark::HeadingLevel::H6),
                HeadingLevel::H6
            );
        }
    }

    mod list_kind {
        use super::*;

        #[test]
        fn unordered_has_no_start_number() {
            let kind = ListKind::Unordered;
            assert!(
                matches!(kind, ListKind::Unordered),
                "unordered lists have no start parameter"
            );
        }

        #[test]
        fn ordered_stores_start_number() {
            let kind = ListKind::Ordered {
                start: 5,
            };
            if let ListKind::Ordered {
                start,
            } = kind
            {
                assert_eq!(start, 5, "ordered lists should store start number");
            } else {
                panic!("expected ordered list");
            }
        }
    }

    mod doc_structure_from_context {
        use super::*;
        use crate::note::parser::{
            config::EventStreamConfig, context::ParserContext,
        };

        #[test]
        fn parses_simple_paragraph() {
            let source = "Hello world";
            let config = EventStreamConfig::default();
            let ctx =
                ParserContext::new(source, config).expect("parse context");

            let structure =
                DocStructure::from_context(&ctx).expect("build structure");

            assert_eq!(
                structure.blocks().len(),
                1,
                "should have one paragraph"
            );
            if let BlockKind::Paragraph {
                events,
            } = &structure.blocks()[0].kind
            {
                assert!(!events.is_empty(), "paragraph should have events");
            } else {
                panic!("expected paragraph block");
            }
        }

        #[test]
        fn parses_heading() {
            let source = "# Title";
            let config = EventStreamConfig::default();
            let ctx =
                ParserContext::new(source, config).expect("parse context");

            let structure =
                DocStructure::from_context(&ctx).expect("build structure");

            assert_eq!(structure.blocks().len(), 1, "should have one heading");
            if let BlockKind::Heading {
                level,
                events,
            } = &structure.blocks()[0].kind
            {
                assert_eq!(*level, HeadingLevel::H1, "should be H1");
                assert!(!events.is_empty(), "heading should have events");
            } else {
                panic!("expected heading block");
            }
        }

        #[test]
        fn parses_code_block() {
            let source = "```rust\nfn main() {}\n```";
            let config = EventStreamConfig::default();
            let ctx =
                ParserContext::new(source, config).expect("parse context");

            let structure =
                DocStructure::from_context(&ctx).expect("build structure");

            assert_eq!(
                structure.blocks().len(),
                1,
                "should have one code block"
            );
            if let BlockKind::CodeBlock {
                language,
                text,
            } = &structure.blocks()[0].kind
            {
                assert_eq!(
                    language.as_ref().map(std::convert::AsRef::as_ref),
                    Some("rust")
                );
                assert!(text.contains("fn main"), "should contain code text");
            } else {
                panic!("expected code block");
            }
        }

        #[test]
        fn parses_unordered_list() {
            let source = "- Item 1\n- Item 2";
            let config = EventStreamConfig::default();
            let ctx =
                ParserContext::new(source, config).expect("parse context");

            let structure =
                DocStructure::from_context(&ctx).expect("build structure");

            assert_eq!(structure.blocks().len(), 1, "should have one list");
            if let BlockKind::List {
                kind,
                children,
            } = &structure.blocks()[0].kind
            {
                assert_eq!(*kind, ListKind::Unordered);
                assert_eq!(children.len(), 2, "should have 2 list items");
            } else {
                panic!("expected list block");
            }
        }

        #[test]
        fn parses_ordered_list() {
            let source = "1. First\n2. Second";
            let config = EventStreamConfig::default();
            let ctx =
                ParserContext::new(source, config).expect("parse context");

            let structure =
                DocStructure::from_context(&ctx).expect("build structure");

            assert_eq!(structure.blocks().len(), 1, "should have one list");
            if let BlockKind::List {
                kind,
                children,
            } = &structure.blocks()[0].kind
            {
                assert!(matches!(kind, ListKind::Ordered {
                    start: 1
                }));
                assert_eq!(children.len(), 2, "should have 2 list items");
            } else {
                panic!("expected list block");
            }
        }

        #[test]
        fn parses_nested_list() {
            let source = "- Parent\n  - Child";
            let config = EventStreamConfig::default();
            let ctx =
                ParserContext::new(source, config).expect("parse context");

            let structure =
                DocStructure::from_context(&ctx).expect("build structure");

            assert_eq!(structure.blocks().len(), 1, "should have one list");
            if let BlockKind::List {
                children,
                ..
            } = &structure.blocks()[0].kind
            {
                assert_eq!(children.len(), 1, "should have 1 item at root");

                // First item (parent) should have depth 0 and contain nested
                // list
                if let BlockKind::ListItem {
                    depth,
                    children: item_children,
                    ..
                } = &children[0].kind
                {
                    assert_eq!(*depth, 0, "parent item should have depth 0");

                    // Find nested list in children
                    let nested_list = item_children
                        .iter()
                        .find(|b| matches!(b.kind, BlockKind::List { .. }))
                        .expect("parent should contain nested list");

                    if let BlockKind::List {
                        children: nested_items,
                        ..
                    } = &nested_list.kind
                    {
                        assert_eq!(
                            nested_items.len(),
                            1,
                            "nested list should have 1 item"
                        );

                        if let BlockKind::ListItem {
                            depth,
                            ..
                        } = &nested_items[0].kind
                        {
                            assert_eq!(
                                depth, &1,
                                "child item should have depth 1"
                            );
                        }
                    }
                } else {
                    panic!("expected list item");
                }
            } else {
                panic!("expected list block");
            }
        }

        #[test]
        fn parses_blockquote() {
            let source = "> Quoted text";
            let config = EventStreamConfig::default();
            let ctx =
                ParserContext::new(source, config).expect("parse context");

            let structure =
                DocStructure::from_context(&ctx).expect("build structure");

            assert_eq!(
                structure.blocks().len(),
                1,
                "should have one blockquote"
            );
            if let BlockKind::BlockQuote {
                children,
            } = &structure.blocks()[0].kind
            {
                assert_eq!(
                    children.len(),
                    1,
                    "blockquote should have one paragraph"
                );
            } else {
                panic!("expected blockquote block");
            }
        }

        #[test]
        fn parses_task_list() {
            let source = "- [x] Done\n- [ ] Todo";
            let config = EventStreamConfig::default();
            let ctx =
                ParserContext::new(source, config).expect("parse context");

            let structure =
                DocStructure::from_context(&ctx).expect("build structure");

            assert_eq!(structure.blocks().len(), 1, "should have one list");
            if let BlockKind::List {
                children,
                ..
            } = &structure.blocks()[0].kind
            {
                assert_eq!(children.len(), 2, "should have 2 list items");

                // First item should be checked
                if let BlockKind::ListItem {
                    is_checkbox,
                    ..
                } = &children[0].kind
                {
                    assert_eq!(
                        *is_checkbox,
                        Some(true),
                        "first item should be checked"
                    );
                } else {
                    panic!("expected list item");
                }

                // Second item should be unchecked
                if let BlockKind::ListItem {
                    is_checkbox,
                    ..
                } = &children[1].kind
                {
                    assert_eq!(
                        *is_checkbox,
                        Some(false),
                        "second item should be unchecked"
                    );
                } else {
                    panic!("expected list item");
                }
            } else {
                panic!("expected list block");
            }
        }

        #[test]
        fn parses_frontmatter() {
            let source = "---\ntags: [test]\n---\n\nContent";
            let config = EventStreamConfig::default();
            let ctx =
                ParserContext::new(source, config).expect("parse context");

            let structure =
                DocStructure::from_context(&ctx).expect("build structure");

            assert!(
                !structure.blocks().is_empty(),
                "should have at least frontmatter"
            );
            if let BlockKind::Frontmatter {
                format,
                text,
            } = &structure.blocks()[0].kind
            {
                assert_eq!(*format, MetadataBlockKind::YamlStyle);
                assert!(
                    text.contains("tags"),
                    "should contain frontmatter text"
                );
            } else {
                panic!("expected frontmatter block");
            }
        }

        #[test]
        fn parses_multiple_root_blocks() {
            let source = "# Heading\n\nParagraph\n\n- List item";
            let config = EventStreamConfig::default();
            let ctx =
                ParserContext::new(source, config).expect("parse context");

            let structure =
                DocStructure::from_context(&ctx).expect("build structure");

            assert_eq!(
                structure.blocks().len(),
                3,
                "should have 3 root blocks"
            );
            assert!(matches!(
                structure.blocks()[0].kind,
                BlockKind::Heading { .. }
            ));
            assert!(matches!(
                structure.blocks()[1].kind,
                BlockKind::Paragraph { .. }
            ));
            assert!(matches!(
                structure.blocks()[2].kind,
                BlockKind::List { .. }
            ));
        }

        #[test]
        fn handles_empty_document() {
            let source = "";
            let config = EventStreamConfig::default();
            let ctx =
                ParserContext::new(source, config).expect("parse context");

            let structure =
                DocStructure::from_context(&ctx).expect("build structure");

            assert_eq!(
                structure.blocks().len(),
                0,
                "empty document should have no blocks"
            );
        }

        #[test]
        fn parses_thematic_break() {
            let source = "Before\n\n---\n\nAfter";
            let config = EventStreamConfig::default();
            let ctx =
                ParserContext::new(source, config).expect("parse context");

            let structure =
                DocStructure::from_context(&ctx).expect("build structure");

            assert_eq!(
                structure.blocks().len(),
                3,
                "should have paragraph, thematic break, paragraph"
            );
            assert!(matches!(
                structure.blocks()[0].kind,
                BlockKind::Paragraph { .. }
            ));
            assert!(matches!(
                structure.blocks()[1].kind,
                BlockKind::ThematicBreak
            ));
            assert!(matches!(
                structure.blocks()[2].kind,
                BlockKind::Paragraph { .. }
            ));
        }

        #[test]
        fn handles_malformed_nesting_gracefully() {
            // CommonMark parser handles structure normalization, so we should
            // always get valid AST even from "malformed" markdown
            let source =
                "- Item\n  > Quote in list\n    ```\n    code\n    ```";
            let config = EventStreamConfig::default();
            let ctx =
                ParserContext::new(source, config).expect("parse context");

            // Should not panic or error
            let structure = DocStructure::from_context(&ctx);
            assert!(structure.is_ok(), "should handle complex nesting");
        }

        #[test]
        fn handles_unicode_content() {
            let source = "# \u{65e5}\u{672c}\u{8a9e} Heading\n\nParagraph \
                          with emoji \u{1f389}\n\n- List \u{9879}\u{76ee}";
            let config = EventStreamConfig::default();
            let ctx =
                ParserContext::new(source, config).expect("parse context");

            let structure =
                DocStructure::from_context(&ctx).expect("build structure");

            assert_eq!(
                structure.blocks().len(),
                3,
                "should parse unicode content"
            );
        }

        #[test]
        fn handles_whitespace_only() {
            let source = "   \n\n  \t  \n\n   ";
            let config = EventStreamConfig::default();
            let ctx =
                ParserContext::new(source, config).expect("parse context");

            let structure =
                DocStructure::from_context(&ctx).expect("build structure");

            // CommonMark treats whitespace-only as empty
            assert_eq!(
                structure.blocks().len(),
                0,
                "whitespace-only should be empty"
            );
        }

        #[test]
        fn parses_code_block_in_list() {
            let source = "- List item\n\n  ```rust\n  fn main() {}\n  ```";
            let config = EventStreamConfig::default();
            let ctx =
                ParserContext::new(source, config).expect("parse context");

            let structure =
                DocStructure::from_context(&ctx).expect("build structure");

            assert_eq!(structure.blocks().len(), 1, "should have one list");

            if let BlockKind::List {
                children,
                ..
            } = &structure.blocks()[0].kind
            {
                assert_eq!(children.len(), 1, "should have one item");

                if let BlockKind::ListItem {
                    children: item_children,
                    ..
                } = &children[0].kind
                {
                    assert_eq!(
                        item_children.len(),
                        2,
                        "item should have paragraph and code block"
                    );
                    assert!(matches!(
                        item_children[0].kind,
                        BlockKind::Paragraph { .. }
                    ));
                    assert!(matches!(
                        item_children[1].kind,
                        BlockKind::CodeBlock { .. }
                    ));
                }
            }
        }
    }
}
