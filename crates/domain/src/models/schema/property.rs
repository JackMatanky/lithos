//! Property domain entities and value objects.

#![allow(
    clippy::module_name_repetitions,
    reason = "Core domain logic and naming convention where Property prefix is descriptive"
)]

use std::{
    fmt::{Debug, Display},
    sync::OnceLock,
};

use uuid::Uuid;

use crate::{errors::DomainError, models::schema::property_spec::PropertySpec};

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
        static NAME_RE: OnceLock<regex::Regex> = OnceLock::new();
        #[expect(
            clippy::expect_used,
            clippy::disallowed_methods,
            reason = "Standard pattern for hardcoded regexes - Regex is known valid"
        )]
        let re = NAME_RE.get_or_init(|| {
            regex::Regex::new("^[a-z0-9_-]+$")
                .expect("Hardcoded regex is valid")
        });
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
    /// Unique identity (UUID v7).
    pub id: Uuid,
    /// Property name.
    pub name: PropertyName,
    /// Whether property is required.
    pub required: bool,
    /// Type-specific validation specification.
    pub spec: PropertySpec,
}

impl Property {
    /// Create a new property with validation.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_domain::models::schema::{Property, PropertyName};
    /// use lithos_domain::models::schema::{PropertySpec, BoolSpec};
    /// use uuid::Uuid;
    ///
    /// let name = PropertyName::new("is_active".to_string()).unwrap();
    /// let spec = PropertySpec::Bool(BoolSpec::default());
    /// let id = Uuid::now_v7();
    ///
    /// let property = Property::new(id, name, true, false, spec).unwrap();
    /// assert!(property.required);
    /// ```
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    #[inline]
    pub fn new(
        id: Uuid,
        name: PropertyName,
        required: bool,
        array: bool,
        spec: PropertySpec,
    ) -> Result<Self, DomainError> {
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

    /// Validate a value against this property's specification.
    ///
    /// This method uses `serde_json::Value` as a universal Intermediate Representation (IR)
    /// for metadata values, allowing validation of data loaded from JSON, YAML, or TOML.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    #[inline]
    pub fn validate_value(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), DomainError> {
        if self.array {
            let arr =
                value.as_array().ok_or_else(|| DomainError::InvalidType {
                    value: value.to_string(),
                    expected: "array".to_owned(),
                })?;
            for item in arr {
                self.spec.validate(item)?;
            }
            Ok(())
        } else {
            self.spec.validate(value)
        }
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
    ///
    /// This variant handles inline definitions in schema files.
    Inline(RawPropertyInline),
    /// A reference to a property in the `PropertyBank`.
    ///
    /// This variant handles `$ref` pointers in schema files.
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
/// Missing identity is assigned by the adapter before entering the Domain resolution.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyInline {
    /// Whether property accepts array of values.
    #[serde(default)]
    pub array: bool,
    /// Unique identity assigned by adapter.
    pub id: Uuid,
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
    use crate::models::schema::property_spec::StringSpec;

    /// `PropertyBuilder` for flexible test data generation.
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
            Property::new(
                Uuid::now_v7(),
                name,
                self.required,
                self.array,
                self.spec,
            )
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
        use crate::models::schema::property_spec::StringSpec;

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
        }
    }
}
