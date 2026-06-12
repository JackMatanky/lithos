//! Template repository trait and error types.

use crate::{
    db::DbError,
    fs::PathKey,
    template::{
        aggregate::{Template, TemplateId, TemplateName},
        views::RawTemplateView,
    },
};

/// Errors returned by template repository implementations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TemplateRepositoryError {
    /// Returned when the underlying storage layer fails.
    #[error(transparent)]
    Storage(#[from] DbError),

    /// Returned when a template is not found by its ID.
    #[error("template not found: {0}")]
    NotFoundById(TemplateId),

    /// Returned when a template is not found by its path.
    #[error("template path not found: {0}")]
    NotFoundByPath(PathKey),
}

/// Segregated read interface for template persistence.
pub trait ReadRepository {
    /// Find a template by its unique identifier.
    ///
    /// Returns `None` if no template exists with the given ID.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateRepositoryError`] if the database read fails.
    fn find_template_by_id(
        &self,
        id: TemplateId,
    ) -> Result<Option<Template>, TemplateRepositoryError>;

    /// Find a template by its derived name.
    ///
    /// Returns `None` if no template exists with the given name.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateRepositoryError`] if the database read fails.
    fn find_template_by_name(
        &self,
        name: &TemplateName,
    ) -> Result<Option<Template>, TemplateRepositoryError>;

    /// List all persisted template aggregates.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateRepositoryError`] if the database read fails.
    fn list_templates(&self) -> Result<Vec<Template>, TemplateRepositoryError>;

    /// Find a raw template view by vault-relative path.
    ///
    /// Returns `None` if no view exists for the given path.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateRepositoryError`] if the database read fails.
    fn find_raw_template_view(
        &self,
        path: &PathKey,
    ) -> Result<Option<RawTemplateView>, TemplateRepositoryError>;

    /// Find raw template views by a set of paths in a single transaction.
    ///
    /// Returns a vector in the same order as the input paths.
    /// Missing views return `None` in the corresponding position.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateRepositoryError`] if the database read fails.
    fn find_raw_template_views_by_paths(
        &self,
        paths: &[PathKey],
    ) -> Result<Vec<Option<RawTemplateView>>, TemplateRepositoryError>;
}

/// Segregated write interface for template persistence.
pub trait WriteRepository {
    /// Persist a template aggregate to the store.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateRepositoryError`] if the database write fails.
    fn save_template(
        &self,
        template: &Template,
    ) -> Result<(), TemplateRepositoryError>;

    /// Delete a template aggregate by ID.
    ///
    /// Idempotent: returns `Ok(())` if the template does not exist.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateRepositoryError`] if the database write fails.
    fn delete_template(
        &self,
        id: TemplateId,
    ) -> Result<(), TemplateRepositoryError>;

    /// Persist a raw template view to the store.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateRepositoryError`] if the database write fails.
    fn save_raw_template_view(
        &self,
        view: &RawTemplateView,
    ) -> Result<(), TemplateRepositoryError>;

    /// Delete a raw template view by vault-relative path.
    ///
    /// Idempotent: returns `Ok(())` if the view does not exist.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateRepositoryError`] if the database write fails.
    fn delete_raw_template_view(
        &self,
        path: &PathKey,
    ) -> Result<(), TemplateRepositoryError>;

    /// Save multiple raw template views in a single transaction.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateRepositoryError`] if the database write fails.
    fn save_many_raw_template_views(
        &self,
        views: &[RawTemplateView],
    ) -> Result<(), TemplateRepositoryError>;
}

/// Unified interface for template persistence and retrieval.
///
/// This trait extends both [`ReadRepository`] and [`WriteRepository`] to
/// provide a complete interface for template storage operations. It is
/// automatically implemented via blanket impl for any type implementing
/// both read and write traits.
///
/// # Blanket Implementation
///
/// ```rust,ignore
/// impl<T> Repository for T
/// where
///     T: ReadRepository + WriteRepository
/// {}
/// ```
pub trait Repository: ReadRepository + WriteRepository {}

impl<T: ReadRepository + WriteRepository> Repository for T {}
