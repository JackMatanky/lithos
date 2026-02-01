//! Note bounded context ports for CQRS operations.
//!
//! This module defines the command and query trait interfaces for the Note
//! aggregate. These are shells for future implementation by adapters.

use async_trait::async_trait;
use uuid::Uuid;

use crate::{errors::DomainError, note::Note};

/// Command port for Note write operations.
///
/// This trait defines the interface for commands that modify Note state.
/// Implementations should be in the adapters layer.
///
/// # Examples
/// ```ignore
/// // Future adapter implementation
/// struct NoteCommandHandler;
///
/// #[async_trait]
/// impl Command for NoteCommandHandler {
///     async fn create(&self, path: String) -> Result<Note, DomainError> {
///         // Implementation would get or generate UUID and persist note
///         let id = self.repository.get_or_create_note_id(&path).await?;
///         Note::new(id, path)
///     }
/// }
/// ```
#[async_trait]
pub trait Command: Send + Sync {
    /// Creates a new note with the given vault-relative path.
    ///
    /// # Errors
    /// Returns `DomainError` if note creation fails validation or persistence.
    async fn create(&self, path: String) -> Result<Note, DomainError>;

    /// Deletes a note by ID.
    ///
    /// # Errors
    /// Returns `DomainError` if note deletion fails.
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;

    /// Updates an existing note.
    ///
    /// # Errors
    /// Returns `DomainError` if note update fails validation or persistence.
    async fn update(&self, note: Note) -> Result<Note, DomainError>;
}

/// Query port for Note read operations.
///
/// This trait defines the interface for queries that retrieve Note state.
/// Implementations should be in the adapters layer.
///
/// # Examples
/// ```ignore
/// // Future adapter implementation
/// struct NoteQueryHandler;
///
/// #[async_trait]
/// impl Query for NoteQueryHandler {
///     async fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, DomainError> {
///         // Implementation would fetch from storage
///         Ok(None)
///     }
/// }
/// ```
#[async_trait]
pub trait Query: Send + Sync {
    /// Finds a note by its UUID v7 identifier.
    ///
    /// # Errors
    /// Returns `DomainError` if query execution fails.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Note>, DomainError>;

    /// Finds a note by its vault-relative path.
    ///
    /// # Errors
    /// Returns `DomainError` if query execution fails.
    async fn find_by_path(
        &self,
        path: &str,
    ) -> Result<Option<Note>, DomainError>;

    /// Lists all notes in the vault.
    ///
    /// # Errors
    /// Returns `DomainError` if query execution fails.
    async fn list(&self) -> Result<Vec<Note>, DomainError>;
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
