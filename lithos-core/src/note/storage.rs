//! Unified note repository backed by redb.
//!
//! Combines read and write operations for the note context in a single
//! repository interface, following the File → Raw → Domain → Storage pipeline.

#![expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Repository methods grouped by behavior"
)]
#![expect(
    clippy::missing_errors_doc,
    reason = "Repository methods share error semantics"
)]

use std::{fmt::Write as _, path::Path, sync::Arc, time::SystemTime};

use uuid::Uuid;

use crate::{
    config::{
        aggregate::Config, frontmatter::FrontmatterKey, task::StatusName,
    },
    db::{BatchWriter, Database, DbError},
    note::{
        ALIAS_TO_ID, FILE_CLASS_TO_ID, FOLDER_TO_ID, FRONTMATTER_KV,
        NOTE_EVENTS, PATH_TO_ID, STORED_NOTES, TAGS_TO_NOTES,
        TASKS_BY_COMPLETED_DATE, TASKS_BY_CREATED_DATE, TASKS_BY_DEPENDS_ON,
        TASKS_BY_DUE_DATE, TASKS_BY_METADATA, TASKS_BY_REMINDER_DATE,
        TASKS_BY_STATUS,
        aggregate::{AliasName, FileClassName, NoteFacts, NoteId},
        error::NoteError,
        events::{
            NoteChangeKind, NoteEvent, NoteEventKind, NoteEventPayload,
            NoteEventPayloadV1,
        },
        frontmatter::Frontmatter,
        paths::{FolderPath, NotePath},
        task::{Task, TaskDateKind, TaskMetadata, TaskPriority, TaskTimestamp},
        value::FieldValue,
    },
};

/// Note view type returned by repository queries.
pub type NoteView = NoteFacts;
/// Task view type returned by repository queries.
pub type TaskView = Task;

/// Unified repository trait for note storage and queries.
pub trait Repository: Send + Sync {
    /// Storage error type for repository operations.
    type Error: std::error::Error;

    /// Archived note type for zero-copy reads.
    type NoteArchived<'archived>;

    /// Rebuilds all note indexes from stored projections.
    fn rebuild_note_indexes(&self) -> Result<usize, Self::Error>;

    /// Rebuilds all task indexes from stored notes.
    fn rebuild_task_indexes(&self) -> Result<usize, Self::Error>;

    /// Deletes a note projection by id.
    fn delete_note(&self, id: NoteId) -> Result<(), Self::Error>;

    /// Persists normalized note facts.
    fn save_note_facts(&self, facts: &NoteFacts)
    -> Result<NoteId, Self::Error>;

    /// Finds a stored note projection by its configured alias.
    fn find_by_alias(
        &self,
        alias: &AliasName,
    ) -> Result<Option<NoteView>, Self::Error>;

    /// Finds a stored note projection by its unique UUID v7 identifier.
    fn find_by_id(&self, id: NoteId) -> Result<Option<NoteView>, Self::Error>;

    /// Finds a stored note projection by its vault-relative path.
    fn find_by_path(
        &self,
        path: &NotePath,
    ) -> Result<Option<NoteView>, Self::Error>;

    /// Lists all stored note projections currently managed in the vault.
    fn list(&self) -> Result<Vec<NoteView>, Self::Error>;

    /// Finds all stored note projections belonging to a specific file class.
    fn list_by_file_class(
        &self,
        class: &FileClassName,
    ) -> Result<Vec<NoteView>, Self::Error>;

    /// Finds all stored note projections located within a specific vault
    /// folder.
    fn list_by_folder(
        &self,
        folder: &FolderPath,
    ) -> Result<Vec<NoteView>, Self::Error>;

    /// Queries stored notes by a generic frontmatter key-value pair.
    fn list_by_frontmatter_kv(
        &self,
        key: &FrontmatterKey,
        value: &str,
    ) -> Result<Vec<NoteView>, Self::Error>;

    /// Finds all stored notes containing tasks completed on a specific date.
    fn list_by_task_completed_date(
        &self,
        completed_date: TaskTimestamp,
    ) -> Result<Vec<NoteView>, Self::Error>;

    /// Finds all stored notes containing tasks created on a specific date.
    fn list_by_task_created_date(
        &self,
        created_date: TaskTimestamp,
    ) -> Result<Vec<NoteView>, Self::Error>;

    /// Finds all stored notes containing tasks due on a specific date.
    fn list_by_task_due_date(
        &self,
        due_date: TaskTimestamp,
    ) -> Result<Vec<NoteView>, Self::Error>;

    /// Finds all stored notes containing tasks with a specific priority level.
    fn list_by_task_priority(
        &self,
        priority: TaskPriority,
    ) -> Result<Vec<NoteView>, Self::Error>;

    /// Finds all stored notes containing tasks assigned to a specific project.
    fn list_by_task_project(
        &self,
        project: &str,
    ) -> Result<Vec<NoteView>, Self::Error>;

    /// Finds all stored notes containing tasks with a specific reminder date.
    fn list_by_task_reminder_date(
        &self,
        reminder_date: TaskTimestamp,
    ) -> Result<Vec<NoteView>, Self::Error>;

    /// Finds all stored notes containing tasks with a specific status name.
    fn list_by_task_status(
        &self,
        status: &StatusName,
    ) -> Result<Vec<NoteView>, Self::Error>;

    /// Lists tasks by a specific task date field.
    fn list_tasks_by_date(
        &self,
        kind: TaskDateKind,
        date: TaskTimestamp,
    ) -> Result<Vec<TaskView>, Self::Error>;

    /// Lists tasks by a metadata field/value pair.
    fn list_tasks_by_metadata(
        &self,
        field: &str,
        value: &FieldValue,
    ) -> Result<Vec<TaskView>, Self::Error>;

    /// Lists tasks with a specific status name.
    fn list_tasks_by_status(
        &self,
        status: &StatusName,
    ) -> Result<Vec<TaskView>, Self::Error>;

    /// Accesses a note by ID as archived data, enabling zero-copy reads.
    fn with_archived_by_id<F, R>(
        &self,
        id: NoteId,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::NoteArchived<'archived>) -> R;
}

/// Redb-backed note repository adapter.
pub struct RedbRepository<'db, 'config> {
    db: &'db Database,
    config: &'config Config,
}

impl<'db, 'config> RedbRepository<'db, 'config> {
    /// Create a new repository with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database, config: &'config Config) -> Self {
        Self {
            db,
            config,
        }
    }

    /// Access the configuration used by this repository.
    #[inline]
    #[must_use]
    pub const fn config(&self) -> &'config Config {
        self.config
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

    fn collect_index_data_from_facts(&self, facts: &NoteFacts) -> IndexData {
        let frontmatter = facts.frontmatter();
        let aliases = frontmatter
            .map(|fm| fm.aliases(self.config).map(Into::into).collect())
            .unwrap_or_default();
        let file_class = frontmatter
            .and_then(|fm| fm.file_class(self.config))
            .map(Into::into);
        let frontmatter_entries =
            frontmatter.map(Self::frontmatter_entries).unwrap_or_default();

        IndexData {
            path: facts.path().as_str().into(),
            folder: Self::note_folder(facts.path().as_str()),
            tags: facts
                .tags()
                .iter()
                .map(|tag| tag.full_path().into())
                .collect(),
            aliases,
            file_class,
            task_indexes: Self::task_indexes_from_facts(facts, self.config),
            frontmatter_entries,
        }
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

    fn build_note_event_from_facts(
        note_id: NoteId,
        path: &NotePath,
        facts: &NoteFacts,
        change: NoteChangeKind,
    ) -> Result<NoteEvent, DbError> {
        let task_count =
            u32::try_from(facts.tasks().len()).map_err(|_error| {
                DbError::Table("task count out of range".into())
            })?;
        let tag_count =
            u32::try_from(facts.tags().len()).map_err(|_error| {
                DbError::Table("tag count out of range".into())
            })?;
        let source_hash = facts.source_hash().into();
        let source_bytes = facts.source_bytes();

        let payload = NoteEventPayload::V1(NoteEventPayloadV1::indexed(
            change,
            task_count,
            tag_count,
            Some(source_hash),
            Some(source_bytes),
            facts.modified_at(),
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

    fn insert_note_event(
        batch: &mut BatchWriter,
        event: &NoteEvent,
    ) -> Result<(), DbError> {
        let mut id_buffer = Uuid::encode_buffer();
        let id_str = event.id().as_hyphenated().encode_lower(&mut id_buffer);
        batch.put(NOTE_EVENTS, id_str, event)
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

        Self::insert_task_indexes(batch, &index_data.task_indexes, id_str)
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

        Self::remove_task_indexes(batch, &index_data.task_indexes, id_str)
    }

    fn task_indexes_from_facts(
        facts: &NoteFacts,
        config: &Config,
    ) -> TaskIndexData {
        let index_all_fields = config.task().indexed().is_empty();
        let dependencies_enabled = config.task().dependencies_enabled();

        let mut status_keys = Vec::new();
        let mut created_dates = Vec::new();
        let mut due_dates = Vec::new();
        let mut reminder_dates = Vec::new();
        let mut completed_dates = Vec::new();
        let mut metadata_keys = Vec::new();
        let mut depends_on = Vec::new();

        for task in facts.tasks() {
            status_keys.push(task.status().as_str().into());
            if let Some(timestamp) = task.created_at() {
                created_dates.push(Self::format_i64(timestamp.as_i64()));
            }
            if let Some(timestamp) = task.due_at() {
                due_dates.push(Self::format_i64(timestamp.as_i64()));
            }
            if let Some(timestamp) = task.reminder_at() {
                reminder_dates.push(Self::format_i64(timestamp.as_i64()));
            }
            if let Some(timestamp) = task.completed_at() {
                completed_dates.push(Self::format_i64(timestamp.as_i64()));
            }

            metadata_keys.extend(Self::task_metadata_keys(
                task.metadata(),
                config,
                index_all_fields,
            ));
            depends_on.extend(Self::task_depends_on(
                task.metadata(),
                dependencies_enabled,
            ));
        }

        TaskIndexData {
            status_keys,
            created_dates,
            due_dates,
            reminder_dates,
            completed_dates,
            metadata_keys,
            depends_on,
        }
    }

    fn insert_task_indexes(
        batch: &mut BatchWriter,
        index_data: &TaskIndexData,
        note_id_str: &str,
    ) -> Result<(), DbError> {
        for key in &index_data.status_keys {
            batch.multimap_insert(
                TASKS_BY_STATUS,
                key.as_ref(),
                note_id_str,
            )?;
        }
        for key in &index_data.created_dates {
            batch.multimap_insert(
                TASKS_BY_CREATED_DATE,
                key.as_ref(),
                note_id_str,
            )?;
        }
        for key in &index_data.due_dates {
            batch.multimap_insert(
                TASKS_BY_DUE_DATE,
                key.as_ref(),
                note_id_str,
            )?;
        }
        for key in &index_data.reminder_dates {
            batch.multimap_insert(
                TASKS_BY_REMINDER_DATE,
                key.as_ref(),
                note_id_str,
            )?;
        }
        for key in &index_data.completed_dates {
            batch.multimap_insert(
                TASKS_BY_COMPLETED_DATE,
                key.as_ref(),
                note_id_str,
            )?;
        }
        for key in &index_data.metadata_keys {
            batch.multimap_insert(
                TASKS_BY_METADATA,
                key.as_ref(),
                note_id_str,
            )?;
        }
        for key in &index_data.depends_on {
            batch.multimap_insert(
                TASKS_BY_DEPENDS_ON,
                key.as_ref(),
                note_id_str,
            )?;
        }
        Ok(())
    }

    fn remove_task_indexes(
        batch: &mut BatchWriter,
        index_data: &TaskIndexData,
        note_id_str: &str,
    ) -> Result<(), DbError> {
        for key in &index_data.status_keys {
            batch.multimap_remove(
                TASKS_BY_STATUS,
                key.as_ref(),
                note_id_str,
            )?;
        }
        for key in &index_data.created_dates {
            batch.multimap_remove(
                TASKS_BY_CREATED_DATE,
                key.as_ref(),
                note_id_str,
            )?;
        }
        for key in &index_data.due_dates {
            batch.multimap_remove(
                TASKS_BY_DUE_DATE,
                key.as_ref(),
                note_id_str,
            )?;
        }
        for key in &index_data.reminder_dates {
            batch.multimap_remove(
                TASKS_BY_REMINDER_DATE,
                key.as_ref(),
                note_id_str,
            )?;
        }
        for key in &index_data.completed_dates {
            batch.multimap_remove(
                TASKS_BY_COMPLETED_DATE,
                key.as_ref(),
                note_id_str,
            )?;
        }
        for key in &index_data.metadata_keys {
            batch.multimap_remove(
                TASKS_BY_METADATA,
                key.as_ref(),
                note_id_str,
            )?;
        }
        for key in &index_data.depends_on {
            batch.multimap_remove(
                TASKS_BY_DEPENDS_ON,
                key.as_ref(),
                note_id_str,
            )?;
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
                keys.extend(Self::metadata_index_keys(field_name, value));
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

    /// Build typed metadata index keys for the provided field/value pair.
    #[inline]
    #[must_use]
    fn metadata_index_keys(field: &str, value: &FieldValue) -> Vec<Box<str>> {
        let mut keys = Vec::new();
        Self::push_metadata_keys(field, value, &mut keys);
        keys
    }

    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &FieldValue are intentional"
    )]
    fn push_metadata_keys(
        field: &str,
        value: &FieldValue,
        out: &mut Vec<Box<str>>,
    ) {
        match value {
            FieldValue::String(value) => {
                out.push(Self::encode_metadata_key(field, "s:", value).into());
            }
            FieldValue::Number(value) => {
                let mut buffer = ryu::Buffer::new();
                let encoded = buffer.format(*value);
                out.push(
                    Self::encode_metadata_key(field, "n:", encoded).into(),
                );
            }
            FieldValue::Boolean(value) => {
                let encoded = if *value {
                    "true"
                } else {
                    "false"
                };
                out.push(
                    Self::encode_metadata_key(field, "b:", encoded).into(),
                );
            }
            FieldValue::Date(value) => {
                let mut buffer = itoa::Buffer::new();
                let encoded = buffer.format(*value);
                out.push(
                    Self::encode_metadata_key(field, "d:", encoded).into(),
                );
            }
            FieldValue::Array(values) => {
                for item in values {
                    Self::push_metadata_keys(field, item, out);
                }
            }
            FieldValue::Object(_) => {
                let encoded = value.to_json_string();
                out.push(
                    Self::encode_metadata_key(field, "o:", encoded.as_str())
                        .into(),
                );
            }
        }
    }

    fn encode_metadata_key(field: &str, prefix: &str, value: &str) -> String {
        let capacity = field
            .len()
            .saturating_add(prefix.len())
            .saturating_add(value.len())
            .saturating_add(1);
        let mut out = String::with_capacity(capacity);
        #[expect(
            clippy::let_underscore_must_use,
            reason = "Writing to String is infallible"
        )]
        let _ = write!(&mut out, "{field}\0{prefix}{value}");
        out
    }

    fn format_i64(value: i64) -> Box<str> {
        let mut buffer = itoa::Buffer::new();
        buffer.format(value).into()
    }

    fn format_f64(value: f64) -> Box<str> {
        let mut buffer = ryu::Buffer::new();
        buffer.format(value).into()
    }

    fn list_notes_by_task_index(
        &self,
        index_table: redb::MultimapTableDefinition<&str, &str>,
        index_key: &str,
    ) -> Result<Vec<NoteFacts>, DbError> {
        use std::collections::BTreeSet;

        let note_refs = self.db.multimap_get(index_table, index_key)?;
        let mut note_ids = BTreeSet::new();
        for note_id_str in note_refs {
            note_ids.insert(note_id_str);
        }

        let mut notes = Vec::with_capacity(note_ids.len());
        for note_id_str in note_ids {
            if let Some(note) =
                self.db.get_owned::<NoteFacts>(STORED_NOTES, &note_id_str)?
            {
                notes.push(note);
            }
        }
        Ok(notes)
    }

    fn list_notes_by_index(
        &self,
        index_table: redb::MultimapTableDefinition<&str, &str>,
        index_key: &str,
    ) -> Result<Vec<NoteFacts>, DbError> {
        let note_refs = self.db.multimap_get(index_table, index_key)?;
        let mut notes = Vec::with_capacity(note_refs.len());
        for note_id_str in note_refs {
            if let Some(note) =
                self.db.get_owned::<NoteFacts>(STORED_NOTES, &note_id_str)?
            {
                notes.push(note);
            }
        }
        Ok(notes)
    }

    fn collect_tasks_matching<F>(notes: &[NoteFacts], predicate: F) -> Vec<Task>
    where
        F: Fn(&Task) -> bool,
    {
        use std::collections::HashSet;

        let mut seen = HashSet::new();
        let mut tasks = Vec::new();
        for note in notes {
            for task in note.tasks() {
                if predicate(task) && seen.insert(task.id()) {
                    tasks.push(task.clone());
                }
            }
        }
        tasks
    }
}

impl Repository for RedbRepository<'_, '_> {
    type Error = DbError;
    type NoteArchived<'archived> = &'archived rkyv::Archived<NoteView>;

    #[inline]
    fn rebuild_note_indexes(&self) -> Result<usize, Self::Error> {
        let stored_notes = self.db.list_owned::<NoteFacts>(STORED_NOTES)?;
        let mut rebuilds = Vec::with_capacity(stored_notes.len());

        for stored in &stored_notes {
            let note_id = stored.id();
            let index_data = self.collect_index_data_from_facts(stored);
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
        let stored_notes = self.db.list_owned::<NoteFacts>(STORED_NOTES)?;
        let mut rebuilds = Vec::with_capacity(stored_notes.len());
        let mut total_tasks = 0usize;

        for note in &stored_notes {
            let task_indexes = Self::task_indexes_from_facts(note, self.config);
            total_tasks = total_tasks.saturating_add(note.tasks().len());
            let id = Uuid::from(note.id());
            let mut id_buffer = Uuid::encode_buffer();
            let id_str = id.as_hyphenated().encode_lower(&mut id_buffer);
            rebuilds.push((id_str.to_owned(), task_indexes));
        }

        self.db.batch_write(|batch| {
            for (id_str, index_data) in
                rebuilds.iter().map(|entry| (entry.0.as_str(), &entry.1))
            {
                Self::remove_task_indexes(batch, index_data, id_str)?;
                Self::insert_task_indexes(batch, index_data, id_str)?;
            }
            Ok(())
        })?;

        Ok(total_tasks)
    }

    #[inline]
    fn delete_note(&self, id: NoteId) -> Result<(), Self::Error> {
        let uuid = Uuid::from(id);
        let mut id_buffer = Uuid::encode_buffer();
        let id_str = uuid.as_hyphenated().encode_lower(&mut id_buffer);
        let id_str: &str = id_str;
        let stored = self.db.get_owned::<NoteFacts>(STORED_NOTES, id_str)?;

        if let Some(stored) = stored {
            let index_data = self.collect_index_data_from_facts(&stored);
            let event = Self::build_note_event_from_facts(
                stored.id(),
                stored.path(),
                &stored,
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
    fn save_note_facts(
        &self,
        facts: &NoteFacts,
    ) -> Result<NoteId, Self::Error> {
        let path = facts.path();
        let existing_id = self.find_note_id_by_path(path)?;
        let note_id = existing_id.unwrap_or_else(NoteId::new);
        let id = Uuid::from(note_id);
        let mut id_buffer = Uuid::encode_buffer();
        let id_str = id.as_hyphenated().encode_lower(&mut id_buffer);
        let id_str: &str = id_str;

        let stored_note = if note_id == facts.id() {
            facts.clone()
        } else {
            facts.clone().with_id(note_id)
        };
        let index_data = self.collect_index_data_from_facts(&stored_note);
        let change_kind = if existing_id.is_some() {
            NoteChangeKind::Updated
        } else {
            NoteChangeKind::Created
        };
        let event = Self::build_note_event_from_facts(
            note_id,
            path,
            facts,
            change_kind,
        )?;

        let old_index_data = if existing_id.is_some() {
            let stored =
                self.db.get_owned::<NoteFacts>(STORED_NOTES, id_str)?;
            if let Some(stored) = stored {
                if stored.path() != path {
                    self.ensure_unique_path(path, Some(id_str))?;
                }
                Some(self.collect_index_data_from_facts(&stored))
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

    #[inline]
    fn find_by_alias(
        &self,
        alias: &AliasName,
    ) -> Result<Option<NoteView>, Self::Error> {
        let ids = self.db.multimap_get(ALIAS_TO_ID, alias.as_str())?;

        if let Some(id_str) = ids.first() {
            self.db.get_owned::<NoteFacts>(STORED_NOTES, id_str)
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn find_by_id(&self, id: NoteId) -> Result<Option<NoteView>, Self::Error> {
        let mut id_buffer = Uuid::encode_buffer();
        let id_str =
            Uuid::from(id).as_hyphenated().encode_lower(&mut id_buffer);
        let id_str: &str = id_str;
        self.db.get_owned::<NoteFacts>(STORED_NOTES, id_str)
    }

    #[inline]
    fn find_by_path(
        &self,
        path: &NotePath,
    ) -> Result<Option<NoteView>, Self::Error> {
        let ids = self.db.multimap_get(PATH_TO_ID, path.as_str())?;

        if let Some(id_str) = ids.first() {
            self.db.get_owned::<NoteFacts>(STORED_NOTES, id_str)
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn list(&self) -> Result<Vec<NoteView>, Self::Error> {
        self.db.list_owned::<NoteFacts>(STORED_NOTES)
    }

    #[inline]
    fn list_by_file_class(
        &self,
        class: &FileClassName,
    ) -> Result<Vec<NoteView>, Self::Error> {
        let ids = self.db.multimap_get(FILE_CLASS_TO_ID, class.as_str())?;

        let mut notes = Vec::with_capacity(ids.len());
        for id_str in ids {
            if let Some(note) =
                self.db.get_owned::<NoteFacts>(STORED_NOTES, &id_str)?
            {
                notes.push(note);
            }
        }
        Ok(notes)
    }

    #[inline]
    fn list_by_folder(
        &self,
        folder: &FolderPath,
    ) -> Result<Vec<NoteView>, Self::Error> {
        let ids = self.db.multimap_get(FOLDER_TO_ID, folder.as_str())?;

        let mut notes = Vec::with_capacity(ids.len());
        for id_str in ids {
            if let Some(note) =
                self.db.get_owned::<NoteFacts>(STORED_NOTES, &id_str)?
            {
                notes.push(note);
            }
        }
        Ok(notes)
    }

    #[inline]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "String length arithmetic is safe and will not overflow"
    )]
    fn list_by_frontmatter_kv(
        &self,
        key: &FrontmatterKey,
        value: &str,
    ) -> Result<Vec<NoteView>, Self::Error> {
        let mut combined_key =
            String::with_capacity(key.as_str().len() + value.len() + 1);
        #[expect(
            clippy::let_underscore_must_use,
            reason = "Writing to String is infallible"
        )]
        let _ = write!(&mut combined_key, "{}:{value}", key.as_str());
        self.list_notes_by_index(FRONTMATTER_KV, &combined_key)
    }

    #[inline]
    fn list_by_task_completed_date(
        &self,
        completed_date: TaskTimestamp,
    ) -> Result<Vec<NoteView>, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        let date_str = buffer.format(completed_date.as_i64());
        self.list_notes_by_task_index(TASKS_BY_COMPLETED_DATE, date_str)
    }

    #[inline]
    fn list_by_task_created_date(
        &self,
        created_date: TaskTimestamp,
    ) -> Result<Vec<NoteView>, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        let date_str = buffer.format(created_date.as_i64());
        self.list_notes_by_task_index(TASKS_BY_CREATED_DATE, date_str)
    }

    #[inline]
    fn list_by_task_due_date(
        &self,
        due_date: TaskTimestamp,
    ) -> Result<Vec<NoteView>, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        let date_str = buffer.format(due_date.as_i64());
        self.list_notes_by_task_index(TASKS_BY_DUE_DATE, date_str)
    }

    #[inline]
    fn list_by_task_priority(
        &self,
        priority: TaskPriority,
    ) -> Result<Vec<NoteView>, Self::Error> {
        let value = FieldValue::Number(priority.as_f64());
        let mut keys = Self::metadata_index_keys("priority", &value);
        if let Some(key) = keys.pop() {
            self.list_notes_by_task_index(TASKS_BY_METADATA, key.as_ref())
        } else {
            Ok(Vec::new())
        }
    }

    #[inline]
    fn list_by_task_project(
        &self,
        project: &str,
    ) -> Result<Vec<NoteView>, Self::Error> {
        let value = FieldValue::String(project.into());
        let mut keys = Self::metadata_index_keys("project", &value);
        if let Some(key) = keys.pop() {
            self.list_notes_by_task_index(TASKS_BY_METADATA, key.as_ref())
        } else {
            Ok(Vec::new())
        }
    }

    #[inline]
    fn list_by_task_reminder_date(
        &self,
        reminder_date: TaskTimestamp,
    ) -> Result<Vec<NoteView>, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        let date_str = buffer.format(reminder_date.as_i64());
        self.list_notes_by_task_index(TASKS_BY_REMINDER_DATE, date_str)
    }

    #[inline]
    fn list_by_task_status(
        &self,
        status: &StatusName,
    ) -> Result<Vec<NoteView>, Self::Error> {
        self.list_notes_by_task_index(TASKS_BY_STATUS, status.as_str())
    }

    #[inline]
    fn list_tasks_by_status(
        &self,
        status: &StatusName,
    ) -> Result<Vec<TaskView>, Self::Error> {
        let notes =
            self.list_notes_by_task_index(TASKS_BY_STATUS, status.as_str())?;
        Ok(Self::collect_tasks_matching(&notes, |task| task.status() == status))
    }

    #[inline]
    fn list_tasks_by_date(
        &self,
        kind: TaskDateKind,
        date: TaskTimestamp,
    ) -> Result<Vec<TaskView>, Self::Error> {
        let date_str = Self::format_i64(date.as_i64());
        let table = match kind {
            TaskDateKind::Created => TASKS_BY_CREATED_DATE,
            TaskDateKind::Due => TASKS_BY_DUE_DATE,
            TaskDateKind::Reminder => TASKS_BY_REMINDER_DATE,
            TaskDateKind::Completed => TASKS_BY_COMPLETED_DATE,
        };
        let notes = self.list_notes_by_task_index(table, date_str.as_ref())?;
        Ok(Self::collect_tasks_matching(&notes, |task| match kind {
            TaskDateKind::Created => task.created_at() == Some(date),
            TaskDateKind::Due => task.due_at() == Some(date),
            TaskDateKind::Reminder => task.reminder_at() == Some(date),
            TaskDateKind::Completed => task.completed_at() == Some(date),
        }))
    }

    #[inline]
    fn list_tasks_by_metadata(
        &self,
        field: &str,
        value: &FieldValue,
    ) -> Result<Vec<TaskView>, Self::Error> {
        let mut tasks = Vec::new();
        for key in Self::metadata_index_keys(field, value) {
            let notes =
                self.list_notes_by_task_index(TASKS_BY_METADATA, &key)?;
            tasks.extend(Self::collect_tasks_matching(&notes, |task| {
                task.metadata().get(field) == Some(value)
            }));
        }
        Ok(tasks)
    }

    #[inline]
    fn with_archived_by_id<F, R>(
        &self,
        id: NoteId,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::NoteArchived<'archived>) -> R,
    {
        let mut id_buffer = Uuid::encode_buffer();
        let id_str =
            Uuid::from(id).as_hyphenated().encode_lower(&mut id_buffer);
        let id_str: &str = id_str;
        self.db.get::<NoteFacts, _, R>(STORED_NOTES, id_str, f)
    }
}

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
    status_keys: Vec<Box<str>>,
    created_dates: Vec<Box<str>>,
    due_dates: Vec<Box<str>>,
    reminder_dates: Vec<Box<str>>,
    completed_dates: Vec<Box<str>>,
    metadata_keys: Vec<Box<str>>,
    depends_on: Vec<Box<str>>,
}

/// Arc-backed repository for sharing across threads.
#[derive(Clone)]
pub struct SharedRepository {
    db: Arc<Database>,
    config: Arc<Config>,
}

impl SharedRepository {
    /// Create a new shared repository.
    #[inline]
    #[must_use]
    pub fn new(db: Arc<Database>, config: Arc<Config>) -> Self {
        Self {
            db,
            config,
        }
    }

    /// Access the underlying database.
    #[inline]
    #[must_use]
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// Access the configuration used by this repository.
    #[inline]
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }
}

impl Repository for SharedRepository {
    type Error = DbError;
    type NoteArchived<'archived> = &'archived rkyv::Archived<NoteView>;

    #[inline]
    fn rebuild_note_indexes(&self) -> Result<usize, Self::Error> {
        RedbRepository::new(&self.db, &self.config).rebuild_note_indexes()
    }

    #[inline]
    fn rebuild_task_indexes(&self) -> Result<usize, Self::Error> {
        RedbRepository::new(&self.db, &self.config).rebuild_task_indexes()
    }

    #[inline]
    fn delete_note(&self, id: NoteId) -> Result<(), Self::Error> {
        RedbRepository::new(&self.db, &self.config).delete_note(id)
    }

    #[inline]
    fn save_note_facts(
        &self,
        facts: &NoteFacts,
    ) -> Result<NoteId, Self::Error> {
        RedbRepository::new(&self.db, &self.config).save_note_facts(facts)
    }

    #[inline]
    fn find_by_alias(
        &self,
        alias: &AliasName,
    ) -> Result<Option<NoteView>, Self::Error> {
        RedbRepository::new(&self.db, &self.config).find_by_alias(alias)
    }

    #[inline]
    fn find_by_id(&self, id: NoteId) -> Result<Option<NoteView>, Self::Error> {
        RedbRepository::new(&self.db, &self.config).find_by_id(id)
    }

    #[inline]
    fn find_by_path(
        &self,
        path: &NotePath,
    ) -> Result<Option<NoteView>, Self::Error> {
        RedbRepository::new(&self.db, &self.config).find_by_path(path)
    }

    #[inline]
    fn list(&self) -> Result<Vec<NoteView>, Self::Error> {
        RedbRepository::new(&self.db, &self.config).list()
    }

    #[inline]
    fn list_by_file_class(
        &self,
        class: &FileClassName,
    ) -> Result<Vec<NoteView>, Self::Error> {
        RedbRepository::new(&self.db, &self.config).list_by_file_class(class)
    }

    #[inline]
    fn list_by_folder(
        &self,
        folder: &FolderPath,
    ) -> Result<Vec<NoteView>, Self::Error> {
        RedbRepository::new(&self.db, &self.config).list_by_folder(folder)
    }

    #[inline]
    fn list_by_frontmatter_kv(
        &self,
        key: &FrontmatterKey,
        value: &str,
    ) -> Result<Vec<NoteView>, Self::Error> {
        RedbRepository::new(&self.db, &self.config)
            .list_by_frontmatter_kv(key, value)
    }

    #[inline]
    fn list_by_task_completed_date(
        &self,
        completed_date: TaskTimestamp,
    ) -> Result<Vec<NoteView>, Self::Error> {
        RedbRepository::new(&self.db, &self.config)
            .list_by_task_completed_date(completed_date)
    }

    #[inline]
    fn list_by_task_created_date(
        &self,
        created_date: TaskTimestamp,
    ) -> Result<Vec<NoteView>, Self::Error> {
        RedbRepository::new(&self.db, &self.config)
            .list_by_task_created_date(created_date)
    }

    #[inline]
    fn list_by_task_due_date(
        &self,
        due_date: TaskTimestamp,
    ) -> Result<Vec<NoteView>, Self::Error> {
        RedbRepository::new(&self.db, &self.config)
            .list_by_task_due_date(due_date)
    }

    #[inline]
    fn list_by_task_priority(
        &self,
        priority: TaskPriority,
    ) -> Result<Vec<NoteView>, Self::Error> {
        RedbRepository::new(&self.db, &self.config)
            .list_by_task_priority(priority)
    }

    #[inline]
    fn list_by_task_project(
        &self,
        project: &str,
    ) -> Result<Vec<NoteView>, Self::Error> {
        RedbRepository::new(&self.db, &self.config)
            .list_by_task_project(project)
    }

    #[inline]
    fn list_by_task_reminder_date(
        &self,
        reminder_date: TaskTimestamp,
    ) -> Result<Vec<NoteView>, Self::Error> {
        RedbRepository::new(&self.db, &self.config)
            .list_by_task_reminder_date(reminder_date)
    }

    #[inline]
    fn list_by_task_status(
        &self,
        status: &StatusName,
    ) -> Result<Vec<NoteView>, Self::Error> {
        RedbRepository::new(&self.db, &self.config).list_by_task_status(status)
    }

    #[inline]
    fn list_tasks_by_date(
        &self,
        kind: TaskDateKind,
        date: TaskTimestamp,
    ) -> Result<Vec<TaskView>, Self::Error> {
        RedbRepository::new(&self.db, &self.config)
            .list_tasks_by_date(kind, date)
    }

    #[inline]
    fn list_tasks_by_metadata(
        &self,
        field: &str,
        value: &FieldValue,
    ) -> Result<Vec<TaskView>, Self::Error> {
        RedbRepository::new(&self.db, &self.config)
            .list_tasks_by_metadata(field, value)
    }

    #[inline]
    fn list_tasks_by_status(
        &self,
        status: &StatusName,
    ) -> Result<Vec<TaskView>, Self::Error> {
        RedbRepository::new(&self.db, &self.config).list_tasks_by_status(status)
    }

    #[inline]
    fn with_archived_by_id<F, R>(
        &self,
        id: NoteId,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::NoteArchived<'archived>) -> R,
    {
        RedbRepository::new(&self.db, &self.config).with_archived_by_id(id, f)
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "Tests use assertions in Result-returning functions."
)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::{
        config::{
            aggregate::Config,
            raw::{
                RawConfig, RawDateFieldSpec, RawFieldSpec, RawIndexingConfig,
                RawTaskConfig, RawTaskDates,
            },
            vault::{VaultId, VaultRoot},
        },
        note::{
            aggregate::{NoteFacts, NoteId, RawNoteContext},
            paths::NotePath,
            position::{SourceByteOffset, SourceByteRange},
            raw::{
                frontmatter::RawFrontmatter, list_items::RawTaskKind,
                note::RawNote, tags::RawTag, task_tokens::RawTaskTokens,
                tasks::RawTask,
            },
        },
    };

    fn test_config() -> Result<Config, String> {
        Config::build(
            &RawConfig::default(),
            VaultId::new(),
            VaultRoot::try_new(std::path::PathBuf::from("/vault"))
                .map_err(|e| e.to_string())?,
            crate::config::aggregate::Version::initial(),
        )
        .map_err(|e| e.to_string())
    }

    fn test_config_with_tasks() -> Result<Config, String> {
        let raw = RawConfig {
            task: Some(RawTaskConfig {
                dates: Some(RawTaskDates {
                    created: Some(RawDateFieldSpec {
                        keyword: String::from("created"),
                        emoji: None,
                        format: String::from("%Y-%m-%d"),
                    }),
                    due: Some(RawDateFieldSpec {
                        keyword: String::from("due"),
                        emoji: None,
                        format: String::from("%Y-%m-%d"),
                    }),
                    reminder: Some(RawDateFieldSpec {
                        keyword: String::from("reminder"),
                        emoji: None,
                        format: String::from("%Y-%m-%d"),
                    }),
                    completed: Some(RawDateFieldSpec {
                        keyword: String::from("completed"),
                        emoji: None,
                        format: String::from("%Y-%m-%d"),
                    }),
                    ..RawTaskDates::default()
                }),
                fields: Some(std::collections::HashMap::from([
                    (String::from("priority"), RawFieldSpec::Float {
                        min: None,
                        max: None,
                    }),
                    (String::from("project"), RawFieldSpec::String {
                        pattern: None,
                    }),
                ])),
                indexing: Some(RawIndexingConfig {
                    indexed_fields: Some(vec![
                        String::from("priority"),
                        String::from("project"),
                    ]),
                }),
                ..RawTaskConfig::default()
            }),
            ..RawConfig::default()
        };

        Config::build(
            &raw,
            VaultId::new(),
            VaultRoot::try_new(std::path::PathBuf::from("/vault"))
                .map_err(|e| e.to_string())?,
            crate::config::aggregate::Version::initial(),
        )
        .map_err(|e| e.to_string())
    }

    fn raw_note(path: NotePath) -> RawNote {
        RawNote::new(
            path,
            "hash".into(),
            4,
            None,
            None,
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    }

    fn raw_note_with_indexes(path: NotePath) -> RawNote {
        let frontmatter = RawFrontmatter::new(
            crate::note::frontmatter::FrontmatterFormat::Yaml,
            "aliases:\n  - Alias\nfile_class: Class\ncategory: docs\n".into(),
            SourceByteRange::new(
                SourceByteOffset::new(0),
                SourceByteOffset::new(0),
            )
            .expect("frontmatter range"),
        );
        let tags = vec![RawTag::new("#tag".into(), SourceByteOffset::new(0))];
        let raw_task_text = "#task Do work [priority:: 2] [project:: lithos] \
                             [created:: 2024-01-01] [due:: 2024-01-02] \
                             [reminder:: 2024-01-03] [completed:: 2024-01-04]";
        let tokens = RawTaskTokens::parse(raw_task_text, &[]);
        let tasks = vec![RawTask::new(
            RawTaskKind::Unchecked(' '),
            raw_task_text.into(),
            vec!["#task".into()],
            tokens.inline_fields().to_vec(),
            tokens.emoji_dates().to_vec(),
            SourceByteOffset::new(0),
        )];

        RawNote::new(
            path,
            "hash".into(),
            4,
            None,
            None,
            Some(frontmatter),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            tags,
            Vec::new(),
            tasks,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    }

    #[test]
    fn save_note_facts_persists_path() -> Result<(), DbError> {
        let dir = tempdir().map_err(|e| DbError::Table(e.to_string()))?;
        let db_path = dir.path().join("notes.redb");
        let db = Database::open(&db_path)?;
        let config = test_config().map_err(DbError::Table)?;
        let repo = RedbRepository::new(&db, &config);

        let path = NotePath::try_new("notes/a.md")
            .map_err(|e| DbError::Table(e.to_string()))?;
        let raw = raw_note(path.clone());
        let facts = NoteFacts::try_from(RawNoteContext::new(
            NoteId::new(),
            &raw,
            &config,
        ))
        .map_err(|e| DbError::Table(e.to_string()))?;

        let note_id = repo.save_note_facts(&facts)?;
        let stored =
            repo.find_by_path(&path)?.expect("stored note should exist");
        assert_eq!(stored.id(), note_id);
        assert_eq!(stored.path().as_str(), "notes/a.md");
        Ok(())
    }

    #[test]
    fn delete_note_removes_note() -> Result<(), DbError> {
        let dir = tempdir().map_err(|e| DbError::Table(e.to_string()))?;
        let db_path = dir.path().join("notes.redb");
        let db = Database::open(&db_path)?;
        let config = test_config().map_err(DbError::Table)?;
        let repo = RedbRepository::new(&db, &config);

        let path = NotePath::try_new("notes/a.md")
            .map_err(|e| DbError::Table(e.to_string()))?;
        let raw = raw_note(path.clone());
        let facts = NoteFacts::try_from(RawNoteContext::new(
            NoteId::new(),
            &raw,
            &config,
        ))
        .map_err(|e| DbError::Table(e.to_string()))?;
        let note_id = repo.save_note_facts(&facts)?;

        repo.delete_note(note_id)?;
        let stored = repo.find_by_id(note_id)?;
        assert!(stored.is_none());
        Ok(())
    }

    #[test]
    fn indexed_queries_return_expected_note() -> Result<(), DbError> {
        let dir = tempdir().map_err(|e| DbError::Table(e.to_string()))?;
        let db_path = dir.path().join("notes.redb");
        let db = Database::open(&db_path)?;
        let config = test_config_with_tasks().map_err(DbError::Table)?;
        let repo = RedbRepository::new(&db, &config);

        let path = NotePath::try_new("notes/a.md")
            .map_err(|e| DbError::Table(e.to_string()))?;
        let raw = raw_note_with_indexes(path.clone());
        let facts = NoteFacts::try_from(RawNoteContext::new(
            NoteId::new(),
            &raw,
            &config,
        ))
        .map_err(|e| DbError::Table(e.to_string()))?;
        let note_id = repo.save_note_facts(&facts)?;

        let alias = AliasName::try_new("Alias")
            .map_err(|e| DbError::Table(e.to_string()))?;
        let by_alias = repo.find_by_alias(&alias)?.expect("alias should match");
        assert_eq!(by_alias.id(), note_id);

        let class = FileClassName::try_new("Class")
            .map_err(|e| DbError::Table(e.to_string()))?;
        let by_class = repo.list_by_file_class(&class)?;
        assert!(by_class.iter().any(|note| note.id() == note_id));

        let folder = FolderPath::try_new("notes")
            .map_err(|e| DbError::Table(e.to_string()))?;
        let by_folder = repo.list_by_folder(&folder)?;
        assert!(by_folder.iter().any(|note| note.id() == note_id));

        let key =
            crate::config::frontmatter::FrontmatterKey::try_new("category")
                .map_err(|e| DbError::Table(e.to_string()))?;
        let by_frontmatter = repo.list_by_frontmatter_kv(&key, "docs")?;
        assert!(by_frontmatter.iter().any(|note| note.id() == note_id));

        let status = config
            .task()
            .status()
            .name_for_symbol(
                crate::config::task::StatusSymbol::try_new(' ')
                    .map_err(|e| DbError::Table(e.to_string()))?,
            )
            .ok_or_else(|| DbError::Table("missing status".into()))?
            .clone();
        let by_status = repo.list_by_task_status(&status)?;
        assert!(by_status.iter().any(|note| note.id() == note_id));

        let priority = TaskPriority::try_new(2.0)
            .map_err(|e| DbError::Table(e.to_string()))?;
        let by_priority = repo.list_by_task_priority(priority)?;
        assert!(by_priority.iter().any(|note| note.id() == note_id));

        let by_project = repo.list_by_task_project("lithos")?;
        assert!(by_project.iter().any(|note| note.id() == note_id));

        let by_created =
            repo.list_by_task_created_date(TaskTimestamp::new(1_704_067_200))?;
        assert!(by_created.iter().any(|note| note.id() == note_id));

        let by_due =
            repo.list_by_task_due_date(TaskTimestamp::new(1_704_153_600))?;
        assert!(by_due.iter().any(|note| note.id() == note_id));

        let by_reminder =
            repo.list_by_task_reminder_date(TaskTimestamp::new(1_704_240_000))?;
        assert!(by_reminder.iter().any(|note| note.id() == note_id));

        let by_completed = repo
            .list_by_task_completed_date(TaskTimestamp::new(1_704_326_400))?;
        assert!(by_completed.iter().any(|note| note.id() == note_id));

        let tasks = repo.list_tasks_by_status(&status)?;
        assert_eq!(tasks.len(), 1);
        Ok(())
    }
}
