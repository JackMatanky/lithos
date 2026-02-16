//! Template domain ports.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use uuid::Uuid;

use super::{
    aggregate::Template, composition::Composition, error::TemplateError,
};

/// Command port for template-related write operations.
pub trait TemplateCommandPort: Send + Sync {
    /// Creates a new template.
    ///
    /// # Errors
    /// Returns `TemplateError` if creation fails.
    fn create(&self, template: &Template) -> Result<(), TemplateError>;

    /// Deletes a template by ID.
    ///
    /// # Errors
    /// Returns `TemplateError` if deletion fails.
    fn delete(&self, id: Uuid) -> Result<(), TemplateError>;

    /// Updates an existing template.
    ///
    /// # Errors
    /// Returns `TemplateError` if update fails.
    fn update(&self, template: &Template) -> Result<(), TemplateError>;
}

/// Query port for template-related read operations.
pub trait TemplateQueryPort: Send + Sync {
    /// Find a template by ID.
    ///
    /// # Errors
    /// Returns `TemplateError` if query fails.
    fn find_by_id(&self, id: Uuid) -> Result<Option<Template>, TemplateError>;

    /// Find a template by name.
    ///
    /// # Errors
    /// Returns `TemplateError` if query fails.
    fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Template>, TemplateError>;

    /// Lists all templates.
    ///
    /// # Errors
    /// Returns `TemplateError` if query fails.
    fn list(&self) -> Result<Vec<Template>, TemplateError>;

    /// Resolves a template composition.
    ///
    /// # Errors
    /// Returns `TemplateError` if resolution fails.
    fn resolve(
        &self,
        composition: &Composition,
    ) -> Result<Template, TemplateError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_trait_is_object_safe() {
        let _: Option<Box<dyn TemplateCommandPort>> = None;
    }

    #[test]
    fn query_trait_is_object_safe() {
        let _: Option<Box<dyn TemplateQueryPort>> = None;
    }

    #[test]
    fn traits_are_send_and_sync() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn TemplateCommandPort>();
        assert_send_sync::<dyn TemplateQueryPort>();
    }
}

/// In-memory template storage for testing.
#[derive(Clone, Default, Debug)]
pub struct FakeTemplateStorage {
    templates: Arc<Mutex<HashMap<Uuid, Template>>>,
    name_index: Arc<Mutex<HashMap<String, Uuid>>>,
}

impl FakeTemplateStorage {
    /// Create a new empty `FakeTemplateStorage`.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl TemplateQueryPort for FakeTemplateStorage {
    #[inline]
    fn find_by_id(&self, id: Uuid) -> Result<Option<Template>, TemplateError> {
        let templates = self
            .templates
            .lock()
            .map_err(|_e| TemplateError::Storage("Lock poisoned".into()))?;
        Ok(templates.get(&id).cloned())
    }

    #[inline]
    fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<Template>, TemplateError> {
        let name_index = self
            .name_index
            .lock()
            .map_err(|_e| TemplateError::Storage("Lock poisoned".into()))?;
        let templates = self
            .templates
            .lock()
            .map_err(|_e| TemplateError::Storage("Lock poisoned".into()))?;

        if let Some(&id) = name_index.get(name) {
            Ok(templates.get(&id).cloned())
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn list(&self) -> Result<Vec<Template>, TemplateError> {
        let templates = self
            .templates
            .lock()
            .map_err(|_e| TemplateError::Storage("Lock poisoned".into()))?;
        Ok(templates.values().cloned().collect())
    }

    #[inline]
    fn resolve(
        &self,
        _composition: &Composition,
    ) -> Result<Template, TemplateError> {
        #[expect(
            clippy::unimplemented,
            reason = "Not needed for catalog tests"
        )]
        {
            unimplemented!("resolve() not needed for catalog tests")
        }
    }
}

impl TemplateCommandPort for FakeTemplateStorage {
    #[inline]
    fn create(&self, template: &Template) -> Result<(), TemplateError> {
        let mut templates = self
            .templates
            .lock()
            .map_err(|_e| TemplateError::Storage("Lock poisoned".into()))?;
        let mut name_index = self
            .name_index
            .lock()
            .map_err(|_e| TemplateError::Storage("Lock poisoned".into()))?;

        if name_index.contains_key(template.name()) {
            return Err(TemplateError::AlreadyExists(template.name().into()));
        }

        templates.insert(template.id(), template.clone());
        name_index.insert(template.name().into(), template.id());

        Ok(())
    }

    #[inline]
    fn update(&self, template: &Template) -> Result<(), TemplateError> {
        let mut templates = self
            .templates
            .lock()
            .map_err(|_e| TemplateError::Storage("Lock poisoned".into()))?;
        let mut name_index = self
            .name_index
            .lock()
            .map_err(|_e| TemplateError::Storage("Lock poisoned".into()))?;

        let old = templates.get(&template.id()).ok_or_else(|| {
            TemplateError::NotFound(template.id().to_string())
        })?;

        if old.name() != template.name() {
            if name_index.contains_key(template.name()) {
                return Err(TemplateError::AlreadyExists(
                    template.name().into(),
                ));
            }

            name_index.remove(old.name());
            name_index.insert(template.name().into(), template.id());
        }

        templates.insert(template.id(), template.clone());

        Ok(())
    }

    #[inline]
    fn delete(&self, id: Uuid) -> Result<(), TemplateError> {
        let mut templates = self
            .templates
            .lock()
            .map_err(|_e| TemplateError::Storage("Lock poisoned".into()))?;
        let mut name_index = self
            .name_index
            .lock()
            .map_err(|_e| TemplateError::Storage("Lock poisoned".into()))?;

        if let Some(template) = templates.remove(&id) {
            name_index.remove(template.name());
        }

        Ok(())
    }
}
