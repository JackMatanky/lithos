//! Note command implementations (CQRS write operations).
//!
//! This module implements the Command port trait for Note write operations,
//! using the Database layer for persistence.

use uuid::Uuid;

use super::{aggregate::Note, error::NoteCommandError, types::NoteId};
use crate::db::Database;

/// Helper type for multimap value tuples.
type TaskIdStr = (String, String);

/// Command implementation for Note write operations.
///
/// Implements the Command port trait using the Database layer.
pub struct Command<'db> {
    db: &'db Database,
}

impl<'db> Command<'db> {
    /// Create a new `Command` with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }
}

impl super::ports::Command for Command<'_> {
    /// Creates a new note with the given vault-relative path.
    ///
    /// # Errors
    /// Returns `NoteCommandError` if note creation fails.
    #[inline]
    fn create(&self, path: String) -> Result<Note, NoteCommandError> {
        let note = Note::new(NoteId::new(), path)?;
        let id_str = Uuid::from(note.id()).to_string();

        self.db
            .put("notes", &id_str, &note)
            .map_err(NoteCommandError::Storage)?;

        self.db
            .multimap_insert("path_to_id", note.path().as_str(), &id_str)
            .map_err(NoteCommandError::Storage)?;

        Ok(note)
    }

    /// Deletes a note by ID.
    ///
    /// # Errors
    /// Returns `NoteCommandError` if note deletion fails.
    #[inline]
    fn delete(&self, id: Uuid) -> Result<(), NoteCommandError> {
        let id_str = id.to_string();

        // 1. Get old data for index cleanup using zero-copy read
        let old_data = self
            .db
            .get::<Note, _, (String, Vec<String>)>(
                "notes",
                &id_str,
                |archived| {
                    let path = archived.path().as_str().to_owned();
                    let tags: Vec<String> = archived
                        .tags()
                        .iter()
                        .map(|t| t.full_path().as_str().to_owned())
                        .collect();
                    (path, tags)
                },
            )
            .map_err(NoteCommandError::Storage)?;

        // Extract old frontmatter for cleanup
        let old_frontmatter = self
            .db
            .get::<Note, _, ((), ())>(
                "notes",
                &id_str,
                |archived| archived.frontmatter()
            )
            .map_err(NoteCommandError::Storage)?;

        if let Some((path, tags)) = old_data {
            // 2. Remove from path index
            self.db
                .multimap_remove("path_to_id", &path, &id_str)
                .map_err(NoteCommandError::Storage)?;

            // 3. Remove from tag indexes
            for tag in old_tags {
                self.db
                    .multimap_remove("tags_to_notes", &tag, &id_str)
                    .map_err(NoteCommandError::Storage)?;
            }

            // 3a. Remove from frontmatter alias index
            if let Some(frontmatter) = old_frontmatter {
                if let Some(aliases) = frontmatter.aliases(&crate::config::aggregate::Config::default()) {
                    for alias in &aliases {
                        self.db
                            .multimap_remove("alias_to_id", alias.as_ref(), &(id_str, "".to_string()))
                            .map_err(NoteCommandError::Storage)?;
                    }
                }

                // 3b. Remove from file class index
                if let Some(file_class) = frontmatter.file_class(&crate::config::aggregate::Config::default()) {
                    self.db
                        .multimap_remove("file_class_to_id", file_class, &(id_str, "".to_string()))
                        .map_err(NoteCommandError::Storage)?;
                }

                // 3c. Remove from folder index
                if let Some(folder) = self.extract_folder_from_path(&path) {
                    self.db
                        .multimap_remove("folder_to_id", folder, &(id_str, "".to_string()))
                        .map_err(NoteCommandError::Storage)?;
                }

                // 3d. Remove from frontmatter KV index
                for (key, value) in frontmatter.fields {
                    if let Some(field_value) = value.as_str() {
                        self.db
                            .multimap_remove("frontmatter_kv_to_id", &(key, field_value.to_string()), &(id_str, "".to_string()))
                            .map_err(NoteCommandError::Storage)?;
        }
    }
}

impl super::ports::Command for Command<'_> {

            // 4. Delete note
            self.db
                .delete("notes", &id_str)
                .map_err(NoteCommandError::Storage)?;
        }

        Ok(())
    }

    /// Extracts folder path from a note path, excluding the note filename.
    ///
    /// Returns None for root-level notes.
    fn extract_folder_from_path(&self, path: &str) -> Option<&str> {
        path.rsplit('/').nth(1)
    }
}
        if let Some(created_at) = task.created_at() {
            self.db.multimap_remove("tasks_by_created_date", &created_at.to_string(), &(note_id_str, TaskIdStr));
        }
        if let Some(reminder_at) = task.reminder_at() {
            self.db.multimap_remove("tasks_by_reminder_date", &reminder_at.to_string(), &(note_id_str, TaskIdStr));
        }
        if let Some(completed_at) = task.completed_at() {
            self.db.multimap_remove("tasks_by_completed_date", &completed_at.to_string(), &(note_id_str, TaskIdStr));
        }

        // Priority index
        if let Some(priority) = task.metadata().get_number("priority") {
            self.db.multimap_remove("tasks_by_priority", &priority.to_string(), &(note_id_str, TaskIdStr));
        }

        // Project index
        if let Some(project) = task.metadata().get_string("project") {
            self.db.multimap_remove("tasks_by_project", project, &(note_id_str, TaskIdStr));
        }

        // Status index
        self.db.multimap_remove("tasks_by_status", task.status().as_str(), &(note_id_str, TaskIdStr));

        // Generic metadata index
        for (field_name, field_value) in task.metadata().fields {
            let field_key = format!("{}::{}", field_name);
            self.db.multimap_remove("tasks_metadata", &(field_key, field_value.to_string()), &(note_id_str, TaskIdStr));
        }
    }
}

            // 3. Update tag index
            // Remove old tags
            for tag in old_tags {
                self.db
                    .multimap_remove("tags_to_notes", &tag, &id_str)
                    .map_err(NoteCommandError::Storage)?;
            }

            // Remove old task-specific indexes
            for task in old_tasks {
                self.remove_task_indexes(&id_str, task)
                    .map_err(NoteCommandError::Storage)?;
            }
        } else {
            // New note (even though it's update call), add path index
            self.db
                .multimap_insert("path_to_id", note.path().as_str(), &id_str)
                .map_err(NoteCommandError::Storage)?;
        }

        // Add new tags
        for tag in note.tags() {
            self.db
                .multimap_insert("tags_to_notes", tag.full_path(), &id_str)
                .map_err(NoteCommandError::Storage)?;
        }

        // 6. Update task-specific indexes for updated note
        for task in note.tasks() {
            self.update_task_indexes(&id_str, task, &note)
                .map_err(NoteCommandError::Storage)?;
        }

        // 6. Update task-specific indexes for updated note
        for task in note.tasks() {
            self.update_task_indexes(&id_str, task, &note)
                .map_err(NoteCommandError::Storage)?;
        }

        // 4. Save new note
        self.db
            .put("notes", &id_str, &note)
            .map_err(NoteCommandError::Storage)?;

        // 5. Update task-specific indexes for new note
        for task in note.tasks() {
            self.update_task_indexes(&id_str, task, &note)
                .map_err(NoteCommandError::Storage)?;
        }

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
    mod fixtures {
        use tempfile::{TempDir, tempdir};
        use uuid::Uuid;

        use super::*;
        use crate::note::{aggregate::NotePath, tag::Tag};

        pub const TEST_MISSING_ID: Uuid =
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0301);

        pub fn test_db() -> Result<(TempDir, Database), String> {
            let dir = tempdir().map_err(|e| e.to_string())?;
            let path = dir.path().join("notes.redb");
            let db = Database::open(&path).map_err(|e| e.to_string())?;
            Ok((dir, db))
        }

        pub fn create_note(
            cmd: &Command,
            path: &str,
        ) -> Result<Note, NoteCommandError> {
            crate::note::ports::Command::create(cmd, path.to_owned())
        }

        pub fn update_note(
            cmd: &Command,
            note: Note,
        ) -> Result<Note, NoteCommandError> {
            crate::note::ports::Command::update(cmd, note)
        }

        pub fn delete_note(
            cmd: &Command,
            id: Uuid,
        ) -> Result<(), NoteCommandError> {
            crate::note::ports::Command::delete(cmd, id)
        }

        pub fn parse_path(path: &str) -> Result<NotePath, String> {
            NotePath::new(path.to_owned()).map_err(|e| e.to_string())
        }

        pub fn parse_tag(tag: &str) -> Result<Tag, String> {
            Tag::new(tag).map_err(|e| e.to_string())
        }

        pub fn stored_note(
            db: &Database,
            id: &str,
        ) -> Result<Option<Note>, String> {
            db.get_owned::<Note>("notes", id).map_err(|e| e.to_string())
        }

        pub fn path_index_ids(
            db: &Database,
            path: &str,
        ) -> Result<Vec<String>, String> {
            db.multimap_get("path_to_id", path).map_err(|e| e.to_string())
        }

        pub fn tag_index_ids(
            db: &Database,
            tag: &str,
        ) -> Result<Vec<String>, String> {
            db.multimap_get("tags_to_notes", tag).map_err(|e| e.to_string())
        }
    }

    use super::*;
    use crate::note::error::NoteError;

    mod persistence {
        use super::*;

        #[test]
        fn create_persists_note_path() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id_str = Uuid::from(note.id()).to_string();

            let stored_note = fixtures::stored_note(&db, &id_str)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?
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
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id_str = Uuid::from(note.id()).to_string();

            let ids = fixtures::path_index_ids(&db, note.path().as_str())
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                ids.contains(&id_str),
                "Path index should contain created note id"
            );
            Ok(())
        }

        #[test]
        fn update_removes_old_path_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id_str = Uuid::from(note.id()).to_string();
            let old_path = note.path().as_str().to_owned();
            let path = fixtures::parse_path("notes/b.md")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            note.set_path(path);

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let old_ids = fixtures::path_index_ids(&db, &old_path)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                !old_ids.contains(&id_str),
                "Old path index should not contain updated note id"
            );
            Ok(())
        }

        #[test]
        fn update_adds_new_path_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id_str = Uuid::from(note.id()).to_string();
            let path = fixtures::parse_path("notes/b.md")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            note.set_path(path);

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let new_ids = fixtures::path_index_ids(&db, "notes/b.md")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                new_ids.contains(&id_str),
                "New path index should contain updated note id"
            );
            Ok(())
        }

        #[test]
        fn update_adds_tag_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id_str = Uuid::from(note.id()).to_string();
            let tag = fixtures::parse_tag("#project")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let tag_key = tag.full_path().to_owned();
            note.add_tag(tag);

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let tag_ids = fixtures::tag_index_ids(&db, &tag_key)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                tag_ids.contains(&id_str),
                "Tag index should contain updated note id"
            );
            Ok(())
        }

        #[test]
        fn update_persists_new_path() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id_str = Uuid::from(note.id()).to_string();
            let path = fixtures::parse_path("notes/b.md")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            note.set_path(path);

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let stored_note = fixtures::stored_note(&db, &id_str)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?
                .expect("Updated note should exist");
            assert_eq!(
                stored_note.path().as_str(),
                "notes/b.md",
                "Stored note path should be updated"
            );
            Ok(())
        }

        #[test]
        fn update_persists_new_tags() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id_str = Uuid::from(note.id()).to_string();
            let tag = fixtures::parse_tag("#project")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            note.add_tag(tag);

            let result = fixtures::update_note(&cmd, note);
            assert!(result.is_ok(), "Update should succeed: {result:?}");

            let stored_note = fixtures::stored_note(&db, &id_str)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?
                .expect("Updated note should exist");
            assert_eq!(
                stored_note.tags().count(),
                1,
                "Stored note should have updated tags"
            );
            Ok(())
        }

        #[test]
        fn delete_removes_note() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id = Uuid::from(note.id());
            let id_str = id.to_string();

            let result = fixtures::delete_note(&cmd, id);
            assert!(result.is_ok(), "Delete should succeed: {result:?}");

            let stored = fixtures::stored_note(&db, &id_str)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(stored.is_none(), "Deleted note should not exist");
            Ok(())
        }

        #[test]
        fn delete_removes_path_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id = Uuid::from(note.id());
            let id_str = id.to_string();
            let tag = fixtures::parse_tag("#project")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            note.add_tag(tag);

            let update_result = fixtures::update_note(&cmd, note.clone());
            assert!(
                update_result.is_ok(),
                "Update should succeed: {update_result:?}"
            );
            let delete_result = fixtures::delete_note(&cmd, id);
            assert!(
                delete_result.is_ok(),
                "Delete should succeed: {delete_result:?}"
            );

            let path_ids = fixtures::path_index_ids(&db, note.path().as_str())
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                !path_ids.contains(&id_str),
                "Path index should not contain deleted note id"
            );
            Ok(())
        }

        #[test]
        fn delete_removes_tag_index() -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/a.md")?;
            let id = Uuid::from(note.id());
            let id_str = id.to_string();
            let tag = fixtures::parse_tag("#project")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let tag_key = tag.full_path().to_owned();
            note.add_tag(tag);

            let update_result_after = fixtures::update_note(&cmd, note);
            assert!(
                update_result_after.is_ok(),
                "Update should succeed: {update_result_after:?}"
            );
            let delete_result = fixtures::delete_note(&cmd, id);
            assert!(
                delete_result.is_ok(),
                "Delete should succeed: {delete_result:?}"
            );

            let tag_ids = fixtures::tag_index_ids(&db, &tag_key)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                !tag_ids.contains(&id_str),
                "Tag index should not contain deleted note id"
            );
            Ok(())
        }

        #[test]
        fn delete_missing_note_is_noop_for_existing_note()
        -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let note = fixtures::create_note(&cmd, "notes/existing.md")?;
            let id_str = Uuid::from(note.id()).to_string();

            let delete_result =
                fixtures::delete_note(&cmd, fixtures::TEST_MISSING_ID);
            assert!(
                delete_result.is_ok(),
                "Deleting missing note should be a no-op: {delete_result:?}"
            );

            let stored = fixtures::stored_note(&db, &id_str)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(stored.is_some(), "Existing note should remain");
            Ok(())
        }

        #[test]
        fn update_removes_old_tag_index_when_tags_change()
        -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/test.md")?;
            let id_str = Uuid::from(note.id()).to_string();
            let old_tag = fixtures::parse_tag("#old-tag")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let old_key = old_tag.full_path().to_owned();
            note.add_tag(old_tag);

            let update_result = fixtures::update_note(&cmd, note.clone());
            assert!(
                update_result.is_ok(),
                "Update should succeed: {update_result:?}"
            );

            let new_tag = fixtures::parse_tag("#new-tag")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            // To replace tags, we'd need a clear method.
            // For now I'll just clear the internal vec if I had access,
            // but Note only has add_tag.
            // I'll add Note::clear_tags or similar.
            // Actually, I'll just create a new note with same ID but new tags.
            let mut updated_note =
                Note::new(note.id(), note.path().as_str().to_owned())?;
            updated_note.add_tag(new_tag);

            let update_result_after = fixtures::update_note(&cmd, updated_note);
            assert!(
                update_result_after.is_ok(),
                "Update should succeed: {update_result_after:?}"
            );

            let old_tag_ids = fixtures::tag_index_ids(&db, &old_key)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                !old_tag_ids.contains(&id_str),
                "Old tag index should not contain note after update"
            );
            Ok(())
        }

        #[test]
        fn update_adds_new_tag_index_when_tags_change()
        -> Result<(), NoteCommandError> {
            let (_dir, db) = fixtures::test_db()
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let cmd = Command::new(&db);

            let mut note = fixtures::create_note(&cmd, "notes/test.md")?;
            let id_str = Uuid::from(note.id()).to_string();
            let old_tag = fixtures::parse_tag("#old-tag")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            note.add_tag(old_tag);

            let update_result = fixtures::update_note(&cmd, note.clone());
            assert!(
                update_result.is_ok(),
                "Update should succeed: {update_result:?}"
            );

            let new_tag = fixtures::parse_tag("#new-tag")
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            let new_key = new_tag.full_path().to_owned();

            let mut updated_note =
                Note::new(note.id(), note.path().as_str().to_owned())?;
            updated_note.add_tag(new_tag);

            let update_result_after = fixtures::update_note(&cmd, updated_note);
            assert!(
                update_result_after.is_ok(),
                "Update should succeed: {update_result_after:?}"
            );

            let new_tag_ids = fixtures::tag_index_ids(&db, &new_key)
                .map_err(|e| NoteCommandError::Domain(NoteError::Storage(e)))?;
            assert!(
                new_tag_ids.contains(&id_str),
                "New tag index should contain note after update"
            );
            Ok(())
        }

        /// Updates all task-specific indexes for a given task.
        ///
        /// This maintains the indexes defined in 005-note-cqrs.md section 2.2.
        fn update_task_indexes(
            &self,
            note_id_str: &str,
            task: &crate::note::task::Task,
            note: &Note,
        ) -> Result<(), crate::db::DbError> {
            let taskIdStr = task.id().as_uuid().to_string();

            // Core temporal indexes (always indexed)
            if let Some(due_at) = task.due_at() {
                self.db.multimap_insert(
                    "tasks_by_due_date",
                    &due_at.to_string(),
                    &(note_id_str, TaskIdStr),
                );
            }
            if let Some(created_at) = task.created_at() {
                self.db.multimap_insert(
                    "tasks_by_created_date",
                    &created_at.to_string(),
                    &(note_id_str, TaskIdStr),
                );
            }
            if let Some(reminder_at) = task.reminder_at() {
                self.db.multimap_insert(
                    "tasks_by_reminder_date",
                    &reminder_at.to_string(),
                    &(note_id_str, TaskIdStr),
                );
            }
            if let Some(completed_at) = task.completed_at() {
                self.db.multimap_insert(
                    "tasks_by_completed_date",
                    &completed_at.to_string(),
                    &(note_id_str, TaskIdStr),
                );
            }

            // Priority index (if priority field exists)
            if let Some(priority) = task.metadata().get_number("priority") {
                self.db.multimap_insert(
                    "tasks_by_priority",
                    &priority.to_string(),
                    &(note_id_str, TaskIdStr),
                );
            }

            // Project index (if project field exists)
            if let Some(project) = task.metadata().get_string("project") {
                self.db.multimap_insert(
                    "tasks_by_project",
                    project,
                    &(note_id_str, TaskIdStr),
                );
            }

            // Status index (always available via task.status())
            self.db.multimap_insert(
                "tasks_by_status",
                task.status().as_str(),
                &(note_id_str, TaskIdStr),
            );

        // Generic metadata index for non-indexed fields
        for (field_name, field_value) in task.metadata().fields {
            let field_key = format!("{}::{}", field_name);
            self.db.multimap_insert("tasks_metadata", &(field_key, field_value.to_json_string()), &(note_id_str, TaskIdStr));
        }
        }
    }
}
