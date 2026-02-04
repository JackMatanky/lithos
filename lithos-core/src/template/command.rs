//! Template command implementations (CQRS write operations).
//!
//! This module implements the Command port trait for Template write operations,
//! using the Database layer for persistence.

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tempfile::{TempDir, tempdir};

    use super::*;
    use crate::template::{aggregate::Metadata, ports::Command as _};

    fn test_db() -> Result<(TempDir, Database), String> {
        let dir = tempdir().map_err(|e| e.to_string())?;
        let path = dir.path().join("templates.redb");
        let db = Database::open(&path).map_err(|e| e.to_string())?;
        Ok((dir, db))
    }

    fn template_fixture(name: &str) -> Result<Template, String> {
        Template::new(
            name.to_owned(),
            "Hello".to_owned(),
            HashMap::new(),
            None,
            Metadata::default(),
        )
        .map_err(|e| e.to_string())
    }

    #[test]
    fn create_persists_template_and_name_index() -> Result<(), String> {
        let (_dir, db) = test_db()?;
        let cmd = Command::new(&db);

        let template = template_fixture("daily")?;
        cmd.create(&template).map_err(|e| e.to_string())?;

        let id_str = template.id().to_string();
        let stored = db
            .get_owned::<Template>("templates", &id_str)
            .map_err(|e| e.to_string())?;
        let Some(stored_template) = stored else {
            return Err("Stored template should exist".to_owned());
        };
        if stored_template.name() != "daily" {
            return Err(format!(
                "Stored template name should match: expected 'daily', got '{}'",
                stored_template.name()
            ));
        }

        let ids = db
            .multimap_get("template_name_to_id", "daily")
            .map_err(|e| e.to_string())?;
        if !ids.contains(&id_str) {
            return Err("Name index should contain template id".to_owned());
        }

        Ok(())
    }

    #[test]
    fn update_refreshes_name_index_when_name_changes() -> Result<(), String> {
        let (_dir, db) = test_db()?;
        let cmd = Command::new(&db);

        let mut template = template_fixture("daily")?;
        cmd.create(&template).map_err(|e| e.to_string())?;

        let id_str = template.id().to_string();
        template.name = "weekly".to_owned();
        cmd.update(&template).map_err(|e| e.to_string())?;

        let old_ids = db
            .multimap_get("template_name_to_id", "daily")
            .map_err(|e| e.to_string())?;
        if old_ids.contains(&id_str) {
            return Err(
                "Old name index should not contain template id".to_owned()
            );
        }

        let new_ids = db
            .multimap_get("template_name_to_id", "weekly")
            .map_err(|e| e.to_string())?;
        if !new_ids.contains(&id_str) {
            return Err("New name index should contain template id".to_owned());
        }

        Ok(())
    }

    #[test]
    fn delete_removes_template_and_name_index() -> Result<(), String> {
        let (_dir, db) = test_db()?;
        let cmd = Command::new(&db);

        let template = template_fixture("daily")?;
        let id = template.id();
        let id_str = id.to_string();
        cmd.create(&template).map_err(|e| e.to_string())?;

        cmd.delete(id).map_err(|e| e.to_string())?;

        let stored = db
            .get_owned::<Template>("templates", &id_str)
            .map_err(|e| e.to_string())?;
        if stored.is_some() {
            return Err("Deleted template should not exist".to_owned());
        }

        let ids = db
            .multimap_get("template_name_to_id", "daily")
            .map_err(|e| e.to_string())?;
        if ids.contains(&id_str) {
            return Err(
                "Name index should not contain deleted template id".to_owned()
            );
        }

        Ok(())
    }
}
