//! Template query implementations (CQRS read operations).

use super::{
    aggregate::{Template, TemplateId},
    error::TemplateError,
    repository::ReadRepository,
};

/// Query implementation for Template read operations.
///
/// This struct is generic over a storage repository to support multiple
/// backends.
pub struct Query<R> {
    repository: R,
}

impl<R> Query<R> {
    /// Creates a new `Query` wrapper with a storage repository.
    #[inline]
    #[must_use]
    pub const fn new(repository: R) -> Self {
        Self {
            repository,
        }
    }
}

impl<R> Query<R>
where
    R: ReadRepository,
{
    /// Find a template by its ID.
    ///
    /// # Errors
    /// Returns `TemplateError` if query fails.
    #[inline]
    pub fn find_by_id(
        &self,
        id: TemplateId,
    ) -> Result<Option<Template>, TemplateError> {
        self.repository.find_template_by_id(id).map_err(TemplateError::from)
    }

    /// Find a template by its unique name.
    ///
    /// # Errors
    /// Returns `TemplateError` if query fails.
    #[inline]
    pub fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Template>, TemplateError> {
        let name_val =
            crate::template::aggregate::TemplateName::try_from(name)?;
        let id = self
            .repository
            .find_template_id_by_name(&name_val)
            .map_err(TemplateError::from)?;
        match id {
            Some(id) => self.find_by_id(id),
            None => Ok(None),
        }
    }

    /// List all templates.
    ///
    /// # Errors
    /// Returns `TemplateError` if query fails.
    #[inline]
    pub fn list(&self) -> Result<Vec<Template>, TemplateError> {
        self.repository.list_templates().map_err(TemplateError::from)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::template::{
        aggregate::TemplateName, repository::WriteRepository,
        storage::testing::InMemoryRepository,
    };

    #[test]
    fn query_wrapper_works() {
        let repo = InMemoryRepository::new();
        let query = Query::new(repo.clone());

        let tn = TemplateName::try_from("test").unwrap();
        let template =
            Template::try_new(&tn, None, vec![], HashMap::new()).unwrap();
        repo.save_template(&template).unwrap();

        let found = query.find_by_id(template.id()).unwrap().unwrap();
        assert_eq!(found.name(), &tn);

        let found_by_name = query.find_by_name("test").unwrap().unwrap();
        assert_eq!(found_by_name.id(), template.id());
    }
}
