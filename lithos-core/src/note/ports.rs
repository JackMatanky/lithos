//! Note bounded context ports for CQRS operations.
//!
//! This module defines the command and query trait interfaces for the Note
//! aggregate. These are shells for future implementation by adapters.
//!
//! # Note on Sync-First
//! This trait is synchronous per the architecture proposal (Phase 3).
//! Async wrappers should be added at the CLI/LSP boundary if needed.

use uuid::Uuid;

use super::{
    aggregate::Note,
    error::{NoteCommandError, NoteQueryError},
};

/// Command port for Note write operations.
///
/// This trait defines the interface for commands that modify Note state.
/// Implementations should be in the adapters layer.
pub trait Command: Send + Sync {
    /// Creates a new note with the given vault-relative path.
    ///
    /// # Errors
    /// Returns `NoteCommandError` if note creation fails.
    fn create(&self, path: String) -> Result<Note, NoteCommandError>;

    /// Deletes a note by ID.
    ///
    /// # Errors
    /// Returns `NoteCommandError` if note deletion fails.
    fn delete(&self, id: Uuid) -> Result<(), NoteCommandError>;

    /// Updates an existing note.
    ///
    /// # Errors
    /// Returns `NoteCommandError` if note update fails.
    fn update(&self, note: Note) -> Result<Note, NoteCommandError>;
}

/// Query port for Note read operations.
///
/// This trait defines the interface for queries that retrieve Note state.
/// Implementations should be in the adapters layer.
pub trait Query: Send + Sync {
    /// Archived note type for zero-copy reads.
    type NoteArchived<'archived>;

    /// Finds notes by alias using the alias index.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_alias(
        &self,
        alias: &str,
    ) -> Result<Option<Note>, NoteQueryError>;

    /// Finds notes by file class using the `file_class` index.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_file_class(
        &self,
        class: &str,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Finds notes by folder path using the folder index.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_folder(&self, folder: &str)
    -> Result<Vec<Note>, NoteQueryError>;

    /// Finds a note by its UUID v7 identifier (owned).
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteQueryError>;

    /// Finds a note by its vault-relative path (owned).
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_path(&self, path: &str) -> Result<Option<Note>, NoteQueryError>;

    /// Finds notes by task completed date using the `tasks_by_completed_date`
    /// index.
    ///
    /// Returns notes containing tasks with the specified completed date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_task_completed_date(
        &self,
        completed_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Finds notes by task created date using the `tasks_by_created_date`
    /// index.
    ///
    /// Returns notes containing tasks with the specified created date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_task_created_date(
        &self,
        created_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Finds notes by task due date using the `tasks_by_due_date` index.
    ///
    /// Returns notes containing tasks with the specified due date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_task_due_date(
        &self,
        due_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Finds notes by task priority using the `tasks_by_priority` index.
    ///
    /// Returns notes containing tasks with the specified priority.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_task_priority(
        &self,
        priority: f64,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Finds notes by task project using the `tasks_by_project` index.
    ///
    /// Returns notes containing tasks with the specified project.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_task_project(
        &self,
        project: &str,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Finds notes by task reminder date using the `tasks_by_reminder_date`
    /// index.
    ///
    /// Returns notes containing tasks with the specified reminder date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_task_reminder_date(
        &self,
        reminder_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Finds notes by task status using the `tasks_by_status` index.
    ///
    /// Returns notes containing tasks with the specified status.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn find_by_task_status(
        &self,
        status: &str,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Lists all notes in the vault (owned).
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn list(&self) -> Result<Vec<Note>, NoteQueryError>;

    /// Queries notes by frontmatter key-value pair using the generic
    /// frontmatter index.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn query_frontmatter_kv(
        &self,
        key: &str,
        value: &str,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Access a note as archived data (zero-copy).
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query execution fails.
    fn with_archived_by_id<F, R>(
        &self,
        id: Uuid,
        f: F,
    ) -> Result<Option<R>, NoteQueryError>
    where
        F: for<'archived> FnOnce(Self::NoteArchived<'archived>) -> R;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_trait_is_object_safe() {
        // GIVEN: the Command trait
        // WHEN: used as a trait object
        fn _assert_object_safe(_: &dyn Command) {}
        // THEN: it compiles
    }

    #[test]
    fn query_trait_is_send_and_sync() {
        // GIVEN: the Query trait
        // WHEN: checking Send + Sync bounds
        fn _assert_query_is_send_sync<T: Query>() {
            fn is_send_sync<U: Send + Sync>() {}
            is_send_sync::<T>();
        }
        // THEN: it satisfies the bounds
    }
}
