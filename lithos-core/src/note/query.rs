//! Note query implementations (CQRS read operations).
//!
//! This module provides the [`Query`] type, which handles read operations
//! through the note query port.

use uuid::Uuid;

use super::{aggregate::Note, error::NoteQueryError, ports as note_ports};

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
    /// Finds a single note by its configured alias.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn find_by_alias(
        &self,
        alias: &str,
    ) -> Result<Option<Note>, NoteQueryError> {
        self.query_port
            .find_by_alias(alias)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all notes belonging to a specific file class.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn find_by_file_class(
        &self,
        class: &str,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.query_port
            .find_by_file_class(class)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all notes located within a specific vault folder.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn find_by_folder(
        &self,
        folder: &str,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.query_port
            .find_by_folder(folder)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds a note by its unique UUID v7 identifier (owned).
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteQueryError> {
        self.query_port
            .find_by_id(id)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds a note by its vault-relative path (owned).
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn find_by_path(
        &self,
        path: &str,
    ) -> Result<Option<Note>, NoteQueryError> {
        self.query_port
            .find_by_path(path)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all notes containing tasks completed on a specific date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn find_by_task_completed_date(
        &self,
        completed_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.query_port
            .find_by_task_completed_date(completed_date)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all notes containing tasks created on a specific date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn find_by_task_created_date(
        &self,
        created_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.query_port
            .find_by_task_created_date(created_date)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all notes containing tasks due on a specific date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn find_by_task_due_date(
        &self,
        due_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.query_port
            .find_by_task_due_date(due_date)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all notes containing tasks with a specific priority level.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn find_by_task_priority(
        &self,
        priority: f64,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.query_port
            .find_by_task_priority(priority)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all notes containing tasks assigned to a specific project.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn find_by_task_project(
        &self,
        project: &str,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.query_port
            .find_by_task_project(project)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all notes containing tasks with a specific reminder date.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn find_by_task_reminder_date(
        &self,
        reminder_date: i64,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.query_port
            .find_by_task_reminder_date(reminder_date)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Finds all notes containing tasks with a specific status name.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn find_by_task_status(
        &self,
        status: &str,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.query_port
            .find_by_task_status(status)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Lists all notes currently managed in the vault (owned).
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn list(&self) -> Result<Vec<Note>, NoteQueryError> {
        self.query_port
            .list()
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Queries notes by a generic frontmatter key-value pair.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn query_frontmatter_kv(
        &self,
        key: &str,
        value: &str,
    ) -> Result<Vec<Note>, NoteQueryError> {
        self.query_port
            .query_frontmatter_kv(key, value)
            .map_err(|error| NoteQueryError::Storage(error.into()))
    }

    /// Accesses a note by ID as archived data, enabling zero-copy reads.
    ///
    /// # Errors
    /// Returns `NoteQueryError` if query fails.
    #[inline]
    pub fn with_archived_by_id<F, R>(
        &self,
        id: Uuid,
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
