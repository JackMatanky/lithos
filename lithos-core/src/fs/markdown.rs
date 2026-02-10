//! Markdown parsing utilities for adapter layers.
//!
//! Provides a thin wrapper over pulldown-cmark to keep markdown parsing
//! concerns in filesystem infrastructure.

use pulldown_cmark::{Options, Parser};

/// Offset-aware markdown iterator type.
pub type MarkdownOffsetIter<'markdown> = pulldown_cmark::OffsetIter<'markdown>;

/// Markdown parser configuration wrapper.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct MarkdownParser {
    options: Options,
}

impl MarkdownParser {
    /// Create a new markdown parser with the provided options.
    #[inline]
    #[must_use]
    pub const fn new(options: Options) -> Self {
        Self {
            options,
        }
    }

    /// Create a parser that enables task list markers.
    #[inline]
    #[must_use]
    pub const fn with_tasklists() -> Self {
        Self {
            options: Options::ENABLE_TASKLISTS,
        }
    }

    /// Create a parser with full Obsidian feature support.
    ///
    /// Enables:
    /// - `WikiLinks`: `[[link]]`, `[[link|alias]]`, `![[embed]]`
    /// - Frontmatter: YAML metadata blocks
    /// - Tables: GFM tables
    /// - Footnotes: Markdown footnotes
    /// - Math: Inline `$...$` and display `$$...$$`
    /// - Strikethrough: `~~text~~`
    /// - Heading Attributes: `# Title {#id .class}`
    /// - Task Lists: `- [ ] task`
    #[inline]
    #[must_use]
    pub const fn with_obsidian_features() -> Self {
        Self {
            options: Options::ENABLE_TASKLISTS
                .union(Options::ENABLE_WIKILINKS)
                .union(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS)
                .union(Options::ENABLE_HEADING_ATTRIBUTES)
                .union(Options::ENABLE_TABLES)
                .union(Options::ENABLE_FOOTNOTES)
                .union(Options::ENABLE_STRIKETHROUGH)
                .union(Options::ENABLE_MATH),
        }
    }

    /// Return the underlying pulldown-cmark options.
    #[inline]
    #[must_use]
    pub const fn options(&self) -> Options {
        self.options
    }

    /// Parse markdown into offset-aware events.
    #[inline]
    #[must_use]
    pub fn parse_offsets<'markdown>(
        &self,
        markdown: &'markdown str,
    ) -> MarkdownOffsetIter<'markdown> {
        Parser::new_ext(markdown, self.options).into_offset_iter()
    }
}

#[cfg(test)]
mod tests {
    use pulldown_cmark::Event;

    use super::*;

    #[test]
    fn parse_offsets_exposes_text_ranges() {
        let parser = MarkdownParser::with_tasklists();
        let markdown = "Hello";
        let iter = parser.parse_offsets(markdown);

        let mut found_text = false;
        for (event, range) in iter {
            if let Event::Text(text) = event {
                assert_eq!(text.as_ref(), "Hello");
                assert_eq!(range.start, 0);
                assert_eq!(range.end, 5);
                found_text = true;
                break;
            }
        }

        assert!(found_text, "expected to find text event");
    }
}
