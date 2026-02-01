//! Property domain entities and value objects.

#![allow(
    clippy::module_name_repetitions,
    reason = "Core domain logic and naming convention where Property prefix \
              is descriptive"
)]

use std::{
    fmt::{Debug, Display},
    sync::LazyLock,
};

use regex::Regex;
use uuid::Uuid;

use super::{error::SchemaError, property_spec::PropertySpec};
use crate::patterns;

/// Validated property name value object.
///
/// Enforces invariants:
/// - Non-empty
/// - Max 64 characters
/// - Matches regex `^[a-zA-Z0-9_-]+$` (alphanumeric, underscores, dashes)
///
/// # Examples
/// ```
/// # use lithos_core::schema::PropertyName;
/// let name = PropertyName::new("status".to_string()).unwrap();
/// assert_eq!(&name.0, "status");
/// assert!(PropertyName::new("".to_string()).is_err());
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "String", into = "String")]
#[non_exhaustive]
pub struct PropertyName(pub String);

impl PropertyName {
    /// Create a new `PropertyName` with validation.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn new(name: String) -> Result<Self, SchemaError> {
        Self::validate(&name)?;
        Ok(Self(name))
    }

    /// Validates a property name string.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn validate(name: &str) -> Result<(), SchemaError> {
        static RE: LazyLock<Regex> = LazyLock::new(|| {
            #[expect(
                clippy::expect_used,
                clippy::disallowed_methods,
                reason = "Static regex literal is safe and efficient"
            )]
            Regex::new(patterns::ALPHANUMERIC_NAME)
                .expect("Static regex literal")
        });

        if name.is_empty() {
            return Err(SchemaError::EmptyPropertyName);
        }
        if name.len() > 64 {
            return Err(SchemaError::PropertyNameTooLong(name.len()));
        }

        if !RE.is_match(name) {
            return Err(SchemaError::InvalidPropertyName(name.to_owned()));
        }
        Ok(())
    }
}

impl std::ops::Deref for PropertyName {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for PropertyName {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for PropertyName {
    type Error = SchemaError;

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
    /// Unique identity (UUID v7).
    id: Uuid,
    /// Property name.
    name: PropertyName,
    /// Whether property is required.
    required: bool,
    /// Whether property accepts array of values.
    array: bool,
    /// Type-specific validation specification.
    spec: PropertySpec,
}

impl Property {
    /// Returns true if this property accepts an array of values.
    #[inline]
    #[must_use]
    pub const fn array(&self) -> bool {
        self.array
    }

    /// Returns the property's unique identifier.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Returns true if this property is required and not an array.
    #[inline]
    #[must_use]
    pub const fn is_required_scalar(&self) -> bool {
        self.required && !self.array
    }

    /// Returns the property's name.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &PropertyName {
        &self.name
    }

    /// Create a new property with validation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::{Property, PropertyName};
    /// # use lithos_core::schema::{PropertySpec, BoolSpec};
    /// # use uuid::Uuid;
    /// let name = PropertyName::new("is_active".to_string()).unwrap();
    /// let spec = PropertySpec::Bool(BoolSpec::default());
    /// let id = Uuid::now_v7();
    ///
    /// let property = Property::new(id, name, true, false, spec).unwrap();
    /// assert!(property.required());
    /// ```
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn new(
        id: Uuid,
        name: PropertyName,
        required: bool,
        array: bool,
        spec: PropertySpec,
    ) -> Result<Self, SchemaError> {
        let property = Self {
            id,
            name,
            required,
            array,
            spec,
        };
        property.validate()?;
        Ok(property)
    }

    /// Returns true if this property is required.
    #[inline]
    #[must_use]
    pub const fn required(&self) -> bool {
        self.required
    }

    /// Returns the type-specific validation specification.
    #[inline]
    #[must_use]
    pub const fn spec(&self) -> &PropertySpec {
        &self.spec
    }

    /// Validate property structure and constraints.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn validate(&self) -> Result<(), SchemaError> {
        // Name validation is handled by PropertyName type.
        // We only validate the spec constraints here.
        self.spec.validate_spec()?;
        Ok(())
    }

    /// Validate a value against this property's specification.
    ///
    /// This method uses `serde_json::Value` as a universal Intermediate
    /// Representation (IR) for metadata values, allowing validation of data
    /// loaded from JSON, YAML, or TOML.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    ///
    /// # Examples
    /// ```
    /// # use lithos_core::schema::{Property, PropertyName, PropertySpec, BoolSpec};
    /// # use uuid::Uuid;
    /// let name = PropertyName::new("enabled".to_string()).unwrap();
    /// let spec = PropertySpec::Bool(BoolSpec::default());
    /// let property = Property::new(Uuid::now_v7(), name, true, false, spec).unwrap();
    /// property.validate_value(&serde_json::json!(true)).unwrap();
    /// ```
    #[inline]
    pub fn validate_value(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), SchemaError> {
        if self.array {
            let arr =
                value.as_array().ok_or_else(|| SchemaError::InvalidType {
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

#[cfg(test)]
#[expect(dead_code, reason = "Test fixtures may be used by other crates")]
/// Test fixtures and builders for `Property`.
pub mod fixtures {
    use uuid::Uuid;

    use super::{super::property_spec::StringSpec, *};

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
            reason = "Test builder uses Result::expect() for creating \
                      properties from hardcoded test data. Failures here \
                      indicate logic errors in test setup."
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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn builder_sets_fields() {
            let property = PropertyBuilder::new()
                .name("priority")
                .required(true)
                .array(true)
                .spec(PropertySpec::String(StringSpec::default()))
                .build();

            assert_eq!(&property.name().0, "priority");
            assert!(property.required());
            assert!(property.array());
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Test module uses Result::expect() for ergonomic arrangement and \
              assertions. Acceptable in test-only code paths."
)]
mod tests {
    mod property {
        use super::super::{super::property_spec::StringSpec, *};

        #[test]
        fn accessors_return_expected_values() {
            // GIVEN: a property aggregate
            let spec = PropertySpec::String(StringSpec::default());
            let property = Property::new(
                Uuid::now_v7(),
                PropertyName::new("status".to_owned()).unwrap(),
                true,
                false,
                spec,
            )
            .unwrap();

            // THEN: accessors expose fields correctly
            assert!(property.required());
            assert!(property.is_required_scalar());
            assert!(!property.array());
            assert_eq!(&property.name().0, "status");
        }

        /// 3.3-UNIT-006: `returns_error_when_property_name_format_is_invalid`.
        /// Priority: P1.
        #[test]
        fn returns_error_when_property_name_format_is_invalid() {
            // GIVEN: a property specification
            let spec = PropertySpec::String(StringSpec::default());
            // AND: a set of invalid names
            let invalid_names = vec!["Invalid Name", "invalid.name", ""];

            // WHEN: validating property names
            for name_str in invalid_names {
                // THEN: it should reject invalid names
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
            // GIVEN: various property name inputs
            // WHEN: creating PropertyName instances
            // THEN: it should accept valid names and reject invalid ones
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
            // GIVEN: invalid and valid name formats
            // WHEN: creating PropertyName instances
            // THEN: it should reject invalid characters
            PropertyName::new("invalid_name!".into()).unwrap_err();
            // AND: accept valid snake/kebab case with underscores
            PropertyName::new("valid_name".into()).unwrap();
        }

        /// 3.3-UNIT-009: `property_name_validates_length`.
        /// Priority: P2.
        #[test]
        fn property_name_validates_length() {
            // GIVEN: a name exceeding the 64 character limit
            let long_name = "a".repeat(65);

            // WHEN: creating a PropertyName
            let res = PropertyName::new(long_name);

            // THEN: it should return a PropertyNameTooLong error
            assert!(matches!(res, Err(SchemaError::PropertyNameTooLong(_))));
        }

        /// 3.3-UNIT-010: `property_name_validates_non_empty`.
        /// Priority: P2.
        #[test]
        fn property_name_validates_non_empty() {
            // GIVEN: an empty name string
            // WHEN: creating a PropertyName
            let res = PropertyName::new(String::new());

            // THEN: it should return an EmptyPropertyName error
            assert!(matches!(res, Err(SchemaError::EmptyPropertyName)));
        }
    }

    mod proptests {
        use lithos_test_utils::data::properties::{
            invalid_identifier, valid_identifier,
        };
        use proptest::prelude::*;

        use super::super::*;

        proptest! {
            /// 3.3-UNIT-015: `validates_property_name_format_proptest`.
            /// Priority: P2.
            #[test]
            fn validates_property_name_format_proptest(name in valid_identifier()) {
                // GIVEN an arbitrary valid property name
                // WHEN creating a PropertyName
                // THEN it must succeed
                PropertyName::new(name).unwrap();
            }

            /// 3.3-UNIT-016: `rejects_invalid_property_name_characters_proptest`.
            /// Priority: P2.
            #[test]
            fn rejects_invalid_property_name_characters_proptest(name in invalid_identifier()) {
                // GIVEN an arbitrary string containing invalid characters
                // WHEN creating a PropertyName (filtering for correct length)
                // THEN it must fail
                PropertyName::new(name).unwrap_err();
            }
        }
    }
}
