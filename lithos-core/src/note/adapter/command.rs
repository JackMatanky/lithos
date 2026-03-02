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
        aggregate::{Note, NoteId},
        db_table::{
            ALIAS_TO_ID, FILE_CLASS_TO_ID, FOLDER_TO_ID, FRONTMATTER_KV, NOTES,
            PATH_TO_ID, TAGS_TO_NOTES, TASKS_BY_COMPLETED_DATE,
            TASKS_BY_CREATED_DATE, TASKS_BY_DUE_DATE, TASKS_BY_PRIORITY,
            TASKS_BY_PROJECT, TASKS_BY_REMINDER_DATE, TASKS_BY_STATUS,
        },
        error::NoteError,
        frontmatter::Frontmatter,
        paths::NotePath,
        ports::Command,
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
    completed_dates: Vec<i64>,
    created_dates: Vec<i64>,
    due_dates: Vec<i64>,
    reminder_dates: Vec<i64>,
    statuses: Vec<Box<str>>,
    priorities: Vec<f64>,
    projects: Vec<Box<str>>,
}

/// Command implementation for Note write operations.
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
    fn get_note_index_data(
        &self,
        id_str: &str,
    ) -> Result<Option<IndexData>, DbError> {
        let note = self.db.get_owned::<Note>(NOTES, id_str)?;
        Ok(note.as_ref().map(|note| self.collect_index_data(note)))
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

    fn collect_index_data(&self, note: &Note) -> IndexData {
        let frontmatter = note.frontmatter();
        let aliases = frontmatter
            .map(|fm| fm.aliases(self.config).map(Into::into).collect())
            .unwrap_or_default();
        let file_class = frontmatter
            .and_then(|fm| fm.file_class(self.config))
            .map(Into::into);
        let frontmatter_entries =
            frontmatter.map(Self::frontmatter_entries).unwrap_or_default();

        IndexData {
            path: note.path().as_str().into(),
            folder: Self::note_folder(note.path().as_str()),
            tags: note.tags().map(|tag| tag.full_path().into()).collect(),
            aliases,
            file_class,
            task_indexes: Self::task_indexes(note),
            frontmatter_entries,
        }
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

    fn task_indexes(note: &Note) -> TaskIndexData {
        let mut data = TaskIndexData::default();

        for task in note.tasks() {
            data.statuses.push(task.status().as_str().into());

            if let Some(timestamp) = task.created_at() {
                data.created_dates.push(timestamp.as_i64());
            }
            if let Some(timestamp) = task.due_at() {
                data.due_dates.push(timestamp.as_i64());
            }
            if let Some(timestamp) = task.reminder_at() {
                data.reminder_dates.push(timestamp.as_i64());
            }
            if let Some(timestamp) = task.completed_at() {
                data.completed_dates.push(timestamp.as_i64());
            }

            if let Some(priority) = task.metadata().get_number("priority") {
                data.priorities.push(priority);
            }
            if let Some(project) = task.metadata().get_string("project") {
                data.projects.push(project.into());
            }
        }

        data
    }

    fn insert_task_indexes(
        batch: &mut BatchWriter,
        index_data: &TaskIndexData,
        id_str: &str,
    ) -> Result<(), DbError> {
        let mut itoa_buffer = itoa::Buffer::new();
        let mut ryu_buffer = ryu::Buffer::new();

        for status in &index_data.statuses {
            batch.multimap_insert(TASKS_BY_STATUS, status.as_ref(), id_str)?;
        }
        for date in &index_data.created_dates {
            batch.multimap_insert(
                TASKS_BY_CREATED_DATE,
                itoa_buffer.format(*date),
                id_str,
            )?;
        }
        for date in &index_data.due_dates {
            batch.multimap_insert(
                TASKS_BY_DUE_DATE,
                itoa_buffer.format(*date),
                id_str,
            )?;
        }
        for date in &index_data.reminder_dates {
            batch.multimap_insert(
                TASKS_BY_REMINDER_DATE,
                itoa_buffer.format(*date),
                id_str,
            )?;
        }
        for date in &index_data.completed_dates {
            batch.multimap_insert(
                TASKS_BY_COMPLETED_DATE,
                itoa_buffer.format(*date),
                id_str,
            )?;
        }
        for priority in &index_data.priorities {
            batch.multimap_insert(
                TASKS_BY_PRIORITY,
                ryu_buffer.format(*priority),
                id_str,
            )?;
        }
        for project in &index_data.projects {
            batch.multimap_insert(
                TASKS_BY_PROJECT,
                project.as_ref(),
                id_str,
            )?;
        }
        Ok(())
    }

    fn remove_task_indexes(
        batch: &mut BatchWriter,
        index_data: &TaskIndexData,
        id_str: &str,
    ) -> Result<(), DbError> {
        let mut itoa_buffer = itoa::Buffer::new();
        let mut ryu_buffer = ryu::Buffer::new();

        for status in &index_data.statuses {
            batch.multimap_remove(TASKS_BY_STATUS, status.as_ref(), id_str)?;
        }
        for date in &index_data.created_dates {
            batch.multimap_remove(
                TASKS_BY_CREATED_DATE,
                itoa_buffer.format(*date),
                id_str,
            )?;
        }
        for date in &index_data.due_dates {
            batch.multimap_remove(
                TASKS_BY_DUE_DATE,
                itoa_buffer.format(*date),
                id_str,
            )?;
        }
        for date in &index_data.reminder_dates {
            batch.multimap_remove(
                TASKS_BY_REMINDER_DATE,
                itoa_buffer.format(*date),
                id_str,
            )?;
        }
        for date in &index_data.completed_dates {
            batch.multimap_remove(
                TASKS_BY_COMPLETED_DATE,
                itoa_buffer.format(*date),
                id_str,
            )?;
        }
        for priority in &index_data.priorities {
            batch.multimap_remove(
                TASKS_BY_PRIORITY,
                ryu_buffer.format(*priority),
                id_str,
            )?;
        }
        for project in &index_data.projects {
            batch.multimap_remove(
                TASKS_BY_PROJECT,
                project.as_ref(),
                id_str,
            )?;
        }
        Ok(())
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
        let note = Note::new(NoteId::new(), path.as_str())
            .map_err(|e| crate::db::DbError::Table(e.to_string()))?;
        self.ensure_unique_path(note.path(), None)?;
        let id = Uuid::from(note.id());
        let mut id_buffer = Uuid::encode_buffer();
        let id_str = id.as_hyphenated().encode_lower(&mut id_buffer);
        let id_str: &str = id_str;
        let index_data = self.collect_index_data(&note);

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
        let old_data = self.get_note_index_data(id_str)?;

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
        let old_data = self.get_note_index_data(id_str)?;
        let current_id = Some(id_str);
        if old_data.as_ref().is_none_or(|index_data| {
            index_data.path.as_ref() != note.path().as_str()
        }) {
            self.ensure_unique_path(note.path(), current_id)?;
        }
        let index_data = self.collect_index_data(&note);

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
            NotePath::new(path).map_err(|e| e.to_string())
        }

        pub fn parse_tag(tag: &str) -> Result<Tag, String> {
            Tag::from_token(tag).map_err(|e| e.to_string())
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
