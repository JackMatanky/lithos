//! Concrete implementation of the [`crate::note::ports::Query`] trait.
//!
//! Provides high-performance, zero-copy read operations for notes and tasks,
//! utilizing specialized database indexes.

use std::sync::Arc;

use uuid::Uuid;

use crate::{
    config::{frontmatter::FrontmatterKey, task::StatusName},
    db::Database,
    note::{
        adapter::stored::{StoredNote, StoredTask, metadata_index_keys},
        aggregate::{AliasName, FileClassName, NoteId},
        db_table::{
            ALIAS_TO_ID, FILE_CLASS_TO_ID, FOLDER_TO_ID, FRONTMATTER_KV,
            PATH_TO_ID, STORED_NOTES, TASKS, TASKS_BY_COMPLETED_DATE,
            TASKS_BY_CREATED_DATE, TASKS_BY_DUE_DATE, TASKS_BY_METADATA,
            TASKS_BY_REMINDER_DATE, TASKS_BY_STATUS,
        },
        paths::{FolderPath, NotePath},
        ports::Query,
        task::{TaskDateKind, TaskPriority, TaskTimestamp},
        value::FieldValue,
    },
};

/// Query implementation for Note read operations.
///
/// Provides indexed lookups over the note database with zero-copy access to
/// archived values where supported by the storage layer.
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
///
/// use lithos_core::{db::Database, note::adapter::query::QueryAdapter};
///
/// let root = std::env::temp_dir()
///     .join(format!("lithos_query_doc_{}", std::process::id()));
/// std::fs::create_dir_all(&root)?;
/// let db_path = root.join("notes.redb");
/// let db = Arc::new(Database::open(&db_path)?);
/// let adapter = QueryAdapter::new(Arc::clone(&db));
/// # let _ = adapter;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct QueryAdapter {
    db: Arc<Database>,
}

impl QueryAdapter {
    /// Create a new `QueryAdapter` with a database reference.
    #[inline]
    #[must_use]
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
        }
    }

    /// Helper method to find notes by any task index.
    fn list_notes_by_task_index(
        &self,
        index_table: redb::MultimapTableDefinition<&str, &str>,
        index_key: &str,
    ) -> Result<Vec<StoredNote>, crate::db::DbError> {
        use std::collections::BTreeSet;

        let task_refs = self.db.multimap_get(index_table, index_key)?;
        let mut note_ids = BTreeSet::new();

        for task_id_str in task_refs {
            if let Some(stored) =
                self.db.get_owned::<StoredTask>(TASKS, &task_id_str)?
            {
                note_ids.insert(stored.note_id());
            }
        }

        let mut notes = Vec::with_capacity(note_ids.len());
        for note_id in note_ids {
            let mut id_buffer = Uuid::encode_buffer();
            let id_str = Uuid::from(note_id)
                .as_hyphenated()
                .encode_lower(&mut id_buffer);
            if let Some(note) =
                self.db.get_owned::<StoredNote>(STORED_NOTES, id_str)?
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
    ) -> Result<Vec<StoredNote>, crate::db::DbError> {
        let note_refs = self.db.multimap_get(index_table, index_key)?;
        let mut notes = Vec::with_capacity(note_refs.len());
        for note_id_str in note_refs {
            if let Some(note) =
                self.db.get_owned::<StoredNote>(STORED_NOTES, &note_id_str)?
            {
                notes.push(note);
            }
        }
        Ok(notes)
    }

    fn list_tasks_by_index(
        &self,
        index_table: redb::MultimapTableDefinition<&str, &str>,
        index_key: &str,
    ) -> Result<Vec<StoredTask>, crate::db::DbError> {
        let task_refs = self.db.multimap_get(index_table, index_key)?;
        let mut tasks = Vec::with_capacity(task_refs.len());

        for task_id_str in task_refs {
            if let Some(task) =
                self.db.get_owned::<StoredTask>(TASKS, &task_id_str)?
            {
                tasks.push(task);
            }
        }
        Ok(tasks)
    }

    fn list_tasks_by_metadata_index(
        &self,
        field: &str,
        value: &FieldValue,
    ) -> Result<Vec<StoredTask>, crate::db::DbError> {
        let keys = metadata_index_keys(field, value);
        let mut tasks = Vec::new();
        for key in keys {
            tasks.extend(
                self.list_tasks_by_index(TASKS_BY_METADATA, key.as_ref())?,
            );
        }
        Ok(tasks)
    }
}

impl Query for QueryAdapter {
    type Error = crate::db::DbError;
    type NoteArchived<'archived>
        = &'archived rkyv::Archived<StoredNote>
    where
        Self: 'archived;

    #[inline]
    fn find_by_alias(
        &self,
        alias: &AliasName,
    ) -> Result<Option<StoredNote>, Self::Error> {
        let ids = self.db.multimap_get(ALIAS_TO_ID, alias.as_str())?;

        if let Some(id_str) = ids.first() {
            self.db.get_owned::<StoredNote>(STORED_NOTES, id_str)
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn list_by_file_class(
        &self,
        class: &FileClassName,
    ) -> Result<Vec<StoredNote>, Self::Error> {
        let ids = self.db.multimap_get(FILE_CLASS_TO_ID, class.as_str())?;

        let mut notes = Vec::with_capacity(ids.len());
        for id_str in ids {
            if let Some(note) =
                self.db.get_owned::<StoredNote>(STORED_NOTES, &id_str)?
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
    ) -> Result<Vec<StoredNote>, Self::Error> {
        let ids = self.db.multimap_get(FOLDER_TO_ID, folder.as_str())?;

        let mut notes = Vec::with_capacity(ids.len());
        for id_str in ids {
            if let Some(note) =
                self.db.get_owned::<StoredNote>(STORED_NOTES, &id_str)?
            {
                notes.push(note);
            }
        }
        Ok(notes)
    }

    #[inline]
    fn find_by_id(
        &self,
        id: NoteId,
    ) -> Result<Option<StoredNote>, Self::Error> {
        let mut id_buffer = Uuid::encode_buffer();
        let id_str =
            Uuid::from(id).as_hyphenated().encode_lower(&mut id_buffer);
        let id_str: &str = id_str;
        self.db.get_owned::<StoredNote>(STORED_NOTES, id_str)
    }

    #[inline]
    fn find_by_path(
        &self,
        path: &NotePath,
    ) -> Result<Option<StoredNote>, Self::Error> {
        let ids = self.db.multimap_get(PATH_TO_ID, path.as_str())?;

        if let Some(id_str) = ids.first() {
            self.db.get_owned::<StoredNote>(STORED_NOTES, id_str)
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn list_by_task_completed_date(
        &self,
        completed_date: TaskTimestamp,
    ) -> Result<Vec<StoredNote>, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        let date_str = buffer.format(completed_date.as_i64());
        self.list_notes_by_task_index(TASKS_BY_COMPLETED_DATE, date_str)
    }

    #[inline]
    fn list_by_task_created_date(
        &self,
        created_date: TaskTimestamp,
    ) -> Result<Vec<StoredNote>, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        let date_str = buffer.format(created_date.as_i64());
        self.list_notes_by_task_index(TASKS_BY_CREATED_DATE, date_str)
    }

    #[inline]
    fn list_by_task_due_date(
        &self,
        due_date: TaskTimestamp,
    ) -> Result<Vec<StoredNote>, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        let date_str = buffer.format(due_date.as_i64());
        self.list_notes_by_task_index(TASKS_BY_DUE_DATE, date_str)
    }

    #[inline]
    fn list_by_task_priority(
        &self,
        priority: TaskPriority,
    ) -> Result<Vec<StoredNote>, Self::Error> {
        let value = FieldValue::Number(priority.as_f64());
        let mut keys = metadata_index_keys("priority", &value);
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
    ) -> Result<Vec<StoredNote>, Self::Error> {
        let value = FieldValue::String(project.into());
        let mut keys = metadata_index_keys("project", &value);
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
    ) -> Result<Vec<StoredNote>, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        let date_str = buffer.format(reminder_date.as_i64());
        self.list_notes_by_task_index(TASKS_BY_REMINDER_DATE, date_str)
    }

    #[inline]
    fn list_by_task_status(
        &self,
        status: &StatusName,
    ) -> Result<Vec<StoredNote>, Self::Error> {
        self.list_notes_by_task_index(TASKS_BY_STATUS, status.as_str())
    }

    #[inline]
    fn list_tasks_by_status(
        &self,
        status: &StatusName,
    ) -> Result<Vec<StoredTask>, Self::Error> {
        self.list_tasks_by_index(TASKS_BY_STATUS, status.as_str())
    }

    #[inline]
    fn list_tasks_by_date(
        &self,
        kind: TaskDateKind,
        date: TaskTimestamp,
    ) -> Result<Vec<StoredTask>, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        let date_str = buffer.format(date.as_i64());
        let table = match kind {
            TaskDateKind::Created => TASKS_BY_CREATED_DATE,
            TaskDateKind::Due => TASKS_BY_DUE_DATE,
            TaskDateKind::Reminder => TASKS_BY_REMINDER_DATE,
            TaskDateKind::Completed => TASKS_BY_COMPLETED_DATE,
        };
        self.list_tasks_by_index(table, date_str)
    }

    #[inline]
    fn list_tasks_by_metadata(
        &self,
        field: &str,
        value: &FieldValue,
    ) -> Result<Vec<StoredTask>, Self::Error> {
        self.list_tasks_by_metadata_index(field, value)
    }

    #[inline]
    fn list(&self) -> Result<Vec<StoredNote>, Self::Error> {
        self.db.list_owned::<StoredNote>(STORED_NOTES)
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
    ) -> Result<Vec<StoredNote>, Self::Error> {
        use std::fmt::Write as _;
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
        self.db.get::<StoredNote, _, R>(STORED_NOTES, id_str, f)
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
    use crate::note::error::{NoteError, NoteQueryError};

    mod fixtures {
        use std::collections::HashMap;

        use super::*;
        use crate::{
            config::{
                aggregate::Config,
                raw::RawConfig,
                task::{StatusName, StatusSymbol},
                vault::{VaultId, VaultRoot},
            },
            note::{
                adapter::command::CommandAdapter,
                aggregate::NoteId,
                frontmatter::Frontmatter,
                paths::NotePath,
                ports::Command,
                position::SourceByteOffset,
                task::{Task, TaskAttributes, TaskMetadata, TaskTimestamp},
                value::FieldValue,
            },
        };
        type QuerySetupResult =
            Result<(TempDir, Database, NoteId, Frontmatter), String>;
        type IndexedNoteSetup =
            Result<(TempDir, Database, NoteId, StatusName), String>;

        pub const TEST_MISSING_ID: Uuid =
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0901);

        pub fn test_db() -> Result<(TempDir, Database), String> {
            let dir = tempdir().map_err(|e| e.to_string())?;
            let path = dir.path().join("test.redb");
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

        pub fn complex_frontmatter() -> Frontmatter {
            Frontmatter::new(HashMap::from([(
                "root".into(),
                FieldValue::Object(HashMap::from([(
                    "nested".into(),
                    FieldValue::Array(vec![
                        FieldValue::String("x".into()),
                        FieldValue::Boolean(true),
                    ]),
                )])),
            )]))
        }

        pub fn note_with_frontmatter() -> QuerySetupResult {
            let (dir, db) = test_db()?;
            let config = test_config()?;
            let fm = complex_frontmatter();
            let cmd = CommandAdapter::new(&db, &config);

            let path =
                NotePath::try_new("notes/a.md").map_err(|e| e.to_string())?;
            let mut note =
                Command::create(&cmd, &path).map_err(|e| e.to_string())?;
            note.set_frontmatter(Some(fm.clone()));
            let id = note.id();
            Command::update(&cmd, note).map_err(|e| e.to_string())?;
            Ok((dir, db, id, fm))
        }

        pub fn note_with_indexes() -> IndexedNoteSetup {
            let (dir, db) = test_db()?;
            let config = test_config()?;
            let cmd = CommandAdapter::new(&db, &config);

            let path =
                NotePath::try_new("notes/a.md").map_err(|e| e.to_string())?;
            let mut note =
                Command::create(&cmd, &path).map_err(|e| e.to_string())?;

            let frontmatter = Frontmatter::new(HashMap::from([
                (
                    "aliases".into(),
                    FieldValue::Array(vec![FieldValue::String("Alias".into())]),
                ),
                ("file_class".into(), FieldValue::String("Class".into())),
                ("category".into(), FieldValue::String("docs".into())),
            ]));
            note.set_frontmatter(Some(frontmatter));

            let status = config
                .task()
                .status()
                .name_for_symbol(
                    StatusSymbol::try_new(' ').map_err(|e| e.to_string())?,
                )
                .ok_or_else(|| String::from("missing default status"))?
                .clone();
            let status_name = status.clone();
            let mut metadata = TaskMetadata::new();
            metadata
                .insert_raw("priority", FieldValue::Number(2.0))
                .map_err(|e| e.to_string())?;
            metadata
                .insert_raw("project", FieldValue::String("lithos".into()))
                .map_err(|e| e.to_string())?;
            let attributes = TaskAttributes::builder()
                .metadata(metadata)
                .created_at(Some(TaskTimestamp::new(1_700_000_000)))
                .due_at(Some(TaskTimestamp::new(1_700_000_100)))
                .reminder_at(Some(TaskTimestamp::new(1_700_000_200)))
                .completed_at(Some(TaskTimestamp::new(1_700_000_300)))
                .build();
            let task = Task::try_new(
                status,
                "Do work",
                SourceByteOffset::new(0),
                attributes,
            )
            .map_err(|e| e.to_string())?;
            note.add_task(task);

            let id = note.id();
            Command::update(&cmd, note).map_err(|e| e.to_string())?;
            Ok((dir, db, id, status_name))
        }
    }

    mod query {
        use super::*;

        #[test]
        fn find_by_id_returns_note_with_matching_id()
        -> Result<(), NoteQueryError> {
            let (_dir, db, id, _fm) = fixtures::note_with_frontmatter()
                .map_err(|e| {
                    NoteQueryError::Domain(NoteError::Storage(e.into()))
                })?;
            let qry = QueryAdapter::new(Arc::new(db));

            let observed = qry
                .find_by_id(id)?
                .expect("Query by id should return Some(note)");
            assert_eq!(observed.id(), id, "Observed id should match");
            Ok(())
        }

        #[test]
        fn find_by_id_returns_none_for_missing_id() -> Result<(), NoteQueryError>
        {
            let (_dir, db) = fixtures::test_db().map_err(|e| {
                NoteQueryError::Domain(NoteError::Storage(e.into()))
            })?;
            let qry = QueryAdapter::new(Arc::new(db));
            let miss =
                qry.find_by_id(NoteId::from(fixtures::TEST_MISSING_ID))?;
            assert!(miss.is_none(), "Non-existent ID should return None");
            Ok(())
        }

        #[test]
        fn list_returns_all_notes() -> Result<(), NoteQueryError> {
            let (_dir, db, _id, _fm) = fixtures::note_with_frontmatter()
                .map_err(|e| {
                    NoteQueryError::Domain(NoteError::Storage(e.into()))
                })?;
            let qry = QueryAdapter::new(Arc::new(db));
            let notes = qry.list()?;
            assert_eq!(notes.len(), 1);
            Ok(())
        }

        #[test]
        fn indexed_queries_return_expected_note() -> Result<(), NoteQueryError>
        {
            let (_dir, db, id, status_name) = fixtures::note_with_indexes()
                .map_err(|e| {
                    NoteQueryError::Domain(NoteError::Storage(e.into()))
                })?;
            let qry = QueryAdapter::new(Arc::new(db));

            let alias =
                AliasName::try_new("Alias").map_err(NoteQueryError::Domain)?;
            let by_alias =
                qry.find_by_alias(&alias)?.expect("alias should match");
            assert_eq!(by_alias.id(), id);

            let class = FileClassName::try_new("Class")
                .map_err(NoteQueryError::Domain)?;
            let by_class = qry.list_by_file_class(&class)?;
            assert!(by_class.iter().any(|note| note.id() == id));

            let folder =
                FolderPath::try_new("notes").map_err(NoteQueryError::Domain)?;
            let by_folder = qry.list_by_folder(&folder)?;
            assert!(by_folder.iter().any(|note| note.id() == id));

            let key =
                crate::config::frontmatter::FrontmatterKey::try_new("category")
                    .map_err(|error| NoteQueryError::Domain(error.into()))?;
            let by_frontmatter = qry.list_by_frontmatter_kv(&key, "docs")?;
            assert!(by_frontmatter.iter().any(|note| note.id() == id));

            let by_status = qry.list_by_task_status(&status_name)?;
            assert!(by_status.iter().any(|note| note.id() == id));

            let priority =
                TaskPriority::try_new(2.0).map_err(NoteQueryError::Domain)?;
            let by_priority = qry.list_by_task_priority(priority)?;
            assert!(by_priority.iter().any(|note| note.id() == id));

            let by_project = qry.list_by_task_project("lithos")?;
            assert!(by_project.iter().any(|note| note.id() == id));

            let by_created = qry
                .list_by_task_created_date(TaskTimestamp::new(1_700_000_000))?;
            assert!(by_created.iter().any(|note| note.id() == id));

            let by_due =
                qry.list_by_task_due_date(TaskTimestamp::new(1_700_000_100))?;
            assert!(by_due.iter().any(|note| note.id() == id));

            let by_reminder = qry.list_by_task_reminder_date(
                TaskTimestamp::new(1_700_000_200),
            )?;
            assert!(by_reminder.iter().any(|note| note.id() == id));

            let by_completed = qry.list_by_task_completed_date(
                TaskTimestamp::new(1_700_000_300),
            )?;
            assert!(by_completed.iter().any(|note| note.id() == id));

            Ok(())
        }
    }
}
