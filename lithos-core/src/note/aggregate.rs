//! Note aggregate root and identity types.

#![expect(missing_docs, reason = "Public API documented at type level")]

use std::{fmt, time::SystemTime};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};
use uuid::Uuid;

use crate::{
    config::aggregate::Config,
    note::{
        error::{NoteError, NoteMetadataError},
        frontmatter::Frontmatter,
        heading::Heading,
        inline_fields::InlineField,
        link::{FrontmatterLink, Link, ReferenceLink},
        list::ListItemEntry,
        paths::NotePath,
        raw::RawNote,
        structure::{BlockRef, Section},
        tag::Tag,
        task::Task,
        value::FieldValue,
    },
};

/// Stable identifier for a note.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct NoteId(Uuid);

impl NoteId {
    /// Creates a new random note identifier (UUID v7).
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Parses a note identifier from a string.
    ///
    /// # Errors
    /// Returns [`uuid::Error`] if the string is not a valid UUID.
    #[inline]
    pub fn parse(id: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(id)?))
    }
}

impl fmt::Display for NoteId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Default for NoteId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for NoteId {
    #[inline]
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<NoteId> for Uuid {
    #[inline]
    fn from(id: NoteId) -> Uuid {
        id.0
    }
}

/// Validated alias name for a note.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct AliasName(Box<str>);

impl AliasName {
    /// Creates a validated alias name.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Metadata`] if the alias is empty.
    #[inline]
    pub fn try_new(value: &str) -> Result<Self, NoteError> {
        if value.trim().is_empty() {
            return Err(NoteError::Metadata(NoteMetadataError::AliasEmpty));
        }
        Ok(Self(value.trim().into()))
    }

    /// Returns the alias as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AliasName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Validated file class name for a note.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct FileClassName(Box<str>);

impl FileClassName {
    /// Creates a validated file class name.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError::Metadata`] if the class is empty.
    #[inline]
    pub fn try_new(value: &str) -> Result<Self, NoteError> {
        if value.trim().is_empty() {
            return Err(NoteError::Metadata(NoteMetadataError::FileClassEmpty));
        }
        Ok(Self(value.trim().into()))
    }

    /// Returns the file class as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FileClassName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Normalized note facts derived from raw extraction output.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct Note {
    id: NoteId,
    path: NotePath,
    source_hash: Box<str>,
    source_bytes: u64,
    #[rkyv(with = Map<AsUnixTime>)]
    created_at: Option<SystemTime>,
    #[rkyv(with = Map<AsUnixTime>)]
    modified_at: Option<SystemTime>,
    frontmatter: Option<Frontmatter>,
    frontmatter_links: Vec<FrontmatterLink>,
    reference_links: Vec<ReferenceLink>,
    tags: Vec<Tag>,
    headings: Vec<Heading>,
    sections: Vec<Section>,
    links: Vec<Link>,
    block_refs: Vec<BlockRef>,
    list_items: Vec<ListItemEntry>,
    tasks: Vec<Task>,
    inline_fields: Vec<InlineField>,
}

impl Note {
    /// Construct normalized facts from ingestion output.
    #[expect(
        clippy::too_many_arguments,
        reason = "Note aggregates all note facts in one struct"
    )]
    pub(crate) fn from_parts(
        id: NoteId,
        path: NotePath,
        source_hash: Box<str>,
        source_bytes: u64,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
        frontmatter: Option<Frontmatter>,
        frontmatter_links: Vec<FrontmatterLink>,
        reference_links: Vec<ReferenceLink>,
        tags: Vec<Tag>,
        headings: Vec<Heading>,
        sections: Vec<Section>,
        links: Vec<Link>,
        block_refs: Vec<BlockRef>,
        list_items: Vec<ListItemEntry>,
        tasks: Vec<Task>,
        inline_fields: Vec<InlineField>,
    ) -> Self {
        Self {
            id,
            path,
            source_hash,
            source_bytes,
            created_at,
            modified_at,
            frontmatter,
            frontmatter_links,
            reference_links,
            tags,
            headings,
            sections,
            links,
            block_refs,
            list_items,
            tasks,
            inline_fields,
        }
    }

    #[inline]
    #[must_use]
    pub fn new(id: NoteId, path: NotePath) -> Self {
        Self {
            id,
            path,
            source_hash: "".into(),
            source_bytes: 0,
            created_at: None,
            modified_at: None,
            frontmatter: None,
            frontmatter_links: Vec::new(),
            reference_links: Vec::new(),
            tags: Vec::new(),
            headings: Vec::new(),
            sections: Vec::new(),
            links: Vec::new(),
            block_refs: Vec::new(),
            list_items: Vec::new(),
            tasks: Vec::new(),
            inline_fields: Vec::new(),
        }
    }

    #[inline]
    #[must_use]
    pub const fn id(&self) -> NoteId {
        self.id
    }

    /// Returns a copy of these facts with a different note id.
    #[inline]
    #[must_use]
    pub fn with_id(mut self, id: NoteId) -> Self {
        self.id = id;
        self
    }

    #[inline]
    #[must_use]
    pub fn path(&self) -> &NotePath {
        &self.path
    }

    #[inline]
    #[must_use]
    pub fn source_hash(&self) -> &str {
        &self.source_hash
    }

    #[inline]
    #[must_use]
    pub const fn source_bytes(&self) -> u64 {
        self.source_bytes
    }

    #[inline]
    #[must_use]
    pub const fn created_at(&self) -> Option<SystemTime> {
        self.created_at
    }

    #[inline]
    #[must_use]
    pub const fn modified_at(&self) -> Option<SystemTime> {
        self.modified_at
    }

    #[inline]
    #[must_use]
    pub fn frontmatter(&self) -> Option<&Frontmatter> {
        self.frontmatter.as_ref()
    }

    #[inline]
    #[must_use]
    pub fn frontmatter_links(&self) -> &[FrontmatterLink] {
        &self.frontmatter_links
    }

    #[inline]
    #[must_use]
    pub fn reference_links(&self) -> &[ReferenceLink] {
        &self.reference_links
    }

    #[inline]
    #[must_use]
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    #[inline]
    #[must_use]
    pub fn headings(&self) -> &[Heading] {
        &self.headings
    }

    #[inline]
    #[must_use]
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    #[inline]
    #[must_use]
    pub fn links(&self) -> &[Link] {
        &self.links
    }

    #[inline]
    #[must_use]
    pub fn block_refs(&self) -> &[BlockRef] {
        &self.block_refs
    }

    #[inline]
    #[must_use]
    pub fn list_items(&self) -> &[ListItemEntry] {
        &self.list_items
    }

    #[inline]
    #[must_use]
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    #[inline]
    #[must_use]
    pub fn inline_fields(&self) -> &[InlineField] {
        &self.inline_fields
    }

    fn add_tag(tags: &mut Vec<Tag>, tag: Tag) {
        if !tags.iter().any(|existing| existing.full_path() == tag.full_path())
        {
            tags.push(tag);
        }
    }

    fn collect_frontmatter_tags(
        frontmatter: &Frontmatter,
        config: &Config,
        tags: &mut Vec<Tag>,
    ) {
        let key = config.frontmatter().tags();
        let Some(value) = frontmatter.get(key.as_str()) else {
            return;
        };

        let mut collect_tokens = |text: &str| {
            for token in text.split(|ch: char| ch.is_whitespace() || ch == ',')
            {
                let token = token.trim();
                if token.is_empty() {
                    continue;
                }
                if let Ok(tag) = Tag::try_from_token(token) {
                    Note::add_tag(tags, tag);
                }
            }
        };

        if let Some(text) = value.as_str() {
            collect_tokens(text);
            return;
        }

        if let Some(values) = value.as_array() {
            for item in values {
                if let Some(text) = item.as_str() {
                    collect_tokens(text);
                }
            }
        }
    }

    fn collect_frontmatter_links(
        frontmatter: &Frontmatter,
        links: &mut Vec<FrontmatterLink>,
    ) {
        for (key, value) in frontmatter.fields() {
            Note::collect_frontmatter_links_for_value(key, value, links);
        }
    }

    fn collect_frontmatter_links_for_value(
        key: &str,
        value: &FieldValue,
        links: &mut Vec<FrontmatterLink>,
    ) {
        if let Some(text) = value.as_str() {
            if let Ok(Some(link)) =
                FrontmatterLink::parse_frontmatter_link(key, text)
            {
                links.push(link);
            }
            return;
        }

        if let Some(values) = value.as_array() {
            for item in values {
                if let Some(text) = Note::array_as_wikilink(item)
                    && let Ok(Some(link)) =
                        FrontmatterLink::parse_frontmatter_link(key, &text)
                {
                    links.push(link);
                    continue;
                }
                Note::collect_frontmatter_links_for_value(key, item, links);
            }
            return;
        }

        if let Some(values) = value.object_fields() {
            for (child_key, child_value) in values {
                let child_key_str: &str = child_key;
                let mut combined = String::with_capacity(
                    key.len()
                        .saturating_add(child_key_str.len())
                        .saturating_add(1),
                );
                combined.push_str(key);
                combined.push('.');
                combined.push_str(child_key_str);
                Note::collect_frontmatter_links_for_value(
                    &combined,
                    child_value,
                    links,
                );
            }
        }
    }

    fn array_as_wikilink(value: &FieldValue) -> Option<String> {
        let outer = value.as_array()?;
        if outer.len() != 1 {
            return None;
        }
        if let Some(text) = outer.first().and_then(FieldValue::as_str) {
            return Some(Note::wrap_wikilink_text(text));
        }
        let inner = outer.first()?.as_array()?;
        if inner.len() != 1 {
            return None;
        }
        let text = inner.first().and_then(FieldValue::as_str)?;
        Some(Note::wrap_wikilink_text(text))
    }

    fn wrap_wikilink_text(text: &str) -> String {
        let mut combined = String::with_capacity(text.len().saturating_add(4));
        combined.push_str("[[");
        combined.push_str(text);
        combined.push_str("]]");
        combined
    }
}

/// Conversion context for building `Note` from `RawNote` + Config.
pub(crate) struct RawNoteContext<'raw> {
    raw: &'raw RawNote,
    config: &'raw Config,
    id: NoteId,
}

impl<'raw> RawNoteContext<'raw> {
    #[inline]
    #[must_use]
    pub(crate) const fn new(
        id: NoteId,
        raw: &'raw RawNote,
        config: &'raw Config,
    ) -> Self {
        Self {
            raw,
            config,
            id,
        }
    }
}

impl<'raw> TryFrom<RawNoteContext<'raw>> for Note {
    type Error = NoteError;

    #[inline]
    fn try_from(ctx: RawNoteContext<'raw>) -> Result<Self, Self::Error> {
        let frontmatter = ctx
            .raw
            .frontmatter()
            .cloned()
            .map(Frontmatter::try_from)
            .transpose()?;

        let mut tags = Vec::new();
        for raw_tag in ctx.raw.tags() {
            if let Ok(tag) = Tag::try_from_token(raw_tag.value()) {
                Note::add_tag(&mut tags, tag);
            }
        }
        if let Some(frontmatter) = frontmatter.as_ref() {
            Note::collect_frontmatter_tags(frontmatter, ctx.config, &mut tags);
        }

        let mut frontmatter_links = Vec::new();
        if let Some(frontmatter) = frontmatter.as_ref() {
            Note::collect_frontmatter_links(
                frontmatter,
                &mut frontmatter_links,
            );
        }

        let reference_links = ctx
            .raw
            .reference_links()
            .iter()
            .cloned()
            .map(ReferenceLink::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        let headings = ctx
            .raw
            .headings()
            .iter()
            .cloned()
            .map(Heading::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let sections = ctx
            .raw
            .sections()
            .iter()
            .cloned()
            .map(Section::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let links = ctx
            .raw
            .links()
            .iter()
            .cloned()
            .map(Link::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let block_refs = ctx
            .raw
            .block_refs()
            .iter()
            .cloned()
            .map(BlockRef::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let mut list_items = ctx
            .raw
            .list_items()
            .iter()
            .cloned()
            .map(ListItemEntry::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        list_items.sort_by_key(ListItemEntry::position);

        let tasks = Task::build_many(ctx.raw, ctx.config)?;
        let inline_fields = ctx
            .raw
            .inline_fields()
            .iter()
            .cloned()
            .map(InlineField::from)
            .collect::<Vec<_>>();

        Ok(Self::from_parts(
            ctx.id,
            ctx.raw.path().clone(),
            ctx.raw.source_hash().into(),
            ctx.raw.source_bytes(),
            ctx.raw.created_at(),
            ctx.raw.modified_at(),
            frontmatter,
            frontmatter_links,
            reference_links,
            tags,
            headings,
            sections,
            links,
            block_refs,
            list_items,
            tasks,
            inline_fields,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{
        config::{
            aggregate::Config,
            raw::{RawConfig, RawFieldSpec, RawTaskConfig},
            vault::{VaultId, VaultRoot},
        },
        note::{
            inline_fields::{InlineFieldDelimiter, InlineFieldScanner},
            position::{SourceByteOffset, SourceByteRange},
            raw::{RawInlineField, RawTag, RawTask, RawTaskKind},
            task::RawTaskContext,
        },
    };

    #[test]
    fn promotes_only_when_task_tag_present() {
        let config = test_config_with_task_tag();

        let promoted = promote_task("#task Do work", &config, &[])
            .expect("task should be promoted");
        assert_eq!(promoted.text(), "Do work");

        let skipped = promote_task("Do work", &config, &[]);
        assert!(skipped.is_none());

        let skipped_partial = promote_task("#tasker Do work", &config, &[]);
        assert!(skipped_partial.is_none());
    }

    #[test]
    fn promoted_checkbox_extracts_text_and_metadata() {
        let config = config_with_fields();
        let task = promote_task(
            "#task Review PR [priority:: 2] [project:: lithos]",
            &config,
            &[],
        )
        .expect("task should be promoted");

        assert_eq!(task.text(), "Review PR");
        assert_eq!(task.metadata().get_number("priority"), Some(2.0f64));
        assert_eq!(task.metadata().get_string("project"), Some("lithos"));
    }

    #[test]
    fn promoted_checkbox_collects_hierarchical_tags() {
        let config = test_config_with_task_tag();
        let task =
            promote_task("#task Fix #work/project/urgent issue", &config, &[])
                .expect("task should be promoted");

        assert!(task.tags().any(|tag| tag.full_path() == "task"));
        assert!(
            task.tags().any(|tag| tag.full_path() == "work/project/urgent")
        );
        assert_eq!(task.tags().count(), 2);
    }

    #[test]
    fn promoted_checkbox_ignores_invalid_tags() {
        let config = test_config_with_task_tag();
        let task = promote_task("#task Review #bad/ tags", &config, &[])
            .expect("task should be promoted");

        assert!(task.tags().any(|tag| tag.full_path() == "task"));
        assert_eq!(task.tags().count(), 1);
    }

    #[test]
    fn promoted_checkbox_parses_dates() {
        let config = test_config_with_task_tag();
        let task = promote_task(
            "#task Test task with dates [created:: 2024-01-01] [due:: \
             2024-12-31]",
            &config,
            &[],
        )
        .expect("task should be promoted");

        if let Some(created_at) = task.created_at() {
            assert_eq!(created_at.as_i64(), 1_704_067_200);
            if let Some(due_at) = task.due_at() {
                assert!(created_at.is_past(Some(due_at)));
            }
        }

        if let Some(due_at) = task.due_at() {
            assert_eq!(due_at.as_i64(), 1_735_689_600);
            if let Some(created_at) = task.created_at() {
                assert!(due_at.is_future(Some(created_at)));
            }
        }
    }

    #[test]
    fn promoted_checkbox_parses_paren_inline_fields() {
        let config = config_with_fields();
        let task = promote_task(
            "#task Review PR (priority:: 2) (project:: lithos)",
            &config,
            &[],
        )
        .expect("task should be promoted");

        assert_eq!(task.text(), "Review PR");
        assert_eq!(task.metadata().get_number("priority"), Some(2.0f64));
        assert_eq!(task.metadata().get_string("project"), Some("lithos"));
    }

    #[test]
    fn promoted_checkbox_parses_default_emoji_dates() {
        let config = test_config_with_task_tag();
        let emojis = default_emoji_markers();
        let task = promote_task(
            "#task Do work \u{2795}2024-01-01 \u{1f4c5}2024-12-31 \
             \u{2705}2025-01-01",
            &config,
            &emojis,
        )
        .expect("task should be promoted");

        assert_eq!(
            task.created_at().map(|ts| ts.as_i64()),
            Some(1_704_067_200)
        );
        assert_eq!(task.due_at().map(|ts| ts.as_i64()), Some(1_735_603_200));
        assert_eq!(
            task.completed_at().map(|ts| ts.as_i64()),
            Some(1_735_689_600)
        );
    }

    fn promote_task(
        text: &str,
        config: &Config,
        emoji_markers: &[char],
    ) -> Option<Task> {
        let raw = raw_task_from_text(text, emoji_markers);
        let ctx = RawTaskContext::new(&raw, config);
        Option::<Task>::try_from(ctx).expect("task conversion")
    }

    fn raw_task_from_text(text: &str, emoji_markers: &[char]) -> RawTask {
        let start = SourceByteOffset::new(0);
        let end = SourceByteOffset::try_from_usize(text.len()).unwrap_or(start);
        let range = SourceByteRange::new(start, end).expect("valid test range");
        let tags = scan_raw_tags(text, start)
            .expect("raw tags")
            .into_iter()
            .map(|tag| tag.value().into())
            .collect();
        let mut inline_fields = Vec::new();
        InlineFieldScanner::scan_delimited(
            text,
            InlineFieldDelimiter::Brackets,
            |k, v, field_start, _| {
                let position = SourceByteOffset::try_from_usize(field_start)
                    .unwrap_or(start);
                inline_fields.push(RawInlineField::new(
                    k.into(),
                    v.into(),
                    position,
                ));
            },
        );
        InlineFieldScanner::scan_delimited(
            text,
            InlineFieldDelimiter::Parentheses,
            |k, v, field_start, _| {
                let position = SourceByteOffset::try_from_usize(field_start)
                    .unwrap_or(start);
                inline_fields.push(RawInlineField::new(
                    k.into(),
                    v.into(),
                    position,
                ));
            },
        );
        InlineFieldScanner::scan_emoji(
            text,
            emoji_markers,
            |k, v, field_start| {
                let position = SourceByteOffset::try_from_usize(field_start)
                    .unwrap_or(start);
                inline_fields.push(RawInlineField::new(
                    k.into(),
                    v.into(),
                    position,
                ));
            },
        );
        RawTask::new(
            RawTaskKind::Unchecked(' '),
            text.into(),
            tags,
            inline_fields,
            range,
        )
    }

    fn scan_raw_tags(
        text: &str,
        base_offset: SourceByteOffset,
    ) -> Result<Vec<RawTag>, NoteError> {
        let mut tags = Vec::new();
        let mut chars = text.char_indices().peekable();
        let mut prev_is_alnum = false;
        let base =
            usize::try_from(u32::from(base_offset)).map_err(|_error| {
                NoteError::Structure("tag offset out of range")
            })?;

        while let Some((start_idx, ch)) = chars.next() {
            if ch != '#' || prev_is_alnum {
                prev_is_alnum = ch.is_alphanumeric();
                continue;
            }

            let Some(mut end_idx) = start_idx.checked_add(ch.len_utf8()) else {
                prev_is_alnum = ch.is_alphanumeric();
                continue;
            };
            while let Some(&(next_idx, next_ch)) = chars.peek() {
                if !(next_ch.is_alphanumeric()
                    || matches!(next_ch, '_' | '-' | '/'))
                {
                    break;
                }
                chars.next();
                let Some(updated) = next_idx.checked_add(next_ch.len_utf8())
                else {
                    break;
                };
                end_idx = updated;
            }

            let Some(raw) = text.get(start_idx..end_idx) else {
                prev_is_alnum = ch.is_alphanumeric();
                continue;
            };

            if raw.len() > 1 {
                let offset = base.saturating_add(start_idx);
                let position = SourceByteOffset::try_from_usize(offset)?;
                tags.push(RawTag::new(raw.into(), position));
            }

            prev_is_alnum =
                raw.chars().last().is_some_and(char::is_alphanumeric);
        }

        Ok(tags)
    }

    fn default_emoji_markers() -> Vec<char> {
        vec![
            '\u{2795}',
            '\u{1f4c5}',
            '\u{2705}',
            '\u{23f3}',
            '\u{1f6eb}',
            '\u{274c}',
        ]
    }

    fn test_config_with_task_tag() -> Config {
        let raw = RawConfig {
            task: Some(RawTaskConfig {
                task_tags: Some(vec!["#task".into()]),
                ..Default::default()
            }),
            ..Default::default()
        };

        Config::build(
            &raw,
            VaultId::new(),
            VaultRoot::try_new(std::path::PathBuf::from("/vault"))
                .expect("vault root"),
            crate::config::aggregate::Version::initial(),
        )
        .expect("failed to build test config")
    }

    fn config_with_fields() -> Config {
        let mut fields = HashMap::new();
        fields.insert("priority".into(), RawFieldSpec::Integer {
            min: None,
            max: None,
        });
        fields.insert("project".into(), RawFieldSpec::String {
            pattern: None,
        });

        let raw = RawConfig {
            task: Some(RawTaskConfig {
                enabled: Some(true),
                task_tags: Some(vec!["#task".into()]),
                status: None,
                dates: None,
                fields: Some(fields),
                indexing: None,
                dependencies: None,
                use_emoji: None,
            }),
            ..Default::default()
        };

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
