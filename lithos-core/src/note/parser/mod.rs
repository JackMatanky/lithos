//! Markdown parsing pipeline and orchestration.
//!
//! This module provides the primary ingestion engine for Obsidian-compatible
//! markdown files. It uses a single-pass event stream driven by
//! `pulldown-cmark` to extract both structural components (headings, sections,
//! lists) and specialized metadata (tags, inline fields, block references,
//! frontmatter).
//!
//! # Pipeline Architecture
//!
//! The parsing process follows a strict multi-stage, event-driven pipeline
//! designed for modularity, zero-cost abstractions, and strong separation of
//! concerns. The phases are:
//!
//! 1. **Adapter Layer** ([`stream::MarkdownEventStream`]): Wraps the core
//!    `pulldown-cmark` parser. Normalizes soft and hard line breaks and merges
//!    adjacent text nodes according to configurable policies.
//! 2. **Structure Builder**: Tracks block depth and nesting. Emits completed
//!    leaf and container blocks without allocating text fragments for container
//!    blocks.
//! 3. **Lexical Metadata Scanner**: Scans leaf text segments using explicit
//!    rules to identify artifacts like tags, inline fields, and block
//!    references.
//! 4. **Semantic Validator**: Enforces schema rules and standardizes metadata
//!    tokens.
//! 5. **Artifact Assembler**: Collects structural blocks and validated metadata
//!    into the final domain [`RawNote`](crate::note::raw::RawNote), applying
//!    routing policies (e.g., whether list-item tags are global).
//!
//! # Core Invariants
//!
//! - **Parser isolates grammar**: The parser does not depend on or import
//!   domain types (e.g. `RawTag` or `RawNote`).
//! - **Semantic rules are explicit**: Rules (like line break handling) are
//!   passed as configuration data, not hidden in branching code.
//! - **Memory efficiency**: No large AST allocations. Uses zero-cost iterator
//!   adapters internally and only clones when extracting reference definitions.
//!
//! # Examples
//!
//! ```ignore
//! // Conceptual usage of the parsing pipeline (Adapter Layer)
//! use lithos_core::note::parser::{
//!     config::EventStreamConfig,
//!     stream::MarkdownEventStream
//! };
//!
//! let source = "# Hello\n[link]: /url";
//! let config = EventStreamConfig::default();
//! let stream = MarkdownEventStream::new(source, config);
//!
//! // Extract reference definitions cleanly:
//! assert_eq!(stream.references().resolve("link"), Some("/url"));
//!
//! // Iterate over normalized events:
//! for event in stream {
//!     // ...
//! }
//! ```
//!
//! The current entry point is [`MarkdownParser`], which composes parser,
//! structure, and scanning responsibilities while this module evolves.

#![expect(
    clippy::pattern_type_mismatch,
    reason = "Parser pipeline matches borrowed event payloads intentionally"
)]

/// Block-domain types for parser AST.
pub(crate) mod block;
/// Configuration types for the event stream.
pub(crate) mod config;
/// Cached parsing context for markdown documents.
pub(crate) mod context;
/// Extracted reference link definitions.
pub(crate) mod references;
/// Event stream processing and normalization.
pub(crate) mod stream;
/// Block structure and AST types for markdown documents.
pub(crate) mod structure;
/// Derived inline text projection types.
pub(crate) mod text;
/// Parser-owned neutral event and payload types.
pub(crate) mod types;

#[cfg(test)]
#[path = "context_integration_test.rs"]
mod context_integration_test;

use pulldown_cmark::Options;

use crate::{
    config::task::TaskConfigSpec,
    note::{
        error::NoteIngestError,
        extractor::BlockExtractor,
        parser::{context::ParserContext, structure::DocTree},
        raw::RawNote,
        scanner::NoteScanner,
    },
};

// ── Primary public API ───────────────────────────────────────────────────────

/// Markdown parser for extracting note facts and structure.
#[non_exhaustive]
pub struct MarkdownParser;

impl MarkdownParser {
    /// Returns the pulldown-cmark option set used for Obsidian-compatible
    /// parsing.
    #[inline]
    #[must_use]
    pub fn extension_options() -> Options {
        config::EventStreamConfig::default_options()
    }

    /// Parses markdown into raw note artifacts.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] if markdown parsing or source position
    /// mapping fails.
    #[inline]
    pub fn parse<'source>(
        source: &'source str,
        task_spec: &TaskConfigSpec,
    ) -> Result<RawNote<'source>, NoteIngestError> {
        let config = config::EventStreamConfig::default();
        let ctx = ParserContext::new(source, config)?;
        let tree = DocTree::from_context(&ctx)?;

        let emoji_markers = if task_spec.use_emoji {
            task_spec.emoji_markers.clone()
        } else {
            Box::new([])
        };
        let scanner = NoteScanner::new(emoji_markers);
        let mut extractor = BlockExtractor::new(source, scanner);
        extractor.process_doc_tree(&tree)?;
        Ok(extractor.finish())
    }
}

// ── Private implementation details ───────────────────────────────────────────

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Tests prioritize readability with assertions"
)]
mod tests {
    use super::*;
    use crate::{
        config::task::TaskConfigSpec,
        note::{error::NoteError, raw::RawListDepth},
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

    fn parse_raw(markdown: &str) -> Result<RawNote<'_>, NoteIngestError> {
        let task_spec = task_spec_fixture();
        MarkdownParser::parse(markdown, &task_spec)
    }

    #[test]
    fn should_extract_block_ref_from_paragraph_tail()
    -> Result<(), NoteIngestError> {
        let md = "Paragraph text ^my-id";
        let raw = parse_raw(md)?;
        assert_eq!(raw.block_refs.len(), 1);
        assert_eq!(
            raw.block_refs
                .first()
                .ok_or(NoteError::Internal("missing block ref".into()))?
                .id,
            "my-id"
        );
        Ok(())
    }

    #[test]
    fn should_capture_yaml_at_start() -> Result<(), NoteIngestError> {
        let md = "---\ntags: [a]\n---\nContent";
        let raw = parse_raw(md)?;
        let fm = raw
            .frontmatter
            .as_ref()
            .ok_or(NoteError::Internal("frontmatter missing".into()))?;
        assert_eq!(fm.text, "tags: [a]\n");
        Ok(())
    }

    #[test]
    fn should_capture_tags_inside_heading() -> Result<(), NoteIngestError> {
        let md = "## Heading #tag";
        let raw = parse_raw(md)?;
        assert!(raw.tags.iter().any(|t| t.value == "#tag"));
        Ok(())
    }

    #[test]
    fn should_ignore_tags_inside_links() -> Result<(), NoteIngestError> {
        let md = "See [[target|#tag]] and [link #tag](http://example.test)";
        let raw = parse_raw(md)?;
        assert!(raw.tags.is_empty());
        Ok(())
    }

    #[test]
    fn should_ignore_block_refs_inside_links() -> Result<(), NoteIngestError> {
        let md = "See [link ^ref](http://example.test)";
        let raw = parse_raw(md)?;
        assert!(raw.block_refs.is_empty());
        Ok(())
    }

    #[test]
    fn should_detect_tag_after_link_label_gap() -> Result<(), NoteIngestError> {
        let md = "See [label](http://example.test) #tag";
        let raw = parse_raw(md)?;
        assert!(raw.tags.iter().any(|tag| tag.value == "#tag"));
        Ok(())
    }

    #[test]
    fn should_detect_block_ref_after_link_label_gap()
    -> Result<(), NoteIngestError> {
        let md = "See [label](http://example.test) ^my-id";
        let raw = parse_raw(md)?;
        assert!(raw.block_refs.iter().any(|block_ref| block_ref.id == "my-id"));
        Ok(())
    }

    #[test]
    fn should_not_scan_code_or_math_for_tags() -> Result<(), NoteIngestError> {
        let md = "`#hidden` $#hidden_math$ #visible";
        let raw = parse_raw(md)?;
        assert_eq!(raw.tags.len(), 1);
        assert_eq!(
            raw.tags.first().map(|tag| tag.value.as_ref()),
            Some("#visible")
        );
        Ok(())
    }

    #[test]
    fn should_extract_bare_fields() -> Result<(), NoteIngestError> {
        let md = "bare_key:: bare_val";
        let raw = parse_raw(md)?;
        let field = raw
            .inline_fields
            .first()
            .ok_or(NoteError::Internal("field missing".into()))?;
        assert_eq!(field.key, "bare_key");
        assert_eq!(field.value, "bare_val");
        Ok(())
    }

    #[test]
    fn should_handle_wikilinks() -> Result<(), NoteIngestError> {
        let md = "Check [[target]] and [[target|alias]]";
        let raw = parse_raw(md)?;
        assert_eq!(raw.links.len(), 2);
        assert_eq!(
            raw.links
                .first()
                .ok_or(NoteError::Internal("link missing".into()))?
                .text
                .target
                .as_ref(),
            "target"
        );
        Ok(())
    }

    #[test]
    fn should_track_list_nesting() -> Result<(), NoteIngestError> {
        let md = "- Parent\n  - Child";
        let raw = parse_raw(md)?;
        assert_eq!(raw.list_items.len(), 2);
        let mut sorted = raw.list_items.clone();
        sorted.sort_by_key(|i| i.range.start().as_usize());
        let child = sorted
            .get(1)
            .ok_or(NoteError::Internal("Child list item missing".into()))?;
        assert!(matches!(child.depth, RawListDepth::Nested(1)));
        Ok(())
    }

    #[test]
    fn should_capture_checkbox_state_and_marker() -> Result<(), NoteIngestError>
    {
        let md = "- [x] Done";
        let raw = parse_raw(md)?;
        let item = raw
            .list_items
            .first()
            .ok_or(NoteError::Internal("list item missing".into()))?;
        assert_eq!(item.is_checkbox, Some(true));
        Ok(())
    }

    #[test]
    fn should_extract_thematic_break() -> Result<(), NoteIngestError> {
        let md = "Paragraph\n\n---\n\nAnother paragraph";
        let raw = parse_raw(md)?;
        assert_eq!(raw.sections.len(), 3);
        assert!(matches!(
            raw.sections.get(1).map(|s| s.kind),
            Some(crate::note::raw::RawSectionKind::ThematicBreak)
        ));
        Ok(())
    }

    #[test]
    fn reference_definitions_first_wins() -> Result<(), NoteIngestError> {
        let md = "[ref]: http://a.example\n[ref]: http://b.example\n\n[ref][]";
        let raw = parse_raw(md)?;
        let link = raw
            .links
            .first()
            .ok_or(NoteError::Internal("link missing".into()))?;
        assert_eq!(link.text.target.as_ref(), "http://a.example");
        Ok(())
    }

    #[test]
    fn reference_definitions_are_case_insensitive()
    -> Result<(), NoteIngestError> {
        let md = "[Ref]: http://example.test\n\n[ref][]";
        let raw = parse_raw(md)?;
        let link = raw
            .links
            .first()
            .ok_or(NoteError::Internal("link missing".into()))?;
        assert_eq!(link.text.target.as_ref(), "http://example.test");
        Ok(())
    }

    #[test]
    fn reference_definitions_in_frontmatter_are_ignored()
    -> Result<(), NoteIngestError> {
        let md = "---\n[ref]: http://frontmatter.test\n---\n\n[ref][]";
        let raw = parse_raw(md)?;
        assert!(raw.links.is_empty());
        Ok(())
    }

    #[test]
    fn reference_definitions_in_fenced_code_are_ignored()
    -> Result<(), NoteIngestError> {
        let md = "```\n[ref]: http://code.test\n```\n\n[ref][]";
        let raw = parse_raw(md)?;
        assert!(raw.links.is_empty());
        Ok(())
    }

    #[test]
    fn reference_definitions_normalize_whitespace()
    -> Result<(), NoteIngestError> {
        let md = "[Foo   Bar]: http://example.test\n\n[foo bar][]";
        let raw = parse_raw(md)?;
        let link = raw
            .links
            .first()
            .ok_or(NoteError::Internal("link missing".into()))?;
        assert_eq!(link.text.target.as_ref(), "http://example.test");
        Ok(())
    }

    #[test]
    fn reference_definitions_unescape_labels() -> Result<(), NoteIngestError> {
        let md = "[Foo\\ Bar]: http://example.test\n\n[foo\\ bar][]";
        let raw = parse_raw(md)?;
        let link = raw
            .links
            .first()
            .ok_or(NoteError::Internal("link missing".into()))?;
        assert_eq!(link.text.target.as_ref(), "http://example.test");
        Ok(())
    }

    #[test]
    fn reference_definitions_allow_multiline_destination()
    -> Result<(), NoteIngestError> {
        let md = "[ref]:\n  http://example.test\n\n[ref][]";
        let raw = parse_raw(md)?;
        let link = raw
            .links
            .first()
            .ok_or(NoteError::Internal("link missing".into()))?;
        assert_eq!(link.text.target.as_ref(), "http://example.test");
        Ok(())
    }

    #[test]
    fn external_scheme_targets_preserve_fragments()
    -> Result<(), NoteIngestError> {
        let md = "[obsidian](obsidian://open?vault=V#frag)";
        let raw = parse_raw(md)?;
        let link = raw
            .links
            .first()
            .ok_or(NoteError::Internal("link missing".into()))?;
        assert_eq!(link.text.target.as_ref(), "obsidian://open?vault=V#frag");
        Ok(())
    }

    #[test]
    fn file_scheme_targets_preserve_fragments() -> Result<(), NoteIngestError> {
        let md = "[file](file:///Users/example/test.md#section)";
        let raw = parse_raw(md)?;
        let link = raw
            .links
            .first()
            .ok_or(NoteError::Internal("link missing".into()))?;
        assert_eq!(
            link.text.target.as_ref(),
            "file:///Users/example/test.md#section"
        );
        Ok(())
    }

    #[test]
    fn s3_scheme_targets_preserve_fragments() -> Result<(), NoteIngestError> {
        let md = "[s3](s3://bucket/key#object)";
        let raw = parse_raw(md)?;
        let link = raw
            .links
            .first()
            .ok_or(NoteError::Internal("link missing".into()))?;
        assert_eq!(link.text.target.as_ref(), "s3://bucket/key#object");
        Ok(())
    }
}
