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

use super::{
    aggregate::{AliasName, FileClassName, FolderPath, Note, NoteId, NotePath},
    task::{TaskPriority, TaskTimestamp},
};
use crate::config::{frontmatter::FrontmatterKey, task::StatusName};

/// Command port for Note write operations.
///
/// This trait defines the interface for commands that modify Note state.
/// Implementations handle the atomic persistence of notes and the maintenance
/// of their secondary indexes.
pub trait Command: Send + Sync {
    /// Storage error type for command operations.
    type Error: std::error::Error;

    /// Creates a new note with the given vault-relative path.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if creation fails.
    fn create(&self, path: &NotePath) -> Result<Note, Self::Error>;

    /// Deletes a note by its unique identifier.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if deletion fails.
    fn delete(&self, id: NoteId) -> Result<(), Self::Error>;

    /// Updates an existing note aggregate.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if update fails.
    fn update(&self, note: Note) -> Result<Note, Self::Error>;
}

/// Query port for Note read operations.
///
/// This trait defines the interface for queries that retrieve Note state.
/// Implementations provide high-performance, zero-copy access to note and
/// task data through specialized indexes.
pub trait Query: Send + Sync {
    /// Storage error type for query operations.
    type Error: std::error::Error;

    /// Archived note type for zero-copy reads.
    type NoteArchived<'archived>;

    /// Finds a single note by its configured alias.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if query fails.
    fn find_by_alias(
        &self,
        alias: &AliasName,
    ) -> Result<Option<Note>, Self::Error>;

    /// Finds all notes belonging to a specific file class.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if query fails.
    fn find_by_file_class(
        &self,
        class: &FileClassName,
    ) -> Result<Vec<Note>, Self::Error>;

    /// Finds all notes located within a specific vault folder.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if query fails.
    fn find_by_folder(
        &self,
        folder: &FolderPath,
    ) -> Result<Vec<Note>, Self::Error>;

    /// Finds a note by its unique UUID v7 identifier (owned).
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if query fails.
    fn find_by_id(&self, id: NoteId) -> Result<Option<Note>, Self::Error>;

    /// Finds a note by its vault-relative path (owned).
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if query fails.
    fn find_by_path(
        &self,
        path: &NotePath,
    ) -> Result<Option<Note>, Self::Error>;

    /// Finds all notes containing tasks completed on a specific date.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if query fails.
    fn find_by_task_completed_date(
        &self,
        completed_date: TaskTimestamp,
    ) -> Result<Vec<Note>, Self::Error>;

    /// Finds all notes containing tasks created on a specific date.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if query fails.
    fn find_by_task_created_date(
        &self,
        created_date: TaskTimestamp,
    ) -> Result<Vec<Note>, Self::Error>;

    /// Finds all notes containing tasks due on a specific date.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if query fails.
    fn find_by_task_due_date(
        &self,
        due_date: TaskTimestamp,
    ) -> Result<Vec<Note>, Self::Error>;

    /// Finds all notes containing tasks with a specific priority level.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if query fails.
    fn find_by_task_priority(
        &self,
        priority: TaskPriority,
    ) -> Result<Vec<Note>, Self::Error>;

    /// Finds all notes containing tasks assigned to a specific project.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if query fails.
    fn find_by_task_project(
        &self,
        project: &str,
    ) -> Result<Vec<Note>, Self::Error>;

    /// Finds all notes containing tasks with a specific reminder date.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if query fails.
    fn find_by_task_reminder_date(
        &self,
        reminder_date: TaskTimestamp,
    ) -> Result<Vec<Note>, Self::Error>;

    /// Finds all notes containing tasks with a specific status name.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if query fails.
    fn find_by_task_status(
        &self,
        status: &StatusName,
    ) -> Result<Vec<Note>, Self::Error>;

    /// Lists all notes currently managed in the vault (owned).
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if query fails.
    fn list(&self) -> Result<Vec<Note>, Self::Error>;

    /// Queries notes by a generic frontmatter key-value pair.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if query fails.
    fn query_frontmatter_kv(
        &self,
        key: &FrontmatterKey,
        value: &str,
    ) -> Result<Vec<Note>, Self::Error>;

    /// Accesses a note by ID as archived data, enabling zero-copy reads.
    ///
    /// This method allows executing a closure against the low-level archived
    /// representation of a note without performing full deserialization.
    ///
    /// # Errors
    ///
    /// Returns a storage-specific error if query fails.
    fn with_archived_by_id<F, R>(
        &self,
        id: NoteId,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: for<'archived> FnOnce(Self::NoteArchived<'archived>) -> R;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::error::NoteCommandError;

    #[test]
    fn command_trait_is_object_safe() {
        // GIVEN: the Command trait
        // WHEN: used as a trait object
        fn _assert_object_safe(_: &dyn Command<Error = NoteCommandError>) {}
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
