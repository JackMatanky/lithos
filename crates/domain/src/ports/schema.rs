//! Schema domain ports for CQRS separation.

use async_trait::async_trait;
use uuid::Uuid;

use crate::{errors::DomainError, models::schema::Schema};

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
    async fn list_all(&self) -> Result<Vec<Schema>, DomainError>;
}
