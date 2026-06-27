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
//! - **Multi-table coordination**: [`save_template`] atomically updates the
//!   template aggregate, name index ([`TEMPLATE_ID_BY_NAME`]), and path index
//!   ([`TEMPLATE_ID_BY_PATH`]).
//!
//! # Cross-Table Invariants
//!
//! - `save_template`: Maintains [`TEMPLATES`], [`TEMPLATE_ID_BY_NAME`], and
//!   [`TEMPLATE_ID_BY_PATH`] consistency.
//! - `delete_template`: Removes template aggregate, name index, and path index
//!   in a single transaction.
//!
//! # Rollback Behavior
//!
//! If serialization or table write fails, the transaction is automatically
//! rolled back by `redb`. No partial writes are visible to concurrent readers.
//!
//! [`WriteRepository`]: crate::storage::WriteRepository
//! [`RedbRepository`]: crate::storage::RedbRepository
//! [`TEMPLATES`]: crate::storage::tables::TEMPLATES
//! [`TEMPLATE_ID_BY_NAME`]: crate::storage::tables::TEMPLATE_ID_BY_NAME
//! [`TEMPLATE_ID_BY_PATH`]: crate::storage::tables::TEMPLATE_ID_BY_PATH
//! [`save_template`]: WriteRepository::save_template
//! [`delete_template`]: WriteRepository::delete_template

use redb::ReadableTable;
use trace_db::{ArchivedEntity, path::DbPathKey};
use trace_fs::PathKey;

use crate::{
    aggregate::{Template, TemplateId},
    storage::{
        RedbRepository, WriteRepository,
        tables::{
            RAW_TEMPLATE_VIEWS, TEMPLATE_ID_BY_NAME, TEMPLATE_ID_BY_PATH,
            TEMPLATES,
        },
    },
    views::RawTemplateView,
};

impl WriteRepository for RedbRepository {
    #[inline]
    fn save_template(
        &self,
        template: &Template,
    ) -> Result<(), crate::error::TemplateRepositoryError> {
        let bytes = template
            .to_bytes()
            .map_err(crate::error::TemplateRepositoryError::from)?;
        let id_bytes = template
            .id()
            .to_bytes()
            .map_err(crate::error::TemplateRepositoryError::from)?;

        self.store
            .write(|tx| {
                let mut table = tx.try_open_table(TEMPLATES.definition())?;
                table.insert(*template.id(), bytes.as_slice())?;

                let mut name_table =
                    tx.try_open_table(TEMPLATE_ID_BY_NAME.definition())?;
                name_table
                    .insert(template.name().as_str(), id_bytes.as_slice())?;

                let mut path_table =
                    tx.try_open_table(TEMPLATE_ID_BY_PATH.definition())?;
                path_table
                    .insert(DbPathKey::from(template.path()), template.id())?;

                Ok(())
            })
            .map_err(crate::error::TemplateRepositoryError::from)
    }

    #[inline]
    fn delete_template(
        &self,
        id: TemplateId,
    ) -> Result<(), crate::error::TemplateRepositoryError> {
        self.store
            .write(|tx| {
                // Open all three tables once. `remove` is a no-op on absent
                // keys, so unconditional opens are simpler than gating the
                // index tables behind the aggregate lookup.
                let mut templates_table =
                    tx.try_open_table(TEMPLATES.definition())?;
                let mut name_index =
                    tx.try_open_table(TEMPLATE_ID_BY_NAME.definition())?;
                let mut path_index =
                    tx.try_open_table(TEMPLATE_ID_BY_PATH.definition())?;

                let template_indexes: Option<(String, PathKey)> =
                    templates_table
                        .get(id)?
                        .map(|g| {
                            Template::from_bytes(g.value()).map(|s| {
                                (s.name().as_str().to_owned(), s.path().clone())
                            })
                        })
                        .transpose()?;

                if let Some((ref name, ref path)) = template_indexes {
                    let _ = name_index.remove(name.as_str())?;
                    let _ = path_index.remove(DbPathKey::from(path))?;
                }

                let _ = templates_table.remove(id)?;

                Ok(())
            })
            .map_err(crate::error::TemplateRepositoryError::from)
    }

    #[inline]
    fn save_raw_template_view(
        &self,
        id: TemplateId,
        view: &RawTemplateView,
    ) -> Result<(), crate::error::TemplateRepositoryError> {
        let bytes = view
            .to_bytes()
            .map_err(crate::error::TemplateRepositoryError::from)?;

        self.store
            .write(|tx| {
                let mut table =
                    tx.try_open_table(RAW_TEMPLATE_VIEWS.definition())?;
                table.insert(id, bytes.as_slice())?;
                Ok(())
            })
            .map_err(crate::error::TemplateRepositoryError::from)
    }

    #[inline]
    fn delete_raw_template_view(
        &self,
        path: &PathKey,
    ) -> Result<(), crate::error::TemplateRepositoryError> {
        self.store
            .write(|tx| {
                // Resolve path -> id; a view is reachable only through its
                // template's ID, so an absent path means an absent view.
                let path_index =
                    tx.try_open_table(TEMPLATE_ID_BY_PATH.definition())?;
                let Some(id) = path_index
                    .get(DbPathKey::from(path))?
                    .map(|guard| guard.value())
                else {
                    return Ok(());
                };

                let mut table =
                    tx.try_open_table(RAW_TEMPLATE_VIEWS.definition())?;
                let _ = table.remove(id)?;
                Ok(())
            })
            .map_err(crate::error::TemplateRepositoryError::from)
    }

    #[inline]
    fn delete_many_templates(
        &self,
        paths: &[PathKey],
    ) -> Result<(), crate::error::TemplateRepositoryError> {
        // Nothing to delete: skip opening any tables.
        if paths.is_empty() {
            return Ok(());
        }

        self.store
            .write(|tx| {
                let mut path_index =
                    tx.try_open_table(TEMPLATE_ID_BY_PATH.definition())?;
                let mut templates_table =
                    tx.try_open_table(TEMPLATES.definition())?;
                let mut name_index =
                    tx.try_open_table(TEMPLATE_ID_BY_NAME.definition())?;
                let mut views_table =
                    tx.try_open_table(RAW_TEMPLATE_VIEWS.definition())?;

                for path in paths {
                    // Resolve the template ID via the path index. A view is
                    // keyed by its template's ID, so no ID means no view and
                    // no aggregate to remove — skip silently.
                    let Some(id) = path_index
                        .remove(DbPathKey::from(path))?
                        .map(|guard| guard.value())
                    else {
                        continue;
                    };

                    // Remove the view by ID (path -> id -> view).
                    let _ = views_table.remove(id)?;

                    let name = templates_table
                        .get(id)?
                        .map(|g| {
                            Template::from_bytes(g.value())
                                .map(|t| t.name().as_str().to_owned())
                        })
                        .transpose()?;
                    if let Some(name) = name {
                        let _ = name_index.remove(name.as_str())?;
                    }
                    let _ = templates_table.remove(id)?;
                }

                Ok(())
            })
            .map_err(crate::error::TemplateRepositoryError::from)
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use trace_db::{ArchivedEntity, Store};
    use trace_fs::PathKey;

    use crate::{
        aggregate::{Template, TemplateId, TemplateName},
        storage::{
            ReadRepository, RedbRepository, WriteRepository, tables::TEMPLATES,
        },
        views::RawTemplateView,
    };

    fn test_template(name: &str) -> Template {
        use std::path::Path;

        use crate::aggregate::TemplateBody;

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

        use trace_fs::metadata::{FileMetadata, FsTimes};
        use trace_support::Blake3Hash;

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
        use pretty_assertions::assert_eq;

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
        fn updates_path_index() {
            let (_, repo) = setup_repo();
            let template = test_template("path-index-test");

            repo.save_template(&template).unwrap();

            let id = repo.find_template_id_by_path(template.path()).unwrap();
            let found = repo.find_template_by_path(template.path()).unwrap();

            assert_eq!(id, Some(*template.id()));
            assert_eq!(found.as_ref().map(Template::id), Some(template.id()));
        }

        #[test]
        fn rolls_back_on_serialization_error() {
            let (store, repo) = setup_repo();
            let template = test_template("rollback-test");
            let id = *template.id();

            repo.save_template(&template).unwrap();

            let result: Result<(), trace_db::DbError> = store.write(|tx| {
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
                    crate::aggregate::TemplateBody::try_new("x").unwrap();
                let t2 = Template::new(id2, path2, name2, body2);
                let bytes = t2.to_bytes()?;
                table.insert(*t2.id(), bytes.as_slice())?;
                Err(trace_db::DbError::Serialization(
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
        fn removes_path_index() {
            let (_, repo) = setup_repo();
            let template = test_template("delete-path");
            let id = *template.id();
            let path = template.path().clone();

            repo.save_template(&template).unwrap();
            repo.delete_template(id).unwrap();

            assert!(repo.find_template_id_by_path(&path).unwrap().is_none());
            assert!(repo.find_template_by_path(&path).unwrap().is_none());
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
        fn retrievable_by_path() {
            let (_, repo) = setup_repo();
            let template = test_template("hello");
            let path = template.path().clone();
            let view = test_view(path.as_str());

            repo.save_template(&template).unwrap();
            repo.save_raw_template_view(*template.id(), &view).unwrap();

            let found = repo.find_raw_template_view(&path).unwrap();
            assert!(found.is_some());
        }
    }

    mod delete_raw_template_view {
        use super::*;

        #[test]
        fn removes_view() {
            let (_, repo) = setup_repo();
            let template = test_template("remove");
            let path = template.path().clone();
            let view = test_view(path.as_str());

            repo.save_template(&template).unwrap();
            repo.save_raw_template_view(*template.id(), &view).unwrap();
            repo.delete_raw_template_view(&path).unwrap();

            assert!(repo.find_raw_template_view(&path).unwrap().is_none());
        }

        #[test]
        fn idempotent_on_missing() {
            let (_, repo) = setup_repo();
            repo.delete_raw_template_view(&key("missing.md")).unwrap();
            repo.delete_raw_template_view(&key("missing.md")).unwrap();
        }
    }

    mod delete_many_templates {
        use super::*;

        #[test]
        fn empty_slice_does_not_error() {
            let (_, repo) = setup_repo();
            repo.delete_many_templates(&[]).unwrap();
        }

        #[test]
        fn removes_template_and_view_for_each_path() {
            let (_, repo) = setup_repo();
            let template = test_template("delete-batch");
            let path = template.path().clone();
            repo.save_template(&template).unwrap();
            repo.save_raw_template_view(
                *template.id(),
                &test_view(path.as_str()),
            )
            .unwrap();

            repo.delete_many_templates(std::slice::from_ref(&path)).unwrap();

            assert!(repo.find_template_by_path(&path).unwrap().is_none());
            assert!(repo.find_raw_template_view(&path).unwrap().is_none());
        }
    }
}
