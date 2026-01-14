//! Template domain ports.
//!
//! This module defines the trait interfaces for template-related operations,
//! following the CQRS pattern.

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    errors::DomainError,
    models::template::{Template, TemplateComposition},
};

/// Command port for template-related write operations.
#[async_trait]
pub trait TemplateCommand: Send + Sync {
    /// Creates a new template.
    async fn create_template(
        &self,
        template: Template,
    ) -> Result<(), DomainError>;

    /// Updates an existing template.
    async fn update_template(
        &self,
        template: Template,
    ) -> Result<(), DomainError>;

    /// Deletes a template by ID.
    async fn delete_template(&self, id: Uuid) -> Result<(), DomainError>;
}

/// Query port for template-related read operations.
#[async_trait]
pub trait TemplateQuery: Send + Sync {
    /// Gets a template by ID.
    async fn get_template(
        &self,
        id: Uuid,
    ) -> Result<Option<Template>, DomainError>;

    /// Gets a template by name.
    async fn get_template_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Template>, DomainError>;

    /// Lists all templates.
    async fn list_templates(&self) -> Result<Vec<Template>, DomainError>;

    /// Resolves a template composition.
    async fn resolve_composition(
        &self,
        composition: TemplateComposition,
    ) -> Result<Template, DomainError>;
}
