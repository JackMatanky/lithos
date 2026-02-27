//! Concrete implementation of the [`crate::note::ports::Query`] trait.
//!
//! Provides high-performance, zero-copy read operations for notes and tasks,
//! utilizing specialized database indexes.

use uuid::Uuid;

use crate::{
    config::{frontmatter::FrontmatterKey, task::StatusName},
    db::Database,
    note::{
        aggregate::{
            AliasName, FileClassName, FolderPath, Note, NoteId, NotePath,
        },
        db_table::{
            ALIAS_TO_ID, FILE_CLASS_TO_ID, FOLDER_TO_ID, FRONTMATTER_KV, NOTES,
            PATH_TO_ID, TASKS_BY_COMPLETED_DATE, TASKS_BY_CREATED_DATE,
            TASKS_BY_DUE_DATE, TASKS_BY_PRIORITY, TASKS_BY_PROJECT,
            TASKS_BY_REMINDER_DATE, TASKS_BY_STATUS,
        },
        ports::Query,
        task::{TaskPriority, TaskTimestamp},
    },
};

/// Query implementation for Note read operations.
pub struct QueryAdapter<'db> {
    db: &'db Database,
}

impl<'db> QueryAdapter<'db> {
    /// Create a new `QueryAdapter` with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }

    /// Helper method to find notes by any task index.
    fn find_notes_by_task_index(
        &self,
        index_table: redb::MultimapTableDefinition<&str, &str>,
        index_key: &str,
    ) -> Result<Vec<Note>, crate::db::DbError> {
        let note_refs = self.db.multimap_get(index_table, index_key)?;

        let mut notes = Vec::with_capacity(note_refs.len());
        for note_id_str in note_refs {
            if let Some(note) =
                self.db.get_owned::<Note>(NOTES, &note_id_str)?
            {
                notes.push(note);
            }
        }
        Ok(notes)
    }
}

impl Query for QueryAdapter<'_> {
    type Error = crate::db::DbError;
    type NoteArchived<'archived> = &'archived rkyv::Archived<Note>;

    #[inline]
    fn find_by_alias(
        &self,
        alias: &AliasName,
    ) -> Result<Option<Note>, Self::Error> {
        let ids = self.db.multimap_get(ALIAS_TO_ID, alias.as_str())?;

        if let Some(id_str) = ids.first() {
            self.db.get_owned::<Note>(NOTES, id_str)
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn find_by_file_class(
        &self,
        class: &FileClassName,
    ) -> Result<Vec<Note>, Self::Error> {
        let ids = self.db.multimap_get(FILE_CLASS_TO_ID, class.as_str())?;

        let mut notes = Vec::with_capacity(ids.len());
        for id_str in ids {
            if let Some(note) = self.db.get_owned::<Note>(NOTES, &id_str)? {
                notes.push(note);
            }
        }
        Ok(notes)
    }

    #[inline]
    fn find_by_folder(
        &self,
        folder: &FolderPath,
    ) -> Result<Vec<Note>, Self::Error> {
        let ids = self.db.multimap_get(FOLDER_TO_ID, folder.as_str())?;

        let mut notes = Vec::with_capacity(ids.len());
        for id_str in ids {
            if let Some(note) = self.db.get_owned::<Note>(NOTES, &id_str)? {
                notes.push(note);
            }
        }
        Ok(notes)
    }

    #[inline]
    fn find_by_id(&self, id: NoteId) -> Result<Option<Note>, Self::Error> {
        let mut id_buffer = Uuid::encode_buffer();
        let id_str =
            Uuid::from(id).as_hyphenated().encode_lower(&mut id_buffer);
        let id_str: &str = id_str;
        self.db.get_owned::<Note>(NOTES, id_str)
    }

    #[inline]
    fn find_by_path(
        &self,
        path: &NotePath,
    ) -> Result<Option<Note>, Self::Error> {
        let ids = self.db.multimap_get(PATH_TO_ID, path.as_str())?;

        if let Some(id_str) = ids.first() {
            self.db.get_owned::<Note>(NOTES, id_str)
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn find_by_task_completed_date(
        &self,
        completed_date: TaskTimestamp,
    ) -> Result<Vec<Note>, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        let date_str = buffer.format(completed_date.as_i64());
        self.find_notes_by_task_index(TASKS_BY_COMPLETED_DATE, date_str)
    }

    #[inline]
    fn find_by_task_created_date(
        &self,
        created_date: TaskTimestamp,
    ) -> Result<Vec<Note>, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        let date_str = buffer.format(created_date.as_i64());
        self.find_notes_by_task_index(TASKS_BY_CREATED_DATE, date_str)
    }

    #[inline]
    fn find_by_task_due_date(
        &self,
        due_date: TaskTimestamp,
    ) -> Result<Vec<Note>, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        let date_str = buffer.format(due_date.as_i64());
        self.find_notes_by_task_index(TASKS_BY_DUE_DATE, date_str)
    }

    #[inline]
    fn find_by_task_priority(
        &self,
        priority: TaskPriority,
    ) -> Result<Vec<Note>, Self::Error> {
        let mut buffer = ryu::Buffer::new();
        let priority_str = buffer.format(priority.as_f64());
        self.find_notes_by_task_index(TASKS_BY_PRIORITY, priority_str)
    }

    #[inline]
    fn find_by_task_project(
        &self,
        project: &str,
    ) -> Result<Vec<Note>, Self::Error> {
        self.find_notes_by_task_index(TASKS_BY_PROJECT, project)
    }

    #[inline]
    fn find_by_task_reminder_date(
        &self,
        reminder_date: TaskTimestamp,
    ) -> Result<Vec<Note>, Self::Error> {
        let mut buffer = itoa::Buffer::new();
        let date_str = buffer.format(reminder_date.as_i64());
        self.find_notes_by_task_index(TASKS_BY_REMINDER_DATE, date_str)
    }

    #[inline]
    fn find_by_task_status(
        &self,
        status: &StatusName,
    ) -> Result<Vec<Note>, Self::Error> {
        self.find_notes_by_task_index(TASKS_BY_STATUS, status.as_str())
    }

    #[inline]
    fn list(&self) -> Result<Vec<Note>, Self::Error> {
        self.db.list_owned::<Note>(NOTES)
    }

    #[inline]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "String length arithmetic is safe and will not overflow"
    )]
    fn query_frontmatter_kv(
        &self,
        key: &FrontmatterKey,
        value: &str,
    ) -> Result<Vec<Note>, Self::Error> {
        use std::fmt::Write as _;
        let mut combined_key =
            String::with_capacity(key.as_str().len() + value.len() + 1);
        #[expect(
            clippy::let_underscore_must_use,
            reason = "Writing to String is infallible"
        )]
        let _ = write!(&mut combined_key, "{}:{value}", key.as_str());
        self.find_notes_by_task_index(FRONTMATTER_KV, &combined_key)
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
        self.db.get::<Note, _, R>(NOTES, id_str, f)
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
                aggregate::{Note, NoteId, NotePath},
                frontmatter::Frontmatter,
                ports::Command,
                task::{Task, TaskAttributes, TaskMetadata, TaskTimestamp},
                types::SourceByteOffset,
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
            let fm = complex_frontmatter();
            let mut note = Note::new(NoteId::new(), "notes/a.md")
                .map_err(|e| e.to_string())?;
            note.set_frontmatter(Some(fm.clone()));
            let id = note.id();
            let uuid = Uuid::from(id);
            db.put_by_uuid(NOTES, uuid, &note).map_err(|e| e.to_string())?;
            db.multimap_insert(PATH_TO_ID, "notes/a.md", &uuid.to_string())
                .map_err(|e| e.to_string())?;
            Ok((dir, db, id, fm))
        }

        pub fn note_with_indexes() -> IndexedNoteSetup {
            let (dir, db) = test_db()?;
            let config = test_config()?;
            let cmd = CommandAdapter::new(&db, &config);

            let path =
                NotePath::new("notes/a.md").map_err(|e| e.to_string())?;
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
            let task = Task::new(
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
            let qry = QueryAdapter::new(&db);

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
            let qry = QueryAdapter::new(&db);
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
            let qry = QueryAdapter::new(&db);
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
            let qry = QueryAdapter::new(&db);

            let alias =
                AliasName::try_new("Alias").map_err(NoteQueryError::Domain)?;
            let by_alias =
                qry.find_by_alias(&alias)?.expect("alias should match");
            assert_eq!(by_alias.id(), id);

            let class = FileClassName::try_new("Class")
                .map_err(NoteQueryError::Domain)?;
            let by_class = qry.find_by_file_class(&class)?;
            assert!(by_class.iter().any(|note| note.id() == id));

            let folder =
                FolderPath::try_new("notes").map_err(NoteQueryError::Domain)?;
            let by_folder = qry.find_by_folder(&folder)?;
            assert!(by_folder.iter().any(|note| note.id() == id));

            let key =
                crate::config::frontmatter::FrontmatterKey::try_new("category")
                    .map_err(|error| {
                        NoteQueryError::Domain(NoteError::ValidationFailed(
                            error.to_string().into(),
                        ))
                    })?;
            let by_frontmatter = qry.query_frontmatter_kv(&key, "docs")?;
            assert!(by_frontmatter.iter().any(|note| note.id() == id));

            let by_status = qry.find_by_task_status(&status_name)?;
            assert!(by_status.iter().any(|note| note.id() == id));

            let priority =
                TaskPriority::try_new(2.0).map_err(NoteQueryError::Domain)?;
            let by_priority = qry.find_by_task_priority(priority)?;
            assert!(by_priority.iter().any(|note| note.id() == id));

            let by_project = qry.find_by_task_project("lithos")?;
            assert!(by_project.iter().any(|note| note.id() == id));

            let by_created = qry
                .find_by_task_created_date(TaskTimestamp::new(1_700_000_000))?;
            assert!(by_created.iter().any(|note| note.id() == id));

            let by_due =
                qry.find_by_task_due_date(TaskTimestamp::new(1_700_000_100))?;
            assert!(by_due.iter().any(|note| note.id() == id));

            let by_reminder = qry.find_by_task_reminder_date(
                TaskTimestamp::new(1_700_000_200),
            )?;
            assert!(by_reminder.iter().any(|note| note.id() == id));

            let by_completed = qry.find_by_task_completed_date(
                TaskTimestamp::new(1_700_000_300),
            )?;
            assert!(by_completed.iter().any(|note| note.id() == id));

            Ok(())
        }
    }
}
