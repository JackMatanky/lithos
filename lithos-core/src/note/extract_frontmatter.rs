//! Frontmatter extraction from markdown event streams.
//!
//! Extracts YAML/TOML metadata blocks, preserves line breaks in block scalar
//! content, and converts parsed values into domain `FieldValue` entries.

use std::{collections::HashMap, ops::Range};

use pulldown_cmark::{
    CowStr, Event, MetadataBlockKind, Tag as CmarkTag, TagEnd,
};

use super::reader::{ExtractionContext, ExtractionState, Extractor};
use crate::note::{
    error::{FrontmatterParseError, NoteError},
    frontmatter::Frontmatter,
    value::FieldValue,
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

        let fields = match kind {
            MetadataBlockKind::YamlStyle => {
                let yaml_value: serde_yaml::Value =
                    serde_yaml::from_str(&self.text).map_err(|_e| {
                        NoteError::Frontmatter(
                            FrontmatterParseError::InvalidYaml {
                                reason: "failed to parse yaml",
                            },
                        )
                    })?;
                Self::yaml_to_field_map(&yaml_value)?
            }
            MetadataBlockKind::PlusesStyle => {
                let toml_value: toml::Value = toml::from_str(&self.text)
                    .map_err(|_e| {
                        NoteError::Frontmatter(
                            FrontmatterParseError::InvalidToml {
                                reason: "failed to parse toml",
                            },
                        )
                    })?;
                Self::toml_to_field_map(&toml_value)?
            }
        };

        self.text.clear();
        Ok(Some(Frontmatter::new(fields)))
    }

    fn yaml_to_field_map(
        value: &serde_yaml::Value,
    ) -> Result<HashMap<Box<str>, FieldValue>, NoteError> {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "matching on &Value keeps conversion concise"
        )]
        let serde_yaml::Value::Mapping(map) = value else {
            return Err(NoteError::Frontmatter(
                FrontmatterParseError::NotYamlMapping,
            ));
        };

        let mut fields = HashMap::with_capacity(map.len());
        for (key, value_item) in map {
            let key_str = key.as_str().ok_or(NoteError::Frontmatter(
                FrontmatterParseError::NonStringKey,
            ))?;

            let field_value =
                FieldValue::try_from_yaml(value_item).map_err(|_error| {
                    NoteError::Frontmatter(
                        FrontmatterParseError::InvalidYamlValue {
                            reason: "invalid yaml value",
                        },
                    )
                })?;

            fields.insert(key_str.into(), field_value);
        }

        Ok(fields)
    }

    fn toml_to_field_map(
        value: &toml::Value,
    ) -> Result<HashMap<Box<str>, FieldValue>, NoteError> {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "matching on &Value keeps conversion concise"
        )]
        let toml::Value::Table(table) = value else {
            return Err(NoteError::Frontmatter(
                FrontmatterParseError::NotTomlTable,
            ));
        };

        let mut fields = HashMap::with_capacity(table.len());
        for (key, value_item) in table {
            let field_value = Self::field_value_from_toml(value_item)?;
            fields.insert(key.as_str().into(), field_value);
        }

        Ok(fields)
    }

    fn field_value_from_toml(
        value: &toml::Value,
    ) -> Result<FieldValue, NoteError> {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "matching on &Value keeps conversion concise"
        )]
        match value {
            toml::Value::String(text) => {
                Ok(FieldValue::String(text.clone().into()))
            }
            toml::Value::Integer(number) => {
                const MAX_SAFE_INTEGER: u64 = 0x0020_0000_0000_0000;
                let magnitude = number.unsigned_abs();
                if magnitude > MAX_SAFE_INTEGER {
                    return Err(NoteError::Frontmatter(
                        FrontmatterParseError::InvalidTomlValue {
                            reason: "integer value exceeds safe f64 range",
                        },
                    ));
                }

                #[expect(
                    clippy::as_conversions,
                    reason = "checked MAX_SAFE_INTEGER ensures exact f64"
                )]
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "checked MAX_SAFE_INTEGER ensures exact f64"
                )]
                let parsed = (*number) as f64;
                Ok(FieldValue::Number(parsed))
            }
            toml::Value::Float(value) => Ok(FieldValue::Number(*value)),
            toml::Value::Boolean(value) => Ok(FieldValue::Boolean(*value)),
            toml::Value::Datetime(datetime) => {
                Ok(FieldValue::String(datetime.to_string().into()))
            }
            toml::Value::Array(values) => {
                let mut items = Vec::with_capacity(values.len());
                for item in values {
                    items.push(Self::field_value_from_toml(item)?);
                }
                Ok(FieldValue::Array(items))
            }
            toml::Value::Table(table) => {
                let mut obj = HashMap::with_capacity(table.len());
                for (key, value_item) in table {
                    obj.insert(
                        key.as_str().into(),
                        Self::field_value_from_toml(value_item)?,
                    );
                }
                Ok(FieldValue::Object(obj))
            }
        }
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
        text: CowStr<'_>,
        _range: Range<usize>,
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
                    self.push_text(&text);
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
    use pulldown_cmark::{
        CowStr, Event, MetadataBlockKind, Tag as CmarkTag, TagEnd,
    };

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
                CowStr::Borrowed(""),
                0..3,
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::Text(CowStr::Borrowed("title: Test\ncount: 2")),
                CowStr::Borrowed("title: Test\ncount: 2"),
                3..24,
                &ctx,
            )
            .unwrap();

        let result = extractor
            .process(
                &Event::End(TagEnd::MetadataBlock(
                    MetadataBlockKind::YamlStyle,
                )),
                CowStr::Borrowed(""),
                24..27,
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
                CowStr::Borrowed(""),
                0..3,
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::Text(CowStr::Borrowed("title = \"Test\"\ncount = 2")),
                CowStr::Borrowed("title = \"Test\"\ncount = 2"),
                3..30,
                &ctx,
            )
            .unwrap();

        let result = extractor
            .process(
                &Event::End(TagEnd::MetadataBlock(
                    MetadataBlockKind::PlusesStyle,
                )),
                CowStr::Borrowed(""),
                30..33,
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
                CowStr::Borrowed(""),
                0..3,
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::Text(CowStr::Borrowed("- item")),
                CowStr::Borrowed("- item"),
                3..9,
                &ctx,
            )
            .unwrap();

        let result = extractor.process(
            &Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle)),
            CowStr::Borrowed(""),
            9..12,
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
                CowStr::Borrowed(""),
                0..3,
                &ctx,
            )
            .unwrap();

        extractor
            .process(
                &Event::Text(CowStr::Borrowed("[[]]")),
                CowStr::Borrowed("[[]]"),
                3..7,
                &ctx,
            )
            .unwrap();

        let result = extractor.process(
            &Event::End(TagEnd::MetadataBlock(MetadataBlockKind::PlusesStyle)),
            CowStr::Borrowed(""),
            7..10,
            &ctx,
        );

        let _err: NoteError = result.unwrap_err();
    }
}
