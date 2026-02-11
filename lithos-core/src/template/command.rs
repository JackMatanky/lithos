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
        // Note: Still need id_str for multimap_insert (not UUID-based yet)
        let id_str = template.id().to_string();
        let name = template.name().to_owned();

        self.db
            .put_by_uuid("templates", template.id(), template)
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
        // Note: Still need id_str for multimap_remove (not UUID-based yet)
        let id_str = id.to_string();

        // 1. Get template first to clean up indexes
        let template = self
            .db
            .get_owned_by_uuid::<Template>("templates", id)
            .map_err(|e| TemplateError::Storage(e.to_string()))?;

        if let Some(t) = template {
            // 2. Remove from name index
            self.db
                .multimap_remove("template_name_to_id", t.name(), &id_str)
                .map_err(|e| TemplateError::Storage(e.to_string()))?;

            // 3. Delete template
            self.db
                .delete_by_uuid("templates", id)
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
        // Note: Still need id_str for multimap operations (not UUID-based yet)
        let id_str = template.id().to_string();
        let name = template.name().to_owned();

        // 1. Get old template to find what changed
        let old_template = self
            .db
            .get_owned_by_uuid::<Template>("templates", template.id())
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
            .put_by_uuid("templates", template.id(), template)
            .map_err(|e| TemplateError::Storage(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures and submodules for readability."
)]
mod tests {
    mod fixtures {
        use super::*;

        pub fn test_db() -> Result<(TempDir, Database), String> {
            let dir = tempdir().map_err(|e| e.to_string())?;
            let path = dir.path().join("templates.redb");
            let db = Database::open(&path).map_err(|e| e.to_string())?;
            Ok((dir, db))
        }

        pub fn template_fixture(name: &str) -> Result<Template, String> {
            Template::new(
                name.to_owned(),
                "Hello".to_owned(),
                HashMap::new(),
                None,
                Metadata::default(),
            )
            .map_err(|e| e.to_string())
        }
    }

    use std::collections::HashMap;

    use tempfile::{TempDir, tempdir};

    use super::*;
    use crate::template::{aggregate::Metadata, ports::Command as _};

    mod persistence {
        use super::*;

        #[expect(
            clippy::disallowed_methods,
            reason = "Test fixture uses expect for deterministic setup. \
                      Failure indicates invalid test data. Expect is \
                      idiomatic in setup."
        )]
        fn created_template() -> (TempDir, Database, Template, String) {
            let (dir, db) =
                fixtures::test_db().expect("Failed to create test db");
            let cmd = Command::new(&db);
            let template = fixtures::template_fixture("daily")
                .expect("Failed to create template fixture");
            cmd.create(&template).expect("Create should succeed");
            let id_str = template.id().to_string();
            (dir, db, template, id_str)
        }

        #[expect(
            clippy::disallowed_methods,
            reason = "Test fixture uses expect for deterministic setup. \
                      Failure indicates invalid test data. Expect is \
                      idiomatic in setup."
        )]
        fn updated_template_name() -> (TempDir, Database, String) {
            let (dir, db, mut template, id_str) = created_template();
            let cmd = Command::new(&db);
            template.name = "weekly".to_owned();
            cmd.update(&template).expect("Update should succeed");
            (dir, db, id_str)
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic fixture setup and \
                      value extraction."
        )]
        fn create_persists_template_and_name_index() {
            let (_dir, db, _template, id_str) = created_template();
            let stored = db
                .get_owned::<Template>("templates", &id_str)
                .expect("Read after create should succeed");
            let stored_template = stored.expect("Stored template should exist");
            assert_eq!(
                stored_template.name(),
                "daily",
                "Stored template name should match"
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic fixture setup and \
                      value extraction."
        )]
        fn create_persists_name_index() {
            let (_dir, db, _template, id_str) = created_template();

            let ids = db
                .multimap_get("template_name_to_id", "daily")
                .expect("Name index read should succeed");
            assert!(
                ids.contains(&id_str),
                "Name index should contain template id"
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic fixture setup and \
                      value extraction."
        )]
        fn update_refreshes_name_index_when_name_changes() {
            let (_dir, db, id_str) = updated_template_name();
            let old_ids = db
                .multimap_get("template_name_to_id", "daily")
                .expect("Old name index read should succeed");
            assert!(
                !old_ids.contains(&id_str),
                "Old name index should not contain template id"
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic fixture setup and \
                      value extraction."
        )]
        fn update_adds_new_name_index_entry() {
            let (_dir, db, id_str) = updated_template_name();

            let new_ids = db
                .multimap_get("template_name_to_id", "weekly")
                .expect("New name index read should succeed");
            assert!(
                new_ids.contains(&id_str),
                "New name index should contain template id"
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic fixture setup and \
                      value extraction."
        )]
        fn delete_removes_template_and_name_index() {
            let (_dir, db, template, id_str) = created_template();
            let cmd = Command::new(&db);
            cmd.delete(template.id()).expect("Delete should succeed");
            let stored = db
                .get_owned::<Template>("templates", &id_str)
                .expect("Read after delete should succeed");
            assert!(stored.is_none(), "Deleted template should not exist");
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic fixture setup and \
                      value extraction."
        )]
        fn delete_removes_name_index_entry() {
            let (_dir, db, template, id_str) = created_template();
            let cmd = Command::new(&db);
            cmd.delete(template.id()).expect("Delete should succeed");

            let ids = db
                .multimap_get("template_name_to_id", "daily")
                .expect("Name index read should succeed");
            assert!(
                !ids.contains(&id_str),
                "Name index should not contain deleted template id"
            );
        }
    }
}
