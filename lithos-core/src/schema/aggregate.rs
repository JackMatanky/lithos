//! Schema domain aggregates for property registration and validation.
//!
//! Provides a pure domain representation of metadata schemas and a centralized
//! `PropertyBank` for O(1) property lookups.

#![allow(
    clippy::module_name_repetitions,
    reason = "Core domain logic and naming convention where \
              Schema/PropertyBank prefixes are descriptive"
)]
#![expect(
    clippy::exhaustive_structs,
    reason = "rkyv generates exhaustive ArchivedSchema/ArchivedSchemaName \
              despite #[non_exhaustive]"
)]

use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    sync::LazyLock,
};

use regex::Regex;
use uuid::Uuid;

use super::{
    error::SchemaError,
    events::{Events, PropertyBankUpdated, SchemaCreated},
    property::Property,
};
use crate::patterns;

/// Registry of reusable Property definitions with dual indexing.
///
/// Provides O(1) lookup by ID and Name.
///
/// # Examples
///
/// ```
/// # use lithos_core::schema::aggregate::PropertyBank;
/// # use lithos_core::schema::property::{Property, PropertyName};
/// # use lithos_core::schema::property_spec::{PropertySpec, BoolSpec};
/// # use uuid::Uuid;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut bank = PropertyBank::new();
/// let name = PropertyName::new("is_active".to_string())?;
/// let spec = PropertySpec::Bool(BoolSpec::default());
/// let id = Uuid::now_v7();
/// let property = Property::new(id, name, true, false, spec)?;
///
/// bank.register(property)?;
/// assert!(bank.has_name("is_active"));
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[non_exhaustive]
pub struct PropertyBank {
    /// Index mapping ID -> index in properties vector.
    id_index: HashMap<Uuid, usize>,
    /// Index mapping Name -> index in properties vector.
    name_index: HashMap<String, usize>,
    /// Dense storage of properties.
    properties: Vec<Property>,
    /// Domain events pending emission.
    #[serde(skip)]
    pending_events: Vec<Events>,
}

/// Schema aggregate defining metadata validation constraints.
///
/// Represents a fully resolved schema used as the source of truth for
/// validating note metadata.
///
/// # Constraints
/// - **Identity**: Name must match alphanumeric/underscore/dash format.
/// - **Integrity**: Properties must be fully resolved and unique by name.
///
/// # Examples
///
/// ```
/// use lithos_core::schema::aggregate::{Schema, SchemaName};
/// use uuid::Uuid;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// let name = SchemaName::new("project-note".into())?;
/// let schema = Schema::new(Uuid::now_v7(), name, vec![])?;
/// assert!(
///     schema.properties().is_empty(),
///     "New schema should have empty properties"
/// );
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
pub struct Schema {
    /// UUID v7 identity for schema.
    id: Uuid,
    /// Unique schema name.
    name: SchemaName,
    /// Fully resolved properties after inheritance.
    properties: Vec<Property>,
    /// Domain events pending emission.
    #[serde(skip)]
    pending_events: Vec<Events>,
}

/// Validated schema name value object.
///
/// Enforces invariants:
/// - Non-empty
/// - Max 64 characters
/// - Matches regex `^[a-zA-Z0-9_-]+$` (alphanumeric, underscores, dashes)
///
/// # Examples
///
/// ```
/// use lithos_core::schema::aggregate::SchemaName;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// let name = SchemaName::new("project-note".to_string())?;
/// assert_eq!(&name.0, "project-note");
///
/// let name2 = SchemaName::new("daily_note".to_string())?;
/// assert_eq!(&name2.0, "daily_note");
///
/// let name3 = SchemaName::new("MySchema".to_string())?;
/// assert_eq!(&name3.0, "MySchema");
///
/// let invalid = SchemaName::new("".to_string());
/// assert!(invalid.is_err(), "Empty name should be rejected");
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
pub struct SchemaName(pub String);

impl AsRef<str> for SchemaName {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for SchemaName {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for SchemaName {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<SchemaName> for String {
    #[inline]
    fn from(val: SchemaName) -> Self {
        val.0
    }
}

impl TryFrom<&str> for SchemaName {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl TryFrom<String> for SchemaName {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl PropertyBank {
    /// Adds a domain event to the pending events collection.
    #[inline]
    fn add_event(&mut self, event: Events) {
        self.pending_events.push(event);
    }

    /// Get all properties in the bank.
    #[inline]
    pub fn all(&self) -> impl Iterator<Item = &Property> {
        self.properties.iter()
    }

    fn create_updated_event(&self) -> Events {
        Events::PropertyBankUpdated(PropertyBankUpdated::new(
            self.properties.len(),
            chrono::Utc::now().timestamp(),
        ))
    }

    /// Decodes a `$ref` path to a Property.
    ///
    /// This method performs a key lookup for a property. Format-specific
    /// parsing (e.g., handling "#/properties/") must be handled by the
    /// adapters.
    ///
    /// # Errors
    /// Returns `PropertyNotFound` if key does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::schema::aggregate::PropertyBank;
    ///
    /// let bank = PropertyBank::new();
    ///
    /// let result = bank.decode("missing");
    /// assert!(result.is_err(), "Decoding missing property should fail");
    /// ```
    #[inline]
    pub fn decode(&self, key: &str) -> Result<&Property, SchemaError> {
        // Try parsing key as UUID first
        if let Ok(id) = Uuid::parse_str(key)
            && let Some(prop) = self.get_by_id(id)
        {
            return Ok(prop);
        }
        // Fall back to name lookup
        self.get_by_name(key)
            .ok_or_else(|| SchemaError::PropertyNotFound(key.to_owned()))
    }

    /// Gets a property by name or ID (string).
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::schema::aggregate::PropertyBank;
    ///
    /// let bank = PropertyBank::new();
    ///
    /// assert!(bank.get("any").is_none());
    /// ```
    #[inline]
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Property> {
        // Try by ID first
        if let Ok(id) = Uuid::parse_str(key)
            && let Some(prop) = self.get_by_id(id)
        {
            return Some(prop);
        }
        // Fall back to name lookup
        self.get_by_name(key)
    }

    /// Lookup property by ID (O(1)).
    #[inline]
    #[must_use]
    pub fn get_by_id(&self, id: Uuid) -> Option<&Property> {
        let &idx = self.id_index.get(&id)?;
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
    pub fn has_id(&self, id: Uuid) -> bool {
        self.id_index.contains_key(&id)
    }

    /// Checks if a property exists by name.
    #[inline]
    #[must_use]
    pub fn has_name(&self, name: &str) -> bool {
        self.name_index.contains_key(name)
    }

    /// Create a new empty `PropertyBank`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to pending domain events.
    #[inline]
    #[must_use]
    pub fn pending_events(&self) -> &[Events] {
        &self.pending_events
    }

    /// Register a property in the bank.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::schema::{
    ///     aggregate::PropertyBank,
    ///     property::{Property, PropertyName},
    ///     property_spec::{BoolSpec, PropertySpec},
    /// };
    /// use uuid::Uuid;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let mut bank = PropertyBank::new();
    ///
    /// let name = PropertyName::new("is_active".to_string())?;
    /// let spec = PropertySpec::Bool(BoolSpec::default());
    /// let id = Uuid::now_v7();
    /// let property = Property::new(id, name, true, false, spec)?;
    ///
    /// bank.register(property)?;
    /// assert_eq!(bank.all().count(), 1);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn register(&mut self, property: Property) -> Result<(), SchemaError> {
        property.validate()?;

        // Idempotent success if ID already exists
        if self.id_index.contains_key(&property.id()) {
            let event = self.create_updated_event();
            self.add_event(event);
            return Ok(());
        }

        // Prevent duplicate names
        self.validate_name_unique(&property.name().0)?;

        let id = property.id();
        let name = property.name().0.clone();
        let idx = self.properties.len();

        self.id_index.insert(id, idx);
        self.name_index.insert(name, idx);
        self.properties.push(property);

        let event = self.create_updated_event();
        self.add_event(event);

        Ok(())
    }

    /// Returns and clears pending domain events.
    #[inline]
    #[must_use]
    pub fn take_events(&mut self) -> Vec<Events> {
        std::mem::take(&mut self.pending_events)
    }

    fn validate_name_unique(&self, name: &str) -> Result<(), SchemaError> {
        if self.name_index.contains_key(name) {
            return Err(SchemaError::DuplicatePropertyName(name.to_owned()));
        }
        Ok(())
    }
}

impl Schema {
    /// Adds a domain event to the pending events collection.
    #[inline]
    fn add_event(&mut self, event: Events) {
        self.pending_events.push(event);
    }

    /// Gets a property by name.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::schema::aggregate::{Schema, SchemaName};
    /// use uuid::Uuid;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let name = SchemaName::new("test".into())?;
    /// let schema = Schema::new(Uuid::now_v7(), name, vec![])?;
    /// assert!(schema.get("missing").is_none());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Property> {
        self.properties.iter().find(|p| p.name().0 == name)
    }

    /// Checks if a property exists by name.
    #[inline]
    #[must_use]
    pub fn has(&self, name: &str) -> bool {
        self.properties.iter().any(|p| p.name().0 == name)
    }

    /// Returns the schema's unique identifier.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Returns the schema's unique name.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &SchemaName {
        &self.name
    }

    /// Create a new resolved Schema.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::schema::aggregate::{Schema, SchemaName};
    /// use uuid::Uuid;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let name = SchemaName::new("project-note".to_string())?;
    /// let schema = Schema::new(Uuid::now_v7(), name, vec![])?;
    /// assert_eq!(&schema.name().0, "project-note");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn new(
        id: Uuid,
        name: SchemaName,
        properties: Vec<Property>,
    ) -> Result<Self, SchemaError> {
        let name_str = name.to_string();
        let mut schema = Self {
            id,
            name,
            properties,
            pending_events: vec![],
        };

        schema.add_event(Events::SchemaCreated(SchemaCreated::new(
            id,
            name_str,
            chrono::Utc::now().timestamp(),
        )));

        Ok(schema)
    }

    /// Returns a reference to pending domain events.
    #[inline]
    #[must_use]
    pub fn pending_events(&self) -> &[Events] {
        &self.pending_events
    }

    /// Returns the fully resolved properties.
    #[inline]
    #[must_use]
    pub fn properties(&self) -> &[Property] {
        &self.properties
    }

    /// Returns and clears pending domain events.
    #[inline]
    #[must_use]
    pub fn take_events(&mut self) -> Vec<Events> {
        std::mem::take(&mut self.pending_events)
    }
}

impl SchemaName {
    /// Create a new `SchemaName` with validation.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn new(name: String) -> Result<Self, SchemaError> {
        Self::validate(&name)?;
        Ok(Self(name))
    }

    /// Validates a schema name string.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn validate(name: &str) -> Result<(), SchemaError> {
        static RE: LazyLock<Result<Regex, regex::Error>> =
            LazyLock::new(|| Regex::new(patterns::ALPHANUMERIC_NAME));

        if name.is_empty() {
            return Err(SchemaError::EmptySchemaName);
        }
        if name.len() > 64 {
            return Err(SchemaError::SchemaNameTooLong(name.len()));
        }

        let re = RE.as_ref().map_err(|error| {
            SchemaError::ValidationFailed(format!(
                "Invalid schema name regex: {error}"
            ))
        })?;

        if !re.is_match(name) {
            return Err(SchemaError::InvalidSchemaName(name.to_owned()));
        }
        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures and submodules for readability."
)]
#[expect(
    clippy::disallowed_methods,
    reason = "Test setup uses expect for deterministic fixtures."
)]
mod tests {
    use uuid::Uuid;

    use super::{
        super::{
            property::{Property, PropertyName},
            property_spec::{BoolSpec, PropertySpec, StringSpec},
        },
        *,
    };

    mod fixtures {
        use super::*;

        pub fn sample_schema() -> Result<Schema, SchemaError> {
            let name = SchemaName::new("status".to_owned())?;
            let property = Property::new(
                TEST_PROPERTY_ID_C,
                PropertyName::new("flag".to_owned())?,
                true,
                false,
                PropertySpec::Bool(BoolSpec::default()),
            )?;
            Schema::new(TEST_SCHEMA_ID_A, name, vec![property])
        }

        pub fn bank_with_property() -> Result<(PropertyBank, Uuid), SchemaError>
        {
            let mut bank = PropertyBank::new();
            let property = Property::new(
                TEST_PROPERTY_ID_A,
                PropertyName::new("flag".to_owned())?,
                true,
                false,
                PropertySpec::Bool(BoolSpec::default()),
            )?;
            let id = property.id();
            bank.register(property)?;
            Ok((bank, id))
        }
    }

    const TEST_PROPERTY_ID_A: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0701);
    const TEST_PROPERTY_ID_B: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0702);
    const TEST_PROPERTY_ID_C: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0703);
    const TEST_SCHEMA_ID_A: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0704);

    mod property_bank {
        use super::*;

        /// 3.3-UNIT-023: `is_idempotent_on_identical_registration`.
        /// Priority: P1.
        #[test]
        fn is_idempotent_on_identical_registration() {
            // GIVEN: a PropertyBank and an existing property
            let mut bank = PropertyBank::new();
            let spec = PropertySpec::String(StringSpec::default());
            let name =
                PropertyName::new("test".to_owned()).expect("Valid name");
            let prop =
                Property::new(TEST_PROPERTY_ID_A, name, false, false, spec)
                    .expect("Valid property");

            // WHEN: registering the same property twice
            bank.register(prop.clone())
                .expect("First registration should succeed");
            bank.register(prop).expect("Second registration should succeed");

            // THEN: the count remains 1
            let count = bank.all().count();
            assert_eq!(
                count, 1,
                "Expected 1 property after identical registrations"
            );
        }

        /// 3.3-UNIT-020: `maintains_dual_indices_for_fast_lookup`.
        /// Priority: P1.
        #[test]
        fn maintains_id_index_for_fast_lookup() {
            let (bank, id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            assert!(
                bank.get_by_id(id).is_some(),
                "Registered property should be retrievable by ID: {id}"
            );
        }

        /// 3.3-UNIT-020: `maintains_dual_indices_for_fast_lookup`.
        /// Priority: P1.
        #[test]
        fn maintains_name_index_for_fast_lookup() {
            let (bank, _id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            assert!(
                bank.get_by_name("flag").is_some(),
                "Registered property should be retrievable by name: 'flag'"
            );
        }

        /// 3.3-UNIT-024: `rejects_duplicate_names_with_different_definitions`.
        /// Priority: P1.
        #[test]
        fn rejects_duplicate_names_with_different_definitions() {
            // GIVEN: a PropertyBank with a registered property
            let mut bank = PropertyBank::new();
            let spec1 = PropertySpec::String(StringSpec::default());
            let name =
                PropertyName::new("test".to_owned()).expect("Valid name");
            let prop1 = Property::new(
                TEST_PROPERTY_ID_A,
                name.clone(),
                false,
                false,
                spec1,
            )
            .expect("Valid property");
            bank.register(prop1).expect("Initial registration should succeed");

            // WHEN: registering a different definition with the same name
            let spec2 = PropertySpec::Bool(BoolSpec::default());
            let prop2 =
                Property::new(TEST_PROPERTY_ID_B, name, false, false, spec2)
                    .expect("Valid property definition");
            let res = bank.register(prop2);

            // THEN: it must return a DuplicatePropertyName error
            assert!(
                matches!(res, Err(SchemaError::DuplicatePropertyName(_))),
                "Duplicate property name should be rejected with \
                 DuplicatePropertyName, got: {res:?}"
            );
        }

        /// 3.2-UNIT-011: `property_bank_accessors_cover_ids_and_names`.
        /// Priority: P1.
        #[test]
        fn property_bank_has_id() {
            let (bank, id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            assert!(
                bank.has_id(id),
                "PropertyBank should contain property by ID"
            );
        }

        /// 3.2-UNIT-011: `property_bank_accessors_cover_ids_and_names`.
        /// Priority: P1.
        #[test]
        fn property_bank_has_name() {
            let (bank, _id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            assert!(
                bank.has_name("flag"),
                "PropertyBank should contain property by name 'flag'"
            );
        }

        /// 3.2-UNIT-011: `property_bank_accessors_cover_ids_and_names`.
        /// Priority: P1.
        #[test]
        fn property_bank_gets_by_id() {
            let (bank, id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            assert!(
                bank.get_by_id(id).is_some(),
                "Should retrieve property by ID: {id}"
            );
        }

        /// 3.2-UNIT-011: `property_bank_accessors_cover_ids_and_names`.
        /// Priority: P1.
        #[test]
        fn property_bank_gets_by_name() {
            let (bank, _id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            assert!(
                bank.get_by_name("flag").is_some(),
                "Should retrieve property by name: 'flag'"
            );
        }

        /// 3.2-UNIT-011: `property_bank_accessors_cover_ids_and_names`.
        /// Priority: P1.
        #[test]
        fn property_bank_gets_by_id_string() {
            let (bank, id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            assert!(
                bank.get(id.to_string().as_str()).is_some(),
                "Should retrieve property by ID string: {id}"
            );
        }

        /// 3.2-UNIT-011: `property_bank_accessors_cover_ids_and_names`.
        /// Priority: P1.
        #[test]
        fn property_bank_decodes_id_string() {
            let (bank, id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            let result = bank.decode(id.to_string().as_str());
            assert!(result.is_ok(), "Decode should succeed: {result:?}");
        }

        /// 3.2-UNIT-011: `property_bank_events_emitted_on_registration`.
        /// Priority: P1.
        #[test]
        fn property_bank_pending_events_len_is_one() {
            let (bank, _id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            assert_eq!(
                bank.pending_events().len(),
                1,
                "Expected exactly 1 pending event"
            );
        }

        /// 3.2-UNIT-011: `property_bank_events_emitted_on_registration`.
        /// Priority: P1.
        #[test]
        fn property_bank_take_events_returns_one() {
            let (mut bank, _id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            let events = bank.take_events();
            assert_eq!(
                events.len(),
                1,
                "take_events should return exactly 1 event after registration"
            );
        }

        /// 3.2-UNIT-011: `property_bank_events_emitted_on_registration`.
        /// Priority: P1.
        #[test]
        fn property_bank_pending_events_cleared_after_take() {
            let (mut bank, _id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            let _events = bank.take_events();
            assert!(
                bank.pending_events().is_empty(),
                "pending_events should be empty after take_events"
            );
        }
    }

    mod schema {
        use super::*;

        /// 3.2-UNIT-010: `schema_accessors_return_expected_values`.
        /// Priority: P1.
        #[test]
        fn schema_name_accessors_return_inner_value() {
            let schema =
                fixtures::sample_schema().expect("Valid schema fixture");

            assert_eq!(
                schema.name().0,
                "status",
                "Schema name should expose inner string"
            );
        }

        /// 3.2-UNIT-010: `schema_accessors_return_expected_values`.
        /// Priority: P1.
        #[test]
        fn schema_name_as_ref_returns_expected_value() {
            let schema =
                fixtures::sample_schema().expect("Valid schema fixture");

            assert_eq!(
                schema.name().as_ref(),
                "status",
                "Schema name as_ref should match"
            );
        }

        /// 3.2-UNIT-010: `schema_accessors_return_expected_values`.
        /// Priority: P1.
        #[test]
        fn schema_name_to_string_returns_expected_value() {
            let schema =
                fixtures::sample_schema().expect("Valid schema fixture");

            assert_eq!(
                schema.name().to_string(),
                "status",
                "Schema name should render to string"
            );
        }

        /// 3.2-UNIT-010: `schema_property_accessors_return_expected_values`.
        /// Priority: P1.
        #[test]
        fn schema_has_property() {
            let schema =
                fixtures::sample_schema().expect("Valid schema fixture");

            assert!(
                schema.has("flag"),
                "Expected schema to have property 'flag'"
            );
        }

        /// 3.2-UNIT-010: `schema_property_accessors_return_expected_values`.
        /// Priority: P1.
        #[test]
        fn schema_gets_property() {
            let schema =
                fixtures::sample_schema().expect("Valid schema fixture");

            assert!(
                schema.get("flag").is_some(),
                "Expected schema.get('flag') to be Some"
            );
        }

        /// 3.2-UNIT-010: `schema_property_accessors_return_expected_values`.
        /// Priority: P1.
        #[test]
        fn schema_properties_len_is_one() {
            let schema =
                fixtures::sample_schema().expect("Valid schema fixture");

            assert_eq!(
                schema.properties().len(),
                1,
                "Expected exactly 1 property"
            );
        }

        /// 3.2-UNIT-010: `schema_pending_events_emitted_on_create`.
        /// Priority: P1.
        #[test]
        fn schema_pending_events_emitted_on_create() {
            let schema =
                fixtures::sample_schema().expect("Valid schema fixture");

            assert_eq!(
                schema.pending_events().len(),
                1,
                "Expected exactly 1 pending event"
            );
        }
    }
}
