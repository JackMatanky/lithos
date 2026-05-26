//! Template write repository implementation for redb.

use redb::ReadableTable as _;

use crate::{
    db::ArchivedEntity,
    template::{
        aggregate::{Template, TemplateId},
        error::TemplateRepositoryError,
        repository::WriteRepository,
        storage::{
            RedbRepository,
            tables::{NAME_TO_ID, TEMPLATES},
        },
    },
};

impl WriteRepository for RedbRepository {
    #[inline]
    fn save_template(
        &self,
        template: &Template,
    ) -> Result<(), TemplateRepositoryError> {
        let bytes =
            template.to_bytes().map_err(TemplateRepositoryError::from)?;

        self.store
            .write(|tx| {
                let mut templates =
                    tx.try_open_table(TEMPLATES.definition())?;
                let mut name_to_id =
                    tx.try_open_table(NAME_TO_ID.definition())?;

                // If template already exists, update name index
                let old_name =
                    if let Some(old_bytes) = templates.get(template.id())? {
                        let old = Template::from_bytes(old_bytes.value())?;
                        if old.name() == template.name() {
                            None
                        } else {
                            Some(old.name().clone())
                        }
                    } else {
                        None
                    };

                if let Some(name) = old_name {
                    name_to_id.remove(name.as_str())?;
                }

                templates.insert(template.id(), bytes.as_slice())?;

                name_to_id.insert(
                    template.name().as_str(),
                    <TemplateId as redb::Value>::as_bytes(&template.id())
                        .as_slice(),
                )?;

                Ok(())
            })
            .map_err(TemplateRepositoryError::from)
    }

    #[inline]
    fn save_many_templates(
        &self,
        templates: &[Template],
    ) -> Result<(), TemplateRepositoryError> {
        // Pre-serialize all templates to minimize critical section
        let mut serialized = Vec::with_capacity(templates.len());
        for template in templates {
            serialized.push(
                template.to_bytes().map_err(TemplateRepositoryError::from)?,
            );
        }

        self.store
            .write(|tx| {
                let mut templates_table =
                    tx.try_open_table(TEMPLATES.definition())?;
                let mut name_to_id =
                    tx.try_open_table(NAME_TO_ID.definition())?;

                for (template, bytes) in templates.iter().zip(&serialized) {
                    let old_name = if let Some(old_bytes) =
                        templates_table.get(template.id())?
                    {
                        let old = Template::from_bytes(old_bytes.value())?;
                        (old.name() != template.name())
                            .then(|| old.name().clone())
                    } else {
                        None
                    };

                    if let Some(name) = old_name {
                        name_to_id.remove(name.as_str())?;
                    }

                    templates_table.insert(template.id(), bytes.as_slice())?;
                    name_to_id.insert(
                        template.name().as_str(),
                        <TemplateId as redb::Value>::as_bytes(&template.id())
                            .as_slice(),
                    )?;
                }
                Ok(())
            })
            .map_err(TemplateRepositoryError::from)
    }

    #[inline]
    fn delete_template(
        &self,
        id: TemplateId,
    ) -> Result<(), TemplateRepositoryError> {
        self.store
            .write(|tx| {
                let mut templates =
                    tx.try_open_table(TEMPLATES.definition())?;
                let mut name_to_id =
                    tx.try_open_table(NAME_TO_ID.definition())?;

                let name_to_remove =
                    if let Some(template_bytes) = templates.get(id)? {
                        let template =
                            Template::from_bytes(template_bytes.value())?;
                        Some(template.name().clone())
                    } else {
                        None
                    };

                if let Some(name) = name_to_remove {
                    name_to_id.remove(name.as_str())?;
                    templates.remove(id)?;
                }
                Ok(())
            })
            .map_err(TemplateRepositoryError::from)
    }
}
