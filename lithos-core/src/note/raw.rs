//! Raw note types and helpers for zero-copy ingestion.

#![expect(
    clippy::pattern_type_mismatch,
    reason = "Pattern matching style is clear in context"
)]
#![expect(
    clippy::iter_over_hash_type,
    reason = "Hash iteration order doesn't affect correctness here"
)]

use std::{borrow::Cow, sync::Arc, time::SystemTime};

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};
use regex::Regex;

use crate::{
    config::{
        frontmatter::FrontmatterConfigSpec, task::TaskConfigSpec,
        value::DateSpec,
    },
    note::{
        error::NoteParseError,
        paths::NotePath,
        position::{SourceByteOffset, SourceByteRange},
        scanner::ScannedArtifact,
        value::FieldValue,
    },
};

/// Raw block reference token extracted from note text.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawBlockRef<'source> {
    pub id: Cow<'source, str>,
    pub position: SourceByteOffset,
}

impl<'source> RawBlockRef<'source> {
    /// Create a raw block reference.
    #[inline]
    #[must_use]
    pub const fn new(
        id: Cow<'source, str>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            id,
            position,
        }
    }
}

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
    pub spec: Arc<FrontmatterConfigSpec>,
    pub kind: RawFrontmatterFormat,
    pub text: Cow<'source, str>,
    pub range: SourceByteRange,
}

impl<'source> RawFrontmatter<'source> {
    /// Create a raw frontmatter block.
    #[inline]
    #[must_use]
    pub const fn new(
        spec: Arc<FrontmatterConfigSpec>,
        kind: RawFrontmatterFormat,
        text: Cow<'source, str>,
        range: SourceByteRange,
    ) -> Self {
        Self {
            spec,
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
    ) -> Result<std::collections::HashMap<Box<str>, FieldValue>, NoteParseError>
    {
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

/// Raw heading extracted from the AST.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawHeading<'source> {
    pub level: u8,
    pub text: Cow<'source, str>,
    pub range: SourceByteRange,
    pub position: SourceByteOffset,
}

impl<'source> RawHeading<'source> {
    /// Create a new raw heading entry.
    #[inline]
    #[must_use]
    pub const fn new(
        level: u8,
        text: Cow<'source, str>,
        range: SourceByteRange,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            level,
            text,
            range,
            position,
        }
    }
}

/// Raw inline field extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawInlineField<'source> {
    pub key: Cow<'source, str>,
    pub value: RawFieldValue<'source>,
    pub range: SourceByteRange,
}

impl<'source> RawInlineField<'source> {
    /// Create a raw inline field entry.
    #[inline]
    #[must_use]
    pub const fn new(
        key: Cow<'source, str>,
        value: RawFieldValue<'source>,
        range: SourceByteRange,
    ) -> Self {
        Self {
            key,
            value,
            range,
        }
    }

    /// Map emoji key to keyword if the key is a recognized emoji in the task
    /// spec.
    ///
    /// Returns the mapped keyword if the key is a single emoji character
    /// that matches a temporal slot emoji in the spec, otherwise returns None.
    pub fn map_emoji_key(
        key: &str,
        task_spec: &TaskConfigSpec,
    ) -> Option<Box<str>> {
        // Check if key is single emoji character
        let mut chars = key.chars();
        let first = chars.next()?;
        if chars.next().is_some() {
            return None; // More than one char, not a single emoji
        }

        // Look up emoji in task spec temporal mappings
        for (keyword, (_slot, _date_spec, emoji_opt)) in
            &task_spec.temporal_specs
        {
            if let Some(emoji) = emoji_opt
                && *emoji == first
            {
                return Some(keyword.clone());
            }
        }

        None
    }
}

/// Typed value extracted from inline field during parsing.
///
/// This enum supports heuristic type detection during ingestion, allowing
/// the parser to type field values once instead of forcing every consumer
/// to re-parse strings.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum RawFieldValue<'source> {
    /// String value (fallback for unrecognized types).
    String(Cow<'source, str>),
    /// Numeric value (float).
    Number(f64),
    /// Date value (YYYY-MM-DD).
    Date(NaiveDate),
    /// Date/time value with offset.
    DateTime(DateTime<FixedOffset>),
    /// Wall clock time.
    Time(NaiveTime),
    /// Boolean value.
    Boolean(bool),
}

impl<'source> RawFieldValue<'source> {
    /// Attempt to parse a string into a typed value.
    ///
    /// Uses `DateSpec` format if provided for spec-aware parsing, otherwise
    /// falls back to heuristic parsing for common formats.
    ///
    /// # Type Detection Order
    /// 1. If `spec` provided: Try spec format first
    /// 2. Heuristic parsing:
    ///    - RFC3339 datetime
    ///    - Common date formats (YYYY-MM-DD, YYYY/MM/DD, etc.)
    ///    - Boolean (true/false, yes/no)
    ///    - Number (f64)
    /// 3. Fallback: String
    pub fn from_str_with_spec(
        text: &'source str,
        _key: &str,
        spec: Option<&DateSpec>,
    ) -> Self {
        // 1. Try spec format if provided
        if let Some(date_spec) = spec {
            if let Ok(d) = NaiveDate::parse_from_str(text, date_spec.format()) {
                return Self::Date(d);
            }
            if let Ok(dt) = DateTime::parse_from_str(text, date_spec.format()) {
                return Self::DateTime(dt);
            }
        }

        // 2. Heuristic parsing

        // Try RFC3339 datetime
        if let Ok(dt) = DateTime::parse_from_rfc3339(text) {
            return Self::DateTime(dt);
        }

        // Try common date formats
        let date_formats =
            ["%Y-%m-%d", "%Y/%m/%d", "%Y.%m.%d", "%d-%m-%Y", "%d/%m/%Y"];
        for fmt in date_formats {
            if let Ok(d) = NaiveDate::parse_from_str(text, fmt) {
                return Self::Date(d);
            }
        }

        // Try time formats
        let time_formats = ["%H:%M:%S", "%H:%M"];
        for fmt in time_formats {
            if let Ok(t) = NaiveTime::parse_from_str(text, fmt) {
                return Self::Time(t);
            }
        }

        // Try boolean
        match text.trim().to_lowercase().as_str() {
            "true" | "yes" => return Self::Boolean(true),
            "false" | "no" => return Self::Boolean(false),
            _ => {}
        }

        // Try number
        if let Ok(n) = text.trim().parse::<f64>() {
            return Self::Number(n);
        }

        // 3. Fallback to string
        Self::String(Cow::Borrowed(text))
    }

    /// Convert to owned variant for crossing lifetime boundaries.
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawFieldValue<'static> {
        match self {
            Self::String(s) => {
                RawFieldValue::String(Cow::Owned(s.into_owned()))
            }
            Self::Number(n) => RawFieldValue::Number(n),
            Self::Date(d) => RawFieldValue::Date(d),
            Self::DateTime(dt) => RawFieldValue::DateTime(dt),
            Self::Time(t) => RawFieldValue::Time(t),
            Self::Boolean(b) => RawFieldValue::Boolean(b),
        }
    }
}

/// Raw link style before validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawLinkStyle {
    Wiki,
    Markdown,
}

/// Raw link extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawLink<'source> {
    pub style: RawLinkStyle,
    pub is_embed: bool,
    pub target: Cow<'source, str>,
    pub alias: Option<Cow<'source, str>>,
    pub anchor: Option<Cow<'source, str>>,
    pub position: SourceByteOffset,
}

impl<'source> RawLink<'source> {
    /// Create a new raw link.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Raw links store full source context"
    )]
    pub const fn new(
        style: RawLinkStyle,
        is_embed: bool,
        target: Cow<'source, str>,
        alias: Option<Cow<'source, str>>,
        anchor: Option<Cow<'source, str>>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            style,
            is_embed,
            target,
            alias,
            anchor,
            position,
        }
    }
}

/// Raw task marker kind extracted from a list item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawTaskMarker {
    /// Unchecked task marker (typically `[ ]`).
    Unchecked(char),
    /// Checked task marker (typically `[x]`).
    Checked(char),
    /// Task marker with a non-standard symbol.
    Other(char),
}

impl RawTaskMarker {
    /// Create a raw task marker from a character.
    #[inline]
    #[must_use]
    pub fn from_char(marker: char) -> Self {
        match marker {
            ' ' => Self::Unchecked(marker),
            'x' | 'X' => Self::Checked(marker),
            _ => Self::Other(marker),
        }
    }

    /// Returns the raw marker character.
    #[inline]
    #[must_use]
    pub const fn marker(self) -> char {
        match self {
            Self::Unchecked(marker)
            | Self::Checked(marker)
            | Self::Other(marker) => marker,
        }
    }
}

/// Raw list type extracted from markdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawListKind {
    Ordered(u64),
    Unordered,
}

/// Raw list nesting depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawListDepth {
    Root,
    Nested(u8),
}

/// Raw list container extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawList {
    pub kind: RawListKind,
    pub depth: RawListDepth,
    pub range: SourceByteRange,
    pub task_spec: Arc<TaskConfigSpec>,
    pub item_positions: Vec<SourceByteOffset>,
}

impl RawList {
    /// Create a new raw list container.
    #[inline]
    #[must_use]
    pub fn new(
        kind: RawListKind,
        depth: RawListDepth,
        range: SourceByteRange,
        task_spec: Arc<TaskConfigSpec>,
        item_positions: Vec<SourceByteOffset>,
    ) -> Self {
        Self {
            kind,
            depth,
            range,
            task_spec,
            item_positions,
        }
    }

    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawList {
        RawList {
            kind: self.kind,
            depth: self.depth,
            range: self.range,
            task_spec: self.task_spec,
            item_positions: self.item_positions,
        }
    }
}

/// Raw list item extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawListItem<'source> {
    pub list_kind: RawListKind,
    pub depth: RawListDepth,
    pub text: Cow<'source, str>,
    pub task_marker: Option<RawTaskMarker>,
    pub range: SourceByteRange,
    pub text_range: SourceByteRange,
    pub parent: Option<SourceByteOffset>,
    pub tags: Vec<RawTag<'source>>,
    pub inline_fields: Vec<RawInlineField<'source>>,
}

impl<'source> RawListItem<'source> {
    /// Create a new raw list item.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Raw list items capture full source metadata"
    )]
    pub fn new(
        list_kind: RawListKind,
        depth: RawListDepth,
        text: Cow<'source, str>,
        task_marker: Option<RawTaskMarker>,
        range: SourceByteRange,
        text_range: SourceByteRange,
        parent: Option<SourceByteOffset>,
        tags: Vec<RawTag<'source>>,
        inline_fields: Vec<RawInlineField<'source>>,
    ) -> Self {
        Self {
            list_kind,
            depth,
            text,
            task_marker,
            range,
            text_range,
            parent,
            tags,
            inline_fields,
        }
    }
}

/// Raw reference-style link definition.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawReferenceLink<'source> {
    pub id: Cow<'source, str>,
    pub target: Cow<'source, str>,
    pub position: SourceByteOffset,
}

impl<'source> RawReferenceLink<'source> {
    /// Create a new raw reference link definition.
    #[inline]
    #[must_use]
    pub const fn new(
        id: Cow<'source, str>,
        target: Cow<'source, str>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            id,
            target,
            position,
        }
    }
}

/// Raw section kinds derived from AST nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawSectionKind {
    /// Heading section.
    Heading,
    /// Paragraph section.
    Paragraph,
    /// Code block section.
    CodeBlock,
    /// Block quote section.
    BlockQuote,
    /// List section.
    List,
    /// Frontmatter section.
    Frontmatter,
}

/// Raw section range with optional heading reference id.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawSection {
    pub kind: RawSectionKind,
    pub range: SourceByteRange,
    pub depth: u32,
}

impl RawSection {
    /// Create a raw section entry.
    #[inline]
    #[must_use]
    pub const fn new(
        kind: RawSectionKind,
        range: SourceByteRange,
        depth: u32,
    ) -> Self {
        Self {
            kind,
            range,
            depth,
        }
    }
}

/// Raw tag token extracted from text.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawTag<'source> {
    pub value: Cow<'source, str>,
    pub range: SourceByteRange,
}

impl<'source> RawTag<'source> {
    /// Create a raw tag token.
    #[inline]
    #[must_use]
    pub const fn new(value: Cow<'source, str>, range: SourceByteRange) -> Self {
        Self {
            value,
            range,
        }
    }
}

/// Raw note container with extracted, unvalidated data.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawNote<'source> {
    pub path: NotePath,
    pub source_hash: Box<str>,
    pub source_bytes: u64,
    pub created_at: Option<SystemTime>,
    pub modified_at: Option<SystemTime>,
    pub frontmatter: Option<RawFrontmatter<'source>>,
    pub headings: Vec<RawHeading<'source>>,
    pub sections: Vec<RawSection>,
    pub links: Vec<RawLink<'source>>,
    pub tags: Vec<RawTag<'source>>,
    pub lists: Vec<RawList>,
    pub list_items: Vec<RawListItem<'source>>,
    pub inline_fields: Vec<RawInlineField<'source>>,
    pub reference_links: Vec<RawReferenceLink<'source>>,
    pub block_refs: Vec<RawBlockRef<'source>>,
    pub master_artifacts: Vec<ScannedArtifact<'source>>,
}

impl<'source> RawNote<'source> {
    /// Create a new raw note container.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "RawNote bundles full extraction output"
    )]
    pub fn new(
        path: NotePath,
        source_hash: Box<str>,
        source_bytes: u64,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
        frontmatter: Option<RawFrontmatter<'source>>,
        headings: Vec<RawHeading<'source>>,
        sections: Vec<RawSection>,
        links: Vec<RawLink<'source>>,
        tags: Vec<RawTag<'source>>,
        lists: Vec<RawList>,
        list_items: Vec<RawListItem<'source>>,
        inline_fields: Vec<RawInlineField<'source>>,
        reference_links: Vec<RawReferenceLink<'source>>,
        block_refs: Vec<RawBlockRef<'source>>,
        master_artifacts: Vec<ScannedArtifact<'source>>,
    ) -> Self {
        Self {
            path,
            source_hash,
            source_bytes,
            created_at,
            modified_at,
            frontmatter,
            headings,
            sections,
            links,
            tags,
            lists,
            list_items,
            inline_fields,
            reference_links,
            block_refs,
            master_artifacts,
        }
    }

    /// Converts this raw note into an owned variant suitable for returning
    /// across file ingestion boundaries.
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawNote<'static> {
        RawNote {
            path: self.path,
            source_hash: self.source_hash,
            source_bytes: self.source_bytes,
            created_at: self.created_at,
            modified_at: self.modified_at,
            frontmatter: self.frontmatter.map(RawFrontmatter::into_owned),
            headings: self
                .headings
                .into_iter()
                .map(RawHeading::into_owned)
                .collect(),
            sections: self.sections,
            links: self.links.into_iter().map(RawLink::into_owned).collect(),
            tags: self.tags.into_iter().map(RawTag::into_owned).collect(),
            lists: self.lists.into_iter().map(RawList::into_owned).collect(),
            list_items: self
                .list_items
                .into_iter()
                .map(RawListItem::into_owned)
                .collect(),
            inline_fields: self
                .inline_fields
                .into_iter()
                .map(RawInlineField::into_owned)
                .collect(),
            reference_links: self
                .reference_links
                .into_iter()
                .map(RawReferenceLink::into_owned)
                .collect(),
            block_refs: self
                .block_refs
                .into_iter()
                .map(RawBlockRef::into_owned)
                .collect(),
            master_artifacts: self
                .master_artifacts
                .into_iter()
                .map(ScannedArtifact::into_owned)
                .collect(),
        }
    }
}

impl RawBlockRef<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawBlockRef<'static> {
        RawBlockRef {
            id: Cow::Owned(self.id.into_owned()),
            position: self.position,
        }
    }
}

impl RawFrontmatter<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawFrontmatter<'static> {
        RawFrontmatter {
            spec: self.spec,
            kind: self.kind,
            text: Cow::Owned(self.text.into_owned()),
            range: self.range,
        }
    }
}

impl RawHeading<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawHeading<'static> {
        RawHeading {
            level: self.level,
            text: Cow::Owned(self.text.into_owned()),
            range: self.range,
            position: self.position,
        }
    }
}

impl RawInlineField<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawInlineField<'static> {
        RawInlineField {
            key: Cow::Owned(self.key.into_owned()),
            value: self.value.into_owned(),
            range: self.range,
        }
    }
}

impl RawLink<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawLink<'static> {
        RawLink {
            style: self.style,
            is_embed: self.is_embed,
            target: Cow::Owned(self.target.into_owned()),
            alias: self.alias.map(|alias| Cow::Owned(alias.into_owned())),
            anchor: self.anchor.map(|anchor| Cow::Owned(anchor.into_owned())),
            position: self.position,
        }
    }
}

impl RawListItem<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawListItem<'static> {
        RawListItem {
            list_kind: self.list_kind,
            depth: self.depth,
            text: Cow::Owned(self.text.into_owned()),
            task_marker: self.task_marker,
            range: self.range,
            text_range: self.text_range,
            parent: self.parent,
            tags: self.tags.into_iter().map(RawTag::into_owned).collect(),
            inline_fields: self
                .inline_fields
                .into_iter()
                .map(RawInlineField::into_owned)
                .collect(),
        }
    }
}

impl RawReferenceLink<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawReferenceLink<'static> {
        RawReferenceLink {
            id: Cow::Owned(self.id.into_owned()),
            target: Cow::Owned(self.target.into_owned()),
            position: self.position,
        }
    }
}

impl RawTag<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawTag<'static> {
        RawTag {
            value: Cow::Owned(self.value.into_owned()),
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
    use std::collections::HashMap;

    use proptest::prelude::*;
    use rstest::*;

    use super::*;

    #[fixture]
    fn default_spec() -> FrontmatterConfigSpec {
        FrontmatterConfigSpec::new(
            "title".into(),
            "aliases".into(),
            "tags".into(),
            "file_class".into(),
            "date_created".into(),
            "date_modified".into(),
        )
    }

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
        FieldValue::Object(Box::new(HashMap::from([("inner".into(), FieldValue::Boolean(true))])))
    )]
    fn should_parse_valid_formats(
        #[case] format: RawFrontmatterFormat,
        #[case] text: &str,
        #[case] key: &str,
        #[case] expected: FieldValue,
        default_spec: FrontmatterConfigSpec,
    ) {
        let raw = RawFrontmatter::new(
            std::sync::Arc::new(default_spec),
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
    fn should_report_yaml_syntax_error_with_location(
        default_spec: FrontmatterConfigSpec,
    ) {
        let raw = RawFrontmatter::new(
            std::sync::Arc::new(default_spec),
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
    fn should_report_toml_syntax_error(default_spec: FrontmatterConfigSpec) {
        let raw = RawFrontmatter::new(
            std::sync::Arc::new(default_spec),
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
    fn should_parse_yaml_with_unquoted_links(
        #[case] input: &str,
        default_spec: FrontmatterConfigSpec,
    ) {
        let raw = RawFrontmatter::new(
            std::sync::Arc::new(default_spec),
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
