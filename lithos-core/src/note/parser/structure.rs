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

#![cfg_attr(
    not(test),
    expect(dead_code, reason = "Structure builder is consumed incrementally")
)]
#![expect(
    clippy::pattern_type_mismatch,
    reason = "Parser stack code intentionally matches borrowed enum shapes"
)]

use super::{
    block::{Block, BlockKind, ContainerBlockKind, LeafBlockKind},
    context::ParserContext,
    text::TextSequence,
    types::{
        BlockEnd, BlockStart, FrontmatterFormat, HeadingLevel, ListKind,
        ParserEvent, RangedEvent,
    },
};
use crate::note::{
    error::{NoteIngestError, NoteParseError},
    position::{SourceByteOffset, SourceByteRange},
};

// ═══════════════════════════════════════════════════════════════════════════
// TRAVERSAL ITERATOR API
// ═══════════════════════════════════════════════════════════════════════════

/// Event emitted during pre-order traversal of the document AST.
///
/// Each container block (List, `BlockQuote`, `ListItem`) emits both an `Enter`
/// and `Exit` event. Leaf blocks (Paragraph, Heading, etc.) emit only `Enter`.
///
/// # Traversal Order
///
/// Pre-order traversal visits parent blocks before their children:
/// 1. Enter container
/// 2. Enter/Exit children (recursively)
/// 3. Exit container
///
/// # Examples
///
/// ```rust,ignore
/// use lithos_core::note::parser::structure::{DocStructure, TraversalEvent};
///
/// let structure = DocStructure::from_context(&ctx)?;
/// for event in structure.iter_preorder() {
///     match event {
///         TraversalEvent::Enter(block, depth) => {
///             println!("Entering block at depth {}", depth);
///         }
///         TraversalEvent::Exit(block, depth) => {
///             println!("Exiting block at depth {}", depth);
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum TraversalEvent<'tree, 'source> {
    /// Entering a block (emitted for all blocks).
    Enter(&'tree Block<'source>, u32),
    /// Exiting a container block (emitted only for containers).
    Exit(&'tree Block<'source>, u32),
}

/// Frame type for the traversal stack.
#[derive(Debug, Clone, Copy)]
enum StackFrame<'tree, 'source> {
    /// Enter a block (emit Enter event, then push children and Exit marker).
    Enter(&'tree Block<'source>, u32),
    /// Exit a container block (emit Exit event).
    Exit(&'tree Block<'source>, u32),
}

/// Pre-order depth-first iterator over the document AST.
///
/// This iterator emits [`TraversalEvent`] instances as it traverses the block
/// tree. Container blocks emit both `Enter` and `Exit` events, while leaf
/// blocks emit only `Enter`.
///
/// # Implementation Notes
///
/// - Uses a stack-based algorithm to avoid recursion
/// - Depth tracking handled automatically
/// - Children pushed in reverse order for forward iteration
/// - Exit markers pushed before children to ensure proper order
pub(crate) struct PreorderIter<'tree, 'source> {
    /// Stack of frames to process.
    stack: Vec<StackFrame<'tree, 'source>>,
}

impl<'tree, 'source> PreorderIter<'tree, 'source> {
    /// Creates a new pre-order iterator from root blocks.
    fn new(roots: &'tree [Block<'source>]) -> Self {
        // Most documents have shallow nesting (2-4 levels).
        // Pre-allocate 8 frames to handle typical nested lists/blockquotes.
        let mut stack = Vec::with_capacity(8.max(roots.len()));
        // Push roots in reverse order so we pop in forward order
        for root in roots.iter().rev() {
            stack.push(StackFrame::Enter(root, 0));
        }
        Self {
            stack,
        }
    }
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Basic iterator implementation; default trait methods are \
              sufficient"
)]
impl<'tree, 'source> Iterator for PreorderIter<'tree, 'source> {
    type Item = TraversalEvent<'tree, 'source>;

    fn next(&mut self) -> Option<Self::Item> {
        let frame = self.stack.pop()?;

        match frame {
            StackFrame::Exit(block, depth) => {
                // Just emit the exit event
                Some(TraversalEvent::Exit(block, depth))
            }
            StackFrame::Enter(block, depth) => {
                // For containers, push Exit marker first (so it's processed
                // last) then push children in reverse order
                match &block.kind {
                    BlockKind::Container(ContainerBlockKind::BlockQuote {
                        children,
                    }) => {
                        // Push exit marker first (will be popped last)
                        self.stack.push(StackFrame::Exit(block, depth));

                        // Push children in reverse order (will be popped in
                        // forward order)
                        let child_depth = depth.saturating_add(1);
                        for child in children.iter().rev() {
                            self.stack
                                .push(StackFrame::Enter(child, child_depth));
                        }
                    }
                    BlockKind::Container(ContainerBlockKind::List {
                        children,
                        ..
                    }) => {
                        // Push exit marker first (will be popped last)
                        self.stack.push(StackFrame::Exit(block, depth));

                        // List items do NOT increment depth
                        for child in children.iter().rev() {
                            self.stack.push(StackFrame::Enter(child, depth));
                        }
                    }
                    BlockKind::Container(ContainerBlockKind::ListItem {
                        children,
                        ..
                    }) => {
                        // Push exit marker first (will be popped last)
                        self.stack.push(StackFrame::Exit(block, depth));

                        // Push children in reverse order
                        let child_depth = depth.saturating_add(1);
                        for child in children.iter().rev() {
                            self.stack
                                .push(StackFrame::Enter(child, child_depth));
                        }
                    }
                    BlockKind::Leaf(_) => {
                        // Leaf blocks have no children, no exit event
                    }
                }

                // Emit Enter event
                Some(TraversalEvent::Enter(block, depth))
            }
        }
    }
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

        for spanned_event in ctx.events().iter().cloned() {
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
    ///         BlockKind::Leaf(LeafBlockKind::Heading { level, .. }) => {
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

    /// Returns an iterator over the document blocks in pre-order.
    ///
    /// This iterator emits [`TraversalEvent`] instances for each block in the
    /// tree. Container blocks emit both `Enter` and `Exit` events, while leaf
    /// blocks emit only `Enter`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let structure = DocStructure::from_context(&ctx)?;
    /// for event in structure.iter_preorder() {
    ///     match event {
    ///         TraversalEvent::Enter(block, depth) => {
    ///             println!("Block at depth {}", depth);
    ///         }
    ///         TraversalEvent::Exit(..) => {}
    ///     }
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub(crate) fn iter_preorder(&self) -> PreorderIter<'_, 'source> {
        PreorderIter::new(&self.blocks)
    }

    /// Execute a callback for each block in pre-order traversal.
    ///
    /// This is a convenience method that calls `f` for each `Enter` event,
    /// ignoring `Exit` events. Use [`iter_preorder`](Self::iter_preorder) if
    /// you need full control over Enter/Exit events.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut count = 0;
    /// structure.for_each_block(|block, depth| {
    ///     count += 1;
    /// });
    /// println!("Found {} blocks", count);
    /// ```
    pub(crate) fn for_each_block<F>(&self, mut f: F)
    where
        F: FnMut(&Block<'source>, u32),
    {
        for event in self.iter_preorder() {
            if let TraversalEvent::Enter(block, depth) = event {
                f(block, depth);
            }
        }
    }

    /// Collect all blocks in pre-order traversal.
    ///
    /// Returns a vector of (block, depth) tuples for all blocks in the
    /// document. This is useful when you need to process blocks multiple times
    /// or when you need random access to the traversal order.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let blocks = structure.blocks_preorder();
    /// for (block, depth) in &blocks {
    ///     println!("Block at depth {}", depth);
    /// }
    /// ```
    #[must_use]
    pub(crate) fn blocks_preorder(&self) -> Vec<(&Block<'source>, u32)> {
        self.iter_preorder()
            .filter_map(|event| match event {
                TraversalEvent::Enter(block, depth) => Some((block, depth)),
                TraversalEvent::Exit(..) => None,
            })
            .collect()
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
    /// In-progress stack and finalized root block storage.
    tree: ProcessingBlockTree<'source>,
    /// List depth and parent-position bookkeeping.
    nesting_stack: NestingStack,
}

impl<'source> StructureBuilder<'source> {
    fn new() -> Self {
        Self {
            tree: ProcessingBlockTree::new(),
            nesting_stack: NestingStack::new(),
        }
    }

    fn process_event(
        &mut self,
        spanned_event: RangedEvent<'source>,
    ) -> Result<(), NoteIngestError> {
        let (event, range) = spanned_event.into_parts();
        match event {
            ParserEvent::BlockStart(block_type) => {
                self.on_start(&block_type, range)?;
            }
            ParserEvent::BlockEnd(block_type) => {
                self.on_end(block_type, range)?;
            }
            inline_event @ ParserEvent::Inline(_) => {
                let inline_ranged = RangedEvent::new(inline_event, range);
                self.on_inline_event(inline_ranged)?;
            }
            ParserEvent::TaskListMarker(checked) => {
                self.on_task_marker(checked);
            }
            ParserEvent::ThematicBreak => {
                // Thematic break (horizontal rule) - standalone block
                let block = Block {
                    kind: BlockKind::Leaf(LeafBlockKind::ThematicBreak),
                    span: range,
                };
                self.tree.attach_completed(block)?;
            }
        }
        Ok(())
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Pattern matches borrowed block tags by design"
    )]
    fn on_start(
        &mut self,
        block_type: &BlockStart<'source>,
        span: SourceByteRange,
    ) -> Result<(), NoteIngestError> {
        self.auto_close_implicit_paragraph(span)?;

        let start = span.start().as_usize();
        match block_type {
            BlockStart::Paragraph => {
                self.tree.push_incomplete(ProcessingNode::Leaf {
                    kind: LeafKind::Paragraph,
                    start,
                    events: Vec::new(),
                })?;
            }
            BlockStart::Heading {
                level,
            } => {
                self.tree.push_incomplete(ProcessingNode::Leaf {
                    kind: LeafKind::Heading(*level),
                    start,
                    events: Vec::new(),
                })?;
            }
            BlockStart::BlockQuote => {
                self.tree.push_incomplete(ProcessingNode::Container {
                    kind: ContainerKind::BlockQuote,
                    start,
                    children: Vec::new(),
                })?;
            }
            BlockStart::CodeBlock {
                info_string,
            } => {
                self.tree.push_incomplete(ProcessingNode::Leaf {
                    kind: LeafKind::CodeBlock(
                        info_string.clone().map(Into::into),
                    ),
                    start,
                    events: Vec::new(),
                })?;
            }
            BlockStart::List {
                kind: list_kind,
            } => {
                self.tree.push_incomplete(ProcessingNode::Container {
                    kind: ContainerKind::List(*list_kind),
                    start,
                    children: Vec::new(),
                })?;
            }
            BlockStart::ListItem => {
                let parent_pos = self.nesting_stack.parent_pos();
                self.tree.push_incomplete(ProcessingNode::Container {
                    kind: ContainerKind::ListItem(ListItemAttrs {
                        depth: self.nesting_stack.depth(),
                        parent_pos,
                        is_checked: None,
                    }),
                    start,
                    children: Vec::new(),
                })?;
                self.nesting_stack.push_item(span.start());
            }
            BlockStart::Frontmatter {
                format,
            } => {
                self.tree.push_incomplete(ProcessingNode::Leaf {
                    kind: LeafKind::Frontmatter(*format),
                    start,
                    events: Vec::new(),
                })?;
            }
        }
        Ok(())
    }

    fn on_end(
        &mut self,
        block_type: BlockEnd,
        span: SourceByteRange,
    ) -> Result<(), NoteIngestError> {
        if block_type != BlockEnd::Paragraph {
            self.auto_close_implicit_paragraph(span)?;
        }

        let block = self.tree.finalize_matching(block_type, span)?;

        if matches!(block_type, BlockEnd::ListItem) {
            self.nesting_stack.pop_item();
        }

        self.tree.attach_completed(block)?;
        Ok(())
    }

    fn on_inline_event(
        &mut self,
        event: RangedEvent<'source>,
    ) -> Result<(), NoteIngestError> {
        let needs_paragraph = match self.tree.last_mut() {
            Some(ProcessingNode::Leaf {
                ..
            }) => false,
            Some(ProcessingNode::Container {
                ..
            })
            | None => true,
        };

        if needs_paragraph {
            self.on_start(&BlockStart::Paragraph, event.range())?;
        }

        let current = self.tree.last_mut().ok_or_else(|| {
            NoteParseError::InvalidTopology {
                code: "parser.structure.inline_outside_leaf",
                detail: "inline event encountered outside of a leaf block"
                    .into(),
                range: Some(event.range()),
            }
        })?;
        current.push_event(event)
    }

    fn auto_close_implicit_paragraph(
        &mut self,
        span: SourceByteRange,
    ) -> Result<(), NoteIngestError> {
        if let Some(top) = self.tree.last_mut()
            && top.expected_end() == BlockEnd::Paragraph
        {
            let block =
                self.tree.finalize_matching(BlockEnd::Paragraph, span)?;
            self.tree.attach_completed(block)?;
        }
        Ok(())
    }

    fn on_task_marker(&mut self, checked: bool) {
        if let Some(current) = self.tree.last_mut() {
            current.set_task_marker(checked);
        }
    }

    fn finalize(self) -> Result<Vec<Block<'source>>, NoteIngestError> {
        // Check that all blocks were closed
        if !self.tree.is_empty() {
            return Err(NoteParseError::UnclosedBlocks {
                open_count: self.tree.stack.len(),
                top_kind: self.tree.stack.last().map(ProcessingNode::kind_name),
                at: SourceByteOffset::new(0),
            }
            .into());
        }
        Ok(self.tree.into_roots())
    }
}

struct NestingStack {
    open_items: Vec<SourceByteOffset>,
}

impl NestingStack {
    const fn new() -> Self {
        Self {
            open_items: Vec::new(),
        }
    }

    fn depth(&self) -> u32 {
        u32::try_from(self.open_items.len()).unwrap_or(u32::MAX)
    }

    fn push_item(&mut self, pos: SourceByteOffset) {
        self.open_items.push(pos);
    }

    fn pop_item(&mut self) {
        self.open_items.pop();
    }

    fn parent_pos(&self) -> Option<SourceByteOffset> {
        self.open_items.last().copied()
    }
}

struct ProcessingBlockTree<'source> {
    stack: Vec<ProcessingNode<'source>>,
    root_blocks: Vec<Block<'source>>,
}

impl<'source> ProcessingBlockTree<'source> {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            root_blocks: Vec::new(),
        }
    }

    fn push_incomplete(
        &mut self,
        node: ProcessingNode<'source>,
    ) -> Result<(), NoteIngestError> {
        if let Some(top) = self.stack.last()
            && matches!(top, ProcessingNode::Leaf { .. })
        {
            let start = node.start();
            let range =
                SourceByteRange::try_from(start..start.saturating_add(1)).ok();

            return Err(NoteParseError::InvalidTopology {
                code: "parser.structure.start_inside_leaf",
                detail: "cannot start a new block inside a leaf block".into(),
                range,
            }
            .into());
        }

        self.stack.push(node);
        Ok(())
    }

    #[expect(
        dead_code,
        reason = "Reserved for future use if direct stack access needed"
    )]
    fn last(&self) -> Option<&ProcessingNode<'source>> {
        self.stack.last()
    }

    fn last_mut(&mut self) -> Option<&mut ProcessingNode<'source>> {
        self.stack.last_mut()
    }

    fn pop_incomplete(&mut self) -> Option<ProcessingNode<'source>> {
        self.stack.pop()
    }

    fn finalize_matching(
        &mut self,
        end: BlockEnd,
        end_range: SourceByteRange,
    ) -> Result<Block<'source>, NoteIngestError> {
        let processing = self.pop_incomplete().ok_or_else(|| {
            NoteParseError::EventStackUnderflow {
                expected: "open block",
                encountered: end.label(),
                depth: self.stack.len(),
                range: end_range,
            }
        })?;

        let expected_end = processing.expected_end();
        if expected_end != end {
            return Err(NoteParseError::EventStackMismatch {
                expected: expected_end.label(),
                found: end.label(),
                depth: self.stack.len(),
                start_range: processing.start_anchor_range(),
                end_range,
            }
            .into());
        }

        processing.finalize(end_range.end().as_usize())
    }

    fn attach_completed(
        &mut self,
        block: Block<'source>,
    ) -> Result<(), NoteIngestError> {
        if let Some(parent) = self.stack.last_mut() {
            parent.push_child(block)
        } else {
            self.root_blocks.push(block);
            Ok(())
        }
    }

    fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    fn into_roots(self) -> Vec<Block<'source>> {
        self.root_blocks
    }
}

pub(crate) enum ProcessingNode<'source> {
    Leaf {
        kind: LeafKind,
        start: usize,
        events: Vec<RangedEvent<'source>>,
    },
    Container {
        kind: ContainerKind,
        start: usize,
        children: Vec<Block<'source>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ListItemAttrs {
    depth: u32,
    parent_pos: Option<SourceByteOffset>,
    is_checked: Option<bool>,
}

impl ListItemAttrs {
    pub(crate) fn depth(&self) -> u32 {
        self.depth
    }

    pub(crate) fn parent_pos(&self) -> Option<SourceByteOffset> {
        self.parent_pos
    }

    pub(crate) fn is_checked(&self) -> Option<bool> {
        self.is_checked
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LeafKind {
    Paragraph,
    Heading(HeadingLevel),
    CodeBlock(Option<Box<str>>),
    Frontmatter(FrontmatterFormat),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ContainerKind {
    BlockQuote,
    List(ListKind),
    ListItem(ListItemAttrs),
}

impl<'source> ProcessingNode<'source> {
    pub(crate) fn kind_name(&self) -> &'static str {
        match self {
            Self::Leaf {
                ..
            } => "leaf",
            Self::Container {
                ..
            } => "container",
        }
    }

    pub(crate) fn expected_end(&self) -> BlockEnd {
        match self {
            Self::Leaf {
                kind,
                ..
            } => match kind {
                LeafKind::Paragraph => BlockEnd::Paragraph,
                LeafKind::Heading(_) => BlockEnd::Heading,
                LeafKind::CodeBlock(_) => BlockEnd::CodeBlock,
                LeafKind::Frontmatter(_) => BlockEnd::Frontmatter,
            },
            Self::Container {
                kind,
                ..
            } => match kind {
                ContainerKind::BlockQuote => BlockEnd::BlockQuote,
                ContainerKind::List(_) => BlockEnd::List,
                ContainerKind::ListItem(_) => BlockEnd::ListItem,
            },
        }
    }

    pub(crate) fn start(&self) -> usize {
        match self {
            Self::Leaf {
                start,
                ..
            }
            | Self::Container {
                start,
                ..
            } => *start,
        }
    }

    pub(crate) fn start_anchor_range(&self) -> Option<SourceByteRange> {
        let start = self.start();
        SourceByteRange::try_from(start..start.saturating_add(1)).ok()
    }

    pub(crate) fn push_event(
        &mut self,
        event: RangedEvent<'source>,
    ) -> Result<(), NoteIngestError> {
        match self {
            Self::Leaf {
                events,
                ..
            } => {
                events.push(event);
                Ok(())
            }
            Self::Container {
                ..
            } => Err(NoteParseError::InvalidTopology {
                code: "parser.structure.push_event_to_container",
                detail: "cannot push inline events to a container block".into(),
                range: Some(event.range()),
            }
            .into()),
        }
    }

    pub(crate) fn push_child(
        &mut self,
        child: Block<'source>,
    ) -> Result<(), NoteIngestError> {
        match self {
            Self::Container {
                children,
                ..
            } => {
                children.push(child);
                Ok(())
            }
            Self::Leaf {
                ..
            } => Err(NoteParseError::InvalidTopology {
                code: "parser.structure.push_child_to_leaf",
                detail: "cannot push child blocks to a leaf block".into(),
                range: Some(child.span),
            }
            .into()),
        }
    }

    pub(crate) fn set_task_marker(&mut self, checked: bool) {
        if let Self::Container {
            kind: ContainerKind::ListItem(attrs),
            ..
        } = self
        {
            attrs.is_checked = Some(checked);
        }
    }

    pub(crate) fn finalize(
        self,
        end: usize,
    ) -> Result<Block<'source>, NoteIngestError> {
        let span = SourceByteRange::try_from(self.start()..end)
            .map_err(NoteIngestError::Domain)?;

        match self {
            Self::Leaf {
                kind,
                events,
                ..
            } => {
                let block_kind = match kind {
                    LeafKind::Paragraph => LeafBlockKind::Paragraph {
                        events,
                    },
                    LeafKind::Heading(level) => LeafBlockKind::Heading {
                        level,
                        events,
                    },
                    LeafKind::CodeBlock(language) => {
                        let text =
                            TextSequence::from_events(&events).as_plain_text();
                        LeafBlockKind::CodeBlock {
                            language,
                            text,
                        }
                    }
                    LeafKind::Frontmatter(format) => {
                        let text =
                            TextSequence::from_events(&events).as_plain_text();
                        LeafBlockKind::Frontmatter {
                            format,
                            text,
                        }
                    }
                };
                Ok(Block {
                    kind: BlockKind::Leaf(block_kind),
                    span,
                })
            }
            Self::Container {
                kind,
                children,
                ..
            } => {
                let block_kind = match kind {
                    ContainerKind::BlockQuote => {
                        ContainerBlockKind::BlockQuote {
                            children,
                        }
                    }
                    ContainerKind::List(list_kind) => {
                        ContainerBlockKind::List {
                            kind: list_kind,
                            children,
                        }
                    }
                    ContainerKind::ListItem(attrs) => {
                        ContainerBlockKind::ListItem {
                            depth: attrs.depth(),
                            parent_pos: attrs.parent_pos(),
                            is_checked: attrs.is_checked(),
                            children,
                        }
                    }
                };
                Ok(Block {
                    kind: BlockKind::Container(block_kind),
                    span,
                })
            }
        }
    }
}

impl BlockEnd {
    const fn label(self) -> &'static str {
        match self {
            Self::Paragraph => "paragraph",
            Self::Heading => "heading",
            Self::BlockQuote => "blockquote",
            Self::List => "list",
            Self::ListItem => "list_item",
            Self::CodeBlock => "code_block",
            Self::Frontmatter => "frontmatter",
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    reason = "Parser fixture assertions intentionally index known-shape \
              structures"
)]
#[expect(
    clippy::panic,
    reason = "Pattern-guarded test assertions use panic for explicit mismatch \
              diagnostics"
)]
#[expect(
    clippy::shadow_unrelated,
    reason = "Reused local names in narrow test scopes improve readability"
)]
#[expect(
    clippy::pattern_type_mismatch,
    reason = "Test pattern matching on borrowed enums favors readability"
)]
mod tests {
    use super::*;
    use crate::note::parser::{
        config::EventStreamConfig, context::ParserContext,
    };

    fn build_structure(source: &str) -> DocStructure<'_> {
        let ctx = ParserContext::new(source, EventStreamConfig::default())
            .expect("parser context should build");
        DocStructure::from_context(&ctx).expect("structure should build")
    }

    #[test]
    fn parses_root_heading_and_paragraph() {
        let structure = build_structure("# Title\n\nBody");
        assert_eq!(structure.blocks().len(), 2);
        assert!(matches!(
            structure.blocks()[0].kind,
            BlockKind::Leaf(LeafBlockKind::Heading { .. })
        ));
        assert!(matches!(
            structure.blocks()[1].kind,
            BlockKind::Leaf(LeafBlockKind::Paragraph { .. })
        ));
    }

    #[test]
    fn preserves_task_marker_state_on_list_items() {
        let structure = build_structure("- [x] done\n- [ ] todo");
        let BlockKind::Container(ContainerBlockKind::List {
            children,
            ..
        }) = &structure.blocks()[0].kind
        else {
            panic!("expected root list");
        };

        let BlockKind::Container(ContainerBlockKind::ListItem {
            is_checked,
            ..
        }) = &children[0].kind
        else {
            panic!("expected list item");
        };
        assert_eq!(*is_checked, Some(true));

        let BlockKind::Container(ContainerBlockKind::ListItem {
            is_checked,
            ..
        }) = &children[1].kind
        else {
            panic!("expected list item");
        };
        assert_eq!(*is_checked, Some(false));
    }

    #[test]
    fn computes_nested_list_item_depth() {
        let structure = build_structure("- parent\n  - child");
        let BlockKind::Container(ContainerBlockKind::List {
            children,
            ..
        }) = &structure.blocks()[0].kind
        else {
            panic!("expected root list");
        };

        let BlockKind::Container(ContainerBlockKind::ListItem {
            depth,
            children,
            ..
        }) = &children[0].kind
        else {
            panic!("expected root item");
        };
        assert_eq!(*depth, 0);

        let nested_list = children
            .iter()
            .find(|b| {
                matches!(
                    b.kind,
                    BlockKind::Container(ContainerBlockKind::List { .. })
                )
            })
            .expect("expected nested list");

        let BlockKind::Container(ContainerBlockKind::List {
            children: nested_items,
            ..
        }) = &nested_list.kind
        else {
            panic!("expected nested list kind");
        };

        let BlockKind::Container(ContainerBlockKind::ListItem {
            depth,
            ..
        }) = &nested_items[0].kind
        else {
            panic!("expected nested item");
        };
        assert_eq!(*depth, 1);
    }

    #[test]
    fn blockquote_nesting_does_not_increment_list_item_depth() {
        let structure = build_structure(
            "- outer\n  > quote\n  > - nested-in-quote\n- sibling",
        );

        let BlockKind::Container(ContainerBlockKind::List {
            children,
            ..
        }) = &structure.blocks()[0].kind
        else {
            panic!("expected root list");
        };

        let BlockKind::Container(ContainerBlockKind::ListItem {
            depth,
            children,
            ..
        }) = &children[0].kind
        else {
            panic!("expected root list item");
        };
        assert_eq!(*depth, 0);

        let quote_block = children
            .iter()
            .find(|block| {
                matches!(
                    block.kind,
                    BlockKind::Container(ContainerBlockKind::BlockQuote { .. })
                )
            })
            .expect("expected blockquote inside first list item");

        let BlockKind::Container(ContainerBlockKind::BlockQuote {
            children,
        }) = &quote_block.kind
        else {
            panic!("expected blockquote container");
        };

        let nested_list = children
            .iter()
            .find(|block| {
                matches!(
                    block.kind,
                    BlockKind::Container(ContainerBlockKind::List { .. })
                )
            })
            .expect("expected nested list under blockquote");

        let BlockKind::Container(ContainerBlockKind::List {
            children,
            ..
        }) = &nested_list.kind
        else {
            panic!("expected list container");
        };

        let BlockKind::Container(ContainerBlockKind::ListItem {
            depth,
            ..
        }) = &children[0].kind
        else {
            panic!("expected nested list item");
        };

        assert_eq!(*depth, 1);
    }

    #[test]
    fn parses_frontmatter_then_content() {
        let structure = build_structure("---\ntags: [a]\n---\n\nBody");
        assert!(matches!(
            structure.blocks()[0].kind,
            BlockKind::Leaf(LeafBlockKind::Frontmatter { .. })
        ));
        assert!(matches!(
            structure.blocks()[1].kind,
            BlockKind::Leaf(LeafBlockKind::Paragraph { .. })
        ));
    }

    #[test]
    fn attach_completed_rejects_leaf_parent_topology() {
        let mut tree = ProcessingBlockTree::new();
        tree.push_incomplete(ProcessingNode::Leaf {
            kind: LeafKind::Paragraph,
            start: 0,
            events: Vec::new(),
        })
        .expect("first push should succeed");

        let span = SourceByteRange::try_from(0..1).expect("valid range");
        let block = Block {
            kind: BlockKind::Leaf(LeafBlockKind::ThematicBreak),
            span,
        };

        let result = tree.attach_completed(block);
        assert!(matches!(
            result,
            Err(NoteIngestError::Parse(NoteParseError::InvalidTopology {
                code: "parser.structure.push_child_to_leaf",
                ..
            }))
        ));
    }

    #[test]
    fn on_end_reports_canonical_underflow_encountered_label() {
        let mut builder = StructureBuilder::new();
        let span = SourceByteRange::try_from(3..4).expect("valid range");

        let result = builder.on_end(BlockEnd::List, span);

        assert!(matches!(
            result,
            Err(NoteIngestError::Parse(NoteParseError::EventStackUnderflow {
                expected: "open block",
                encountered: "list",
                ..
            }))
        ));
    }

    #[test]
    fn on_start_returns_error_when_top_is_leaf() {
        let mut builder = StructureBuilder::new();
        let start = SourceByteRange::try_from(0..1).expect("valid range");
        builder
            .on_start(
                &BlockStart::Heading {
                    level: HeadingLevel::H1,
                },
                start,
            )
            .expect("first start should succeed");

        let result = builder.on_start(&BlockStart::Paragraph, start);

        assert!(matches!(
            result,
            Err(NoteIngestError::Parse(NoteParseError::InvalidTopology {
                code: "parser.structure.start_inside_leaf",
                ..
            }))
        ));
    }

    #[test]
    fn on_end_reports_exact_end_kind_mismatch() {
        let mut builder = StructureBuilder::new();
        let start = SourceByteRange::try_from(0..1).expect("valid range");
        builder
            .on_start(
                &BlockStart::Heading {
                    level: HeadingLevel::H1,
                },
                start,
            )
            .expect("start should succeed");

        let end = SourceByteRange::try_from(2..3).expect("valid range");
        let result = builder.on_end(BlockEnd::Paragraph, end);

        let start_expected =
            SourceByteRange::try_from(0..1).expect("valid range");
        assert!(matches!(
            result,
            Err(NoteIngestError::Parse(NoteParseError::EventStackMismatch {
                expected: "heading",
                found: "paragraph",
                start_range: Some(start_range),
                end_range,
                ..
            })) if start_range == start_expected && end_range == end
        ));
    }

    #[test]
    fn nested_list_item_parent_pos_matches_enclosing_parent_item_pos() {
        let structure = build_structure("- parent\n  - child");

        let BlockKind::Container(ContainerBlockKind::List {
            children,
            ..
        }) = &structure.blocks()[0].kind
        else {
            panic!("expected root list");
        };

        let BlockKind::Container(ContainerBlockKind::ListItem {
            children: parent_children,
            ..
        }) = &children[0].kind
        else {
            panic!("expected parent list item");
        };

        let expected_parent_pos = children[0].span.start();
        let nested_list = parent_children
            .iter()
            .find(|block| {
                matches!(
                    block.kind,
                    BlockKind::Container(ContainerBlockKind::List { .. })
                )
            })
            .expect("expected nested list");

        let BlockKind::Container(ContainerBlockKind::List {
            children: nested_items,
            ..
        }) = &nested_list.kind
        else {
            panic!("expected nested list container");
        };

        let BlockKind::Container(ContainerBlockKind::ListItem {
            parent_pos,
            ..
        }) = &nested_items[0].kind
        else {
            panic!("expected nested list item");
        };

        assert_eq!(*parent_pos, Some(expected_parent_pos));
    }

    #[test]
    fn multi_level_nested_list_items_use_immediate_parent_positions() {
        let structure =
            build_structure("- parent\n  - child\n    - grandchild");

        let BlockKind::Container(ContainerBlockKind::List {
            children,
            ..
        }) = &structure.blocks()[0].kind
        else {
            panic!("expected root list");
        };

        let parent = &children[0];
        let BlockKind::Container(ContainerBlockKind::ListItem {
            children: parent_children,
            ..
        }) = &parent.kind
        else {
            panic!("expected parent list item");
        };

        let child_list = parent_children
            .iter()
            .find(|block| {
                matches!(
                    block.kind,
                    BlockKind::Container(ContainerBlockKind::List { .. })
                )
            })
            .expect("expected child list");

        let BlockKind::Container(ContainerBlockKind::List {
            children: child_items,
            ..
        }) = &child_list.kind
        else {
            panic!("expected child list container");
        };

        let child = &child_items[0];
        let BlockKind::Container(ContainerBlockKind::ListItem {
            parent_pos: child_parent_pos,
            children: child_children,
            ..
        }) = &child.kind
        else {
            panic!("expected child list item");
        };

        let grandchild_list = child_children
            .iter()
            .find(|block| {
                matches!(
                    block.kind,
                    BlockKind::Container(ContainerBlockKind::List { .. })
                )
            })
            .expect("expected grandchild list");

        let BlockKind::Container(ContainerBlockKind::List {
            children: grandchild_items,
            ..
        }) = &grandchild_list.kind
        else {
            panic!("expected grandchild list container");
        };

        let BlockKind::Container(ContainerBlockKind::ListItem {
            parent_pos: grandchild_parent_pos,
            ..
        }) = &grandchild_items[0].kind
        else {
            panic!("expected grandchild list item");
        };

        assert_ne!(parent.span, child.span);
        assert_eq!(*child_parent_pos, Some(parent.span.start()));
        assert_eq!(*grandchild_parent_pos, Some(child.span.start()));
    }

    #[test]
    fn nested_list_item_parent_positions_do_not_leak_across_top_level_branches()
    {
        let structure =
            build_structure("- parent-a\n  - child-a\n- parent-b\n  - child-b");

        let BlockKind::Container(ContainerBlockKind::List {
            children,
            ..
        }) = &structure.blocks()[0].kind
        else {
            panic!("expected root list");
        };

        let parent_a = &children[0];
        let BlockKind::Container(ContainerBlockKind::ListItem {
            children: parent_alpha_children,
            ..
        }) = &parent_a.kind
        else {
            panic!("expected parent-a list item");
        };

        let nested_list_a = parent_alpha_children
            .iter()
            .find(|block| {
                matches!(
                    block.kind,
                    BlockKind::Container(ContainerBlockKind::List { .. })
                )
            })
            .expect("expected nested list under parent-a");

        let BlockKind::Container(ContainerBlockKind::List {
            children: nested_items_a,
            ..
        }) = &nested_list_a.kind
        else {
            panic!("expected nested list container under parent-a");
        };

        let BlockKind::Container(ContainerBlockKind::ListItem {
            parent_pos: child_alpha_parent_pos,
            ..
        }) = &nested_items_a[0].kind
        else {
            panic!("expected child-a list item");
        };

        let parent_b = &children[1];
        let BlockKind::Container(ContainerBlockKind::ListItem {
            children: parent_beta_children,
            ..
        }) = &parent_b.kind
        else {
            panic!("expected parent-b list item");
        };

        let nested_list_b = parent_beta_children
            .iter()
            .find(|block| {
                matches!(
                    block.kind,
                    BlockKind::Container(ContainerBlockKind::List { .. })
                )
            })
            .expect("expected nested list under parent-b");

        let BlockKind::Container(ContainerBlockKind::List {
            children: nested_items_b,
            ..
        }) = &nested_list_b.kind
        else {
            panic!("expected nested list container under parent-b");
        };

        let BlockKind::Container(ContainerBlockKind::ListItem {
            parent_pos: child_beta_parent_pos,
            ..
        }) = &nested_items_b[0].kind
        else {
            panic!("expected child-b list item");
        };

        assert_ne!(parent_a.span, parent_b.span);
        assert_eq!(*child_alpha_parent_pos, Some(parent_a.span.start()));
        assert_eq!(*child_beta_parent_pos, Some(parent_b.span.start()));
    }

    #[test]
    fn sibling_nested_list_items_share_same_enclosing_parent_pos() {
        let structure = build_structure("- parent\n  - child-a\n  - child-b");

        let BlockKind::Container(ContainerBlockKind::List {
            children,
            ..
        }) = &structure.blocks()[0].kind
        else {
            panic!("expected root list");
        };

        let parent = &children[0];
        let BlockKind::Container(ContainerBlockKind::ListItem {
            children: parent_children,
            ..
        }) = &parent.kind
        else {
            panic!("expected parent list item");
        };

        let nested_list = parent_children
            .iter()
            .find(|block| {
                matches!(
                    block.kind,
                    BlockKind::Container(ContainerBlockKind::List { .. })
                )
            })
            .expect("expected nested list");

        let BlockKind::Container(ContainerBlockKind::List {
            children: nested_items,
            ..
        }) = &nested_list.kind
        else {
            panic!("expected nested list container");
        };

        let BlockKind::Container(ContainerBlockKind::ListItem {
            parent_pos: child_alpha_parent_pos,
            ..
        }) = &nested_items[0].kind
        else {
            panic!("expected child-a list item");
        };

        let BlockKind::Container(ContainerBlockKind::ListItem {
            parent_pos: child_beta_parent_pos,
            ..
        }) = &nested_items[1].kind
        else {
            panic!("expected child-b list item");
        };

        assert_eq!(*child_alpha_parent_pos, Some(parent.span.start()));
        assert_eq!(*child_beta_parent_pos, Some(parent.span.start()));
    }

    #[test]
    fn start_block_inside_leaf_returns_invalid_topology() {
        let mut tree = ProcessingBlockTree::new();
        let leaf_start = 0;
        tree.push_incomplete(ProcessingNode::Leaf {
            kind: LeafKind::Paragraph,
            start: leaf_start,
            events: Vec::new(),
        })
        .expect("first push should succeed");

        let result = tree.push_incomplete(ProcessingNode::Leaf {
            kind: LeafKind::Heading(HeadingLevel::H1),
            start: 5,
            events: Vec::new(),
        });

        assert!(matches!(
            result,
            Err(NoteIngestError::Parse(NoteParseError::InvalidTopology {
                code: "parser.structure.start_inside_leaf",
                ..
            }))
        ));
    }

    // ═════════════════════════════════════════════════════════════════════════
    // TRAVERSAL ITERATOR TESTS
    // ═════════════════════════════════════════════════════════════════════════

    #[test]
    fn iter_preorder_handles_empty_document() {
        let structure = build_structure("");
        let count = structure.iter_preorder().count();
        assert_eq!(count, 0, "empty document should have no events");
    }

    #[test]
    fn iter_preorder_emits_enter_only_for_leaf_blocks() {
        let structure = build_structure("# Heading\n\nParagraph");
        let events: Vec<_> = structure.iter_preorder().collect();

        // Should have 2 Enter events, no Exit events (both are leaves)
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], TraversalEvent::Enter(_, 0)));
        assert!(matches!(events[1], TraversalEvent::Enter(_, 0)));
    }

    #[test]
    fn iter_preorder_emits_enter_exit_for_containers() {
        let structure = build_structure("- Item 1\n- Item 2");
        let events: Vec<_> = structure.iter_preorder().collect();

        // Tight list structure:
        // List (Enter) + Item1 (Enter) + Para (Enter) + Item1 (Exit) +
        // Item2 (Enter) + Para (Enter) + Item2 (Exit) + List (Exit)
        // = 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 = 8 events
        assert_eq!(events.len(), 8);

        // First event should be List Enter at depth 0
        assert!(matches!(events[0], TraversalEvent::Enter(_, 0)));
        // Last event should be List Exit at depth 0
        assert!(matches!(events[7], TraversalEvent::Exit(_, 0)));
    }

    #[test]
    fn iter_preorder_tracks_depth_correctly() {
        // Use loose list in blockquote to get depth-2 paragraph
        let structure = build_structure("> - Item\n>\n>   Paragraph");
        let events: Vec<_> = structure.iter_preorder().collect();

        let depths: Vec<u32> = events
            .iter()
            .filter_map(|event| match event {
                TraversalEvent::Enter(_, depth) => Some(*depth),
                TraversalEvent::Exit(..) => None,
            })
            .collect();

        assert!(depths.contains(&0), "should have depth 0 (blockquote)");
        assert!(depths.contains(&1), "should have depth 1 (list/items)");
        assert!(depths.contains(&2), "should have depth 2 (paragraph in item)");
    }

    #[test]
    fn iter_preorder_nested_list_depth_matches_visitor() {
        // This test verifies depth behavior matches visitor:
        // List items do NOT increment depth, only their children do
        //
        // Note: Tight lists (no blank lines) create list items that contain
        // the nested list directly. The structure is:
        // - List (depth 0)
        //   - Item "Item" (depth 0)
        //     - Paragraph "Item" (depth 1)
        //     - List (depth 1)
        //       - Item "Nested" (depth 1)
        //         - Paragraph "Nested" (depth 2)
        let structure = build_structure("- Item\n  - Nested");
        let events: Vec<_> = structure.iter_preorder().collect();

        let enter_depths: Vec<u32> = events
            .iter()
            .filter_map(|event| match event {
                TraversalEvent::Enter(_, depth) => Some(*depth),
                TraversalEvent::Exit(..) => None,
            })
            .collect();

        // Actual structure with tight list:
        // List(0), Item(0), Para(1), List(1), Item(1), Para(2)
        assert_eq!(
            enter_depths,
            vec![0, 0, 1, 1, 1, 2],
            "depths should match actual tight list structure"
        );
    }

    #[test]
    fn iter_preorder_event_order_is_preorder() {
        let structure = build_structure("> Quote\n>\n> - Item");

        let events: Vec<_> = structure.iter_preorder().collect();

        // Pre-order means:
        // 1. Enter BlockQuote
        // 2. Enter Paragraph (Quote text)
        // 3. Enter List
        // 4. Enter ListItem
        // 5. Enter Paragraph (Item text)
        // 6. Exit ListItem
        // 7. Exit List
        // 8. Exit BlockQuote

        assert!(events.len() >= 8);

        // First Enter should be the blockquote
        if let TraversalEvent::Enter(block, depth) = events[0] {
            assert_eq!(depth, 0);
            assert!(matches!(
                block.kind,
                BlockKind::Container(ContainerBlockKind::BlockQuote { .. })
            ));
        } else {
            panic!("First event should be Enter");
        }

        // Find the last event (should be blockquote exit)
        let last_exit = events
            .iter()
            .rev()
            .find(|e| matches!(e, TraversalEvent::Exit(..)))
            .expect("should have at least one exit event");

        if let TraversalEvent::Exit(block, depth) = last_exit {
            assert_eq!(*depth, 0);
            assert!(matches!(
                block.kind,
                BlockKind::Container(ContainerBlockKind::BlockQuote { .. })
            ));
        }
    }

    #[test]
    fn for_each_block_visits_all_blocks() {
        let structure = build_structure("# Heading\n\nParagraph");
        let mut count = 0usize;

        structure.for_each_block(|_block, _depth| {
            count += 1;
        });

        assert_eq!(count, 2usize, "should visit heading and paragraph");
    }

    #[test]
    fn for_each_block_ignores_exit_events() {
        let structure = build_structure("- Item");
        let mut count = 0usize;

        structure.for_each_block(|_block, _depth| {
            count += 1;
        });

        // List + ListItem + Paragraph = 3 blocks (tight list creates paragraph)
        // Should NOT count Exit events
        assert_eq!(count, 3usize, "should count only Enter events");
    }

    #[test]
    fn blocks_preorder_returns_all_blocks() {
        let structure = build_structure("# Heading\n\n- Item 1\n- Item 2");
        let blocks = structure.blocks_preorder();

        // Heading + List + Item1 + Para1 + Item2 + Para2 = 6 blocks
        assert_eq!(blocks.len(), 6);
        assert_eq!(blocks[0].1, 0, "heading at depth 0");
        assert_eq!(blocks[1].1, 0, "list at depth 0");
        assert_eq!(blocks[2].1, 0, "item1 at depth 0");
        assert_eq!(blocks[3].1, 1, "para1 at depth 1");
        assert_eq!(blocks[4].1, 0, "item2 at depth 0");
        assert_eq!(blocks[5].1, 1, "para2 at depth 1");
    }

    #[test]
    fn blocks_preorder_preserves_depth_info() {
        let structure = build_structure("> - Item\n>\n>   Text");
        let blocks = structure.blocks_preorder();

        let depths: Vec<u32> = blocks.iter().map(|(_, depth)| *depth).collect();

        // BlockQuote(0) + List(1) + Item(1) + Para("Item")(2) + Para("Text")(2)
        // Loose list creates separate paragraphs for each text block
        assert_eq!(depths, vec![0, 1, 1, 2, 2]);
    }

    #[test]
    fn iter_preorder_enter_exit_events_are_balanced() {
        let structure =
            build_structure("> - Item 1\n> - Item 2\n>\n> Paragraph");

        let events: Vec<_> = structure.iter_preorder().collect();

        let mut stack = Vec::new();
        for event in &events {
            match event {
                TraversalEvent::Enter(block, _) => {
                    if matches!(block.kind, BlockKind::Container(_)) {
                        stack.push(std::ptr::from_ref(*block));
                    }
                }
                TraversalEvent::Exit(block, _) => {
                    if matches!(block.kind, BlockKind::Container(_)) {
                        let expected =
                            stack.pop().expect("Exit without matching Enter");
                        assert_eq!(
                            std::ptr::from_ref(*block),
                            expected,
                            "Exit event does not match corresponding Enter"
                        );
                    }
                }
            }
        }

        assert!(stack.is_empty(), "Unmatched Enter events remaining");
    }
}
