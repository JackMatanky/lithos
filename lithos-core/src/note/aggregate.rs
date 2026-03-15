//! Note aggregate root and identity types.

use std::{fmt, time::SystemTime};

use rkyv::{Archive, Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config::{aggregate::Config, task::StatusSymbol},
    note::{
        error::{NoteError, NoteMetadataError, TaskError},
        frontmatter::Frontmatter,
        heading::Heading,
        link::{Anchor, EmbedType, FrontmatterLink, Link, Target},
        list::ListItemEntry,
        paths::NotePath,
        raw::{note::RawNote, tasks::RawTask},
        structure::{BlockRef, Section},
        tag::Tag,
        task::{
            Task, TaskAttributes, TaskAttributesBuilder, TaskFieldKey,
            TaskMetadata, TaskTimestamp,
        },
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
pub struct NoteFacts {
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
}

impl NoteFacts {
    /// Construct normalized facts from ingestion output.
    #[expect(
        clippy::too_many_arguments,
        reason = "NoteFacts aggregates all note facts in one struct"
    )]
    pub(crate) fn new(
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

/// Conversion context for building NoteFacts from RawNote + Config.
pub(crate) struct RawNoteContext<'a> {
    raw: &'a RawNote,
    config: &'a Config,
    id: NoteId,
}

impl<'a> RawNoteContext<'a> {
    #[inline]
    #[must_use]
    pub(crate) const fn new(
        id: NoteId,
        raw: &'a RawNote,
        config: &'a Config,
    ) -> Self {
        Self {
            raw,
            config,
            id,
        }
    }
}

impl<'a> TryFrom<RawNoteContext<'a>> for NoteFacts {
    type Error = NoteError;

    fn try_from(ctx: RawNoteContext<'a>) -> Result<Self, Self::Error> {
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

        Ok(Self::new(
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

struct RawTaskContext<'a> {
    raw: &'a RawTask,
    config: &'a Config,
}

impl<'a> RawTaskContext<'a> {
    #[inline]
    const fn new(raw: &'a RawTask, config: &'a Config) -> Self {
        Self {
            raw,
            config,
        }
    }
}

impl<'a> TryFrom<RawTaskContext<'a>> for Option<Task> {
    type Error = NoteError;

    fn try_from(ctx: RawTaskContext<'a>) -> Result<Self, Self::Error> {
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
        inline_fields: &[(Box<str>, Box<str>)],
        emoji_dates: &[(Box<str>, Box<str>)],
    ) -> Result<ParsedInlineFields, NoteError> {
        let mut state = InlineFieldState::new();

        for (keyword, raw_value) in inline_fields {
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
        tokens: &[(Box<str>, Box<str>)],
    ) -> Result<(), NoteError> {
        for (emoji, value) in tokens {
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
        tokens: &[(Box<str>, Box<str>)],
    ) -> Result<(), NoteError> {
        for (emoji, value) in tokens {
            if Self::emoji_matches(emoji, '\u{2795}') {
                self.fill_default_slot_value(
                    DateSlot::Created,
                    "created",
                    value,
                )?;
            } else if Self::emoji_matches(emoji, '\u{1f4c5}') {
                self.fill_default_slot_value(DateSlot::Due, "due", value)?;
            } else if Self::emoji_matches(emoji, '\u{2705}') {
                self.fill_default_slot_value(
                    DateSlot::Completed,
                    "completed",
                    value,
                )?;
            } else if Self::emoji_matches(emoji, '\u{23f3}') {
                self.fill_default_metadata_value("scheduled", value)?;
            } else if Self::emoji_matches(emoji, '\u{1f6eb}') {
                self.fill_default_metadata_value("start", value)?;
            } else if Self::emoji_matches(emoji, '\u{274c}') {
                self.fill_default_metadata_value("cancelled", value)?;
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
        let timestamp = spec.parse_str(raw_value).map_err(|_error| {
            TaskError::InvalidTimestamp {
                raw: raw_value.into(),
                reason: "failed parsing",
            }
        })?;
        Ok(TaskTimestamp::new(timestamp))
    }

    fn parse_default_date(
        raw_value: &str,
        field: &str,
    ) -> Result<TaskTimestamp, NoteError> {
        let parsed = raw_value.parse::<i64>().map_err(|_error| {
            NoteError::Task(TaskError::InvalidMetadataField {
                keyword: field.into(),
                reason: "failed parsing",
            })
        })?;
        Ok(TaskTimestamp::new(parsed))
    }

    fn match_date_spec_by_emoji<'config>(
        config: &'config crate::config::task::Task,
        emoji: &str,
    ) -> Option<(DateSlot, &'config crate::config::value::DateSpec)> {
        if let Some(spec) = config.created()
            && Self::emoji_matches(spec.emoji(), emoji)
        {
            return Some((DateSlot::Created, spec));
        }
        if let Some(spec) = config.due()
            && Self::emoji_matches(spec.emoji(), emoji)
        {
            return Some((DateSlot::Due, spec));
        }
        if let Some(spec) = config.reminder()
            && Self::emoji_matches(spec.emoji(), emoji)
        {
            return Some((DateSlot::Reminder, spec));
        }
        if let Some(spec) = config.completed()
            && Self::emoji_matches(spec.emoji(), emoji)
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
