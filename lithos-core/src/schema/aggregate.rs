//! Schema domain aggregates for property registration and validation.
//!
//! Provides a pure domain representation of metadata schemas and a centralized
//! `PropertyBank` for O(1) property lookups.

#![allow(
    clippy::module_name_repetitions,
    reason = "Core domain logic and naming convention where \
              Schema/PropertyBank prefixes are descriptive"
)]

use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::{Debug, Display},
    sync::LazyLock,
};

use regex::Regex;
use uuid::Uuid;

use super::{
    error::SchemaError,
    events::{Events, PropertyBankUpdated, SchemaCreated},
    property::{Property, PropertyId, PropertyName},
};
use crate::patterns;

/// Registry of reusable Property definitions with dual indexing.
///
/// Provides O(1) lookup by ID and Name.
///
/// The `PropertyBank` acts as a singleton registry with a stable UUID identity.
/// It is loaded first at program start and versioned for incremental
/// resolution.
///
/// # Storage Strategy
///
/// The `PropertyBank` uses UUID-first storage with a singleton identity:
/// - **Primary key**: `PropertyBankId::singleton()` (fixed UUID)
/// - **Lifecycle**: Loaded once at startup, persisted on modification
/// - **Versioning**: `BankVersion` increments on any property change
///
/// # Examples
///
/// ```
/// # use lithos_core::schema::aggregate::PropertyBank;
/// # use lithos_core::schema::property::{
/// #     Cardinality, Multiplicity, Property, PropertyId, PropertyName,
/// # };
/// # use lithos_core::schema::property_spec::{PropertySpec, BoolSpec};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut bank = PropertyBank::new();
/// let name = PropertyName::new("is_active")?;
/// let spec = PropertySpec::Bool(BoolSpec::default());
/// let id = PropertyId::new();
/// let property = Property::new(
///     id,
///     name.clone(),
///     Cardinality::Required,
///     Multiplicity::Single,
///     spec,
/// )?;
///
/// bank.register(property)?;
/// assert!(bank.has_name(&name), "Bank should contain property name");
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
pub struct PropertyBank {
    /// Unique identity for the property bank (singleton).
    id: PropertyBankId,
    /// Index mapping ID -> index in properties vector.
    id_index: HashMap<PropertyId, usize>,
    /// Index mapping Name -> index in properties vector.
    name_index: HashMap<PropertyName, usize>,
    /// Dense storage of properties.
    properties: Vec<Property>,
    /// Version counter for staleness detection.
    version: BankVersion,
    /// Domain events pending emission.
    #[serde(skip)]
    pending_events: Vec<Events>,
}

impl Default for PropertyBank {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
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

/// Unique identity for a property bank.
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
pub struct PropertyBankId(Uuid);

impl PropertyBankId {
    /// Wraps a UUID into a `PropertyBankId`.
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

    /// Creates a new UUID v7-based `PropertyBankId`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Returns the singleton `PropertyBank` ID.
    ///
    /// The `PropertyBank` uses a fixed UUID to act as a singleton registry.
    /// This ensures consistent identity across all program runs.
    #[inline]
    #[must_use]
    pub const fn singleton() -> Self {
        // Fixed UUID v7 for singleton PropertyBank
        Self(Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0001))
    }
}

impl Default for PropertyBankId {
    #[inline]
    fn default() -> Self {
        Self::singleton()
    }
}

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

    /// Creates a new UUID v7-based `SchemaId`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
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

/// `PropertyBank` version counter for staleness detection.
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
pub struct BankVersion(u64);

impl BankVersion {
    /// Returns the initial version.
    #[inline]
    #[must_use]
    pub const fn initial() -> Self {
        Self(0)
    }

    /// Returns the next version value.
    #[inline]
    #[must_use]
    pub const fn increment(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    /// Returns the version as a raw integer.
    #[inline]
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns true when this version is older than the other.
    #[inline]
    #[must_use]
    pub const fn is_older_than(self, other: Self) -> bool {
        self.0 < other.0
    }
}

impl Default for BankVersion {
    #[inline]
    fn default() -> Self {
        Self::initial()
    }
}

impl Display for BankVersion {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}", self.0)
    }
}

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
            Timestamp::now(),
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
            && let Some(prop) = self.get_by_id(PropertyId::from_uuid(id))
        {
            return Ok(prop);
        }

        // Fall back to name lookup
        let name = PropertyName::try_from(key)?;
        self.get_by_name(&name)
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
    /// assert!(bank.get("any").is_none(), "Missing property should be None");
    /// ```
    #[inline]
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Property> {
        // Try by ID first
        if let Ok(id) = Uuid::parse_str(key)
            && let Some(prop) = self.get_by_id(PropertyId::from_uuid(id))
        {
            return Some(prop);
        }

        // Fall back to name lookup
        let name = PropertyName::try_from(key).ok()?;
        self.get_by_name(&name)
    }

    /// Lookup property by ID (O(1)).
    #[inline]
    #[must_use]
    pub fn get_by_id(&self, id: PropertyId) -> Option<&Property> {
        let &idx = self.id_index.get(&id)?;
        self.properties.get(idx)
    }

    /// Lookup property by Name (O(1)).
    #[inline]
    #[must_use]
    pub fn get_by_name(&self, name: &PropertyName) -> Option<&Property> {
        let &idx = self.name_index.get(name)?;
        self.properties.get(idx)
    }

    /// Checks if a property exists by ID.
    #[inline]
    #[must_use]
    pub fn has_id(&self, id: PropertyId) -> bool {
        self.id_index.contains_key(&id)
    }

    /// Checks if a property exists by name.
    #[inline]
    #[must_use]
    pub fn has_name(&self, name: &PropertyName) -> bool {
        self.name_index.contains_key(name)
    }

    /// Returns the `PropertyBank`'s unique identifier.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> PropertyBankId {
        self.id
    }

    /// Create a new empty `PropertyBank`.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: PropertyBankId::singleton(),
            id_index: HashMap::new(),
            name_index: HashMap::new(),
            properties: Vec::new(),
            version: BankVersion::initial(),
            pending_events: Vec::new(),
        }
    }

    /// Returns the current `PropertyBank` version.
    #[inline]
    #[must_use]
    pub const fn version(&self) -> BankVersion {
        self.version
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
    ///     property::{
    ///         Cardinality, Multiplicity, Property, PropertyId, PropertyName,
    ///     },
    ///     property_spec::{BoolSpec, PropertySpec},
    /// };
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let mut bank = PropertyBank::new();
    ///
    /// let name = PropertyName::new("is_active")?;
    /// let spec = PropertySpec::Bool(BoolSpec::default());
    /// let id = PropertyId::new();
    /// let property = Property::new(
    ///     id,
    ///     name,
    ///     Cardinality::Required,
    ///     Multiplicity::Single,
    ///     spec,
    /// )?;
    ///
    /// bank.register(property)?;
    /// assert_eq!(bank.all().count(), 1, "Bank should contain one property");
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
        self.validate_name_unique(property.name())?;

        let id = property.id();
        let name = property.name().clone();
        let idx = self.properties.len();

        self.id_index.insert(id, idx);
        self.name_index.insert(name, idx);
        self.properties.push(property);
        self.version = self.version.increment();

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

    fn validate_name_unique(
        &self,
        name: &PropertyName,
    ) -> Result<(), SchemaError> {
        if self.name_index.contains_key(name) {
            return Err(SchemaError::DuplicatePropertyName(
                name.as_str().to_owned(),
            ));
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
            property_spec::{BoolSpec, PropertySpec, StringSpec},
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

        pub fn bank_with_property()
        -> Result<(PropertyBank, PropertyId), SchemaError> {
            let mut bank = PropertyBank::new();
            let property = Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID_A),
                PropertyName::new("flag")?,
                Cardinality::Required,
                Multiplicity::Single,
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
            let name = PropertyName::new("test").expect("Valid name");
            let prop = Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID_A),
                name,
                Cardinality::Optional,
                Multiplicity::Single,
                spec,
            )
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
                "Registered property should be retrievable by ID: {id:?}"
            );
        }

        /// 3.3-UNIT-020: `maintains_dual_indices_for_fast_lookup`.
        /// Priority: P1.
        #[test]
        fn maintains_name_index_for_fast_lookup() {
            let (bank, _id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            let name = PropertyName::new("flag").expect("Valid name");

            assert!(
                bank.get_by_name(&name).is_some(),
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
            let name = PropertyName::new("test").expect("Valid name");
            let prop1 = Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID_A),
                name.clone(),
                Cardinality::Optional,
                Multiplicity::Single,
                spec1,
            )
            .expect("Valid property");
            bank.register(prop1).expect("Initial registration should succeed");

            // WHEN: registering a different definition with the same name
            let spec2 = PropertySpec::Bool(BoolSpec::default());
            let prop2 = Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID_B),
                name,
                Cardinality::Optional,
                Multiplicity::Single,
                spec2,
            )
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

            let name = PropertyName::new("flag").expect("Valid name");

            assert!(
                bank.has_name(&name),
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
                "Should retrieve property by ID: {id:?}"
            );
        }

        /// 3.2-UNIT-011: `property_bank_accessors_cover_ids_and_names`.
        /// Priority: P1.
        #[test]
        fn property_bank_gets_by_name() {
            let (bank, _id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            let name = PropertyName::new("flag").expect("Valid name");

            assert!(
                bank.get_by_name(&name).is_some(),
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
                bank.get(id.as_uuid().to_string().as_str()).is_some(),
                "Should retrieve property by ID string: {id:?}"
            );
        }

        /// 3.2-UNIT-011: `property_bank_accessors_cover_ids_and_names`.
        /// Priority: P1.
        #[test]
        fn property_bank_decodes_id_string() {
            let (bank, id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            let result = bank.decode(id.as_uuid().to_string().as_str());
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
