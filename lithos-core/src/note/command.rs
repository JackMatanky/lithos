//! Note command implementations (CQRS write operations).
//!
//! This module provides the [`Command`] type, which handles write operations
//! through the note command port.

use super::{
    aggregate::{Note, NoteId, NotePath},
    error::NoteCommandError,
    ports as note_ports,
};

/// Command implementation for note write operations.
///
/// This struct is generic over a storage port to support multiple backends.
pub struct Command<C> {
    command_port: C,
}

impl<C> Command<C> {
    /// Create a new `Command` with a storage port.
    #[inline]
    #[must_use]
    pub const fn new(command_port: C) -> Self {
        Self {
            command_port,
        }
    }
}

impl<C> Command<C>
where
    C: note_ports::Command,
    C::Error: Into<crate::db::DbError>,
{
    /// Creates a new note with the given vault-relative path.
    ///
    /// # Errors
    /// Returns `NoteCommandError` if creation fails.
    #[inline]
    pub fn create(&self, path: &NotePath) -> Result<Note, NoteCommandError> {
        self.command_port
            .create(path)
            .map_err(|error| NoteCommandError::Storage(error.into()))
    }

    /// Deletes a note by its unique identifier.
    ///
    /// # Errors
    /// Returns `NoteCommandError` if deletion fails.
    #[inline]
    pub fn delete(&self, id: NoteId) -> Result<(), NoteCommandError> {
        self.command_port
            .delete(id)
            .map_err(|error| NoteCommandError::Storage(error.into()))
    }

    /// Updates an existing note aggregate.
    ///
    /// # Errors
    /// Returns `NoteCommandError` if update fails.
    #[inline]
    pub fn update(&self, note: Note) -> Result<Note, NoteCommandError> {
        self.command_port
            .update(note)
            .map_err(|error| NoteCommandError::Storage(error.into()))
    }
}
