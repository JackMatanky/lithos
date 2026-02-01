//! Schema domain ports for CQRS separation.

use async_trait::async_trait;
use uuid::Uuid;

use crate::{errors::DomainError, schema::aggregate::Schema};

/// Command port for Schema bounded context.
#[async_trait]
pub trait Command: Send + Sync {
    /// Delete a schema by ID.
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;

    /// Save a schema to persistence.
    async fn save(&self, schema: Schema) -> Result<(), DomainError>;
}

/// Query port for Schema bounded context.
#[async_trait]
pub trait Query: Send + Sync {
    /// Find a schema by its ID.
    async fn find_by_id(&self, id: Uuid)
    -> Result<Option<Schema>, DomainError>;

    /// Find a schema by its unique name.
    async fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Schema>, DomainError>;

    /// List all available schemas.
    async fn list(&self) -> Result<Vec<Schema>, DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_trait_is_object_safe() {
        // GIVEN: the Command trait
        // WHEN: checking for object safety
        let _: Option<Box<dyn Command>> = None;
        // THEN: it compiles
    }

    #[test]
    fn query_trait_is_object_safe() {
        // GIVEN: the Query trait
        // WHEN: checking for object safety
        let _: Option<Box<dyn Query>> = None;
        // THEN: it compiles
    }

    #[test]
    fn traits_are_send_and_sync() {
        // GIVEN: the port traits
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}

        // WHEN: checking Send + Sync bounds
        assert_send_sync::<dyn Command>();
        assert_send_sync::<dyn Query>();

        // THEN: they satisfy the bounds
    }
}
