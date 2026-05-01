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
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Pattern matches borrowed block kinds by design"
    )]
    fn walk_block<V>(block: &Block<'source>, visitor: &mut V, depth: u32)
    where
        V: super::visitor::BlockVisitor<'source>,
    {
        match &block.kind {
            BlockKind::Leaf(LeafBlockKind::Paragraph {
                ..
            }) => {
                visitor.visit_paragraph(block, depth);
            }
            BlockKind::Leaf(LeafBlockKind::Heading {
                level,
                ..
            }) => {
                visitor.visit_heading(block, *level, depth);
            }
            BlockKind::Leaf(LeafBlockKind::CodeBlock {
                language,
                ..
            }) => {
                visitor.visit_code_block(block, language.as_deref(), depth);
            }
            BlockKind::Leaf(LeafBlockKind::Frontmatter {
                format,
                ..
            }) => {
                visitor.visit_frontmatter(block, *format, depth);
            }
            BlockKind::Leaf(LeafBlockKind::ThematicBreak) => {
                visitor.visit_thematic_break(block, depth);
            }
            BlockKind::Container(ContainerBlockKind::BlockQuote {
                children,
            }) => {
                visitor.visit_blockquote(block, depth);
                // Visit children with incremented depth
                for child in children {
                    Self::walk_block(child, visitor, depth.saturating_add(1));
                }
            }
            BlockKind::Container(ContainerBlockKind::List {
                kind,
                children,
            }) => {
                visitor.visit_list(block, *kind, depth);
                // Visit list items (children are ListItem blocks)
                for child in children {
                    Self::walk_block(child, visitor, depth);
                }
            }
            BlockKind::Container(ContainerBlockKind::ListItem {
                is_checked,
                children,
                ..
            }) => {
                visitor.visit_list_item(block, *is_checked, depth);
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
    /// In-progress stack and finalized root block storage.
    tree: ProcessingBlockTree<'source>,
    /// List depth and parent-span bookkeeping.
    list_state: ListNestingState,
}

impl<'source> StructureBuilder<'source> {
    fn new() -> Self {
        Self {
            tree: ProcessingBlockTree::new(),
            list_state: ListNestingState::new(),
        }
    }

    fn process_event(
        &mut self,
        spanned_event: &RangedEvent<'source>,
    ) -> Result<(), NoteIngestError> {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Matching on borrowed enum variant from accessor"
        )]
        match spanned_event.event() {
            ParserEvent::BlockStart(block_type) => {
                self.on_start(block_type, spanned_event.range())?;
            }
            ParserEvent::BlockEnd(block_type) => {
                self.on_end(*block_type, spanned_event.range())?;
            }
            ParserEvent::Inline(_) => {
                self.on_inline_event(spanned_event);
            }
            ParserEvent::TaskListMarker(checked) => {
                self.on_task_marker(*checked);
            }
            ParserEvent::ThematicBreak => {
                // Thematic break (horizontal rule) - standalone block
                let block = Block {
                    kind: BlockKind::Leaf(LeafBlockKind::ThematicBreak),
                    span: spanned_event.range(),
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
        let start = span.start().as_usize();
        match block_type {
            BlockStart::Paragraph => {
                self.push_leaf(ProcessingLeafKind::Paragraph, start)?;
            }
            BlockStart::Heading {
                level,
            } => {
                self.push_leaf(
                    ProcessingLeafKind::Heading {
                        level: *level,
                    },
                    start,
                )?;
            }
            BlockStart::BlockQuote => {
                self.push_container(
                    ProcessingContainer::new_blockquote(),
                    start,
                    None,
                )?;
            }
            BlockStart::CodeBlock {
                info_string,
            } => {
                self.push_leaf(
                    ProcessingLeafKind::CodeBlock {
                        language: info_string.clone().map(Into::into),
                    },
                    start,
                )?;
            }
            BlockStart::List {
                kind: list_kind,
            } => {
                self.push_container(
                    ProcessingContainer::new_list(*list_kind),
                    start,
                    None,
                )?;
                self.list_state.increase_depth();
            }
            BlockStart::ListItem => {
                let parent_span = ListNestingState::parent_span_for_next_item();
                self.push_container(
                    ProcessingContainer::new_list_item(),
                    start,
                    parent_span,
                )?;
            }
            BlockStart::Frontmatter {
                format,
            } => {
                self.push_leaf(
                    ProcessingLeafKind::Frontmatter {
                        format: *format,
                    },
                    start,
                )?;
            }
        }
        Ok(())
    }

    fn push_leaf(
        &mut self,
        kind: ProcessingLeafKind,
        start: usize,
    ) -> Result<(), NoteIngestError> {
        self.tree.push_incomplete(ProcessingNode::Leaf(ProcessingLeaf::new(
            kind, start,
        )))
    }

    fn push_container(
        &mut self,
        container: ProcessingContainer<'source>,
        start: usize,
        parent_span: Option<SourceByteRange>,
    ) -> Result<(), NoteIngestError> {
        self.tree.push_incomplete(ProcessingNode::Container(
            container.with_position(
                start,
                self.list_state.depth(),
                parent_span,
            ),
        ))
    }

    fn on_end(
        &mut self,
        block_type: BlockEnd,
        span: SourceByteRange,
    ) -> Result<(), NoteIngestError> {
        let block = self.tree.finalize_matching(block_type, span)?;

        if matches!(block_type, BlockEnd::List) {
            self.list_state.decrease_depth();
        }

        let mut block = block;
        if matches!(block_type, BlockEnd::ListItem) {
            Self::backfill_nested_list_item_parent_spans(&mut block);
        }

        self.tree.attach_completed(block)?;
        Ok(())
    }

    fn backfill_nested_list_item_parent_spans(block: &mut Block<'source>) {
        let (parent_depth, parent_span, children) = match &mut block.kind {
            BlockKind::Container(ContainerBlockKind::ListItem {
                depth,
                children,
                ..
            }) => (*depth, block.span, children),
            BlockKind::Leaf(_) | BlockKind::Container(_) => return,
        };

        for child in children {
            Self::assign_parent_span_to_matching_descendants(
                child,
                parent_depth.saturating_add(1),
                parent_span,
            );
        }
    }

    fn assign_parent_span_to_matching_descendants(
        block: &mut Block<'source>,
        target_depth: u32,
        parent_span: SourceByteRange,
    ) {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Pattern matching borrowed container variants in-place"
        )]
        match &mut block.kind {
            BlockKind::Container(
                ContainerBlockKind::List {
                    children,
                    ..
                }
                | ContainerBlockKind::BlockQuote {
                    children,
                },
            ) => {
                for child in children {
                    Self::assign_parent_span_to_matching_descendants(
                        child,
                        target_depth,
                        parent_span,
                    );
                }
            }
            BlockKind::Container(ContainerBlockKind::ListItem {
                depth,
                parent_span: child_parent_span,
                children,
                ..
            }) => {
                if *depth == target_depth {
                    *child_parent_span = Some(parent_span);
                }
                for child in children {
                    Self::assign_parent_span_to_matching_descendants(
                        child,
                        target_depth,
                        parent_span,
                    );
                }
            }
            BlockKind::Leaf(_) => {}
        }
    }

    fn on_inline_event(&mut self, event: &RangedEvent<'source>) {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Match ergonomics on mutable borrowed stack node"
        )]
        if let Some(ProcessingNode::Leaf(current)) = self.tree.last_mut() {
            current.push_event(event.clone());
        }
    }

    fn on_task_marker(&mut self, checked: bool) {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Match ergonomics on mutable borrowed stack node"
        )]
        if let Some(ProcessingNode::Container(current)) = self.tree.last_mut() {
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

struct ListNestingState {
    depth: u32,
}

impl ListNestingState {
    const fn new() -> Self {
        Self {
            depth: 0,
        }
    }

    const fn depth(&self) -> u32 {
        self.depth
    }

    fn increase_depth(&mut self) {
        self.depth = self.depth.saturating_add(1);
    }

    fn decrease_depth(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    fn parent_span_for_next_item() -> Option<SourceByteRange> {
        None
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
        if let Some(ProcessingNode::Leaf(_)) = self.stack.last() {
            let start = match &node {
                ProcessingNode::Leaf(leaf) => leaf.start,
                ProcessingNode::Container(container) => container.start(),
            };

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

    fn last_mut(&mut self) -> Option<&mut ProcessingNode<'source>> {
        self.stack.last_mut()
    }

    fn attach_completed(
        &mut self,
        block: Block<'source>,
    ) -> Result<(), NoteIngestError> {
        if let Some(parent) = self.stack.last_mut() {
            #[expect(
                clippy::pattern_type_mismatch,
                reason = "Match ergonomics on mutable borrowed stack node"
            )]
            match parent {
                ProcessingNode::Container(container) => {
                    container.push_child(block);
                    Ok(())
                }
                ProcessingNode::Leaf(_) => {
                    Err(NoteParseError::InvalidTopology {
                        code: "parser.structure.attach_to_leaf",
                        detail: "attempted to attach completed block under \
                                 leaf"
                            .into(),
                        range: Some(block.span),
                    }
                    .into())
                }
            }
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

impl<'source> ProcessingNode<'source> {
    const fn kind_name(&self) -> &'static str {
        match self {
            Self::Leaf(_) => "leaf",
            Self::Container(_) => "container",
        }
    }

    const fn expected_end(&self) -> BlockEnd {
        match self {
            Self::Leaf(leaf) => leaf.expected_end(),
            Self::Container(container) => container.expected_end(),
        }
    }

    fn start_anchor_range(&self) -> Option<SourceByteRange> {
        let start = match self {
            Self::Leaf(leaf) => leaf.start,
            Self::Container(container) => container.start(),
        };

        SourceByteRange::try_from(start..start.saturating_add(1)).ok()
    }

    fn finalize(self, end: usize) -> Result<Block<'source>, NoteIngestError> {
        match self {
            Self::Leaf(leaf) => leaf.finalize(end),
            Self::Container(container) => container.finalize(end),
        }
    }
}

/// Temporary state for a leaf block being constructed.
struct ProcessingLeaf<'source> {
    kind: ProcessingLeafKind,
    start: usize,
    events: Vec<RangedEvent<'source>>,
}

impl<'source> ProcessingLeaf<'source> {
    fn new(kind: ProcessingLeafKind, start: usize) -> Self {
        Self {
            kind,
            start,
            events: Vec::new(),
        }
    }

    fn push_event(&mut self, event: RangedEvent<'source>) {
        self.events.push(event);
    }

    const fn expected_end(&self) -> BlockEnd {
        match self.kind {
            ProcessingLeafKind::Paragraph => BlockEnd::Paragraph,
            ProcessingLeafKind::Heading {
                ..
            } => BlockEnd::Heading,
            ProcessingLeafKind::CodeBlock {
                ..
            } => BlockEnd::CodeBlock,
            ProcessingLeafKind::Frontmatter {
                ..
            } => BlockEnd::Frontmatter,
        }
    }

    fn finalize(self, end: usize) -> Result<Block<'source>, NoteIngestError> {
        let span = SourceByteRange::try_from(self.start..end)
            .map_err(NoteIngestError::Domain)?;

        let text = TextSequence::from_events(&self.events).as_plain_text();

        let kind = match self.kind {
            ProcessingLeafKind::Paragraph => LeafBlockKind::Paragraph {
                events: self.events,
            },
            ProcessingLeafKind::Heading {
                level,
            } => LeafBlockKind::Heading {
                level,
                events: self.events,
            },
            ProcessingLeafKind::CodeBlock {
                language,
            } => LeafBlockKind::CodeBlock {
                language,
                text,
            },
            ProcessingLeafKind::Frontmatter {
                format,
            } => LeafBlockKind::Frontmatter {
                format,
                text,
            },
        };

        Ok(Block {
            kind: BlockKind::Leaf(kind),
            span,
        })
    }
}

/// Temporary state for a container block being constructed.
struct ProcessingContainer<'source> {
    kind: ProcessingContainerKind<'source>,
    start: usize,
}

impl<'source> ProcessingContainer<'source> {
    fn new_blockquote() -> Self {
        Self {
            kind: ProcessingContainerKind::BlockQuote(BlockQuotePayload {
                children: Vec::new(),
            }),
            start: 0,
        }
    }

    fn new_list(kind: ListKind) -> Self {
        Self {
            kind: ProcessingContainerKind::List(ListPayload {
                kind,
                children: Vec::new(),
            }),
            start: 0,
        }
    }

    fn new_list_item() -> Self {
        Self {
            kind: ProcessingContainerKind::ListItem(ListItemPayload {
                depth: 0,
                parent_span: None,
                is_checked: None,
                children: Vec::new(),
            }),
            start: 0,
        }
    }

    fn with_position(
        mut self,
        start: usize,
        depth: u32,
        parent_span: Option<SourceByteRange>,
    ) -> Self {
        self.start = start;
        if let ProcessingContainerKind::ListItem(list_item) = &mut self.kind {
            list_item.depth = depth;
            list_item.parent_span = parent_span;
        }
        self
    }

    fn push_child(&mut self, child: Block<'source>) {
        match &mut self.kind {
            ProcessingContainerKind::BlockQuote(blockquote) => {
                blockquote.children.push(child);
            }
            ProcessingContainerKind::List(list) => list.children.push(child),
            ProcessingContainerKind::ListItem(list_item) => {
                list_item.children.push(child);
            }
        }
    }

    fn set_task_marker(&mut self, checked: bool) {
        if let ProcessingContainerKind::ListItem(list_item) = &mut self.kind {
            list_item.is_checked = Some(checked);
        }
    }

    const fn expected_end(&self) -> BlockEnd {
        match &self.kind {
            ProcessingContainerKind::BlockQuote(_) => BlockEnd::BlockQuote,
            ProcessingContainerKind::List(_) => BlockEnd::List,
            ProcessingContainerKind::ListItem(_) => BlockEnd::ListItem,
        }
    }

    const fn start(&self) -> usize {
        self.start
    }

    fn finalize(self, end: usize) -> Result<Block<'source>, NoteIngestError> {
        let kind = match self.kind {
            ProcessingContainerKind::BlockQuote(blockquote) => {
                ContainerBlockKind::BlockQuote {
                    children: blockquote.children,
                }
            }
            ProcessingContainerKind::List(list) => ContainerBlockKind::List {
                kind: list.kind,
                children: list.children,
            },
            ProcessingContainerKind::ListItem(list_item) => {
                ContainerBlockKind::ListItem {
                    depth: list_item.depth.saturating_sub(1),
                    parent_span: list_item.parent_span,
                    is_checked: list_item.is_checked,
                    children: list_item.children,
                }
            }
        };

        let span = SourceByteRange::try_from(self.start..end)
            .map_err(NoteIngestError::Domain)?;

        Ok(Block {
            kind: BlockKind::Container(kind),
            span,
        })
    }
}

enum ProcessingContainerKind<'source> {
    BlockQuote(BlockQuotePayload<'source>),
    List(ListPayload<'source>),
    ListItem(ListItemPayload<'source>),
}

struct BlockQuotePayload<'source> {
    children: Vec<Block<'source>>,
}

struct ListPayload<'source> {
    kind: ListKind,
    children: Vec<Block<'source>>,
}

struct ListItemPayload<'source> {
    depth: u32,
    parent_span: Option<SourceByteRange>,
    is_checked: Option<bool>,
    children: Vec<Block<'source>>,
}

enum ProcessingNode<'source> {
    Leaf(ProcessingLeaf<'source>),
    Container(ProcessingContainer<'source>),
}

enum ProcessingLeafKind {
    Paragraph,
    Heading {
        level: HeadingLevel,
    },
    CodeBlock {
        language: Option<Box<str>>,
    },
    Frontmatter {
        format: FrontmatterFormat,
    },
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
        tree.push_incomplete(ProcessingNode::Leaf(ProcessingLeaf::new(
            ProcessingLeafKind::Paragraph,
            0,
        )))
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
                code: "parser.structure.attach_to_leaf",
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
            .on_start(&BlockStart::Paragraph, start)
            .expect("first start should succeed");

        let result = builder.on_start(
            &BlockStart::Heading {
                level: HeadingLevel::H1,
            },
            start,
        );

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
            .on_start(&BlockStart::Paragraph, start)
            .expect("start should succeed");

        let end = SourceByteRange::try_from(2..3).expect("valid range");
        let result = builder.on_end(BlockEnd::Heading, end);

        let start_expected =
            SourceByteRange::try_from(0..1).expect("valid range");
        assert!(matches!(
            result,
            Err(NoteIngestError::Parse(NoteParseError::EventStackMismatch {
                expected: "paragraph",
                found: "heading",
                start_range: Some(start_range),
                end_range,
                ..
            })) if start_range == start_expected && end_range == end
        ));
    }

    #[test]
    fn nested_list_item_parent_span_matches_enclosing_parent_item_span() {
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

        let expected_parent_span = children[0].span;
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
            parent_span,
            ..
        }) = &nested_items[0].kind
        else {
            panic!("expected nested list item");
        };

        assert_eq!(*parent_span, Some(expected_parent_span));
    }

    #[test]
    fn multi_level_nested_list_items_use_immediate_parent_spans() {
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
            parent_span: child_parent_span,
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
            parent_span: grandchild_parent_span,
            ..
        }) = &grandchild_items[0].kind
        else {
            panic!("expected grandchild list item");
        };

        assert_ne!(parent.span, child.span);
        assert_eq!(*child_parent_span, Some(parent.span));
        assert_eq!(*grandchild_parent_span, Some(child.span));
    }

    #[test]
    fn nested_list_item_parent_spans_do_not_leak_across_top_level_branches() {
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
            parent_span: child_alpha_parent_span,
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
            parent_span: child_beta_parent_span,
            ..
        }) = &nested_items_b[0].kind
        else {
            panic!("expected child-b list item");
        };

        assert_ne!(parent_a.span, parent_b.span);
        assert_eq!(*child_alpha_parent_span, Some(parent_a.span));
        assert_eq!(*child_beta_parent_span, Some(parent_b.span));
    }

    #[test]
    fn sibling_nested_list_items_share_same_enclosing_parent_span() {
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
            parent_span: child_alpha_parent_span,
            ..
        }) = &nested_items[0].kind
        else {
            panic!("expected child-a list item");
        };

        let BlockKind::Container(ContainerBlockKind::ListItem {
            parent_span: child_beta_parent_span,
            ..
        }) = &nested_items[1].kind
        else {
            panic!("expected child-b list item");
        };

        assert_eq!(*child_alpha_parent_span, Some(parent.span));
        assert_eq!(*child_beta_parent_span, Some(parent.span));
    }

    #[test]
    fn start_block_inside_leaf_returns_invalid_topology() {
        let mut tree = ProcessingBlockTree::new();
        let leaf_start = 0;
        tree.push_incomplete(ProcessingNode::Leaf(ProcessingLeaf::new(
            ProcessingLeafKind::Paragraph,
            leaf_start,
        )))
        .expect("first push should succeed");

        let result =
            tree.push_incomplete(ProcessingNode::Leaf(ProcessingLeaf::new(
                ProcessingLeafKind::Heading {
                    level: HeadingLevel::H1,
                },
                5,
            )));

        assert!(matches!(
            result,
            Err(NoteIngestError::Parse(NoteParseError::InvalidTopology {
                code: "parser.structure.start_inside_leaf",
                ..
            }))
        ));
    }
}
