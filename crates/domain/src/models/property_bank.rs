//! PropertyBank domain entity for centralized property management.

use std::collections::HashMap;

use crate::{
    errors::DomainError,
    events::PropertyBankUpdated,
    models::{property::Property, schema::DomainEvent},
};

/// Registry of reusable Property definitions with dual indexing.
///
/// Provides O(1) lookup by ID and Name.
#[derive(
    Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub struct PropertyBank {
    /// Index mapping ID -> index in properties vector.
    pub id_index: HashMap<String, usize>,
    /// Index mapping Name -> index in properties vector.
    pub name_index: HashMap<String, usize>,
    /// Dense storage of properties.
    pub properties: Vec<Property>,
}

impl PropertyBank {
    /// Get all properties in the bank.
    #[inline]
    pub fn all(&self) -> impl Iterator<Item = &Property> {
        self.properties.iter()
    }

    fn create_updated_event(&self) -> DomainEvent {
        DomainEvent::PropertyBankUpdated(PropertyBankUpdated::new(
            self.properties.len(),
            chrono::Utc::now().timestamp(),
        ))
    }

    /// Decodes a `$ref` path to a Property.
    ///
    /// This method performs a key lookup for a property. Format-specific parsing
    /// (e.g., handling "#/properties/") must be handled by the adapters.
    ///
    /// # Errors
    /// Returns `PropertyNotFound` if key does not exist.
    #[inline]
    pub fn decode(&self, key: &str) -> Result<&Property, DomainError> {
        self.get_by_name(key)
            .ok_or_else(|| DomainError::PropertyNotFound(key.to_owned()))
    }

    /// Gets a property by name or ID.
    #[inline]
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Property> {
        // Try by ID first (HashMap lookup is O(1))
        if let Some(prop) = self.get_by_id(key) {
            return Some(prop);
        }
        // Fall back to name lookup (O(1))
        self.get_by_name(key)
    }

    /// Lookup property by ID (O(1)).
    #[inline]
    #[must_use]
    pub fn get_by_id(&self, id: &str) -> Option<&Property> {
        let &idx = self.id_index.get(id)?;
        self.properties.get(idx)
    }

    /// Lookup property by Name (O(1)).
    #[inline]
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&Property> {
        let &idx = self.name_index.get(name)?;
        self.properties.get(idx)
    }

    /// Checks if a property exists by ID.
    #[inline]
    #[must_use]
    pub fn has_id(&self, id: &str) -> bool {
        self.id_index.contains_key(id)
    }

    /// Checks if a property exists by name.
    #[inline]
    #[must_use]
    pub fn has_name(&self, name: &str) -> bool {
        self.name_index.contains_key(name)
    }

    /// Create a new empty `PropertyBank`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_domain::models::property_bank::PropertyBank;
    ///
    /// let bank = PropertyBank::new();
    /// assert_eq!(bank.all().count(), 0);
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a property in the bank.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_domain::models::property_bank::PropertyBank;
    /// use lithos_domain::models::property::{Property, PropertyName};
    /// use lithos_domain::models::property_spec::{PropertySpec, BoolSpec};
    ///
    /// let mut bank = PropertyBank::new();
    /// let name = PropertyName::new("is_active".to_string()).unwrap();
    /// let spec = PropertySpec::Bool(BoolSpec::default());
    /// let id = Property::compute_id(name.as_str(), &spec).unwrap();
    /// let property = Property::new(id, name, true, false, spec).unwrap();
    ///
    /// let (count, event) = bank.register(property).unwrap();
    /// assert_eq!(count, 1);
    /// ```
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    #[inline]
    pub fn register(
        &mut self,
        property: Property,
    ) -> Result<(usize, DomainEvent), DomainError> {
        property.validate()?;

        // Idempotent success if ID already exists
        if self.id_index.contains_key(&property.id) {
            return Ok((self.properties.len(), self.create_updated_event()));
        }

        // Prevent duplicate names
        self.validate_name_unique(property.name.as_str())?;

        let id = property.id.clone();
        let name = property.name.to_string();
        let idx = self.properties.len();

        self.id_index.insert(id, idx);
        self.name_index.insert(name, idx);
        self.properties.push(property);

        Ok((self.properties.len(), self.create_updated_event()))
    }

    fn validate_name_unique(&self, name: &str) -> Result<(), DomainError> {
        if self.name_index.contains_key(name) {
            return Err(DomainError::DuplicatePropertyName(name.to_owned()));
        }
        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Unit tests use unwrap/expect for simplicity"
)]
mod tests {
    use super::*;
    use crate::models::{
        property::{Property, PropertyName},
        property_spec::{BoolSpec, PropertySpec, StringSpec},
    };

    /// 3.3-UNIT-023: `is_idempotent_on_identical_registration`.
    /// Priority: P1.
    #[test]
    fn is_idempotent_on_identical_registration() {
        // GIVEN a PropertyBank and an existing property
        let mut bank = PropertyBank::new();
        let spec = PropertySpec::String(StringSpec::default());
        let name = PropertyName::new("test".to_owned()).unwrap();
        let id = Property::compute_id(name.as_str(), &spec).unwrap();
        let prop = Property::new(id, name, false, false, spec).unwrap();

        // WHEN registering the same property twice
        bank.register(prop.clone()).unwrap();
        let (count, _) = bank.register(prop).unwrap();

        // THEN the count remains 1
        assert_eq!(count, 1);
        assert_eq!(bank.all().count(), 1);
    }

    /// 3.3-UNIT-020: `maintains_dual_indices_for_fast_lookup`.
    /// Priority: P1.
    #[test]
    fn maintains_dual_indices_for_fast_lookup() {
        // GIVEN a PropertyBank and a Property definition
        let mut bank = PropertyBank::new();
        let spec = PropertySpec::String(StringSpec::default());
        let name_str = "test".to_owned();
        let name = PropertyName::new(name_str.clone()).unwrap();
        let id = Property::compute_id(&name_str, &spec).unwrap();
        let prop = Property::new(id.clone(), name, false, false, spec).unwrap();

        // WHEN registering the property
        bank.register(prop).unwrap();

        // THEN it should be accessible by both ID and name
        assert!(bank.get_by_id(&id).is_some());
        assert!(bank.get_by_name("test").is_some());
    }

    /// 3.3-UNIT-024: `rejects_duplicate_names_with_different_definitions`.
    /// Priority: P1.
    #[test]
    fn rejects_duplicate_names_with_different_definitions() {
        // GIVEN a PropertyBank with a registered property
        let mut bank = PropertyBank::new();
        let spec1 = PropertySpec::String(StringSpec::default());
        let name = PropertyName::new("test".to_owned()).unwrap();
        let id1 = Property::compute_id(name.as_str(), &spec1).unwrap();
        let prop1 =
            Property::new(id1, name.clone(), false, false, spec1).unwrap();
        bank.register(prop1).unwrap();

        // WHEN registering a different definition with the same name
        let spec2 = PropertySpec::Bool(BoolSpec::default());
        let id2 = Property::compute_id(name.as_str(), &spec2).unwrap();
        let prop2 = Property::new(id2, name, false, false, spec2).unwrap();
        let res = bank.register(prop2);

        // THEN it must return a DuplicatePropertyName error
        assert!(matches!(res, Err(DomainError::DuplicatePropertyName(_))));
    }
}
