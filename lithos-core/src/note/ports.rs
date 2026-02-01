//! Note bounded context ports for CQRS operations.
//!
//! This module defines the command and query trait interfaces for the Note
//! aggregate. These are shells for future implementation by adapters.
//!
//! # Note on Sync-First
//! This trait is synchronous per the architecture proposal (Phase 3).
//! Async wrappers should be added at the CLI/LSP boundary if needed.

use uuid::Uuid;

use super::{Note, NoteError};

/// Command port for Note write operations.
///
/// This trait defines the interface for commands that modify Note state.
/// Implementations should be in the adapters layer.
pub trait Command: Send + Sync {
    /// Creates a new note with the given vault-relative path.
    ///
    /// # Errors
    /// Returns `NoteError` if note creation fails validation or persistence.
    fn create(&self, path: String) -> Result<Note, NoteError>;

    /// Deletes a note by ID.
    ///
    /// # Errors
    /// Returns `NoteError` if note deletion fails.
    fn delete(&self, id: Uuid) -> Result<(), NoteError>;

    /// Updates an existing note.
    ///
    /// # Errors
    /// Returns `NoteError` if note update fails validation or persistence.
    fn update(&self, note: Note) -> Result<Note, NoteError>;
}

/// Query port for Note read operations.
///
/// This trait defines the interface for queries that retrieve Note state.
/// Implementations should be in the adapters layer.
pub trait Query: Send + Sync {
    /// Finds a note by its UUID v7 identifier.
    ///
    /// # Errors
    /// Returns `NoteError` if query execution fails.
    fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, NoteError>;

    /// Finds a note by its vault-relative path.
    ///
    /// # Errors
    /// Returns `NoteError` if query execution fails.
    fn find_by_path(&self, path: &str) -> Result<Option<Note>, NoteError>;

    /// Lists all notes in the vault.
    ///
    /// # Errors
    /// Returns `NoteError` if query execution fails.
    fn list(&self) -> Result<Vec<Note>, NoteError>;
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
    fn query_trait_is_object_safe() {
        // GIVEN: the Query trait
        // WHEN: used as a trait object
        fn _assert_object_safe(_: &dyn Query) {}
        // THEN: it compiles
    }

    #[test]
    fn traits_are_send_and_sync() {
        // GIVEN: the port traits
        #[expect(
            dead_code,
            reason = "Helper function for compile-time trait checking"
        )]
        fn is_send_sync<T: Send + Sync>() {}

        // WHEN: checking Send + Sync bounds
        fn _assert_command_is_send_sync<T: Command>() {
            is_send_sync::<T>();
        }
        fn _assert_query_is_send_sync<T: Query>() {
            is_send_sync::<T>();
        }
        // THEN: they satisfy the bounds
    }
}
