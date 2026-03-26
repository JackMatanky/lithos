//! Note aggregate root and identity types.
//!
//! This module defines the core domain entities for the Note context.
//! It handles the transition from unvalidated extraction artifacts
//! ([`RawNote`][crate::note::raw::RawNote]) to validated, normalized domain
//! facts ([`Note`]).
//!
//! The [`Note`] struct serves as the aggregate root, consolidating all
//! metadata, structure, and content facts for a single markdown file in
//! the vault.

use std::{fmt, sync::Arc, time::SystemTime};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};
use uuid::Uuid;

use crate::note::{
    error::NoteError,
    frontmatter::Frontmatter,
    heading::Heading,
    inline_fields::InlineField,
    link::{FrontmatterLink, Link, ReferenceLink},
    list::{ListItem, TaskExt as _},
    paths::NotePath,
    raw::{
        RawBlockRef, RawHeading, RawInlineField, RawLink, RawList, RawListItem,
        RawNote, RawReferenceLink, RawSection, RawTag,
    },
    structure::{BlockRef, Section},
    tag::Tag,
    task::{Task, TaskRef},
    value::FieldValue,
};

/// Stable identifier for a note.
///
/// `NoteId` uses UUID v7 by default to provide time-ordered,
/// collision-resistant identifiers that are efficient for database indexing.
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::aggregate::NoteId;
    /// let id = NoteId::new();
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Parses a note identifier from a string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::note::aggregate::NoteId;
    /// let id_str = "018e5462-8e31-7000-8000-000000000000";
    /// let id = NoteId::parse(id_str).unwrap();
    /// ```
    ///
    /// # Errors
    ///
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

/// Normalized note facts derived from raw extraction output.
///
/// `Note` is the primary domain entity representing a fully processed markdown
/// note. It contains all extracted metadata, structure, and content facts
/// in a validated and normalized domain format.
///
/// This struct is optimized for storage density using `Box<[T]>` for immutable
/// collections and supports zero-copy deserialization via `rkyv`.
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
    frontmatter_links: Box<[FrontmatterLink]>,
    reference_links: Box<[ReferenceLink]>,
    tags: Box<[Tag]>,
    headings: Box<[Heading]>,
    sections: Box<[Section]>,
    links: Box<[Link]>,
    block_refs: Box<[BlockRef]>,
    list_items: Box<[ListItem]>,
    tasks: Box<[Task]>,
    inline_fields: Box<[InlineField]>,
}

impl Note {
    /// Construct normalized facts from ingestion output.
    ///
    /// This method is primarily used by the raw note conversion
    /// to build a `Note` from a [`RawNote`].
    #[expect(
        clippy::too_many_arguments,
        reason = "Note aggregates all note facts in one struct"
    )]
    pub(crate) fn from_parts<
        FLinks,
        RLinks,
        Tags,
        Headings,
        Sections,
        Links,
        BlockRefs,
        ListItems,
        Tasks,
        IFields,
    >(
        id: NoteId,
        path: NotePath,
        source_hash: Box<str>,
        source_bytes: u64,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
        frontmatter: Option<Frontmatter>,
        frontmatter_links: FLinks,
        reference_links: RLinks,
        tags: Tags,
        headings: Headings,
        sections: Sections,
        links: Links,
        block_refs: BlockRefs,
        list_items: ListItems,
        tasks: Tasks,
        inline_fields: IFields,
    ) -> Self
    where
        FLinks: Into<Box<[FrontmatterLink]>>,
        RLinks: Into<Box<[ReferenceLink]>>,
        Tags: Into<Box<[Tag]>>,
        Headings: Into<Box<[Heading]>>,
        Sections: Into<Box<[Section]>>,
        Links: Into<Box<[Link]>>,
        BlockRefs: Into<Box<[BlockRef]>>,
        ListItems: Into<Box<[ListItem]>>,
        Tasks: Into<Box<[Task]>>,
        IFields: Into<Box<[InlineField]>>,
    {
        Self {
            id,
            path,
            source_hash,
            source_bytes,
            created_at,
            modified_at,
            frontmatter,
            frontmatter_links: frontmatter_links.into(),
            reference_links: reference_links.into(),
            tags: tags.into(),
            headings: headings.into(),
            sections: sections.into(),
            links: links.into(),
            block_refs: block_refs.into(),
            list_items: list_items.into(),
            tasks: tasks.into(),
            inline_fields: inline_fields.into(),
        }
    }

    /// Creates a minimal `Note` shell with the given ID and path.
    ///
    /// This is useful for representing a note before its content has been
    /// fully ingested or when performing lightweight operations.
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
            frontmatter_links: Box::new([]),
            reference_links: Box::new([]),
            tags: Box::new([]),
            headings: Box::new([]),
            sections: Box::new([]),
            links: Box::new([]),
            block_refs: Box::new([]),
            list_items: Box::new([]),
            tasks: Box::new([]),
            inline_fields: Box::new([]),
        }
    }

    /// Returns the stable identifier for this note.
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

    /// Returns the vault-relative path of the note.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &NotePath {
        &self.path
    }

    /// Returns the BLAKE3 hash of the note's source content.
    #[inline]
    #[must_use]
    pub fn source_hash(&self) -> &str {
        &self.source_hash
    }

    /// Returns the size of the note's source content in bytes.
    #[inline]
    #[must_use]
    pub const fn source_bytes(&self) -> u64 {
        self.source_bytes
    }

    /// Returns the filesystem creation time of the note, if available.
    #[inline]
    #[must_use]
    pub const fn created_at(&self) -> Option<SystemTime> {
        self.created_at
    }

    /// Returns the filesystem last modification time of the note, if available.
    #[inline]
    #[must_use]
    pub const fn modified_at(&self) -> Option<SystemTime> {
        self.modified_at
    }

    /// Returns the parsed frontmatter of the note, if present.
    #[inline]
    #[must_use]
    pub fn frontmatter(&self) -> Option<&Frontmatter> {
        self.frontmatter.as_ref()
    }

    /// Returns the collection of links extracted from the note's frontmatter.
    #[inline]
    #[must_use]
    pub fn frontmatter_links(&self) -> &[FrontmatterLink] {
        &self.frontmatter_links
    }

    /// Returns the collection of reference-style link definitions.
    #[inline]
    #[must_use]
    pub fn reference_links(&self) -> &[ReferenceLink] {
        &self.reference_links
    }

    /// Returns the collection of tags extracted from the note.
    #[inline]
    #[must_use]
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    /// Returns the collection of headings found in the note.
    #[inline]
    #[must_use]
    pub fn headings(&self) -> &[Heading] {
        &self.headings
    }

    /// Returns the collection of structural sections (paragraphs, lists, etc.).
    #[inline]
    #[must_use]
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    /// Returns the collection of inline links (Markdown and `WikiLinks`).
    #[inline]
    #[must_use]
    pub fn links(&self) -> &[Link] {
        &self.links
    }

    /// Returns the collection of block reference identifiers.
    #[inline]
    #[must_use]
    pub fn block_refs(&self) -> &[BlockRef] {
        &self.block_refs
    }

    /// Returns the collection of list items and their hierarchical
    /// relationships.
    #[inline]
    #[must_use]
    pub fn list_items(&self) -> &[ListItem] {
        &self.list_items
    }

    /// Returns the collection of tasks extracted from list items.
    #[inline]
    #[must_use]
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Returns the collection of inline metadata fields (`key:: value`).
    #[inline]
    #[must_use]
    pub fn inline_fields(&self) -> &[InlineField] {
        &self.inline_fields
    }

    pub(crate) fn add_tag(tags: &mut Vec<Tag>, tag: Tag) {
        if !tags.iter().any(|t| t.full_path() == tag.full_path()) {
            tags.push(tag);
        }
    }

    pub(crate) fn collect_frontmatter_links(
        frontmatter: &Frontmatter,
        links: &mut Vec<FrontmatterLink>,
    ) {
        for (key, value) in frontmatter.list_fields() {
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

    fn collect_tags_from_raw(
        raw_tags: &[RawTag<'_>],
        list_items: &[ListItem],
        tasks: &[Task],
        frontmatter: Option<&Frontmatter>,
    ) -> Vec<Tag> {
        let mut tags = Vec::new();
        for raw_tag in raw_tags {
            if let Ok(tag) =
                Tag::try_new_with_range(raw_tag.value.as_ref(), raw_tag.range)
            {
                Note::add_tag(&mut tags, tag);
            }
        }
        for item in list_items {
            for tag in item.tags() {
                Note::add_tag(&mut tags, tag.clone());
            }
        }
        for task in tasks {
            for tag in task.tags() {
                Note::add_tag(&mut tags, tag.clone());
            }
        }
        if let Some(frontmatter) = frontmatter
            && let Some(fm_tags) = frontmatter.tags()
        {
            for tag in fm_tags {
                Note::add_tag(&mut tags, tag.clone());
            }
        }
        tags
    }

    fn collect_list_items_from(
        raw_list_items: Vec<RawListItem<'_>>,
    ) -> Result<Vec<ListItem>, NoteError> {
        let mut items = Vec::with_capacity(raw_list_items.len());
        for raw in raw_list_items {
            items.push(ListItem::try_from(&raw)?);
        }
        items.sort_by_key(ListItem::position);
        Ok(items)
    }

    fn collect_tasks_from(
        list_items: &mut [ListItem],
        lists: &[RawList],
    ) -> Result<Vec<Task>, NoteError> {
        let mut spec_by_position =
            std::collections::HashMap::with_capacity(list_items.len());
        for list in lists {
            for position in &list.item_positions {
                spec_by_position.insert(*position, Arc::clone(&list.task_spec));
            }
        }

        let mut tasks = Vec::new();
        for item in list_items.iter_mut() {
            if !item.is_checkbox() {
                continue;
            }

            let spec = spec_by_position
                .get(&item.position())
                .or_else(|| spec_by_position.values().next());

            if let Some(spec) = spec
                && Self::should_promote_task(spec, item.tags())
            {
                let task = Task::promote(item, spec.as_ref())?;
                item.set_task_ref(TaskRef::new(task.range()));
                tasks.push(task);
            }
        }

        tasks.sort_by_key(|task| task.range().start());
        Ok(tasks)
    }

    fn should_promote_task(
        spec: &crate::config::task::TaskConfigSpec,
        tags: &[Tag],
    ) -> bool {
        if !spec.enabled {
            return false;
        }
        if spec.promotion_tags.is_empty() {
            return true;
        }
        spec.promotion_tags.iter().any(|config_tag| {
            let config_tag = config_tag.strip_prefix('#').unwrap_or(config_tag);
            tags.iter().any(|tag| {
                let raw = tag.full_path();
                raw == config_tag
            })
        })
    }

    fn collect_inline_fields_from(
        raw_inline_fields: Vec<RawInlineField<'_>>,
        list_items: &[ListItem],
    ) -> Vec<InlineField> {
        let mut inline_fields = raw_inline_fields
            .into_iter()
            .map(|raw| InlineField::from_raw(&raw))
            .collect::<Vec<_>>();

        for item in list_items {
            inline_fields.extend(item.fields().iter().cloned());
        }

        inline_fields
    }

    fn collect_frontmatter_links_from(
        frontmatter: Option<&Frontmatter>,
    ) -> Vec<FrontmatterLink> {
        let mut frontmatter_links = Vec::new();
        if let Some(frontmatter) = frontmatter {
            Note::collect_frontmatter_links(
                frontmatter,
                &mut frontmatter_links,
            );
        }
        frontmatter_links
    }

    fn collect_reference_links_from(
        reference_links: Vec<RawReferenceLink<'_>>,
    ) -> Result<Vec<ReferenceLink>, NoteError> {
        reference_links.into_iter().map(ReferenceLink::try_from).collect()
    }

    fn collect_headings_from(
        headings: Vec<RawHeading<'_>>,
    ) -> Result<Vec<Heading>, NoteError> {
        headings.into_iter().map(|raw| Heading::try_from(&raw)).collect()
    }

    fn collect_sections_from(
        sections: Vec<RawSection>,
    ) -> Result<Vec<Section>, NoteError> {
        sections.into_iter().map(|raw| Section::try_from(&raw)).collect()
    }

    fn collect_links_from(
        links: Vec<RawLink<'_>>,
    ) -> Result<Vec<Link>, NoteError> {
        links.into_iter().map(Link::try_from).collect()
    }

    fn collect_block_refs_from(
        block_refs: Vec<RawBlockRef<'_>>,
    ) -> Result<Vec<BlockRef>, NoteError> {
        block_refs.into_iter().map(|raw| BlockRef::try_from(&raw)).collect()
    }
}

impl<'source> TryFrom<(RawNote<'source>, NoteId)> for Note {
    type Error = NoteError;

    #[inline]
    fn try_from(
        (raw, id): (RawNote<'source>, NoteId),
    ) -> Result<Self, Self::Error> {
        let RawNote {
            path,
            source_hash,
            source_bytes,
            created_at,
            modified_at,
            frontmatter,
            headings,
            sections,
            links,
            tags: raw_tags,
            lists,
            list_items,
            inline_fields: raw_inline_fields,
            reference_links,
            block_refs,
            ..
        } = raw;

        let frontmatter = frontmatter.map(Frontmatter::try_from).transpose()?;
        let mut list_items = Note::collect_list_items_from(list_items)?;
        let tasks = Note::collect_tasks_from(&mut list_items, &lists)?;
        let inline_fields =
            Note::collect_inline_fields_from(raw_inline_fields, &list_items);
        let tags = Note::collect_tags_from_raw(
            &raw_tags,
            &list_items,
            &tasks,
            frontmatter.as_ref(),
        );
        let frontmatter_links =
            Note::collect_frontmatter_links_from(frontmatter.as_ref());
        let reference_links =
            Note::collect_reference_links_from(reference_links)?;
        let headings = Note::collect_headings_from(headings)?;
        let sections = Note::collect_sections_from(sections)?;
        let links = Note::collect_links_from(links)?;
        let block_refs = Note::collect_block_refs_from(block_refs)?;

        Ok(Self::from_parts(
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
        ))
    }
}

#[cfg(test)]
#[expect(
    clippy::pattern_type_mismatch,
    clippy::shadow_unrelated,
    reason = "Test code prioritizes readability"
)]
mod tests {
    use std::collections::HashMap;

    use chrono::NaiveDate;

    use super::*;
    use crate::{
        config::{
            aggregate::Config,
            raw::{RawConfig, RawFieldSpec, RawTaskConfig},
            task::TaskConfigSpec,
            vault::{VaultId, VaultRoot},
        },
        note::{
            position::{SourceByteOffset, SourceByteRange},
            raw::{RawFieldValue, RawInlineField, RawTag, RawTaskMarker},
            scanner::{NoteScanner, ScannedArtifact},
        },
    };

    #[test]
    fn promoted_task_strips_promotion_tags() {
        let config = test_config_with_task_tag();
        let task_spec = config.to_task_spec();

        let promoted = promote_task("#task Do work", &task_spec, &[]);
        assert_eq!(promoted.text(), "Do work");

        let untagged = promote_task("Do work", &task_spec, &[]);
        assert_eq!(untagged.text(), "Do work");
    }

    #[test]
    fn promoted_checkbox_extracts_text_and_metadata() {
        let config = config_with_fields();
        let task_spec = config.to_task_spec();
        let task = promote_task(
            "#task Review PR [priority:: 2] [project:: lithos]",
            &task_spec,
            &[],
        );

        assert_eq!(task.text(), "Review PR");

        let priority_field = task
            .fields()
            .iter()
            .find(|(k, _)| k.as_str() == "priority")
            .map(|(_, v)| v);
        assert_eq!(
            priority_field.and_then(super::super::value::FieldValue::as_number),
            Some(2.0f64)
        );

        let project_field = task
            .fields()
            .iter()
            .find(|(k, _)| k.as_str() == "project")
            .map(|(_, v)| v);
        assert_eq!(project_field.and_then(|v| v.as_str()), Some("lithos"));
    }

    #[test]
    fn promoted_checkbox_collects_hierarchical_tags() {
        let config = test_config_with_task_tag();
        let task_spec = config.to_task_spec();
        let task = promote_task(
            "#task Fix #work/project/urgent issue",
            &task_spec,
            &[],
        );

        assert!(task.tags().any(|tag| tag.full_path() == "task"));
        assert!(
            task.tags().any(|tag| tag.full_path() == "work/project/urgent")
        );
        assert_eq!(task.tags().count(), 2);
    }

    #[test]
    fn promoted_checkbox_ignores_invalid_tags() {
        let config = test_config_with_task_tag();
        let task_spec = config.to_task_spec();
        let task = promote_task("#task Review #bad/ tags", &task_spec, &[]);

        assert!(task.tags().any(|tag| tag.full_path() == "task"));
        assert_eq!(task.tags().count(), 1);
    }

    #[test]
    fn promoted_checkbox_parses_dates() {
        let config = test_config_with_task_tag();
        let task_spec = config.to_task_spec();
        let task = promote_task(
            "#task Test task with dates [created:: 2024-01-01] [due:: \
             2024-12-31]",
            &task_spec,
            &[],
        );

        let created_date =
            NaiveDate::from_ymd_opt(2024, 1, 1).expect("created date");
        let due_date = NaiveDate::from_ymd_opt(2024, 12, 31).expect("due date");

        if let Some(created_at) = task.dates().created() {
            assert_eq!(created_at.as_naive_date(), Some(created_date));
        }

        if let Some(due_at) = task.dates().due() {
            assert_eq!(due_at.as_naive_date(), Some(due_date));
        }

        if let (Some(created_at), Some(due_at)) =
            (task.dates().created(), task.dates().due())
        {
            let created_date = date_of(created_at);
            let due_date = date_of(due_at);
            assert!(created_date < due_date);
        }
    }

    #[test]
    fn promoted_checkbox_parses_paren_inline_fields() {
        let config = config_with_fields();
        let task_spec = config.to_task_spec();
        let task = promote_task(
            "#task Review PR (priority:: 2) (project:: lithos)",
            &task_spec,
            &[],
        );

        assert_eq!(task.text(), "Review PR");

        let priority_field = task
            .fields()
            .iter()
            .find(|(k, _)| k.as_str() == "priority")
            .map(|(_, v)| v);
        assert_eq!(
            priority_field.and_then(super::super::value::FieldValue::as_number),
            Some(2.0f64)
        );

        let project_field = task
            .fields()
            .iter()
            .find(|(k, _)| k.as_str() == "project")
            .map(|(_, v)| v);
        assert_eq!(project_field.and_then(|v| v.as_str()), Some("lithos"));
    }

    fn date_of(value: &crate::note::task::TaskDateValue) -> NaiveDate {
        value.as_naive_date().expect("date")
    }

    fn promote_task(
        promoted_text: &str,
        task_spec: &TaskConfigSpec,
        emoji_markers: &[char],
    ) -> Task {
        let item = list_item_from_text(promoted_text, emoji_markers);
        Task::promote(&item, task_spec).expect("task conversion")
    }

    fn list_item_from_text(raw_text: &str, emoji_markers: &[char]) -> ListItem {
        let scanner = NoteScanner::new(emoji_markers.to_vec());
        let start = SourceByteOffset::new(0);
        let end = SourceByteOffset::try_from(raw_text.len()).unwrap_or(start);
        let range = SourceByteRange::new(start, end).expect("valid test range");

        let artifacts = scanner
            .scan_block(raw_text, SourceByteOffset::new(0))
            .expect("scan artifacts");

        let mut tags = Vec::new();
        let mut inline_fields = Vec::new();
        let mut task_marker = None;

        for artifact in artifacts {
            match artifact {
                ScannedArtifact::Tag {
                    text: tag_text,
                    range,
                } => tags.push(RawTag::new(tag_text, range)),
                ScannedArtifact::InlineField {
                    key,
                    value,
                    range,
                } => {
                    let typed_value = RawFieldValue::from_str_with_spec(
                        value.as_ref(),
                        key.as_ref(),
                        None,
                    )
                    .into_owned();
                    inline_fields.push(RawInlineField::new(
                        key,
                        typed_value,
                        range,
                    ));
                }
                ScannedArtifact::TaskMarker {
                    marker,
                    ..
                } => {
                    task_marker = Some(RawTaskMarker::from_char(marker));
                }
                ScannedArtifact::BlockRef {
                    ..
                } => {}
            }
        }

        let mut is_checked = task_marker
            .map(|marker| matches!(marker, RawTaskMarker::Checked(_)));
        if task_marker.is_none() {
            task_marker = Some(RawTaskMarker::Unchecked(' '));
            is_checked = Some(false);
        }

        let raw = RawListItem::new(
            crate::note::raw::RawListKind::Unordered,
            crate::note::raw::RawListDepth::Root,
            raw_text.into(),
            is_checked,
            task_marker,
            range,
            range,
            None,
            tags,
            inline_fields,
        );

        ListItem::try_from(&raw).expect("valid list item")
    }

    fn test_config_with_task_tag() -> Config {
        let raw = RawConfig {
            task: Some(RawTaskConfig {
                use_emoji: Some(true),
                task_tags: Some(vec!["#task".into()]),
                dates: Some(crate::config::raw::RawTaskDates {
                    created: Some(crate::config::raw::RawDateFieldSpec {
                        keyword: String::from("created"),
                        emoji: Some('\u{2795}'),
                        format: String::from("%Y-%m-%d"),
                    }),
                    due: Some(crate::config::raw::RawDateFieldSpec {
                        keyword: String::from("due"),
                        emoji: Some('\u{1f4c5}'),
                        format: String::from("%Y-%m-%d"),
                    }),
                    completed: Some(crate::config::raw::RawDateFieldSpec {
                        keyword: String::from("completed"),
                        emoji: Some('\u{2705}'),
                        format: String::from("%Y-%m-%d"),
                    }),
                    ..Default::default()
                }),
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
