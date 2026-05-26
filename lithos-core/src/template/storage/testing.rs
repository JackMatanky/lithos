//! Testing utilities for template storage.
#![cfg(any(test, feature = "testing"))]
#![allow(
    dead_code,
    reason = "Testing utilities may be unused when features are active but \
              tests are not running"
)]

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    db::testing::{FailurePoint, InMemoryHarness, read_lock, write_lock},
    template::{
        aggregate::{Template, TemplateId, TemplateName},
        error::TemplateRepositoryError,
        repository::{ReadRepository, WriteRepository},
    },
};

/// HashMap-backed [`Repository`] implementation for pure unit tests.
#[derive(Debug, Clone)]
pub(crate) struct InMemoryRepository {
    harness: Arc<InMemoryHarness>,
    templates: Arc<RwLock<HashMap<TemplateId, Template>>>,
    name_to_id: Arc<RwLock<HashMap<TemplateName, TemplateId>>>,
}

impl InMemoryRepository {
    /// Creates a new empty in-memory repository.
    #[must_use]
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            harness: Arc::new(InMemoryHarness::new()),
            templates: Arc::new(RwLock::new(HashMap::new())),
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Returns a reference to the test harness for instrumentation.
    #[must_use]
    pub(crate) fn harness(&self) -> &InMemoryHarness {
        &self.harness
    }

    /// Creates a new repository with the specified test harness.
    #[must_use]
    pub(crate) fn with_harness(harness: InMemoryHarness) -> Self {
        Self {
            harness: Arc::new(harness),
            templates: Arc::new(RwLock::new(HashMap::new())),
            name_to_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// Convert `db::testing::InMemoryDbError` directly to
/// `TemplateRepositoryError`.
#[cfg(any(test, feature = "testing"))]
impl From<crate::db::testing::InMemoryDbError> for TemplateRepositoryError {
    #[inline]
    fn from(err: crate::db::testing::InMemoryDbError) -> Self {
        use crate::db::{DbError, testing::InMemoryDbError as DbTestError};

        let db_error = match err {
            DbTestError::LockPoisoned {
                context,
            } => DbError::Corruption(format!("Lock poisoned: {context}")),
            DbTestError::InjectedFailure {
                reason,
                ..
            } => DbError::Corruption(format!("Injected failure: {reason}")),
            DbTestError::InvariantViolation {
                message,
            } => DbError::Corruption(message.into()),
        };

        TemplateRepositoryError::Storage(db_error.to_string().into_boxed_str())
    }
}

impl Default for InMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadRepository for InMemoryRepository {
    fn find_template_by_id(
        &self,
        id: TemplateId,
    ) -> Result<Option<Template>, TemplateRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let templates = read_lock(&self.templates, "find_template_by_id")?;
        self.harness.counters().inc_read();

        Ok(templates.get(&id).cloned())
    }

    fn find_many_templates_by_id(
        &self,
        ids: &[TemplateId],
    ) -> Result<Vec<Option<Template>>, TemplateRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let templates =
            read_lock(&self.templates, "find_many_templates_by_id")?;
        self.harness.counters().inc_read();

        Ok(ids.iter().map(|id| templates.get(id).cloned()).collect())
    }

    fn find_template_id_by_name(
        &self,
        name: &TemplateName,
    ) -> Result<Option<TemplateId>, TemplateRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let name_to_id =
            read_lock(&self.name_to_id, "find_template_id_by_name")?;
        self.harness.counters().inc_read();

        Ok(name_to_id.get(name).copied())
    }

    fn list_templates(&self) -> Result<Vec<Template>, TemplateRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeRead)?;

        let templates = read_lock(&self.templates, "list_templates")?;
        self.harness.counters().inc_read();

        Ok(templates.values().cloned().collect())
    }
}

impl WriteRepository for InMemoryRepository {
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

        templates.insert(template.id(), template.clone());
        name_to_id.insert(template.name().clone(), template.id());

        Ok(())
    }

    fn save_many_templates(
        &self,
        templates: &[Template],
    ) -> Result<(), TemplateRepositoryError> {
        self.harness.fail_at(FailurePoint::BeforeWrite)?;

        let mut templates_map =
            write_lock(&self.templates, "save_many_templates (templates)")?;
        self.harness.counters().inc_write();

        let mut name_to_id_map =
            write_lock(&self.name_to_id, "save_many_templates (name_to_id)")?;
        self.harness.counters().inc_write();

        for template in templates {
            templates_map.insert(template.id(), template.clone());
            name_to_id_map.insert(template.name().clone(), template.id());
        }

        Ok(())
    }

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

        if let Some(template) = templates.remove(&id) {
            let name = template.name().clone();
            let _: Option<TemplateId> = name_to_id.remove(&name);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::testing::{FailureInjector, FailurePoint, InMemoryDbError};

    mod fixtures {
        use super::*;

        pub(super) struct FailOnWrite;
        pub(super) struct FailOnRead;

        impl FailureInjector for FailOnWrite {
            fn fail_at(
                &self,
                point: FailurePoint,
            ) -> Result<(), InMemoryDbError> {
                if point == FailurePoint::BeforeWrite {
                    return Err(InMemoryDbError::InjectedFailure {
                        point,
                        reason: "write injection".into(),
                    });
                }

                Ok(())
            }
        }

        impl FailureInjector for FailOnRead {
            fn fail_at(
                &self,
                point: FailurePoint,
            ) -> Result<(), InMemoryDbError> {
                if point == FailurePoint::BeforeRead {
                    return Err(InMemoryDbError::InjectedFailure {
                        point,
                        reason: "read injection".into(),
                    });
                }

                Ok(())
            }
        }
    }

    mod lookup {
        use super::*;
        use crate::template::storage::testing::tests::fixtures::FailOnRead;

        #[test]
        fn increments_read_counter_when_find_template_by_id_succeeds() {
            let repo = InMemoryRepository::new();
            let name = TemplateName::try_from("test").unwrap();
            let template =
                Template::try_new(&name, None, vec![], HashMap::new()).unwrap();
            repo.save_template(&template).unwrap();

            let _result = repo.find_template_by_id(template.id()).unwrap();

            let snapshot = repo.harness().counters().snapshot();
            assert_eq!(snapshot.reads, 1);
        }

        #[test]
        fn find_many_templates_by_id_preserves_order_and_handles_missing() {
            let repo = InMemoryRepository::new();
            let name1 = TemplateName::try_from("t1").unwrap();
            let t1 = Template::try_new(&name1, None, vec![], HashMap::new())
                .unwrap();
            let name2 = TemplateName::try_from("t2").unwrap();
            let t2 = Template::try_new(&name2, None, vec![], HashMap::new())
                .unwrap();

            repo.save_template(&t1).unwrap();
            repo.save_template(&t2).unwrap();

            let missing_id = TemplateId::new();
            let ids = vec![t1.id(), missing_id, t2.id()];
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
        fn returns_storage_error_when_before_read_failure_is_injected() {
            let harness = InMemoryHarness::with_injector(Box::new(FailOnRead));
            let repo = InMemoryRepository::with_harness(harness);

            let result = repo.find_template_by_id(TemplateId::new());

            assert!(matches!(result, Err(TemplateRepositoryError::Storage(_))));
        }
    }

    mod update {
        use super::*;
        use crate::template::storage::testing::tests::fixtures::FailOnWrite;

        #[test]
        fn persists_template_when_saved() {
            let repo = InMemoryRepository::new();
            let name = TemplateName::try_from("test-template").unwrap();
            let template =
                Template::try_new(&name, None, vec![], HashMap::new()).unwrap();
            let id = template.id();

            repo.save_template(&template).expect("save should succeed");

            let found =
                repo.find_template_by_id(id).expect("lookup should succeed");
            assert_eq!(found.unwrap().name(), &name);
        }

        #[test]
        fn save_many_templates_persists_all() {
            let repo = InMemoryRepository::new();
            let name1 = TemplateName::try_from("t1").unwrap();
            let t1 = Template::try_new(&name1, None, vec![], HashMap::new())
                .unwrap();
            let name2 = TemplateName::try_from("t2").unwrap();
            let t2 = Template::try_new(&name2, None, vec![], HashMap::new())
                .unwrap();

            repo.save_many_templates(&[t1.clone(), t2.clone()]).unwrap();

            assert_eq!(repo.find_template_by_id(t1.id()).unwrap().unwrap(), t1);
            assert_eq!(repo.find_template_by_id(t2.id()).unwrap().unwrap(), t2);
        }

        #[test]
        fn delete_template_is_idempotent() {
            let repo = InMemoryRepository::new();
            let name = TemplateName::try_from("test").unwrap();
            let template =
                Template::try_new(&name, None, vec![], HashMap::new()).unwrap();
            let id = template.id();

            repo.save_template(&template).unwrap();
            repo.delete_template(id).unwrap();
            repo.delete_template(id).unwrap();

            assert!(repo.find_template_by_id(id).unwrap().is_none());
        }

        #[test]
        fn increments_write_counter_when_save_template_succeeds() {
            let repo = InMemoryRepository::new();
            let name = TemplateName::try_from("test").unwrap();
            let template =
                Template::try_new(&name, None, vec![], HashMap::new()).unwrap();

            repo.save_template(&template).unwrap();

            let snapshot = repo.harness().counters().snapshot();
            // 1 for templates map, 1 for name_to_id map
            assert_eq!(snapshot.writes, 2);
        }

        #[test]
        fn returns_storage_error_when_before_write_failure_is_injected() {
            let harness = InMemoryHarness::with_injector(Box::new(FailOnWrite));
            let repo = InMemoryRepository::with_harness(harness);
            let name = TemplateName::try_from("test").unwrap();
            let template =
                Template::try_new(&name, None, vec![], HashMap::new()).unwrap();

            let result = repo.save_template(&template);

            assert!(matches!(result, Err(TemplateRepositoryError::Storage(_))));
        }
    }
}
