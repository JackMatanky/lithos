//! Template command implementations (CQRS write operations).

use uuid::Uuid;

use crate::{
    db::Database,
    template::{
        aggregate::Template,
        db_table::{NAME_TO_ID, TEMPLATES},
        error::TemplateError,
        ports::Command,
    },
};

/// Redb implementation of the template command port.
pub struct CommandAdapter<'db> {
    db: &'db Database,
}

impl<'db> CommandAdapter<'db> {
    /// Creates a new `CommandAdapter` with a database reference.
    #[inline]
    #[must_use]
    pub const fn new(db: &'db Database) -> Self {
        Self {
            db,
        }
    }
}

impl Command for CommandAdapter<'_> {
    #[inline]
    fn create(&self, template: &Template) -> Result<(), TemplateError> {
        let id_str = template.id().to_string();

        self.db
            .put_by_uuid(TEMPLATES, template.id(), template)
            .map_err(|e| TemplateError::Storage(e.to_string()))?;

        self.db
            .multimap_insert(NAME_TO_ID, template.name().as_str(), &id_str)
            .map_err(|e| TemplateError::Storage(e.to_string()))?;

        Ok(())
    }

    #[inline]
    fn delete(&self, id: Uuid) -> Result<(), TemplateError> {
        let id_str = id.to_string();

        let template = self
            .db
            .get_owned::<Template>(TEMPLATES, &id_str)
            .map_err(|e| TemplateError::Storage(e.to_string()))?;

        if let Some(t) = template {
            self.db
                .multimap_remove(NAME_TO_ID, t.name().as_str(), &id_str)
                .map_err(|e| TemplateError::Storage(e.to_string()))?;

            self.db
                .delete_by_uuid(TEMPLATES, id)
                .map_err(|e| TemplateError::Storage(e.to_string()))?;
        }

        Ok(())
    }

    #[inline]
    fn update(&self, template: &Template) -> Result<(), TemplateError> {
        let id_str = template.id().to_string();

        let old_template = self
            .db
            .get_owned::<Template>(TEMPLATES, &template.id().to_string())
            .map_err(|e| TemplateError::Storage(e.to_string()))?;

        if let Some(old) = old_template
            && old.name() != template.name()
        {
            self.db
                .multimap_remove(NAME_TO_ID, old.name().as_str(), &id_str)
                .map_err(|e| TemplateError::Storage(e.to_string()))?;
            self.db
                .multimap_insert(NAME_TO_ID, template.name().as_str(), &id_str)
                .map_err(|e| TemplateError::Storage(e.to_string()))?;
        }

        self.db
            .put_by_uuid(TEMPLATES, template.id(), template)
            .map_err(|e| TemplateError::Storage(e.to_string()))?;

        Ok(())
    }
}
