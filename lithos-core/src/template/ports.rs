//! Template domain ports.

use uuid::Uuid;

use super::{Composition, Template, TemplateError};

/// Command port for template-related write operations.
pub trait Command: Send + Sync {
    /// Creates a new template.
    ///
    /// # Errors
    /// Returns `TemplateError` if creation fails.
    fn create(&self, template: Template) -> Result<(), TemplateError>;

    /// Deletes a template by ID.
    ///
    /// # Errors
    /// Returns `TemplateError` if deletion fails.
    fn delete(&self, id: Uuid) -> Result<(), TemplateError>;

    /// Updates an existing template.
    ///
    /// # Errors
    /// Returns `TemplateError` if update fails.
    fn update(&self, template: Template) -> Result<(), TemplateError>;
}

/// Query port for template-related read operations.
pub trait Query: Send + Sync {
    /// Find a template by ID.
    ///
    /// # Errors
    /// Returns `TemplateError` if query fails.
    fn find_by_id(&self, id: Uuid) -> Result<Option<Template>, TemplateError>;

    /// Find a template by name.
    ///
    /// # Errors
    /// Returns `TemplateError` if query fails.
    fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Template>, TemplateError>;

    /// Lists all templates.
    ///
    /// # Errors
    /// Returns `TemplateError` if query fails.
    fn list(&self) -> Result<Vec<Template>, TemplateError>;

    /// Resolves a template composition.
    ///
    /// # Errors
    /// Returns `TemplateError` if resolution fails.
    fn resolve(
        &self,
        composition: Composition,
    ) -> Result<Template, TemplateError>;
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
