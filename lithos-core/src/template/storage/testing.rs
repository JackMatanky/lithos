//! Testing utilities for template storage components.
//!
//! This module provides test doubles for the template repository traits,
//! enabling pure unit tests without filesystem dependencies. Code in this
//! module is compiled for both `#[cfg(test)]` and benchmarks.
//!
//! # Exports
//!
//! - [`InMemoryRepository`] - HashMap-backed [`Repository`] implementation
//!
//! # Design Rationale
//!
//! This module exists to enable **pure unit tests** following matklad's
//! test purity hierarchy:
//!
//! - **Pure computation** (fastest, most reliable)
//! - Threads → Filesystem → Network → Processes (slowest, least reliable)
//!
//! By providing an in-memory Repository implementation, we eliminate filesystem
//! IO from unit tests while maintaining test extent (can still test full
//! pipelines end-to-end).
//!
//! When the redb adapter is built, both `InMemoryRepository` and
//! `RedbRepository` should be tested against a shared contract suite in
//! `lithos-core/tests/template_storage.rs`.
//!
//! [`Repository`]: crate::template::repository::Repository

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    db::testing::{FailurePoint, InMemoryHarness, read_lock, write_lock},
    fs::PathKey,
    template::{
        aggregate::{Template, TemplateId, TemplateName},
        error::TemplateRepositoryError,
        repository::{ReadRepository, WriteRepository},
        views::RawTemplateView,
    },
};

// ============================================================================
// InMemoryRepository - For Pure Unit Tests
// ============================================================================

/// HashMap-backed [`Repository`] implementation for pure unit tests.
///
/// This is NOT a mock - it's a fully functional [`Repository`] implementation
/// that uses `HashMap` for storage instead of a persistent database. All
/// [`Repository`] trait methods are implemented with identical semantics to
/// the future `RedbRepository`, except data is stored in memory only.
///
/// # Thread Safety
///
/// All internal state is protected by `RwLock` for thread-safe concurrent
/// access. Lock poisoning is reported via
/// `TemplateRepositoryError::Storage(DbError::Corruption(...))`.
///
/// # When to Use
///
/// **Use for:**
/// - Unit tests in `#[cfg(test)]` modules (pure computation testing)
///
/// **Do NOT use for:**
/// - Integration tests (use `RedbRepository` to verify
///   serialization/durability)
/// - Production code (no persistence guarantees)
///
/// [`Repository`]: crate::template::repository::Repository
#[derive(Debug, Clone)]
pub(crate) struct InMemoryRepository {
    /// Test harness for operation instrumentation and failure injection.
    harness: Arc<InMemoryHarness>,

    /// Template storage: `TemplateId` → `Template`
    templates: Arc<RwLock<HashMap<TemplateId, Template>>>,

    /// Name-to-ID lookup: `TemplateName` → `TemplateId`
    name_to_id: Arc<RwLock<HashMap<TemplateName, TemplateId>>>,

    /// Path-to-ID lookup: `PathKey` → `TemplateId`
    path_to_id: Arc<RwLock<HashMap<PathKey, TemplateId>>>,

    /// Raw template views for staleness detection: path → `RawTemplateView`
    raw_views: Arc<RwLock<HashMap<PathKey, RawTemplateView>>>,
}

#[cfg_attr(
    not(test),
    allow(dead_code, reason = "test double, called from test code only")
)]
impl InMemoryRepository {
    /// Creates a new empty in-memory repository.
    #[must_use]
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            harness: Arc::new(InMemoryHarness::new()),
            templates: Arc::new(RwLock::new(HashMap::new())),
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
            path_to_id: Arc::new(RwLock::new(HashMap::new())),
            raw_views: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Replaces the test harness with a custom one (test helper).
    #[must_use]
    pub(crate) fn with_harness(
        mut self,
        harness: Arc<InMemoryHarness>,
    ) -> Self {
        self.harness = harness;
        self
    }

    /// Configures a failure injector for the repository (test helper).
    #[must_use]
    pub(crate) fn with_failure_injector(
        self,
        injector: Box<dyn crate::db::testing::FailureInjector + Send + Sync>,
    ) -> Self {
        self.with_harness(Arc::new(InMemoryHarness::with_injector(injector)))
    }

    /// Returns a reference to the test harness for instrumentation.
    #[must_use]
    pub(crate) fn harness(&self) -> &InMemoryHarness {
        &self.harness
    }

    /// Returns the number of templates currently stored (test helper).
    #[must_use]
    #[inline]
    #[expect(clippy::expect_used, reason = "test helper panics on lock poison")]
    pub(crate) fn template_count(&self) -> usize {
        self.templates.read().expect("Lock poisoned").len()
    }

    /// Clears all stored data (test helper).
    #[inline]
    #[expect(clippy::expect_used, reason = "test helper panics on lock poison")]
    pub(crate) fn clear(&self) {
        self.templates.write().expect("Lock poisoned").clear();
        self.name_to_id.write().expect("Lock poisoned").clear();
        self.path_to_id.write().expect("Lock poisoned").clear();
        self.raw_views.write().expect("Lock poisoned").clear();
    }
}

impl Default for InMemoryRepository {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl ReadRepository for InMemoryRepository {
    #[inline]
    fn find_template_by_id(
        &self,
        id: TemplateId,
    ) -> Result<Option<Template>, TemplateRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let templates = read_lock(&self.templates, "find_template_by_id")?;
        self.harness.counters().inc_read();

        Ok(templates.get(&id).cloned())
    }

    #[inline]
    fn find_template_by_name(
        &self,
        name: &TemplateName,
    ) -> Result<Option<Template>, TemplateRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let name_to_id =
            read_lock(&self.name_to_id, "find_template_by_name (name_to_id)")?;
        self.harness.counters().inc_read();

        let templates =
            read_lock(&self.templates, "find_template_by_name (templates)")?;
        self.harness.counters().inc_read();

        Ok(name_to_id.get(name).and_then(|id| templates.get(id).cloned()))
    }

    #[inline]
    fn find_template_id_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<TemplateId>, TemplateRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let path_to_id =
            read_lock(&self.path_to_id, "find_template_id_by_path")?;
        self.harness.counters().inc_read();

        Ok(path_to_id.get(path).copied())
    }

    #[inline]
    fn find_template_by_path(
        &self,
        path: &PathKey,
    ) -> Result<Option<Template>, TemplateRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let path_to_id =
            read_lock(&self.path_to_id, "find_template_by_path (path_to_id)")?;
        self.harness.counters().inc_read();

        let templates =
            read_lock(&self.templates, "find_template_by_path (templates)")?;
        self.harness.counters().inc_read();

        Ok(path_to_id.get(path).and_then(|id| templates.get(id).cloned()))
    }

    #[inline]
    fn list_templates(&self) -> Result<Vec<Template>, TemplateRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let templates = read_lock(&self.templates, "list_templates")?;
        self.harness.counters().inc_read();

        Ok(templates.values().cloned().collect())
    }

    #[inline]
    fn find_raw_template_view(
        &self,
        path: &PathKey,
    ) -> Result<Option<RawTemplateView>, TemplateRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let views = read_lock(&self.raw_views, "find_raw_template_view")?;
        self.harness.counters().inc_read();

        Ok(views.get(path).cloned())
    }

    #[inline]
    fn find_raw_template_views_by_paths(
        &self,
        paths: &[PathKey],
    ) -> Result<Vec<Option<RawTemplateView>>, TemplateRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let views =
            read_lock(&self.raw_views, "find_raw_template_views_by_paths")?;
        self.harness.counters().inc_read();

        let mut results = Vec::with_capacity(paths.len());
        for path in paths {
            results.push(views.get(path).cloned());
        }
        Ok(results)
    }
}

impl WriteRepository for InMemoryRepository {
    #[inline]
    fn save_template(
        &self,
        template: &Template,
    ) -> Result<(), TemplateRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut templates =
            write_lock(&self.templates, "save_template (templates)")?;
        self.harness.counters().inc_write();

        let mut name_to_id =
            write_lock(&self.name_to_id, "save_template (name_to_id)")?;
        self.harness.counters().inc_write();

        let mut path_to_id =
            write_lock(&self.path_to_id, "save_template (path_to_id)")?;
        self.harness.counters().inc_write();

        templates.insert(*template.id(), template.clone());
        name_to_id.insert(template.name().clone(), *template.id());
        path_to_id.insert(template.path().clone(), *template.id());
        Ok(())
    }

    #[inline]
    fn delete_template(
        &self,
        id: TemplateId,
    ) -> Result<(), TemplateRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut templates =
            write_lock(&self.templates, "delete_template (templates)")?;
        self.harness.counters().inc_write();

        let mut name_to_id =
            write_lock(&self.name_to_id, "delete_template (name_to_id)")?;
        self.harness.counters().inc_write();

        let mut path_to_id =
            write_lock(&self.path_to_id, "delete_template (path_to_id)")?;
        self.harness.counters().inc_write();

        if let Some(template) = templates.remove(&id) {
            name_to_id.remove(template.name());
            path_to_id.remove(template.path());
        }
        Ok(())
    }

    #[inline]
    fn save_raw_template_view(
        &self,
        view: &RawTemplateView,
    ) -> Result<(), TemplateRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut views = write_lock(&self.raw_views, "save_raw_template_view")?;
        self.harness.counters().inc_write();

        views.insert(view.path().clone(), view.clone());
        Ok(())
    }

    #[inline]
    fn delete_raw_template_view(
        &self,
        path: &PathKey,
    ) -> Result<(), TemplateRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut views =
            write_lock(&self.raw_views, "delete_raw_template_view")?;
        self.harness.counters().inc_write();

        views.remove(path);
        Ok(())
    }

    #[inline]
    fn save_many_raw_template_views(
        &self,
        views: &[RawTemplateView],
    ) -> Result<(), TemplateRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut view_map =
            write_lock(&self.raw_views, "save_many_raw_template_views")?;
        self.harness.counters().inc_write();

        for view in views {
            view_map.insert(view.path().clone(), view.clone());
        }
        Ok(())
    }
}

// ============================================================================
// Error Conversion
// ============================================================================

/// Convert `db::testing::InMemoryDbError` directly to
/// `TemplateRepositoryError`.
///
/// This avoids an intermediate custom error conversion at call sites that
/// only need to satisfy repository trait error contracts.
#[cfg(any(test, feature = "testing"))]
impl From<crate::db::testing::InMemoryDbError> for TemplateRepositoryError {
    #[inline]
    fn from(err: crate::db::testing::InMemoryDbError) -> Self {
        use crate::db::testing::InMemoryDbError as DbTestError;

        let db_error = match err {
            DbTestError::LockPoisoned {
                context,
            } => crate::db::DbError::Corruption(format!(
                "Lock poisoned: {context}"
            )),
            DbTestError::InjectedFailure {
                reason,
                ..
            } => crate::db::DbError::Corruption(format!(
                "Injected failure: {reason}"
            )),
            DbTestError::InvariantViolation {
                message,
            } => crate::db::DbError::Corruption(message.into()),
        };

        TemplateRepositoryError::Storage(db_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{
        DbError,
        testing::{FailureInjector, FailurePoint as FPFake, InMemoryDbError},
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

    mod fixtures {
        use super::*;

        pub(super) struct FailOnWrite;

        impl FailureInjector for FailOnWrite {
            fn fail_at(&self, point: FPFake) -> Result<(), InMemoryDbError> {
                if point == FPFake::BeforeWrite {
                    return Err(InMemoryDbError::InjectedFailure {
                        point,
                        reason: "write injection".into(),
                    });
                }
                Ok(())
            }
        }

        pub(super) struct FailOnRead;

        impl FailureInjector for FailOnRead {
            fn fail_at(&self, point: FPFake) -> Result<(), InMemoryDbError> {
                if point == FPFake::BeforeRead {
                    return Err(InMemoryDbError::InjectedFailure {
                        point,
                        reason: "read injection".into(),
                    });
                }
                Ok(())
            }
        }
    }

    mod defaults {
        use super::*;

        #[test]
        fn returns_zero_template_count_when_new() {
            let repo = InMemoryRepository::new();
            assert_eq!(repo.template_count(), 0);
        }

        #[test]
        fn returns_zero_template_count_when_default() {
            let repo = InMemoryRepository::default();
            assert_eq!(repo.template_count(), 0);
        }
    }

    mod accessors {
        use super::*;

        #[test]
        fn returns_zero_template_count_after_clear() {
            let repo = InMemoryRepository::new();
            let template = test_template("test");
            repo.save_template(&template).unwrap();
            assert_eq!(repo.template_count(), 1);

            repo.clear();
            assert_eq!(repo.template_count(), 0);
        }

        #[test]
        fn returns_zeroed_counters_when_harness_is_fresh() {
            let repo = InMemoryRepository::new();
            let harness = repo.harness();
            let snapshot = harness.counters().snapshot();
            assert_eq!(snapshot.reads, 0);
            assert_eq!(snapshot.writes, 0);
        }
    }

    mod template {
        use super::*;
        use crate::template::storage::testing::tests::fixtures::FailOnWrite;

        mod save {
            use super::*;

            #[test]
            fn roundtrip_save_and_find_by_id() {
                let repo = InMemoryRepository::new();
                let template = test_template("roundtrip");

                repo.save_template(&template).unwrap();

                let found = repo.find_template_by_id(*template.id()).unwrap();
                assert!(found.is_some());
                let found = found.unwrap();
                assert_eq!(found.id(), template.id());
                assert_eq!(found.path(), template.path());
                assert_eq!(found.name(), template.name());
                assert_eq!(found.body(), template.body());
            }

            #[test]
            fn roundtrip_save_and_find_by_name() {
                let repo = InMemoryRepository::new();
                let template = test_template("byname");

                repo.save_template(&template).unwrap();

                let found =
                    repo.find_template_by_name(template.name()).unwrap();
                assert!(found.is_some());
                let found = found.unwrap();
                assert_eq!(found.id(), template.id());
            }

            #[test]
            fn increments_write_counter() {
                let repo = InMemoryRepository::new();
                let template = test_template("counter");

                repo.save_template(&template).unwrap();

                let snapshot = repo.harness().counters().snapshot();
                assert_eq!(snapshot.writes, 3);
            }

            #[test]
            fn returns_storage_error_when_before_write_failure_is_injected() {
                let repo = InMemoryRepository::new()
                    .with_failure_injector(Box::new(FailOnWrite));
                let template = test_template("failwrite");

                let result = repo.save_template(&template);

                assert!(matches!(
                    result,
                    Err(TemplateRepositoryError::Storage(_))
                ));
            }
        }

        mod lookup {
            use super::*;
            use crate::template::storage::testing::tests::fixtures::FailOnRead;

            #[test]
            fn find_by_id_returns_none_when_missing() {
                let repo = InMemoryRepository::new();
                let result =
                    repo.find_template_by_id(TemplateId::new()).unwrap();
                assert!(result.is_none());
            }

            #[test]
            fn find_by_name_returns_none_when_missing() {
                let repo = InMemoryRepository::new();
                let result = repo
                    .find_template_by_name(
                        &TemplateName::try_new(
                            std::path::Path::new("templates/missing.md"),
                            std::path::Path::new("templates"),
                        )
                        .unwrap(),
                    )
                    .unwrap();
                assert!(result.is_none());
            }

            #[test]
            fn find_template_id_by_path_returns_none_when_repository_is_empty()
            {
                let repo = InMemoryRepository::new();
                let path = PathKey::try_new("templates/missing.md").unwrap();
                let result = repo.find_template_id_by_path(&path).unwrap();
                assert!(result.is_none());
            }

            #[test]
            fn find_template_id_by_path_returns_some_id_after_saving_template()
            {
                let repo = InMemoryRepository::new();
                let template = test_template("some-id-by-path");
                repo.save_template(&template).unwrap();

                let result =
                    repo.find_template_id_by_path(template.path()).unwrap();
                assert_eq!(result, Some(*template.id()));
            }

            #[test]
            fn find_template_by_path_returns_none_for_unknown_path_after_saving_different_template()
             {
                let repo = InMemoryRepository::new();
                let template = test_template("one-template");
                repo.save_template(&template).unwrap();

                let unknown_path =
                    PathKey::try_new("templates/unknown.md").unwrap();
                let result = repo.find_template_by_path(&unknown_path).unwrap();
                assert!(result.is_none());
            }

            #[test]
            fn find_template_by_path_returns_template_after_saving() {
                let repo = InMemoryRepository::new();
                let template = test_template("full-by-path");
                repo.save_template(&template).unwrap();

                let result =
                    repo.find_template_by_path(template.path()).unwrap();
                assert!(result.is_some());
                assert_eq!(result.unwrap().id(), template.id());
            }

            #[test]
            fn list_returns_empty_when_none_saved() {
                let repo = InMemoryRepository::new();
                let result = repo.list_templates().unwrap();
                assert!(result.is_empty());
            }

            #[test]
            fn list_returns_all_saved_templates() {
                let repo = InMemoryRepository::new();
                let t1 = test_template("alpha");
                let t2 = test_template("beta");

                repo.save_template(&t1).unwrap();
                repo.save_template(&t2).unwrap();

                let results = repo.list_templates().unwrap();
                assert_eq!(results.len(), 2);
                let ids: Vec<TemplateId> =
                    results.iter().map(|t| *t.id()).collect();
                assert!(ids.contains(t1.id()));
                assert!(ids.contains(t2.id()));
            }

            #[test]
            fn increments_read_counter() {
                let repo = InMemoryRepository::new();
                let template = test_template("readcount");
                repo.save_template(&template).unwrap();

                let _ = repo.find_template_by_id(*template.id()).unwrap();

                let snapshot = repo.harness().counters().snapshot();
                assert_eq!(snapshot.reads, 1);
            }

            #[test]
            fn returns_storage_error_when_before_read_failure_is_injected() {
                let repo = InMemoryRepository::new()
                    .with_failure_injector(Box::new(FailOnRead));

                let result = repo.find_template_by_id(TemplateId::new());

                assert!(matches!(
                    result,
                    Err(TemplateRepositoryError::Storage(_))
                ));
            }
        }

        mod delete {
            use super::*;

            #[test]
            fn removes_template() {
                let repo = InMemoryRepository::new();
                let template = test_template("delete-me");

                repo.save_template(&template).unwrap();
                repo.delete_template(*template.id()).unwrap();

                let found = repo.find_template_by_id(*template.id()).unwrap();
                assert!(found.is_none());
            }

            #[test]
            fn idempotent_on_missing() {
                let repo = InMemoryRepository::new();
                repo.delete_template(TemplateId::new()).unwrap();
                repo.delete_template(TemplateId::new()).unwrap();
            }

            #[test]
            fn removes_name_index() {
                let repo = InMemoryRepository::new();
                let template = test_template("name-index");

                repo.save_template(&template).unwrap();
                repo.delete_template(*template.id()).unwrap();

                let found =
                    repo.find_template_by_name(template.name()).unwrap();
                assert!(found.is_none());
            }
        }

        mod update {
            use super::*;

            #[test]
            fn upsert_overwrites_existing() {
                let repo = InMemoryRepository::new();
                let t1 = test_template("upsert");
                repo.save_template(&t1).unwrap();

                let t2 = Template::new(
                    *t1.id(),
                    t1.path().clone(),
                    TemplateName::try_new(
                        std::path::Path::new("templates/upsert.md"),
                        std::path::Path::new("templates"),
                    )
                    .unwrap(),
                    crate::template::aggregate::TemplateBody::try_new(
                        "updated content",
                    )
                    .unwrap(),
                );
                repo.save_template(&t2).unwrap();

                let found = repo.find_template_by_id(*t1.id()).unwrap();
                assert!(found.is_some());
                let found = found.unwrap();
                assert_eq!(found.body().as_str(), "updated content");
            }
        }
    }

    mod raw_view {
        use super::*;
        use crate::template::storage::testing::tests::fixtures::FailOnWrite;

        mod save {
            use super::*;

            #[test]
            fn roundtrip_save_and_find() {
                let repo = InMemoryRepository::new();
                let view = test_view("templates/note.md");

                repo.save_raw_template_view(&view).unwrap();

                let found = repo.find_raw_template_view(view.path()).unwrap();
                assert!(found.is_some());
            }

            #[test]
            fn increments_write_counter() {
                let repo = InMemoryRepository::new();
                let view = test_view("templates/counter.md");

                repo.save_raw_template_view(&view).unwrap();

                let snapshot = repo.harness().counters().snapshot();
                assert_eq!(snapshot.writes, 1);
            }

            #[test]
            fn returns_storage_error_when_before_write_failure_is_injected() {
                let repo = InMemoryRepository::new()
                    .with_failure_injector(Box::new(FailOnWrite));
                let view = test_view("templates/fail.md");

                let result = repo.save_raw_template_view(&view);

                assert!(matches!(
                    result,
                    Err(TemplateRepositoryError::Storage(_))
                ));
            }
        }

        mod lookup {
            use super::*;
            use crate::template::storage::testing::tests::fixtures::FailOnRead;

            #[test]
            fn find_by_path_returns_none_when_missing() {
                let repo = InMemoryRepository::new();
                let path = PathKey::try_new("templates/missing.md").unwrap();
                let result = repo.find_raw_template_view(&path).unwrap();
                assert!(result.is_none());
            }

            #[test]
            fn increments_read_counter() {
                let repo = InMemoryRepository::new();
                let view = test_view("templates/readcount.md");
                repo.save_raw_template_view(&view).unwrap();

                let _ = repo.find_raw_template_view(view.path()).unwrap();

                let snapshot = repo.harness().counters().snapshot();
                assert_eq!(snapshot.reads, 1);
            }

            #[test]
            fn returns_storage_error_when_before_read_failure_is_injected() {
                let repo = InMemoryRepository::new()
                    .with_failure_injector(Box::new(FailOnRead));

                let path = PathKey::try_new("templates/any.md").unwrap();
                let result = repo.find_raw_template_view(&path);

                assert!(matches!(
                    result,
                    Err(TemplateRepositoryError::Storage(_))
                ));
            }
        }

        mod delete {
            use super::*;

            #[test]
            fn removes_view() {
                let repo = InMemoryRepository::new();
                let view = test_view("templates/delete-me.md");

                repo.save_raw_template_view(&view).unwrap();
                repo.delete_raw_template_view(view.path()).unwrap();

                let found = repo.find_raw_template_view(view.path()).unwrap();
                assert!(found.is_none());
            }

            #[test]
            fn idempotent_on_missing() {
                let repo = InMemoryRepository::new();
                let path = PathKey::try_new("templates/missing.md").unwrap();
                repo.delete_raw_template_view(&path).unwrap();
                repo.delete_raw_template_view(&path).unwrap();
            }
        }

        mod batch {
            use super::*;

            #[test]
            fn save_many_persists_all_views() {
                let repo = InMemoryRepository::new();
                let v1 = test_view("templates/a.md");
                let v2 = test_view("templates/b.md");
                let v3 = test_view("templates/c.md");

                repo.save_many_raw_template_views(&[
                    v1.clone(),
                    v2.clone(),
                    v3.clone(),
                ])
                .unwrap();

                assert!(
                    repo.find_raw_template_view(v1.path()).unwrap().is_some()
                );
                assert!(
                    repo.find_raw_template_view(v2.path()).unwrap().is_some()
                );
                assert!(
                    repo.find_raw_template_view(v3.path()).unwrap().is_some()
                );
            }

            #[test]
            fn find_by_paths_returns_correct_order_with_nones() {
                let repo = InMemoryRepository::new();
                let v1 = test_view("templates/alpha.md");
                let v2 = test_view("templates/beta.md");

                repo.save_many_raw_template_views(&[v1.clone(), v2.clone()])
                    .unwrap();

                let missing = PathKey::try_new("templates/missing.md").unwrap();
                let paths =
                    vec![v1.path().clone(), missing.clone(), v2.path().clone()];

                let results =
                    repo.find_raw_template_views_by_paths(&paths).unwrap();
                assert_eq!(results.len(), 3);
                assert!(results.first().is_some_and(Option::is_some));
                assert!(results.get(1).is_some_and(Option::is_none));
                assert!(results.get(2).is_some_and(Option::is_some));
            }

            #[test]
            fn find_by_paths_empty_slice() {
                let repo = InMemoryRepository::new();
                let results =
                    repo.find_raw_template_views_by_paths(&[]).unwrap();
                assert!(results.is_empty());
            }

            #[test]
            fn save_many_empty_slice() {
                let repo = InMemoryRepository::new();
                repo.save_many_raw_template_views(&[]).unwrap();
            }
        }
    }

    mod error_conversion {
        use super::*;

        #[test]
        fn lock_poisoned_converts_to_storage_corruption() {
            let err: TemplateRepositoryError = InMemoryDbError::LockPoisoned {
                context: "test lock",
            }
            .into();
            assert!(matches!(
                err,
                TemplateRepositoryError::Storage(DbError::Corruption(ref msg))
                    if msg.contains("Lock poisoned")
            ));
        }

        #[test]
        fn injected_failure_converts_to_storage_corruption() {
            let err: TemplateRepositoryError =
                InMemoryDbError::InjectedFailure {
                    point: FPFake::BeforeRead,
                    reason: "deliberate".into(),
                }
                .into();
            assert!(matches!(
                err,
                TemplateRepositoryError::Storage(DbError::Corruption(ref msg))
                    if msg.contains("deliberate")
            ));
        }

        #[test]
        fn invariant_violation_converts_to_storage_corruption() {
            let err: TemplateRepositoryError =
                InMemoryDbError::InvariantViolation {
                    message: "bad state".into(),
                }
                .into();
            assert!(matches!(
                err,
                TemplateRepositoryError::Storage(DbError::Corruption(ref msg))
                    if msg.contains("bad state")
            ));
        }
    }
}
