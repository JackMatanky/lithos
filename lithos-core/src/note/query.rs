//! Note query implementations (CQRS read operations).
//!
//! This module provides the [`Query`] type, which handles read operations
//! through the note query port.

use super::{
    aggregate::{AliasName, FileClassName, NoteId},
    error::NoteQueryError,
    paths::{FolderPath, NotePath},
    ports as note_ports,
    stored::{StoredNote, StoredTask},
    task::{TaskDateKind, TaskPriority, TaskTimestamp},
    value::FieldValue,
};
use crate::config::{frontmatter::FrontmatterKey, task::StatusName};

/// Query implementation for note read operations.
///
/// This struct is generic over a storage port to support multiple backends.
pub struct Query<Q> {
    query_port: Q,
}

impl<Q> Query<Q> {
    /// Create a new `Query` with a storage port.
    #[inline]
    #[must_use]
    pub const fn new(query_port: Q) -> Self {
        Self {
            query_port,
        }
    }
}

impl<Q> Query<Q>
where
    Q: note_ports::Query,
    Q::Error: Into<crate::db::DbError>,
{
    /// Finds a single stored note projection by its configured alias.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn find_by_alias(
        &self,
        alias: &AliasName,
    ) -> Result<Option<StoredNote>, NoteQueryError> {
        self.query_port
            .find_by_alias(alias)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all stored note projections belonging to a specific file class.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn list_by_file_class(
        &self,
        class: &FileClassName,
    ) -> Result<Vec<StoredNote>, NoteQueryError> {
        self.query_port
            .list_by_file_class(class)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all stored note projections located within a specific vault
    /// folder.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn list_by_folder(
        &self,
        folder: &FolderPath,
    ) -> Result<Vec<StoredNote>, NoteQueryError> {
        self.query_port
            .list_by_folder(folder)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds a stored note projection by its unique UUID v7 identifier.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn find_by_id(
        &self,
        id: NoteId,
    ) -> Result<Option<StoredNote>, NoteQueryError> {
        self.query_port
            .find_by_id(id)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds a stored note projection by its vault-relative path.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn find_by_path(
        &self,
        path: &NotePath,
    ) -> Result<Option<StoredNote>, NoteQueryError> {
        self.query_port
            .find_by_path(path)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all stored notes containing tasks completed on a specific date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn list_by_task_completed_date(
        &self,
        completed_date: TaskTimestamp,
    ) -> Result<Vec<StoredNote>, NoteQueryError> {
        self.query_port
            .list_by_task_completed_date(completed_date)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all stored notes containing tasks created on a specific date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn list_by_task_created_date(
        &self,
        created_date: TaskTimestamp,
    ) -> Result<Vec<StoredNote>, NoteQueryError> {
        self.query_port
            .list_by_task_created_date(created_date)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all stored notes containing tasks due on a specific date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn list_by_task_due_date(
        &self,
        due_date: TaskTimestamp,
    ) -> Result<Vec<StoredNote>, NoteQueryError> {
        self.query_port
            .list_by_task_due_date(due_date)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all stored notes containing tasks with a specific priority level.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn list_by_task_priority(
        &self,
        priority: TaskPriority,
    ) -> Result<Vec<StoredNote>, NoteQueryError> {
        self.query_port
            .list_by_task_priority(priority)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all stored notes containing tasks assigned to a specific project.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn list_by_task_project(
        &self,
        project: &str,
    ) -> Result<Vec<StoredNote>, NoteQueryError> {
        self.query_port
            .list_by_task_project(project)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all stored notes containing tasks with a specific reminder date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn list_by_task_reminder_date(
        &self,
        reminder_date: TaskTimestamp,
    ) -> Result<Vec<StoredNote>, NoteQueryError> {
        self.query_port
            .list_by_task_reminder_date(reminder_date)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all stored notes containing tasks with a specific status name.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn list_by_task_status(
        &self,
        status: &StatusName,
    ) -> Result<Vec<StoredNote>, NoteQueryError> {
        self.query_port
            .list_by_task_status(status)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Lists tasks with a specific status name.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn list_tasks_by_status(
        &self,
        status: &StatusName,
    ) -> Result<Vec<StoredTask>, NoteQueryError> {
        self.query_port
            .list_tasks_by_status(status)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Lists tasks by a specific task date field.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn list_tasks_by_date(
        &self,
        kind: TaskDateKind,
        date: TaskTimestamp,
    ) -> Result<Vec<StoredTask>, NoteQueryError> {
        self.query_port
            .list_tasks_by_date(kind, date)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Lists tasks by a metadata field/value pair.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn list_tasks_by_metadata(
        &self,
        field: &str,
        value: &FieldValue,
    ) -> Result<Vec<StoredTask>, NoteQueryError> {
        self.query_port
            .list_tasks_by_metadata(field, value)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Lists all stored note projections currently managed in the vault.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn list(&self) -> Result<Vec<StoredNote>, NoteQueryError> {
        self.query_port
            .list()
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Queries stored notes by a generic frontmatter key-value pair.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn list_by_frontmatter_kv(
        &self,
        key: &FrontmatterKey,
        value: &str,
    ) -> Result<Vec<StoredNote>, NoteQueryError> {
        self.query_port
            .list_by_frontmatter_kv(key, value)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Accesses a note by ID as archived data, enabling zero-copy reads.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn with_archived_by_id<F, R>(
        &self,
        id: NoteId,
        f: F,
    ) -> Result<Option<R>, NoteQueryError>
    where
        F: for<'archived> FnOnce(Q::NoteArchived<'archived>) -> R,
    {
        self.query_port
            .with_archived_by_id(id, f)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }
}
