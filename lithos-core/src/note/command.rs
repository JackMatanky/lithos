//! Note command implementations (CQRS write operations).
//!
//! This module provides the [`Command`] type, which handles write operations
//! through the note command port.

use super::{
    ParsedNote, aggregate::NoteId, error::NoteCommandError, paths::NotePath,
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
    /// Records a parsed note projection.
    ///
    /// # Errors
    /// Returns `NoteCommandError` if persistence fails.
    #[inline]
    pub fn record_parsed_note(
        &self,
        path: &NotePath,
        parsed: &ParsedNote,
    ) -> Result<NoteId, NoteCommandError> {
        self.command_port
            .record_parsed_note(path, parsed)
            .map_err(|error| NoteCommandError::Storage(error.into()))
    }

    /// Records deletion of a note projection.
    ///
    /// # Errors
    /// Returns `NoteCommandError` if deletion fails.
    #[inline]
    pub fn record_deleted_note(
        &self,
        id: NoteId,
    ) -> Result<(), NoteCommandError> {
        self.command_port
            .record_deleted_note(id)
            .map_err(|error| NoteCommandError::Storage(error.into()))
    }

    /// Rebuilds all task indexes from stored notes.
    ///
    /// Returns the number of tasks rebuilt.
    ///
    /// # Errors
    /// Returns `NoteCommandError` if rebuild fails.
    #[inline]
    pub fn rebuild_task_indexes(&self) -> Result<usize, NoteCommandError> {
        self.command_port
            .rebuild_task_indexes()
            .map_err(|error| NoteCommandError::Storage(error.into()))
    }

    /// Rebuilds all note indexes from stored projections.
    ///
    /// Returns the number of notes rebuilt.
    ///
    /// # Errors
    /// Returns `NoteCommandError` if rebuild fails.
    #[inline]
    pub fn rebuild_note_indexes(&self) -> Result<usize, NoteCommandError> {
        self.command_port
            .rebuild_note_indexes()
            .map_err(|error| NoteCommandError::Storage(error.into()))
    }
}
