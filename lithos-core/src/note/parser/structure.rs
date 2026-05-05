//! Hierarchical document structure (AST).
//!
//! This module provides the [`DocTree`], which represents a fully parsed
//! markdown document as a tree of blocks. It handles the transition from
//! a linear event stream to a nested structure, enforcing CommonMark
//! nesting rules and tracking block depth.

#![cfg_attr(
    not(test),
    expect(dead_code, reason = "Structure builder is consumed incrementally")
)]
#![expect(
    clippy::pattern_type_mismatch,
    reason = "Parser stack code intentionally matches borrowed enum shapes"
)]

use super::{
    block::{
        Block, BlockKind, Closed, ContainerBlockKind, LeafBlockKind, Open,
    },
    context::ParserContext,
    types::{BlockEnd, BlockStart, ParserEvent, RangedEvent},
};
use crate::note::{
    error::{NoteIngestError, NoteParseError},
    position::{SourceByteOffset, SourceByteRange},
};

// ----------------------------------------------------------- //
//                        State Markers                        //
// ----------------------------------------------------------- //

/// Trait for the document tree's lifecycle state.
pub trait DocState: std::fmt::Debug {}

/// Marker for a document tree currently being assembled.
#[derive(Debug, Default)]
pub(crate) struct Processing<'source> {
    /// In-progress stack of open blocks.
    stack: Vec<Block<'source, Open>>,
    /// List depth and parent-position bookkeeping.
    open_items: Vec<SourceByteOffset>,
    /// The end offset of the last processed event.
    last_end: SourceByteOffset,
}
impl DocState for Processing<'_> {}

/// Marker for a finalized, read-only document tree.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Complete;
impl DocState for Complete {}

// ----------------------------------------------------------- //
//                   `Doctree` Document AST                    //
// ----------------------------------------------------------- //

/// The hierarchical document structure (AST) for a markdown document.
#[derive(Clone, Debug, PartialEq)]
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "pub(crate) fields follow project convention for internal APIs"
)]
pub struct DocTree<'source, S: DocState = Complete> {
    /// Root-level blocks in the document.
    pub(crate) blocks: Vec<Block<'source, Closed>>,
    /// State-specific data.
    pub(crate) state: S,
}

// ----------------------------------------------------------- //
//                    Complete Document API                    //
// ----------------------------------------------------------- //

impl<'source> DocTree<'source, Complete> {
    /// Build the document structure from a parser context.
    pub(crate) fn from_context(
        ctx: &ParserContext<'source>,
    ) -> Result<Self, NoteIngestError> {
        let mut tree = DocTree::<Processing<'source>>::new();

        for event in ctx.events().iter().cloned() {
            tree.process_event(event)?;
        }

        tree.finish()
    }

    /// Returns a borrowed slice of the root-level blocks.
    #[must_use]
    #[inline]
    pub(crate) fn blocks(&self) -> &[Block<'source, Closed>] {
        &self.blocks
    }

    /// Returns an iterator over the document blocks in pre-order.
    #[inline]
    #[must_use]
    pub(crate) fn iter_preorder(&self) -> PreorderIter<'_, 'source> {
        PreorderIter::new(&self.blocks)
    }

    /// Execute a callback for each block in pre-order traversal.
    pub(crate) fn for_each_block<F>(&self, mut f: F)
    where
        F: FnMut(&Block<'source, Closed>, u32),
    {
        for event in self.iter_preorder() {
            if let TraversalEvent::Enter(block, depth) = event {
                f(block, depth);
            }
        }
    }

    /// Collect all blocks in pre-order traversal.
    #[must_use]
    pub(crate) fn blocks_preorder(
        &self,
    ) -> Vec<(&Block<'source, Closed>, u32)> {
        self.iter_preorder()
            .filter_map(|event| match event {
                TraversalEvent::Enter(block, depth) => Some((block, depth)),
                TraversalEvent::Exit(..) => None,
            })
            .collect()
    }
}

// ----------------------------------------------------------- //
//                   Processing Document API                   //
// ----------------------------------------------------------- //
impl<'source> DocTree<'source, Processing<'source>> {
    fn new() -> Self {
        Self {
            blocks: Vec::new(),
            state: Processing::default(),
        }
    }

    /// Processes a single ranged event, driving the state machine forward.
    ///
    /// This is the heart of the structure builder. it converts linear events
    /// into a nested tree by maintaining a stack of open blocks and applying
    /// auto-closing rules for paragraphs and lists.
    fn process_event(
        &mut self,
        event: RangedEvent<'source>,
    ) -> Result<(), NoteIngestError> {
        let (event, range) = event.into_parts();
        self.state.last_end = range.end();

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
                self.auto_close_implicit_paragraph(range.clone())?;
                let block = Block {
                    kind: BlockKind::Leaf(LeafBlockKind::ThematicBreak),
                    span: range.start(),
                };
                let closed = block.close(range.end())?;
                self.attach_completed(closed)?;
            }
        }
        Ok(())
    }

    /// Finalizes the document assembly.
    ///
    /// Closes any remaining open blocks (like a trailing paragraph) and
    /// validates that the stack is empty.
    fn finish(self) -> Result<DocTree<'source, Complete>, NoteIngestError> {
        let mut this = self;

        // Auto-close trailing implicit paragraph
        if let Some(top) = this.state.stack.last()
            && top.kind.expected_end() == BlockEnd::Paragraph
        {
            let last_end = this.state.last_end;
            let range =
                SourceByteRange::new(top.span, last_end).map_err(|e| {
                    NoteParseError::InvalidTopology {
                        code: "parser.structure.invalid_paragraph_range",
                        detail: format!(
                            "failed to construct range for trailing \
                             paragraph: {e}"
                        )
                        .into(),
                        range: None,
                    }
                })?;

            this.on_end(BlockEnd::Paragraph, range)?;
        }

        if let Some(top) = this.state.stack.last() {
            return Err(NoteParseError::UnclosedBlocks {
                open_count: this.state.stack.len(),
                top_kind: Some(match top.kind {
                    BlockKind::Leaf(_) => "leaf",
                    BlockKind::Container(_) => "container",
                }),
                at: top.span,
            }
            .into());
        }

        Ok(DocTree {
            blocks: this.blocks,
            state: Complete,
        })
    }

    // ------------------- Event Handlers -------------------- //

    /// Handles the start of a new block element.
    ///
    /// # Invariants
    ///
    /// - **Implicit Paragraphs**: If a paragraph is open, it is auto-closed
    ///   before starting any new block (except for thematic breaks which handle
    ///   this themselves).
    /// - **Leaf Isolation**: A new block cannot be started inside an existing
    ///   leaf block (e.g., you cannot start a list inside a heading).
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Pattern matches borrowed block tags by design"
    )]
    fn on_start(
        &mut self,
        block_type: &BlockStart<'source>,
        span: SourceByteRange,
    ) -> Result<(), NoteIngestError> {
        self.auto_close_implicit_paragraph(span.clone())?;

        if let Some(top) = self.state.stack.last()
            && matches!(top.kind, BlockKind::Leaf(_))
        {
            return Err(NoteParseError::InvalidTopology {
                code: "parser.structure.start_inside_leaf",
                detail: "cannot start a new block inside a leaf block".into(),
                range: Some(span),
            }
            .into());
        }

        let kind = match block_type {
            BlockStart::Paragraph => {
                BlockKind::Leaf(LeafBlockKind::Paragraph {
                    events: Vec::new(),
                })
            }
            BlockStart::Heading {
                level,
            } => BlockKind::Leaf(LeafBlockKind::Heading {
                level: *level,
                events: Vec::new(),
            }),
            BlockStart::BlockQuote => {
                BlockKind::Container(ContainerBlockKind::BlockQuote {
                    children: Vec::new(),
                })
            }
            BlockStart::CodeBlock {
                info_string,
            } => BlockKind::Leaf(LeafBlockKind::CodeBlock {
                language: info_string.clone().map(Into::into),
                text: Vec::new(),
            }),
            BlockStart::List {
                kind: list_kind,
            } => BlockKind::Container(ContainerBlockKind::List {
                kind: *list_kind,
                children: Vec::new(),
            }),
            BlockStart::ListItem => {
                let parent_pos = self.state.open_items.last().copied();
                let depth = u32::try_from(self.state.open_items.len())
                    .unwrap_or(u32::MAX);

                BlockKind::Container(ContainerBlockKind::ListItem {
                    depth,
                    parent_pos,
                    is_checked: None,
                    children: Vec::new(),
                })
            }
            BlockStart::Frontmatter {
                format,
            } => BlockKind::Leaf(LeafBlockKind::Frontmatter {
                format: *format,
                text: Vec::new(),
            }),
        };

        if matches!(block_type, BlockStart::ListItem) {
            self.state.open_items.push(span.start());
        }

        self.state.stack.push(Block {
            kind,
            span: span.start(),
        });
        Ok(())
    }

    /// Handles the end of a block element.
    ///
    /// Pops the top block from the stack, validates that it matches the
    /// expected `block_type`, and attaches it to its parent or the root.
    fn on_end(
        &mut self,
        block_type: BlockEnd,
        span: SourceByteRange,
    ) -> Result<(), NoteIngestError> {
        if block_type != BlockEnd::Paragraph {
            self.auto_close_implicit_paragraph(span.clone())?;
        }

        let processing = self.state.stack.pop().ok_or_else(|| {
            NoteParseError::EventStackUnderflow {
                expected: "open block",
                encountered: block_type.label(),
                depth: self.state.stack.len(),
                range: span.clone(),
            }
        })?;

        if processing.kind.expected_end() != block_type {
            let start = processing.span;
            let start_range =
                SourceByteRange::new(start, start.saturating_add(1u32.into()))
                    .ok();

            return Err(NoteParseError::EventStackMismatch {
                expected: processing.kind.expected_end().label(),
                found: block_type.label(),
                depth: self.state.stack.len(),
                start_range,
                end_range: span,
            }
            .into());
        }

        if matches!(block_type, BlockEnd::ListItem) {
            self.state.open_items.pop();
        }

        let block = processing.close(span.end())?;
        self.attach_completed(block)?;
        Ok(())
    }

    /// Handles inline events (text, code, math, etc.).
    ///
    /// If no leaf block is currently open, an implicit paragraph is
    /// automatically started to contain the inline content.
    fn on_inline_event(
        &mut self,
        event: RangedEvent<'source>,
    ) -> Result<(), NoteIngestError> {
        let needs_paragraph = match self.state.stack.last() {
            Some(block) => matches!(block.kind, BlockKind::Container(_)),
            None => true,
        };

        if needs_paragraph {
            self.on_start(&BlockStart::Paragraph, event.range())?;
        }

        let current = self.state.stack.last_mut().ok_or_else(|| {
            NoteParseError::InvalidTopology {
                code: "parser.structure.inline_outside_leaf",
                detail: "inline event encountered outside of a leaf block"
                    .into(),
                range: Some(event.range()),
            }
        })?;

        current.kind.push_inline_event(event).map_err(Into::into)
    }

    /// Automatically closes an open paragraph if it exists.
    ///
    /// In `CommonMark`, paragraphs do not always have explicit closing tags in
    /// the event stream (e.g., when followed immediately by a heading or list).
    /// This helper ensures they are finalized correctly.
    fn auto_close_implicit_paragraph(
        &mut self,
        span: SourceByteRange,
    ) -> Result<(), NoteIngestError> {
        if let Some(top) = self.state.stack.last()
            && top.kind.expected_end() == BlockEnd::Paragraph
        {
            self.on_end(BlockEnd::Paragraph, span)?;
        }
        Ok(())
    }

    fn on_task_marker(&mut self, checked: bool) {
        if let Some(Block {
            kind:
                BlockKind::Container(ContainerBlockKind::ListItem {
                    is_checked,
                    ..
                }),
            ..
        }) = self.state.stack.last_mut()
        {
            *is_checked = Some(checked);
        }
    }

    fn attach_completed(
        &mut self,
        block: Block<'source, Closed>,
    ) -> Result<(), NoteIngestError> {
        if let Some(parent) = self.state.stack.last_mut() {
            parent.kind.push_child(block).map_err(Into::into)
        } else {
            self.blocks.push(block);
            Ok(())
        }
    }
}

// ----------------------------------------------------------- //
//                     Traversal Iterator                      //
// ----------------------------------------------------------- //

/// Event emitted during pre-order traversal of the document AST.
///
/// Each container block (List, `BlockQuote`, `ListItem`) emits both an `Enter`
/// and `Exit` event. Leaf blocks (Paragraph, Heading, etc.) emit only `Enter`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum TraversalEvent<'tree, 'source> {
    /// Entering a block (emitted for all blocks).
    Enter(&'tree Block<'source, Closed>, u32),
    /// Exiting a container block (emitted only for containers).
    Exit(&'tree Block<'source, Closed>, u32),
}

/// Frame type for the traversal stack.
#[derive(Copy, Clone, Debug)]
enum StackFrame<'tree, 'source> {
    /// Enter a block (emit Enter event, then push children and Exit marker).
    Enter(&'tree Block<'source, Closed>, u32),
    /// Exit a container block (emit Exit event).
    Exit(&'tree Block<'source, Closed>, u32),
}

/// Pre-order depth-first iterator over the document AST.
pub(crate) struct PreorderIter<'tree, 'source> {
    /// Stack of frames to process.
    stack: Vec<StackFrame<'tree, 'source>>,
}

impl<'tree, 'source> PreorderIter<'tree, 'source> {
    /// Creates a new pre-order iterator from root blocks.
    fn new(roots: &'tree [Block<'source, Closed>]) -> Self {
        let mut stack = Vec::with_capacity(8.max(roots.len()));
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
                Some(TraversalEvent::Exit(block, depth))
            }
            StackFrame::Enter(block, depth) => {
                let children = block.children();
                if !children.is_empty() {
                    self.stack.push(StackFrame::Exit(block, depth));

                    let is_list = matches!(
                        &block.kind,
                        BlockKind::Container(ContainerBlockKind::List { .. })
                    );
                    let child_depth = if is_list {
                        depth
                    } else {
                        depth.saturating_add(1)
                    };

                    for child in children.iter().rev() {
                        self.stack.push(StackFrame::Enter(child, child_depth));
                    }
                }
                Some(TraversalEvent::Enter(block, depth))
            }
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
    use pulldown_cmark::Options;

    use super::*;
    use crate::note::parser::{
        config::{BreakPolicy, EventStreamConfig},
        context::ParserContext,
        types::{HeadingLevel, InlineToken},
    };

    fn build_structure(source: &str) -> DocTree<'_, Complete> {
        let ctx = ParserContext::new(source, EventStreamConfig::default())
            .expect("parser context should build");
        DocTree::from_context(&ctx).expect("structure should build")
    }

    #[test]
    fn parses_root_heading_and_paragraph() {
        let tree = build_structure("# Title\n\nBody");
        assert_eq!(tree.blocks().len(), 2);
        assert!(matches!(
            tree.blocks()[0].kind,
            BlockKind::Leaf(LeafBlockKind::Heading { .. })
        ));
        assert!(matches!(
            tree.blocks()[1].kind,
            BlockKind::Leaf(LeafBlockKind::Paragraph { .. })
        ));
    }

    #[test]
    fn preserves_task_marker_state_on_list_items() {
        let tree = build_structure("- [x] done\n- [ ] todo");
        let BlockKind::Container(ContainerBlockKind::List {
            children,
            ..
        }) = &tree.blocks()[0].kind
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
        let tree = build_structure("- parent\n  - child");
        let BlockKind::Container(ContainerBlockKind::List {
            children,
            ..
        }) = &tree.blocks()[0].kind
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
        let tree = build_structure(
            "- outer\n  > quote\n  > - nested-in-quote\n- sibling",
        );

        let BlockKind::Container(ContainerBlockKind::List {
            children,
            ..
        }) = &tree.blocks()[0].kind
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
        let tree = build_structure("---\ntags: [a]\n---\n\nBody");
        assert!(matches!(
            tree.blocks()[0].kind,
            BlockKind::Leaf(LeafBlockKind::Frontmatter { .. })
        ));
        assert!(matches!(
            tree.blocks()[1].kind,
            BlockKind::Leaf(LeafBlockKind::Paragraph { .. })
        ));
    }

    #[test]
    fn on_end_reports_canonical_underflow_encountered_label() {
        let mut tree = DocTree::<Processing<'_>>::new();
        let span = SourceByteRange::try_from(3..4).expect("valid range");

        let result = tree.on_end(BlockEnd::List, span);

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
        let mut tree = DocTree::<Processing<'_>>::new();
        let start = SourceByteRange::try_from(0..1).expect("valid range");
        tree.on_start(
            &BlockStart::Heading {
                level: HeadingLevel::H1,
            },
            start.clone(),
        )
        .expect("first start should succeed");

        let result = tree.on_start(&BlockStart::Paragraph, start);

        assert!(matches!(
            result,
            Err(NoteIngestError::Parse(NoteParseError::InvalidTopology {
                code: "parser.structure.start_inside_leaf",
                ..
            }))
        ));
    }

    #[test]
    fn start_block_inside_leaf_returns_invalid_topology() {
        let mut tree = DocTree::<Processing<'_>>::new();
        // Start a Heading (which is a leaf and doesn't auto-close)
        let range = SourceByteRange::try_from(0..1).expect("valid range");
        tree.on_start(
            &BlockStart::Heading {
                level: HeadingLevel::H1,
            },
            range.clone(),
        )
        .expect("first push should succeed");

        let result = tree.on_start(
            &BlockStart::Paragraph,
            SourceByteRange::try_from(5..6).expect("valid range"),
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
    fn nested_list_item_parent_pos_matches_enclosing_parent_item_pos() {
        let tree = build_structure("- parent\n  - child");

        let BlockKind::Container(ContainerBlockKind::List {
            children,
            ..
        }) = &tree.blocks()[0].kind
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
        let tree = build_structure("- parent\n  - child\n    - grandchild");

        let BlockKind::Container(ContainerBlockKind::List {
            children,
            ..
        }) = &tree.blocks()[0].kind
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
        let tree =
            build_structure("- parent-a\n  - child-a\n- parent-b\n  - child-b");

        let BlockKind::Container(ContainerBlockKind::List {
            children,
            ..
        }) = &tree.blocks()[0].kind
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
        let tree = build_structure("- parent\n  - child-a\n  - child-b");

        let BlockKind::Container(ContainerBlockKind::List {
            children,
            ..
        }) = &tree.blocks()[0].kind
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

    // ═════════════════════════════════════════════════════════════════════════
    // TRAVERSAL ITERATOR TESTS
    // ═════════════════════════════════════════════════════════════════════════

    #[test]
    fn iter_preorder_handles_empty_document() {
        let tree = build_structure("");
        let count = tree.iter_preorder().count();
        assert_eq!(count, 0, "empty document should have no events");
    }

    #[test]
    fn iter_preorder_emits_enter_only_for_leaf_blocks() {
        let tree = build_structure("# Heading\n\nParagraph");
        let events: Vec<_> = tree.iter_preorder().collect();

        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], TraversalEvent::Enter(_, 0)));
        assert!(matches!(events[1], TraversalEvent::Enter(_, 0)));
    }

    #[test]
    fn iter_preorder_emits_enter_exit_for_containers() {
        let tree = build_structure("- Item 1\n- Item 2");
        let events: Vec<_> = tree.iter_preorder().collect();

        assert_eq!(events.len(), 8);
        assert!(matches!(events[0], TraversalEvent::Enter(_, 0)));
        assert!(matches!(events[7], TraversalEvent::Exit(_, 0)));
    }

    #[test]
    fn iter_preorder_tracks_depth_correctly() {
        let tree = build_structure("> - Item\n>\n>   Paragraph");
        let events: Vec<_> = tree.iter_preorder().collect();

        let depths: Vec<u32> = events
            .iter()
            .filter_map(|event| match event {
                TraversalEvent::Enter(_, depth) => Some(*depth),
                TraversalEvent::Exit(..) => None,
            })
            .collect();

        assert!(depths.contains(&0), "blockquote");
        assert!(depths.contains(&1), "list/items");
        assert!(depths.contains(&2), "paragraph");
    }

    #[test]
    fn iter_preorder_nested_list_depth_matches_visitor() {
        let tree = build_structure("- Item\n  - Nested");
        let events: Vec<_> = tree.iter_preorder().collect();

        let enter_depths: Vec<u32> = events
            .iter()
            .filter_map(|event| match event {
                TraversalEvent::Enter(_, depth) => Some(*depth),
                TraversalEvent::Exit(..) => None,
            })
            .collect();

        assert_eq!(enter_depths, vec![0, 0, 1, 1, 1, 2]);
    }

    #[test]
    fn iter_preorder_event_order_is_preorder() {
        let tree = build_structure("> Quote\n>\n> - Item");
        let events: Vec<_> = tree.iter_preorder().collect();

        assert!(events.len() >= 8);
        if let TraversalEvent::Enter(block, depth) = events[0] {
            assert_eq!(depth, 0);
            assert!(matches!(
                block.kind,
                BlockKind::Container(ContainerBlockKind::BlockQuote { .. })
            ));
        } else {
            panic!("First event should be Enter");
        }

        let last_exit = events
            .iter()
            .rev()
            .find(|e| matches!(e, TraversalEvent::Exit(..)))
            .expect("should have exit");

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
        let tree = build_structure("# Heading\n\nParagraph");
        let mut count = 0usize;
        tree.for_each_block(|_block, _depth| {
            count += 1;
        });
        assert_eq!(count, 2);
    }

    #[test]
    fn blocks_preorder_returns_all_blocks() {
        let tree = build_structure("# Heading\n\n- Item 1\n- Item 2");
        let blocks = tree.blocks_preorder();

        assert_eq!(blocks.len(), 6);
        assert_eq!(blocks[0].1, 0); // heading
        assert_eq!(blocks[1].1, 0); // list
        assert_eq!(blocks[2].1, 0); // item1
        assert_eq!(blocks[3].1, 1); // para1
        assert_eq!(blocks[4].1, 0); // item2
        assert_eq!(blocks[5].1, 1); // para2
    }

    #[test]
    fn iter_preorder_enter_exit_events_are_balanced() {
        let tree = build_structure("> - Item 1\n> - Item 2\n>\n> Paragraph");
        let events: Vec<_> = tree.iter_preorder().collect();

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
                        assert_eq!(std::ptr::from_ref(*block), expected);
                    }
                }
            }
        }
        assert!(stack.is_empty());
    }

    #[test]
    fn on_start_fails_when_top_is_non_paragraph_leaf() {
        let mut tree = DocTree::<Processing<'_>>::new();
        let range = SourceByteRange::try_from(0..1).expect("valid range");

        // Heading is a leaf that doesn't auto-close on new block start
        tree.on_start(
            &BlockStart::Heading {
                level: HeadingLevel::H1,
            },
            range.clone(),
        )
        .expect("start heading");

        let result = tree.on_start(&BlockStart::Paragraph, range);

        assert!(matches!(
            result,
            Err(NoteIngestError::Parse(NoteParseError::InvalidTopology {
                code: "parser.structure.start_inside_leaf",
                ..
            }))
        ));
    }

    #[test]
    fn on_inline_event_fails_when_inside_unsupported_leaf() {
        // Since ThematicBreak is never on stack, and
        // Paragraph/Heading/CodeBlock/Frontmatter all support inline
        // events, this is hard to trigger via public API. We can test
        // by manually manipulating the stack if needed, but the current
        // implementation handles all variants on stack.

        // However, we can test that it correctly opens a paragraph when needed.
        let mut tree = DocTree::<Processing<'_>>::new();
        let range = SourceByteRange::try_from(0..1).expect("valid range");
        let event = RangedEvent::new(
            ParserEvent::Inline(InlineToken::Text("foo".into())),
            range,
        );

        tree.on_inline_event(event).expect("should auto-open paragraph");

        assert_eq!(tree.state.stack.len(), 1);
        assert!(matches!(
            tree.state.stack[0].kind,
            BlockKind::Leaf(LeafBlockKind::Paragraph { .. })
        ));
    }

    #[test]
    fn attach_completed_fails_when_top_is_leaf() {
        let mut tree = DocTree::<Processing<'_>>::new();
        let range = SourceByteRange::try_from(0..1).expect("valid range");

        tree.on_start(
            &BlockStart::Heading {
                level: HeadingLevel::H1,
            },
            range.clone(),
        )
        .expect("start heading");

        let child = Block {
            kind: BlockKind::Leaf(LeafBlockKind::Paragraph {
                events: Vec::new(),
            }),
            span: SourceByteRange::try_from(2..3).expect("valid range"),
        };

        let result = tree.attach_completed(child);

        assert!(matches!(
            result,
            Err(NoteIngestError::Parse(NoteParseError::InvalidTopology {
                code: "parser.structure.push_child_to_leaf",
                ..
            }))
        ));
    }

    #[test]
    fn on_end_fails_on_mismatched_tag() {
        let mut tree = DocTree::<Processing<'_>>::new();
        let range = SourceByteRange::try_from(0..1).expect("valid range");

        tree.on_start(&BlockStart::BlockQuote, range.clone())
            .expect("start quote");

        let result = tree.on_end(BlockEnd::List, range);

        assert!(matches!(
            result,
            Err(NoteIngestError::Parse(NoteParseError::EventStackMismatch {
                expected: "blockquote",
                found: "list",
                ..
            }))
        ));
    }

    #[test]
    fn thematic_break_closes_implicit_paragraph() {
        let mut tree = DocTree::<Processing<'_>>::new();

        // 1. Inline event opens implicit paragraph
        let range1 = SourceByteRange::try_from(0..4).expect("valid range");
        tree.process_event(RangedEvent::new(
            ParserEvent::Inline(InlineToken::Text("foo".into())),
            range1,
        ))
        .expect("process inline");

        assert_eq!(tree.state.stack.len(), 1);

        // 2. Thematic break should close it
        let range2 = SourceByteRange::try_from(5..8).expect("valid range");
        tree.process_event(RangedEvent::new(
            ParserEvent::ThematicBreak,
            range2,
        ))
        .expect("process thematic break");

        assert_eq!(tree.state.stack.len(), 0);
        assert_eq!(tree.blocks.len(), 2);
        assert!(matches!(
            tree.blocks[0].kind,
            BlockKind::Leaf(LeafBlockKind::Paragraph { .. })
        ));
        assert!(matches!(
            tree.blocks[1].kind,
            BlockKind::Leaf(LeafBlockKind::ThematicBreak)
        ));
    }

    #[test]
    fn finish_auto_closes_trailing_implicit_paragraph() {
        let mut tree = DocTree::<Processing<'_>>::new();

        // 1. Inline event opens implicit paragraph
        let range = SourceByteRange::try_from(0..4).expect("valid range");
        tree.process_event(RangedEvent::new(
            ParserEvent::Inline(InlineToken::Text("foo".into())),
            range,
        ))
        .expect("process inline");

        // 2. finish should succeed
        let result = tree.finish();

        assert!(result.is_ok(), "finish should auto-close trailing paragraph");
    }

    #[test]
    fn captures_text_inside_tables_as_paragraphs() {
        let source =
            "| Header 1 | Header 2 |\n| --- | --- |\n| Cell 1 | Cell 2 |";
        let config = EventStreamConfig::new(
            Options::ENABLE_TABLES,
            BreakPolicy::NormalizeAsText,
            true,
        );
        let ctx = ParserContext::new(source, config).expect("ctx");
        let tree = DocTree::from_context(&ctx).expect("tree");

        assert!(!tree.blocks().is_empty());
        assert!(tree.blocks().iter().any(|b| matches!(
            b.kind,
            BlockKind::Leaf(LeafBlockKind::Paragraph { .. })
        )));
    }

    #[test]
    fn captures_text_inside_footnotes_as_paragraphs() {
        let source = "Text with a footnote[^1]\n\n[^1]: The footnote content.";
        let config = EventStreamConfig::new(
            Options::ENABLE_FOOTNOTES,
            BreakPolicy::NormalizeAsText,
            true,
        );
        let ctx = ParserContext::new(source, config).expect("ctx");
        let tree = DocTree::from_context(&ctx).expect("tree");

        assert!(tree.blocks().iter().any(|b| matches!(
            b.kind,
            BlockKind::Leaf(LeafBlockKind::Paragraph { .. })
        )));
    }

    #[test]
    fn captures_text_inside_definition_lists_as_paragraphs() {
        let source = "Term\n: Definition";
        let config = EventStreamConfig::new(
            Options::ENABLE_DEFINITION_LIST,
            BreakPolicy::NormalizeAsText,
            true,
        );
        let ctx = ParserContext::new(source, config).expect("ctx");
        let tree = DocTree::from_context(&ctx).expect("tree");

        assert!(tree.blocks().iter().any(|b| matches!(
            b.kind,
            BlockKind::Leaf(LeafBlockKind::Paragraph { .. })
        )));
    }
}
