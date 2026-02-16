//! Template domain ports.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use uuid::Uuid;

use super::{
    aggregate::{Template, TemplateName},
    error::TemplateError,
};

/// Command port for template-related write operations.
pub trait Command: Send + Sync {
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
pub trait Query: Send + Sync {
    /// Archived template type for zero-copy reads.
    type Archived<'archived>;

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

    /// Access a template with zero-copy.
    ///
    /// # Errors
    /// Returns `TemplateError` if query fails.
    fn with_archived<F, R>(
        &self,
        id: Uuid,
        f: F,
    ) -> Result<Option<R>, TemplateError>
    where
        F: for<'archived> FnOnce(Self::Archived<'archived>) -> R;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_trait_is_object_safe() {
        let _: Option<Box<dyn Command>> = None;
    }

    #[test]
    fn traits_are_send_and_sync() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn Command>();
    }

    #[test]
    fn query_trait_is_send_and_sync() {
        fn assert_send_sync<T: Query>() {
            fn is_send_sync<U: Send + Sync>() {}
            is_send_sync::<T>();
        }
        assert_send_sync::<FakeTemplateStorage>();
    }
}

/// In-memory template storage for testing.
#[derive(Clone, Default, Debug)]
pub struct FakeTemplateStorage {
    templates: Arc<Mutex<HashMap<Uuid, Template>>>,
    name_index: Arc<Mutex<HashMap<TemplateName, Uuid>>>,
}

impl FakeTemplateStorage {
    /// Create a new empty `FakeTemplateStorage`.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Query for FakeTemplateStorage {
    type Archived<'archived> = &'archived Template;

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
        let Ok(tn) = TemplateName::try_from(name) else {
            return Ok(None);
        };

        let id = {
            let name_index = self
                .name_index
                .lock()
                .map_err(|_e| TemplateError::Storage("Lock poisoned".into()))?;
            name_index.get(&tn).copied()
        };

        match id {
            Some(id) => self.find_by_id(id),
            None => Ok(None),
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
    fn with_archived<F, R>(
        &self,
        id: Uuid,
        f: F,
    ) -> Result<Option<R>, TemplateError>
    where
        F: for<'archived> FnOnce(Self::Archived<'archived>) -> R,
    {
        let templates = self
            .templates
            .lock()
            .map_err(|_e| TemplateError::Storage("Lock poisoned".into()))?;
        Ok(templates.get(&id).map(f))
    }
}

impl Command for FakeTemplateStorage {
    #[inline]
    fn create(&self, template: &Template) -> Result<(), TemplateError> {
        let mut name_index = self
            .name_index
            .lock()
            .map_err(|_e| TemplateError::Storage("Lock poisoned".into()))?;

        if name_index.contains_key(template.name()) {
            return Err(TemplateError::AlreadyExists(
                template.name().to_string(),
            ));
        }

        let mut templates = self
            .templates
            .lock()
            .map_err(|_e| TemplateError::Storage("Lock poisoned".into()))?;

        templates.insert(template.id(), template.clone());
        name_index.insert(template.name().clone(), template.id());

        drop(name_index);
        drop(templates);

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
                    template.name().to_string(),
                ));
            }

            let old_name = old.name().clone();
            name_index.remove(&old_name);
            name_index.insert(template.name().clone(), template.id());
        }

        templates.insert(template.id(), template.clone());

        drop(name_index);
        drop(templates);

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

        drop(name_index);
        drop(templates);

        Ok(())
    }
}
