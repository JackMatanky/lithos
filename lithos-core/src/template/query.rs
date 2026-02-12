//! Template query implementations (CQRS read operations).
//!
//! This module implements the Query port trait for Template read operations,
//! using the Database layer for zero-copy reads.

use std::collections::HashMap;

use uuid::Uuid;

use super::{
    aggregate::Template, composition::Composition, error::TemplateError,
};
use crate::{
    db::Database,
    template::db_table::{NAME_TO_ID, TEMPLATES},
};

/// Query implementation for Template read operations.
///
/// Implements the Query port trait using the Database layer.
pub struct Query<'db> {
    db: &'db Database,
}

impl<'db> Query<'db> {
    /// Create a new `Query` with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }
}

impl super::ports::Query for Query<'_> {
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

    /// Resolves a template composition.
    ///
    /// # Errors
    /// Returns `TemplateError` if resolution fails.
    #[inline]
    fn resolve(
        &self,
        composition: &Composition,
    ) -> Result<Template, TemplateError> {
        let base = self.find_by_name(&composition.base_template)?.ok_or_else(
            || TemplateError::NotFound(composition.base_template.clone()),
        )?;

        // We need all templates for cycle detection and include resolution
        let all_templates_list = self.list()?;

        // Use borrowed keys to avoid allocating template names
        let all_templates: HashMap<&str, &Template> =
            all_templates_list.iter().map(|t| (t.name(), t)).collect();

        Template::compose(&base, composition, &all_templates)
    }
}
