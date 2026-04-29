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

use pulldown_cmark::{CowStr, MetadataBlockKind};

use super::{
    block::{
        Block, BlockKind, ContainerBlockKind, HeadingLevel, LeafBlockKind,
        ListKind, inline_events_text,
    },
    context::ParserContext,
    stream::{BlockType, EventWithRange, ParserEvent},
};
use crate::note::{
    error::{NoteIngestError, NoteParseError},
    position::SourceByteRange,
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
                visitor.visit_code_block(block, language.as_ref(), depth);
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
        spanned_event: &EventWithRange<'source>,
    ) -> Result<(), NoteIngestError> {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Matching on borrowed enum variant from accessor"
        )]
        match spanned_event.event() {
            ParserEvent::BlockStart(block_type) => {
                self.on_start(block_type, spanned_event.range());
            }
            ParserEvent::BlockEnd(block_type) => {
                self.on_end(block_type, spanned_event.range())?;
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
                self.tree.attach_completed(block);
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
        block_type: &BlockType<'source>,
        span: SourceByteRange,
    ) {
        let start = span.start().as_usize();
        match block_type {
            BlockType::Paragraph => {
                self.push_leaf(ProcessingLeafKind::Paragraph, start);
            }
            BlockType::Heading {
                level,
            } => {
                self.push_leaf(
                    ProcessingLeafKind::Heading {
                        level: (*level).into(),
                    },
                    start,
                );
            }
            BlockType::BlockQuote => {
                self.push_container(
                    ProcessingContainerKind::BlockQuote,
                    start,
                    None,
                );
                self.list_state.increase_depth();
            }
            BlockType::CodeBlock {
                language,
            } => {
                let language = language.as_ref().map(ToString::to_string);
                self.push_leaf(
                    ProcessingLeafKind::CodeBlock {
                        language,
                    },
                    start,
                );
            }
            BlockType::List {
                start: list_start,
            } => {
                let kind = match list_start {
                    Some(n) => ListKind::Ordered {
                        start: *n,
                    },
                    None => ListKind::Unordered,
                };
                self.push_container(
                    ProcessingContainerKind::List {
                        kind,
                    },
                    start,
                    None,
                );
                self.list_state.increase_depth();
            }
            BlockType::Item => {
                let parent_span = self.list_state.parent_span_for_next_item();
                self.push_container(
                    ProcessingContainerKind::ListItem,
                    start,
                    parent_span,
                );
            }
            BlockType::Frontmatter {
                format: kind,
            } => {
                self.push_leaf(
                    ProcessingLeafKind::Frontmatter {
                        format: *kind,
                    },
                    start,
                );
            }
        }
    }

    fn push_leaf(&mut self, kind: ProcessingLeafKind, start: usize) {
        self.tree.push_incomplete(ProcessingNode::Leaf(ProcessingLeaf::new(
            kind, start,
        )));
    }

    fn push_container(
        &mut self,
        kind: ProcessingContainerKind,
        start: usize,
        parent_span: Option<SourceByteRange>,
    ) {
        self.tree.push_incomplete(ProcessingNode::Container(
            ProcessingContainer::new(
                kind,
                start,
                self.list_state.depth(),
                parent_span,
            ),
        ));
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Pattern matches borrowed block tags by design"
    )]
    fn on_end(
        &mut self,
        block_type: &BlockType<'source>,
        span: SourceByteRange,
    ) -> Result<(), NoteIngestError> {
        match block_type {
            BlockType::Paragraph
            | BlockType::Heading {
                ..
            }
            | BlockType::CodeBlock {
                ..
            }
            | BlockType::Frontmatter {
                ..
            } => {
                self.finalize_and_attach_leaf(
                    span,
                    "stack underflow: End tag without matching Start",
                )?;
            }
            BlockType::BlockQuote => {
                self.finalize_and_attach_container(
                    span,
                    "stack underflow: End BlockQuote without Start",
                )?;
                self.list_state.decrease_depth();
            }
            BlockType::List {
                ..
            } => {
                self.finalize_and_attach_container(
                    span,
                    "stack underflow: End List without Start",
                )?;
                self.list_state.decrease_depth();
            }
            BlockType::Item => {
                let block = self.finalize_container(
                    span,
                    "stack underflow: End Item without Start",
                )?;
                self.list_state.record_item_parent(block.span);
                self.tree.attach_completed(block);
            }
        }
        Ok(())
    }

    fn on_inline_event(&mut self, event: &EventWithRange<'source>) {
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
            return Err(NoteParseError::Markdown {
                line: 0,
                column: 0,
                reason: "unclosed blocks at end of document".into(),
            }
            .into());
        }
        Ok(self.tree.into_roots())
    }

    fn pop_incomplete_or(
        &mut self,
        reason: &'static str,
    ) -> Result<ProcessingNode<'source>, NoteIngestError> {
        self.tree.pop_incomplete().ok_or_else(|| {
            NoteParseError::Markdown {
                line: 0,
                column: 0,
                reason: reason.into(),
            }
            .into()
        })
    }

    fn finalize_leaf(
        &mut self,
        span: SourceByteRange,
        underflow_reason: &'static str,
    ) -> Result<Block<'source>, NoteIngestError> {
        let processing = self.pop_incomplete_or(underflow_reason)?;
        match processing {
            ProcessingNode::Leaf(leaf) => leaf.finalize(span.end().as_usize()),
            ProcessingNode::Container(_) => Err(Self::role_mismatch_error()),
        }
    }

    fn finalize_and_attach_leaf(
        &mut self,
        span: SourceByteRange,
        underflow_reason: &'static str,
    ) -> Result<(), NoteIngestError> {
        let block = self.finalize_leaf(span, underflow_reason)?;
        self.tree.attach_completed(block);
        Ok(())
    }

    fn finalize_container(
        &mut self,
        span: SourceByteRange,
        underflow_reason: &'static str,
    ) -> Result<Block<'source>, NoteIngestError> {
        let processing = self.pop_incomplete_or(underflow_reason)?;
        match processing {
            ProcessingNode::Container(container) => {
                container.finalize(span.end().as_usize())
            }
            ProcessingNode::Leaf(_) => Err(Self::role_mismatch_error()),
        }
    }

    fn finalize_and_attach_container(
        &mut self,
        span: SourceByteRange,
        underflow_reason: &'static str,
    ) -> Result<(), NoteIngestError> {
        let block = self.finalize_container(span, underflow_reason)?;
        self.tree.attach_completed(block);
        Ok(())
    }

    fn role_mismatch_error() -> NoteIngestError {
        NoteParseError::Markdown {
            line: 0,
            column: 0,
            reason: "builder role mismatch during block finalization".into(),
        }
        .into()
    }
}

struct ListNestingState {
    depth: u32,
    list_item_parents: Vec<SourceByteRange>,
}

impl ListNestingState {
    const fn new() -> Self {
        Self {
            depth: 0,
            list_item_parents: Vec::new(),
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

    fn parent_span_for_next_item(&self) -> Option<SourceByteRange> {
        if self.depth > 1 {
            let parent_depth_index =
                usize::try_from(self.depth).unwrap_or(0).saturating_sub(2);
            self.list_item_parents.get(parent_depth_index).copied()
        } else {
            None
        }
    }

    fn record_item_parent(&mut self, span: SourceByteRange) {
        let depth_index =
            usize::try_from(self.depth).unwrap_or(0).saturating_sub(1);
        if self.list_item_parents.len() <= depth_index {
            self.list_item_parents.resize(depth_index.saturating_add(1), span);
        }
        if let Some(slot) = self.list_item_parents.get_mut(depth_index) {
            *slot = span;
        }
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

    fn push_incomplete(&mut self, node: ProcessingNode<'source>) {
        self.stack.push(node);
    }

    fn pop_incomplete(&mut self) -> Option<ProcessingNode<'source>> {
        self.stack.pop()
    }

    fn last_mut(&mut self) -> Option<&mut ProcessingNode<'source>> {
        self.stack.last_mut()
    }

    fn attach_completed(&mut self, block: Block<'source>) {
        if let Some(parent) = self.stack.last_mut() {
            #[expect(
                clippy::pattern_type_mismatch,
                reason = "Match ergonomics on mutable borrowed stack node"
            )]
            match parent {
                ProcessingNode::Container(container) => {
                    container.push_child(block);
                }
                ProcessingNode::Leaf(_) => {
                    // This indicates an invalid nesting logic in the builder
                    // but we can't easily recover here without changing
                    // signatures We shouldn't hit this if
                    // the markdown structure is valid
                }
            }
        } else {
            self.root_blocks.push(block);
        }
    }

    fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    fn into_roots(self) -> Vec<Block<'source>> {
        self.root_blocks
    }
}

/// Temporary state for a leaf block being constructed.
struct ProcessingLeaf<'source> {
    kind: ProcessingLeafKind,
    start: usize,
    events: Vec<EventWithRange<'source>>,
}

impl<'source> ProcessingLeaf<'source> {
    fn new(kind: ProcessingLeafKind, start: usize) -> Self {
        Self {
            kind,
            start,
            events: Vec::new(),
        }
    }

    fn push_event(&mut self, event: EventWithRange<'source>) {
        self.events.push(event);
    }

    fn finalize(self, end: usize) -> Result<Block<'source>, NoteIngestError> {
        let span = SourceByteRange::try_from(self.start..end)
            .map_err(NoteIngestError::Domain)?;

        let text = inline_events_text(&self.events);

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
                language: language.map(|s| CowStr::Boxed(s.into_boxed_str())),
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
    kind: ProcessingContainerKind,
    start: usize,
    children: Vec<Block<'source>>,
    depth: u32,
    is_checked: Option<bool>,
    parent_span: Option<SourceByteRange>,
}

impl<'source> ProcessingContainer<'source> {
    fn new(
        kind: ProcessingContainerKind,
        start: usize,
        depth: u32,
        parent_span: Option<SourceByteRange>,
    ) -> Self {
        Self {
            kind,
            start,
            children: Vec::new(),
            depth,
            is_checked: None,
            parent_span,
        }
    }

    fn push_child(&mut self, child: Block<'source>) {
        self.children.push(child);
    }

    fn set_task_marker(&mut self, checked: bool) {
        if matches!(self.kind, ProcessingContainerKind::ListItem) {
            self.is_checked = Some(checked);
        }
    }

    fn finalize(self, end: usize) -> Result<Block<'source>, NoteIngestError> {
        let span = SourceByteRange::try_from(self.start..end)
            .map_err(NoteIngestError::Domain)?;

        let kind = match self.kind {
            ProcessingContainerKind::BlockQuote => {
                ContainerBlockKind::BlockQuote {
                    children: self.children,
                }
            }
            ProcessingContainerKind::List {
                kind,
            } => ContainerBlockKind::List {
                kind,
                children: self.children,
            },
            ProcessingContainerKind::ListItem => ContainerBlockKind::ListItem {
                depth: self.depth.saturating_sub(1),
                parent_span: self.parent_span,
                is_checked: self.is_checked,
                children: self.children,
            },
        };

        Ok(Block {
            kind: BlockKind::Container(kind),
            span,
        })
    }
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
        language: Option<String>,
    },
    Frontmatter {
        format: MetadataBlockKind,
    },
}

enum ProcessingContainerKind {
    BlockQuote,
    List {
        kind: ListKind,
    },
    ListItem,
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
}
