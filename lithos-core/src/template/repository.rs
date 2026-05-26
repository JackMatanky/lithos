//! Template repository traits and marker types.

use crate::template::{
    aggregate::{Template, TemplateId, TemplateName},
    error::TemplateRepositoryError,
};

/// Segregated read interface for template persistence.
pub trait ReadRepository {
    /// Find a template by its unique identifier.
    ///
    /// Returns `None` if no template exists with the given ID.
    ///
    /// # Errors
    /// Returns [`TemplateRepositoryError`] if the database read or
    /// deserialization fails.
    fn find_template_by_id(
        &self,
        id: TemplateId,
    ) -> Result<Option<Template>, TemplateRepositoryError>;

    /// Find multiple templates by ID in a single transaction.
    ///
    /// Returns a vector in the same order as the input IDs.
    /// Missing templates return `None` in the corresponding position.
    ///
    /// # Errors
    /// Returns [`TemplateRepositoryError`] if database read or deserialization
    /// fails.
    fn find_many_templates_by_id(
        &self,
        ids: &[TemplateId],
    ) -> Result<Vec<Option<Template>>, TemplateRepositoryError>;

    /// Find a template ID by its name.
    ///
    /// Returns `None` if no template with the given name exists.
    ///
    /// # Errors
    /// Returns [`TemplateRepositoryError`] if database read or deserialization
    /// fails.
    fn find_template_id_by_name(
        &self,
        name: &TemplateName,
    ) -> Result<Option<TemplateId>, TemplateRepositoryError>;

    /// List all persisted template aggregates.
    ///
    /// # Errors
    /// Returns [`TemplateRepositoryError`] if database read or deserialization
    /// fails.
    fn list_templates(&self) -> Result<Vec<Template>, TemplateRepositoryError>;
}

/// Segregated write interface for template persistence.
pub trait WriteRepository {
    /// Persist a template aggregate to the store.
    ///
    /// # Errors
    /// Returns [`TemplateRepositoryError`] if serialization or database write
    /// fails.
    fn save_template(
        &self,
        template: &Template,
    ) -> Result<(), TemplateRepositoryError>;

    /// Save multiple templates in a single transaction.
    ///
    /// # Errors
    /// Returns [`TemplateRepositoryError`] if serialization or database write
    /// fails.
    fn save_many_templates(
        &self,
        templates: &[Template],
    ) -> Result<(), TemplateRepositoryError>;

    /// Delete a template aggregate and all related indexes.
    ///
    /// # Errors
    /// Returns [`TemplateRepositoryError`] if database write fails.
    fn delete_template(
        &self,
        id: TemplateId,
    ) -> Result<(), TemplateRepositoryError>;
}

/// Unified interface for template persistence and retrieval.
pub trait Repository: ReadRepository + WriteRepository {}

impl<T> Repository for T where T: ReadRepository + WriteRepository {}
