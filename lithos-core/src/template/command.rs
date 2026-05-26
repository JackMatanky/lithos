//! Template command implementations (CQRS write operations).

use super::{
    aggregate::{Template, TemplateId},
    error::TemplateError,
    repository::WriteRepository,
};

/// Command implementation for Template write operations.
///
/// This struct is generic over a storage repository to support multiple
/// backends.
pub struct Command<R> {
    repository: R,
}

impl<R> Command<R> {
    /// Creates a new `Command` wrapper with a storage repository.
    #[inline]
    #[must_use]
    pub const fn new(repository: R) -> Self {
        Self {
            repository,
        }
    }
}

impl<R> Command<R>
where
    R: WriteRepository,
{
    /// Creates a new template.
    ///
    /// # Errors
    /// Returns `TemplateError` if creation fails.
    #[inline]
    pub fn create(&self, template: &Template) -> Result<(), TemplateError> {
        self.repository.save_template(template).map_err(TemplateError::from)
    }

    /// Deletes a template by its unique identifier.
    ///
    /// # Errors
    /// Returns `TemplateError` if deletion fails.
    #[inline]
    pub fn delete(&self, id: TemplateId) -> Result<(), TemplateError> {
        self.repository.delete_template(id).map_err(TemplateError::from)
    }

    /// Updates an existing template.
    ///
    /// # Errors
    /// Returns `TemplateError` if update fails.
    #[inline]
    pub fn update(&self, template: &Template) -> Result<(), TemplateError> {
        self.repository.save_template(template).map_err(TemplateError::from)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::template::{
        aggregate::{Template, TemplateName},
        storage::testing::InMemoryRepository,
    };

    #[test]
    fn command_wrapper_works() {
        let repo = InMemoryRepository::new();
        let cmd = Command::new(repo.clone());

        let tn = TemplateName::try_from("test").unwrap();
        let template =
            Template::try_new(&tn, None, vec![], HashMap::new()).unwrap();

        // Create
        cmd.create(&template).unwrap();

        // Update
        cmd.update(&template).unwrap();

        // Delete
        cmd.delete(template.id()).unwrap();
    }
}
