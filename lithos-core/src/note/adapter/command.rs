//! Note command implementations (CQRS write operations).
//!
//! This module implements the Command port trait for Note write operations,
//! using the Database layer for persistence.

use std::path::Path;

use uuid::Uuid;

use crate::{
    config::aggregate::Config,
    db::{BatchWriter, Database, DbError},
    note::{
        adapter::stored::{StoredTask, metadata_index_keys},
        aggregate::{Note, NoteId},
        db_table::{
            ALIAS_TO_ID, FILE_CLASS_TO_ID, FOLDER_TO_ID, FRONTMATTER_KV, NOTES,
            PATH_TO_ID, TAGS_TO_NOTES, TASKS, TASKS_BY_COMPLETED_DATE,
            TASKS_BY_CREATED_DATE, TASKS_BY_DEPENDS_ON, TASKS_BY_DUE_DATE,
            TASKS_BY_METADATA, TASKS_BY_REMINDER_DATE, TASKS_BY_STATUS,
        },
        error::NoteError,
        frontmatter::Frontmatter,
        paths::NotePath,
        ports::Command,
        position::SourceByteOffset,
        structure::Heading,
        task::{TaskMetadata, TaskText},
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

#[derive(Debug)]
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

    /// Helper: Extract index data from stored note for cleanup.
    fn find_note_index_data(
        &self,
        id_str: &str,
    ) -> Result<Option<IndexData>, DbError> {
        let note = self.db.get_owned::<Note>(NOTES, id_str)?;
        note.as_ref().map(|note| self.collect_index_data(note)).transpose()
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

    fn collect_index_data(&self, note: &Note) -> Result<IndexData, DbError> {
        let frontmatter = note.frontmatter();
        let aliases = frontmatter
            .map(|fm| fm.aliases(self.config).map(Into::into).collect())
            .unwrap_or_default();
        let file_class = frontmatter
            .and_then(|fm| fm.file_class(self.config))
            .map(Into::into);
        let frontmatter_entries =
            frontmatter.map(Self::frontmatter_entries).unwrap_or_default();

        Ok(IndexData {
            path: note.path().as_str().into(),
            folder: Self::note_folder(note.path().as_str()),
            tags: note.tags().map(|tag| tag.full_path().into()).collect(),
            aliases,
            file_class,
            task_indexes: Self::task_indexes(note, self.config)?,
            frontmatter_entries,
        })
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

    fn task_indexes(
        note: &Note,
        config: &Config,
    ) -> Result<TaskIndexData, DbError> {
        let tasks = Self::task_index_entries(note, config)?;

        Ok(TaskIndexData {
            tasks,
            dependencies_enabled: config.task().dependencies_enabled(),
        })
    }

    fn task_index_entries(
        note: &Note,
        config: &Config,
    ) -> Result<Vec<TaskIndexEntry>, DbError> {
        let mut entries = Vec::new();
        let status_config = config.task().status();
        let index_all_fields = config.task().indexed().is_empty();
        let dependencies_enabled = config.task().dependencies_enabled();

        for task in note.tasks() {
            let status_symbol = status_config
                .symbol_for_name(task.status())
                .ok_or_else(|| {
                    DbError::Table(
                        "task status should have a symbol mapping".into(),
                    )
                })?;
            let heading = Self::task_heading(note, task.position());
            let metadata = task.metadata().clone();
            let metadata_keys =
                Self::task_metadata_keys(&metadata, config, index_all_fields);
            let depends_on =
                Self::task_depends_on(&metadata, dependencies_enabled);

            let text = TaskText::try_new(task.text())
                .map_err(|err| DbError::Table(err.to_string()))?;
            let stored = StoredTask::new(
                task.id(),
                note.id(),
                note.path().clone(),
                heading,
                task.position(),
                None,
                task.status().clone(),
                status_symbol,
                "unknown".into(),
                text,
                task.tags().cloned().collect(),
                metadata,
                task.schedule().clone(),
                None,
                None,
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

    fn task_heading(
        note: &Note,
        position: SourceByteOffset,
    ) -> Option<Heading> {
        for section in note.sections() {
            let range = section.range();
            if position >= range.start() && position < range.end() {
                return section.heading().cloned();
            }
        }
        None
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

    /// Creates a new note with the given vault-relative path.
    #[inline]
    fn create(&self, path: &NotePath) -> Result<Note, Self::Error> {
        let note = Note::try_new(NoteId::new(), path.as_str())
            .map_err(|e| crate::db::DbError::Table(e.to_string()))?;
        self.ensure_unique_path(note.path(), None)?;
        let id = Uuid::from(note.id());
        let mut id_buffer = Uuid::encode_buffer();
        let id_str = id.as_hyphenated().encode_lower(&mut id_buffer);
        let id_str: &str = id_str;
        let index_data = self.collect_index_data(&note)?;

        self.db.batch_write(|batch| {
            batch.put(NOTES, id_str, &note)?;
            Self::insert_indexes(batch, &index_data, id_str)?;
            Ok(())
        })?;

        Ok(note)
    }

    /// Deletes a note by ID.
    #[inline]
    fn delete(&self, id: NoteId) -> Result<(), Self::Error> {
        let uuid = Uuid::from(id);
        let mut id_buffer = Uuid::encode_buffer();
        let id_str = uuid.as_hyphenated().encode_lower(&mut id_buffer);
        let id_str: &str = id_str;
        let old_data = self.find_note_index_data(id_str)?;

        if let Some(index_data) = old_data {
            self.db.batch_write(|batch| {
                Self::remove_indexes(batch, &index_data, id_str)?;
                batch.delete(NOTES, id_str)?;
                Ok(())
            })?;
        }

        Ok(())
    }

    /// Updates an existing note.
    #[inline]
    fn update(&self, note: Note) -> Result<Note, Self::Error> {
        let id = Uuid::from(note.id());
        let mut id_buffer = Uuid::encode_buffer();
        let id_str = id.as_hyphenated().encode_lower(&mut id_buffer);
        let id_str: &str = id_str;
        let old_data = self.find_note_index_data(id_str)?;
        let current_id = Some(id_str);
        if old_data.as_ref().is_none_or(|index_data| {
            index_data.path.as_ref() != note.path().as_str()
        }) {
            self.ensure_unique_path(note.path(), current_id)?;
        }
        let index_data = self.collect_index_data(&note)?;

        self.db.batch_write(|batch| {
            if let Some(old_index_data) = old_data {
                Self::remove_indexes(batch, &old_index_data, id_str)?;
            }

            Self::insert_indexes(batch, &index_data, id_str)?;
            batch.put(NOTES, id_str, &note)?;
            Ok(())
        })?;

        Ok(note)
    }

    #[inline]
    fn rebuild_task_indexes(&self) -> Result<usize, Self::Error> {
        let stored_tasks = self.db.list_owned::<StoredTask>(TASKS)?;
        let notes = self.db.list_owned::<Note>(NOTES)?;

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

        let mut rebuilt_entries = Vec::new();
        for note in &notes {
            rebuilt_entries
                .extend(Self::task_index_entries(note, self.config)?);
        }
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
    use crate::note::error::{NoteCommandError, NoteError};

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
            )
            .map_err(|e| e.to_string())
        }

        pub fn create_note(
            cmd: &CommandAdapter<'_, '_>,
            path: &NotePath,
        ) -> Result<Note, NoteCommandError> {
            Ok(Command::create(cmd, path)?)
        }

        pub fn update_note(
            cmd: &CommandAdapter<'_, '_>,
            note: Note,
        ) -> Result<Note, NoteCommandError> {
            Ok(Command::update(cmd, note)?)
        }

        pub fn delete_note(
            cmd: &CommandAdapter<'_, '_>,
            id: NoteId,
        ) -> Result<(), NoteCommandError> {
            Ok(Command::delete(cmd, id)?)
        }

        pub fn parse_path(path: &str) -> Result<NotePath, String> {
            NotePath::try_new(path).map_err(|e| e.to_string())
        }

        pub fn parse_tag(tag: &str) -> Result<Tag, String> {
            Tag::try_from_token(tag).map_err(|e| e.to_string())
        }

        pub fn stored_note(
            db: &Database,
            id: Uuid,
        ) -> Result<Option<Note>, String> {
            db.get_owned::<Note>(NOTES, &id.to_string())
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
    }

    mod persistence {
        use super::*;

        #[test]
        fn create_persists_note_path() -> Result<(), NoteCommandError> {
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
            let note = fixtures::create_note(&cmd, &path)?;
            let id = Uuid::from(note.id());

            let stored_note = fixtures::stored_note(&db, id)
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
        fn create_persists_path_index() -> Result<(), NoteCommandError> {
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
            let note = fixtures::create_note(&cmd, &path)?;
            let id = Uuid::from(note.id());

            let ids = fixtures::path_index_ids(&db, note.path().as_str())
                .map_err(|e| {
                    NoteCommandError::Domain(NoteError::Storage(e.into()))
                })?;
            assert!(
                ids.contains(&id.to_string()),
                "Path index should contain created note id"
            );
            Ok(())
        }

        #[test]
        fn update_removes_old_path_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let config = fixtures::test_config().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let cmd = CommandAdapter::new(&db, &config);

            let path_a = fixtures::parse_path("notes/a.md").map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let mut note = fixtures::create_note(&cmd, &path_a)?;
            let id = Uuid::from(note.id());
            let old_path = note.path().as_str().to_owned();
            let path_b = fixtures::parse_path("notes/b.md").map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            note.set_path(path_b);

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let old_ids =
                fixtures::path_index_ids(&db, &old_path).map_err(|e| {
                    NoteCommandError::Domain(NoteError::Storage(e.into()))
                })?;
            assert!(
                !old_ids.contains(&id.to_string()),
                "Old path index should not contain updated note id"
            );
            Ok(())
        }

        #[test]
        fn update_adds_new_path_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let config = fixtures::test_config().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let cmd = CommandAdapter::new(&db, &config);

            let path_a = fixtures::parse_path("notes/a.md").map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let mut note = fixtures::create_note(&cmd, &path_a)?;
            let id = Uuid::from(note.id());
            let path_b = fixtures::parse_path("notes/b.md").map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            note.set_path(path_b);

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let new_ids =
                fixtures::path_index_ids(&db, "notes/b.md").map_err(|e| {
                    NoteCommandError::Domain(NoteError::Storage(e.into()))
                })?;
            assert!(
                new_ids.contains(&id.to_string()),
                "New path index should contain updated note id"
            );
            Ok(())
        }

        #[test]
        fn update_adds_tag_index() -> Result<(), NoteCommandError> {
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
            let mut note = fixtures::create_note(&cmd, &path)?;
            let id = Uuid::from(note.id());
            let tag = fixtures::parse_tag("#project").map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let tag_key = tag.full_path().to_owned();
            note.add_tag(tag);

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

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
            let note = fixtures::create_note(&cmd, &path)?;
            let id = Uuid::from(note.id());

            let result = fixtures::delete_note(&cmd, note.id());
            assert!(result.is_ok(), "Delete should succeed: {result:?}");

            let stored = fixtures::stored_note(&db, id).map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            assert!(stored.is_none(), "Deleted note should not exist");
            Ok(())
        }

        #[test]
        fn create_rejects_duplicate_paths() -> Result<(), NoteCommandError> {
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
            let _note = fixtures::create_note(&cmd, &path)?;
            let duplicate = fixtures::create_note(&cmd, &path);

            assert!(duplicate.is_err(), "duplicate path should be rejected");
            Ok(())
        }

        #[test]
        fn update_rejects_duplicate_paths() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let config = fixtures::test_config().map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let cmd = CommandAdapter::new(&db, &config);

            let path_a = fixtures::parse_path("notes/a.md").map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let path_b = fixtures::parse_path("notes/b.md").map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            let note_a = fixtures::create_note(&cmd, &path_a)?;
            let mut note_b = fixtures::create_note(&cmd, &path_b)?;

            let path = fixtures::parse_path("notes/a.md").map_err(|e| {
                NoteCommandError::Domain(NoteError::Storage(e.into()))
            })?;
            note_b.set_path(path);
            let updated = fixtures::update_note(&cmd, note_b);

            assert!(
                updated.is_err(),
                "duplicate path updates should be rejected"
            );

            let stored = fixtures::stored_note(&db, Uuid::from(note_a.id()))
                .map_err(|e| {
                    NoteCommandError::Domain(NoteError::Storage(e.into()))
                })?
                .expect("stored note should exist");
            assert_eq!(stored.path().as_str(), "notes/a.md");
            Ok(())
        }
    }
}
