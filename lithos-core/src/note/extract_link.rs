//! Link extraction from markdown event streams.
//!
//! Extracts wiki-links, markdown links, embeds, and anchors while preserving
//! alias text. Internal links are split into target path plus anchor, and
//! external links are detected by scheme.

use std::ops::Range;

use pulldown_cmark::{
    CowStr, Event, LinkType as PLinkType, Tag as CmarkTag, TagEnd,
};

use super::reader::{ExtractionContext, ExtractionState, Extractor};
use crate::{
    config::aggregate::Config,
    note::{
        error::NoteError,
        link::{AliasMode, EmbedState, Link, LinkBuilder, Style},
        position::SourceByteOffset,
    },
};

/// Extractor for markdown links, wiki-links, and embeds.
///
/// Processes markdown link events and builds domain `Link` entities.
/// Handles both wiki-style (`[[target]]`) and markdown-style (`[text](url)`)
/// links. Also extracts embeds (prefixed with `!`).
pub struct LinkExtractor<'config> {
    #[expect(
        dead_code,
        reason = "Reserved for future link resolution features"
    )]
    config: &'config Config,
    current: Option<LinkBuilder>,
}

impl<'config> LinkExtractor<'config> {
    /// Creates a new link extractor bound to the provided configuration.
    #[inline]
    pub(super) const fn new(config: &'config Config) -> Self {
        Self {
            config,
            current: None,
        }
    }

    fn start_link(
        &mut self,
        link_type: PLinkType,
        dest_url: &CowStr<'_>,
        position: SourceByteOffset,
        is_embed: bool,
    ) {
        let embed = if is_embed {
            EmbedState::Embed
        } else {
            EmbedState::Link
        };
        match link_type {
            PLinkType::WikiLink {
                has_pothole,
            } => {
                let alias_mode = if has_pothole {
                    AliasMode::Collect
                } else {
                    AliasMode::Ignore
                };
                self.current = Some(LinkBuilder::new(
                    dest_url.as_ref(),
                    position,
                    Style::WikiLink,
                    embed,
                    alias_mode,
                ));
            }
            PLinkType::Autolink | PLinkType::Email => {
                self.current = Some(LinkBuilder::new(
                    dest_url.as_ref(),
                    position,
                    Style::MdLink,
                    embed,
                    AliasMode::Ignore,
                ));
            }
            PLinkType::Inline
            | PLinkType::Reference
            | PLinkType::ReferenceUnknown
            | PLinkType::Collapsed
            | PLinkType::CollapsedUnknown
            | PLinkType::Shortcut
            | PLinkType::ShortcutUnknown => {
                self.current = Some(LinkBuilder::new(
                    dest_url.as_ref(),
                    position,
                    Style::MdLink,
                    embed,
                    AliasMode::Collect,
                ));
            }
        }
    }
}

impl Extractor for LinkExtractor<'_> {
    type Error = NoteError;
    type Output = Link;

    fn finish(self) -> Result<Vec<Link>, NoteError> {
        // No incomplete links to flush
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
    ) -> Result<ExtractionState<Link>, NoteError> {
        match event {
            Event::Start(CmarkTag::Link {
                link_type,
                dest_url,
                ..
            }) => {
                let position = SourceByteOffset::try_from_usize(range.start)?;
                self.start_link(*link_type, dest_url, position, false);
                Ok(ExtractionState::Continue)
            }

            Event::Start(CmarkTag::Image {
                link_type,
                dest_url,
                ..
            }) => {
                let position = SourceByteOffset::try_from_usize(range.start)?;
                self.start_link(*link_type, dest_url, position, true);
                Ok(ExtractionState::Continue)
            }

            Event::Text(_) => {
                // Collect alias text if needed
                if let Some(builder) = self.current.as_mut() {
                    builder.add_alias_text(text);
                }
                Ok(ExtractionState::Continue)
            }

            Event::SoftBreak | Event::HardBreak => {
                if let Some(builder) = self.current.as_mut() {
                    builder.add_alias_text(" ");
                }
                Ok(ExtractionState::Continue)
            }

            Event::End(TagEnd::Link | TagEnd::Image) => {
                if let Some(builder) = self.current.take() {
                    let link = builder.build()?;
                    return Ok(ExtractionState::Emit(link));
                }
                Ok(ExtractionState::Continue)
            }

            // Ignore other events
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
    use pulldown_cmark::{CowStr, Event, Tag as CmarkTag, TagEnd};

    use super::*;
    use crate::{
        config::{
            aggregate::Config,
            raw::RawConfig,
            vault::{VaultId, VaultRoot},
        },
        note::link::EmbedType,
    };

    #[test]
    fn extracts_simple_wikilink() {
        let config = test_config();
        let mut extractor = LinkExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Wiki-link: [[Main Page]]
        // Start link
        extractor
            .process(
                &Event::Start(CmarkTag::Link {
                    link_type: pulldown_cmark::LinkType::WikiLink {
                        has_pothole: false,
                    },
                    dest_url: CowStr::Borrowed("Main Page"),
                    title: CowStr::Borrowed(""),
                    id: CowStr::Borrowed(""),
                }),
                "",
                &(0..2),
                &ctx,
            )
            .unwrap();

        // Text event (can be ignored for simple wikilink without alias)
        extractor
            .process(
                &Event::Text("Main Page".into()),
                "Main Page",
                &(2..11),
                &ctx,
            )
            .unwrap();

        // End link - should emit
        let result = extractor
            .process(&Event::End(TagEnd::Link), "", &(11..13), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(link) = result else {
            panic!("Expected link emission");
        };

        assert!(!link.is_embed());
        assert_eq!(link.target().vault_path(), Some("Main Page"));
        assert!(link.anchor().is_none());
    }

    #[test]
    fn extracts_wikilink_with_alias() {
        let config = test_config();
        let mut extractor = LinkExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Wiki-link: [[Main Page|Home]]
        // Start link (has_pothole = true)
        extractor
            .process(
                &Event::Start(CmarkTag::Link {
                    link_type: pulldown_cmark::LinkType::WikiLink {
                        has_pothole: true,
                    },
                    dest_url: CowStr::Borrowed("Main Page"),
                    title: CowStr::Borrowed(""),
                    id: CowStr::Borrowed(""),
                }),
                "",
                &(0..2),
                &ctx,
            )
            .unwrap();

        // Alias text
        extractor
            .process(&Event::Text("Home".into()), "Home", &(14..18), &ctx)
            .unwrap();

        // End link
        let result = extractor
            .process(&Event::End(TagEnd::Link), "", &(18..20), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(link) = result else {
            panic!("Expected link emission");
        };

        assert_eq!(link.target().vault_path(), Some("Main Page"));
        assert_eq!(link.alias(), Some("Home"));
    }

    #[test]
    fn extracts_wikilink_with_heading_anchor() {
        let config = test_config();
        let mut extractor = LinkExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Wiki-link: [[Note#Heading]]
        extractor
            .process(
                &Event::Start(CmarkTag::Link {
                    link_type: pulldown_cmark::LinkType::WikiLink {
                        has_pothole: false,
                    },
                    dest_url: CowStr::Borrowed("Note#Heading"),
                    title: CowStr::Borrowed(""),
                    id: CowStr::Borrowed(""),
                }),
                "",
                &(0..2),
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::Text("Note#Heading".into()),
                "Note#Heading",
                &(2..14),
                &ctx,
            )
            .unwrap();

        let result = extractor
            .process(&Event::End(TagEnd::Link), "", &(14..16), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(link) = result else {
            panic!("Expected link emission");
        };

        assert_eq!(link.target().vault_path(), Some("Note"));
        assert!(link.anchor().is_some());
        assert!(link.anchor().unwrap().is_heading());
        assert_eq!(link.anchor().unwrap().text(), "Heading");
    }

    #[test]
    fn extracts_wikilink_with_block_anchor() {
        let config = test_config();
        let mut extractor = LinkExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Wiki-link: [[Note#^abc123]]
        extractor
            .process(
                &Event::Start(CmarkTag::Link {
                    link_type: pulldown_cmark::LinkType::WikiLink {
                        has_pothole: false,
                    },
                    dest_url: CowStr::Borrowed("Note#^abc123"),
                    title: CowStr::Borrowed(""),
                    id: CowStr::Borrowed(""),
                }),
                "",
                &(0..2),
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::Text("Note#^abc123".into()),
                "Note#^abc123",
                &(2..14),
                &ctx,
            )
            .unwrap();

        let result = extractor
            .process(&Event::End(TagEnd::Link), "", &(14..16), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(link) = result else {
            panic!("Expected link emission");
        };

        assert_eq!(link.target().vault_path(), Some("Note"));
        assert!(link.anchor().is_some());
        assert!(link.anchor().unwrap().is_block_ref());
        assert_eq!(link.anchor().unwrap().text(), "abc123");
    }

    #[test]
    fn extracts_markdown_link() {
        let config = test_config();
        let mut extractor = LinkExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Markdown link: [text](https://example.com)
        extractor
            .process(
                &Event::Start(CmarkTag::Link {
                    link_type: pulldown_cmark::LinkType::Inline,
                    dest_url: CowStr::Borrowed("https://example.com"),
                    title: CowStr::Borrowed(""),
                    id: CowStr::Borrowed(""),
                }),
                "",
                &(0..1),
                &ctx,
            )
            .unwrap();

        // Link text (becomes alias)
        extractor
            .process(&Event::Text("Example".into()), "Example", &(1..8), &ctx)
            .unwrap();

        let result = extractor
            .process(&Event::End(TagEnd::Link), "", &(29..30), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(link) = result else {
            panic!("Expected link emission");
        };

        assert!(link.target().is_external());
        assert_eq!(link.alias(), Some("Example"));
    }

    #[test]
    fn extracts_embed_with_image_extension() {
        let config = test_config();
        let mut extractor = LinkExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Embed: ![[image.png]]
        extractor
            .process(
                &Event::Start(CmarkTag::Image {
                    link_type: pulldown_cmark::LinkType::WikiLink {
                        has_pothole: false,
                    },
                    dest_url: CowStr::Borrowed("image.png"),
                    title: CowStr::Borrowed(""),
                    id: CowStr::Borrowed(""),
                }),
                "",
                &(0..3),
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::Text("image.png".into()),
                "image.png",
                &(3..12),
                &ctx,
            )
            .unwrap();

        let result = extractor
            .process(&Event::End(TagEnd::Image), "", &(12..14), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(link) = result else {
            panic!("Expected link emission");
        };

        assert!(link.is_embed());
        assert_eq!(link.embed_type(), Some(EmbedType::Image));
        assert_eq!(link.target().vault_path(), Some("image.png"));
    }

    #[test]
    fn extracts_markdown_image() {
        let config = test_config();
        let mut extractor = LinkExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Markdown image: ![alt](photo.jpg)
        extractor
            .process(
                &Event::Start(CmarkTag::Image {
                    link_type: pulldown_cmark::LinkType::Inline,
                    dest_url: CowStr::Borrowed("photo.jpg"),
                    title: CowStr::Borrowed(""),
                    id: CowStr::Borrowed(""),
                }),
                "",
                &(0..2),
                &ctx,
            )
            .unwrap();

        extractor
            .process(&Event::Text("alt".into()), "alt", &(2..5), &ctx)
            .unwrap();

        let result = extractor
            .process(&Event::End(TagEnd::Image), "", &(5..6), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(link) = result else {
            panic!("Expected link emission");
        };

        assert!(link.is_embed());
        assert_eq!(link.embed_type(), Some(EmbedType::Image));
        assert_eq!(link.target().vault_path(), Some("photo.jpg"));
    }

    #[test]
    fn extracts_embed_with_video_extension() {
        let config = test_config();
        let mut extractor = LinkExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Embed: ![[clip.mp4]]
        extractor
            .process(
                &Event::Start(CmarkTag::Image {
                    link_type: pulldown_cmark::LinkType::WikiLink {
                        has_pothole: false,
                    },
                    dest_url: CowStr::Borrowed("clip.mp4"),
                    title: CowStr::Borrowed(""),
                    id: CowStr::Borrowed(""),
                }),
                "",
                &(0..3),
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::Text("clip.mp4".into()),
                "clip.mp4",
                &(3..11),
                &ctx,
            )
            .unwrap();

        let result = extractor
            .process(&Event::End(TagEnd::Image), "", &(11..13), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(link) = result else {
            panic!("Expected link emission");
        };

        assert!(link.is_embed());
        assert_eq!(link.embed_type(), Some(EmbedType::Video));
        assert_eq!(link.target().vault_path(), Some("clip.mp4"));
    }

    #[test]
    fn extracts_embed_with_audio_extension() {
        let config = test_config();
        let mut extractor = LinkExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Embed: ![[song.mp3]]
        extractor
            .process(
                &Event::Start(CmarkTag::Image {
                    link_type: pulldown_cmark::LinkType::WikiLink {
                        has_pothole: false,
                    },
                    dest_url: CowStr::Borrowed("song.mp3"),
                    title: CowStr::Borrowed(""),
                    id: CowStr::Borrowed(""),
                }),
                "",
                &(0..3),
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::Text("song.mp3".into()),
                "song.mp3",
                &(3..11),
                &ctx,
            )
            .unwrap();

        let result = extractor
            .process(&Event::End(TagEnd::Image), "", &(11..13), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(link) = result else {
            panic!("Expected link emission");
        };

        assert!(link.is_embed());
        assert_eq!(link.embed_type(), Some(EmbedType::Audio));
        assert_eq!(link.target().vault_path(), Some("song.mp3"));
    }

    #[test]
    fn detects_external_url() {
        let config = test_config();
        let mut extractor = LinkExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Markdown link: [site](https://example.com)
        extractor
            .process(
                &Event::Start(CmarkTag::Link {
                    link_type: pulldown_cmark::LinkType::Inline,
                    dest_url: CowStr::Borrowed("https://example.com"),
                    title: CowStr::Borrowed(""),
                    id: CowStr::Borrowed(""),
                }),
                "",
                &(0..1),
                &ctx,
            )
            .unwrap();

        extractor
            .process(&Event::Text("site".into()), "site", &(1..5), &ctx)
            .unwrap();

        let result = extractor
            .process(&Event::End(TagEnd::Link), "", &(5..6), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(link) = result else {
            panic!("Expected link emission");
        };

        assert!(link.target().is_external());
    }

    #[test]
    fn handles_unclosed_link() {
        let config = test_config();
        let mut extractor = LinkExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Start link, no end
        extractor
            .process(
                &Event::Start(CmarkTag::Link {
                    link_type: pulldown_cmark::LinkType::Inline,
                    dest_url: CowStr::Borrowed("note"),
                    title: CowStr::Borrowed(""),
                    id: CowStr::Borrowed(""),
                }),
                "",
                &(0..1),
                &ctx,
            )
            .unwrap();

        let pending = extractor.finish().unwrap();
        assert!(pending.is_empty());
    }

    #[test]
    fn handles_empty_target() {
        let config = test_config();
        let mut extractor = LinkExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Empty target link
        extractor
            .process(
                &Event::Start(CmarkTag::Link {
                    link_type: pulldown_cmark::LinkType::Inline,
                    dest_url: CowStr::Borrowed(""),
                    title: CowStr::Borrowed(""),
                    id: CowStr::Borrowed(""),
                }),
                "",
                &(0..1),
                &ctx,
            )
            .unwrap();

        let result =
            extractor.process(&Event::End(TagEnd::Link), "", &(1..2), &ctx);

        let _err: NoteError = result.unwrap_err();
    }

    #[test]
    fn collects_alias_across_multiple_text_events() {
        let config = test_config();
        let mut extractor = LinkExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Markdown link with alias split across events: [Hello World](note)
        extractor
            .process(
                &Event::Start(CmarkTag::Link {
                    link_type: pulldown_cmark::LinkType::Inline,
                    dest_url: CowStr::Borrowed("note"),
                    title: CowStr::Borrowed(""),
                    id: CowStr::Borrowed(""),
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
            .process(&Event::End(TagEnd::Link), "", &(12..13), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(link) = result else {
            panic!("Expected link emission");
        };

        assert_eq!(link.alias(), Some("Hello World"));
    }

    #[test]
    fn detects_mailto_external_url() {
        let config = test_config();
        let mut extractor = LinkExtractor::new(&config);
        let ctx = ExtractionContext::default();

        // Markdown link: [Email](mailto:dev@example.com)
        extractor
            .process(
                &Event::Start(CmarkTag::Link {
                    link_type: pulldown_cmark::LinkType::Inline,
                    dest_url: CowStr::Borrowed("mailto:dev@example.com"),
                    title: CowStr::Borrowed(""),
                    id: CowStr::Borrowed(""),
                }),
                "",
                &(0..1),
                &ctx,
            )
            .unwrap();

        extractor
            .process(&Event::Text("Email".into()), "Email", &(1..6), &ctx)
            .unwrap();

        let result = extractor
            .process(&Event::End(TagEnd::Link), "", &(6..7), &ctx)
            .unwrap();

        #[expect(clippy::panic, reason = "Test assertion")]
        let ExtractionState::Emit(link) = result else {
            panic!("Expected link emission");
        };

        assert!(link.target().is_external());
    }

    fn test_config() -> Config {
        let raw = RawConfig::default();
        Config::build(
            &raw,
            VaultId::new(),
            VaultRoot::try_new(std::path::PathBuf::from("/vault"))
                .expect("vault root"),
            crate::config::aggregate::Version::initial(),
        )
        .expect("failed to build test config")
    }
}
