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

/// Reusable property definition with type-specific validation.
///
/// This is the resolved entity used in the Domain layer.
///
/// # Examples
/// ```
/// use lithos_core::schema::{
///     property::{
///         Multiplicity, Optionality, Property, PropertyId, PropertyName,
///     },
///     property_spec::{BoolSpec, PropertySpec},
/// };
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let name = PropertyName::try_new("is_active")?;
/// let spec = PropertySpec::Bool(BoolSpec::default());
/// let property = Property::new(
///     PropertyId::new(),
///     name,
///     Optionality::Required,
///     Multiplicity::Single,
///     spec,
/// );
/// assert!(property.is_required_scalar());
/// # Ok(())
/// # }
/// ```
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
    optionality: Optionality,
    /// Whether property accepts array of values.
    multiplicity: Multiplicity,
    /// Type-specific validation specification.
    spec: PropertySpec,
}

impl Property {
    /// Create a new property.
    ///
    /// All validation is done at the component level (`PropertyName`,
    /// `PropertySpec`), so this constructor is infallible.
    #[inline]
    #[must_use]
    pub const fn new(
        id: PropertyId,
        name: PropertyName,
        optionality: Optionality,
        multiplicity: Multiplicity,
        spec: PropertySpec,
    ) -> Self {
        Self {
            id,
            name,
            optionality,
            multiplicity,
            spec,
        }
    }

    /// Returns the property's unique identifier.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> PropertyId {
        self.id
    }

    /// Returns the property's name.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &PropertyName {
        &self.name
    }

    /// Returns the property's optionality.
    #[inline]
    #[must_use]
    pub const fn optionality(&self) -> Optionality {
        self.optionality
    }

    /// Returns the property's multiplicity.
    #[inline]
    #[must_use]
    pub const fn multiplicity(&self) -> Multiplicity {
        self.multiplicity
    }

    /// Returns the type-specific validation specification.
    #[inline]
    #[must_use]
    pub const fn spec(&self) -> &PropertySpec {
        &self.spec
    }

    /// Returns true if this property is required.
    #[inline]
    #[must_use]
    pub fn is_required_scalar(&self) -> bool {
        self.optionality == Optionality::Required
            && self.multiplicity == Multiplicity::Single
    }

    /// Validate a value against this property's specification.
    ///
    /// This method uses `serde_json::Value` as a universal Intermediate
    /// Representation (IR) for metadata values, allowing validation of data
    /// loaded from JSON, YAML, or TOML.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn validate_value(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), SchemaError> {
        if self.multiplicity == Multiplicity::Many {
            let arr =
                value.as_array().ok_or_else(|| SchemaError::InvalidType {
                    value: value.to_string(),
                    expected: "array".into(),
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

/// Unique identity for a property.
///
/// # Examples
/// ```
/// use lithos_core::schema::property::PropertyId;
///
/// let id = PropertyId::new();
/// let _ = id.as_uuid();
/// ```
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord))]
#[serde(transparent)]
#[non_exhaustive]
pub struct PropertyId(Uuid);

impl PropertyId {
    /// Creates a new UUID v7-based `PropertyId`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

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
}

impl Default for PropertyId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Display for PropertyId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Validated property name value object.
///
/// Enforces invariants:
/// - Non-empty
/// - Max 64 characters
/// - Matches regex `^[A-Za-z_][A-Za-z0-9_-]*$` (must start with letter or
///   underscore, may contain letters, digits, underscores, hyphens)
///
/// # Examples
/// ```
/// # use lithos_core::schema::property::PropertyName;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let name = PropertyName::try_new("status")?;
/// assert_eq!(name.as_str(), "status", "Name should match input");
/// assert!(
///     PropertyName::try_new("").is_err(),
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
    PartialOrd,
    Ord,
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
    const MAX_LEN: usize = 64;
    /// Property name validation pattern: mixed-case letters, underscores, and
    /// hyphens.
    ///
    /// Pattern: `^[A-Za-z_][A-Za-z0-9_-]*$`.
    ///
    /// Must start with a letter (uppercase or lowercase) or underscore.
    /// May contain letters, digits, underscores, and hyphens.
    ///
    /// # Examples
    /// - Valid: `status`, `MyProperty`, `_internal`, `tag-name`, `Priority1`
    /// - Invalid: `123prop`, `-prop`, `prop!`, `my prop`
    const PATTERN: &'static str = "^[A-Za-z_][A-Za-z0-9_-]*$";

    /// Create a new `PropertyName` with validation.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::property::PropertyName;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = PropertyName::try_new("status")?;
    /// assert_eq!(name.as_str(), "status");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn try_new(name: &str) -> Result<Self, SchemaError> {
        Self::validate(name)?;
        Ok(Self(name.into()))
    }

    #[inline]
    fn validate(name: &str) -> Result<(), SchemaError> {
        static RE: LazyLock<Result<Regex, regex::Error>> =
            LazyLock::new(|| Regex::new(PropertyName::PATTERN));

        if name.is_empty() {
            return Err(SchemaError::EmptyPropertyName);
        }
        if name.len() > Self::MAX_LEN {
            return Err(SchemaError::PropertyNameTooLong(name.len()));
        }

        let re = RE.as_ref().map_err(|error| {
            SchemaError::ValidationFailed(format!(
                "Invalid property name regex: {error}"
            ))
        })?;

        if !re.is_match(name) {
            return Err(SchemaError::InvalidPropertyName(name.into()));
        }
        Ok(())
    }

    /// Returns the inner string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
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
        Self::try_new(value)
    }
}

impl TryFrom<String> for PropertyName {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.into_boxed_str())
    }
}

impl TryFrom<Box<str>> for PropertyName {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
        Self::validate(&value)?;
        Ok(Self(value))
    }
}

/// Typed reference to a property definition in the property bank.
///
/// The only valid format is `property_bank#/<name>` where `<name>` is a
/// valid property name. This format is defined by the vault schema format.
///
/// # Examples
/// ```
/// use lithos_core::schema::property::BankPropertyRef;
///
/// let reference = BankPropertyRef::parse("property_bank#/flag")?;
/// assert_eq!(reference.name().as_str(), "flag");
/// # Ok::<_, lithos_core::schema::error::SchemaError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct BankPropertyRef(PropertyName);

impl BankPropertyRef {
    const PREFIX: &'static str = "property_bank#/";

    /// Parse a reference string into a typed property reference.
    ///
    /// The only accepted format is `property_bank#/<name>`.
    ///
    /// # Errors
    /// Returns `SchemaError::InvalidPropertyRef` if the format is invalid.
    #[inline]
    pub fn parse(reference: &str) -> Result<Self, SchemaError> {
        let name = reference
            .strip_prefix(Self::PREFIX)
            .ok_or_else(|| SchemaError::InvalidPropertyRef(reference.into()))?;
        Ok(Self(PropertyName::try_from(name)?))
    }

    /// Returns the property name being referenced.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &PropertyName {
        &self.0
    }
}

impl TryFrom<&str> for BankPropertyRef {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

/// Whether a property is required or optional.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
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
pub enum Optionality {
    /// Optional property.
    #[default]
    Optional,
    /// Required property.
    Required,
}

impl From<bool> for Optionality {
    #[inline]
    fn from(required: bool) -> Self {
        if required {
            Self::Required
        } else {
            Self::Optional
        }
    }
}

/// Whether a property accepts a single value or multiple values.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
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
    #[default]
    Single,
    /// Multiple values (array).
    Many,
}

impl From<bool> for Multiplicity {
    #[inline]
    fn from(multi: bool) -> Self {
        if multi {
            Self::Many
        } else {
            Self::Single
        }
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
            optionality: Optionality,
            multiplicity: Multiplicity,
            name: String,
            spec: PropertySpec,
        }

        impl Default for PropertyBuilder {
            #[inline]
            fn default() -> Self {
                Self {
                    optionality: Optionality::default(),
                    multiplicity: Multiplicity::default(),
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
            /// Returns `SchemaError` if the property name is invalid.
            #[inline]
            pub fn build(self) -> Result<Property, SchemaError> {
                let name = PropertyName::try_new(&self.name)?;
                Ok(Property::new(
                    PropertyId::from_uuid(TEST_PROPERTY_ID),
                    name,
                    self.optionality,
                    self.multiplicity,
                    self.spec,
                ))
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

            /// Sets the property to required.
            #[inline]
            #[must_use]
            pub fn required(mut self) -> Self {
                self.optionality = Optionality::Required;
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

            fn build_property() -> Property {
                PropertyBuilder::new()
                    .name("priority")
                    .required()
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
                    property.optionality() == Optionality::Required,
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

        fn required_scalar_property() -> Property {
            let spec = PropertySpec::String(StringSpec::default());
            let name = PropertyName::try_new("status")
                .expect("Expected valid property name");

            Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID),
                name,
                Optionality::Required,
                Multiplicity::Single,
                spec,
            )
        }

        #[test]
        fn returns_required_flag_when_required_true() {
            let property = required_scalar_property();

            assert!(
                property.optionality() == Optionality::Required,
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
        #[case("123prop")]
        #[case("-prop")]
        #[case("my prop")]
        #[case("prop!")]
        fn returns_error_when_property_name_format_is_invalid(
            #[case] name: &str,
        ) {
            assert!(
                PropertyName::try_new(name).is_err(),
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
        #[case("MyProperty")]
        #[case("_internal")]
        #[case("Status")]
        #[case("tag-name")]
        #[case("Priority1")]
        fn property_name_validates_regex_and_length(#[case] name: &str) {
            let result = PropertyName::try_new(name);

            assert!(result.is_ok(), "Expected {name} to pass, got: {result:?}");
        }

        /// 3.3-UNIT-008: `property_name_validates_format`.
        /// Priority: P1.
        #[test]
        fn property_name_validates_format() {
            let invalid = PropertyName::try_new("invalid_name!");
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
            let res = PropertyName::try_new(&long_name);

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
            let res = PropertyName::try_new("");

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
            fn validates_property_name_format_proptest(name in "[A-Za-z_][A-Za-z0-9_-]{0,63}") {
                // GIVEN an arbitrary valid property name
                // WHEN creating a PropertyName
                // THEN it must succeed
                prop_assert!(
                    PropertyName::try_new(&name).is_ok(),
                    "Expected valid name, got error"
                );
            }
        }

        // 3.3-UNIT-016: `rejects_invalid_property_name_characters_proptest`.
        // Priority: P2.
        proptest! {
            #[test]
            fn rejects_invalid_property_name_characters_proptest(name in ".*[^A-Za-z0-9_-].*") {
                prop_assume!(!name.is_empty() && name.len() <= 64);

                // GIVEN an arbitrary string containing invalid characters
                // WHEN creating a PropertyName (filtering for correct length)
                // THEN it must fail
                prop_assert!(
                    PropertyName::try_new(&name).is_err(),
                    "Expected invalid name to be rejected"
                );
            }
        }
    }
}
