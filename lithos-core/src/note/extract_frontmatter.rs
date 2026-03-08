//! Frontmatter extraction from markdown event streams.
//!
//! Extracts YAML/TOML metadata blocks, preserves line breaks in block scalar
//! content, and converts parsed values into domain `FieldValue` entries.

use std::ops::Range;

use pulldown_cmark::{Event, MetadataBlockKind, Tag as CmarkTag, TagEnd};

use super::reader::{ExtractionContext, ExtractionState, Extractor};
use crate::note::{
    error::NoteError,
    frontmatter::{Frontmatter, FrontmatterFormat},
};

/// Extractor for YAML/TOML frontmatter blocks.
pub struct FrontmatterExtractor {
    kind: Option<MetadataBlockKind>,
    text: String,
}

impl FrontmatterExtractor {
    #[inline]
    pub(super) const fn new() -> Self {
        Self {
            kind: None,
            text: String::new(),
        }
    }

    fn start(&mut self, kind: MetadataBlockKind) {
        self.kind = Some(kind);
        self.text.clear();
    }

    fn push_text(&mut self, text: &str) {
        self.text.push_str(text);
    }

    fn push_break(&mut self) {
        self.text.push('\n');
    }

    fn end(
        &mut self,
        kind: MetadataBlockKind,
    ) -> Result<Option<Frontmatter>, NoteError> {
        if self.kind != Some(kind) {
            self.kind = None;
            self.text.clear();
            return Ok(None);
        }
        self.kind = None;

        if self.text.is_empty() {
            return Ok(None);
        }

        let format = match kind {
            MetadataBlockKind::YamlStyle => FrontmatterFormat::Yaml,
            MetadataBlockKind::PlusesStyle => FrontmatterFormat::Toml,
        };

        let frontmatter = Frontmatter::parse(format, &self.text)
            .map_err(NoteError::Frontmatter)?;
        self.text.clear();
        Ok(Some(frontmatter))
    }
}

impl Extractor for FrontmatterExtractor {
    type Error = NoteError;
    type Output = Frontmatter;

    fn finish(self) -> Result<Vec<Frontmatter>, NoteError> {
        Ok(Vec::new())
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &Event preferred for clarity"
    )]
    fn process(
        &mut self,
        event: &Event<'_>,
        text: &str,
        _range: &Range<usize>,
        _ctx: &ExtractionContext,
    ) -> Result<ExtractionState<Frontmatter>, NoteError> {
        match event {
            Event::Start(CmarkTag::MetadataBlock(kind)) => {
                self.start(*kind);
                Ok(ExtractionState::Continue)
            }
            Event::End(TagEnd::MetadataBlock(kind)) => {
                if let Some(frontmatter) = self.end(*kind)? {
                    return Ok(ExtractionState::Emit(frontmatter));
                }
                Ok(ExtractionState::Continue)
            }
            Event::Text(_) => {
                if self.kind.is_some() {
                    self.push_text(text);
                }
                Ok(ExtractionState::Continue)
            }
            Event::SoftBreak | Event::HardBreak => {
                if self.kind.is_some() {
                    self.push_break();
                }
                Ok(ExtractionState::Continue)
            }
            Event::Start(_)
            | Event::End(_)
            | Event::Code(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
            | Event::Rule
            | Event::TaskListMarker(_) => Ok(ExtractionState::Continue),
        }
    }
}

#[cfg(test)]
mod tests {
    use pulldown_cmark::{Event, MetadataBlockKind, Tag as CmarkTag, TagEnd};

    use super::*;

    #[test]
    fn parses_yaml_frontmatter() {
        let mut extractor = FrontmatterExtractor::new();
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Start(CmarkTag::MetadataBlock(
                    MetadataBlockKind::YamlStyle,
                )),
                "",
                &(0..3),
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::Text("title: Test\ncount: 2".into()),
                "title: Test\ncount: 2",
                &(3..24),
                &ctx,
            )
            .unwrap();

        let result = extractor
            .process(
                &Event::End(TagEnd::MetadataBlock(
                    MetadataBlockKind::YamlStyle,
                )),
                "",
                &(24..27),
                &ctx,
            )
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(frontmatter) = result else {
            panic!("Expected frontmatter emission");
        };

        assert!(frontmatter.has_raw("title"));
        assert!(frontmatter.has_raw("count"));
    }

    #[test]
    fn parses_toml_frontmatter() {
        let mut extractor = FrontmatterExtractor::new();
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Start(CmarkTag::MetadataBlock(
                    MetadataBlockKind::PlusesStyle,
                )),
                "",
                &(0..3),
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::Text("title = \"Test\"\ncount = 2".into()),
                "title = \"Test\"\ncount = 2",
                &(3..30),
                &ctx,
            )
            .unwrap();

        let result = extractor
            .process(
                &Event::End(TagEnd::MetadataBlock(
                    MetadataBlockKind::PlusesStyle,
                )),
                "",
                &(30..33),
                &ctx,
            )
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(frontmatter) = result else {
            panic!("Expected frontmatter emission");
        };

        assert!(frontmatter.has_raw("title"));
        assert!(frontmatter.has_raw("count"));
    }

    #[test]
    fn rejects_non_mapping_yaml() {
        let mut extractor = FrontmatterExtractor::new();
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Start(CmarkTag::MetadataBlock(
                    MetadataBlockKind::YamlStyle,
                )),
                "",
                &(0..3),
                &ctx,
            )
            .unwrap();

        extractor
            .process(&Event::Text("- item".into()), "- item", &(3..9), &ctx)
            .unwrap();

        let result = extractor.process(
            &Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle)),
            "",
            &(9..12),
            &ctx,
        );

        let _err: NoteError = result.unwrap_err();
    }

    #[test]
    fn rejects_non_table_toml() {
        let mut extractor = FrontmatterExtractor::new();
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Start(CmarkTag::MetadataBlock(
                    MetadataBlockKind::PlusesStyle,
                )),
                "",
                &(0..3),
                &ctx,
            )
            .unwrap();

        extractor
            .process(&Event::Text("[[]]".into()), "[[]]", &(3..7), &ctx)
            .unwrap();

        let result = extractor.process(
            &Event::End(TagEnd::MetadataBlock(MetadataBlockKind::PlusesStyle)),
            "",
            &(7..10),
            &ctx,
        );

        let _err: NoteError = result.unwrap_err();
    }
}
