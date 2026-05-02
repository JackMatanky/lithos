# Block Component Design - Final Specification

**Status**: Approved for Implementation
**Created**: 2026-04-27

---

## Design Goals

1. **Clear names**: Each type's purpose is obvious from the name
2. **Single responsibility**: No overlapping concerns
3. **Minimal duplication**: Don't repeat the same fields across types
4. **Type safety**: Impossible to use an incomplete node as a complete one
5. **Private builder state**: Implementation details hidden from consumers

---

## Component Responsibilities

| Type                | Visibility       | Lifecycle        | Purpose                                    |
| ------------------- | ---------------- | ---------------- | ------------------------------------------ |
| **`Block`**         | `pub(crate)`       | Permanent (AST)  | Final, immutable block in document tree    |
| **`BlockKind`**         | `pub(crate)`       | Permanent (AST)  | The type and content of a block            |
| **`ProcessingBlock`**   | `private`          | Temporary        | Builder state for a block under construction |
| **`ProcessingBlockKind`** | `private`          | Temporary        | Incomplete block type during building      |

---

## Final AST Components (Public within crate)

### **`Block<'source>`**

**Purpose**: A complete, immutable block in the markdown document tree.

**When it exists**: After `ProcessingBlock::finalize()` is called during AST building.

**Design**:
```rust
/// A complete block in the markdown document tree.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Block<'source> {
    /// The type and content of this block
    pub(crate) kind: BlockKind<'source>,
    /// The complete source byte range (both start and end known)
    pub(crate) span: SourceByteRange,
}

impl<'source> Block<'source> {
    /// Extract plain text from inline events (lazy evaluation).
    ///
    /// Returns `Some(String)` for leaf blocks (Paragraph, Heading),
    /// `None` for containers and code blocks.
    pub(crate) fn text(&self) -> Option<String> {
        match &self.kind {
            BlockKind::Paragraph { events } |
            BlockKind::Heading { events, .. } => {
                Some(Self::events_to_text(events))
            }
            _ => None
        }
    }

    /// Returns true if this block should be scanned for metadata.
    ///
    /// Code blocks return false (we don't scan code for tags/fields).
    pub(crate) fn is_scannable(&self) -> bool {
        !matches!(self.kind, BlockKind::CodeBlock { .. })
    }

    fn events_to_text(events: &[SpannedEvent<'source>]) -> String {
        events.iter()
            .filter_map(|e| match &e.event {
                Event::Text(s) => Some(s.as_ref()),
                _ => None
            })
            .collect::<Vec<_>>()
            .join("")
    }
}
```

---

### **`BlockKind<'source>`**

**Purpose**: The type and content of a markdown block.

**Design**:
```rust
/// The type and content of a markdown block.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BlockKind<'source> {
    // ═══════════════════════════════════════════════════════════
    // LEAF BLOCKS (Content-bearing)
    // ═══════════════════════════════════════════════════════════

    /// Paragraph block containing inline content
    Paragraph {
        events: Vec<SpannedEvent<'source>>
    },

    /// Heading block (H1-H6) with inline content
    Heading {
        level: HeadingLevel,
        events: Vec<SpannedEvent<'source>>
    },

    /// Fenced or indented code block
    CodeBlock {
        language: Option<CowStr<'source>>,
        text: String,  // Flattened from events
    },

    /// YAML or Pluses-delimited frontmatter block
    Frontmatter {
        format: MetadataBlockKind,  // pulldown_cmark::MetadataBlockKind
        text: String,  // Raw frontmatter content
    },

    /// Thematic break (horizontal rule)
    ThematicBreak,

    // ═══════════════════════════════════════════════════════════
    // CONTAINER BLOCKS (Structure-bearing)
    // ═══════════════════════════════════════════════════════════

    /// Blockquote containing nested blocks
    BlockQuote {
        children: Vec<Block<'source>>
    },

    /// Ordered or unordered list containing list items
    List {
        kind: ListKind,
        children: Vec<Block<'source>>
    },

    /// Individual list item (can contain paragraphs, sublists, etc.)
    ListItem {
        /// Nesting depth (0 = root, 1 = first level, etc.)
        depth: u32,
        /// Span of parent list item (if nested)
        parent_span: Option<SourceByteRange>,
        /// Checkbox state (Some(true) = checked, Some(false) = unchecked, None = not a task)
        is_checkbox: Option<bool>,
        /// Child blocks (paragraphs, code, sublists, etc.)
        children: Vec<Block<'source>>
    },
}

/// Heading level (H1 through H6)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeadingLevel {
    H1, H2, H3, H4, H5, H6
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
    /// Convert to numeric level (1-6)
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

/// List type (ordered or unordered)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListKind {
    Unordered,
    Ordered { start: u64 },
}
```

---

## Builder State Components (Private to DocTree)

### **`ProcessingBlock<'source>`**

**Purpose**: Temporary state for a block being constructed during AST building.

**Lifecycle**: Created on `Event::Start(tag)`, finalized on `Event::End(tag)`.

**Visibility**: Private to `DocTree::from_context()` implementation.

**Design**:
```rust
/// Temporary state for a block being constructed.
///
/// This type only exists during the AST building phase and is never
/// exposed outside of `DocTree::from_context()`. It accumulates
/// events and child blocks until the closing tag arrives, then
/// finalizes into a complete `Block`.
struct ProcessingBlock<'source> {
    /// What kind of block this will become (incomplete until closed)
    kind: ProcessingBlockKind,

    /// Byte offset where this block started
    start: usize,

    /// Accumulated inline events (for leaf blocks like Paragraph, Heading)
    events: Vec<SpannedEvent<'source>>,

    /// Accumulated child blocks (for container blocks like List, BlockQuote)
    children: Vec<Block<'source>>,

    /// Current nesting depth (tracked for list items)
    depth: u32,
}

impl<'source> ProcessingBlock<'source> {
    /// Create a new processing block for a leaf node.
    fn new_leaf(kind: ProcessingBlockKind, start: usize, depth: u32) -> Self {
        Self {
            kind,
            start,
            events: Vec::new(),
            children: Vec::new(),
            depth,
        }
    }

    /// Create a new processing block for a container node.
    fn new_container(kind: ProcessingBlockKind, start: usize, depth: u32) -> Self {
        Self {
            kind,
            start,
            events: Vec::new(),
            children: Vec::new(),
            depth,
        }
    }

    /// Add an inline event to this block (for leaf blocks).
    fn push_event(&mut self, event: SpannedEvent<'source>) {
        self.events.push(event);
    }

    /// Add a child block to this container (for container blocks).
    fn push_child(&mut self, child: Block<'source>) {
        self.children.push(child);
    }

    /// Finalize this processing block into a complete Block.
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if the span (start..end) is invalid.
    fn finalize(self, end: usize) -> Result<Block<'source>, ParseError> {
        let span = SourceByteRange::try_from(self.start..end)
            .map_err(|_| ParseError::InvalidSpan {
                start: self.start,
                end
            })?;

        let kind = match self.kind {
            ProcessingBlockKind::Paragraph => {
                BlockKind::Paragraph { events: self.events }
            }

            ProcessingBlockKind::Heading { level } => {
                BlockKind::Heading { level, events: self.events }
            }

            ProcessingBlockKind::CodeBlock { language } => {
                let text = Self::flatten_text(&self.events);
                BlockKind::CodeBlock {
                    language: language.map(|s| CowStr::Boxed(s.into_boxed_str())),
                    text
                }
            }

            ProcessingBlockKind::Frontmatter { format } => {
                let text = Self::flatten_text(&self.events);
                BlockKind::Frontmatter { format, text }
            }

            ProcessingBlockKind::ThematicBreak => {
                BlockKind::ThematicBreak
            }

            ProcessingBlockKind::BlockQuote => {
                BlockKind::BlockQuote { children: self.children }
            }

            ProcessingBlockKind::List { kind } => {
                BlockKind::List { kind, children: self.children }
            }

            ProcessingBlockKind::ListItem { is_checkbox, parent_span } => {
                BlockKind::ListItem {
                    depth: self.depth,
                    parent_span,
                    is_checkbox,
                    children: self.children,
                }
            }
        };

        Ok(Block { kind, span })
    }

    /// Helper: Flatten events into a single string.
    fn flatten_text(events: &[SpannedEvent<'source>]) -> String {
        events.iter()
            .filter_map(|e| match &e.event {
                Event::Text(s) => Some(s.as_ref()),
                _ => None
            })
            .collect::<Vec<_>>()
            .join("")
    }
}
```

---

### **`ProcessingBlockKind`**

**Purpose**: The incomplete type of a block being built (parallel to `BlockKind` but without finalized data).

**Design**:
```rust
/// The type of block being built (parallel to BlockKind but incomplete).
enum ProcessingBlockKind {
    Paragraph,
    Heading { level: HeadingLevel },
    CodeBlock { language: Option<String> },
    Frontmatter { format: MetadataBlockKind },
    ThematicBreak,
    BlockQuote,
    List { kind: ListKind },
    ListItem {
        is_checkbox: Option<bool>,
        parent_span: Option<SourceByteRange>,
    },
}
```

**Why separate from `BlockKind`?**
- `ProcessingBlockKind::CodeBlock` stores `language: Option<String>` (owned)
- `BlockKind::CodeBlock` stores `language: Option<CowStr<'source>>` (borrowed)
- Type safety: Cannot accidentally use `ProcessingBlockKind` where `BlockKind` is expected

---

## Usage Example (Inside DocTree::from_context)

```rust
impl<'source> DocTree<'source> {
    pub(crate) fn from_context(ctx: &ParserContext<'source>) -> Result<Self, ParseError> {
        let mut stack: Vec<ProcessingBlock<'source>> = Vec::new();
        let mut root_blocks: Vec<Block<'source>> = Vec::new();
        let mut current_depth: u32 = 0;

        for spanned_event in ctx.events() {
            match &spanned_event.event {
                Event::Start(tag) => {
                    match tag {
                        Tag::Paragraph => {
                            stack.push(ProcessingBlock::new_leaf(
                                ProcessingBlockKind::Paragraph,
                                spanned_event.span.start().as_usize(),
                                current_depth,
                            ));
                        }
                        Tag::Heading { level, .. } => {
                            stack.push(ProcessingBlock::new_leaf(
                                ProcessingBlockKind::Heading {
                                    level: (*level).into()
                                },
                                spanned_event.span.start().as_usize(),
                                current_depth,
                            ));
                        }
                        Tag::List(start_num) => {
                            let kind = match start_num {
                                Some(n) => ListKind::Ordered { start: *n },
                                None => ListKind::Unordered,
                            };
                            stack.push(ProcessingBlock::new_container(
                                ProcessingBlockKind::List { kind },
                                spanned_event.span.start().as_usize(),
                                current_depth,
                            ));
                            current_depth += 1;
                        }
                        // ... handle other tags
                        _ => {}
                    }
                }

                Event::End(tag_end) => {
                    let processing = stack.pop()
                        .ok_or(ParseError::StackUnderflow)?;

                    let block = processing.finalize(spanned_event.span.end().as_usize())?;

                    // Append to parent or root
                    if let Some(parent) = stack.last_mut() {
                        parent.push_child(block);
                    } else {
                        root_blocks.push(block);
                    }

                    // Decrease depth after closing a list
                    if matches!(tag_end, TagEnd::List(_)) {
                        current_depth = current_depth.saturating_sub(1);
                    }
                }

                Event::Text(_) | Event::Code(_) => {
                    if let Some(current) = stack.last_mut() {
                        current.push_event(spanned_event.clone());
                    }
                }

                Event::TaskListMarker(checked) => {
                    if let Some(current) = stack.last_mut() {
                        if let ProcessingBlockKind::ListItem { is_checkbox, .. } = &mut current.kind {
                            *is_checkbox = Some(*checked);
                        }
                    }
                }

                _ => {}
            }
        }

        Ok(Self { blocks: root_blocks })
    }
}
```

---

## Design Rationale

### Why `ProcessingBlock` instead of `OpenBlock`?
- **Active verb**: "Processing" emphasizes this is temporary, under construction
- **Clear intent**: This is part of a process, not final output
- **Industry standard**: Compilers use "processing" terminology (processing tokens, processing nodes)

### Why not `ClosedBlock` for the final AST?
- **Awkward terminology**: "Closed" implies an action, but the AST is a static data structure
- **Existing convention**: AST nodes are typically called `Node`, `AstNode`, or `Block`
- **Our choice**: `Block` is clean and matches our existing naming (`BlockKind`, `BlockVisitor`)

### Why separate `ProcessingBlockKind` from `BlockKind`?
- **Type safety**: Cannot accidentally use incomplete data as complete
- **Different ownership**: Processing uses `String`, final uses `CowStr<'source>` or `Vec<SpannedEvent>`
- **Clear lifecycle**: Processing → Finalize → Final

### Why store `depth` in `ProcessingBlock` instead of `ProcessingBlockKind::ListItem`?
- **Availability**: Every block on the stack has depth information (needed for tracking)
- **Simplicity**: Don't need to pattern-match to get depth during building
- **Efficiency**: Single field vs duplicated across enum variants

---

## Key Invariants

1. **`Block` is always complete**: `span` has both start and end
2. **`ProcessingBlock` is always on the stack**: Never exposed to external code
3. **Stack discipline**: Every `Event::Start` pushes, every `Event::End` pops
4. **Depth tracking**: Incremented on container open, decremented on container close
5. **Parent tracking**: List items store parent span computed from stack state

---

## Testing Strategy

### Unit Tests for `ProcessingBlock`
```rust
#[test]
fn processing_paragraph_finalizes_with_events() {
    let mut pb = ProcessingBlock::new_leaf(ProcessingBlockKind::Paragraph, 0, 0);
    pb.push_event(SpannedEvent::new(Event::Text("hello".into()), span(0, 5)));

    let block = pb.finalize(5).unwrap();

    assert!(matches!(block.kind, BlockKind::Paragraph { .. }));
    assert_eq!(block.span, span(0, 5));
    assert_eq!(block.text(), Some("hello".to_string()));
}

#[test]
fn processing_list_finalizes_with_children() {
    let mut pb = ProcessingBlock::new_container(
        ProcessingBlockKind::List { kind: ListKind::Unordered },
        0,
        0,
    );

    let child = Block {
        kind: BlockKind::ListItem { depth: 1, parent_span: None, is_checkbox: None, children: vec![] },
        span: span(2, 10),
    };
    pb.push_child(child);

    let block = pb.finalize(12).unwrap();

    if let BlockKind::List { children, .. } = block.kind {
        assert_eq!(children.len(), 1);
    } else {
        panic!("Expected List block");
    }
}
```

### Integration Tests for `DocTree`
- Parse simple paragraph
- Parse nested lists with correct depth tracking
- Parse blockquote containing list
- Parse frontmatter at document start
- Parse mixed leaf and container blocks

---

**Status**: ✅ Design Complete - Ready for Implementation

**Next Step**: Implement Phase 1 (ParserContext)
