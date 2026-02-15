//! Schema bounded context ports for CQRS operations.
//!
//! This module defines the command and query trait interfaces for the Schema
//! aggregate.

use super::{
    aggregate::{Schema, SchemaId, SchemaName},
    error::SchemaError,
};

/// Command port for Schema write operations.
pub trait Command: Send + Sync {
    /// Delete a schema by ID.
    ///
    /// # Errors
    /// Returns `SchemaError` if deletion fails.
    fn delete(&self, name: &SchemaName) -> Result<(), SchemaError>;

    /// Save a schema to persistence.
    ///
    /// # Errors
    /// Returns `SchemaError` if saving fails.
    fn save(&self, schema: &Schema) -> Result<(), SchemaError>;
}

/// Query port for Schema read operations.
pub trait Query: Send + Sync {
    /// Find a schema by its ID.
    ///
    /// # Errors
    /// Returns `SchemaError` if query fails.
    fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, SchemaError>;

    /// Find a schema by its unique name.
    ///
    /// # Errors
    /// Returns `SchemaError` if query fails.
    fn find_by_name(
        &self,
        name: &SchemaName,
    ) -> Result<Option<Schema>, SchemaError>;

    /// List all available schemas.
    ///
    /// # Errors
    /// Returns `SchemaError` if query fails.
    fn list(&self) -> Result<Vec<Schema>, SchemaError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_trait_is_object_safe() {
        let _: Option<Box<dyn Command>> = None;
    }

    #[test]
    fn query_trait_is_object_safe() {
        let _: Option<Box<dyn Query>> = None;
    }

    #[test]
    fn traits_are_send_and_sync() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn Command>();
        assert_send_sync::<dyn Query>();
    }
}
