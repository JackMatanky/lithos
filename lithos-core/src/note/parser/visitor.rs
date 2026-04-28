//! Block visitor trait for AST traversal.
//!
//! This module provides the visitor pattern for traversing the block AST
//! produced by the parsing layer. Visitors can extract metadata, transform
//! content, or perform analysis without coupling to the AST structure.
//!
//! # Design Philosophy
//!
//! - **Separation of concerns**: Traversal logic in `DocStructure::walk()`,
//!   application logic in visitor implementations
//! - **Depth tracking**: Visitor receives current nesting depth automatically
//! - **Pre-order traversal**: Parent blocks visited before their children
//! - **Immutable by default**: Visitors receive `&Block` (mutation requires
//!   explicit design)
//!
//! # Examples
//!
//! ```rust,ignore
//! use lithos_core::note::parser::{
//!     structure::{Block, BlockKind},
//!     visitor::BlockVisitor,
//! };
//!
//! struct HeadingCollector {
//!     headings: Vec<String>,
//! }
//!
//! impl<'source> BlockVisitor<'source> for HeadingCollector {
//!     fn visit_heading(
//!         &mut self,
//!         block: &Block<'source>,
//!         level: HeadingLevel,
//!         depth: u32,
//!     ) {
//!         if let Some(text) = block.text() {
//!             self.headings.push(text);
//!         }
//!     }
//! }
//!
//! // Usage
//! let mut collector = HeadingCollector { headings: Vec::new() };
//! structure.walk(&mut collector);
//! println!("Found {} headings", collector.headings.len());
//! ```

use pulldown_cmark::{CowStr, MetadataBlockKind};

use super::structure::{Block, HeadingLevel, ListKind};

/// Visitor trait for traversing the block AST.
///
/// Implement this trait to extract metadata, transform content, or perform
/// analysis on the parsed document structure. The `DocStructure::walk()` method
/// handles traversal logic and depth tracking.
///
/// # Method Naming Convention
///
/// All methods follow the pattern `visit_<block_type>` where `<block_type>`
/// matches the `BlockKind` variant name in `snake_case`.
///
/// # Default Implementations
///
/// All methods have empty default implementations, allowing visitors to only
/// override the methods they care about.
///
/// # Traversal Order
///
/// Visitors use **pre-order traversal**: parent blocks are visited before their
/// children. For example, in a nested list:
///
/// 1. `visit_list()` for outer list
/// 2. `visit_list_item()` for first item
/// 3. `visit_paragraph()` for paragraph in first item
/// 4. `visit_list()` for nested list
/// 5. ... and so on
///
/// # Depth Tracking
///
/// The `depth` parameter indicates nesting level:
/// - `0` = root-level blocks
/// - `1` = first level of nesting (e.g., items in a root list)
/// - `2` = second level (e.g., paragraphs in list items)
/// - etc.
pub trait BlockVisitor<'source> {
    /// Visit a blockquote container.
    ///
    /// This method is called **before** visiting the blockquote's children.
    /// After this method returns, the traversal will automatically visit all
    /// child blocks with incremented depth.
    ///
    /// # Parameters
    ///
    /// - `block`: The blockquote block (contains `children` field)
    /// - `depth`: Nesting depth
    #[expect(unused_variables, reason = "default trait impl has empty body")]
    fn visit_blockquote(&mut self, block: &Block<'source>, depth: u32) {}

    /// Visit a code block (fenced or indented).
    ///
    /// # Parameters
    ///
    /// - `block`: The code block (contains `text` field)
    /// - `language`: Optional language identifier (e.g., "rust", "python")
    /// - `depth`: Nesting depth
    #[expect(unused_variables, reason = "default trait impl has empty body")]
    fn visit_code_block(
        &mut self,
        block: &Block<'source>,
        language: Option<&CowStr<'source>>,
        depth: u32,
    ) {
    }

    /// Visit a frontmatter block (YAML or Pluses-delimited).
    ///
    /// # Parameters
    ///
    /// - `block`: The frontmatter block (contains `text` field)
    /// - `format`: Metadata format (YAML, Pluses, etc.)
    /// - `depth`: Always 0 (frontmatter only appears at document root)
    #[expect(unused_variables, reason = "default trait impl has empty body")]
    fn visit_frontmatter(
        &mut self,
        block: &Block<'source>,
        format: MetadataBlockKind,
        depth: u32,
    ) {
    }

    /// Visit a heading block (H1-H6).
    ///
    /// # Parameters
    ///
    /// - `block`: The heading block (contains `events` field)
    /// - `level`: Heading level (H1 = 1, H6 = 6)
    /// - `depth`: Nesting depth (typically 0, but can be nested in blockquotes)
    #[expect(unused_variables, reason = "default trait impl has empty body")]
    fn visit_heading(
        &mut self,
        block: &Block<'source>,
        level: HeadingLevel,
        depth: u32,
    ) {
    }

    /// Visit a list container (ordered or unordered).
    ///
    /// This method is called **before** visiting the list's items. After this
    /// method returns, the traversal will automatically visit all list item
    /// children.
    ///
    /// # Parameters
    ///
    /// - `block`: The list block (contains `children` field)
    /// - `kind`: List kind (ordered with start number, or unordered)
    /// - `depth`: Nesting depth
    #[expect(unused_variables, reason = "default trait impl has empty body")]
    fn visit_list(
        &mut self,
        block: &Block<'source>,
        kind: ListKind,
        depth: u32,
    ) {
    }

    /// Visit a list item container.
    ///
    /// This method is called **before** visiting the item's children. List
    /// items can contain paragraphs, code blocks, nested lists, etc.
    ///
    /// # Parameters
    ///
    /// - `block`: The list item block (contains `children` field)
    /// - `is_task`: Task list checkbox state (Some(true) = checked, Some(false)
    ///   = unchecked, None = regular item)
    /// - `depth`: Nesting depth
    #[expect(unused_variables, reason = "default trait impl has empty body")]
    fn visit_list_item(
        &mut self,
        block: &Block<'source>,
        is_task: Option<bool>,
        depth: u32,
    ) {
    }
    /// Visit a paragraph block.
    ///
    /// Paragraphs contain inline content (text, links, emphasis, etc.) stored
    /// as events. Use `block.text()` to extract plain text.
    ///
    /// # Parameters
    ///
    /// - `block`: The paragraph block (contains `events` field)
    /// - `depth`: Nesting depth (0 = root, 1+ = nested)
    #[expect(unused_variables, reason = "default trait impl has empty body")]
    fn visit_paragraph(&mut self, block: &Block<'source>, depth: u32) {}

    /// Visit a thematic break (horizontal rule).
    ///
    /// # Parameters
    ///
    /// - `block`: The thematic break block
    /// - `depth`: Nesting depth
    #[expect(unused_variables, reason = "default trait impl has empty body")]
    fn visit_thematic_break(&mut self, block: &Block<'source>, depth: u32) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::parser::{
        config::EventStreamConfig, context::ParserContext,
        structure::DocStructure,
    };

    /// Test visitor that collects block type counts.
    struct BlockCounter {
        paragraphs: usize,
        headings: usize,
        code_blocks: usize,
        lists: usize,
        list_items: usize,
        blockquotes: usize,
        thematic_breaks: usize,
        frontmatter: usize,
    }

    impl BlockCounter {
        fn new() -> Self {
            Self {
                paragraphs: 0,
                headings: 0,
                code_blocks: 0,
                lists: 0,
                list_items: 0,
                blockquotes: 0,
                thematic_breaks: 0,
                frontmatter: 0,
            }
        }

        fn total(&self) -> usize {
            self.paragraphs
                + self.headings
                + self.code_blocks
                + self.lists
                + self.list_items
                + self.blockquotes
                + self.thematic_breaks
                + self.frontmatter
        }
    }

    impl<'source> BlockVisitor<'source> for BlockCounter {
        fn visit_paragraph(&mut self, _block: &Block<'source>, _depth: u32) {
            self.paragraphs += 1;
        }

        fn visit_heading(
            &mut self,
            _block: &Block<'source>,
            _level: HeadingLevel,
            _depth: u32,
        ) {
            self.headings += 1;
        }

        fn visit_code_block(
            &mut self,
            _block: &Block<'source>,
            _language: Option<&CowStr<'source>>,
            _depth: u32,
        ) {
            self.code_blocks += 1;
        }

        fn visit_frontmatter(
            &mut self,
            _block: &Block<'source>,
            _format: MetadataBlockKind,
            _depth: u32,
        ) {
            self.frontmatter += 1;
        }

        fn visit_thematic_break(
            &mut self,
            _block: &Block<'source>,
            _depth: u32,
        ) {
            self.thematic_breaks += 1;
        }

        fn visit_blockquote(&mut self, _block: &Block<'source>, _depth: u32) {
            self.blockquotes += 1;
        }

        fn visit_list(
            &mut self,
            _block: &Block<'source>,
            _kind: ListKind,
            _depth: u32,
        ) {
            self.lists += 1;
        }

        fn visit_list_item(
            &mut self,
            _block: &Block<'source>,
            _is_task: Option<bool>,
            _depth: u32,
        ) {
            self.list_items += 1;
        }
    }

    /// Test visitor that tracks depth information.
    struct DepthTracker {
        max_depth: u32,
        depth_counts: Vec<usize>,
    }

    impl DepthTracker {
        fn new() -> Self {
            Self {
                max_depth: 0,
                depth_counts: Vec::new(),
            }
        }

        fn record_depth(&mut self, depth: u32) {
            self.max_depth = self.max_depth.max(depth);
            let depth_idx = depth as usize;
            if self.depth_counts.len() <= depth_idx {
                self.depth_counts.resize(depth_idx + 1, 0);
            }
            self.depth_counts[depth_idx] += 1;
        }
    }

    impl<'source> BlockVisitor<'source> for DepthTracker {
        fn visit_paragraph(&mut self, _block: &Block<'source>, depth: u32) {
            self.record_depth(depth);
        }

        fn visit_heading(
            &mut self,
            _block: &Block<'source>,
            _level: HeadingLevel,
            depth: u32,
        ) {
            self.record_depth(depth);
        }

        fn visit_code_block(
            &mut self,
            _block: &Block<'source>,
            _language: Option<&CowStr<'source>>,
            depth: u32,
        ) {
            self.record_depth(depth);
        }

        fn visit_list(
            &mut self,
            _block: &Block<'source>,
            _kind: ListKind,
            depth: u32,
        ) {
            self.record_depth(depth);
        }

        fn visit_list_item(
            &mut self,
            _block: &Block<'source>,
            _is_task: Option<bool>,
            depth: u32,
        ) {
            self.record_depth(depth);
        }

        fn visit_blockquote(&mut self, _block: &Block<'source>, depth: u32) {
            self.record_depth(depth);
        }
    }

    #[test]
    fn visitor_counts_simple_blocks() {
        let source = "# Heading\n\nParagraph\n\n```\ncode\n```";
        let config = EventStreamConfig::default();
        let ctx = ParserContext::new(source, config).expect("parse");
        let structure = DocStructure::from_context(&ctx).expect("build");

        let mut counter = BlockCounter::new();
        structure.walk(&mut counter);

        assert_eq!(counter.headings, 1);
        assert_eq!(counter.paragraphs, 1);
        assert_eq!(counter.code_blocks, 1);
        assert_eq!(counter.total(), 3);
    }

    #[test]
    fn visitor_counts_nested_list() {
        let source =
            "- Item 1\n\n  Loose item text\n\n  - Nested item\n\n- Item 2";
        let config = EventStreamConfig::default();
        let ctx = ParserContext::new(source, config).expect("parse");
        let structure = DocStructure::from_context(&ctx).expect("build");

        let mut counter = BlockCounter::new();
        structure.walk(&mut counter);

        assert_eq!(counter.lists, 2, "should have outer and inner list");
        assert_eq!(counter.list_items, 3, "should have 3 items total");
        // In "loose" lists (with blank lines), items DO contain paragraphs
        assert!(
            counter.paragraphs >= 3,
            "loose list items should have paragraph children, got {}",
            counter.paragraphs
        );
    }

    #[test]
    fn visitor_tracks_depth_correctly() {
        // Use loose list in blockquote to get depth-2 paragraph
        let source = "# Root\n\n> Blockquote\n>\n> - Item in quote\n>\n>   \
                      Paragraph in item";
        let config = EventStreamConfig::default();
        let ctx = ParserContext::new(source, config).expect("parse");
        let structure = DocStructure::from_context(&ctx).expect("build");

        let mut tracker = DepthTracker::new();
        structure.walk(&mut tracker);

        assert_eq!(tracker.max_depth, 2, "should have depth 0, 1, 2");
        assert!(
            tracker.depth_counts[0] >= 2,
            "should have root-level blocks (heading + blockquote)"
        );
        assert!(
            tracker.depth_counts[1] >= 1,
            "should have depth-1 blocks (list in quote)"
        );
        assert!(
            tracker.depth_counts[2] >= 1,
            "should have depth-2 blocks (paragraph in item)"
        );
    }

    #[test]
    fn visitor_handles_empty_document() {
        let source = "";
        let config = EventStreamConfig::default();
        let ctx = ParserContext::new(source, config).expect("parse");
        let structure = DocStructure::from_context(&ctx).expect("build");

        let mut counter = BlockCounter::new();
        structure.walk(&mut counter);

        assert_eq!(counter.total(), 0, "empty document should have no blocks");
    }

    #[test]
    fn visitor_handles_thematic_break() {
        let source = "Before\n\n---\n\nAfter";
        let config = EventStreamConfig::default();
        let ctx = ParserContext::new(source, config).expect("parse");
        let structure = DocStructure::from_context(&ctx).expect("build");

        let mut counter = BlockCounter::new();
        structure.walk(&mut counter);

        assert_eq!(counter.paragraphs, 2);
        assert_eq!(counter.thematic_breaks, 1);
        assert_eq!(counter.total(), 3);
    }

    #[test]
    fn visitor_handles_frontmatter() {
        let source = "---\ntags: [test]\n---\n\n# Content";
        let config = EventStreamConfig::default();
        let ctx = ParserContext::new(source, config).expect("parse");
        let structure = DocStructure::from_context(&ctx).expect("build");

        let mut counter = BlockCounter::new();
        structure.walk(&mut counter);

        assert_eq!(counter.frontmatter, 1);
        assert_eq!(counter.headings, 1);
    }

    #[test]
    fn visitor_traverses_complex_nested_structure() {
        let source = "
# Top

> Quote level 1
>
> - List in quote
>   - Nested list
>     - Double nested
>
> More quote text

Regular paragraph
";
        let config = EventStreamConfig::default();
        let ctx = ParserContext::new(source, config).expect("parse");
        let structure = DocStructure::from_context(&ctx).expect("build");

        let mut counter = BlockCounter::new();
        structure.walk(&mut counter);

        // Should visit all blocks in document
        assert!(counter.headings >= 1);
        assert!(counter.blockquotes >= 1);
        assert!(counter.lists >= 2, "should have at least 2 lists");
        assert!(counter.list_items >= 3, "should have at least 3 list items");
        assert!(counter.paragraphs >= 2);
    }

    /// Test visitor that collects text from all paragraphs.
    struct TextCollector {
        texts: Vec<String>,
    }

    impl TextCollector {
        fn new() -> Self {
            Self {
                texts: Vec::new(),
            }
        }
    }

    impl<'source> BlockVisitor<'source> for TextCollector {
        fn visit_paragraph(&mut self, block: &Block<'source>, _depth: u32) {
            if let Some(text) = block.text() {
                self.texts.push(text);
            }
        }
    }

    #[test]
    fn visitor_can_extract_content() {
        // Use loose list (blank lines) to get paragraphs
        let source = "First paragraph\n\nSecond paragraph\n\n- Item \
                      paragraph\n\n- Second item";
        let config = EventStreamConfig::default();
        let ctx = ParserContext::new(source, config).expect("parse");
        let structure = DocStructure::from_context(&ctx).expect("build");

        let mut collector = TextCollector::new();
        structure.walk(&mut collector);

        // Should collect: 2 root paragraphs + 2 list item paragraphs
        assert!(
            collector.texts.len() >= 4,
            "should collect at least 4 paragraphs"
        );
        assert_eq!(collector.texts[0], "First paragraph");
        assert_eq!(collector.texts[1], "Second paragraph");
        assert_eq!(collector.texts[2], "Item paragraph");
        assert_eq!(collector.texts[3], "Second item");
    }
}
