//! Heading extraction from markdown event streams.
//!
//! Extracts headings (H1-H6), captures their source offset, and accumulates
//! text across nested events while normalizing soft/hard breaks to spaces.

use std::ops::Range;

use pulldown_cmark::{
    Event, HeadingLevel as CmarkHeadingLevel, Tag as CmarkTag, TagEnd,
};

use super::reader::{ExtractionContext, ExtractionState, Extractor};
use crate::note::{
    error::NoteError,
    heading::{Heading, HeadingBuilder, HeadingLevel},
    position::SourceByteOffset,
};

/// Extractor for markdown headings (H1-H6).
pub struct HeadingExtractor {
    current: Option<HeadingBuilder>,
}

impl HeadingExtractor {
    #[inline]
    pub(super) const fn new() -> Self {
        Self {
            current: None,
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

        self.current = Some(HeadingBuilder::new(level, position));
        Ok(())
    }
}

impl Extractor for HeadingExtractor {
    type Error = NoteError;
    type Output = Heading;

    fn finish(self) -> Result<Vec<Heading>, NoteError> {
        // No incomplete headings to flush
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
        range: &Range<usize>,
        _ctx: &ExtractionContext,
    ) -> Result<ExtractionState<Heading>, NoteError> {
        match event {
            Event::Start(CmarkTag::Heading {
                level,
                ..
            }) => {
                let position = SourceByteOffset::try_from_usize(range.start)?;
                self.start_heading(*level, position)?;
                Ok(ExtractionState::Continue)
            }

            Event::Text(_) | Event::Code(_) => {
                if let Some(builder) = self.current.as_mut() {
                    builder.push_text(text);
                }
                Ok(ExtractionState::Continue)
            }

            Event::SoftBreak | Event::HardBreak => {
                if let Some(builder) = self.current.as_mut() {
                    builder.push_break();
                }
                Ok(ExtractionState::Continue)
            }

            Event::End(TagEnd::Heading(_)) => {
                if let Some(builder) = self.current.take() {
                    let heading = builder.build()?;
                    return Ok(ExtractionState::Emit(heading));
                }
                Ok(ExtractionState::Continue)
            }

            Event::Start(_)
            | Event::End(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
            | Event::Rule
            | Event::TaskListMarker(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_) => Ok(ExtractionState::Continue),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test organization groups proptests with unit tests"
)]
mod tests {
    use pulldown_cmark::{CowStr, Event, Tag as CmarkTag, TagEnd};

    use super::*;

    mod proptests {
        use proptest::prelude::*;

        use super::*;

        #[expect(
            clippy::arithmetic_side_effects,
            reason = "Test ranges use bounded string lengths"
        )]
        #[expect(clippy::panic, reason = "Test assertion")]
        mod ranges {
            use super::*;

            fn heading_level(level: u8) -> CmarkHeadingLevel {
                match level {
                    1 => CmarkHeadingLevel::H1,
                    2 => CmarkHeadingLevel::H2,
                    3 => CmarkHeadingLevel::H3,
                    4 => CmarkHeadingLevel::H4,
                    5 => CmarkHeadingLevel::H5,
                    _ => CmarkHeadingLevel::H6,
                }
            }

            fn extract_heading(level: u8, text: &str) -> Heading {
                let mut extractor = HeadingExtractor::new();
                let ctx = ExtractionContext::default();
                let heading_level = heading_level(level);

                extractor
                    .process(
                        &Event::Start(CmarkTag::Heading {
                            level: heading_level,
                            id: None,
                            classes: Vec::new(),
                            attrs: Vec::new(),
                        }),
                        "",
                        &(0..1),
                        &ctx,
                    )
                    .unwrap();

                let start = 1usize;
                let end = start + text.len();
                extractor
                    .process(
                        &Event::Text(text.into()),
                        text,
                        &(start..end),
                        &ctx,
                    )
                    .unwrap();

                let result = extractor
                    .process(
                        &Event::End(TagEnd::Heading(heading_level)),
                        "",
                        &(end..(end + 1)),
                        &ctx,
                    )
                    .unwrap();

                match result {
                    ExtractionState::Emit(heading) => heading,
                    ExtractionState::Continue => {
                        panic!("Expected heading emission")
                    }
                }
            }

            proptest! {
                #[test]
                fn extracts_heading_text_verbatim(
                    level in 1u8..=6,
                    text in "[A-Za-z0-9_-]{1,40}",
                ) {
                    let heading = extract_heading(level, &text);
                    prop_assert_eq!(heading.text(), text);
                    prop_assert_eq!(heading.level().as_u8(), level);
                }
            }

            proptest! {
                #[test]
                fn converts_soft_breaks_to_space(
                    level in 1u8..=6,
                    left in "[A-Za-z0-9_-]{1,20}",
                    right in "[A-Za-z0-9_-]{1,20}",
                ) {
                    let mut extractor = HeadingExtractor::new();
                    let ctx = ExtractionContext::default();
                    let heading_level = heading_level(level);

                    extractor
                        .process(
                            &Event::Start(CmarkTag::Heading {
                                level: heading_level,
                                id: None,
                                classes: Vec::new(),
                                attrs: Vec::new(),
                            }),
                            "",
                            &(0..1),
                            &ctx,
                        )
                        .unwrap();

                    let left_start = 1usize;
                    let left_end = left_start + left.len();
                    extractor
                        .process(
                            &Event::Text(left.as_str().into()),
                            left.as_str(),
                            &(left_start..left_end),
                            &ctx,
                        )
                        .unwrap();

                    let break_start = left_end;
                    let break_end = break_start + 1;
                    extractor
                        .process(
                            &Event::SoftBreak,
                            "",
                            &(break_start..break_end),
                            &ctx,
                        )
                        .unwrap();

                    let right_start = break_end;
                    let right_end = right_start + right.len();
                    extractor
                        .process(
                            &Event::Text(right.as_str().into()),
                            right.as_str(),
                            &(right_start..right_end),
                            &ctx,
                        )
                        .unwrap();

                    let result = extractor
                        .process(
                            &Event::End(TagEnd::Heading(heading_level)),
                            "",
                            &(right_end..(right_end + 1)),
                            &ctx,
                        )
                        .unwrap();

                    let heading = match result {
                        ExtractionState::Emit(heading) => heading,
                        ExtractionState::Continue => {
                            panic!("Expected heading emission")
                        }
                    };

                    let expected = format!("{left} {right}");
                    prop_assert_eq!(heading.text(), expected);
                }
            }
        }
    }

    #[test]
    fn extracts_h1_through_h6() {
        let mut extractor = HeadingExtractor::new();
        let ctx = ExtractionContext::default();

        for (level, expected) in [
            (CmarkHeadingLevel::H1, 1),
            (CmarkHeadingLevel::H2, 2),
            (CmarkHeadingLevel::H3, 3),
            (CmarkHeadingLevel::H4, 4),
            (CmarkHeadingLevel::H5, 5),
            (CmarkHeadingLevel::H6, 6),
        ] {
            extractor
                .process(
                    &Event::Start(CmarkTag::Heading {
                        level,
                        id: None,
                        classes: Vec::new(),
                        attrs: Vec::new(),
                    }),
                    "",
                    &(0..1),
                    &ctx,
                )
                .unwrap();

            extractor
                .process(&Event::Text("Title".into()), "Title", &(1..6), &ctx)
                .unwrap();

            let result = extractor
                .process(&Event::End(TagEnd::Heading(level)), "", &(6..7), &ctx)
                .unwrap();

            #[expect(clippy::panic, reason = "Test assertion")]
            let ExtractionState::Emit(heading) = result else {
                panic!("Expected heading emission");
            };

            assert_eq!(heading.level().as_u8(), expected);
            assert_eq!(heading.text(), "Title");
        }
    }

    #[test]
    fn accumulates_text_across_events() {
        let mut extractor = HeadingExtractor::new();
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Start(CmarkTag::Heading {
                    level: CmarkHeadingLevel::H2,
                    id: None,
                    classes: Vec::new(),
                    attrs: Vec::new(),
                }),
                "",
                &(0..1),
                &ctx,
            )
            .unwrap();

        extractor
            .process(&Event::Text("Hello ".into()), "Hello ", &(1..7), &ctx)
            .unwrap();

        extractor
            .process(&Event::Text("World".into()), "World", &(7..12), &ctx)
            .unwrap();

        let result = extractor
            .process(
                &Event::End(TagEnd::Heading(CmarkHeadingLevel::H2)),
                "",
                &(12..13),
                &ctx,
            )
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(heading) = result else {
            panic!("Expected heading emission");
        };

        assert_eq!(heading.text(), "Hello World");
    }

    #[test]
    fn converts_breaks_to_spaces() {
        let mut extractor = HeadingExtractor::new();
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Start(CmarkTag::Heading {
                    level: CmarkHeadingLevel::H3,
                    id: None,
                    classes: Vec::new(),
                    attrs: Vec::new(),
                }),
                "",
                &(0..1),
                &ctx,
            )
            .unwrap();

        extractor
            .process(&Event::Text("Hello".into()), "Hello", &(1..6), &ctx)
            .unwrap();

        extractor.process(&Event::SoftBreak, "", &(6..7), &ctx).unwrap();

        extractor
            .process(&Event::Text("World".into()), "World", &(7..12), &ctx)
            .unwrap();

        let result = extractor
            .process(
                &Event::End(TagEnd::Heading(CmarkHeadingLevel::H3)),
                "",
                &(12..13),
                &ctx,
            )
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(heading) = result else {
            panic!("Expected heading emission");
        };

        assert_eq!(heading.text(), "Hello World");
    }

    #[test]
    fn handles_empty_heading() {
        let mut extractor = HeadingExtractor::new();
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Start(CmarkTag::Heading {
                    level: CmarkHeadingLevel::H1,
                    id: None,
                    classes: Vec::new(),
                    attrs: Vec::new(),
                }),
                "",
                &(0..1),
                &ctx,
            )
            .unwrap();

        let result = extractor.process(
            &Event::End(TagEnd::Heading(CmarkHeadingLevel::H1)),
            "",
            &(1..2),
            &ctx,
        );

        let _err: NoteError = result.unwrap_err();
    }

    #[test]
    fn handles_heading_with_code() {
        let mut extractor = HeadingExtractor::new();
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Start(CmarkTag::Heading {
                    level: CmarkHeadingLevel::H2,
                    id: None,
                    classes: Vec::new(),
                    attrs: Vec::new(),
                }),
                "",
                &(0..1),
                &ctx,
            )
            .unwrap();

        extractor
            .process(&Event::Code("code".into()), "code", &(1..7), &ctx)
            .unwrap();

        let result = extractor
            .process(
                &Event::End(TagEnd::Heading(CmarkHeadingLevel::H2)),
                "",
                &(7..8),
                &ctx,
            )
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(heading) = result else {
            panic!("Expected heading emission");
        };

        assert_eq!(heading.text(), "code");
    }

    #[test]
    fn handles_heading_with_link() {
        let mut extractor = HeadingExtractor::new();
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Start(CmarkTag::Heading {
                    level: CmarkHeadingLevel::H2,
                    id: None,
                    classes: Vec::new(),
                    attrs: Vec::new(),
                }),
                "",
                &(0..1),
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::Start(CmarkTag::Link {
                    link_type: pulldown_cmark::LinkType::Inline,
                    dest_url: CowStr::Borrowed("note"),
                    title: CowStr::Borrowed(""),
                    id: CowStr::Borrowed(""),
                }),
                "",
                &(1..2),
                &ctx,
            )
            .unwrap();

        extractor
            .process(&Event::Text("Link".into()), "Link", &(2..6), &ctx)
            .unwrap();

        extractor
            .process(&Event::End(TagEnd::Link), "", &(6..7), &ctx)
            .unwrap();

        let result = extractor
            .process(
                &Event::End(TagEnd::Heading(CmarkHeadingLevel::H2)),
                "",
                &(7..8),
                &ctx,
            )
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(heading) = result else {
            panic!("Expected heading emission");
        };

        assert_eq!(heading.text(), "Link");
    }

    #[test]
    fn handles_unclosed_heading() {
        let mut extractor = HeadingExtractor::new();
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Start(CmarkTag::Heading {
                    level: CmarkHeadingLevel::H1,
                    id: None,
                    classes: Vec::new(),
                    attrs: Vec::new(),
                }),
                "",
                &(0..1),
                &ctx,
            )
            .unwrap();

        let pending = extractor.finish().unwrap();
        assert!(pending.is_empty());
    }
}
