//! Segregated repository trait definitions for template persistence.
//!
//! Defines [`ReadRepository`] and [`WriteRepository`] following a
//! capability-based segregation pattern. Consumers that only need read access
//! can depend on [`ReadRepository`] without coupling to write logic, and vice
//! versa.
//!
//! The unified [`Repository`] trait is automatically implemented via blanket
//! impl for any type implementing both read and write traits.
//!
//! # Exports
//!
//! - [`ReadRepository`] — read-only template persistence
//! - [`WriteRepository`] — write-only template persistence
//! - [`Repository`] — combined read/write interface (blanket impl)
//!
//! # Implementations
//!
//! - [`crate::template::storage::RedbRepository`] — `redb`-backed production
//!   adapter
//! - [`crate::template::storage::testing::InMemoryRepository`] — in-memory test
//!   double
//!
//! Repository errors are reported via [`TemplateRepositoryError`].

use crate::{
    fs::PathKey,
    template::{
        aggregate::{Template, TemplateId, TemplateName},
        error::TemplateRepositoryError,
        views::RawTemplateView,
    },
};

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

    /// Find a template ID by its vault-relative path.
    ///
    /// Returns `None` if no template exists at the given path.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateRepositoryError`] if the database read fails.
    fn find_template_id_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<TemplateId>, TemplateRepositoryError>;

    /// Find a template by its vault-relative path.
    ///
    /// Returns `None` if no template exists at the given path.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateRepositoryError`] if the database read fails.
    fn find_template_by_path(
        &self,
        path: &PathKey,
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

    /// List all raw template view paths currently cached.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateRepositoryError`] if the database read fails.
    fn list_raw_template_view_paths(
        &self,
    ) -> Result<Vec<PathKey>, TemplateRepositoryError>;

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
