//! Section extraction from markdown event streams.
//!
//! Tracks block boundaries to build top-level sections with source ranges and
//! associates the first heading in a block with the section when present.

use std::ops::Range;

use pulldown_cmark::{
    CowStr, Event, HeadingLevel as CmarkHeadingLevel, Tag as CmarkTag, TagEnd,
};

use super::reader::{ExtractionContext, ExtractionState, Extractor};
use crate::note::{
    error::NoteError,
    position::{SourceByteOffset, SourceByteRange},
    structure::{Heading, HeadingLevel, Section},
};

/// Extractor for markdown sections.
pub struct SectionExtractor<'source> {
    source: &'source str,
    block_depth: u32,
    current: Option<SectionState>,
    last_offset: usize,
    current_heading: Option<HeadingBuilder>,
    sections: Vec<Section>,
}

#[derive(Debug)]
struct SectionState {
    start: SourceByteOffset,
    heading: Option<Heading>,
    awaiting_heading: bool,
}

struct HeadingBuilder {
    level: HeadingLevel,
    text: String,
    position: SourceByteOffset,
}

impl HeadingBuilder {
    fn new(level: HeadingLevel, position: SourceByteOffset) -> Self {
        Self {
            level,
            text: String::new(),
            position,
        }
    }

    fn push_text(&mut self, text: &str) {
        self.text.push_str(text);
    }

    fn push_break(&mut self) {
        self.text.push(' ');
    }

    fn build(self) -> Result<Heading, NoteError> {
        Heading::try_new(self.level, self.text, self.position)
    }
}

impl<'source> SectionExtractor<'source> {
    #[inline]
    pub(super) const fn new(source: &'source str) -> Self {
        Self {
            source,
            block_depth: 0,
            current: None,
            last_offset: 0,
            current_heading: None,
            sections: Vec::new(),
        }
    }

    fn update_last_offset(&mut self, offset: usize) {
        self.last_offset = offset;
    }

    fn start_block(
        &mut self,
        position: usize,
        is_heading: bool,
    ) -> Result<(), NoteError> {
        if self.block_depth == 0 {
            let start = SourceByteOffset::try_from_usize(position)?;
            self.current = Some(SectionState {
                start,
                heading: None,
                awaiting_heading: is_heading,
            });
        }
        self.block_depth = self.block_depth.saturating_add(1);
        Ok(())
    }

    fn end_block(&mut self) -> Result<(), NoteError> {
        if self.block_depth == 0 {
            return Ok(());
        }
        self.block_depth = self.block_depth.saturating_sub(1);
        if self.block_depth == 0 {
            self.close_current()?;
        }
        Ok(())
    }

    fn add_rule_section(
        &mut self,
        start: usize,
        end: usize,
    ) -> Result<(), NoteError> {
        if self.block_depth != 0 {
            return Ok(());
        }
        let start = SourceByteOffset::try_from_usize(start)?;
        let end = SourceByteOffset::try_from_usize(end)?;
        self.push_section(None, start, end)
    }

    fn maybe_assign_heading(&mut self, heading: &Heading) {
        if let Some(section) = self.current.as_mut()
            && section.awaiting_heading
        {
            section.heading = Some(heading.clone());
            section.awaiting_heading = false;
        }
    }

    fn start_heading(
        &mut self,
        level: CmarkHeadingLevel,
        position: SourceByteOffset,
    ) -> Result<(), NoteError> {
        let level = match level {
            CmarkHeadingLevel::H1 => HeadingLevel::try_new(1)?,
            CmarkHeadingLevel::H2 => HeadingLevel::try_new(2)?,
            CmarkHeadingLevel::H3 => HeadingLevel::try_new(3)?,
            CmarkHeadingLevel::H4 => HeadingLevel::try_new(4)?,
            CmarkHeadingLevel::H5 => HeadingLevel::try_new(5)?,
            CmarkHeadingLevel::H6 => HeadingLevel::try_new(6)?,
        };

        self.current_heading = Some(HeadingBuilder::new(level, position));
        Ok(())
    }

    fn end_heading(&mut self) -> Result<(), NoteError> {
        let Some(builder) = self.current_heading.take() else {
            return Ok(());
        };
        let heading = builder.build()?;
        self.maybe_assign_heading(&heading);
        Ok(())
    }

    fn close_current(&mut self) -> Result<(), NoteError> {
        let Some(section) = self.current.take() else {
            return Ok(());
        };
        let end = SourceByteOffset::try_from_usize(self.last_offset)?;
        self.push_section(section.heading, section.start, end)
    }

    fn push_section(
        &mut self,
        heading: Option<Heading>,
        start: SourceByteOffset,
        end: SourceByteOffset,
    ) -> Result<(), NoteError> {
        let range = SourceByteRange::new(start, end)?;
        let start = usize::from(start);
        let end = usize::from(end);
        self.source.get(start..end).ok_or({
            NoteError::Structure("section range is not on a boundary")
        })?;
        self.sections.push(Section::new(heading, range));
        Ok(())
    }

    fn close(&mut self) -> Result<(), NoteError> {
        if self.current.is_some() {
            self.close_current()?;
        }
        Ok(())
    }

    fn take_sections(self) -> Vec<Section> {
        self.sections
    }

    fn is_block_tag(tag: &CmarkTag<'_>) -> bool {
        matches!(
            tag,
            CmarkTag::List(_)
                | CmarkTag::Heading { .. }
                | CmarkTag::Paragraph
                | CmarkTag::BlockQuote(_)
                | CmarkTag::CodeBlock(_)
                | CmarkTag::HtmlBlock
                | CmarkTag::FootnoteDefinition(_)
                | CmarkTag::DefinitionList
                | CmarkTag::Table(_)
        )
    }
}

impl Extractor for SectionExtractor<'_> {
    type Error = NoteError;
    type Output = Section;

    fn finish(mut self) -> Result<Vec<Section>, NoteError> {
        self.close()?;
        Ok(self.take_sections())
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &Event preferred for clarity"
    )]
    fn process(
        &mut self,
        event: &Event<'_>,
        text: CowStr<'_>,
        range: Range<usize>,
        _ctx: &ExtractionContext,
    ) -> Result<ExtractionState<Section>, NoteError> {
        self.update_last_offset(range.end);

        match event {
            Event::Start(tag) => {
                if Self::is_block_tag(tag) {
                    let is_heading = matches!(tag, CmarkTag::Heading { .. });
                    self.start_block(range.start, is_heading)?;
                }
                if let CmarkTag::Heading {
                    level,
                    ..
                } = tag
                {
                    let position =
                        SourceByteOffset::try_from_usize(range.start)?;
                    self.start_heading(*level, position)?;
                }
                Ok(ExtractionState::Continue)
            }

            Event::End(tag_end) => {
                let mut close_block = false;
                match tag_end {
                    TagEnd::Heading(_) => {
                        self.end_heading()?;
                        close_block = true;
                    }
                    TagEnd::List(_)
                    | TagEnd::Paragraph
                    | TagEnd::BlockQuote(_)
                    | TagEnd::HtmlBlock
                    | TagEnd::FootnoteDefinition
                    | TagEnd::DefinitionList
                    | TagEnd::Table
                    | TagEnd::CodeBlock => {
                        close_block = true;
                    }
                    TagEnd::Item
                    | TagEnd::DefinitionListTitle
                    | TagEnd::DefinitionListDefinition
                    | TagEnd::TableHead
                    | TagEnd::TableRow
                    | TagEnd::TableCell
                    | TagEnd::Emphasis
                    | TagEnd::Strong
                    | TagEnd::Strikethrough
                    | TagEnd::Superscript
                    | TagEnd::Subscript
                    | TagEnd::Link
                    | TagEnd::Image
                    | TagEnd::MetadataBlock(_) => {}
                }

                if close_block {
                    self.end_block()?;
                }
                Ok(ExtractionState::Continue)
            }

            Event::Text(_) | Event::Code(_) => {
                if let Some(builder) = self.current_heading.as_mut() {
                    builder.push_text(&text);
                }
                Ok(ExtractionState::Continue)
            }

            Event::SoftBreak | Event::HardBreak => {
                if let Some(builder) = self.current_heading.as_mut() {
                    builder.push_break();
                }
                Ok(ExtractionState::Continue)
            }

            Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
            | Event::Rule
            | Event::TaskListMarker(_) => {
                if matches!(event, Event::Rule) {
                    self.add_rule_section(range.start, range.end)?;
                }
                Ok(ExtractionState::Continue)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pulldown_cmark::{CowStr, Event, Tag as CmarkTag, TagEnd};

    use super::*;

    #[test]
    fn extracts_section_for_paragraph() {
        let source = "Paragraph text";
        let mut extractor = SectionExtractor::new(source);
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Start(CmarkTag::Paragraph),
                CowStr::Borrowed(""),
                0..1,
                &ctx,
            )
            .unwrap();

        let end = source.len();
        extractor
            .process(
                &Event::Text(CowStr::Borrowed("Paragraph text")),
                CowStr::Borrowed("Paragraph text"),
                1..end,
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::End(TagEnd::Paragraph),
                CowStr::Borrowed(""),
                end..end,
                &ctx,
            )
            .unwrap();

        let sections = extractor.finish().unwrap();
        let section = sections.first().expect("section should exist");
        assert!(section.heading().is_none());
        assert_eq!(section.range().start(), SourceByteOffset::new(0));
        let end_offset =
            SourceByteOffset::try_from_usize(end).expect("end offset");
        assert_eq!(section.range().end(), end_offset);
    }

    #[test]
    fn associates_heading_with_section() {
        let source = "# Heading";
        let mut extractor = SectionExtractor::new(source);
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Start(CmarkTag::Heading {
                    level: CmarkHeadingLevel::H1,
                    id: None,
                    classes: Vec::new(),
                    attrs: Vec::new(),
                }),
                CowStr::Borrowed(""),
                0..1,
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::Text(CowStr::Borrowed("Heading")),
                CowStr::Borrowed("Heading"),
                2..9,
                &ctx,
            )
            .unwrap();

        let end = source.len();
        extractor
            .process(
                &Event::End(TagEnd::Heading(CmarkHeadingLevel::H1)),
                CowStr::Borrowed(""),
                end..end,
                &ctx,
            )
            .unwrap();

        let sections = extractor.finish().unwrap();
        let section = sections.first().expect("section should exist");
        let heading = section.heading().expect("heading assigned");
        assert_eq!(heading.text(), "Heading");
    }

    #[test]
    fn adds_rule_section() {
        let source = "---";
        let mut extractor = SectionExtractor::new(source);
        let ctx = ExtractionContext::default();

        extractor
            .process(&Event::Rule, CowStr::Borrowed(""), 0..3, &ctx)
            .unwrap();

        let sections = extractor.finish().unwrap();
        let section = sections.first().expect("section should exist");
        assert_eq!(section.range().start(), SourceByteOffset::new(0));
        assert_eq!(section.range().end(), SourceByteOffset::new(3));
    }

    #[test]
    fn rejects_invalid_utf8_boundary() {
        let source = "h\u{e9}"; // 'e' with accent is two bytes
        let mut extractor = SectionExtractor::new(source);
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Start(CmarkTag::Paragraph),
                CowStr::Borrowed(""),
                0..1,
                &ctx,
            )
            .unwrap();

        // Set last_offset to an invalid UTF-8 boundary (1 byte into 'é')
        extractor
            .process(
                &Event::Text(CowStr::Borrowed("h\u{e9}")),
                CowStr::Borrowed("h\u{e9}"),
                0..2,
                &ctx,
            )
            .unwrap();

        let result = extractor.process(
            &Event::End(TagEnd::Paragraph),
            CowStr::Borrowed(""),
            2..2,
            &ctx,
        );

        let _err: NoteError = result.unwrap_err();
    }

    #[test]
    fn closes_section_at_eof() {
        let source = "Trailing";
        let mut extractor = SectionExtractor::new(source);
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Start(CmarkTag::Paragraph),
                CowStr::Borrowed(""),
                0..1,
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::Text(CowStr::Borrowed("Trailing")),
                CowStr::Borrowed("Trailing"),
                1..source.len(),
                &ctx,
            )
            .unwrap();

        // No end tag; finish should close section.
        let sections = extractor.finish().unwrap();
        let section = sections.first().expect("section should exist");
        let end_offset =
            SourceByteOffset::try_from_usize(source.len()).expect("end offset");
        assert_eq!(section.range().end(), end_offset);
    }

    #[test]
    fn tracks_block_depth_for_lists() {
        let source = "- item";
        let mut extractor = SectionExtractor::new(source);
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Start(CmarkTag::List(None)),
                CowStr::Borrowed(""),
                0..1,
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::Start(CmarkTag::Item),
                CowStr::Borrowed(""),
                1..2,
                &ctx,
            )
            .unwrap();

        let end = source.len();
        extractor
            .process(
                &Event::Text(CowStr::Borrowed("item")),
                CowStr::Borrowed("item"),
                2..end,
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::End(TagEnd::Item),
                CowStr::Borrowed(""),
                end..end,
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::End(TagEnd::List(false)),
                CowStr::Borrowed(""),
                end..end,
                &ctx,
            )
            .unwrap();

        let sections = extractor.finish().unwrap();
        assert_eq!(sections.len(), 1);
    }
}
