//! Template storage implementations.

pub mod read;
pub(crate) mod tables;
pub mod write;

#[cfg(any(test, feature = "testing"))]
pub mod testing;

use std::sync::Arc;

use crate::db::Store;

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::template::{
        aggregate::{Template, TemplateId, TemplateName},
        repository::{ReadRepository, WriteRepository},
    };

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
    fn redb_repository_batch_operations() {
        let (_dir, store) = Store::open_temp_arc().unwrap();
        let repo = RedbRepository::new(store);

        let name1 = TemplateName::try_from("t1").unwrap();
        let t1 =
            Template::try_new(&name1, None, vec![], HashMap::new()).unwrap();
        let name2 = TemplateName::try_from("t2").unwrap();
        let t2 =
            Template::try_new(&name2, None, vec![], HashMap::new()).unwrap();

        // Batch save
        repo.save_many_templates(&[t1.clone(), t2.clone()]).unwrap();

        // Batch lookup
        let ids = vec![t1.id(), TemplateId::new(), t2.id()];
        let results = repo.find_many_templates_by_id(&ids).unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(
            results.first().and_then(Option::as_ref).map(Template::id),
            Some(t1.id())
        );
        assert!(results.get(1).is_some_and(Option::is_none));
        assert_eq!(
            results.get(2).and_then(Option::as_ref).map(Template::id),
            Some(t2.id())
        );
    }

    #[test]
    fn redb_repository_deleting_missing_is_idempotent() {
        let (_dir, store) = Store::open_temp_arc().unwrap();
        let repo = RedbRepository::new(store);

        let id = TemplateId::new();
        repo.delete_template(id).expect("delete missing should succeed");
        repo.delete_template(id).expect("second delete should succeed");
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
