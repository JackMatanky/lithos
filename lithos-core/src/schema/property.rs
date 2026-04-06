//! Property domain entities and value objects.

#![expect(
    clippy::module_name_repetitions,
    reason = "Property* types are descriptive and namespaced intentionally"
)]
#![expect(
    clippy::exhaustive_enums,
    reason = "rkyv Archive derive generates exhaustive archived enums"
)]

use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::{Debug, Display},
    sync::LazyLock,
};

use regex::Regex;
use rkyv::{Archive, Deserialize, Serialize};
use uuid::Uuid;

use super::{
    error::SchemaError,
    property_spec::PropertySpec,
    raw::property::{RawPropertyBankEntry, RawPropertyInline, RawPropertyMap},
};

/// Map of properties keyed by name.
///
/// This wrapper preserves the invariant that a property's name is stored only
/// in the map key, not inside the `Property` value.
#[derive(Debug, Clone, PartialEq, Default, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PropertyMap(HashMap<PropertyName, Property>);

impl PropertyMap {
    /// Creates an empty property map.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Returns true if the map is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of properties in the map.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns a property by name.
    #[inline]
    #[must_use]
    pub fn get(&self, name: &PropertyName) -> Option<&Property> {
        self.0.get(name)
    }

    /// Returns true if the map contains the given name.
    #[inline]
    #[must_use]
    pub fn has(&self, name: &PropertyName) -> bool {
        self.0.contains_key(name)
    }

    /// Inserts a property with the given name.
    #[inline]
    pub fn insert(
        &mut self,
        name: PropertyName,
        property: Property,
    ) -> Option<Property> {
        self.0.insert(name, property)
    }

    /// Removes a property by name.
    #[inline]
    pub fn remove(&mut self, name: &PropertyName) -> Option<Property> {
        self.0.remove(name)
    }

    /// Returns an iterator over property values.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &Property> {
        self.0.values()
    }

    /// Extends the map with entries from another `PropertyMap`.
    #[inline]
    pub fn extend(&mut self, other: PropertyMap) {
        for (name, property) in other {
            self.insert(name, property);
        }
    }

    /// Returns an iterator over property names.
    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &PropertyName> {
        self.0.keys()
    }

    /// Returns an iterator over named properties.
    #[inline]
    #[must_use]
    pub fn iter(
        &self,
    ) -> std::collections::hash_map::Iter<'_, PropertyName, Property> {
        self.0.iter()
    }
}

impl TryFrom<RawPropertyMap<RawPropertyInline>> for PropertyMap {
    type Error = SchemaError;

    #[inline]
    fn try_from(
        value: RawPropertyMap<RawPropertyInline>,
    ) -> Result<Self, Self::Error> {
        let mut map = PropertyMap::new();
        for (name, raw) in &value {
            let property = Property::new(
                PropertyId::new(),
                Optionality::from(raw.required),
                Multiplicity::from(raw.multi),
                raw.spec.clone().try_into()?,
            );
            map.insert(name.clone(), property);
        }
        Ok(map)
    }
}

impl TryFrom<RawPropertyMap<RawPropertyBankEntry>> for PropertyMap {
    type Error = SchemaError;

    #[inline]
    fn try_from(
        value: RawPropertyMap<RawPropertyBankEntry>,
    ) -> Result<Self, Self::Error> {
        let mut map = PropertyMap::new();
        for (name, raw) in &value {
            let property = Property::new(
                PropertyId::new(),
                Optionality::default(),
                Multiplicity::from(raw.multi),
                raw.spec.clone().try_into()?,
            );
            map.insert(name.clone(), property);
        }
        Ok(map)
    }
}

impl TryFrom<HashMap<PropertyName, RawPropertyInline>> for PropertyMap {
    type Error = SchemaError;

    #[inline]
    fn try_from(
        value: HashMap<PropertyName, RawPropertyInline>,
    ) -> Result<Self, Self::Error> {
        let mut map = PropertyMap::new();
        #[expect(
            clippy::iter_over_hash_type,
            reason = "Property map construction preserves key/value pairs"
        )]
        for (name, raw) in value {
            let property = Property::new(
                PropertyId::new(),
                Optionality::from(raw.required),
                Multiplicity::from(raw.multi),
                raw.spec.try_into()?,
            );
            map.insert(name, property);
        }
        Ok(map)
    }
}

impl TryFrom<HashMap<PropertyName, RawPropertyBankEntry>> for PropertyMap {
    type Error = SchemaError;

    #[inline]
    fn try_from(
        value: HashMap<PropertyName, RawPropertyBankEntry>,
    ) -> Result<Self, Self::Error> {
        let mut map = PropertyMap::new();
        #[expect(
            clippy::iter_over_hash_type,
            reason = "Property map construction preserves key/value pairs"
        )]
        for (name, raw) in value {
            let property = Property::new(
                PropertyId::new(),
                Optionality::default(),
                Multiplicity::from(raw.multi),
                raw.spec.try_into()?,
            );
            map.insert(name, property);
        }
        Ok(map)
    }
}

impl AsRef<HashMap<PropertyName, Property>> for PropertyMap {
    #[inline]
    fn as_ref(&self) -> &HashMap<PropertyName, Property> {
        &self.0
    }
}

impl From<HashMap<PropertyName, Property>> for PropertyMap {
    #[inline]
    fn from(map: HashMap<PropertyName, Property>) -> Self {
        Self(map)
    }
}

impl<'map> IntoIterator for &'map PropertyMap {
    type IntoIter =
        std::collections::hash_map::Iter<'map, PropertyName, Property>;
    type Item = (&'map PropertyName, &'map Property);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for PropertyMap {
    type IntoIter =
        std::collections::hash_map::IntoIter<PropertyName, Property>;
    type Item = (PropertyName, Property);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

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
/// let spec = PropertySpec::Bool(BoolSpec::default());
/// let property = Property::new(
///     PropertyId::new(),
///     Optionality::Required,
///     Multiplicity::Single,
///     spec,
/// );
/// assert!(property.is_required_scalar());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Property {
    /// Unique identity (UUID v7).
    id: PropertyId,
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
        optionality: Optionality,
        multiplicity: Multiplicity,
        spec: PropertySpec,
    ) -> Self {
        Self {
            id,
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
            let arr = value.as_array().ok_or_else(|| {
                SchemaError::PropertyValue(
                    super::error::PropertyValueError::InvalidType {
                        value: value.to_string().into(),
                        expected: "array".into(),
                    },
                )
            })?;
            for item in arr {
                self.spec.validate(item)?;
            }
            Ok(())
        } else {
            self.spec.validate(value)
        }
    }

    /// Returns a copy of this property with a new id.
    #[inline]
    #[must_use]
    pub fn with_id(self, id: PropertyId) -> Self {
        Self {
            id,
            ..self
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
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord))]
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
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord))]
#[non_exhaustive]
#[derive(serde::Serialize)]
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
        Self::try_new_with_context(
            name,
            super::error::PropertyNameContext::SchemaProperty,
        )
    }

    /// Create a new `PropertyName` with validation and context.
    ///
    /// # Errors
    /// Returns `SchemaError::Syntax` if the name is empty, too long, or fails
    /// the property name format validation for the provided context.
    #[inline]
    pub fn try_new_with_context(
        name: &str,
        context: super::error::PropertyNameContext,
    ) -> Result<Self, SchemaError> {
        Self::validate(name, context)?;
        Ok(Self(name.into()))
    }

    #[inline]
    fn validate(
        name: &str,
        context: super::error::PropertyNameContext,
    ) -> Result<(), SchemaError> {
        static RE: LazyLock<Result<Regex, regex::Error>> =
            LazyLock::new(|| Regex::new(PropertyName::PATTERN));

        if name.is_empty() {
            return Err(super::error::PropertyNameError::Empty {
                context,
            }
            .into());
        }
        if name.len() > Self::MAX_LEN {
            return Err(super::error::PropertyNameError::TooLong {
                len: name.len(),
                max: Self::MAX_LEN,
                context,
            }
            .into());
        }

        let re = RE.as_ref().map_err(|error| {
            super::error::PropertyNameError::InvalidRegex {
                reason: error.to_string().into(),
            }
        })?;

        if !re.is_match(name) {
            return Err(super::error::PropertyNameError::InvalidFormat {
                name: name.into(),
                context,
            }
            .into());
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
        Self::validate(
            &value,
            crate::schema::error::PropertyNameContext::SchemaProperty,
        )?;
        Ok(Self(value))
    }
}

#[expect(
    clippy::missing_trait_methods,
    reason = "deserialize_in_place is not applicable for this wrapper"
)]
impl<'de> serde::Deserialize<'de> for PropertyName {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = Box::<str>::deserialize(deserializer)?;
        Self::try_from(s).map_err(serde::de::Error::custom)
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
    Archive,
    Serialize,
    Deserialize,
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
    Archive,
    Serialize,
    Deserialize,
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
                let _name = PropertyName::try_new(&self.name)?;
                Ok(Property::new(
                    PropertyId::from_uuid(TEST_PROPERTY_ID),
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
            Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID),
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

            // THEN: it should return a PropertyNameError::TooLong error
            assert!(
                matches!(
                    res,
                    Err(SchemaError::Syntax(
                        crate::schema::error::SchemaSyntaxError::PropertyName(
                            crate::schema::error::PropertyNameError::TooLong { .. }
                        )
                    ))
                ),
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

            // THEN: it should return a PropertyNameError::Empty error
            assert!(
                matches!(
                    res,
                    Err(SchemaError::Syntax(
                        crate::schema::error::SchemaSyntaxError::PropertyName(
                            crate::schema::error::PropertyNameError::Empty { .. }
                        )
                    ))
                ),
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
