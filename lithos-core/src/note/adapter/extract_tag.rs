//! Tag extraction from markdown event streams.
//!
//! Extracts Obsidian-style tags and deduplicates them.

use std::{collections::HashSet, ops::Range};

use pulldown_cmark::{CowStr, Event};

use super::{
    reader::{ExtractionContext, ExtractionState, Extractor},
    tag_scanner::TagScanner,
};
use crate::{
    config::aggregate::Config,
    note::{frontmatter::Frontmatter, tag::Tag as NoteTag},
};

/// Extractor for Obsidian-style tags.
pub struct TagExtractor<'config> {
    config: &'config Config,
    tags: Vec<NoteTag>,
    tag_set: HashSet<Box<str>>,
    frontmatter: Option<Frontmatter>,
}

impl<'config> TagExtractor<'config> {
    #[inline]
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Used in tests; will be used by reader orchestration"
        )
    )]
    pub(super) fn new(config: &'config Config) -> Self {
        Self {
            config,
            tags: Vec::new(),
            tag_set: HashSet::new(),
            frontmatter: None,
        }
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Used by reader orchestration to supply frontmatter"
        )
    )]
    pub(crate) fn set_frontmatter(&mut self, frontmatter: Frontmatter) {
        self.frontmatter = Some(frontmatter);
    }

    fn add_tag(&mut self, tag: NoteTag) {
        if !self.tag_set.contains(tag.full_path()) {
            self.tag_set.insert(tag.full_path().into());
            self.tags.push(tag);
        }
    }

    fn collect_from_text(&mut self, text: &str) {
        for tag in TagScanner::new(text).collect_tags() {
            self.add_tag(tag);
        }
    }

    fn collect_from_tokens(&mut self, text: &str) {
        for token in text.split(|ch: char| ch.is_whitespace() || ch == ',') {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }

            if let Ok(tag) = NoteTag::from_token(token) {
                self.add_tag(tag);
            }
        }
    }

    fn collect_from_frontmatter(&mut self, frontmatter: &Frontmatter) {
        let key = self.config.frontmatter().tags();
        let Some(value) = frontmatter.get(key) else {
            return;
        };

        if let Some(text) = value.as_str() {
            self.collect_from_tokens(text);
            return;
        }

        if let Some(values) = value.as_array() {
            for item in values {
                if let Some(text) = item.as_str() {
                    self.collect_from_tokens(text);
                }
            }
        }
    }
}

impl Extractor for TagExtractor<'_> {
    type Error = crate::note::error::NoteError;
    type Output = NoteTag;

    fn finish(mut self) -> Result<Vec<NoteTag>, crate::note::error::NoteError> {
        if let Some(frontmatter) = self.frontmatter.take() {
            self.collect_from_frontmatter(&frontmatter);
        }
        Ok(self.tags)
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &Event preferred for clarity"
    )]
    fn process(
        &mut self,
        event: &Event<'_>,
        text: CowStr<'_>,
        _range: Range<usize>,
        ctx: &ExtractionContext,
    ) -> Result<ExtractionState<NoteTag>, crate::note::error::NoteError> {
        let _tags_key = self.config.frontmatter().tags();
        match event {
            Event::Text(_) => {
                if !ctx.inside_code_block && !ctx.inside_link {
                    self.collect_from_text(&text);
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
            | Event::SoftBreak
            | Event::HardBreak
            | Event::Rule
            | Event::TaskListMarker(_) => Ok(ExtractionState::Continue),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pulldown_cmark::{CowStr, Event};

    use super::*;
    use crate::{
        config::{
            aggregate::Config,
            raw::RawConfig,
            vault::{VaultId, VaultRoot},
        },
        note::{frontmatter::Frontmatter, value::FieldValue},
    };

    #[test]
    fn extracts_simple_tag() {
        let config = test_config();
        let mut extractor = TagExtractor::new(&config);
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Text(CowStr::Borrowed("#tag")),
                CowStr::Borrowed("#tag"),
                0..4,
                &ctx,
            )
            .unwrap();

        let tags = extractor.finish().unwrap();
        let tag = tags.first().expect("tag should exist");
        assert_eq!(tag.full_path(), "tag");
    }

    #[test]
    fn extracts_nested_tag() {
        let config = test_config();
        let mut extractor = TagExtractor::new(&config);
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Text(CowStr::Borrowed("#parent/child")),
                CowStr::Borrowed("#parent/child"),
                0..13,
                &ctx,
            )
            .unwrap();

        let tags = extractor.finish().unwrap();
        let tag = tags.first().expect("tag should exist");
        assert_eq!(tag.full_path(), "parent/child");
    }

    #[test]
    fn extracts_tag_with_numbers() {
        let config = test_config();
        let mut extractor = TagExtractor::new(&config);
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Text(CowStr::Borrowed("#tag123")),
                CowStr::Borrowed("#tag123"),
                0..7,
                &ctx,
            )
            .unwrap();

        let tags = extractor.finish().unwrap();
        let tag = tags.first().expect("tag should exist");
        assert_eq!(tag.full_path(), "tag123");
    }

    #[test]
    fn extracts_tag_with_hyphens() {
        let config = test_config();
        let mut extractor = TagExtractor::new(&config);
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Text(CowStr::Borrowed("#my-tag")),
                CowStr::Borrowed("#my-tag"),
                0..7,
                &ctx,
            )
            .unwrap();

        let tags = extractor.finish().unwrap();
        let tag = tags.first().expect("tag should exist");
        assert_eq!(tag.full_path(), "my-tag");
    }

    #[test]
    fn extracts_tag_with_underscores() {
        let config = test_config();
        let mut extractor = TagExtractor::new(&config);
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Text(CowStr::Borrowed("#my_tag")),
                CowStr::Borrowed("#my_tag"),
                0..7,
                &ctx,
            )
            .unwrap();

        let tags = extractor.finish().unwrap();
        let tag = tags.first().expect("tag should exist");
        assert_eq!(tag.full_path(), "my_tag");
    }

    #[test]
    fn ignores_tag_in_code() {
        let config = test_config();
        let mut extractor = TagExtractor::new(&config);
        let ctx = ExtractionContext {
            inside_code_block: true,
            ..Default::default()
        };

        extractor
            .process(
                &Event::Text(CowStr::Borrowed("#tag")),
                CowStr::Borrowed("#tag"),
                0..4,
                &ctx,
            )
            .unwrap();

        let tags = extractor.finish().unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn ignores_tag_in_link() {
        let config = test_config();
        let mut extractor = TagExtractor::new(&config);
        let ctx = ExtractionContext {
            inside_link: true,
            ..Default::default()
        };

        extractor
            .process(
                &Event::Text(CowStr::Borrowed("#tag")),
                CowStr::Borrowed("#tag"),
                0..4,
                &ctx,
            )
            .unwrap();

        let tags = extractor.finish().unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn deduplicates_tags() {
        let config = test_config();
        let mut extractor = TagExtractor::new(&config);
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Text(CowStr::Borrowed("#tag #tag")),
                CowStr::Borrowed("#tag #tag"),
                0..9,
                &ctx,
            )
            .unwrap();

        let tags = extractor.finish().unwrap();
        assert_eq!(tags.len(), 1);
    }

    #[test]
    fn extracts_tags_from_frontmatter() {
        let config = test_config();
        let mut extractor = TagExtractor::new(&config);

        let mut fields = HashMap::new();
        fields.insert(
            config.frontmatter().tags().as_str().into(),
            FieldValue::String("alpha beta, #gamma".into()),
        );
        let frontmatter = Frontmatter::new(fields);

        extractor.set_frontmatter(frontmatter);

        let tags = extractor.finish().unwrap();
        assert_eq!(tags.len(), 3);
    }

    #[test]
    fn handles_invalid_tag_characters() {
        let config = test_config();
        let mut extractor = TagExtractor::new(&config);
        let ctx = ExtractionContext::default();

        extractor
            .process(
                &Event::Text(CowStr::Borrowed("#/bad")),
                CowStr::Borrowed("#/bad"),
                0..5,
                &ctx,
            )
            .unwrap();

        let tags = extractor.finish().unwrap();
        assert!(tags.is_empty());
    }

    fn test_config() -> Config {
        let raw = RawConfig::default();
        Config::build(
            &raw,
            VaultId::new(),
            VaultRoot::try_new(std::path::PathBuf::from("/vault"))
                .expect("vault root"),
        )
        .expect("failed to build test config")
    }
}
