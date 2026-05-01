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
}
