//! Property domain entities and value objects.

#![allow(
    clippy::module_name_repetitions,
    reason = "Core domain logic and naming convention where Property prefix is descriptive"
)]

use std::fmt::{Debug, Display};

use crate::{errors::DomainError, models::property_spec::PropertySpec};

/// Validated property name value object.
///
/// Enforces invariants:
/// - Non-empty
/// - Max 64 characters
/// - Matches regex `^[a-z0-9_-]+$`
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "String", into = "String")]
pub struct PropertyName(String);

impl PropertyName {
    /// Get string reference.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Create a new `PropertyName` with validation.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    #[inline]
    pub fn new(name: String) -> Result<Self, DomainError> {
        Self::validate_non_empty(&name)?;
        Self::validate_length(&name)?;
        Self::validate_format(&name)?;
        Ok(Self(name))
    }

    fn validate_format(name: &str) -> Result<(), DomainError> {
        let re = regex::Regex::new("^[a-z0-9_-]+$")
            .map_err(|e| DomainError::ValidationFailed(e.to_string()))?;
        if !re.is_match(name) {
            return Err(DomainError::InvalidPropertyName(name.to_owned()));
        }
        Ok(())
    }

    fn validate_length(name: &str) -> Result<(), DomainError> {
        if name.len() > 64 {
            return Err(DomainError::PropertyNameTooLong(name.len()));
        }
        Ok(())
    }

    fn validate_non_empty(name: &str) -> Result<(), DomainError> {
        if name.is_empty() {
            return Err(DomainError::EmptyPropertyName);
        }
        Ok(())
    }
}

impl Display for PropertyName {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for PropertyName {
    type Error = DomainError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PropertyName> for String {
    #[inline]
    fn from(val: PropertyName) -> Self {
        val.0
    }
}

impl AsRef<str> for PropertyName {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Reusable property definition with type-specific validation.
///
/// This is the resolved entity used in the Domain layer.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Property {
    /// Whether property accepts array of values.
    pub array: bool,
    /// Deterministic ID: hash(name + spec content).
    pub id: String,
    /// Property name.
    pub name: PropertyName,
    /// Whether property is required.
    pub required: bool,
    /// Type-specific validation specification.
    pub spec: PropertySpec,
}

impl Property {
    /// Compute deterministic ID from name and spec using Blake3.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_domain::models::property::Property;
    /// use lithos_domain::models::property_spec::{PropertySpec, BoolSpec};
    ///
    /// let name = "is_active";
    /// let spec = PropertySpec::Bool(BoolSpec::default());
    /// let id = Property::compute_id(name, &spec).unwrap();
    /// assert_eq!(id.len(), 64); // Blake3 hex length
    /// ```
    ///
    /// # Errors
    /// Returns `DomainError` if canonicalization fails.
    #[inline]
    pub fn compute_id(
        name: &str,
        spec: &PropertySpec,
    ) -> Result<String, DomainError> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(name.as_bytes());
        // Use canonical JSON representation for spec content to ensure absolute determinism
        let spec_json = serde_json::to_string(spec).map_err(|e| {
            DomainError::ValidationFailed(format!(
                "Failed to canonicalize spec: {e}"
            ))
        })?;
        hasher.update(spec_json.as_bytes());
        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Create a new property with validation.
    ///
    /// # Identity Integrity
    /// The `id` must be provided by the caller but will be validated against the
    /// computed hash of the property's definition (name + spec) using Blake3.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_domain::models::property::{Property, PropertyName};
    /// use lithos_domain::models::property_spec::{PropertySpec, BoolSpec};
    ///
    /// let name = PropertyName::new("is_active".to_string()).unwrap();
    /// let spec = PropertySpec::Bool(BoolSpec::default());
    /// let id = Property::compute_id(name.as_str(), &spec).unwrap();
    ///
    /// let property = Property::new(id, name, true, false, spec).unwrap();
    /// assert!(property.required);
    /// ```
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails or the provided ID does not match
    /// the computed definition hash.
    #[inline]
    pub fn new(
        id: String,
        name: PropertyName,
        required: bool,
        array: bool,
        spec: PropertySpec,
    ) -> Result<Self, DomainError> {
        Self::validate_id_integrity(&id, name.as_str(), &spec)?;
        let property = Self {
            array,
            id,
            name,
            required,
            spec,
        };
        property.validate()?;
        Ok(property)
    }

    /// Validate property structure and constraints.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    #[inline]
    pub fn validate(&self) -> Result<(), DomainError> {
        // Name validation is handled by PropertyName type.
        // We only validate the spec constraints here.
        self.spec.validate_spec()?;
        Ok(())
    }

    fn validate_id_integrity(
        id: &str,
        name: &str,
        spec: &PropertySpec,
    ) -> Result<(), DomainError> {
        let computed_id = Self::compute_id(name, spec)?;
        if id != computed_id {
            return Err(DomainError::ValidationFailed(format!(
                "Property ID mismatch for {name}. Expected {computed_id}, got {id}"
            )));
        }
        Ok(())
    }
}

/// Raw property input definition (Inline or Ref).
///
/// Matches the `PropertyOrRef` schema definition used in input files.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
#[expect(
    clippy::module_name_repetitions,
    reason = "Core domain logic and naming"
)]
pub enum RawProperty {
    /// An inline property definition.
    Inline(RawPropertyInline),
    /// A reference to a property in the `PropertyBank`.
    Ref(RawPropertyRef),
}

/// Reference variant of a raw property.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyRef {
    /// The reference string (e.g., "#/properties/title").
    #[serde(rename = "$ref")]
    pub ref_path: String,
}

/// Inline variant of a raw property.
///
/// Corresponds to the `Property` definition in the JSON schema but used as input.
/// Differs from `Property` entity by missing the `id` (which is computed later)
/// and using raw types before validation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyInline {
    /// Whether property accepts array of values.
    #[serde(default)]
    pub array: bool,
    /// Property name.
    pub name: String,
    /// Whether property is required.
    #[serde(default)]
    pub required: bool,
    /// Type-specific validation constraints.
    pub spec: PropertySpec,
}

/// Test fixtures and builders for `Property`.
#[cfg(test)]
pub mod fixtures {
    use super::*;
    use crate::models::property_spec::StringSpec;

    /// 3.3-UNIT-018: `PropertyBuilder` for flexible test data generation.
    pub struct PropertyBuilder {
        array: bool,
        name: String,
        required: bool,
        spec: PropertySpec,
    }

    impl Default for PropertyBuilder {
        #[inline]
        fn default() -> Self {
            Self {
                array: false,
                name: "test_property".to_owned(),
                required: false,
                spec: PropertySpec::String(StringSpec::default()),
            }
        }
    }

    impl PropertyBuilder {
        /// Sets whether the property is an array.
        #[inline]
        #[must_use]
        pub fn array(mut self, array: bool) -> Self {
            self.array = array;
            self
        }

        /// Builds the `Property` entity.
        ///
        /// # Panics
        /// Panics if the property configuration is invalid.
        #[inline]
        #[must_use]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test builder setup expects valid inputs"
        )]
        pub fn build(self) -> Property {
            let name = PropertyName::new(self.name).expect("Valid name");
            let id = Property::compute_id(name.as_str(), &self.spec)
                .expect("Valid ID");
            Property::new(id, name, self.required, self.array, self.spec)
                .expect("Valid property")
        }

        /// Sets the name of the property.
        #[inline]
        #[must_use]
        pub fn name(mut self, name: &str) -> Self {
            self.name = name.to_owned();
            self
        }

        /// Creates a new `PropertyBuilder` with default values.
        #[inline]
        #[must_use]
        pub fn new() -> Self {
            Self::default()
        }

        /// Sets whether the property is required.
        #[inline]
        #[must_use]
        pub fn required(mut self, required: bool) -> Self {
            self.required = required;
            self
        }

        /// Sets the specification for the property.
        #[inline]
        #[must_use]
        pub fn spec(mut self, spec: PropertySpec) -> Self {
            self.spec = spec;
            self
        }
    }

    /// Helper for creating a default property.
    #[inline]
    #[must_use]
    pub fn example_property() -> Property {
        PropertyBuilder::new().build()
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Unit tests use unwrap/expect for simplicity"
)]
mod tests {
    mod property {
        use super::super::*;
        use crate::models::property_spec::StringSpec;

        /// 3.3-UNIT-005: `id_is_deterministic_using_blake3_and_canonical_json`.
        /// Priority: P0.
        #[test]
        fn id_is_deterministic_using_blake3_and_canonical_json() {
            // GIVEN a property specification
            let spec = PropertySpec::String(StringSpec::default());

            // WHEN computing IDs for identical and different definitions
            let id1 = Property::compute_id("title", &spec).unwrap();
            let id2 = Property::compute_id("title", &spec).unwrap();
            let id3 = Property::compute_id("other", &spec).unwrap();

            // THEN identical definitions must produce same ID
            assert_eq!(id1, id2, "Identical definitions must produce same ID");
            // AND different names must produce different IDs
            assert_ne!(id1, id3, "Different names must produce different IDs");
        }

        /// 3.3-UNIT-006: `returns_error_when_property_name_format_is_invalid`.
        /// Priority: P1.
        #[test]
        fn returns_error_when_property_name_format_is_invalid() {
            // GIVEN a property specification
            let spec = PropertySpec::String(StringSpec::default());
            // AND a set of invalid names
            let invalid_names = vec!["Invalid Name", "invalid.name", ""];

            // WHEN validating property names
            for name_str in invalid_names {
                // THEN it should reject invalid names
                assert!(
                    PropertyName::new(name_str.to_owned()).is_err(),
                    "Should reject invalid name: {name_str}"
                );
            }
            let _: PropertySpec = spec;
        }
    }

    mod property_name {
        use super::super::*;

        /// 3.3-UNIT-007: `property_name_validates_regex_and_length`.
        /// Priority: P1.
        #[test]
        fn property_name_validates_regex_and_length() {
            // GIVEN various property name inputs
            // WHEN creating PropertyName instances
            // THEN it should accept valid names and reject invalid ones
            PropertyName::new("valid_name".into()).unwrap();
            PropertyName::new("valid-name-123".into()).unwrap();
            PropertyName::new(String::new()).unwrap_err();
            PropertyName::new("Invalid Name".into()).unwrap_err();
            PropertyName::new("a".repeat(65)).unwrap_err();
        }

        /// 3.3-UNIT-008: `property_name_validates_format`.
        /// Priority: P1.
        #[test]
        fn property_name_validates_format() {
            // GIVEN invalid and valid name formats
            // WHEN creating PropertyName instances
            // THEN it should reject invalid characters
            PropertyName::new("invalid_name!".into()).unwrap_err();
            // AND accept valid snake/kebab case with underscores
            PropertyName::new("valid_name".into()).unwrap();
        }

        /// 3.3-UNIT-009: `property_name_validates_length`.
        /// Priority: P2.
        #[test]
        fn property_name_validates_length() {
            // GIVEN a name exceeding the 64 character limit
            let long_name = "a".repeat(65);

            // WHEN creating a PropertyName
            let res = PropertyName::new(long_name);

            // THEN it should return a PropertyNameTooLong error
            assert!(matches!(res, Err(DomainError::PropertyNameTooLong(_))));
        }

        /// 3.3-UNIT-010: `property_name_validates_non_empty`.
        /// Priority: P2.
        #[test]
        fn property_name_validates_non_empty() {
            // GIVEN an empty name string
            // WHEN creating a PropertyName
            let res = PropertyName::new(String::new());

            // THEN it should return an EmptyPropertyName error
            assert!(matches!(res, Err(DomainError::EmptyPropertyName)));
        }
    }

    mod proptests {
        use proptest::prelude::*;

        use super::super::*;
        use crate::models::property_spec::BoolSpec;

        proptest! {
            /// 3.3-UNIT-015: `validates_property_name_format_proptest`.
            /// Priority: P2.
            #[test]
            fn validates_property_name_format_proptest(name in "[a-z0-9_-]{1,64}") {
                // GIVEN an arbitrary valid property name
                // WHEN creating a PropertyName
                // THEN it must succeed
                PropertyName::new(name).unwrap();
            }

            /// 3.3-UNIT-016: `rejects_invalid_property_name_characters_proptest`.
            /// Priority: P2.
            #[test]
            fn rejects_invalid_property_name_characters_proptest(name in ".*[^a-z0-9_-].*") {
                // GIVEN an arbitrary string containing invalid characters
                // WHEN creating a PropertyName (filtering for correct length)
                if !name.is_empty() && name.len() <= 64 {
                    // THEN it must fail
                    PropertyName::new(name).unwrap_err();
                }
            }

            /// 3.3-UNIT-017: `compute_id_is_deterministic_proptest`.
            /// Priority: P1.
            #[test]
            fn compute_id_is_deterministic_proptest(name in "[a-z0-9_-]{1,64}") {
                // GIVEN an arbitrary name and a fixed spec
                let spec = PropertySpec::Bool(BoolSpec::default());
                // WHEN computing IDs multiple times
                let id1 = Property::compute_id(&name, &spec).unwrap();
                let id2 = Property::compute_id(&name, &spec).unwrap();
                // THEN results must be identical
                assert_eq!(id1, id2);
            }
        }
    }
}
