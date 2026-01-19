//! Template domain ports.
//!
//! This module defines the trait interfaces for template-related operations,
//! following the CQRS pattern.

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    errors::DomainError,
    template::{Composition, Template},
};

/// Command port for template-related write operations.
#[async_trait]
pub trait Command: Send + Sync {
    /// Creates a new template.
    async fn create(&self, template: Template) -> Result<(), DomainError>;

    /// Deletes a template by ID.
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;

    /// Updates an existing template.
    async fn update(&self, template: Template) -> Result<(), DomainError>;
}

/// Query port for template-related read operations.
#[async_trait]
pub trait Query: Send + Sync {
    /// Find a template by ID.
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<Template>, DomainError>;

    /// Find a template by name.
    async fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Template>, DomainError>;

    /// Lists all templates.
    async fn list_all(&self) -> Result<Vec<Template>, DomainError>;

    /// Resolves a template composition.
    async fn resolve(
        &self,
        composition: Composition,
    ) -> Result<Template, DomainError>;
}
