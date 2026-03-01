//! PropertyBank domain aggregate for centralized property registration.
//!
//! Provides name-indexed property lookup with singleton persistence and
//! versioning for incremental resolution.

use std::{collections::BTreeMap, fmt::Display};

use super::{
    error::SchemaError,
    events::{Events, PropertyRegistered},
    property::{Property, PropertyId, PropertyName},
};

/// Registry of reusable Property definitions keyed by name.
///
/// The `PropertyBank` acts as a singleton registry with a stable UUID identity.
/// It is loaded first at program start and versioned for incremental
/// resolution.
///
/// # Storage Strategy
///
/// The `PropertyBank` is a singleton registry persisted by the adapter layer.
/// - **Lifecycle**: Loaded once at startup, persisted on modification
/// - **Versioning**: `BankVersion` increments on any property change
///
/// # Examples
///
/// ```
/// # use lithos_core::schema::bank::PropertyBank;
/// # use lithos_core::schema::property::{
/// #     Multiplicity, Optionality, Property, PropertyId, PropertyName,
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
///     Optionality::Required,
///     Multiplicity::Single,
///     spec,
/// );
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
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct PropertyBank {
    /// Registered properties keyed by name.
    ///
    /// Stored as `BTreeMap` for deterministic iteration order.
    properties: BTreeMap<PropertyName, Property>,
    /// Version counter for staleness detection.
    version: BankVersion,
    /// Domain events pending emission.
    pending_events: Option<Vec<Events>>,
}

/// Design note: `PropertyBank` stores properties by name for consistent
/// dereferencing behavior and deterministic iteration order.
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
            properties: BTreeMap::new(),
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
            let prop_name = PropertyName::try_from(name)?;
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
                super::property::Optionality::Optional,
                multiplicity,
                spec,
            );

            bank.register(property)?;
        }

        Ok(bank)
    }

    /// Reconstruct a `PropertyBank` from stored properties.
    ///
    /// This skips event emission and preserves the provided version.
    pub(crate) fn reconstruct(
        properties: Vec<Property>,
        version: BankVersion,
    ) -> Result<Self, SchemaError> {
        let mut bank = Self::new();
        bank.version = version;

        for property in properties {
            let name = property.name().clone();
            if bank.properties.contains_key(&name) {
                return Err(SchemaError::DuplicatePropertyName(
                    name.as_str().into(),
                ));
            }
            if bank.properties.values().any(|prop| prop.id() == property.id()) {
                return Err(SchemaError::AlreadyExists(format!(
                    "Property ID {} already registered under a different name",
                    property.id()
                )));
            }
            bank.properties.insert(name, property);
        }

        Ok(bank)
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
    ///         Multiplicity, Optionality, Property, PropertyId, PropertyName,
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
    ///     Optionality::Required,
    ///     Multiplicity::Single,
    ///     spec,
    /// );
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

        if let Some(existing) = self.properties.get(&name) {
            if existing == &property {
                return Ok(());
            }
            return Err(SchemaError::DuplicatePropertyName(
                name.as_str().into(),
            ));
        }

        if self.properties.values().any(|prop| prop.id() == id) {
            return Err(SchemaError::AlreadyExists(format!(
                "Property ID {id} already registered under a different name"
            )));
        }

        self.properties.insert(name.clone(), property);
        self.version = self.version.increment();

        let event = Events::PropertyRegistered(PropertyRegistered::new(
            id,
            &name,
            super::aggregate::Timestamp::now(),
        ));
        self.add_event(event);
        Ok(())
    }

    /// Lookup property by name (O(log n)).
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
        self.properties.get(name)
    }

    /// Gets a property by name (string).
    ///
    /// # Errors
    /// Returns `SchemaError::PropertyNotFound` if the property does not exist
    /// or `SchemaError::InvalidPropertyName` if the name is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::schema::bank::PropertyBank;
    ///
    /// let bank = PropertyBank::new();
    ///
    /// assert!(bank.get("any").is_err(), "Missing property should error");
    /// ```
    #[inline]
    pub fn get(&self, key: &str) -> Result<&Property, SchemaError> {
        // Fall back to name lookup
        let name = PropertyName::try_from(key)?;
        self.get_by_name(&name)
            .ok_or_else(|| SchemaError::PropertyNotFound(key.into()))
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
        self.properties.contains_key(name)
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
        self.properties.values()
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
                Multiplicity, Optionality, Property, PropertyId, PropertyName,
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
                Optionality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            );
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
                Optionality::Optional,
                Multiplicity::Single,
                spec,
            );

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
                Optionality::Optional,
                Multiplicity::Single,
                spec1,
            );

            bank.register(prop1).expect("First registration should succeed");

            // WHEN: attempting to register different content with same ID
            let spec2 = PropertySpec::Bool(BoolSpec::default());
            let name2 = PropertyName::new("priority").expect("Valid name");
            let prop2 = Property::new(
                id,
                name2,
                Optionality::Required,
                Multiplicity::Many,
                spec2,
            );

            let result = bank.register(prop2);

            // THEN: registration should fail
            assert!(
                result.is_err(),
                "Should reject same ID with different content"
            );

            assert!(
                matches!(
                    &result,
                    Err(SchemaError::AlreadyExists(msg))
                        if msg.contains("already registered under a different name")
                ),
                "Expected SchemaError::AlreadyExists, got: {result:?}"
            );
        }

        /// 3.3-UNIT-020: `maintains_name_lookup_for_fast_access`.
        /// Priority: P1.
        #[test]
        fn maintains_name_lookup_for_fast_access() {
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
                Optionality::Optional,
                Multiplicity::Single,
                spec1,
            );
            bank.register(prop1).expect("Initial registration should succeed");

            // WHEN: registering a different definition with the same name
            let spec2 = PropertySpec::Bool(BoolSpec::default());
            let prop2 = Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID_B),
                name,
                Optionality::Optional,
                Multiplicity::Single,
                spec2,
            );
            let res = bank.register(prop2);

            // THEN: it must return a DuplicatePropertyName error
            assert!(
                matches!(res, Err(SchemaError::DuplicatePropertyName(_))),
                "Duplicate property name should be rejected with \
                 DuplicatePropertyName, got: {res:?}"
            );
        }

        /// 3.2-UNIT-011: `property_bank_accessors_cover_names`.
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

        /// 3.2-UNIT-011: `property_bank_accessors_cover_names`.
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

        /// 3.2-UNIT-011: `property_bank_accessors_cover_names`.
        /// Priority: P1.
        #[test]
        fn property_bank_gets_by_name_string() {
            let (bank, _id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            let result = bank.get("flag");
            assert!(result.is_ok(), "Get should succeed: {result:?}");
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
