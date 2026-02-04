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
    fn create_persists_template_and_name_index() {
        let db_result = test_db();
        assert!(db_result.is_ok(), "Failed to create test db: {db_result:?}");
        let Ok((_dir, db)) = db_result else {
            return;
        };
        let cmd = Command::new(&db);

        let template_result = template_fixture("daily");
        assert!(
            template_result.is_ok(),
            "Failed to create template fixture: {template_result:?}"
        );
        let Ok(template) = template_result else {
            return;
        };

        let create_result = cmd.create(&template).map_err(|e| e.to_string());
        assert!(
            create_result.is_ok(),
            "Create should succeed, got: {create_result:?}"
        );

        let id_str = template.id().to_string();
        let stored_result = db
            .get_owned::<Template>("templates", &id_str)
            .map_err(|e| e.to_string());
        assert!(
            stored_result.is_ok(),
            "Read after create should succeed, got: {stored_result:?}"
        );
        let Ok(stored) = stored_result else {
            return;
        };
        assert!(stored.is_some(), "Stored template should exist");
        let Some(stored_template) = stored else {
            return;
        };
        assert_eq!(
            stored_template.name(),
            "daily",
            "Stored template name should match"
        );

        let ids_result = db
            .multimap_get("template_name_to_id", "daily")
            .map_err(|e| e.to_string());
        assert!(
            ids_result.is_ok(),
            "Name index read should succeed, got: {ids_result:?}"
        );
        let Ok(ids) = ids_result else {
            return;
        };
        assert!(ids.contains(&id_str), "Name index should contain template id");
    }

    #[test]
    fn update_refreshes_name_index_when_name_changes() {
        let db_result = test_db();
        assert!(db_result.is_ok(), "Failed to create test db: {db_result:?}");
        let Ok((_dir, db)) = db_result else {
            return;
        };
        let cmd = Command::new(&db);

        let template_result = template_fixture("daily");
        assert!(
            template_result.is_ok(),
            "Failed to create template fixture: {template_result:?}"
        );
        let Ok(mut template) = template_result else {
            return;
        };

        let create_result = cmd.create(&template).map_err(|e| e.to_string());
        assert!(
            create_result.is_ok(),
            "Create should succeed, got: {create_result:?}"
        );

        let id_str = template.id().to_string();
        template.name = "weekly".to_owned();
        let update_result = cmd.update(&template).map_err(|e| e.to_string());
        assert!(
            update_result.is_ok(),
            "Update should succeed, got: {update_result:?}"
        );

        let old_ids_result = db
            .multimap_get("template_name_to_id", "daily")
            .map_err(|e| e.to_string());
        assert!(
            old_ids_result.is_ok(),
            "Old name index read should succeed, got: {old_ids_result:?}"
        );
        let Ok(old_ids) = old_ids_result else {
            return;
        };
        assert!(
            !old_ids.contains(&id_str),
            "Old name index should not contain template id"
        );

        let new_ids_result = db
            .multimap_get("template_name_to_id", "weekly")
            .map_err(|e| e.to_string());
        assert!(
            new_ids_result.is_ok(),
            "New name index read should succeed, got: {new_ids_result:?}"
        );
        let Ok(new_ids) = new_ids_result else {
            return;
        };
        assert!(
            new_ids.contains(&id_str),
            "New name index should contain template id"
        );
    }

    #[test]
    fn delete_removes_template_and_name_index() {
        let db_result = test_db();
        assert!(db_result.is_ok(), "Failed to create test db: {db_result:?}");
        let Ok((_dir, db)) = db_result else {
            return;
        };
        let cmd = Command::new(&db);

        let template_result = template_fixture("daily");
        assert!(
            template_result.is_ok(),
            "Failed to create template fixture: {template_result:?}"
        );
        let Ok(template) = template_result else {
            return;
        };
        let id = template.id();
        let id_str = id.to_string();
        let create_result = cmd.create(&template).map_err(|e| e.to_string());
        assert!(
            create_result.is_ok(),
            "Create should succeed, got: {create_result:?}"
        );

        let delete_result = cmd.delete(id).map_err(|e| e.to_string());
        assert!(
            delete_result.is_ok(),
            "Delete should succeed, got: {delete_result:?}"
        );

        let stored_result = db
            .get_owned::<Template>("templates", &id_str)
            .map_err(|e| e.to_string());
        assert!(
            stored_result.is_ok(),
            "Read after delete should succeed, got: {stored_result:?}"
        );
        let Ok(stored) = stored_result else {
            return;
        };
        assert!(stored.is_none(), "Deleted template should not exist");

        let ids_result = db
            .multimap_get("template_name_to_id", "daily")
            .map_err(|e| e.to_string());
        assert!(
            ids_result.is_ok(),
            "Name index read should succeed, got: {ids_result:?}"
        );
        let Ok(ids) = ids_result else {
            return;
        };
        assert!(
            !ids.contains(&id_str),
            "Name index should not contain deleted template id"
        );
    }
}
