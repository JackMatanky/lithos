//! Note command implementations (CQRS write operations).
//!
//! This module implements the Command port trait for Note write operations,
//! using the Database layer for persistence.

use std::{path::Path, time::SystemTime};

use uuid::Uuid;

use crate::{
    config::aggregate::Config,
    db::{BatchWriter, Database, DbError},
    note::{
        adapter::{
            reader::ParsedNote,
            stored::{
                StoredLocationRange, StoredNote, StoredTask,
                metadata_index_keys,
            },
        },
        aggregate::NoteId,
        db_table::{
            ALIAS_TO_ID, FILE_CLASS_TO_ID, FOLDER_TO_ID, FRONTMATTER_KV,
            NOTE_EVENTS, PATH_TO_ID, STORED_NOTES, TAGS_TO_NOTES, TASKS,
            TASKS_BY_COMPLETED_DATE, TASKS_BY_CREATED_DATE,
            TASKS_BY_DEPENDS_ON, TASKS_BY_DUE_DATE, TASKS_BY_METADATA,
            TASKS_BY_REMINDER_DATE, TASKS_BY_STATUS,
        },
        error::NoteError,
        events::{
            NoteChangeKind, NoteEvent, NoteEventKind, NoteEventPayload,
            NoteEventPayloadV1,
        },
        frontmatter::Frontmatter,
        paths::NotePath,
        ports::Command,
        position::SourceByteOffset,
        structure::Heading,
        task::{TaskId, TaskMetadata, TaskText},
        value::FieldValue,
    },
};

/// Index data extracted from a note for cleanup operations.
struct IndexData {
    path: Box<str>,
    folder: Option<Box<str>>,
    tags: Vec<Box<str>>,
    aliases: Vec<Box<str>>,
    file_class: Option<Box<str>>,
    task_indexes: TaskIndexData,
    frontmatter_entries: Vec<Box<str>>,
}

#[derive(Debug, Default)]
struct TaskIndexData {
    tasks: Vec<TaskIndexEntry>,
    dependencies_enabled: bool,
}

#[derive(Debug, Clone)]
struct TaskIndexEntry {
    stored: StoredTask,
    metadata_keys: Vec<Box<str>>,
}

/// Command implementation for Note write operations.
///
/// Persists notes and maintains secondary indexes (paths, tags, tasks, and
/// frontmatter keys) so query adapters can execute fast lookups.
///
/// # Examples
///
/// ```no_run
/// use lithos_core::{
///     config::{
///         aggregate::Config,
///         raw::RawConfig,
///         vault::{VaultId, VaultRoot},
///     },
///     db::Database,
///     note::adapter::command::CommandAdapter,
/// };
///
/// let root = std::env::temp_dir()
///     .join(format!("lithos_cmd_doc_{}", std::process::id()));
/// std::fs::create_dir_all(&root)?;
/// let config = Config::build(
///     &RawConfig::default(),
///     VaultId::new(),
///     VaultRoot::try_new(root.clone())?,
///     lithos_core::config::aggregate::Version::initial(),
/// )?;
/// let db_path = root.join("notes.redb");
/// let db = Database::open(&db_path)?;
/// let adapter = CommandAdapter::new(&db, &config);
/// # let _ = adapter;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct CommandAdapter<'db, 'config> {
    db: &'db Database,
    config: &'config Config,
}

impl<'db, 'config> CommandAdapter<'db, 'config> {
    /// Create a new `CommandAdapter` with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database, config: &'config Config) -> Self {
        Self {
            db,
            config,
        }
    }

    fn ensure_unique_path(
        &self,
        path: &NotePath,
        current_id: Option<&str>,
    ) -> Result<(), DbError> {
        let ids = self.db.multimap_get(PATH_TO_ID, path.as_str())?;
        if ids.iter().any(|id| Some(id.as_str()) != current_id) {
            return Err(DbError::Table(
                NoteError::AlreadyExists(path.clone()).to_string(),
            ));
        }
        Ok(())
    }

    fn collect_index_data_from_parsed(
        &self,
        note_id: NoteId,
        path: &NotePath,
        parsed: &ParsedNote,
    ) -> Result<IndexData, DbError> {
        let frontmatter = parsed.frontmatter();
        let aliases = frontmatter
            .map(|fm| fm.aliases(self.config).map(Into::into).collect())
            .unwrap_or_default();
        let file_class = frontmatter
            .and_then(|fm| fm.file_class(self.config))
            .map(Into::into);
        let frontmatter_entries =
            frontmatter.map(Self::frontmatter_entries).unwrap_or_default();

        Ok(IndexData {
            path: path.as_str().into(),
            folder: Self::note_folder(path.as_str()),
            tags: parsed
                .tags()
                .iter()
                .map(|tag| tag.full_path().into())
                .collect(),
            aliases,
            file_class,
            task_indexes: Self::task_indexes_from_parsed(
                note_id,
                path,
                parsed,
                self.config,
            )?,
            frontmatter_entries,
        })
    }

    fn collect_index_data_from_stored(
        &self,
        stored: &StoredNote,
        task_indexes: TaskIndexData,
    ) -> IndexData {
        let frontmatter = stored.frontmatter();
        let aliases = frontmatter
            .map(|fm| fm.aliases(self.config).map(Into::into).collect())
            .unwrap_or_default();
        let file_class = frontmatter
            .and_then(|fm| fm.file_class(self.config))
            .map(Into::into);
        let frontmatter_entries =
            frontmatter.map(Self::frontmatter_entries).unwrap_or_default();

        IndexData {
            path: stored.path().as_str().into(),
            folder: Self::note_folder(stored.path().as_str()),
            tags: stored
                .tags()
                .iter()
                .map(|tag| tag.full_path().into())
                .collect(),
            aliases,
            file_class,
            task_indexes,
            frontmatter_entries,
        }
    }

    fn build_stored_note_from_parsed(
        &self,
        note_id: NoteId,
        path: &NotePath,
        parsed: &ParsedNote,
    ) -> Result<StoredNote, DbError> {
        let frontmatter = parsed.frontmatter().cloned();
        let title = frontmatter
            .as_ref()
            .and_then(|fm| fm.title(self.config))
            .map(Into::into);
        let heading_locations = parsed
            .headings()
            .iter()
            .map(|heading| parsed.location_for_offset(heading.position()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error: NoteError| DbError::Table(error.to_string()))?;
        let section_locations = parsed
            .sections()
            .iter()
            .map(|section| {
                let range = section.range();
                let start = parsed.location_for_offset(range.start())?;
                let end = parsed.location_for_offset(range.end())?;
                Ok(StoredLocationRange::new(start, end))
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error: NoteError| DbError::Table(error.to_string()))?;
        let source_bytes =
            u64::try_from(parsed.source().len()).map_err(|error| {
                DbError::Table(format!("source length out of range: {error}"))
            })?;
        let source_hash =
            blake3::hash(parsed.source().as_bytes()).to_hex().to_string();

        Ok(StoredNote::new(
            note_id,
            path.clone(),
            title,
            frontmatter,
            parsed.tags().to_vec(),
            parsed.headings().to_vec(),
            Some(heading_locations),
            parsed.sections().to_vec(),
            Some(section_locations),
            parsed.links().to_vec(),
            source_hash.into_boxed_str(),
            source_bytes,
            parsed.created_at(),
            parsed.modified_at(),
            SystemTime::now(),
        ))
    }

    fn find_note_id_by_path(
        &self,
        path: &NotePath,
    ) -> Result<Option<NoteId>, DbError> {
        let ids = self.db.multimap_get(PATH_TO_ID, path.as_str())?;
        if let Some(id_str) = ids.first() {
            let uuid = Uuid::parse_str(id_str)
                .map_err(|error| DbError::Deserialization(error.to_string()))?;
            Ok(Some(NoteId::from(uuid)))
        } else {
            Ok(None)
        }
    }

    fn load_task_indexes_for_note(
        &self,
        note_id: NoteId,
    ) -> Result<TaskIndexData, DbError> {
        let stored_tasks = self.db.list_owned::<StoredTask>(TASKS)?;
        let index_all_fields = self.config.task().indexed().is_empty();
        let dependencies_enabled = self.config.task().dependencies_enabled();

        let mut tasks = Vec::new();
        for stored in stored_tasks {
            if stored.note_id() != note_id {
                continue;
            }

            let metadata_keys = Self::task_metadata_keys(
                stored.metadata(),
                self.config,
                index_all_fields,
            );
            tasks.push(TaskIndexEntry {
                stored,
                metadata_keys,
            });
        }

        Ok(TaskIndexData {
            tasks,
            dependencies_enabled,
        })
    }

    fn build_note_event_from_parsed(
        note_id: NoteId,
        path: &NotePath,
        parsed: &ParsedNote,
        change: NoteChangeKind,
    ) -> Result<NoteEvent, DbError> {
        let task_count =
            u32::try_from(parsed.tasks().len()).map_err(|_error| {
                DbError::Table("task count out of range".into())
            })?;
        let tag_count =
            u32::try_from(parsed.tags().len()).map_err(|_error| {
                DbError::Table("tag count out of range".into())
            })?;
        let source_hash =
            blake3::hash(parsed.source().as_bytes()).to_hex().to_string();
        let source_bytes =
            u64::try_from(parsed.source().len()).map_err(|error| {
                DbError::Table(format!("source length out of range: {error}"))
            })?;

        let payload = NoteEventPayload::V1(NoteEventPayloadV1::indexed(
            change,
            task_count,
            tag_count,
            Some(source_hash.into_boxed_str()),
            Some(source_bytes),
            parsed.modified_at(),
        ));

        Ok(NoteEvent::new(
            Uuid::now_v7(),
            note_id,
            path.clone(),
            NoteEventKind::Indexed,
            SystemTime::now(),
            payload,
        ))
    }

    fn build_note_event_from_stored(
        stored: &StoredNote,
        task_count: usize,
        change: NoteChangeKind,
    ) -> Result<NoteEvent, DbError> {
        let task_count = u32::try_from(task_count).map_err(|_error| {
            DbError::Table("task count out of range".into())
        })?;
        let tag_count =
            u32::try_from(stored.tags().len()).map_err(|_error| {
                DbError::Table("tag count out of range".into())
            })?;

        let payload = NoteEventPayload::V1(NoteEventPayloadV1::changed(
            change,
            task_count,
            tag_count,
            Some(stored.source_hash().into()),
            Some(stored.source_bytes()),
            stored.modified_at(),
        ));

        Ok(NoteEvent::new(
            Uuid::now_v7(),
            stored.id(),
            stored.path().clone(),
            NoteEventKind::Changed,
            SystemTime::now(),
            payload,
        ))
    }

    fn insert_note_event(
        batch: &mut BatchWriter,
        event: &NoteEvent,
    ) -> Result<(), DbError> {
        let mut id_buffer = Uuid::encode_buffer();
        let id_str = event.id().as_hyphenated().encode_lower(&mut id_buffer);
        batch.put(NOTE_EVENTS, id_str, event)
    }

    /// Upsert a parsed note projection with filesystem timestamps.
    ///
    /// This adapter-specific entry point is intended for ingestion pipelines
    /// that parse markdown files via `NoteReader` and want to persist
    /// projections without constructing the legacy Note aggregate.
    ///
    /// # Errors
    /// Returns `DbError` if persistence fails.
    #[inline]
    pub fn upsert_parsed_note(
        &self,
        path: &NotePath,
        parsed: &ParsedNote,
    ) -> Result<NoteId, DbError> {
        let existing_id = self.find_note_id_by_path(path)?;
        let note_id = existing_id.unwrap_or_else(NoteId::new);
        let id = Uuid::from(note_id);
        let mut id_buffer = Uuid::encode_buffer();
        let id_str = id.as_hyphenated().encode_lower(&mut id_buffer);
        let id_str: &str = id_str;

        let stored_note =
            self.build_stored_note_from_parsed(note_id, path, parsed)?;
        let index_data =
            self.collect_index_data_from_parsed(note_id, path, parsed)?;
        let change_kind = if existing_id.is_some() {
            NoteChangeKind::Updated
        } else {
            NoteChangeKind::Created
        };
        let event = Self::build_note_event_from_parsed(
            note_id,
            path,
            parsed,
            change_kind,
        )?;

        let old_index_data = if existing_id.is_some() {
            let stored =
                self.db.get_owned::<StoredNote>(STORED_NOTES, id_str)?;
            if let Some(stored) = stored {
                if stored.path() != path {
                    self.ensure_unique_path(path, Some(id_str))?;
                }
                let tasks = self.load_task_indexes_for_note(note_id)?;
                Some(self.collect_index_data_from_stored(&stored, tasks))
            } else {
                None
            }
        } else {
            self.ensure_unique_path(path, None)?;
            None
        };

        self.db.batch_write(|batch| {
            if let Some(old_index_data) = old_index_data {
                Self::remove_indexes(batch, &old_index_data, id_str)?;
            }

            Self::insert_indexes(batch, &index_data, id_str)?;
            batch.put(STORED_NOTES, id_str, &stored_note)?;
            Self::insert_note_event(batch, &event)?;
            Ok(())
        })?;

        Ok(note_id)
    }

    /// Lookup a stored note projection by vault-relative path.
    ///
    /// # Errors
    /// Returns `DbError` if the lookup fails.
    #[inline]
    pub fn stored_note_by_path(
        &self,
        path: &NotePath,
    ) -> Result<Option<StoredNote>, DbError> {
        let ids = self.db.multimap_get(PATH_TO_ID, path.as_str())?;
        if let Some(id_str) = ids.first() {
            self.db.get_owned::<StoredNote>(STORED_NOTES, id_str)
        } else {
            Ok(None)
        }
    }

    /// List all stored note projections.
    ///
    /// # Errors
    /// Returns `DbError` if the scan fails.
    #[inline]
    pub fn list_stored_notes(&self) -> Result<Vec<StoredNote>, DbError> {
        self.db.list_owned::<StoredNote>(STORED_NOTES)
    }

    fn insert_indexes(
        batch: &mut BatchWriter,
        index_data: &IndexData,
        id_str: &str,
    ) -> Result<(), DbError> {
        batch.multimap_insert(PATH_TO_ID, index_data.path.as_ref(), id_str)?;

        if let Some(folder) = index_data.folder.as_deref() {
            batch.multimap_insert(FOLDER_TO_ID, folder, id_str)?;
        }

        for tag in &index_data.tags {
            batch.multimap_insert(TAGS_TO_NOTES, tag.as_ref(), id_str)?;
        }

        for alias in &index_data.aliases {
            batch.multimap_insert(ALIAS_TO_ID, alias.as_ref(), id_str)?;
        }

        if let Some(file_class) = index_data.file_class.as_deref() {
            batch.multimap_insert(FILE_CLASS_TO_ID, file_class, id_str)?;
        }

        for entry in &index_data.frontmatter_entries {
            batch.multimap_insert(FRONTMATTER_KV, entry.as_ref(), id_str)?;
        }

        Self::insert_task_indexes(batch, &index_data.task_indexes)
    }

    fn remove_indexes(
        batch: &mut BatchWriter,
        index_data: &IndexData,
        id_str: &str,
    ) -> Result<(), DbError> {
        batch.multimap_remove(PATH_TO_ID, index_data.path.as_ref(), id_str)?;

        if let Some(folder) = index_data.folder.as_deref() {
            batch.multimap_remove(FOLDER_TO_ID, folder, id_str)?;
        }

        for tag in &index_data.tags {
            batch.multimap_remove(TAGS_TO_NOTES, tag.as_ref(), id_str)?;
        }

        for alias in &index_data.aliases {
            batch.multimap_remove(ALIAS_TO_ID, alias.as_ref(), id_str)?;
        }

        if let Some(file_class) = index_data.file_class.as_deref() {
            batch.multimap_remove(FILE_CLASS_TO_ID, file_class, id_str)?;
        }

        for entry in &index_data.frontmatter_entries {
            batch.multimap_remove(FRONTMATTER_KV, entry.as_ref(), id_str)?;
        }

        Self::remove_task_indexes(batch, &index_data.task_indexes)
    }

    fn task_indexes_from_parsed(
        note_id: NoteId,
        path: &NotePath,
        parsed: &ParsedNote,
        config: &Config,
    ) -> Result<TaskIndexData, DbError> {
        let tasks = Self::task_index_entries_from_parsed(
            note_id, path, parsed, config,
        )?;

        Ok(TaskIndexData {
            tasks,
            dependencies_enabled: config.task().dependencies_enabled(),
        })
    }

    fn task_index_entries_from_parsed(
        note_id: NoteId,
        path: &NotePath,
        parsed: &ParsedNote,
        config: &Config,
    ) -> Result<Vec<TaskIndexEntry>, DbError> {
        let mut entries = Vec::new();
        let status_config = config.task().status();
        let index_all_fields = config.task().indexed().is_empty();
        let dependencies_enabled = config.task().dependencies_enabled();

        for task in parsed.tasks() {
            let status_symbol = status_config
                .symbol_for_name(task.status())
                .ok_or_else(|| {
                    DbError::Table(
                        "task status should have a symbol mapping".into(),
                    )
                })?;
            let heading =
                Self::task_heading_from_parsed(parsed, task.position());
            let location = parsed
                .location_for_offset(task.position())
                .map_err(|err| DbError::Table(err.to_string()))?;
            let metadata = task.metadata().clone();
            let metadata_keys =
                Self::task_metadata_keys(&metadata, config, index_all_fields);
            let depends_on =
                Self::task_depends_on(&metadata, dependencies_enabled);
            let block_id = Self::task_block_id(&metadata);
            let parent_id = Self::task_parent_id(&metadata);

            let text = TaskText::try_new(task.text())
                .map_err(|err| DbError::Table(err.to_string()))?;
            let stored = StoredTask::new(
                task.id(),
                note_id,
                path.clone(),
                heading,
                task.position(),
                Some(location),
                task.status().clone(),
                status_symbol,
                "unknown".into(),
                text,
                task.tags().cloned().collect(),
                metadata,
                task.schedule().clone(),
                parent_id,
                block_id,
                depends_on,
            );

            entries.push(TaskIndexEntry {
                stored,
                metadata_keys,
            });
        }

        Ok(entries)
    }

    fn insert_task_indexes(
        batch: &mut BatchWriter,
        index_data: &TaskIndexData,
    ) -> Result<(), DbError> {
        let mut itoa_buffer = itoa::Buffer::new();

        for entry in &index_data.tasks {
            let stored = &entry.stored;
            let task_id = Uuid::from(stored.id());
            let mut id_buffer = Uuid::encode_buffer();
            let id_str = task_id.as_hyphenated().encode_lower(&mut id_buffer);

            batch.put(TASKS, id_str, stored)?;
            batch.multimap_insert(
                TASKS_BY_STATUS,
                stored.status_name().as_str(),
                id_str,
            )?;

            if let Some(timestamp) = stored.created_at() {
                batch.multimap_insert(
                    TASKS_BY_CREATED_DATE,
                    itoa_buffer.format(timestamp.as_i64()),
                    id_str,
                )?;
            }
            if let Some(timestamp) = stored.due_at() {
                batch.multimap_insert(
                    TASKS_BY_DUE_DATE,
                    itoa_buffer.format(timestamp.as_i64()),
                    id_str,
                )?;
            }
            if let Some(timestamp) = stored.reminder_at() {
                batch.multimap_insert(
                    TASKS_BY_REMINDER_DATE,
                    itoa_buffer.format(timestamp.as_i64()),
                    id_str,
                )?;
            }
            if let Some(timestamp) = stored.completed_at() {
                batch.multimap_insert(
                    TASKS_BY_COMPLETED_DATE,
                    itoa_buffer.format(timestamp.as_i64()),
                    id_str,
                )?;
            }

            for key in &entry.metadata_keys {
                batch.multimap_insert(
                    TASKS_BY_METADATA,
                    key.as_ref(),
                    id_str,
                )?;
            }

            if index_data.dependencies_enabled {
                for depends_on in stored.depends_on() {
                    batch.multimap_insert(
                        TASKS_BY_DEPENDS_ON,
                        depends_on.as_ref(),
                        id_str,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn remove_task_indexes(
        batch: &mut BatchWriter,
        index_data: &TaskIndexData,
    ) -> Result<(), DbError> {
        let mut itoa_buffer = itoa::Buffer::new();

        for entry in &index_data.tasks {
            let stored = &entry.stored;
            let task_id = Uuid::from(stored.id());
            let mut id_buffer = Uuid::encode_buffer();
            let id_str = task_id.as_hyphenated().encode_lower(&mut id_buffer);

            batch.delete(TASKS, id_str)?;
            batch.multimap_remove(
                TASKS_BY_STATUS,
                stored.status_name().as_str(),
                id_str,
            )?;

            if let Some(timestamp) = stored.created_at() {
                batch.multimap_remove(
                    TASKS_BY_CREATED_DATE,
                    itoa_buffer.format(timestamp.as_i64()),
                    id_str,
                )?;
            }
            if let Some(timestamp) = stored.due_at() {
                batch.multimap_remove(
                    TASKS_BY_DUE_DATE,
                    itoa_buffer.format(timestamp.as_i64()),
                    id_str,
                )?;
            }
            if let Some(timestamp) = stored.reminder_at() {
                batch.multimap_remove(
                    TASKS_BY_REMINDER_DATE,
                    itoa_buffer.format(timestamp.as_i64()),
                    id_str,
                )?;
            }
            if let Some(timestamp) = stored.completed_at() {
                batch.multimap_remove(
                    TASKS_BY_COMPLETED_DATE,
                    itoa_buffer.format(timestamp.as_i64()),
                    id_str,
                )?;
            }

            for key in &entry.metadata_keys {
                batch.multimap_remove(
                    TASKS_BY_METADATA,
                    key.as_ref(),
                    id_str,
                )?;
            }

            if index_data.dependencies_enabled {
                for depends_on in stored.depends_on() {
                    batch.multimap_remove(
                        TASKS_BY_DEPENDS_ON,
                        depends_on.as_ref(),
                        id_str,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn task_metadata_keys(
        metadata: &TaskMetadata,
        config: &Config,
        index_all: bool,
    ) -> Vec<Box<str>> {
        let indexed = config.task().indexed();
        let mut keys = Vec::new();

        for (field, value) in metadata.fields() {
            let field_name = field.as_str();
            if index_all
                || indexed.iter().any(|name| name.as_ref() == field_name)
            {
                keys.extend(metadata_index_keys(field_name, value));
            }
        }

        keys
    }

    fn task_depends_on(
        metadata: &TaskMetadata,
        enabled: bool,
    ) -> Vec<Box<str>> {
        if !enabled {
            return Vec::new();
        }

        let Some(value) = metadata.get("dependsOn") else {
            return Vec::new();
        };

        if let Some(text) = value.as_str() {
            return text
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(Into::into)
                .collect();
        }

        if let Some(arr) = value.as_array() {
            return arr
                .iter()
                .filter_map(|item| item.as_str())
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(Into::into)
                .collect();
        }

        Vec::new()
    }

    fn task_block_id(metadata: &TaskMetadata) -> Option<Box<str>> {
        metadata
            .get_string("block_id")
            .or_else(|| metadata.get_string("blockId"))
            .or_else(|| metadata.get_string("block"))
            .map(Into::into)
    }

    fn task_parent_id(metadata: &TaskMetadata) -> Option<TaskId> {
        let raw = metadata
            .get_string("parent_id")
            .or_else(|| metadata.get_string("parentId"))
            .or_else(|| metadata.get_string("parent"))?;
        let parsed = Uuid::parse_str(raw).ok()?;
        Some(TaskId::from(parsed))
    }

    fn task_heading_from_parsed(
        parsed: &ParsedNote,
        position: SourceByteOffset,
    ) -> Option<Heading> {
        parsed
            .headings()
            .iter()
            .filter(|heading| heading.position() <= position)
            .max_by_key(|heading| heading.position())
            .cloned()
    }

    fn frontmatter_entries(frontmatter: &Frontmatter) -> Vec<Box<str>> {
        let mut entries = Vec::new();
        let mut fields: Vec<_> = frontmatter.fields().collect();
        fields.sort_by(|left, right| left.0.cmp(right.0));

        for (key, value) in fields {
            let values = Self::field_value_index_values(value);
            for value_str in values {
                let capacity =
                    key.len().saturating_add(value_str.len()).saturating_add(1);
                let mut combined = String::with_capacity(capacity);
                combined.push_str(key);
                combined.push(':');
                combined.push_str(value_str.as_ref());
                entries.push(combined.into_boxed_str());
            }
        }
        entries
    }

    fn note_folder(path: &str) -> Option<Box<str>> {
        Path::new(path)
            .parent()
            .and_then(|parent| parent.to_str())
            .filter(|folder| !folder.is_empty())
            .map(Into::into)
    }

    fn field_value_index_values(value: &FieldValue) -> Vec<Box<str>> {
        if let Some(text) = value.as_str() {
            return vec![text.into()];
        }
        if let Some(flag) = value.as_bool() {
            return if flag {
                vec!["true".into()]
            } else {
                vec!["false".into()]
            };
        }
        if let Some(timestamp) = value.as_date() {
            return vec![Self::format_i64(timestamp)];
        }
        if let Some(number) = value.as_number() {
            return vec![Self::format_f64(number)];
        }
        if let Some(values) = value.as_array() {
            let mut out = Vec::new();
            for item in values {
                out.extend(Self::field_value_index_values(item));
            }
            return out;
        }

        vec![value.to_json_string().into()]
    }

    fn format_i64(value: i64) -> Box<str> {
        let mut buffer = itoa::Buffer::new();
        buffer.format(value).into()
    }

    fn format_f64(value: f64) -> Box<str> {
        let mut buffer = ryu::Buffer::new();
        buffer.format(value).into()
    }
}

impl Command for CommandAdapter<'_, '_> {
    type Error = crate::db::DbError;

    #[inline]
    fn record_parsed_note(
        &self,
        path: &NotePath,
        parsed: &ParsedNote,
    ) -> Result<NoteId, Self::Error> {
        self.upsert_parsed_note(path, parsed)
    }

    #[inline]
    fn record_deleted_note(&self, id: NoteId) -> Result<(), Self::Error> {
        let uuid = Uuid::from(id);
        let mut id_buffer = Uuid::encode_buffer();
        let id_str = uuid.as_hyphenated().encode_lower(&mut id_buffer);
        let id_str: &str = id_str;
        let stored = self.db.get_owned::<StoredNote>(STORED_NOTES, id_str)?;

        if let Some(stored) = stored {
            let tasks = self.load_task_indexes_for_note(id)?;
            let index_data =
                self.collect_index_data_from_stored(&stored, tasks);
            let event = Self::build_note_event_from_stored(
                &stored,
                index_data.task_indexes.tasks.len(),
                NoteChangeKind::Deleted,
            )?;

            self.db.batch_write(|batch| {
                Self::remove_indexes(batch, &index_data, id_str)?;
                batch.delete(STORED_NOTES, id_str)?;
                Self::insert_note_event(batch, &event)?;
                Ok(())
            })?;
        }

        Ok(())
    }

    #[inline]
    fn rebuild_note_indexes(&self) -> Result<usize, Self::Error> {
        let stored_notes = self.db.list_owned::<StoredNote>(STORED_NOTES)?;
        let mut rebuilds = Vec::with_capacity(stored_notes.len());

        for stored in &stored_notes {
            let note_id = stored.id();
            let tasks = self.load_task_indexes_for_note(note_id)?;
            let index_data = self.collect_index_data_from_stored(stored, tasks);
            let id = Uuid::from(note_id);
            let mut id_buffer = Uuid::encode_buffer();
            let id_str = id.as_hyphenated().encode_lower(&mut id_buffer);
            rebuilds.push((id_str.to_owned(), index_data));
        }

        self.db.batch_write(|batch| {
            for (id_str, index_data) in
                rebuilds.iter().map(|entry| (entry.0.as_str(), &entry.1))
            {
                Self::remove_indexes(batch, index_data, id_str)?;
                Self::insert_indexes(batch, index_data, id_str)?;
            }
            Ok(())
        })?;

        Ok(stored_notes.len())
    }

    #[inline]
    fn rebuild_task_indexes(&self) -> Result<usize, Self::Error> {
        let stored_tasks = self.db.list_owned::<StoredTask>(TASKS)?;

        let index_all_fields = self.config.task().indexed().is_empty();
        let dependencies_enabled = self.config.task().dependencies_enabled();

        let mut existing = TaskIndexData {
            tasks: Vec::with_capacity(stored_tasks.len()),
            dependencies_enabled,
        };
        for stored in stored_tasks {
            let metadata_keys = Self::task_metadata_keys(
                stored.metadata(),
                self.config,
                index_all_fields,
            );
            existing.tasks.push(TaskIndexEntry {
                stored,
                metadata_keys,
            });
        }

        let rebuilt_entries = existing.tasks.clone();
        let total = rebuilt_entries.len();
        let rebuilt = TaskIndexData {
            tasks: rebuilt_entries,
            dependencies_enabled,
        };

        self.db.batch_write(|batch| {
            Self::remove_task_indexes(batch, &existing)?;
            Self::insert_task_indexes(batch, &rebuilt)?;
            Ok(())
        })?;

        Ok(total)
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    clippy::panic_in_result_fn,
    reason = "Test module groups fixtures and submodules for readability."
)]
mod tests {
    use tempfile::{TempDir, tempdir};

    use super::*;
    use crate::{
        fs::FsReader,
        note::{
            adapter::reader::NoteReader,
            error::{NoteCommandError, NoteError},
        },
    };

    mod fixtures {
        use super::*;
        use crate::{
            config::{
                aggregate::Config,
                raw::RawConfig,
                vault::{VaultId, VaultRoot},
            },
            note::{aggregate::NoteId, paths::NotePath, tag::Tag},
        };

        // pub const TEST_MISSING_ID: Uuid =
        //     Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0301);

        pub fn test_db() -> Result<(TempDir, Database), String> {
            let dir = tempdir().map_err(|e| e.to_string())?;
            let path = dir.path().join("notes.redb");
            let db = Database::open(&path).map_err(|e| e.to_string())?;
            Ok((dir, db))
        }

        pub fn test_config() -> Result<Config, String> {
            Config::build(
                &RawConfig::default(),
                VaultId::new(),
                VaultRoot::try_new(std::path::PathBuf::from("/vault"))
                    .map_err(|e| e.to_string())?,
                crate::config::aggregate::Version::initial(),
            )
            .map_err(|e| e.to_string())
        }

        pub fn upsert_parsed(
            cmd: &CommandAdapter<'_, '_>,
            path: &NotePath,
            parsed: &ParsedNote,
        ) -> Result<NoteId, NoteCommandError> {
            Ok(Command::record_parsed_note(cmd, path, parsed)?)
        }

        pub fn parse_path(path: &str) -> Result<NotePath, String> {
            NotePath::try_new(path).map_err(|e| e.to_string())
        }

        pub fn parse_tag(tag: &str) -> Result<Tag, String> {
            Tag::try_from_token(tag).map_err(|e| e.to_string())
        }

        pub fn stored_note_projection(
            db: &Database,
            id: Uuid,
        ) -> Result<Option<StoredNote>, String> {
            db.get_owned::<StoredNote>(STORED_NOTES, &id.to_string())
                .map_err(|e| e.to_string())
        }

        pub fn path_index_ids(
            db: &Database,
            path: &str,
        ) -> Result<Vec<String>, String> {
            db.multimap_get(PATH_TO_ID, path).map_err(|e| e.to_string())
        }

        pub fn tag_index_ids(
            db: &Database,
            tag: &str,
        ) -> Result<Vec<String>, String> {
            db.multimap_get(TAGS_TO_NOTES, tag).map_err(|e| e.to_string())
        }

        pub fn note_events(db: &Database) -> Result<Vec<NoteEvent>, String> {
            db.list_owned::<NoteEvent>(NOTE_EVENTS).map_err(|e| e.to_string())
        }

        pub fn stored_tasks(db: &Database) -> Result<Vec<StoredTask>, String> {
            db.list_owned::<StoredTask>(TASKS).map_err(|e| e.to_string())
        }
    }

    mod persistence {
        use super::*;

        #[test]
        fn upsert_persists_note_path() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let config = fixtures::test_config().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let cmd = CommandAdapter::new(&db, &config);

            let path = fixtures::parse_path("notes/a.md").map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let parsed =
                NoteReader::new(&config).parse_str("# Title").map_err(
                    |error| NoteCommandError::Domain(NoteError::from(error)),
                )?;
            let note_id = fixtures::upsert_parsed(&cmd, &path, &parsed)?;
            let stored_note =
                fixtures::stored_note_projection(&db, Uuid::from(note_id))
                    .map_err(|e| {
                        NoteCommandError::Domain(NoteError::Storage(e.into()))
                    })?
                    .expect("Stored note should exist");
            assert_eq!(
                stored_note.path().as_str(),
                "notes/a.md",
                "Stored note path should match"
            );
            Ok(())
        }

        #[test]
        fn upsert_persists_stored_note_projection()
        -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let config = fixtures::test_config().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let cmd = CommandAdapter::new(&db, &config);

            let path = fixtures::parse_path("notes/a.md").map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let parsed =
                NoteReader::new(&config).parse_str("# Title").map_err(
                    |error| NoteCommandError::Domain(NoteError::from(error)),
                )?;
            let id = Uuid::from(fixtures::upsert_parsed(&cmd, &path, &parsed)?);

            let stored = fixtures::stored_note_projection(&db, id)
                .map_err(|e| {
                    NoteCommandError::Domain(NoteError::Storage(e.into()))
                })?
                .expect("Stored note projection should exist");
            assert_eq!(stored.path().as_str(), "notes/a.md");
            Ok(())
        }

        #[test]
        fn upsert_parsed_note_persists_file_timestamps()
        -> Result<(), NoteCommandError> {
            let (dir, db) = fixtures::test_db().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let config = fixtures::test_config().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let cmd = CommandAdapter::new(&db, &config);

            let notes_dir = dir.path().join("notes");
            std::fs::create_dir_all(&notes_dir).map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(
                    e.to_string().into(),
                ))
            })?;
            std::fs::write(notes_dir.join("ingest.md"), "# Title").map_err(
                |e| {
                    NoteCommandError::Domain(NoteError::Storage(
                        e.to_string().into(),
                    ))
                },
            )?;

            let reader = FsReader::new(dir.path());
            let parsed = NoteReader::new(&config)
                .parse(&reader, std::path::Path::new("notes/ingest.md"))
                .map_err(|error| {
                    NoteCommandError::Domain(NoteError::from(error))
                })?;
            let path =
                fixtures::parse_path("notes/ingest.md").map_err(|e| {
                    NoteCommandError::Domain(NoteError::Storage(e.into()))
                })?;
            let note_id =
                cmd.upsert_parsed_note(&path, &parsed).map_err(|e| {
                    NoteCommandError::Domain(NoteError::Storage(
                        e.to_string().into(),
                    ))
                })?;

            let stored =
                fixtures::stored_note_projection(&db, Uuid::from(note_id))
                    .map_err(|e| {
                        NoteCommandError::Domain(NoteError::Storage(e.into()))
                    })?
                    .expect("stored note should exist");
            let stored_created = stored
                .created_at()
                .and_then(|time| {
                    time.duration_since(SystemTime::UNIX_EPOCH).ok()
                })
                .map(|duration| duration.as_secs());
            let parsed_created = parsed
                .created_at()
                .and_then(|time| {
                    time.duration_since(SystemTime::UNIX_EPOCH).ok()
                })
                .map(|duration| duration.as_secs());
            assert_eq!(stored_created, parsed_created);

            let stored_modified = stored
                .modified_at()
                .and_then(|time| {
                    time.duration_since(SystemTime::UNIX_EPOCH).ok()
                })
                .map(|duration| duration.as_secs());
            let parsed_modified = parsed
                .modified_at()
                .and_then(|time| {
                    time.duration_since(SystemTime::UNIX_EPOCH).ok()
                })
                .map(|duration| duration.as_secs());
            assert_eq!(stored_modified, parsed_modified);
            let recorded = stored
                .last_indexed_at()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(|error| {
                    NoteCommandError::Domain(NoteError::Storage(
                        error.to_string().into(),
                    ))
                })?
                .as_secs();
            assert!(recorded > 0);
            Ok(())
        }

        #[test]
        fn upsert_parsed_note_records_task_location()
        -> Result<(), NoteCommandError> {
            let (dir, db) = fixtures::test_db().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let config = fixtures::test_config().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let cmd = CommandAdapter::new(&db, &config);

            let notes_dir = dir.path().join("notes");
            std::fs::create_dir_all(&notes_dir).map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(
                    e.to_string().into(),
                ))
            })?;
            std::fs::write(
                notes_dir.join("ingest.md"),
                "first\r\n- [ ] #task Review PR",
            )
            .map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(
                    e.to_string().into(),
                ))
            })?;

            let reader = FsReader::new(dir.path());
            let parsed = NoteReader::new(&config)
                .parse(&reader, std::path::Path::new("notes/ingest.md"))
                .map_err(|error| {
                    NoteCommandError::Domain(NoteError::from(error))
                })?;
            let path =
                fixtures::parse_path("notes/ingest.md").map_err(|e| {
                    NoteCommandError::Domain(NoteError::Storage(e.into()))
                })?;
            let note_id =
                cmd.upsert_parsed_note(&path, &parsed).map_err(|e| {
                    NoteCommandError::Domain(NoteError::Storage(
                        e.to_string().into(),
                    ))
                })?;

            let tasks = fixtures::stored_tasks(&db).map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let task = tasks
                .iter()
                .find(|stored| stored.note_id() == note_id)
                .expect("stored task should exist");
            let location =
                task.location().expect("task should store a source location");
            assert_eq!(location.line().value(), 2);
            assert_eq!(location.column().value(), 1);
            Ok(())
        }

        #[test]
        fn upsert_emits_note_event() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let config = fixtures::test_config().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let cmd = CommandAdapter::new(&db, &config);

            let path = fixtures::parse_path("notes/a.md").map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let parsed =
                NoteReader::new(&config).parse_str("# Title").map_err(
                    |error| NoteCommandError::Domain(NoteError::from(error)),
                )?;
            let note_id = fixtures::upsert_parsed(&cmd, &path, &parsed)?;

            let events = fixtures::note_events(&db).map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            assert_eq!(events.len(), 1);
            let event = events.first().ok_or_else(|| {
                NoteCommandError::Domain(NoteError::Storage(
                    "missing event".into(),
                ))
            })?;
            assert_eq!(event.note_id(), note_id);
            assert_eq!(event.kind(), NoteEventKind::Indexed);
            let payload = event.payload_v1().ok_or_else(|| {
                NoteCommandError::Domain(NoteError::Storage(
                    "missing payload".into(),
                ))
            })?;
            assert_eq!(payload.change(), Some(NoteChangeKind::Created));
            Ok(())
        }

        #[test]
        fn upsert_persists_path_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let config = fixtures::test_config().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let cmd = CommandAdapter::new(&db, &config);

            let path = fixtures::parse_path("notes/a.md").map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let parsed =
                NoteReader::new(&config).parse_str("# Title").map_err(
                    |error| NoteCommandError::Domain(NoteError::from(error)),
                )?;
            let id = Uuid::from(fixtures::upsert_parsed(&cmd, &path, &parsed)?);

            let ids =
                fixtures::path_index_ids(&db, path.as_str()).map_err(|e| {
                    NoteCommandError::Domain(NoteError::Storage(e.into()))
                })?;
            assert!(
                ids.contains(&id.to_string()),
                "Path index should contain created note id"
            );
            Ok(())
        }

        #[test]
        fn upsert_emits_update_event() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let config = fixtures::test_config().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let cmd = CommandAdapter::new(&db, &config);

            let path = fixtures::parse_path("notes/a.md").map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let parsed =
                NoteReader::new(&config).parse_str("# Title").map_err(
                    |error| NoteCommandError::Domain(NoteError::from(error)),
                )?;
            let note_id = fixtures::upsert_parsed(&cmd, &path, &parsed)?;
            let updated =
                NoteReader::new(&config).parse_str("# Title\n#tag").map_err(
                    |error| NoteCommandError::Domain(NoteError::from(error)),
                )?;
            let _note_id = fixtures::upsert_parsed(&cmd, &path, &updated)?;

            let events = fixtures::note_events(&db).map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            assert_eq!(events.len(), 2);
            assert!(events.iter().any(|event| {
                event.note_id() == note_id
                    && event.kind() == NoteEventKind::Indexed
                    && event.payload_v1().and_then(NoteEventPayloadV1::change)
                        == Some(NoteChangeKind::Created)
            }));
            assert!(events.iter().any(|event| {
                event.note_id() == note_id
                    && event.kind() == NoteEventKind::Indexed
                    && event.payload_v1().and_then(NoteEventPayloadV1::change)
                        == Some(NoteChangeKind::Updated)
            }));
            Ok(())
        }

        #[test]
        fn upsert_updates_tag_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let config = fixtures::test_config().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let cmd = CommandAdapter::new(&db, &config);

            let path = fixtures::parse_path("notes/a.md").map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let parsed =
                NoteReader::new(&config).parse_str("# Title").map_err(
                    |error| NoteCommandError::Domain(NoteError::from(error)),
                )?;
            let id = Uuid::from(fixtures::upsert_parsed(&cmd, &path, &parsed)?);
            let updated = NoteReader::new(&config)
                .parse_str("# Title\n#project")
                .map_err(|error| {
                    NoteCommandError::Domain(NoteError::from(error))
                })?;
            let _note_id = fixtures::upsert_parsed(&cmd, &path, &updated)?;

            let tag_key = fixtures::parse_tag("#project")
                .map_err(|e| {
                    NoteCommandError::Domain(NoteError::Storage(e.into()))
                })?
                .full_path()
                .to_owned();

            let tag_ids =
                fixtures::tag_index_ids(&db, &tag_key).map_err(|e| {
                    NoteCommandError::Domain(NoteError::Storage(e.into()))
                })?;
            assert!(
                tag_ids.contains(&id.to_string()),
                "Tag index should contain updated note id"
            );
            Ok(())
        }

        #[test]
        fn delete_removes_note() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let config = fixtures::test_config().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let cmd = CommandAdapter::new(&db, &config);

            let path = fixtures::parse_path("notes/a.md").map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let parsed =
                NoteReader::new(&config).parse_str("# Title").map_err(
                    |error| NoteCommandError::Domain(NoteError::from(error)),
                )?;
            let note_id = fixtures::upsert_parsed(&cmd, &path, &parsed)?;
            let id = Uuid::from(note_id);

            cmd.record_deleted_note(note_id)
                .map_err(NoteCommandError::Storage)?;

            let stored =
                fixtures::stored_note_projection(&db, id).map_err(|e| {
                    NoteCommandError::Domain(NoteError::Storage(e.into()))
                })?;
            assert!(stored.is_none(), "Deleted note should not exist");
            Ok(())
        }

        #[test]
        fn delete_emits_note_event() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let config = fixtures::test_config().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let cmd = CommandAdapter::new(&db, &config);

            let path = fixtures::parse_path("notes/a.md").map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let parsed =
                NoteReader::new(&config).parse_str("# Title").map_err(
                    |error| NoteCommandError::Domain(NoteError::from(error)),
                )?;
            let id = fixtures::upsert_parsed(&cmd, &path, &parsed)?;

            cmd.record_deleted_note(id).map_err(NoteCommandError::Storage)?;
            let events = fixtures::note_events(&db).map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            assert_eq!(events.len(), 2);
            assert!(events.iter().any(|event| {
                event.note_id() == id
                    && event.kind() == NoteEventKind::Changed
                    && event.payload_v1().and_then(NoteEventPayloadV1::change)
                        == Some(NoteChangeKind::Deleted)
            }));
            Ok(())
        }

        #[test]
        fn upsert_same_path_is_idempotent() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let config = fixtures::test_config().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let cmd = CommandAdapter::new(&db, &config);

            let path = fixtures::parse_path("notes/a.md").map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let parsed =
                NoteReader::new(&config).parse_str("# Title").map_err(
                    |error| NoteCommandError::Domain(NoteError::from(error)),
                )?;
            let first = fixtures::upsert_parsed(&cmd, &path, &parsed)?;
            let updated =
                NoteReader::new(&config).parse_str("# Title\n#tag").map_err(
                    |error| NoteCommandError::Domain(NoteError::from(error)),
                )?;
            let second = fixtures::upsert_parsed(&cmd, &path, &updated)?;

            assert_eq!(first, second, "idempotent upsert should reuse id");
            Ok(())
        }
    }
}
