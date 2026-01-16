//! `PropertyBank` registry for reusable property definitions.

use std::collections::HashMap;

use crate::{
    errors::DomainError,
    models::schema::{
        core::DomainEvent,
        property::{Property, PropertySpec},
    },
};

/// Singleton registry of reusable Property definitions.
///
/// # Examples
///
/// ```
/// use lithos_domain::models::schema::{PropertyBank, Property, PropertySpec, StringSpec};
///
/// let mut bank = PropertyBank::new();
/// let spec = PropertySpec::String(StringSpec::default());
/// let name = "status".to_string();
/// let id = Property::compute_id(&name, &spec).unwrap();
/// let property = Property::new(id, name, true, false, spec).unwrap();
///
/// bank.register(property).expect("Successfully registered");
/// assert_eq!(bank.all().count(), 1);
/// ```
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct PropertyBank {
    /// Domain events pending emission.
    #[serde(skip)]
    pub pending_events: Vec<DomainEvent>,
    /// Map of property ID -> Property.
    pub properties: HashMap<String, Property>,
}

impl PropertyBank {
    /// Adds a domain event to the pending events collection.
    #[inline]
    fn add_event(&mut self, event: DomainEvent) {
        self.pending_events.push(event);
    }

    /// Get all properties in the bank.
    #[inline]
    pub fn all(&self) -> impl Iterator<Item = &Property> {
        self.properties.values()
    }

    /// Lookup a property by ID.
    #[inline]
    #[must_use]
    pub fn lookup(&self, id: &str) -> Option<&Property> {
        self.properties.get(id)
    }

    /// Lookup a property by name and spec (computes ID internally).
    #[inline]
    #[must_use]
    pub fn lookup_by_definition(
        &self,
        name: &str,
        spec: &PropertySpec,
    ) -> Option<&Property> {
        let id = Property::compute_id(name, spec).ok()?;
        self.lookup(&id)
    }

    /// Create a new empty `PropertyBank`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending_events: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Register a property in the bank.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    #[inline]
    pub fn register(
        &mut self,
        property: Property,
    ) -> Result<&Property, DomainError> {
        property.validate()?;
        let id = property.id.clone();

        self.properties.entry(id.clone()).or_insert(property);

        let count = self.properties.len();
        self.add_event(DomainEvent::PropertyBankUpdated(
            crate::events::PropertyBankUpdated::new(
                count,
                chrono::Utc::now().timestamp(),
            ),
        ));

        self.properties.get(&id).ok_or_else(|| {
            DomainError::Unexpected(
                "Failed to retrieve property after registration".to_owned(),
            )
        })
    }

    /// Resolve $ref pointer to Property.
    ///
    /// Format: `#/properties/name`.
    ///
    /// # Errors
    /// Returns `DomainError` if resolution fails.
    #[inline]
    pub fn resolve_ref(
        &self,
        ref_path: &str,
    ) -> Result<&Property, DomainError> {
        let prefix = "#/properties/";
        if !ref_path.starts_with(prefix) {
            return Err(DomainError::ValidationFailed(format!(
                "Invalid ref path format: {ref_path}"
            )));
        }
        let name = ref_path.get(prefix.len()..).ok_or_else(|| {
            DomainError::ValidationFailed(format!(
                "Empty property name in ref: {ref_path}"
            ))
        })?;
        self.properties
            .values()
            .find(|p| p.name == name)
            .ok_or_else(|| DomainError::PropertyNotFound(name.to_owned()))
    }

    /// Returns all pending domain events and clears the collection.
    #[inline]
    #[must_use]
    pub fn take_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.pending_events)
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Unit tests use unwrap/expect for simplicity"
)]
mod tests {
    mod register {
        use super::super::*;
        use crate::models::schema::fixtures::example_property;

        /// 3.3-UNIT-008: `deduplicates_properties_on_registration_when_id_matches`.
        #[test]
        fn deduplicates_properties_on_registration_when_id_matches() {
            let mut bank = PropertyBank::new();
            let prop = example_property();

            bank.register(prop.clone()).unwrap();
            bank.register(prop.clone()).unwrap();

            assert_eq!(
                bank.all().count(),
                1,
                "Should not duplicate identical properties"
            );
        }
    }

    mod resolve_ref {
        use super::super::*;
        use crate::models::schema::fixtures::example_property;

        /// 3.3-UNIT-009: `resolves_valid_ref_path_to_property`.
        #[test]
        fn resolves_valid_ref_path_to_property() {
            let mut bank = PropertyBank::new();
            let prop = example_property(); // name is "status"
            bank.register(prop).unwrap();

            let p = bank.resolve_ref("#/properties/status").unwrap();
            assert_eq!(p.name, "status");
        }
    }
}
