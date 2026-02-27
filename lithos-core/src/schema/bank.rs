//! PropertyBank domain aggregate for centralized property registration.
//!
//! Provides O(1) dual-indexed property lookup by ID and Name with singleton
//! identity and versioning for incremental resolution.

use std::{
    collections::{HashMap, hash_map::Entry},
    fmt::Display,
};

use uuid::Uuid;

use super::{
    error::SchemaError,
    events::{Events, PropertyRegistered},
    property::{Property, PropertyId, PropertyName},
};

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
/// # use lithos_core::schema::bank::PropertyBank;
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
/// Registry of reusable properties for schema validation.
///
/// # Examples
/// ```
/// use lithos_core::schema::bank::PropertyBank;
///
/// let bank = PropertyBank::new();
/// assert_eq!(bank.all().count(), 0);
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
    /// O(1) lookup by `PropertyId`.
    id_index: HashMap<PropertyId, usize>,
    /// Index mapping Name -> index in properties vector.
    /// O(1) lookup by `PropertyName`.
    name_index: HashMap<PropertyName, usize>,
    /// Dense storage of properties.
    /// Uses Vec for cache-friendly iteration and index-based access.
    properties: Vec<Property>,
    /// Version counter for staleness detection.
    version: BankVersion,
    /// Domain events pending emission.
    #[serde(skip)]
    pending_events: Option<Vec<Events>>,
}

/// Design note: `PropertyBank` uses a triple-index structure (`id_index`,
/// `name_index`, properties) to optimize different access patterns.
///
/// **Tradeoffs:**
/// - Memory: 3N data structures for N properties (2 `HashMaps` + 1 Vec)
/// - Benefit: O(1) lookups by both ID and name, plus cache-friendly iteration
/// - Alternative: Could use single `HashMap`<`PropertyName`, Property> but
///   would lose O(1) ID lookups needed for $ref resolution in schemas
///
/// This design was chosen because:
/// 1. `PropertyBank` is singleton (only one instance per vault)
/// 2. Property count is bounded (~100s, not 1000s)
/// 3. Both ID and name lookups are frequently needed
/// 4. The memory overhead is acceptable for a singleton
impl PropertyBank {
    /// Create a new empty `PropertyBank`.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::bank::PropertyBank;
    ///
    /// let bank = PropertyBank::new();
    /// assert_eq!(bank.all().count(), 0);
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: PropertyBankId::singleton(),
            id_index: HashMap::new(),
            name_index: HashMap::new(),
            properties: Vec::new(),
            version: BankVersion::initial(),
            pending_events: None,
        }
    }

    /// Build a `PropertyBank` from raw vault data.
    ///
    /// Preserves existing property IDs by name when `existing` is provided.
    /// New properties (not found in existing by name) get generated IDs.
    ///
    /// # Errors
    /// Returns `SchemaError` if any property fails validation.
    ///
    /// # Examples
    /// ```ignore
    /// use lithos_core::schema::{
    ///     bank::PropertyBank,
    ///     raw::RawPropertyBank,
    /// };
    ///
    /// let raw = RawPropertyBank { properties: std::collections::HashMap::new() };
    /// let bank = PropertyBank::from_raw(raw, None)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    #[expect(
        clippy::iter_over_hash_type,
        reason = "Property order in bank is not deterministic; from_raw \
                  preserves insertion order"
    )]
    pub fn from_raw(
        raw: super::raw::RawPropertyBank,
        existing: Option<&Self>,
    ) -> Result<Self, SchemaError> {
        let mut bank = Self::new();

        for (name, entry) in raw.properties {
            let prop_name = PropertyName::new(&name)?;
            let spec = entry.spec.try_into_validated()?;
            let multiplicity = if entry.multi {
                super::property::Multiplicity::Many
            } else {
                super::property::Multiplicity::Single
            };

            // Reuse ID from existing bank if name matches
            let id = existing
                .and_then(|b| b.get_by_name(&prop_name))
                .map_or_else(PropertyId::new, super::property::Property::id);

            let property = Property::new(
                id,
                prop_name,
                super::property::Cardinality::Optional,
                multiplicity,
                spec,
            )?;

            bank.register(property)?;
        }

        Ok(bank)
    }

    /// Returns the `PropertyBank`'s unique identifier.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::bank::PropertyBank;
    ///
    /// let bank = PropertyBank::new();
    /// let _id = bank.id();
    /// ```
    #[inline]
    #[must_use]
    pub const fn id(&self) -> PropertyBankId {
        self.id
    }

    /// Returns the current `PropertyBank` version.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::bank::PropertyBank;
    ///
    /// let bank = PropertyBank::new();
    /// let _version = bank.version();
    /// ```
    #[inline]
    #[must_use]
    pub const fn version(&self) -> BankVersion {
        self.version
    }

    /// Register a property in the bank.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::schema::{
    ///     bank::PropertyBank,
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
        let id = property.id();
        let name = property.name().clone();

        match self.id_index.entry(id) {
            Entry::Occupied(id_entry) => {
                // Verify that the existing property matches the new one
                let idx = *id_entry.get();
                #[expect(
                    clippy::indexing_slicing,
                    reason = "Index guaranteed valid: idx came from id_index \
                              which points to properties Vec. Invariant \
                              maintained by register() logic."
                )]
                let existing = &self.properties[idx];

                // Check if content matches (idempotent case)
                if existing == &property {
                    // Idempotent success: no event, no version increment
                    Ok(())
                } else {
                    // Same ID, different content - this is an error
                    Err(SchemaError::AlreadyExists(format!(
                        "Property ID {} already registered with different \
                         content: existing name={}, new name={}",
                        id,
                        existing.name().as_str(),
                        property.name().as_str()
                    )))
                }
            }
            Entry::Vacant(id_entry) => {
                // Prevent duplicate names
                if self.name_index.contains_key(&name) {
                    return Err(SchemaError::DuplicatePropertyName(
                        name.as_str().into(),
                    ));
                }

                let idx = self.properties.len();
                id_entry.insert(idx);
                self.name_index.insert(name.clone(), idx);
                self.properties.push(property);
                self.version = self.version.increment();

                let event =
                    Events::PropertyRegistered(PropertyRegistered::new(
                        id,
                        &name,
                        super::aggregate::Timestamp::now(),
                    ));
                self.add_event(event);
                Ok(())
            }
        }
    }

    /// Lookup property by ID (O(1)).
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::bank::PropertyBank;
    ///
    /// let bank = PropertyBank::new();
    /// let missing =
    ///     bank.get_by_id(lithos_core::schema::property::PropertyId::new());
    /// assert!(missing.is_none());
    /// ```
    #[inline]
    #[must_use]
    pub fn get_by_id(&self, id: PropertyId) -> Option<&Property> {
        let &idx = self.id_index.get(&id)?;
        self.properties.get(idx)
    }

    /// Lookup property by Name (O(1)).
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::{bank::PropertyBank, property::PropertyName};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bank = PropertyBank::new();
    /// let name = PropertyName::new("flag")?;
    /// let missing = bank.get_by_name(&name);
    /// assert!(missing.is_none());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn get_by_name(&self, name: &PropertyName) -> Option<&Property> {
        let &idx = self.name_index.get(name)?;
        self.properties.get(idx)
    }

    /// Gets a property by name or ID (string).
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::schema::bank::PropertyBank;
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
    /// use lithos_core::schema::bank::PropertyBank;
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
            .ok_or_else(|| SchemaError::PropertyNotFound(key.into()))
    }

    /// Checks if a property exists by ID.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::{bank::PropertyBank, property::PropertyId};
    ///
    /// let bank = PropertyBank::new();
    /// let id = PropertyId::new();
    /// assert!(!bank.has_id(id));
    /// ```
    #[inline]
    #[must_use]
    pub fn has_id(&self, id: PropertyId) -> bool {
        self.id_index.contains_key(&id)
    }

    /// Checks if a property exists by name.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::{bank::PropertyBank, property::PropertyName};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bank = PropertyBank::new();
    /// let name = PropertyName::new("flag")?;
    /// assert!(!bank.has_name(&name));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn has_name(&self, name: &PropertyName) -> bool {
        self.name_index.contains_key(name)
    }

    /// Get all properties in the bank.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::bank::PropertyBank;
    ///
    /// let bank = PropertyBank::new();
    /// let count = bank.all().count();
    /// assert_eq!(count, 0);
    /// ```
    #[inline]
    pub fn all(&self) -> impl Iterator<Item = &Property> {
        self.properties.iter()
    }

    /// Returns a reference to pending domain events.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::bank::PropertyBank;
    ///
    /// let bank = PropertyBank::new();
    /// let _events = bank.pending_events();
    /// ```
    #[inline]
    #[must_use]
    pub fn pending_events(&self) -> &[Events] {
        self.pending_events.as_deref().unwrap_or_default()
    }

    /// Returns and clears pending domain events.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::bank::PropertyBank;
    ///
    /// let mut bank = PropertyBank::new();
    /// let _events = bank.take_events();
    /// ```
    #[inline]
    #[must_use]
    pub fn take_events(&mut self) -> Vec<Events> {
        self.pending_events.take().unwrap_or_default()
    }

    /// Adds a domain event to the pending events collection.
    #[inline]
    fn add_event(&mut self, event: Events) {
        self.pending_events.get_or_insert_with(Vec::new).push(event);
    }
}

impl Default for PropertyBank {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identity for a property bank.
///
/// # Examples
/// ```
/// use lithos_core::schema::bank::PropertyBankId;
///
/// let id = PropertyBankId::new();
/// let _ = id.as_uuid();
/// ```
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
    /// Creates a new UUID v7-based `PropertyBankId`.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::bank::PropertyBankId;
    ///
    /// let id = PropertyBankId::new();
    /// let _ = id.as_uuid();
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Returns the singleton `PropertyBank` ID.
    ///
    /// The `PropertyBank` uses a fixed UUID to act as a singleton registry.
    /// This ensures consistent identity across all program runs.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::bank::PropertyBankId;
    ///
    /// let id = PropertyBankId::singleton();
    /// let _ = id.as_uuid();
    /// ```
    #[inline]
    #[must_use]
    pub const fn singleton() -> Self {
        // Fixed UUID v7 for singleton PropertyBank
        Self(Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0001))
    }

    /// Wraps a UUID into a `PropertyBankId`.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::bank::PropertyBankId;
    /// use uuid::Uuid;
    ///
    /// let uuid = Uuid::now_v7();
    /// let id = PropertyBankId::from_uuid(uuid);
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
    /// use lithos_core::schema::bank::PropertyBankId;
    ///
    /// let id = PropertyBankId::new();
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
    /// use lithos_core::schema::bank::PropertyBankId;
    ///
    /// let id = PropertyBankId::new();
    /// let _uuid = id.into_uuid();
    /// ```
    #[inline]
    #[must_use]
    pub const fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for PropertyBankId {
    #[inline]
    fn default() -> Self {
        Self::singleton()
    }
}

/// `PropertyBank` version counter for staleness detection.
///
/// # Examples
/// ```
/// use lithos_core::schema::bank::BankVersion;
///
/// let version = BankVersion::initial();
/// let next = version.increment();
/// assert!(version.is_older_than(next));
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
pub struct BankVersion(u64);

impl BankVersion {
    /// Returns the initial version.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::bank::BankVersion;
    ///
    /// let version = BankVersion::initial();
    /// assert_eq!(version.as_u64(), 0);
    /// ```
    #[inline]
    #[must_use]
    pub const fn initial() -> Self {
        Self(0)
    }

    /// Returns the next version value.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::bank::BankVersion;
    ///
    /// let version = BankVersion::initial();
    /// let next = version.increment();
    /// assert!(version.is_older_than(next));
    /// ```
    #[inline]
    #[must_use]
    pub const fn increment(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    /// Returns the version as a raw integer.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::bank::BankVersion;
    ///
    /// let version = BankVersion::initial();
    /// assert_eq!(version.as_u64(), 0);
    /// ```
    #[inline]
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns true when this version is older than the other.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::bank::BankVersion;
    ///
    /// let old = BankVersion::initial();
    /// let new = old.increment();
    /// assert!(old.is_older_than(new));
    /// ```
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
            property_spec::{BoolSpec, PropertySpec, StringSpec},
        },
        *,
    };

    mod fixtures {
        use super::*;

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

        /// Test: `rejects_same_id_different_content`.
        /// Verifies that registering a property with the same ID but different
        /// content fails with an error (HIGH-005 fix).
        #[test]
        fn rejects_same_id_different_content() {
            // GIVEN: a PropertyBank with a registered property
            let mut bank = PropertyBank::new();
            let id = PropertyId::from_uuid(TEST_PROPERTY_ID_A);

            let spec1 = PropertySpec::String(StringSpec::default());
            let name1 = PropertyName::new("status").expect("Valid name");
            let prop1 = Property::new(
                id,
                name1,
                Cardinality::Optional,
                Multiplicity::Single,
                spec1,
            )
            .expect("Valid property");

            bank.register(prop1).expect("First registration should succeed");

            // WHEN: attempting to register different content with same ID
            let spec2 = PropertySpec::Bool(BoolSpec::default());
            let name2 = PropertyName::new("priority").expect("Valid name");
            let prop2 = Property::new(
                id,
                name2,
                Cardinality::Required,
                Multiplicity::Many,
                spec2,
            )
            .expect("Valid property");

            let result = bank.register(prop2);

            // THEN: registration should fail
            assert!(
                result.is_err(),
                "Should reject same ID with different content"
            );

            if let Err(SchemaError::AlreadyExists(msg)) = result {
                assert!(
                    msg.contains("already registered with different content"),
                    "Error message should explain the conflict: {msg}"
                );
                assert!(
                    msg.contains("status"),
                    "Error should mention existing name: {msg}"
                );
                assert!(
                    msg.contains("priority"),
                    "Error should mention new name: {msg}"
                );
            } else {
                #[expect(
                    clippy::panic,
                    reason = "Test assertion: Expected specific error variant"
                )]
                {
                    panic!(
                        "Expected SchemaError::AlreadyExists, got: {result:?}"
                    );
                }
            }
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

        // E-01: Concurrency tests

        /// Verify `PropertyBank` implements Send (required for thread-safe
        /// usage). This ensures `PropertyBank` can be moved between
        /// threads.
        #[test]
        fn property_bank_is_send() {
            fn assert_send<T: Send>() {}
            assert_send::<PropertyBank>();
        }

        /// Verify `PropertyBank` implements Sync (required for shared thread
        /// access). This ensures `PropertyBank` can be shared between
        /// threads via &Arc<PropertyBank>.
        #[test]
        fn property_bank_is_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<PropertyBank>();
        }

        /// E-01b: Test concurrent reads from multiple threads.
        /// `PropertyBank` should handle concurrent access without data races.
        #[test]
        fn property_bank_concurrent_reads() {
            use std::{
                sync::{Arc, Mutex},
                thread,
            };

            // Create a bank with some properties
            let (bank, _id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            let bank = Arc::new(bank);
            let error_count = Arc::new(Mutex::new(0usize));

            // Spawn 10 threads that all try to read from the bank
            // Using repeat_with to avoid map_with_unused_argument_over_ranges
            // lint; nested closure required for thread spawn - acceptable for
            // test
            #[expect(
                clippy::excessive_nesting,
                reason = "nested closure required for thread spawn"
            )]
            let handles: Vec<_> = std::iter::repeat_with(|| {
                let bank = Arc::clone(&bank);
                let _error_count = Arc::clone(&error_count);
                thread::spawn(move || {
                    // Each thread tries to access properties multiple times
                    for _ in 0i32..100i32 {
                        // Just read operations - these should be safe
                        let _all = bank.all().count();
                    }
                })
            })
            .take(10)
            .collect();

            // Wait for all threads to complete
            for handle in handles {
                handle.join().expect("Thread join succeeded");
            }
        }
    }
}
