//! Template read repository implementation for redb.

use redb::ReadableTable as _;

use crate::{
    db::ArchivedEntity,
    template::{
        aggregate::{Template, TemplateId, TemplateName},
        error::TemplateRepositoryError,
        repository::ReadRepository,
        storage::{
            RedbRepository,
            tables::{NAME_TO_ID, TEMPLATES},
        },
    },
};

impl ReadRepository for RedbRepository {
    #[inline]
    fn find_template_by_id(
        &self,
        id: TemplateId,
    ) -> Result<Option<Template>, TemplateRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) = tx.try_open_table(TEMPLATES.definition())?
                else {
                    return Ok(None);
                };

                let Some(guard) = table.get(id)? else {
                    return Ok(None);
                };

                Ok(Some(Template::from_bytes(guard.value())?))
            })
            .map_err(TemplateRepositoryError::from)
    }

    #[inline]
    fn find_many_templates_by_id(
        &self,
        ids: &[TemplateId],
    ) -> Result<Vec<Option<Template>>, TemplateRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) = tx.try_open_table(TEMPLATES.definition())?
                else {
                    return Ok(ids.iter().map(|_| None).collect());
                };

                let mut results = Vec::with_capacity(ids.len());
                for id in ids {
                    let template = table
                        .get(*id)?
                        .map(|g| Template::from_bytes(g.value()))
                        .transpose()?;
                    results.push(template);
                }
                Ok(results)
            })
            .map_err(TemplateRepositoryError::from)
    }

    #[inline]
    fn find_template_id_by_name(
        &self,
        name: &TemplateName,
    ) -> Result<Option<TemplateId>, TemplateRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) = tx.try_open_table(NAME_TO_ID.definition())?
                else {
                    return Ok(None);
                };

                let Some(guard) = table.get(name.as_str())? else {
                    return Ok(None);
                };

                Ok(Some(<TemplateId as redb::Value>::from_bytes(guard.value())))
            })
            .map_err(TemplateRepositoryError::from)
    }

    #[inline]
    fn list_templates(&self) -> Result<Vec<Template>, TemplateRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) = tx.try_open_table(TEMPLATES.definition())?
                else {
                    return Ok(Vec::new());
                };

                let mut results = Vec::new();
                for entry in table.iter()? {
                    let (_, guard) = entry?;
                    results.push(Template::from_bytes(guard.value())?);
                }
                Ok(results)
            })
            .map_err(TemplateRepositoryError::from)
    }
}
