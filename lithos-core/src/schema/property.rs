//! Property domain entities and value objects.

#![allow(
    clippy::module_name_repetitions,
    clippy::exhaustive_structs,
    reason = "Core domain logic. rkyv generates exhaustive Archived types \
              despite #[non_exhaustive]. Property prefix is descriptive"
)]

use std::{
    fmt::{Debug, Display},
    sync::LazyLock,
};

use regex::Regex;
use uuid::Uuid;

use super::{error::SchemaError, property_spec::PropertySpec};
use crate::patterns;

/// Reusable property definition with type-specific validation.
///
/// This is the resolved entity used in the Domain layer.
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
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

/// Validated property name value object.
///
/// Enforces invariants:
/// - Non-empty
/// - Max 64 characters
/// - Matches regex `^[a-zA-Z0-9_-]+$` (alphanumeric, underscores, dashes)
///
/// # Examples
/// ```
/// # use lithos_core::schema::property::PropertyName;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let name = PropertyName::new("status".to_string())?;
/// assert_eq!(&name.0, "status", "Name should match input");
/// assert!(
///     PropertyName::new("".to_string()).is_err(),
///     "Empty name should be rejected"
/// );
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[serde(try_from = "String", into = "String")]
#[non_exhaustive]
pub struct PropertyName(pub String);

impl AsRef<str> for PropertyName {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
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

impl From<PropertyName> for String {
    #[inline]
    fn from(val: PropertyName) -> Self {
        val.0
    }
}

impl TryFrom<String> for PropertyName {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
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
    /// # use lithos_core::schema::property::{Property, PropertyName};
    /// # use lithos_core::schema::property_spec::{PropertySpec, BoolSpec};
    /// # use uuid::Uuid;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = PropertyName::new("is_active".to_string())?;
    /// let spec = PropertySpec::Bool(BoolSpec::default());
    /// let id = Uuid::now_v7();
    ///
    /// let property = Property::new(id, name, true, false, spec)?;
    /// assert!(property.required(), "Property should be required");
    /// # Ok(())
    /// # }
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
    /// # use lithos_core::schema::property::{Property, PropertyName};
    /// # use lithos_core::schema::property_spec::{PropertySpec, BoolSpec};
    /// # use uuid::Uuid;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = PropertyName::new("enabled".to_string())?;
    /// let spec = PropertySpec::Bool(BoolSpec::default());
    /// let property = Property::new(Uuid::now_v7(), name, true, false, spec)?;
    /// property.validate_value(&serde_json::json!(true))?;
    /// # Ok(())
    /// # }
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
        static RE: LazyLock<Result<Regex, regex::Error>> =
            LazyLock::new(|| Regex::new(patterns::ALPHANUMERIC_NAME));

        if name.is_empty() {
            return Err(SchemaError::EmptyPropertyName);
        }
        if name.len() > 64 {
            return Err(SchemaError::PropertyNameTooLong(name.len()));
        }

        let re = RE.as_ref().map_err(|error| {
            SchemaError::ValidationFailed(format!(
                "Invalid property name regex: {error}"
            ))
        })?;

        if !re.is_match(name) {
            return Err(SchemaError::InvalidPropertyName(name.to_owned()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    /// Test fixtures and builders for `Property`.
    mod fixtures {
        use uuid::Uuid;

        use super::super::{super::property_spec::StringSpec, *};

        const TEST_PROPERTY_ID: Uuid =
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0801);

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
            /// # Errors
            /// Returns `SchemaError` if the property configuration is invalid.
            #[inline]
            pub fn build(self) -> Result<Property, SchemaError> {
                let name = PropertyName::new(self.name)?;
                Property::new(
                    TEST_PROPERTY_ID,
                    name,
                    self.required,
                    self.array,
                    self.spec,
                )
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

        #[cfg(test)]
        mod tests {
            use super::*;

            #[expect(
                clippy::disallowed_methods,
                reason = "Test fixture uses expect for deterministic setup. \
                          Failure indicates invalid test data. Expect is \
                          idiomatic in setup."
            )]
            fn build_property() -> Property {
                PropertyBuilder::new()
                    .name("priority")
                    .required(true)
                    .array(true)
                    .spec(PropertySpec::String(StringSpec::default()))
                    .build()
                    .expect("Expected builder to produce a valid Property")
            }

            #[test]
            fn builder_sets_name() {
                let property = build_property();

                assert_eq!(
                    &property.name().0,
                    "priority",
                    "Builder should set property name to 'priority'"
                );
            }

            #[test]
            fn builder_sets_required_flag() {
                let property = build_property();

                assert!(
                    property.required(),
                    "Builder should set required flag to true"
                );
            }

            #[test]
            fn builder_sets_array_flag() {
                let property = build_property();

                assert!(
                    property.array(),
                    "Builder should set array flag to true"
                );
            }
        }
    }

    mod property {
        use rstest::rstest;
        use uuid::Uuid;

        use super::super::{super::property_spec::StringSpec, *};

        const TEST_PROPERTY_ID: Uuid =
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0802);

        #[expect(
            clippy::disallowed_methods,
            reason = "Test fixture uses expect for deterministic setup. \
                      Failure indicates invalid test data. Expect is \
                      idiomatic in setup."
        )]
        fn required_scalar_property() -> Property {
            let spec = PropertySpec::String(StringSpec::default());
            let name = PropertyName::new("status".to_owned())
                .expect("Expected valid property name");

            Property::new(TEST_PROPERTY_ID, name, true, false, spec)
                .expect("Expected valid property")
        }

        #[test]
        fn returns_required_flag_when_required_true() {
            let property = required_scalar_property();

            assert!(
                property.required(),
                "Property should be required when required flag is true"
            );
        }

        #[test]
        fn returns_required_scalar_when_required_and_not_array() {
            let property = required_scalar_property();

            assert!(
                property.is_required_scalar(),
                "Property should be a required scalar (not array)"
            );
        }

        #[test]
        fn returns_array_flag_false_when_not_array() {
            let property = required_scalar_property();

            assert!(
                !property.array(),
                "Property should not be an array when array flag is false"
            );
        }

        #[test]
        fn returns_name_from_accessor() {
            let property = required_scalar_property();

            assert_eq!(
                &property.name().0,
                "status",
                "Property name should match"
            );
        }

        /// 3.3-UNIT-006: `returns_error_when_property_name_format_is_invalid`.
        /// Priority: P1.
        #[rstest]
        #[case("Invalid Name")]
        #[case("invalid.name")]
        #[case("")]
        fn returns_error_when_property_name_format_is_invalid(
            #[case] name: &str,
        ) {
            assert!(
                PropertyName::new(name.to_owned()).is_err(),
                "Should reject invalid name: {name}"
            );
        }
    }

    mod property_name {
        use rstest::rstest;

        use super::super::*;

        /// 3.3-UNIT-007: `property_name_validates_regex_and_length`.
        /// Priority: P1.
        #[rstest]
        #[case("valid_name")]
        #[case("valid-name-123")]
        fn property_name_validates_regex_and_length(#[case] name: &str) {
            let result = PropertyName::new(name.to_owned());

            assert!(result.is_ok(), "Expected {name} to pass, got: {result:?}");
        }

        /// 3.3-UNIT-008: `property_name_validates_format`.
        /// Priority: P1.
        #[test]
        fn property_name_validates_format() {
            let invalid = PropertyName::new("invalid_name!".into());
            assert!(
                invalid.is_err(),
                "Expected invalid_name! to fail, got: {invalid:?}"
            );
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
            assert!(
                matches!(res, Err(SchemaError::PropertyNameTooLong(_))),
                "Property name >64 chars should be rejected, got: {res:?}"
            );
        }

        /// 3.3-UNIT-010: `property_name_validates_non_empty`.
        /// Priority: P2.
        #[test]
        fn property_name_validates_non_empty() {
            // GIVEN: an empty name string
            // WHEN: creating a PropertyName
            let res = PropertyName::new(String::new());

            // THEN: it should return an EmptyPropertyName error
            assert!(
                matches!(res, Err(SchemaError::EmptyPropertyName)),
                "Empty property name should be rejected, got: {res:?}"
            );
        }
    }

    mod proptests {
        use proptest::prelude::*;

        use super::super::*;

        // 3.3-UNIT-015: `validates_property_name_format_proptest`.
        // Priority: P2.
        proptest! {
            #[test]
            fn validates_property_name_format_proptest(name in "[a-zA-Z0-9_-]{1,64}") {
                // GIVEN an arbitrary valid property name
                // WHEN creating a PropertyName
                // THEN it must succeed
                prop_assert!(PropertyName::new(name).is_ok());
            }
        }

        // 3.3-UNIT-016: `rejects_invalid_property_name_characters_proptest`.
        // Priority: P2.
        proptest! {
            #[test]
            fn rejects_invalid_property_name_characters_proptest(name in ".*[^a-zA-Z0-9_-].*") {
                prop_assume!(!name.is_empty() && name.len() <= 64);

                // GIVEN an arbitrary string containing invalid characters
                // WHEN creating a PropertyName (filtering for correct length)
                // THEN it must fail
                prop_assert!(PropertyName::new(name).is_err());
            }
        }
    }
}
