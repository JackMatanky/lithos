use std::{borrow::Cow, collections::HashMap};

use regex::Regex;

use crate::note::{
    error::NoteParseError, position::SourceByteRange, value::FieldValue,
};

/// Input format for frontmatter parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawFrontmatterFormat {
    /// YAML frontmatter block.
    Yaml,
    /// TOML frontmatter block.
    Toml,
}

/// Raw frontmatter block captured from metadata events.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawFrontmatter<'source> {
    pub kind: RawFrontmatterFormat,
    pub text: Cow<'source, str>,
    pub range: SourceByteRange,
}

impl<'source> RawFrontmatter<'source> {
    /// Create a raw frontmatter block.
    #[inline]
    #[must_use]
    pub const fn new(
        kind: RawFrontmatterFormat,
        text: Cow<'source, str>,
        range: SourceByteRange,
    ) -> Self {
        Self {
            kind,
            text,
            range,
        }
    }

    /// Parses the raw frontmatter block into a field map.
    ///
    /// # Errors
    ///
    /// Returns [`NoteParseError`] if the content cannot be parsed.
    pub fn parse_fields(
        &self,
    ) -> Result<HashMap<Box<str>, FieldValue>, NoteParseError> {
        match self.kind {
            RawFrontmatterFormat::Yaml => {
                let sanitized =
                    sanitize_yaml_obsidian_links(self.text.as_ref());
                serde_yaml::from_str(&sanitized).map_err(|e| {
                    let location = e.location();
                    NoteParseError::Frontmatter {
                        format: "YAML",
                        line: location.as_ref().map(serde_yaml::Location::line),
                        column: location
                            .as_ref()
                            .map(serde_yaml::Location::column),
                        reason: e.to_string().into(),
                    }
                })
            }
            RawFrontmatterFormat::Toml => toml::from_str(self.text.as_ref())
                .map_err(|e| NoteParseError::Frontmatter {
                    format: "TOML",
                    line: None,
                    column: None,
                    reason: e.to_string().into(),
                }),
        }
    }
}

impl RawFrontmatter<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawFrontmatter<'static> {
        RawFrontmatter {
            kind: self.kind,
            text: Cow::Owned(self.text.into_owned()),
            range: self.range,
        }
    }
}

fn sanitize_yaml_obsidian_links(text: &str) -> Cow<'_, str> {
    let step1 = YAML_MAP_LINK_RE.replace_all(text, r#"$1"$2""#);
    match step1 {
        Cow::Borrowed(_) => YAML_LIST_LINK_RE.replace_all(text, r#"$1"$2""#),
        Cow::Owned(s1) => {
            let step2 = YAML_LIST_LINK_RE.replace_all(&s1, r#"$1"$2""#);
            match step2 {
                Cow::Borrowed(_) => Cow::Owned(s1),
                Cow::Owned(s2) => Cow::Owned(s2),
            }
        }
    }
}

/// Regex for identifying unquoted Obsidian wikilinks in YAML mapping entries.
///
/// Pattern breakdown:
/// 1. `^(\s*[\w_-]+\s*:\s*)`: Matches the key and colon, including indentation.
/// 2. `([^"'\s|>].*?\[\[.*\]\].*|\[\[.*\]\].*)`: Matches values starting with a
///    non-quote/special char that contain a wikilink, or start with one.
#[expect(clippy::expect_used, reason = "Static regex compilation")]
static YAML_MAP_LINK_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(
    || {
        Regex::new(r#"(?m)^(\s*[\w_-]+\s*:\s*)([^"'\s|>].*?\[\[.*\]\].*|\[\[.*\]\].*)$"#)
            .expect("valid regex")
    },
);

/// Regex for identifying unquoted Obsidian wikilinks in YAML list items.
///
/// Pattern breakdown:
/// 1. `^(\s*-\s*)`: Matches the list dash and indentation.
/// 2. `([^"'\s].*?\[\[.*\]\].*|\[\[.*\]\].*)`: Matches values starting with a
///    non-quote/space that contain a wikilink, or start with one.
#[expect(clippy::expect_used, reason = "Static regex compilation")]
static YAML_LIST_LINK_RE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| {
        Regex::new(r#"(?m)^(\s*-\s*)([^"'\s].*?\[\[.*\]\].*|\[\[.*\]\].*)$"#)
            .expect("valid regex")
    });

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rstest::*;

    use super::*;
    use crate::note::position::SourceByteOffset;

    #[rstest]
    #[case::yaml_simple(
        RawFrontmatterFormat::Yaml,
        "key: value\nnum: 42",
        "key",
        FieldValue::String("value".into())
    )]
    #[case::toml_simple(
        RawFrontmatterFormat::Toml,
        "key = \"value\"\nnum = 42",
        "key",
        FieldValue::String("value".into())
    )]
    #[case::yaml_nested(
        RawFrontmatterFormat::Yaml,
        "outer:\n  inner: true",
        "outer",
        FieldValue::Object(Box::new(HashMap::from([(
            "inner".into(),
            FieldValue::Boolean(true),
        )])))
    )]
    fn should_parse_valid_formats(
        #[case] format: RawFrontmatterFormat,
        #[case] text: &str,
        #[case] key: &str,
        #[case] expected: FieldValue,
    ) {
        let raw = RawFrontmatter::new(
            format,
            text.into(),
            SourceByteRange::new(
                SourceByteOffset::new(0),
                SourceByteOffset::new(0),
            )
            .expect("valid range"),
        );
        let fields = raw.parse_fields();
        assert!(
            fields.is_ok(),
            "Failed to parse {:?}: {:?}",
            format,
            fields.err()
        );
        let fields = fields.unwrap();
        assert_eq!(
            fields.get(key),
            Some(&expected),
            "Field mismatch for key: {key}"
        );
    }

    #[rstest]
    fn should_report_yaml_syntax_error_with_location() {
        let raw = RawFrontmatter::new(
            RawFrontmatterFormat::Yaml,
            "key: : invalid".into(),
            SourceByteRange::new(
                SourceByteOffset::new(0),
                SourceByteOffset::new(0),
            )
            .expect("valid range"),
        );
        let result = raw.parse_fields();

        assert!(
            matches!(
                result,
                Err(NoteParseError::Frontmatter {
                    format: "YAML",
                    ..
                })
            ),
            "Expected YAML parse error, got: {result:?}"
        );

        if let Err(NoteParseError::Frontmatter {
            line,
            ..
        }) = result
        {
            assert!(line.is_some(), "Expected line number in YAML error");
        }
    }

    #[rstest]
    fn should_report_toml_syntax_error() {
        let raw = RawFrontmatter::new(
            RawFrontmatterFormat::Toml,
            "key = invalid_no_quotes".into(),
            SourceByteRange::new(
                SourceByteOffset::new(0),
                SourceByteOffset::new(0),
            )
            .expect("valid range"),
        );
        let result = raw.parse_fields();

        assert!(
            matches!(
                result,
                Err(NoteParseError::Frontmatter {
                    format: "TOML",
                    ..
                })
            ),
            "Expected TOML parse error, got: {result:?}"
        );
    }

    #[rstest]
    #[case::yaml_map_link("link: [[My Page]]")]
    #[case::yaml_map_link_with_display("link: [[My Page|Display]]")]
    fn should_parse_yaml_with_unquoted_links(#[case] input: &str) {
        let raw = RawFrontmatter::new(
            RawFrontmatterFormat::Yaml,
            input.into(),
            SourceByteRange::new(
                SourceByteOffset::new(0),
                SourceByteOffset::new(0),
            )
            .expect("valid range"),
        );
        let fields = raw.parse_fields().expect("frontmatter parsed");
        if let Some(value) = fields.get("link") {
            let parsed = value.as_str().expect("string value");
            assert!(
                parsed.starts_with("[[") && parsed.ends_with("]]"),
                "Expected wikilink parsing, got: {parsed}"
            );
        }
    }

    #[rstest]
    #[case::yaml_map_link("link: [[My Page]]", "link: \"[[My Page]]\"")]
    #[case::yaml_list_link("- [[Another Page]]", "- \"[[Another Page]]\"")]
    #[case::yaml_map_link_with_display(
        "link: [[My Page|Display]]",
        "link: \"[[My Page|Display]]\""
    )]
    #[case::yaml_mixed(
        "title: Hello\nlink: [[Page]]\n- [[Item]]",
        "title: Hello\nlink: \"[[Page]]\"\n- \"[[Item]]\""
    )]
    #[case::yaml_already_quoted(
        "link: \"[[Already Quoted]]\"",
        "link: \"[[Already Quoted]]\""
    )]
    fn should_sanitize_obsidian_links_in_yaml(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        let result = super::sanitize_yaml_obsidian_links(input);
        assert_eq!(
            result.as_ref(),
            expected,
            "Sanitization failed for input: {input}"
        );
    }

    proptest! {
        #[test]
        fn sanitize_is_idempotent(s in ".*") {
            let s1 = super::sanitize_yaml_obsidian_links(&s);
            let s2 = super::sanitize_yaml_obsidian_links(&s1);
            prop_assert_eq!(s1.as_ref(), s2.as_ref());
        }
    }
}
