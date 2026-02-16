//! Template query implementations (CQRS read operations).

use uuid::Uuid;

use super::{
    aggregate::Template, error::TemplateError, ports as template_ports,
};

/// Query implementation for Template read operations.
///
/// This struct is generic over a storage port to support multiple backends.
pub struct Query<Q> {
    port: Q,
}

impl<Q> Query<Q> {
    /// Creates a new `Query` wrapper with a storage port.
    #[inline]
    #[must_use]
    pub const fn new(port: Q) -> Self {
        Self {
            port,
        }
    }
}

impl<Q> Query<Q>
where
    Q: template_ports::Query,
{
    /// Find a template by its ID.
    ///
    /// # Errors
    /// Returns `TemplateError` if query fails.
    #[inline]
    pub fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<Template>, TemplateError> {
        self.port.find_by_id(id)
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
        self.port.find_by_name(name)
    }

    /// List all templates.
    ///
    /// # Errors
    /// Returns `TemplateError` if query fails.
    #[inline]
    pub fn list(&self) -> Result<Vec<Template>, TemplateError> {
        self.port.list()
    }

    /// Access a template with zero-copy.
    ///
    /// # Errors
    /// Returns `TemplateError` if query fails.
    #[inline]
    pub fn with_archived<F, R>(
        &self,
        id: Uuid,
        f: F,
    ) -> Result<Option<R>, TemplateError>
    where
        F: for<'archived> FnOnce(Q::Archived<'archived>) -> R,
    {
        self.port.with_archived(id, f)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tempfile::tempdir;

    use super::*;
    use crate::{
        db::Database,
        template::{
            adapter::{command::CommandAdapter, query::QueryAdapter},
            aggregate::TemplateName,
            ports::Command as _,
        },
    };

    #[test]

    fn with_archived_zero_copy() {
        let temp = tempdir().unwrap();
        let db_path = temp.path().join("test.db");
        let db = Database::open(&db_path).unwrap();

        let adapter = QueryAdapter::new(&db);
        let query = Query::new(adapter);
        let command = CommandAdapter::new(&db);

        let tn = TemplateName::try_from("test").unwrap();
        let template =
            Template::new(&tn, None, vec![], HashMap::new()).unwrap();
        command.create(&template).unwrap();

        // Zero-copy read
        let name_read = query
            .with_archived(template.id(), |archived| {
                archived.name.0.to_string()
            })
            .unwrap()
            .unwrap();

        assert_eq!(name_read, "test");
    }
}
