//! Note aggregate root and identity types.

#![expect(missing_docs, reason = "Public API documented at type level")]

use std::{fmt, time::SystemTime};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};
use uuid::Uuid;

use crate::{
    config::{aggregate::Config, task::StatusSymbol},
    note::{
        error::{NoteError, NoteMetadataError, TaskError},
        frontmatter::Frontmatter,
        heading::{Heading, HeadingLevel},
        link::{Anchor, EmbedType, FrontmatterLink, Link, Target},
        list::ListItemEntry,
        paths::NotePath,
        raw::{
            block_refs::RawBlockRef,
            headings::RawHeading,
            links::{RawLink, RawLinkStyle},
            note::RawNote,
            sections::{RawSection, RawSectionKind},
            tasks::RawTask,
        },
        structure::{BlockRef, BlockRefId, Section, SectionKind},
        tag::Tag,
        task::{
            Task, TaskAttributes, TaskAttributesBuilder, TaskFieldKey,
            TaskMetadata, TaskTimestamp,
        },
        value::FieldValue,
    },
};

type RawInlineField = (Box<str>, Box<str>);

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
            position::SourceByteOffset,
            raw::{
                tags::scan_raw_tags, task_tokens::RawTaskTokens, tasks::RawTask,
            },
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
        let base = SourceByteOffset::new(0);
        let tags = scan_raw_tags(text, base)
            .expect("raw tags")
            .into_iter()
            .map(|tag| tag.value().into())
            .collect();
        let tokens = RawTaskTokens::parse(text, emoji_markers);
        RawTask::new(
            Some(' '),
            text.into(),
            tags,
            tokens.inline_fields().to_vec(),
            tokens.emoji_dates().to_vec(),
            base,
        )
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
pub struct NoteFacts {
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
    tags: Vec<Tag>,
    headings: Vec<Heading>,
    sections: Vec<Section>,
    links: Vec<Link>,
    block_refs: Vec<BlockRef>,
    list_items: Vec<ListItemEntry>,
    tasks: Vec<Task>,
}

impl NoteFacts {
    /// Construct normalized facts from ingestion output.
    #[expect(
        clippy::too_many_arguments,
        reason = "NoteFacts aggregates all note facts in one struct"
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
        tags: Vec<Tag>,
        headings: Vec<Heading>,
        sections: Vec<Section>,
        links: Vec<Link>,
        block_refs: Vec<BlockRef>,
        list_items: Vec<ListItemEntry>,
        tasks: Vec<Task>,
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
            tags,
            headings,
            sections,
            links,
            block_refs,
            list_items,
            tasks,
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
            tags: Vec::new(),
            headings: Vec::new(),
            sections: Vec::new(),
            links: Vec::new(),
            block_refs: Vec::new(),
            list_items: Vec::new(),
            tasks: Vec::new(),
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
}

impl TryFrom<RawHeading> for Heading {
    type Error = NoteError;

    #[inline]
    fn try_from(raw: RawHeading) -> Result<Self, Self::Error> {
        let level = HeadingLevel::try_new(raw.level())?;
        Heading::try_new(level, raw.text(), raw.position())
    }
}

impl TryFrom<RawSection> for Section {
    type Error = NoteError;

    #[inline]
    fn try_from(raw: RawSection) -> Result<Self, Self::Error> {
        let kind = match raw.kind() {
            RawSectionKind::Heading => SectionKind::Heading,
            RawSectionKind::Paragraph => SectionKind::Paragraph,
            RawSectionKind::CodeBlock => SectionKind::Code,
            RawSectionKind::BlockQuote => SectionKind::BlockQuote,
            RawSectionKind::List => SectionKind::List,
        };
        Ok(Section::new(kind, None, raw.range()))
    }
}

impl TryFrom<RawLink> for Link {
    type Error = NoteError;

    #[inline]
    fn try_from(raw: RawLink) -> Result<Self, Self::Error> {
        let target_text = raw.target();
        let is_external =
            crate::note::raw::links::is_external_target(target_text);
        let anchor = if is_external {
            None
        } else {
            raw.anchor().map(anchor_from_raw).transpose()?
        };
        let target = if is_external {
            Target::External {
                url: target_text.into(),
            }
        } else {
            Target::Unresolved {
                raw: target_text.into(),
            }
        };
        let alias = raw.alias();

        match (raw.is_embed(), raw.style()) {
            (true, RawLinkStyle::Wiki) => Link::try_new_embed(
                target,
                EmbedType::from_extension(target_text),
                alias,
                anchor,
                raw.position(),
            ),
            (true, RawLinkStyle::Markdown) => Link::try_new_markdown_embed(
                target,
                EmbedType::from_extension(target_text),
                alias,
                raw.position(),
            ),
            (false, RawLinkStyle::Wiki) => {
                Link::try_new_wikilink(target, alias, anchor, raw.position())
            }
            (false, RawLinkStyle::Markdown) => Link::try_new_markdown_link(
                target,
                alias,
                anchor,
                raw.position(),
            ),
        }
    }
}

impl TryFrom<RawBlockRef> for BlockRef {
    type Error = NoteError;

    #[inline]
    fn try_from(raw: RawBlockRef) -> Result<Self, Self::Error> {
        let id = BlockRefId::try_new(raw.id())?;
        Ok(BlockRef::new(id, raw.position()))
    }
}

fn anchor_from_raw(text: &str) -> Result<Anchor, NoteError> {
    if let Some(block_ref) = text.strip_prefix('^') {
        Anchor::block_ref(block_ref)
    } else {
        Anchor::heading(text)
    }
}

/// Conversion context for building `NoteFacts` from `RawNote` + Config.
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

impl<'raw> TryFrom<RawNoteContext<'raw>> for NoteFacts {
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
                add_tag(&mut tags, tag);
            }
        }
        if let Some(frontmatter) = frontmatter.as_ref() {
            collect_frontmatter_tags(frontmatter, ctx.config, &mut tags);
        }

        let mut frontmatter_links = Vec::new();
        if let Some(frontmatter) = frontmatter.as_ref() {
            collect_frontmatter_links(frontmatter, &mut frontmatter_links);
        }

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

        let tasks = build_tasks(ctx.raw, ctx.config)?;

        Ok(Self::from_parts(
            ctx.id,
            ctx.raw.path().clone(),
            ctx.raw.source_hash().into(),
            ctx.raw.source_bytes(),
            ctx.raw.created_at(),
            ctx.raw.modified_at(),
            frontmatter,
            frontmatter_links,
            tags,
            headings,
            sections,
            links,
            block_refs,
            list_items,
            tasks,
        ))
    }
}

fn build_tasks(raw: &RawNote, config: &Config) -> Result<Vec<Task>, NoteError> {
    if raw.tasks().is_empty() {
        return Ok(Vec::new());
    }

    let mut tasks = Vec::new();
    for raw_task in raw.tasks() {
        let ctx = RawTaskContext::new(raw_task, config);
        if let Some(task) = Option::<Task>::try_from(ctx)? {
            tasks.push(task);
        }
    }
    Ok(tasks)
}

fn add_tag(tags: &mut Vec<Tag>, tag: Tag) {
    if !tags.iter().any(|existing| existing.full_path() == tag.full_path()) {
        tags.push(tag);
    }
}

fn collect_frontmatter_tags(
    frontmatter: &Frontmatter,
    config: &Config,
    tags: &mut Vec<Tag>,
) {
    let key = config.frontmatter().tags();
    let Some(value) = frontmatter.get(key) else {
        return;
    };

    let mut collect_tokens = |text: &str| {
        for token in text.split(|ch: char| ch.is_whitespace() || ch == ',') {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }
            if let Ok(tag) = Tag::try_from_token(token) {
                add_tag(tags, tag);
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
        collect_frontmatter_links_for_value(key, value, links);
    }
}

fn collect_frontmatter_links_for_value(
    key: &str,
    value: &FieldValue,
    links: &mut Vec<FrontmatterLink>,
) {
    if let Some(text) = value.as_str() {
        if let Ok(Some(link)) = parse_frontmatter_link(key, text) {
            links.push(link);
        }
        return;
    }

    if let Some(values) = value.as_array() {
        for item in values {
            if let Some(text) = array_as_wikilink(item)
                && let Ok(Some(link)) = parse_frontmatter_link(key, &text)
            {
                links.push(link);
                continue;
            }
            collect_frontmatter_links_for_value(key, item, links);
        }
        return;
    }

    if let Some(values) = value.object_fields() {
        for (child_key, child_value) in values {
            let child_key_str: &str = child_key;
            let mut combined = String::with_capacity(
                key.len().saturating_add(child_key_str.len()).saturating_add(1),
            );
            combined.push_str(key);
            combined.push('.');
            combined.push_str(child_key_str);
            collect_frontmatter_links_for_value(&combined, child_value, links);
        }
    }
}

fn array_as_wikilink(value: &FieldValue) -> Option<String> {
    let outer = value.as_array()?;
    if outer.len() != 1 {
        return None;
    }
    if let Some(text) = outer.first().and_then(FieldValue::as_str) {
        return Some(wrap_wikilink_text(text));
    }
    let inner = outer.first()?.as_array()?;
    if inner.len() != 1 {
        return None;
    }
    let text = inner.first().and_then(FieldValue::as_str)?;
    Some(wrap_wikilink_text(text))
}

fn wrap_wikilink_text(text: &str) -> String {
    let mut combined = String::with_capacity(text.len().saturating_add(4));
    combined.push_str("[[");
    combined.push_str(text);
    combined.push_str("]]");
    combined
}

fn parse_frontmatter_link(
    key: &str,
    value: &str,
) -> Result<Option<FrontmatterLink>, NoteError> {
    let trimmed = value.trim();
    let (embed, inner) = if let Some(rest) =
        trimmed.strip_prefix("![[").and_then(|rest| rest.strip_suffix("]]"))
    {
        (true, rest)
    } else if let Some(rest) =
        trimmed.strip_prefix("[[").and_then(|rest| rest.strip_suffix("]]"))
    {
        (false, rest)
    } else {
        return Ok(None);
    };

    let (target_text, alias) =
        if let Some((left, right)) = inner.split_once('|') {
            (left.trim(), Some(right.trim()))
        } else {
            (inner.trim(), None)
        };

    if target_text.is_empty() {
        return Ok(None);
    }

    let (target_path, anchor) = split_target_and_anchor(target_text)?;
    let target = if is_external_target(target_path) {
        Target::External {
            url: target_path.into(),
        }
    } else {
        Target::Unresolved {
            raw: target_path.into(),
        }
    };
    let embed_type = embed.then(|| EmbedType::from_extension(target_path));

    Ok(Some(FrontmatterLink::new(
        key.into(),
        target,
        anchor,
        alias.filter(|text| !text.is_empty()).map(Into::into),
        embed_type,
    )))
}

fn split_target_and_anchor(
    target: &str,
) -> Result<(&str, Option<Anchor>), NoteError> {
    let Some((path, anchor_text)) = target.split_once('#') else {
        return Ok((target, None));
    };

    if let Some(block_ref) = anchor_text.strip_prefix('^') {
        Ok((path, Some(Anchor::block_ref(block_ref)?)))
    } else {
        Ok((path, Some(Anchor::heading(anchor_text)?)))
    }
}

fn is_external_target(target: &str) -> bool {
    target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("ftp://")
        || target.starts_with("mailto:")
}

struct RawTaskContext<'raw> {
    raw: &'raw RawTask,
    config: &'raw Config,
}

impl<'raw> RawTaskContext<'raw> {
    #[inline]
    const fn new(raw: &'raw RawTask, config: &'raw Config) -> Self {
        Self {
            raw,
            config,
        }
    }
}

impl<'raw> TryFrom<RawTaskContext<'raw>> for Option<Task> {
    type Error = NoteError;

    #[inline]
    fn try_from(ctx: RawTaskContext<'raw>) -> Result<Self, Self::Error> {
        let Some(symbol) = ctx.raw.status_symbol() else {
            return Ok(None);
        };
        let status_symbol = StatusSymbol::try_new(symbol)?;
        let tags = ctx
            .raw
            .tags()
            .iter()
            .filter_map(|tag| Tag::try_from_token(tag).ok())
            .collect::<Vec<_>>();
        let builder = TaskBuilder::new(ctx.config.task());
        builder.promote_from_raw(ctx.raw, tags, status_symbol)
    }
}

struct TaskBuilder<'config> {
    config: &'config crate::config::task::Task,
}

impl<'config> TaskBuilder<'config> {
    #[inline]
    const fn new(config: &'config crate::config::task::Task) -> Self {
        Self {
            config,
        }
    }

    fn promote_from_raw(
        &self,
        raw: &RawTask,
        tags: Vec<Tag>,
        status_symbol: StatusSymbol,
    ) -> Result<Option<Task>, NoteError> {
        if !self.should_promote_from_tags(&tags) {
            return Ok(None);
        }

        let status = self
            .config
            .status()
            .name_for_symbol(status_symbol)
            .ok_or_else(|| {
                NoteError::Task(TaskError::UnrecognizedStatusSymbol {
                    symbol: status_symbol.value(),
                })
            })?
            .clone();
        let text = self.extract_clean_text(raw.text())?;
        let parsed =
            self.parse_inline_fields(raw.inline_fields(), raw.emoji_dates())?;
        let attributes = parsed.into_attributes(tags);

        Task::try_new(status, text, raw.position(), attributes).map(Some)
    }

    fn should_promote_from_tags(&self, tags: &[Tag]) -> bool {
        self.config.tags().iter().any(|config_tag| {
            tags.iter().any(|tag| {
                config_tag
                    .as_str()
                    .strip_prefix('#')
                    .is_some_and(|raw| raw == tag.full_path())
            })
        })
    }

    fn extract_clean_text(
        &self,
        raw_text: &str,
    ) -> Result<Box<str>, NoteError> {
        let mut text = raw_text.trim();

        let mut stripped = true;
        while stripped {
            stripped = false;
            for tag in self.config.tags() {
                if let Some(rest) = text.strip_prefix(tag.as_str()) {
                    text = rest.trim_start();
                    stripped = true;
                }
            }
        }

        if let Some(prefix) = Self::strip_inline_fields(text) {
            text = prefix.trim_end();
        }

        if text.trim().is_empty() {
            return Err(NoteError::Task(TaskError::EmptyText));
        }

        Ok(text.into())
    }

    fn parse_inline_fields(
        &self,
        inline_fields: &[RawInlineField],
        emoji_dates: &[RawInlineField],
    ) -> Result<ParsedInlineFields, NoteError> {
        let mut state = InlineFieldState::new();

        for (keyword, raw_value) in
            inline_fields.iter().map(|pair| (pair.0.as_ref(), pair.1.as_ref()))
        {
            state.handle_inline_field(self.config, keyword, raw_value)?;
        }

        state.fill_emoji_dates_from_tokens(self.config, emoji_dates)?;
        state.fill_default_emoji_dates_from_tokens(emoji_dates)?;

        Ok(state.finish())
    }

    fn strip_inline_fields(text: &str) -> Option<&str> {
        let bracket = Self::inline_field_start(text, b'[', b']');
        let paren = Self::inline_field_start(text, b'(', b')');
        let start = match (bracket, paren) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }?;
        text.get(..start)
    }

    fn inline_field_start(
        text: &str,
        open_delim: u8,
        close_delim: u8,
    ) -> Option<usize> {
        let bytes = text.as_bytes();
        let mut cursor = 0;
        while let Some(open_rel) = bytes
            .get(cursor..)
            .and_then(|slice| slice.iter().position(|&b| b == open_delim))
        {
            let open = cursor.saturating_add(open_rel);
            let after_open = open.saturating_add(1);
            let Some(close_rel) = bytes
                .get(after_open..)
                .and_then(|slice| slice.iter().position(|&b| b == close_delim))
            else {
                break;
            };
            let close = after_open.saturating_add(close_rel);
            let Some(inner) = text.get(after_open..close) else {
                break;
            };
            if let Some((key, value)) = inner.split_once("::")
                && !key.trim().is_empty()
                && !value.trim().is_empty()
            {
                return Some(open);
            }
            cursor = close.saturating_add(1);
        }
        None
    }
}

#[derive(Debug)]
struct ParsedInlineFields {
    slots: TemporalSlots,
    metadata: TaskMetadata,
}

impl ParsedInlineFields {
    fn into_attributes(self, tags: Vec<Tag>) -> TaskAttributes {
        self.slots
            .apply_to_builder(TaskAttributes::builder().tags(tags))
            .metadata(self.metadata)
            .build()
    }
}

#[derive(Debug, Clone, Copy)]
enum DateSlot {
    Created,
    Due,
    Reminder,
    Completed,
}

#[derive(Debug, Default)]
struct TemporalSlots {
    created: Option<TaskTimestamp>,
    due: Option<TaskTimestamp>,
    reminder: Option<TaskTimestamp>,
    completed: Option<TaskTimestamp>,
}

impl TemporalSlots {
    fn finish(self, metadata: TaskMetadata) -> ParsedInlineFields {
        ParsedInlineFields {
            slots: self,
            metadata,
        }
    }

    fn get(&self, slot: DateSlot) -> Option<TaskTimestamp> {
        match slot {
            DateSlot::Created => self.created,
            DateSlot::Due => self.due,
            DateSlot::Reminder => self.reminder,
            DateSlot::Completed => self.completed,
        }
    }

    fn set(&mut self, slot: DateSlot, value: TaskTimestamp) {
        match slot {
            DateSlot::Created => self.created = Some(value),
            DateSlot::Due => self.due = Some(value),
            DateSlot::Reminder => self.reminder = Some(value),
            DateSlot::Completed => self.completed = Some(value),
        }
    }

    fn apply_to_builder(
        self,
        builder: TaskAttributesBuilder,
    ) -> TaskAttributesBuilder {
        builder
            .created_at(self.created)
            .due_at(self.due)
            .reminder_at(self.reminder)
            .completed_at(self.completed)
    }
}

#[derive(Debug, Default)]
struct InlineFieldState {
    slots: TemporalSlots,
    metadata: TaskMetadata,
}

impl InlineFieldState {
    fn new() -> Self {
        Self::default()
    }

    fn handle_inline_field(
        &mut self,
        config: &crate::config::task::Task,
        keyword: &str,
        raw_value: &str,
    ) -> Result<(), NoteError> {
        if let Some((slot, spec)) = Self::match_date_spec(config, keyword) {
            let parsed = Self::parse_date_str(raw_value, spec)?;
            self.slots.set(slot, parsed);
            return Ok(());
        }

        Self::insert_metadata(config, &mut self.metadata, keyword, raw_value)
    }

    fn fill_emoji_dates_from_tokens(
        &mut self,
        config: &crate::config::task::Task,
        tokens: &[RawInlineField],
    ) -> Result<(), NoteError> {
        for (emoji, value) in
            tokens.iter().map(|pair| (pair.0.as_ref(), pair.1.as_ref()))
        {
            if let Some((slot, spec)) =
                Self::match_date_spec_by_emoji(config, emoji)
            {
                if self.slots.get(slot).is_some() {
                    continue;
                }
                let parsed = Self::parse_date_str(value, spec)?;
                self.slots.set(slot, parsed);
            }
        }

        Ok(())
    }

    fn fill_default_emoji_dates_from_tokens(
        &mut self,
        tokens: &[RawInlineField],
    ) -> Result<(), NoteError> {
        for (emoji, value) in
            tokens.iter().map(|pair| (pair.0.as_ref(), pair.1.as_ref()))
        {
            match () {
                () if Self::emoji_matches(emoji, '\u{2795}') => {
                    self.fill_default_slot_value(
                        DateSlot::Created,
                        "created",
                        value,
                    )?;
                }
                () if Self::emoji_matches(emoji, '\u{1f4c5}') => {
                    self.fill_default_slot_value(DateSlot::Due, "due", value)?;
                }
                () if Self::emoji_matches(emoji, '\u{2705}') => {
                    self.fill_default_slot_value(
                        DateSlot::Completed,
                        "completed",
                        value,
                    )?;
                }
                () if Self::emoji_matches(emoji, '\u{23f3}') => {
                    self.fill_default_metadata_value("scheduled", value)?;
                }
                () if Self::emoji_matches(emoji, '\u{1f6eb}') => {
                    self.fill_default_metadata_value("start", value)?;
                }
                () if Self::emoji_matches(emoji, '\u{274c}') => {
                    self.fill_default_metadata_value("cancelled", value)?;
                }
                () => {}
            }
        }

        Ok(())
    }

    fn finish(self) -> ParsedInlineFields {
        self.slots.finish(self.metadata)
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics keep spec parsing concise."
    )]
    fn parse_metadata_value(
        raw_value: &str,
        spec: &crate::config::value::FieldSpec,
    ) -> Result<serde_json::Value, NoteError> {
        match spec {
            crate::config::value::FieldSpec::Integer {
                ..
            } => {
                let value = raw_value.parse::<i64>().map_err(|_error| {
                    NoteError::Task(TaskError::InvalidInteger {
                        raw: raw_value.into(),
                        reason: "failed to parse integer",
                    })
                })?;
                Ok(serde_json::Value::Number(value.into()))
            }
            crate::config::value::FieldSpec::Float {
                ..
            } => {
                let value = raw_value.parse::<f64>().map_err(|_error| {
                    NoteError::Task(TaskError::InvalidFloat {
                        raw: raw_value.into(),
                        reason: "failed to parse float",
                    })
                })?;
                let number =
                    serde_json::Number::from_f64(value).ok_or_else(|| {
                        NoteError::Task(TaskError::InvalidFloat {
                            raw: raw_value.into(),
                            reason: "float value is not finite",
                        })
                    })?;
                Ok(serde_json::Value::Number(number))
            }
            crate::config::value::FieldSpec::Enum {
                ..
            }
            | crate::config::value::FieldSpec::String {
                ..
            }
            | crate::config::value::FieldSpec::DateTime {
                ..
            } => Ok(serde_json::Value::String(raw_value.into())),
        }
    }

    fn match_date_spec<'config>(
        config: &'config crate::config::task::Task,
        keyword: &str,
    ) -> Option<(DateSlot, &'config crate::config::value::DateSpec)> {
        if let Some(spec) = config.created()
            && spec.keyword().as_str() == keyword
        {
            return Some((DateSlot::Created, spec));
        }
        if let Some(spec) = config.due()
            && spec.keyword().as_str() == keyword
        {
            return Some((DateSlot::Due, spec));
        }
        if let Some(spec) = config.reminder()
            && spec.keyword().as_str() == keyword
        {
            return Some((DateSlot::Reminder, spec));
        }
        if let Some(spec) = config.completed()
            && spec.keyword().as_str() == keyword
        {
            return Some((DateSlot::Completed, spec));
        }
        None
    }

    fn insert_metadata(
        config: &crate::config::task::Task,
        metadata: &mut TaskMetadata,
        keyword: &str,
        raw_value: &str,
    ) -> Result<(), NoteError> {
        if let Some(spec) = config.field_spec(keyword) {
            let json_value = Self::parse_metadata_value(raw_value, spec)?;
            spec.validate_raw_value(&json_value).map_err(|_error| {
                NoteError::Task(TaskError::InvalidMetadataField {
                    keyword: keyword.into(),
                    reason: "failed validation",
                })
            })?;
            let field_value =
                FieldValue::try_from_json(&json_value).map_err(|_error| {
                    NoteError::Task(TaskError::InvalidMetadataField {
                        keyword: keyword.into(),
                        reason: "failed conversion",
                    })
                })?;
            let key = TaskFieldKey::try_new(keyword)?;
            metadata.insert(key, field_value);
        } else {
            let key = TaskFieldKey::try_new(keyword)?;
            metadata.insert(key, FieldValue::String(raw_value.into()));
        }

        Ok(())
    }

    fn parse_date_str(
        raw_value: &str,
        spec: &crate::config::value::DateSpec,
    ) -> Result<TaskTimestamp, NoteError> {
        if let Ok(naive) =
            chrono::NaiveDateTime::parse_from_str(raw_value, spec.format())
        {
            return Ok(TaskTimestamp::new(naive.and_utc().timestamp()));
        }

        let date = chrono::NaiveDate::parse_from_str(raw_value, spec.format())
            .map_err(|_error| {
                NoteError::Task(TaskError::InvalidDate {
                    keyword: spec.keyword().as_str().into(),
                    reason: "failed to parse date string",
                })
            })?;
        let naive = date.and_hms_opt(0, 0, 0).ok_or_else(|| {
            NoteError::Task(TaskError::InvalidDateTime {
                keyword: spec.keyword().as_str().into(),
            })
        })?;

        Ok(TaskTimestamp::new(naive.and_utc().timestamp()))
    }

    fn parse_default_date(
        raw_value: &str,
        field: &str,
    ) -> Result<TaskTimestamp, NoteError> {
        let formats = ["%Y-%m-%dT%H:%M", "%Y-%m-%d %H:%M"];
        for format in formats {
            if let Ok(naive) =
                chrono::NaiveDateTime::parse_from_str(raw_value, format)
            {
                return Ok(TaskTimestamp::new(naive.and_utc().timestamp()));
            }
        }

        let date = chrono::NaiveDate::parse_from_str(raw_value, "%Y-%m-%d")
            .map_err(|_error| {
                NoteError::Task(TaskError::InvalidDate {
                    keyword: field.into(),
                    reason: "failed to parse date string",
                })
            })?;
        let naive = date.and_hms_opt(0, 0, 0).ok_or_else(|| {
            NoteError::Task(TaskError::InvalidDateTime {
                keyword: field.into(),
            })
        })?;
        Ok(TaskTimestamp::new(naive.and_utc().timestamp()))
    }

    fn match_date_spec_by_emoji<'config>(
        config: &'config crate::config::task::Task,
        emoji: &str,
    ) -> Option<(DateSlot, &'config crate::config::value::DateSpec)> {
        if let Some(spec) = config.created()
            && spec.emoji().is_some_and(|spec_emoji| {
                Self::emoji_matches(emoji, spec_emoji)
            })
        {
            return Some((DateSlot::Created, spec));
        }
        if let Some(spec) = config.due()
            && spec.emoji().is_some_and(|spec_emoji| {
                Self::emoji_matches(emoji, spec_emoji)
            })
        {
            return Some((DateSlot::Due, spec));
        }
        if let Some(spec) = config.reminder()
            && spec.emoji().is_some_and(|spec_emoji| {
                Self::emoji_matches(emoji, spec_emoji)
            })
        {
            return Some((DateSlot::Reminder, spec));
        }
        if let Some(spec) = config.completed()
            && spec.emoji().is_some_and(|spec_emoji| {
                Self::emoji_matches(emoji, spec_emoji)
            })
        {
            return Some((DateSlot::Completed, spec));
        }
        None
    }

    fn emoji_matches(token: &str, emoji: char) -> bool {
        let mut chars = token.chars();
        matches!(chars.next(), Some(first) if first == emoji)
            && chars.next().is_none()
    }

    fn fill_default_slot_value(
        &mut self,
        slot: DateSlot,
        label: &str,
        value: &str,
    ) -> Result<(), NoteError> {
        if self.slots.get(slot).is_some() {
            return Ok(());
        }
        let parsed = Self::parse_default_date(value, label)?;
        self.slots.set(slot, parsed);
        Ok(())
    }

    fn fill_default_metadata_value(
        &mut self,
        key: &str,
        value: &str,
    ) -> Result<(), NoteError> {
        if self.metadata.get(key).is_some() {
            return Ok(());
        }
        let parsed = Self::parse_default_date(value, key)?;
        let key = TaskFieldKey::try_new(key)?;
        self.metadata.insert(key, FieldValue::Date(parsed.as_i64()));
        Ok(())
    }
}
