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

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Test code uses unwrap/expect for clarity"
)]
mod tests {
    use std::collections::HashMap;

    use tempfile::tempdir;

    use super::*;
    use crate::template::{aggregate::Metadata, ports::Command as _};

    fn template_fixture(name: &str) -> Template {
        Template::new(
            name.to_owned(),
            "Hello".to_owned(),
            HashMap::new(),
            None,
            Metadata::default(),
        )
        .unwrap()
    }

    #[test]
    fn create_persists_template_and_name_index() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("templates.redb");
        let db = Database::open(&path).unwrap();
        let cmd = Command::new(&db);

        let template = template_fixture("daily");
        cmd.create(&template).unwrap();

        let id_str = template.id().to_string();
        let stored = db.get_owned::<Template>("templates", &id_str).unwrap();
        let stored_template = stored.expect("Stored template should exist");
        assert_eq!(
            stored_template.name(),
            "daily",
            "Stored template name should match"
        );

        let ids = db.multimap_get("template_name_to_id", "daily").unwrap();
        assert!(ids.contains(&id_str), "Name index should contain template id");
    }

    #[test]
    fn update_refreshes_name_index_when_name_changes() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("templates.redb");
        let db = Database::open(&path).unwrap();
        let cmd = Command::new(&db);

        let mut template = template_fixture("daily");
        cmd.create(&template).unwrap();

        let id_str = template.id().to_string();
        template.name = "weekly".to_owned();
        cmd.update(&template).unwrap();

        let old_ids = db.multimap_get("template_name_to_id", "daily").unwrap();
        assert!(
            !old_ids.contains(&id_str),
            "Old name index should not contain template id"
        );

        let new_ids = db.multimap_get("template_name_to_id", "weekly").unwrap();
        assert!(
            new_ids.contains(&id_str),
            "New name index should contain template id"
        );
    }

    #[test]
    fn delete_removes_template_and_name_index() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("templates.redb");
        let db = Database::open(&path).unwrap();
        let cmd = Command::new(&db);

        let template = template_fixture("daily");
        let id = template.id();
        let id_str = id.to_string();
        cmd.create(&template).unwrap();

        cmd.delete(id).unwrap();

        let stored = db.get_owned::<Template>("templates", &id_str).unwrap();
        assert!(stored.is_none(), "Deleted template should not exist");

        let ids = db.multimap_get("template_name_to_id", "daily").unwrap();
        assert!(
            !ids.contains(&id_str),
            "Name index should not contain deleted template id"
        );
    }
}
