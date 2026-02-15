//! Schema domain aggregate and supporting value objects.
//!
//! Provides a pure domain representation of metadata schemas for validation.

#![allow(
    clippy::module_name_repetitions,
    reason = "Core domain logic and naming convention where \
              Schema/PropertyBank prefixes are descriptive"
)]
#![allow(
    clippy::pub_use,
    reason = "Re-exports from bank.rs for backward compatibility after module \
              extraction"
)]

use std::{
    borrow::Borrow,
    fmt::{Debug, Display},
    sync::LazyLock,
};

use regex::Regex;
use uuid::Uuid;

// Re-export PropertyBank types from bank module for backward
// Re-export PropertyBank types from bank module for backward
// compatibility. This maintains the existing public API after extraction
// to bank.rs.
pub use super::bank::{BankVersion, PropertyBank, PropertyBankId};
use super::{
    error::SchemaError,
    events::{Events, SchemaCreated},
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
/// let schema = Schema::new(SchemaId::new(), name, vec![])?;
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
    id: SchemaId,
    /// Unique schema name.
    name: SchemaName,
    /// Fully resolved properties after inheritance.
    properties: Vec<Property>,
    /// Domain events pending emission.
    #[serde(skip)]
    pending_events: Vec<Events>,
}

impl Schema {
    /// Create a new resolved Schema.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::schema::aggregate::{Schema, SchemaId, SchemaName};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let name = SchemaName::new("project-note")?;
    /// let schema = Schema::new(SchemaId::new(), name, vec![])?;
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
        properties: Vec<Property>,
    ) -> Result<Self, SchemaError> {
        let mut schema = Self {
            id,
            name,
            properties,
            pending_events: vec![],
        };

        schema.add_event(Events::SchemaCreated(SchemaCreated::new(
            id,
            &schema.name,
            Timestamp::now(),
        )));

        Ok(schema)
    }

    /// Returns the schema's unique identifier.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> SchemaId {
        self.id
    }

    /// Returns the schema's unique name.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &SchemaName {
        &self.name
    }

    /// Returns the fully resolved properties.
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
    /// let schema = Schema::new(SchemaId::new(), name, vec![])?;
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
        self.properties.iter().find(|p| p.name() == name)
    }

    /// Checks if a property exists by name.
    #[inline]
    #[must_use]
    pub fn has(&self, name: &PropertyName) -> bool {
        self.properties.iter().any(|p| p.name() == name)
    }

    /// Returns a reference to pending domain events.
    #[inline]
    #[must_use]
    pub fn pending_events(&self) -> &[Events] {
        &self.pending_events
    }

    /// Returns and clears pending domain events.
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

/// Resolution metadata for incremental schema resolution.
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
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct ResolutionMetadata {
    schema_id: SchemaId,
    resolved_at: Timestamp,
    parent_hash: Option<SchemaHash>,
    bank_version: BankVersion,
    file_modified: Option<Timestamp>,
}

impl ResolutionMetadata {
    /// Create a new resolution metadata snapshot.
    #[inline]
    #[must_use]
    pub const fn new(
        schema_id: SchemaId,
        resolved_at: Timestamp,
        parent_hash: Option<SchemaHash>,
        bank_version: BankVersion,
        file_modified: Option<Timestamp>,
    ) -> Self {
        Self {
            schema_id,
            resolved_at,
            parent_hash,
            bank_version,
            file_modified,
        }
    }

    /// Returns the schema id associated with this metadata.
    #[inline]
    #[must_use]
    pub const fn schema_id(&self) -> SchemaId {
        self.schema_id
    }

    /// Returns the resolution timestamp.
    #[inline]
    #[must_use]
    pub const fn resolved_at(&self) -> Timestamp {
        self.resolved_at
    }

    /// Returns the stored parent hash.
    #[inline]
    #[must_use]
    pub const fn parent_hash(&self) -> Option<SchemaHash> {
        self.parent_hash
    }

    /// Returns the property bank version.
    #[inline]
    #[must_use]
    pub const fn bank_version(&self) -> BankVersion {
        self.bank_version
    }

    /// Returns the file modification timestamp, if any.
    #[inline]
    #[must_use]
    pub const fn file_modified(&self) -> Option<Timestamp> {
        self.file_modified
    }

    /// Returns true if this metadata is stale relative to current values.
    #[inline]
    #[must_use]
    pub fn is_stale(
        &self,
        current_bank_version: BankVersion,
        current_parent_hash: Option<SchemaHash>,
        current_file_mtime: Option<Timestamp>,
    ) -> bool {
        if self.bank_version.is_older_than(current_bank_version) {
            return true;
        }

        if self.parent_hash != current_parent_hash {
            return true;
        }

        if let Some(stored_mtime) = self.file_modified
            && let Some(current_mtime) = current_file_mtime
            && stored_mtime < current_mtime
        {
            return true;
        }

        false
    }
}

// ----------------------------------------------------------- //
//                    Primary Value Objects                    //
// ----------------------------------------------------------- //

/// Unique identity for a schema.
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
#[serde(transparent)]
#[non_exhaustive]
pub struct SchemaId(Uuid);

impl SchemaId {
    /// Creates a new UUID v7-based `SchemaId`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Wraps a UUID into a `SchemaId`.
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

    /// Validates a schema name string.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
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
            return Err(SchemaError::InvalidSchemaName(name.to_owned()));
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

/// Content hash for schema staleness detection.
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
#[serde(transparent)]
#[non_exhaustive]
pub struct SchemaHash(u64);

impl SchemaHash {
    /// Wraps a raw hash value.
    #[inline]
    #[must_use]
    pub const fn from_u64(value: u64) -> Self {
        Self(value)
    }

    /// Returns the hash as a raw integer.
    #[inline]
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Compute a stable content hash for a schema.
    #[inline]
    #[must_use]
    pub fn compute(schema: &Schema) -> Self {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{Hash as _, Hasher as _},
        };

        let mut hasher = DefaultHasher::new();

        schema.name().as_str().hash(&mut hasher);
        for prop in schema.properties() {
            prop.name().as_str().hash(&mut hasher);
            prop.cardinality().hash(&mut hasher);
            prop.multiplicity().hash(&mut hasher);
            prop.spec().hash(&mut hasher);
        }

        Self::from_u64(hasher.finish())
    }
}

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
    #[inline]
    #[must_use]
    pub fn now() -> Self {
        Self(chrono::Utc::now().timestamp())
    }

    /// Wraps a timestamp in seconds.
    #[inline]
    #[must_use]
    pub const fn from_secs(secs: i64) -> Self {
        Self(secs)
    }

    /// Returns the timestamp in seconds.
    #[inline]
    #[must_use]
    pub const fn as_secs(self) -> i64 {
        self.0
    }
}

// ----------------------------------------------------------- //
//                        Storage Keys                         //
// ----------------------------------------------------------- //

/// Normalized schema name for storage indexing.
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
#[serde(transparent)]
#[non_exhaustive]
pub struct SchemaNameKey(Box<str>);

impl SchemaNameKey {
    /// Returns the normalized key as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&SchemaName> for SchemaNameKey {
    #[inline]
    fn from(name: &SchemaName) -> Self {
        Self(name.as_str().to_lowercase().into_boxed_str())
    }
}

/// Normalized property name for storage indexing.
///
/// Used for composite indexes and property lookups in storage projections.
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
#[serde(transparent)]
#[non_exhaustive]
pub struct PropertyNameKey(Box<str>);

impl PropertyNameKey {
    /// Returns the normalized key as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&super::property::PropertyName> for PropertyNameKey {
    #[inline]
    fn from(name: &super::property::PropertyName) -> Self {
        Self(name.as_str().to_lowercase().into_boxed_str())
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
#[expect(
    clippy::disallowed_methods,
    reason = "Test setup uses expect for deterministic fixtures."
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
            Schema::new(SchemaId::from_uuid(TEST_SCHEMA_ID_A), name, vec![
                property,
            ])
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
                1,
                "Expected exactly 1 pending event"
            );
        }
    }
}
