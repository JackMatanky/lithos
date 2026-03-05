//! Template query implementations (CQRS read operations).

use uuid::Uuid;

use crate::{
    db::Database,
    template::{
        aggregate::Template,
        db_table::{NAME_TO_ID, TEMPLATES},
        error::TemplateError,
        ports::Query as QueryPort,
    },
};

/// Redb implementation of the template query port.
pub struct Query<'db> {
    db: &'db Database,
}

impl<'db> Query<'db> {
    /// Creates a new `QueryAdapter` with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }
}

impl QueryPort for Query<'_> {
    type Archived<'archived> = &'archived rkyv::Archived<Template>;

    #[inline]
    fn find_by_id(&self, id: Uuid) -> Result<Option<Template>, TemplateError> {
        self.db.get_owned(TEMPLATES, &id.to_string()).map_err(
            |e: crate::db::DbError| TemplateError::Storage(e.to_string()),
        )
    }

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

    #[inline]
    fn list(&self) -> Result<Vec<Template>, TemplateError> {
        self.db.list_owned::<Template>(TEMPLATES).map_err(
            |e: crate::db::DbError| TemplateError::Storage(e.to_string()),
        )
    }

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
