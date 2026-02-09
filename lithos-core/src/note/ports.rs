//! Command and Query port traits for the Note bounded context.
//!
//! Defines the architectural boundaries for interacting with note data,
//! following the Port-Based CQRS pattern.

//! Note bounded context ports for CQRS operations.
//!
//! This module defines the command and query trait interfaces for the Note
//! aggregate. These are implemented by storage adapters to provide
//! persistent data access.
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
/// Implementations handle the atomic persistence of notes and the maintenance
/// of their secondary indexes.
pub trait Command: Send + Sync {
    /// Creates a new note with the given vault-relative path.
    ///
    /// # Errors
    ///
    /// Returns [`NoteCommandError`] if:
    /// - The note already exists at the given path.
    /// - Initial index population fails.
    /// - The storage layer returns an error.
    fn create(&self, path: String) -> Result<Note, NoteCommandError>;

    /// Deletes a note by its unique identifier.
    ///
    /// # Errors
    ///
    /// Returns [`NoteCommandError`] if:
    /// - The note does not exist.
    /// - Cleanup of secondary indexes fails.
    /// - The storage layer returns an error.
    fn delete(&self, id: Uuid) -> Result<(), NoteCommandError>;

    /// Updates an existing note aggregate.
    ///
    /// # Errors
    ///
    /// Returns [`NoteCommandError`] if:
    /// - The note does not exist.
    /// - Atomically updating the note and its indexes fails.
    /// - The storage layer returns an error.
    fn update(&self, note: Note) -> Result<Note, NoteCommandError>;
}

/// Query port for Note read operations.
///
/// This trait defines the interface for queries that retrieve Note state.
/// Implementations provide high-performance, zero-copy access to note and
/// task data through specialized indexes.
pub trait Query: Send + Sync {
    /// Archived note type for zero-copy reads.
    type NoteArchived<'archived>;

    /// Finds a single note by its configured alias.
    ///
    /// # Errors
    ///
    /// Returns [`NoteQueryError`] if the underlying storage fails.
    fn find_by_alias(
        &self,
        alias: &str,
    ) -> Result<Option<Note>, NoteQueryError>;

    /// Finds all notes belonging to a specific file class.
    ///
    /// # Errors
    ///
    /// Returns [`NoteQueryError`] if the underlying storage fails.
    fn find_by_file_class(
        &self,
        class: &str,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Finds all notes located within a specific vault folder.
    ///
    /// # Errors
    ///
    /// Returns [`NoteQueryError`] if the underlying storage fails.
    fn find_by_folder(&self, folder: &str)
    -> Result<Vec<Note>, NoteQueryError>;

    /// Finds a note by its unique UUID v7 identifier (owned).
    ///
    /// # Errors
    ///
    /// Returns [`NoteQueryError`] if the underlying storage fails.
    fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteQueryError>;

    /// Finds a note by its vault-relative path (owned).
    ///
    /// # Errors
    ///
    /// Returns [`NoteQueryError`] if the underlying storage fails.
    fn find_by_path(&self, path: &str) -> Result<Option<Note>, NoteQueryError>;

    /// Finds all notes containing tasks completed on a specific date.
    ///
    /// # Errors
    ///
    /// Returns [`NoteQueryError`] if the underlying storage fails.
    fn find_by_task_completed_date(
        &self,
        completed_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Finds all notes containing tasks created on a specific date.
    ///
    /// # Errors
    ///
    /// Returns [`NoteQueryError`] if the underlying storage fails.
    fn find_by_task_created_date(
        &self,
        created_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Finds all notes containing tasks due on a specific date.
    ///
    /// # Errors
    ///
    /// Returns [`NoteQueryError`] if the underlying storage fails.
    fn find_by_task_due_date(
        &self,
        due_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Finds all notes containing tasks with a specific priority level.
    ///
    /// # Errors
    ///
    /// Returns [`NoteQueryError`] if the underlying storage fails.
    fn find_by_task_priority(
        &self,
        priority: f64,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Finds all notes containing tasks assigned to a specific project.
    ///
    /// # Errors
    ///
    /// Returns [`NoteQueryError`] if the underlying storage fails.
    fn find_by_task_project(
        &self,
        project: &str,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Finds all notes containing tasks with a specific reminder date.
    ///
    /// # Errors
    ///
    /// Returns [`NoteQueryError`] if the underlying storage fails.
    fn find_by_task_reminder_date(
        &self,
        reminder_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Finds all notes containing tasks with a specific status name.
    ///
    /// # Errors
    ///
    /// Returns [`NoteQueryError`] if the underlying storage fails.
    fn find_by_task_status(
        &self,
        status: &str,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Lists all notes currently managed in the vault (owned).
    ///
    /// # Errors
    ///
    /// Returns [`NoteQueryError`] if the underlying storage fails.
    fn list(&self) -> Result<Vec<Note>, NoteQueryError>;

    /// Queries notes by a generic frontmatter key-value pair.
    ///
    /// # Errors
    ///
    /// Returns [`NoteQueryError`] if the underlying storage fails.
    fn query_frontmatter_kv(
        &self,
        key: &str,
        value: &str,
    ) -> Result<Vec<Note>, NoteQueryError>;

    /// Accesses a note by ID as archived data, enabling zero-copy reads.
    ///
    /// This method allows executing a closure against the low-level archived
    /// representation of a note without performing full deserialization.
    ///
    /// # Errors
    ///
    /// Returns [`NoteQueryError`] if:
    /// - The note is not found.
    /// - The storage layer fails.
    /// - Zero-copy access validation fails.
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
