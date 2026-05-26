//! Template storage implementations.

pub mod read;
pub(crate) mod tables;
pub mod write;

#[cfg(any(test, feature = "testing"))]
pub mod testing;

use std::sync::Arc;

use crate::{
    db::Store,
    template::repository::{ReadRepository, WriteRepository},
};

/// Redb-backed implementation of the template repository traits.
#[derive(Debug, Clone)]
pub struct RedbRepository {
    store: Arc<Store>,
}

impl RedbRepository {
    /// Creates a new `RedbRepository` with the specified store.
    #[must_use]
    #[inline]
    pub fn new(store: Arc<Store>) -> Self {
        Self {
            store,
        }
    }
}

impl ReadRepository for RedbRepository {
    #[inline]
    fn find_template_by_id(
        &self,
        id: crate::template::aggregate::TemplateId,
    ) -> Result<
        Option<crate::template::aggregate::Template>,
        crate::template::error::TemplateRepositoryError,
    > {
        self.read_repo().find_template_by_id(id)
    }

    #[inline]
    fn find_many_templates_by_id(
        &self,
        ids: &[crate::template::aggregate::TemplateId],
    ) -> Result<
        Vec<Option<crate::template::aggregate::Template>>,
        crate::template::error::TemplateRepositoryError,
    > {
        self.read_repo().find_many_templates_by_id(ids)
    }

    #[inline]
    fn find_template_id_by_name(
        &self,
        name: &crate::template::aggregate::TemplateName,
    ) -> Result<
        Option<crate::template::aggregate::TemplateId>,
        crate::template::error::TemplateRepositoryError,
    > {
        self.read_repo().find_template_id_by_name(name)
    }

    #[inline]
    fn list_templates(
        &self,
    ) -> Result<
        Vec<crate::template::aggregate::Template>,
        crate::template::error::TemplateRepositoryError,
    > {
        self.read_repo().list_templates()
    }
}

impl WriteRepository for RedbRepository {
    #[inline]
    fn save_template(
        &self,
        template: &crate::template::aggregate::Template,
    ) -> Result<(), crate::template::error::TemplateRepositoryError> {
        self.write_repo().save_template(template)
    }

    #[inline]
    fn save_many_templates(
        &self,
        templates: &[crate::template::aggregate::Template],
    ) -> Result<(), crate::template::error::TemplateRepositoryError> {
        self.write_repo().save_many_templates(templates)
    }

    #[inline]
    fn delete_template(
        &self,
        id: crate::template::aggregate::TemplateId,
    ) -> Result<(), crate::template::error::TemplateRepositoryError> {
        self.write_repo().delete_template(id)
    }
}

impl RedbRepository {
    fn read_repo(&self) -> read::TemplateReadRepository<'_> {
        read::TemplateReadRepository::new(&self.store)
    }

    fn write_repo(&self) -> write::TemplateWriteRepository<'_> {
        write::TemplateWriteRepository::new(&self.store)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::template::aggregate::{Template, TemplateName};

    #[test]
    fn redb_repository_roundtrip() {
        let (_dir, store) = Store::open_temp_arc().unwrap();
        let repo = RedbRepository::new(store);

        let name = TemplateName::try_from("test-tpl").unwrap();
        let template =
            Template::try_new(&name, None, vec![], HashMap::new()).unwrap();
        let id = template.id();

        // 1. Save
        repo.save_template(&template).expect("save");

        // 2. Lookup by ID
        let found = repo.find_template_by_id(id).expect("lookup id").unwrap();
        assert_eq!(found.name(), &name);

        // 3. Lookup by Name
        let found_id =
            repo.find_template_id_by_name(&name).expect("lookup name").unwrap();
        assert_eq!(found_id, id);

        // 4. List
        let list = repo.list_templates().expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list.first().unwrap().id(), id);

        // 5. Delete
        repo.delete_template(id).expect("delete");
        assert!(repo.find_template_by_id(id).unwrap().is_none());
        assert!(repo.find_template_id_by_name(&name).unwrap().is_none());
    }

    #[test]
    fn redb_repository_updates_name_index() {
        let (_dir, store) = Store::open_temp_arc().unwrap();
        let repo = RedbRepository::new(store);

        let name1 = TemplateName::try_from("test-1").unwrap();
        let mut template =
            Template::try_new(&name1, None, vec![], HashMap::new()).unwrap();
        let id = template.id();

        repo.save_template(&template).unwrap();
        assert_eq!(repo.find_template_id_by_name(&name1).unwrap(), Some(id));

        // Update name
        let name2 = TemplateName::try_from("test-2").unwrap();
        template.name = name2.clone();
        repo.save_template(&template).unwrap();

        assert_eq!(repo.find_template_id_by_name(&name2).unwrap(), Some(id));
        assert!(repo.find_template_id_by_name(&name1).unwrap().is_none());
    }
}
