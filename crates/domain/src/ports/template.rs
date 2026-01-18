//! Template domain ports.
//!
//! This module defines the trait interfaces for template-related operations,
//! following the CQRS pattern.

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    errors::DomainError,
    models::{template::Template, template_comp::Composition},
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
    /// Gets a template by ID.
    async fn get(&self, id: Uuid) -> Result<Option<Template>, DomainError>;

    /// Gets a template by name.
    async fn get_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Template>, DomainError>;

    /// Lists all templates.
    async fn list_all(&self) -> Result<Vec<Template>, DomainError>;

    /// Resolves a template composition.
    async fn resolve_composition(
        &self,
        composition: Composition,
    ) -> Result<Template, DomainError>;
}
