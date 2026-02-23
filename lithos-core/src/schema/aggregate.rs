//! Schema domain aggregate and supporting value objects.
//!
//! Provides a pure domain representation of metadata schemas for validation.

#![allow(
    clippy::module_name_repetitions,
    reason = "Core domain logic and naming convention where Schema/SchemaName \
              prefixes are descriptive"
)]

use std::{
    borrow::Borrow,
    fmt::{Debug, Display},
    sync::LazyLock,
};

use regex::Regex;
use uuid::Uuid;

use super::{
    error::SchemaError,
    events::{Events, SchemaCreated, SchemaResolved},
    property::{Property, PropertyName},
};
use crate::patterns;

// ----------------------------------------------------------- //
//                    Schema Aggregate Root                    //
// ----------------------------------------------------------- //

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
/// use lithos_core::schema::aggregate::{Schema, SchemaId, SchemaName};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// let name = SchemaName::new("project-note")?;
/// let schema = Schema::new(SchemaId::new(), name, None, vec![])?;
/// assert!(
///     schema.properties().is_empty(),
///     "New schema should have empty properties"
/// );
/// # Ok(())
/// # }
/// ```
/// Unix timestamp in seconds.
///
/// # Examples
/// ```
/// use lithos_core::schema::aggregate::Timestamp;
///
/// let ts = Timestamp::from_secs(1_234);
/// assert_eq!(ts.as_secs(), 1_234);
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
    id: SchemaId,
    /// Unique schema name.
    name: SchemaName,
    /// Parent schema ID, for inheritance tree reconstruction.
    parent_id: Option<SchemaId>,
    /// Fully resolved properties after inheritance.
    properties: Vec<Property>,
    /// Domain events pending emission.
    #[serde(skip)]
    pending_events: Vec<Events>,
}

impl Schema {
    /// Create a new resolved Schema.
    ///
    /// This constructor is intended for genuinely new schemas. It emits both
    /// [`Events::SchemaCreated`] and [`Events::SchemaResolved`].
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::schema::aggregate::{Schema, SchemaId, SchemaName};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let name = SchemaName::new("project-note")?;
    /// let schema = Schema::new(SchemaId::new(), name, None, vec![])?;
    /// assert_eq!(
    ///     schema.name().as_str(),
    ///     "project-note",
    ///     "Schema name should match"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    #[inline]
    pub fn new(
        id: SchemaId,
        name: SchemaName,
        parent_id: Option<SchemaId>,
        properties: Vec<Property>,
    ) -> Result<Self, SchemaError> {
        let mut schema = Self {
            id,
            name,
            parent_id,
            properties,
            pending_events: vec![],
        };

        let now = Timestamp::now();
        schema.add_event(Events::SchemaCreated(SchemaCreated::new(
            id,
            &schema.name,
            now,
        )));
        schema.add_event(Events::SchemaResolved(SchemaResolved::new(
            id,
            &schema.name,
            now,
        )));

        Ok(schema)
    }

    /// Resolve an existing Schema.
    ///
    /// This constructor is intended for re-resolution of schemas that already
    /// exist in the database. It emits [`Events::SchemaResolved`] only.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::{Schema, SchemaId, SchemaName};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = SchemaName::new("existing")?;
    /// let schema = Schema::resolve_existing(SchemaId::new(), name, None, vec![])?;
    /// assert_eq!(schema.name().as_str(), "existing");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn resolve_existing(
        id: SchemaId,
        name: SchemaName,
        parent_id: Option<SchemaId>,
        properties: Vec<Property>,
    ) -> Result<Self, SchemaError> {
        let mut schema = Self {
            id,
            name,
            parent_id,
            properties,
            pending_events: vec![],
        };

        schema.add_event(Events::SchemaResolved(SchemaResolved::new(
            id,
            &schema.name,
            Timestamp::now(),
        )));

        Ok(schema)
    }

    /// Reconstruct a schema loaded from storage without emitting domain events.
    ///
    /// Use this when loading a previously-persisted schema from the database.
    /// Unlike [`Schema::new`], no events are emitted.
    #[inline]
    #[must_use]
    pub(crate) fn reconstruct(
        id: SchemaId,
        name: SchemaName,
        parent_id: Option<SchemaId>,
        properties: Vec<Property>,
    ) -> Self {
        Self {
            id,
            name,
            parent_id,
            properties,
            pending_events: vec![],
        }
    }

    /// Returns the schema's unique identifier.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::{Schema, SchemaId, SchemaName};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = SchemaName::new("test")?;
    /// let schema = Schema::new(SchemaId::new(), name, None, vec![])?;
    /// let _id = schema.id();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub const fn id(&self) -> SchemaId {
        self.id
    }

    /// Returns the schema's unique name.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::{Schema, SchemaId, SchemaName};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = SchemaName::new("test")?;
    /// let schema = Schema::new(SchemaId::new(), name, None, vec![])?;
    /// let _name = schema.name();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &SchemaName {
        &self.name
    }

    /// Returns the parent schema ID, if this schema extends another.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::{Schema, SchemaId, SchemaName};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = SchemaName::new("test")?;
    /// let schema = Schema::new(SchemaId::new(), name, None, vec![])?;
    /// assert!(schema.parent_id().is_none());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub const fn parent_id(&self) -> Option<SchemaId> {
        self.parent_id
    }

    /// Returns the fully resolved properties.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::{Schema, SchemaId, SchemaName};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = SchemaName::new("test")?;
    /// let schema = Schema::new(SchemaId::new(), name, None, vec![])?;
    /// assert!(schema.properties().is_empty());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn properties(&self) -> &[Property] {
        &self.properties
    }

    /// Gets a property by name.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::schema::{
    ///     aggregate::{Schema, SchemaId, SchemaName},
    ///     property::PropertyName,
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let name = SchemaName::new("test")?;
    /// let schema = Schema::new(SchemaId::new(), name, None, vec![])?;
    /// let missing = PropertyName::new("missing")?;
    /// assert!(
    ///     schema.get(&missing).is_none(),
    ///     "Missing property should return None"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn get(&self, name: &PropertyName) -> Option<&Property> {
        let i = self
            .properties
            .binary_search_by(|p| p.name().as_str().cmp(name.as_str()))
            .ok()?;
        self.properties.get(i)
    }

    /// Checks if a property exists by name.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::{
    ///     aggregate::{Schema, SchemaId, SchemaName},
    ///     property::PropertyName,
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = SchemaName::new("test")?;
    /// let schema = Schema::new(SchemaId::new(), name, None, vec![])?;
    /// let prop = PropertyName::new("missing")?;
    /// assert!(!schema.has(&prop));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn has(&self, name: &PropertyName) -> bool {
        self.properties
            .binary_search_by(|p| p.name().as_str().cmp(name.as_str()))
            .is_ok()
    }

    /// Returns a reference to pending domain events.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::{Schema, SchemaId, SchemaName};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = SchemaName::new("test")?;
    /// let schema = Schema::new(SchemaId::new(), name, None, vec![])?;
    /// let _events = schema.pending_events();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn pending_events(&self) -> &[Events] {
        &self.pending_events
    }

    /// Returns and clears pending domain events.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::{Schema, SchemaId, SchemaName};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = SchemaName::new("test")?;
    /// let mut schema = Schema::new(SchemaId::new(), name, None, vec![])?;
    /// let _events = schema.take_events();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn take_events(&mut self) -> Vec<Events> {
        std::mem::take(&mut self.pending_events)
    }

    /// Adds a domain event to the pending events collection.
    #[inline]
    fn add_event(&mut self, event: Events) {
        self.pending_events.push(event);
    }
}

// ----------------------------------------------------------- //
//                    Primary Value Objects                    //
// ----------------------------------------------------------- //

/// Unique identity for a schema.
///
/// # Examples
/// ```
/// use lithos_core::schema::aggregate::SchemaId;
///
/// let id = SchemaId::new();
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
#[rkyv(derive(Debug))]
#[serde(transparent)]
#[non_exhaustive]
pub struct SchemaId(Uuid);

impl std::fmt::Display for SchemaId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl SchemaId {
    /// Creates a new UUID v7-based `SchemaId`.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::SchemaId;
    ///
    /// let id = SchemaId::new();
    /// let _ = id.as_uuid();
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Wraps a UUID into a `SchemaId`.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::SchemaId;
    /// use uuid::Uuid;
    ///
    /// let uuid = Uuid::now_v7();
    /// let id = SchemaId::from_uuid(uuid);
    /// assert_eq!(*id.as_uuid(), uuid);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the inner UUID reference.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::SchemaId;
    ///
    /// let id = SchemaId::new();
    /// let _ = id.as_uuid();
    /// ```
    #[inline]
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Returns the inner UUID by value.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::SchemaId;
    ///
    /// let id = SchemaId::new();
    /// let _uuid = id.into_uuid();
    /// ```
    #[inline]
    #[must_use]
    pub const fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for SchemaId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Validated schema name value object.
///
/// Enforces invariants:
/// - Non-empty
/// - Max 64 characters
/// - Matches regex `^[a-z0-9_-]+$` (lowercase alphanumeric, underscores,
///   dashes)
///
/// # Examples
///
/// ```
/// use lithos_core::schema::aggregate::SchemaName;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// let name = SchemaName::new("project-note")?;
/// assert_eq!(name.as_str(), "project-note", "Schema name should match");
///
/// let name2 = SchemaName::new("daily_note")?;
/// assert_eq!(name2.as_str(), "daily_note", "Schema name should match");
///
/// let name3 = SchemaName::new("myschema")?;
/// assert_eq!(name3.as_str(), "myschema", "Schema name should match");
///
/// let invalid = SchemaName::new("");
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
pub struct SchemaName(Box<str>);

impl SchemaName {
    /// Create a new `SchemaName` with validation.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::SchemaName;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = SchemaName::new("project")?;
    /// assert_eq!(name.as_str(), "project");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn new(name: &str) -> Result<Self, SchemaError> {
        Self::validate(name)?;
        Ok(Self(name.into()))
    }

    /// Returns the inner string slice.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::SchemaName;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = SchemaName::new("project")?;
    /// assert_eq!(name.as_str(), "project");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates a schema name string.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::SchemaName;
    ///
    /// SchemaName::validate("project")?;
    /// # Ok::<_, lithos_core::schema::error::SchemaError>(())
    /// ```
    #[inline]
    pub fn validate(name: &str) -> Result<(), SchemaError> {
        static RE: LazyLock<Result<Regex, regex::Error>> =
            LazyLock::new(|| Regex::new(patterns::ALPHANUMERIC_NAME_LOWER));

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
            return Err(SchemaError::InvalidSchemaName(name.into()));
        }
        Ok(())
    }
}

impl AsRef<str> for SchemaName {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for SchemaName {
    #[inline]
    fn borrow(&self) -> &str {
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
        val.0.into()
    }
}

impl TryFrom<&str> for SchemaName {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for SchemaName {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

// ----------------------------------------------------------- //
//                  Supporting Value Objects                   //
// ----------------------------------------------------------- //

/// Unix timestamp (seconds since epoch).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[serde(transparent)]
#[non_exhaustive]
pub struct Timestamp(i64);

impl Timestamp {
    /// Returns the current UTC timestamp.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::Timestamp;
    ///
    /// let _now = Timestamp::now();
    /// ```
    #[inline]
    #[must_use]
    pub fn now() -> Self {
        Self(chrono::Utc::now().timestamp())
    }

    /// Wraps a timestamp in seconds.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::Timestamp;
    ///
    /// let ts = Timestamp::from_secs(10);
    /// assert_eq!(ts.as_secs(), 10);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_secs(secs: i64) -> Self {
        Self(secs)
    }

    /// Returns the timestamp in seconds.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::aggregate::Timestamp;
    ///
    /// let ts = Timestamp::from_secs(10);
    /// assert_eq!(ts.as_secs(), 10);
    /// ```
    #[inline]
    #[must_use]
    pub const fn as_secs(self) -> i64 {
        self.0
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures and submodules for readability."
)]
mod tests {
    use uuid::Uuid;

    use super::{
        super::{
            property::{
                Cardinality, Multiplicity, Property, PropertyId, PropertyName,
            },
            property_spec::{BoolSpec, PropertySpec},
        },
        *,
    };

    mod fixtures {
        use super::*;

        pub fn sample_schema() -> Result<Schema, SchemaError> {
            let name = SchemaName::new("status")?;
            let property = Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID_C),
                PropertyName::new("flag")?,
                Cardinality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            )?;
            Schema::new(
                SchemaId::from_uuid(TEST_SCHEMA_ID_A),
                name,
                None,
                vec![property],
            )
        }
    }

    const TEST_PROPERTY_ID_C: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0703);
    const TEST_SCHEMA_ID_A: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0704);

    mod schema {
        use super::*;

        /// 3.2-UNIT-010: `schema_accessors_return_expected_values`.
        /// Priority: P1.
        #[test]
        fn schema_name_accessors_return_inner_value() {
            let schema =
                fixtures::sample_schema().expect("Valid schema fixture");

            assert_eq!(
                schema.name().as_str(),
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

            let name = PropertyName::new("flag").expect("Valid name");

            assert!(
                schema.has(&name),
                "Expected schema to have property 'flag'"
            );
        }

        /// 3.2-UNIT-010: `schema_property_accessors_return_expected_values`.
        /// Priority: P1.
        #[test]
        fn schema_gets_property() {
            let schema =
                fixtures::sample_schema().expect("Valid schema fixture");

            let name = PropertyName::new("flag").expect("Valid name");

            assert!(
                schema.get(&name).is_some(),
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
                2,
                "Expected 2 pending events: SchemaCreated and SchemaResolved"
            );
        }
    }
}
