//! Property domain entities and value objects.

#![allow(
    clippy::module_name_repetitions,
    clippy::exhaustive_structs,
    reason = "Core domain logic. rkyv generates exhaustive Archived types \
              despite #[non_exhaustive]. Property prefix is descriptive"
)]
#![expect(
    clippy::exhaustive_enums,
    reason = "rkyv generates exhaustive archived enums despite \
              #[non_exhaustive] on the source enums."
)]

use std::{
    borrow::Borrow,
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
    id: PropertyId,
    /// Property name.
    name: PropertyName,
    /// Whether property is required.
    cardinality: Cardinality,
    /// Whether property accepts array of values.
    multiplicity: Multiplicity,
    /// Type-specific validation specification.
    spec: PropertySpec,
}

impl Property {
    /// Returns the property's unique identifier.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> PropertyId {
        self.id
    }

    /// Returns the property's cardinality.
    #[inline]
    #[must_use]
    pub const fn cardinality(&self) -> Cardinality {
        self.cardinality
    }

    /// Returns the property's multiplicity.
    #[inline]
    #[must_use]
    pub const fn multiplicity(&self) -> Multiplicity {
        self.multiplicity
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
    /// # use lithos_core::schema::property::{
    /// #     Cardinality, Multiplicity, Property, PropertyId, PropertyName,
    /// # };
    /// # use lithos_core::schema::property_spec::{PropertySpec, BoolSpec};
    /// # use uuid::Uuid;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = PropertyName::new("is_active")?;
    /// let spec = PropertySpec::Bool(BoolSpec::default());
    /// let id = PropertyId::new();
    ///
    /// let property = Property::new(
    ///     id,
    ///     name,
    ///     Cardinality::Required,
    ///     Multiplicity::Single,
    ///     spec,
    /// )?;
    /// assert!(property.is_required_scalar(), "Property should be required");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn new(
        id: PropertyId,
        name: PropertyName,
        cardinality: Cardinality,
        multiplicity: Multiplicity,
        spec: PropertySpec,
    ) -> Result<Self, SchemaError> {
        let property = Self {
            id,
            name,
            cardinality,
            multiplicity,
            spec,
        };
        property.validate()?;
        Ok(property)
    }

    /// Returns true if this property is required.
    #[inline]
    #[must_use]
    pub fn is_required_scalar(&self) -> bool {
        self.cardinality == Cardinality::Required
            && self.multiplicity == Multiplicity::Single
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
    /// # use lithos_core::schema::property::{
    /// #     Cardinality, Multiplicity, Property, PropertyId, PropertyName,
    /// # };
    /// # use lithos_core::schema::property_spec::{PropertySpec, BoolSpec};
    /// # use uuid::Uuid;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = PropertyName::new("enabled")?;
    /// let spec = PropertySpec::Bool(BoolSpec::default());
    /// let property = Property::new(
    ///     PropertyId::new(),
    ///     name,
    ///     Cardinality::Required,
    ///     Multiplicity::Single,
    ///     spec,
    /// )?;
    /// property.validate_value(&serde_json::json!(true))?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn validate_value(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), SchemaError> {
        if self.multiplicity == Multiplicity::Many {
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

/// Whether a property is required or optional.
#[derive(
    Debug,
    Clone,
    Copy,
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
#[non_exhaustive]
pub enum Cardinality {
    /// Optional property.
    Optional,
    /// Required property.
    Required,
}

/// Whether a property accepts a single value or multiple values.
#[derive(
    Debug,
    Clone,
    Copy,
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
#[non_exhaustive]
pub enum Multiplicity {
    /// Single scalar value.
    Single,
    /// Multiple values (array).
    Many,
}

/// Unique identity for a property.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug, Hash, PartialEq, Eq))]
#[serde(transparent)]
#[non_exhaustive]
pub struct PropertyId(Uuid);

impl PropertyId {
    /// Wraps a UUID into a `PropertyId`.
    #[inline]
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the inner UUID reference.
    #[inline]
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Returns the inner UUID by value.
    #[inline]
    #[must_use]
    pub const fn into_uuid(self) -> Uuid {
        self.0
    }

    /// Creates a new UUID v7-based `PropertyId`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for PropertyId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Validated property name value object.
///
/// Enforces invariants:
/// - Non-empty
/// - Max 64 characters
/// - Matches regex `^[a-z0-9_-]+$` (lowercase alphanumeric, underscores,
///   dashes)
///
/// # Examples
/// ```
/// # use lithos_core::schema::property::PropertyName;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let name = PropertyName::new("status")?;
/// assert_eq!(name.as_str(), "status", "Name should match input");
/// assert!(PropertyName::new("").is_err(), "Empty name should be rejected");
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
#[rkyv(derive(Debug, Hash, PartialEq, Eq))]
#[serde(try_from = "String", into = "String")]
#[non_exhaustive]
pub struct PropertyName(Box<str>);

impl PropertyName {
    /// Create a new `PropertyName` with validation.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn new(name: &str) -> Result<Self, SchemaError> {
        Self::validate(name)?;
        Ok(Self(name.into()))
    }

    /// Returns the inner string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
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

impl AsRef<str> for PropertyName {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for PropertyName {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Borrow<str> for PropertyName {
    #[inline]
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<PropertyName> for String {
    #[inline]
    fn from(val: PropertyName) -> Self {
        val.0.into()
    }
}

impl TryFrom<&str> for PropertyName {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for PropertyName {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

/// Typed reference to a property definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PropertyRef {
    /// Reference by property id.
    ById(PropertyId),
    /// Reference by property name.
    ByName(PropertyName),
}

impl PropertyRef {
    /// Parse a reference string into a typed property reference.
    ///
    /// Accepted forms:
    /// - "#/properties/<name>" (by name)
    /// - "$bank:<uuid>" (by id)
    /// - "<name>" (by name)
    /// - "<uuid>" (by id)
    ///
    /// # Errors
    /// Returns `SchemaError` when the reference is not valid.
    #[inline]
    pub fn parse(reference: &str) -> Result<Self, SchemaError> {
        if let Some(name) = reference.strip_prefix("#/properties/") {
            return Ok(Self::ByName(PropertyName::try_from(name)?));
        }

        if let Some(id_str) = reference.strip_prefix("$bank:") {
            let id = Uuid::parse_str(id_str).map_err(|error| {
                SchemaError::ValidationFailed(format!(
                    "Invalid property id reference: {error}"
                ))
            })?;
            return Ok(Self::ById(PropertyId::from_uuid(id)));
        }

        if let Ok(id) = Uuid::parse_str(reference) {
            return Ok(Self::ById(PropertyId::from_uuid(id)));
        }

        Ok(Self::ByName(PropertyName::try_from(reference)?))
    }
}

impl TryFrom<&str> for PropertyRef {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
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
            cardinality: Cardinality,
            multiplicity: Multiplicity,
            name: String,
            spec: PropertySpec,
        }

        impl Default for PropertyBuilder {
            #[inline]
            fn default() -> Self {
                Self {
                    cardinality: Cardinality::Optional,
                    multiplicity: Multiplicity::Single,
                    name: "test_property".to_owned(),
                    spec: PropertySpec::String(StringSpec::default()),
                }
            }
        }

        impl PropertyBuilder {
            /// Sets whether the property is an array.
            #[inline]
            #[must_use]
            pub fn array(mut self, array: bool) -> Self {
                self.multiplicity = if array {
                    Multiplicity::Many
                } else {
                    Multiplicity::Single
                };
                self
            }

            /// Builds the `Property` entity.
            ///
            /// # Errors
            /// Returns `SchemaError` if the property configuration is invalid.
            #[inline]
            pub fn build(self) -> Result<Property, SchemaError> {
                let name = PropertyName::new(&self.name)?;
                Property::new(
                    PropertyId::from_uuid(TEST_PROPERTY_ID),
                    name,
                    self.cardinality,
                    self.multiplicity,
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
                self.cardinality = if required {
                    Cardinality::Required
                } else {
                    Cardinality::Optional
                };
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
                    property.name().as_str(),
                    "priority",
                    "Builder should set property name to 'priority'"
                );
            }

            #[test]
            fn builder_sets_required_flag() {
                let property = build_property();

                assert!(
                    property.cardinality() == Cardinality::Required,
                    "Builder should set required flag to true"
                );
            }

            #[test]
            fn builder_sets_array_flag() {
                let property = build_property();

                assert!(
                    property.multiplicity() == Multiplicity::Many,
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
            let name = PropertyName::new("status")
                .expect("Expected valid property name");

            Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID),
                name,
                Cardinality::Required,
                Multiplicity::Single,
                spec,
            )
            .expect("Expected valid property")
        }

        #[test]
        fn returns_required_flag_when_required_true() {
            let property = required_scalar_property();

            assert!(
                property.cardinality() == Cardinality::Required,
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
                property.multiplicity() == Multiplicity::Single,
                "Property should not be an array when array flag is false"
            );
        }

        #[test]
        fn returns_name_from_accessor() {
            let property = required_scalar_property();

            assert_eq!(
                property.name().as_str(),
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
                PropertyName::new(name).is_err(),
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
            let result = PropertyName::new(name);

            assert!(result.is_ok(), "Expected {name} to pass, got: {result:?}");
        }

        /// 3.3-UNIT-008: `property_name_validates_format`.
        /// Priority: P1.
        #[test]
        fn property_name_validates_format() {
            let invalid = PropertyName::new("invalid_name!");
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
            let res = PropertyName::new(&long_name);

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
            let res = PropertyName::new("");

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
            fn validates_property_name_format_proptest(name in "[a-z0-9_-]{1,64}") {
                // GIVEN an arbitrary valid property name
                // WHEN creating a PropertyName
                // THEN it must succeed
                prop_assert!(
                    PropertyName::new(&name).is_ok(),
                    "Expected valid name, got error"
                );
            }
        }

        // 3.3-UNIT-016: `rejects_invalid_property_name_characters_proptest`.
        // Priority: P2.
        proptest! {
            #[test]
            fn rejects_invalid_property_name_characters_proptest(name in ".*[^a-z0-9_-].*") {
                prop_assume!(!name.is_empty() && name.len() <= 64);

                // GIVEN an arbitrary string containing invalid characters
                // WHEN creating a PropertyName (filtering for correct length)
                // THEN it must fail
                prop_assert!(
                    PropertyName::new(&name).is_err(),
                    "Expected invalid name to be rejected"
                );
            }
        }
    }
}
