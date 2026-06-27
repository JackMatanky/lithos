//! [`ReadRepository`] trait implementation for [`RedbRepository`].
//!
//! Provides read-only template persistence operations backed by `redb`. All
//! methods execute within independent read transactions managed by the
//! [`Store`].
//!
//! # Transaction Boundaries
//!
//! Each method call opens a new read transaction via `Store::read()`. Methods
//! like `find_raw_template_views_by_paths` batch multiple lookups into a
//! single transaction for efficiency.
//!
//! # Table Access
//!
//! Uses table definitions from [`crate::storage::tables`]:
//! - [`TEMPLATES`]: Template aggregates by ID
//! - [`TEMPLATE_ID_BY_NAME`]: Name-to-ID index
//! - [`TEMPLATE_ID_BY_PATH`]: Path-to-ID index
//! - [`RAW_TEMPLATE_VIEWS`]: Raw template views by template ID
//!
//! [`ReadRepository`]: crate::storage::ReadRepository
//! [`RedbRepository`]: crate::storage::RedbRepository
//! [`Store`]: traces_db::Store
//! [`TEMPLATES`]: crate::storage::tables::TEMPLATES
//! [`TEMPLATE_ID_BY_NAME`]: crate::storage::tables::TEMPLATE_ID_BY_NAME
//! [`TEMPLATE_ID_BY_PATH`]: crate::storage::tables::TEMPLATE_ID_BY_PATH
//! [`RAW_TEMPLATE_VIEWS`]: crate::storage::tables::RAW_TEMPLATE_VIEWS

use redb::ReadableTable;
use traces_db::{ArchivedEntity, path::DbPathKey};
use traces_fs::PathKey;

use crate::{
    aggregate::{Template, TemplateId, TemplateName},
    storage::{
        ReadRepository, RedbRepository,
        tables::{
            RAW_TEMPLATE_VIEWS, TEMPLATE_ID_BY_NAME, TEMPLATE_ID_BY_PATH,
            TEMPLATES,
        },
    },
    views::RawTemplateView,
};

impl ReadRepository for RedbRepository {
    #[inline]
    fn find_template_by_id(
        &self,
        id: TemplateId,
    ) -> Result<Option<Template>, crate::error::TemplateRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) = tx.try_open_table(TEMPLATES.definition())?
                else {
                    return Ok(None);
                };

                let Some(guard) = table.get(id)? else {
                    return Ok(None);
                };

                let template = Template::from_bytes(guard.value())?;
                Ok(Some(template))
            })
            .map_err(crate::error::TemplateRepositoryError::from)
    }

    #[inline]
    fn find_template_by_name(
        &self,
        name: &TemplateName,
    ) -> Result<Option<Template>, crate::error::TemplateRepositoryError> {
        self.store
            .read(|tx| {
                let Some(name_table) =
                    tx.try_open_table(TEMPLATE_ID_BY_NAME.definition())?
                else {
                    return Ok(None);
                };

                let Some(id_guard) = name_table.get(name.as_str())? else {
                    return Ok(None);
                };

                let id = TemplateId::from_bytes(id_guard.value())?;

                let Some(template_table) =
                    tx.try_open_table(TEMPLATES.definition())?
                else {
                    return Ok(None);
                };

                let Some(template_guard) = template_table.get(id)? else {
                    return Ok(None);
                };

                let template = Template::from_bytes(template_guard.value())?;
                Ok(Some(template))
            })
            .map_err(crate::error::TemplateRepositoryError::from)
    }

    #[inline]
    fn find_template_id_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<TemplateId>, crate::error::TemplateRepositoryError> {
        self.store
            .read(|tx| {
                let Some(path_table) = tx.try_open_table(
                    crate::storage::tables::TEMPLATE_ID_BY_PATH.definition(),
                )?
                else {
                    return Ok(None);
                };

                let Some(id_guard) = path_table.get(DbPathKey::from(path))?
                else {
                    return Ok(None);
                };

                let id = id_guard.value();
                Ok(Some(id))
            })
            .map_err(crate::error::TemplateRepositoryError::from)
    }

    #[inline]
    fn find_template_ids_by_paths(
        &self,
        paths: &[PathKey],
    ) -> Result<Vec<Option<TemplateId>>, crate::error::TemplateRepositoryError>
    {
        self.store
            .read(|tx| {
                let Some(path_table) = tx.try_open_table(
                    crate::storage::tables::TEMPLATE_ID_BY_PATH.definition(),
                )?
                else {
                    return Ok(vec![None; paths.len()]);
                };

                let mut results = Vec::with_capacity(paths.len());
                for path in paths {
                    let id = path_table
                        .get(DbPathKey::from(path))?
                        .map(|guard| guard.value());
                    results.push(id);
                }
                Ok(results)
            })
            .map_err(crate::error::TemplateRepositoryError::from)
    }

    #[inline]
    fn find_template_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<Template>, crate::error::TemplateRepositoryError> {
        self.store
            .read(|tx| {
                let Some(path_table) = tx.try_open_table(
                    crate::storage::tables::TEMPLATE_ID_BY_PATH.definition(),
                )?
                else {
                    return Ok(None);
                };

                let Some(id_guard) = path_table.get(DbPathKey::from(path))?
                else {
                    return Ok(None);
                };

                let id = id_guard.value();

                let Some(template_table) =
                    tx.try_open_table(TEMPLATES.definition())?
                else {
                    return Ok(None);
                };

                let Some(template_guard) = template_table.get(id)? else {
                    return Ok(None);
                };

                let template = Template::from_bytes(template_guard.value())?;
                Ok(Some(template))
            })
            .map_err(crate::error::TemplateRepositoryError::from)
    }

    #[inline]
    fn list_templates(
        &self,
    ) -> Result<Vec<Template>, crate::error::TemplateRepositoryError> {
        self.store
            .read(|tx| {
                let Some(table) = tx.try_open_table(TEMPLATES.definition())?
                else {
                    return Ok(Vec::new());
                };

                let mut templates = Vec::new();
                for result in table.iter()? {
                    let (_id_guard, template_guard) = result?;
                    templates
                        .push(Template::from_bytes(template_guard.value())?);
                }

                Ok(templates)
            })
            .map_err(crate::error::TemplateRepositoryError::from)
    }

    #[inline]
    fn find_raw_template_view(
        &self,
        path: &PathKey,
    ) -> Result<Option<RawTemplateView>, crate::error::TemplateRepositoryError>
    {
        self.store
            .read(|tx| {
                // path -> TemplateId
                let Some(path_table) =
                    tx.try_open_table(TEMPLATE_ID_BY_PATH.definition())?
                else {
                    return Ok(None);
                };
                let Some(id_guard) = path_table.get(DbPathKey::from(path))?
                else {
                    return Ok(None);
                };
                let id = id_guard.value();

                // TemplateId -> RawTemplateView
                let Some(view_table) =
                    tx.try_open_table(RAW_TEMPLATE_VIEWS.definition())?
                else {
                    return Ok(None);
                };
                let Some(guard) = view_table.get(id)? else {
                    return Ok(None);
                };

                let view = RawTemplateView::from_bytes(guard.value())?;
                Ok(Some(view))
            })
            .map_err(crate::error::TemplateRepositoryError::from)
    }

    #[inline]
    fn list_template_path_keys(
        &self,
    ) -> Result<Vec<PathKey>, crate::error::TemplateRepositoryError> {
        self.store
            .read(|tx| {
                // Every cached path is recorded in the path index, which is
                // maintained in lock-step with the aggregate and its view.
                let Some(table) =
                    tx.try_open_table(TEMPLATE_ID_BY_PATH.definition())?
                else {
                    return Ok(Vec::new());
                };

                let mut paths = Vec::new();
                for result in table.iter()? {
                    let (path_guard, _) = result?;
                    paths.push(path_guard.value().into_inner());
                }

                Ok(paths)
            })
            .map_err(crate::error::TemplateRepositoryError::from)
    }

    #[inline]
    fn find_raw_template_views_by_paths(
        &self,
        paths: &[PathKey],
    ) -> Result<
        Vec<Option<RawTemplateView>>,
        crate::error::TemplateRepositoryError,
    > {
        self.store
            .read(|tx| {
                let (Some(path_table), Some(view_table)) = (
                    tx.try_open_table(TEMPLATE_ID_BY_PATH.definition())?,
                    tx.try_open_table(RAW_TEMPLATE_VIEWS.definition())?,
                ) else {
                    return Ok(paths.iter().map(|_| None).collect());
                };

                let mut results = Vec::with_capacity(paths.len());
                for path in paths {
                    // path -> id -> view; an absent id yields None.
                    let view = match path_table.get(DbPathKey::from(path))? {
                        Some(id_guard) => view_table
                            .get(id_guard.value())?
                            .map(|g| RawTemplateView::from_bytes(g.value()))
                            .transpose()?,
                        None => None,
                    };
                    results.push(view);
                }

                Ok(results)
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

    use traces_db::{ArchivedEntity, Store};
    use traces_fs::PathKey;

    use crate::{
        aggregate::{Template, TemplateId, TemplateName},
        storage::{
            ReadRepository, RedbRepository,
            tables::{TEMPLATE_ID_BY_NAME, TEMPLATES},
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

        use traces_fs::metadata::{FileMetadata, FsTimes};
        use traces_support::Blake3Hash;

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

    mod find_by_id {
        use super::*;

        #[test]
        fn returns_none_when_empty() {
            let (_, repo) = setup_repo();
            let result = repo.find_template_by_id(TemplateId::new()).unwrap();
            assert!(result.is_none());
        }

        #[test]
        fn returns_template_after_direct_insert() {
            let (store, repo) = setup_repo();
            let template = test_template("direct");
            let id = *template.id();
            let bytes = template.to_bytes().expect("serialize");
            let id_bytes = id.to_bytes().expect("serialize id");

            store
                .write(|tx| {
                    let mut table =
                        tx.try_open_table(TEMPLATES.definition())?;
                    table.insert(id, bytes.as_slice())?;
                    let mut name_table =
                        tx.try_open_table(TEMPLATE_ID_BY_NAME.definition())?;
                    name_table.insert(
                        template.name().as_str(),
                        id_bytes.as_slice(),
                    )?;
                    Ok(())
                })
                .expect("direct insert");

            let found = repo.find_template_by_id(id).unwrap();
            assert!(found.is_some());
            let found = found.unwrap();
            assert_eq!(found.id(), template.id());
            assert_eq!(found.name(), template.name());
        }
    }

    mod find_by_name {
        use super::*;

        #[test]
        fn returns_none_when_name_missing() {
            use std::path::Path;

            let (_, repo) = setup_repo();
            let name = TemplateName::try_new(
                Path::new("templates/missing.md"),
                Path::new("templates"),
            )
            .expect("name");
            let result = repo.find_template_by_name(&name).unwrap();
            assert!(result.is_none());
        }

        #[test]
        fn returns_template_after_direct_insert() {
            let (store, repo) = setup_repo();
            let template = test_template("byname");
            let id = *template.id();
            let bytes = template.to_bytes().expect("serialize");
            let id_bytes = id.to_bytes().expect("serialize id");

            store
                .write(|tx| {
                    let mut table =
                        tx.try_open_table(TEMPLATES.definition())?;
                    table.insert(id, bytes.as_slice())?;
                    let mut name_table =
                        tx.try_open_table(TEMPLATE_ID_BY_NAME.definition())?;
                    name_table.insert(
                        template.name().as_str(),
                        id_bytes.as_slice(),
                    )?;
                    Ok(())
                })
                .expect("direct insert");

            let found = repo.find_template_by_name(template.name()).unwrap();
            assert!(found.is_some());
            let found = found.unwrap();
            assert_eq!(found.id(), template.id());
        }
    }

    mod list {
        use super::*;

        #[test]
        fn returns_empty_when_no_templates() {
            let (_, repo) = setup_repo();
            let result = repo.list_templates().unwrap();
            assert!(result.is_empty());
        }

        #[test]
        fn returns_all_saved_templates() {
            let (store, repo) = setup_repo();
            let t1 = test_template("one");
            let t2 = test_template("two");

            store
                .write(|tx| {
                    let mut table =
                        tx.try_open_table(TEMPLATES.definition())?;
                    let mut name_table =
                        tx.try_open_table(TEMPLATE_ID_BY_NAME.definition())?;

                    for template in [&t1, &t2] {
                        let bytes = template.to_bytes().expect("serialize");
                        let id_bytes =
                            template.id().to_bytes().expect("serialize id");
                        table.insert(*template.id(), bytes.as_slice())?;
                        name_table.insert(
                            template.name().as_str(),
                            id_bytes.as_slice(),
                        )?;
                    }
                    Ok(())
                })
                .expect("direct insert");

            let result = repo.list_templates().unwrap();
            assert_eq!(result.len(), 2);
        }
    }

    mod find_view {
        use super::*;

        #[test]
        fn returns_none_when_view_missing() {
            let (_, repo) = setup_repo();
            let result =
                repo.find_raw_template_view(&key("missing.json")).unwrap();
            assert!(result.is_none());
        }

        #[test]
        fn returns_view_by_path_after_saving_template_and_view() {
            use crate::storage::WriteRepository;

            let (_store, repo) = setup_repo();
            let template = test_template("hello");
            let id = *template.id();
            let view = test_view(template.path().as_str());
            let path = template.path().clone();

            // The view is keyed by TemplateId; reading it back by path must
            // resolve path -> id -> view through TEMPLATE_ID_BY_PATH.
            repo.save_template(&template).unwrap();
            repo.save_raw_template_view(id, &view).unwrap();

            let found = repo.find_raw_template_view(&path).unwrap();
            assert!(
                found.is_some(),
                "expected the view to be retrievable by path via the id index"
            );
        }
    }

    mod batch_find_views {
        use super::*;

        #[test]
        fn returns_correct_order_with_missing_in_between() {
            use crate::storage::WriteRepository;

            let (_store, repo) = setup_repo();
            let template_a = test_template("a");
            let template_c = test_template("c");
            let view_a = test_view(template_a.path().as_str());
            let view_c = test_view(template_c.path().as_str());

            repo.save_template(&template_a).unwrap();
            repo.save_raw_template_view(*template_a.id(), &view_a).unwrap();
            repo.save_template(&template_c).unwrap();
            repo.save_raw_template_view(*template_c.id(), &view_c).unwrap();

            let paths = vec![
                template_a.path().clone(),
                key("templates/b.md"),
                template_c.path().clone(),
            ];
            let results =
                repo.find_raw_template_views_by_paths(&paths).unwrap();
            assert_eq!(results.len(), 3);
            assert!(results.first().is_some_and(Option::is_some));
            assert!(results.get(1).is_some_and(Option::is_none));
            assert!(results.get(2).is_some_and(Option::is_some));
        }

        #[test]
        fn empty_slice_returns_empty() {
            let (_, repo) = setup_repo();
            let results = repo.find_raw_template_views_by_paths(&[]).unwrap();
            assert!(results.is_empty());
        }
    }
}
