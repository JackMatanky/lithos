//! Template command implementations (CQRS write operations).
//!
//! This module implements the Command port trait for Template write operations,
//! using the Database layer for persistence.

#![allow(
    clippy::missing_inline_in_public_items,
    clippy::elidable_lifetime_names,
    reason = "CQRS pattern: trait impls don't need inline"
)]

use uuid::Uuid;

use super::{aggregate::Template, error::TemplateError};
use crate::db::Database;

/// Command implementation for Template write operations.
///
/// Implements the Command port trait using the Database layer.
pub struct Command<'db> {
    db: &'db Database,
}

impl<'db> Command<'db> {
    /// Create a new `Command` with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }
}

impl super::ports::Command for Command<'_> {
    /// Creates a new template.
    ///
    /// # Errors
    /// Returns `TemplateError` if creation fails.
    #[inline]
    fn create(&self, template: &Template) -> Result<(), TemplateError> {
        let id_str = template.id().to_string();
        let name = template.name().to_owned();

        self.db
            .put("templates", &id_str, template)
            .map_err(|e| TemplateError::Storage(e.to_string()))?;

        self.db
            .multimap_insert("template_name_to_id", &name, &id_str)
            .map_err(|e| TemplateError::Storage(e.to_string()))?;

        Ok(())
    }

    /// Deletes a template by ID.
    ///
    /// # Errors
    /// Returns `TemplateError` if deletion fails.
    #[inline]
    fn delete(&self, id: Uuid) -> Result<(), TemplateError> {
        let id_str = id.to_string();

        // 1. Get template first to clean up indexes
        let template = self
            .db
            .get_owned::<Template>("templates", &id_str)
            .map_err(|e| TemplateError::Storage(e.to_string()))?;

        if let Some(t) = template {
            // 2. Remove from name index
            self.db
                .multimap_remove("template_name_to_id", t.name(), &id_str)
                .map_err(|e| TemplateError::Storage(e.to_string()))?;

            // 3. Delete template
            self.db
                .delete("templates", &id_str)
                .map_err(|e| TemplateError::Storage(e.to_string()))?;
        }

        Ok(())
    }

    /// Updates an existing template.
    ///
    /// # Errors
    /// Returns `TemplateError` if update fails.
    #[inline]
    fn update(&self, template: &Template) -> Result<(), TemplateError> {
        let id_str = template.id().to_string();
        let name = template.name().to_owned();

        // 1. Get old template to find what changed
        let old_template = self
            .db
            .get_owned::<Template>("templates", &id_str)
            .map_err(|e| TemplateError::Storage(e.to_string()))?;

        if let Some(old) = old_template {
            // 2. Update name index if changed
            if old.name() != template.name() {
                self.db
                    .multimap_remove("template_name_to_id", old.name(), &id_str)
                    .map_err(|e| TemplateError::Storage(e.to_string()))?;
                self.db
                    .multimap_insert("template_name_to_id", &name, &id_str)
                    .map_err(|e| TemplateError::Storage(e.to_string()))?;
            }
        }

        // 3. Save new template
        self.db
            .put("templates", &id_str, template)
            .map_err(|e| TemplateError::Storage(e.to_string()))?;

        Ok(())
    }
}
