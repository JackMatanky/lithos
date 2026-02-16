//! Template query implementations (CQRS read operations).
//!
//! This module implements the Query port trait for Template read operations,
//! using the Database layer for zero-copy reads.

use uuid::Uuid;

use super::{aggregate::Template, error::TemplateError};
use crate::{
    db::Database,
    template::db_table::{NAME_TO_ID, TEMPLATES},
};

/// Query implementation for Template read operations.
///
/// Implements the Query port trait using the Database layer.
pub struct RedbTemplateQuery<'db> {
    db: &'db Database,
}

impl<'db> RedbTemplateQuery<'db> {
    /// Create a new `RedbTemplateQuery` with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }
}

impl super::ports::Query for RedbTemplateQuery<'_> {
    type Archived<'archived> = &'archived rkyv::Archived<Template>;

    /// Find a template by ID.
    ///
    /// # Errors
    /// Returns `TemplateError` if query fails.
    #[inline]
    fn find_by_id(&self, id: Uuid) -> Result<Option<Template>, TemplateError> {
        self.db.get_owned(TEMPLATES, &id.to_string()).map_err(
            |e: crate::db::DbError| TemplateError::Storage(e.to_string()),
        )
    }

    /// Find a template by name.
    ///
    /// # Errors
    /// Returns `TemplateError` if query fails.
    #[inline]
    fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Template>, TemplateError> {
        let ids = self.db.multimap_get(NAME_TO_ID, name).map_err(
            |e: crate::db::DbError| TemplateError::Storage(e.to_string()),
        )?;

        if let Some(id_str) = ids.first() {
            self.db.get_owned::<Template>(TEMPLATES, id_str).map_err(
                |e: crate::db::DbError| TemplateError::Storage(e.to_string()),
            )
        } else {
            Ok(None)
        }
    }

    /// Lists all templates.
    ///
    /// # Errors
    /// Returns `TemplateError` if query fails.
    #[inline]
    fn list(&self) -> Result<Vec<Template>, TemplateError> {
        self.db.list_owned::<Template>(TEMPLATES).map_err(
            |e: crate::db::DbError| TemplateError::Storage(e.to_string()),
        )
    }

    /// Access a template with zero-copy.
    ///
    /// # Errors
    /// Returns `TemplateError` if query fails.
    #[inline]
    fn with_archived<F, R>(
        &self,
        id: Uuid,
        f: F,
    ) -> Result<Option<R>, TemplateError>
    where
        F: for<'archived> FnOnce(Self::Archived<'archived>) -> R,
    {
        self.db
            .get::<Template, _, _>(TEMPLATES, &id.to_string(), |archived| {
                f(archived)
            })
            .map_err(|e| TemplateError::Storage(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tempfile::tempdir;

    use super::*;
    use crate::template::{
        aggregate::TemplateName,
        command::Command,
        ports::{Command as _, Query as _},
    };

    #[test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Tests use unwrap for concise setup"
    )]
    fn with_archived_zero_copy() {
        let temp = tempdir().unwrap();
        let db_path = temp.path().join("test.db");
        let db = Database::open(&db_path).unwrap();

        let query = RedbTemplateQuery::new(&db);
        let command = Command::new(&db);

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
