//! Repository traits for Note persistence.
//!
//! Defines segregated read and write interfaces following ADR 016.

use super::{
    aggregate::{Note, NoteId},
    error::NoteRepositoryError,
    paths::NotePath,
};

/// Read-only repository operations for Note aggregates.
pub trait ReadRepository {
    /// Find a note by its stable identifier.
    ///
    /// # Errors
    ///
    /// Returns `NoteRepositoryError::Storage` if the database operation fails.
    fn find_by_id(
        &self,
        id: NoteId,
    ) -> Result<Option<Note>, NoteRepositoryError>;

    /// Find a note by its vault path.
    ///
    /// # Errors
    ///
    /// Returns `NoteRepositoryError::Storage` if the database operation fails.
    fn find_by_path(
        &self,
        path: &NotePath,
    ) -> Result<Option<Note>, NoteRepositoryError>;

    /// Find multiple notes by their identifiers.
    ///
    /// Returns notes in the same order as the input IDs. Missing notes are
    /// skipped.
    ///
    /// # Errors
    ///
    /// Returns `NoteRepositoryError::Storage` if the database operation fails.
    fn find_many_by_id(
        &self,
        ids: &[NoteId],
    ) -> Result<Vec<Note>, NoteRepositoryError>;

    /// List all notes in the repository.
    ///
    /// # Errors
    ///
    /// Returns `NoteRepositoryError::Storage` if the database operation fails.
    fn list(&self) -> Result<Vec<Note>, NoteRepositoryError>;
}

/// Write operations for Note persistence.
pub trait WriteRepository {
    /// Save a note, returning its stable identifier.
    ///
    /// Creates or updates the note and maintains the path index atomically.
    ///
    /// # Errors
    ///
    /// Returns `NoteRepositoryError::DuplicatePath` if another note exists at
    /// the same path. Returns `NoteRepositoryError::Storage` if the
    /// database operation fails.
    fn save(&self, note: &Note) -> Result<NoteId, NoteRepositoryError>;

    /// Save multiple notes in a single transaction.
    ///
    /// Returns all saved note IDs in the same order as the input.
    ///
    /// # Errors
    ///
    /// Returns `NoteRepositoryError::DuplicatePath` if any note has a
    /// conflicting path. Returns `NoteRepositoryError::Storage` if the
    /// database operation fails.
    fn save_many(
        &self,
        notes: &[Note],
    ) -> Result<Vec<NoteId>, NoteRepositoryError>;

    /// Delete a note by its identifier.
    ///
    /// Removes the note and all associated indexes atomically. Idempotent (no
    /// error if missing).
    ///
    /// # Errors
    ///
    /// Returns `NoteRepositoryError::Storage` if the database operation fails.
    fn delete(&self, id: NoteId) -> Result<(), NoteRepositoryError>;

    /// Delete multiple notes in a single transaction.
    ///
    /// Idempotent for each ID (no error if any are missing).
    ///
    /// # Errors
    ///
    /// Returns `NoteRepositoryError::Storage` if the database operation fails.
    fn delete_many(&self, ids: &[NoteId]) -> Result<(), NoteRepositoryError>;
}

/// Unified repository combining read and write capabilities.
///
/// This is a marker trait automatically implemented for any type that
/// implements both `ReadRepository` and `WriteRepository`.
pub trait Repository: ReadRepository + WriteRepository {}

// Blanket implementation: any type with both read and write gets Repository for
// free
impl<T> Repository for T where T: ReadRepository + WriteRepository {}
