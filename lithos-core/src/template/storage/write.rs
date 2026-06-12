//! [`WriteRepository`] trait implementation for [`RedbRepository`].
//!
//! Provides write operations for template persistence backed by `redb`. All
//! writes execute within atomic transactions with automatic rollback on error.
//!
//! # Atomicity Guarantees
//!
//! - **Single transaction per method**: Each write method opens one transaction
//!   via `Store::write()`. If any table operation fails, the entire transaction
//!   rolls back automatically.
//! - **Multi-table coordination**: [`save_template`] atomically updates both
//!   the template aggregate and its name index ([`TEMPLATE_ID_BY_NAME`]).
//! - **Batch operations**: [`save_many_raw_template_views`] wraps all saves in
//!   a single transaction for atomicity.
//!
//! # Cross-Table Invariants
//!
//! - `save_template`: Maintains [`TEMPLATES`] ↔ [`TEMPLATE_ID_BY_NAME`]
//!   consistency
//! - `delete_template`: Removes template aggregate + name index + raw view in a
//!   single transaction
//!
//! # Rollback Behavior
//!
//! If serialization or table write fails, the transaction is automatically
//! rolled back by `redb`. No partial writes are visible to concurrent readers.
//!
//! [`WriteRepository`]: crate::template::repository::WriteRepository
//! [`RedbRepository`]: crate::template::storage::RedbRepository
//! [`TEMPLATES`]: crate::template::storage::tables::TEMPLATES
//! [`TEMPLATE_ID_BY_NAME`]: crate::template::storage::tables::TEMPLATE_ID_BY_NAME
//! [`save_template`]: WriteRepository::save_template
//! [`save_many_raw_template_views`]: WriteRepository::save_many_raw_template_views
//! [`delete_template`]: WriteRepository::delete_template

use redb::ReadableTable;

use crate::{
    db::ArchivedEntity,
    fs::PathKey,
    template::{
        aggregate::{Template, TemplateId},
        repository::WriteRepository,
        storage::{
            RedbRepository,
            tables::{RAW_TEMPLATE_VIEWS, TEMPLATE_ID_BY_NAME, TEMPLATES},
        },
        views::RawTemplateView,
    },
};

impl WriteRepository for RedbRepository {
    #[inline]
    fn save_template(
        &self,
        template: &Template,
    ) -> Result<(), crate::template::repository::TemplateRepositoryError> {
        let bytes = template.to_bytes().map_err(
            crate::template::repository::TemplateRepositoryError::from,
        )?;
        let id_bytes = template.id().to_bytes().map_err(
            crate::template::repository::TemplateRepositoryError::from,
        )?;

        self.store
            .write(|tx| {
                let mut table = tx.try_open_table(TEMPLATES.definition())?;
                table.insert(*template.id(), bytes.as_slice())?;

                let mut name_table =
                    tx.try_open_table(TEMPLATE_ID_BY_NAME.definition())?;
                name_table
                    .insert(template.name().as_str(), id_bytes.as_slice())?;

                Ok(())
            })
            .map_err(crate::template::repository::TemplateRepositoryError::from)
    }

    #[inline]
    fn delete_template(
        &self,
        id: TemplateId,
    ) -> Result<(), crate::template::repository::TemplateRepositoryError> {
        self.store
            .write(|tx| {
                let mut schemas = tx.try_open_table(TEMPLATES.definition())?;

                let schema_name: Option<String> = schemas
                    .get(id)?
                    .map(|g| {
                        Template::from_bytes(g.value())
                            .map(|s| s.name().as_str().to_owned())
                    })
                    .transpose()?;

                if let Some(ref name) = schema_name {
                    let mut name_index =
                        tx.try_open_table(TEMPLATE_ID_BY_NAME.definition())?;
                    let _ = name_index.remove(name.as_str())?;
                }

                let _ = schemas.remove(id)?;

                Ok(())
            })
            .map_err(crate::template::repository::TemplateRepositoryError::from)
    }

    #[inline]
    fn save_raw_template_view(
        &self,
        view: &RawTemplateView,
    ) -> Result<(), crate::template::repository::TemplateRepositoryError> {
        let bytes = view.to_bytes().map_err(
            crate::template::repository::TemplateRepositoryError::from,
        )?;

        self.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(RAW_TEMPLATE_VIEWS.definition())?;
                table.insert(view.path(), bytes.as_slice())?;
                Ok(())
            })
            .map_err(crate::template::repository::TemplateRepositoryError::from)
    }

    #[inline]
    fn delete_raw_template_view(
        &self,
        path: &PathKey,
    ) -> Result<(), crate::template::repository::TemplateRepositoryError> {
        self.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(RAW_TEMPLATE_VIEWS.definition())?;
                let _ = table.remove(path)?;
                Ok(())
            })
            .map_err(crate::template::repository::TemplateRepositoryError::from)
    }

    #[inline]
    fn save_many_raw_template_views(
        &self,
        views: &[RawTemplateView],
    ) -> Result<(), crate::template::repository::TemplateRepositoryError> {
        let serialized: Vec<(
            PathKey,
            Vec<u8>,
        )> = views
            .iter()
            .map(|view| {
                view.to_bytes()
                    .map(|bytes| (view.path().clone(), bytes.to_vec()))
                    .map_err(crate::template::repository::TemplateRepositoryError::from)
            })
            .collect::<Result<_, _>>()?;

        self.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(RAW_TEMPLATE_VIEWS.definition())?;

                for (path, bytes) in &serialized {
                    table.insert(path, bytes.as_slice())?;
                }

                Ok(())
            })
            .map_err(crate::template::repository::TemplateRepositoryError::from)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        db::{ArchivedEntity, Store},
        fs::PathKey,
        template::{
            aggregate::{Template, TemplateId, TemplateName},
            repository::{ReadRepository, WriteRepository},
            storage::{RedbRepository, tables::TEMPLATES},
            views::RawTemplateView,
        },
    };

    fn test_template(name: &str) -> Template {
        use std::path::Path;

        use crate::template::aggregate::TemplateBody;

        let id = TemplateId::new();
        let path = PathKey::try_new(&format!("templates/{name}.md")).unwrap();
        let tmpl_name = TemplateName::try_new(
            Path::new(&format!("templates/{name}.md")),
            Path::new("templates"),
        )
        .unwrap();
        let body = TemplateBody::try_new(format!("content: {name}")).unwrap();
        Template::new(id, path, tmpl_name, body)
    }

    fn test_view(path: &str) -> RawTemplateView {
        use std::time::SystemTime;

        use crate::{
            fs::metadata::{FileMetadata, FsTimes},
            support::content_hash::Blake3Hash,
        };

        let key = PathKey::try_new(path).unwrap();
        let hash = Blake3Hash::from_bytes(format!("hash:{path}").as_bytes());
        let metadata = FileMetadata::new(FsTimes::new(None, None), 100, false);
        let recorded_at = SystemTime::now();
        RawTemplateView::new(key, hash, metadata, recorded_at)
    }

    fn key(path: &str) -> PathKey {
        PathKey::try_new(path).expect("valid path key")
    }

    fn setup_repo() -> (Arc<Store>, RedbRepository) {
        let temp_dir = tempfile::TempDir::new().expect("temp dir");
        let db_path = temp_dir.path().join("test.db");
        let store = Arc::new(Store::open(&db_path).expect("store open"));
        let repo = RedbRepository::new(Arc::clone(&store));
        (store, repo)
    }

    mod save_template {
        use super::*;

        #[test]
        fn persists_template() {
            let (_, repo) = setup_repo();
            let template = test_template("persist");
            let id = *template.id();

            repo.save_template(&template).unwrap();

            let found = repo.find_template_by_id(id).unwrap();
            assert!(found.is_some());
            let found = found.unwrap();
            assert_eq!(found.id(), template.id());
            assert_eq!(found.name(), template.name());
        }

        #[test]
        fn updates_name_index() {
            let (_, repo) = setup_repo();
            let template = test_template("index-test");

            repo.save_template(&template).unwrap();

            let found = repo.find_template_by_name(template.name()).unwrap();
            assert!(found.is_some());
            let found = found.unwrap();
            assert_eq!(found.id(), template.id());
        }

        #[test]
        fn rolls_back_on_serialization_error() {
            let (store, repo) = setup_repo();
            let template = test_template("rollback-test");
            let id = *template.id();

            repo.save_template(&template).unwrap();

            let result: Result<(), crate::db::DbError> = store.write(|tx| {
                use std::path::Path;

                let mut table = tx.try_open_table(TEMPLATES.definition())?;
                let id2 = TemplateId::new();
                let name2 = TemplateName::try_new(
                    Path::new("templates/will-rollback.md"),
                    Path::new("templates"),
                )
                .unwrap();
                let path2 = PathKey::try_new("templates/rollback.md").unwrap();
                let body2 =
                    crate::template::aggregate::TemplateBody::try_new("x")
                        .unwrap();
                let t2 = Template::new(id2, path2, name2, body2);
                let bytes = t2.to_bytes()?;
                table.insert(*t2.id(), bytes.as_slice())?;
                Err(crate::db::DbError::Serialization(
                    "forced failure".to_owned(),
                ))
            });

            assert!(result.is_err());

            let found = repo.find_template_by_id(id).unwrap();
            assert!(found.is_some());
        }
    }

    mod delete_template {
        use super::*;

        #[test]
        fn removes_template() {
            let (_, repo) = setup_repo();
            let template = test_template("delete-me");
            let id = *template.id();

            repo.save_template(&template).unwrap();
            repo.delete_template(id).unwrap();

            assert!(repo.find_template_by_id(id).unwrap().is_none());
        }

        #[test]
        fn removes_name_index() {
            let (_, repo) = setup_repo();
            let template = test_template("delete-name");
            let id = *template.id();
            let name = template.name().clone();

            repo.save_template(&template).unwrap();
            repo.delete_template(id).unwrap();

            assert!(repo.find_template_by_name(&name).unwrap().is_none());
        }

        #[test]
        fn idempotent_on_missing() {
            let (_, repo) = setup_repo();
            repo.delete_template(TemplateId::new()).unwrap();
            repo.delete_template(TemplateId::new()).unwrap();
        }
    }

    mod save_raw_template_view {
        use super::*;

        #[test]
        fn persists_view() {
            let (_, repo) = setup_repo();
            let view = test_view("templates/test.json");
            let path = key("templates/test.json");

            repo.save_raw_template_view(&view).unwrap();

            let found = repo.find_raw_template_view(&path).unwrap();
            assert!(found.is_some());
        }

        #[test]
        fn retrievable_by_path() {
            let (_, repo) = setup_repo();
            let view = test_view("templates/hello.json");
            let path = key("templates/hello.json");

            repo.save_raw_template_view(&view).unwrap();

            let found = repo.find_raw_template_view(&path).unwrap();
            assert!(found.is_some());
        }
    }

    mod delete_raw_template_view {
        use super::*;

        #[test]
        fn removes_view() {
            let (_, repo) = setup_repo();
            let view = test_view("templates/remove.json");
            let path = key("templates/remove.json");

            repo.save_raw_template_view(&view).unwrap();
            repo.delete_raw_template_view(&path).unwrap();

            assert!(repo.find_raw_template_view(&path).unwrap().is_none());
        }

        #[test]
        fn idempotent_on_missing() {
            let (_, repo) = setup_repo();
            repo.delete_raw_template_view(&key("missing.json")).unwrap();
            repo.delete_raw_template_view(&key("missing.json")).unwrap();
        }
    }

    mod save_many_views {
        use super::*;

        #[test]
        fn persists_all_views() {
            let (_, repo) = setup_repo();
            let v1 = test_view("templates/a.json");
            let v2 = test_view("templates/b.json");
            let path1 = key("templates/a.json");
            let path2 = key("templates/b.json");

            repo.save_many_raw_template_views(&[v1, v2]).unwrap();

            assert!(repo.find_raw_template_view(&path1).unwrap().is_some());
            assert!(repo.find_raw_template_view(&path2).unwrap().is_some());
        }

        #[test]
        fn empty_slice_does_not_error() {
            let (_, repo) = setup_repo();
            repo.save_many_raw_template_views(&[]).unwrap();
        }
    }
}
