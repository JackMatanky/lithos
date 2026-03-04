//! Template command implementations (CQRS write operations).

use uuid::Uuid;

use super::{
    aggregate::Template, error::TemplateError, ports as template_ports,
};

/// Command implementation for Template write operations.
///
/// This struct is generic over a storage port to support multiple backends.
pub struct Command<C> {
    port: C,
}

impl<C> Command<C> {
    /// Creates a new `Command` wrapper with a storage port.
    #[inline]
    #[must_use]
    pub const fn new(port: C) -> Self {
        Self {
            port,
        }
    }
}

impl<C> Command<C>
where
    C: template_ports::Command,
{
    /// Creates a new template.
    ///
    /// # Errors
    /// Returns `TemplateError` if creation fails.
    #[inline]
    pub fn create(&self, template: &Template) -> Result<(), TemplateError> {
        self.port.create(template)
    }

    /// Deletes a template by its unique identifier.
    ///
    /// # Errors
    /// Returns `TemplateError` if deletion fails.
    #[inline]
    pub fn delete(&self, id: Uuid) -> Result<(), TemplateError> {
        self.port.delete(id)
    }

    /// Updates an existing template.
    ///
    /// # Errors
    /// Returns `TemplateError` if update fails.
    #[inline]
    pub fn update(&self, template: &Template) -> Result<(), TemplateError> {
        self.port.update(template)
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures and submodules for readability."
)]
mod tests {
    mod fixtures {
        use std::collections::HashMap;

        use tempfile::{TempDir, tempdir};

        use crate::{
            db::Database,
            template::aggregate::{Template, TemplateName},
        };

        pub fn test_db() -> Result<(TempDir, Database), String> {
            let dir = tempdir().map_err(|e| e.to_string())?;
            let path = dir.path().join("templates.redb");
            let db = Database::open(&path).map_err(|e| e.to_string())?;
            Ok((dir, db))
        }

        pub fn template_fixture(name: &str) -> Result<Template, String> {
            let tn = TemplateName::try_from(name).map_err(|e| e.to_string())?;
            Template::try_new(&tn, None, vec![], HashMap::new())
                .map_err(|e| e.to_string())
        }
    }

    use tempfile::TempDir;

    use super::*;
    use crate::{
        db::Database,
        template::{adapter::command::CommandAdapter, aggregate::Template},
    };

    mod persistence {
        use super::*;

        fn created_template() -> (TempDir, Database, Template, String) {
            let (dir, db) =
                fixtures::test_db().expect("Failed to create test db");
            let adapter = CommandAdapter::new(&db);
            let cmd = Command::new(adapter);
            let template = fixtures::template_fixture("daily")
                .expect("Failed to create template fixture");
            cmd.create(&template).expect("Create should succeed");
            let id_str = template.id().to_string();
            (dir, db, template, id_str)
        }

        #[test]

        fn create_persists_template() {
            let (_dir, db, _template, id_str) = created_template();
            let stored = db
                .get_owned::<Template>(
                    crate::template::db_table::TEMPLATES,
                    &id_str,
                )
                .expect("Read after create should succeed");
            let stored_template = stored.expect("Stored template should exist");
            assert_eq!(
                stored_template.name().as_str(),
                "daily",
                "Stored template name should match"
            );
        }
    }
}
