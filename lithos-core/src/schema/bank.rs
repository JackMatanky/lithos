//! PropertyBank domain aggregate for centralized property registration.
//!
//! Provides name-indexed property lookup with singleton persistence and
//! versioning for incremental resolution.

use std::{collections::BTreeMap, fmt::Display, time::SystemTime};

use rkyv::{Archive, Deserialize, Serialize, with::AsUnixTime};

use super::{
    error::SchemaError,
    property::{Multiplicity, Optionality, Property, PropertyId, PropertyName},
    raw::RawPropertyBank,
};

/// Registry of reusable Property definitions keyed by name.
///
/// The `PropertyBank` acts as a singleton registry with versioned persistence.
/// It is loaded first at program start and versioned for incremental
/// resolution.
///
/// # Storage Strategy
///
/// The `PropertyBank` is a singleton registry persisted by the adapter layer.
/// - **Lifecycle**: Loaded once at startup, persisted on modification
/// - **Versioning**: `BankVersion` increments on any property change
/// - **Storage**: `bank_metadata` and versioned `bank_property_by_*` tables
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
/// let name = PropertyName::try_new("is_active")?;
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
/// assert!(bank.has(&name), "Bank should contain property name");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PropertyBank {
    /// Registered properties keyed by name.
    ///
    /// Stored as `BTreeMap` for deterministic iteration order.
    properties: BTreeMap<PropertyName, Property>,
    /// Version counter for staleness detection.
    version: BankVersion,
    /// Ingestion timestamp (private - not exposed in public API).
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

/// Design note: `PropertyBank` stores properties by name for consistent
/// ref-expansion behavior and deterministic iteration order.
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
            recorded_at: SystemTime::now(),
        }
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
    /// let name = PropertyName::try_new("is_active")?;
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

        Ok(())
    }

    /// Lookup property by name (O(log n)).
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::{bank::PropertyBank, property::PropertyName};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bank = PropertyBank::new();
    /// let name = PropertyName::try_new("flag")?;
    /// let missing = bank.get(&name);
    /// assert!(missing.is_none());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn get(&self, name: &PropertyName) -> Option<&Property> {
        self.properties.get(name)
    }

    /// Checks if a property exists by name.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::{bank::PropertyBank, property::PropertyName};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bank = PropertyBank::new();
    /// let name = PropertyName::try_new("flag")?;
    /// assert!(!bank.has(&name));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn has(&self, name: &PropertyName) -> bool {
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

    /// Update specific properties from raw data based on changed property
    /// names.
    ///
    /// This method performs incremental updates to the `PropertyBank` by:
    /// - Updating properties that exist in both raw and changed list
    /// - Adding new properties that appear in changed list
    /// - Removing properties in changed list that don't exist in raw
    /// - Preserving IDs for properties that already exist
    ///
    /// The version counter is incremented only if changes were actually made.
    ///
    /// Update properties incrementally from raw property bank.
    ///
    /// Only processes the properties specified in `changed`.
    /// More efficient than rebuilding entire bank from scratch when
    /// only a few properties changed.
    ///
    /// Used by the ingestor during incremental property bank updates.
    ///
    /// # Examples
    /// ```ignore
    /// use lithos_core::schema::{
    ///     bank::PropertyBank,
    ///     property::PropertyName,
    ///     raw::RawPropertyBank,
    /// };
    ///
    /// let mut bank = PropertyBank::new();
    /// let raw = load_raw_property_bank()?;
    /// let changed = vec![PropertyName::try_new("title")?];
    ///
    /// bank.update_from_raw(&raw, &changed)?;
    /// ```
    ///
    /// # Errors
    /// Returns `SchemaError` if any property validation fails.
    #[inline]
    pub fn update_from_raw(
        &mut self,
        raw: &RawPropertyBank,
        changed: &[PropertyName],
    ) -> Result<(), SchemaError> {
        if changed.is_empty() {
            return Ok(());
        }

        let mut any_changed = false;

        for name in changed {
            if let Some(raw_entry) = raw.properties.get(name.as_ref()) {
                // Property exists in raw - update or insert
                let spec = raw_entry.spec.clone().try_into()?;
                let multiplicity = if raw_entry.multi {
                    Multiplicity::Many
                } else {
                    Multiplicity::Single
                };

                // Preserve ID if property already exists, otherwise create new
                let id =
                    self.get(name).map_or_else(PropertyId::new, Property::id);

                let property = Property::new(
                    id,
                    name.clone(),
                    Optionality::Optional,
                    multiplicity,
                    spec,
                );

                self.properties.insert(name.clone(), property);
                any_changed = true;
            } else {
                // Property was removed from raw
                if self.properties.remove(name).is_some() {
                    any_changed = true;
                }
            }
        }

        if any_changed {
            self.version = self.version.increment();
            self.recorded_at = SystemTime::now();
        }

        Ok(())
    }
}

impl Default for PropertyBank {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl TryFrom<RawPropertyBank> for PropertyBank {
    type Error = SchemaError;

    /// Build a `PropertyBank` from raw vault data with fresh IDs.
    ///
    /// All properties get newly generated IDs (no ID preservation).
    /// Properties are loaded in deterministic name order.
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
    /// use std::convert::TryFrom;
    ///
    /// // RawPropertyBank is non-exhaustive, so this is demonstration only
    /// let raw = load_raw_property_bank()?;
    /// let bank = PropertyBank::try_from(raw)?;
    /// ```
    #[inline]
    fn try_from(raw: RawPropertyBank) -> Result<Self, Self::Error> {
        let mut bank = Self::new();

        let mut entries: Vec<_> = raw.properties.into_iter().collect();
        entries.sort_by(|left, right| left.0.cmp(&right.0));

        for (name, entry) in entries {
            let prop_name = PropertyName::try_from(name)?;
            let spec = entry.spec.try_into()?;
            let multiplicity = if entry.multi {
                Multiplicity::Many
            } else {
                Multiplicity::Single
            };

            let property = Property::new(
                PropertyId::new(),
                prop_name,
                Optionality::Optional,
                multiplicity,
                spec,
            );

            bank.register(property)?;
        }

        Ok(bank)
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
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug))]
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

    /// Constructs a version from a raw integer.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::bank::BankVersion;
    ///
    /// let version = BankVersion::from_u64(5);
    /// assert_eq!(version.as_u64(), 5);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_u64(value: u64) -> Self {
        Self(value)
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
                PropertyName::try_new("flag")?,
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
        use std::collections::HashMap;

        use super::*;
        use crate::schema::raw::{
            RawPropertyBank, RawSchemaMetadata, RawSchemaVersion,
            property::RawPropertyBankEntry,
            property_spec::{RawPropertySpec, RawStringSpec},
        };

        /// 3.3-UNIT-023: `is_idempotent_on_identical_registration`.
        /// Priority: P1.
        #[test]
        fn is_idempotent_on_identical_registration() {
            // GIVEN: a PropertyBank and an existing property
            let mut bank = PropertyBank::new();
            let spec = PropertySpec::String(StringSpec::default());
            let name = PropertyName::try_new("test").expect("Valid name");
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
            let name1 = PropertyName::try_new("status").expect("Valid name");
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
            let name2 = PropertyName::try_new("priority").expect("Valid name");
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

            let name = PropertyName::try_new("flag").expect("Valid name");

            assert!(
                bank.get(&name).is_some(),
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
            let name = PropertyName::try_new("test").expect("Valid name");
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

        /// 3.3-UNIT-025: `update_from_raw_preserves_ids_by_name`.
        /// Priority: P1.
        #[test]
        fn update_from_raw_preserves_ids_by_name() -> Result<(), SchemaError> {
            let mut bank = PropertyBank::new();
            let name = PropertyName::try_new("status")?;
            let prop = Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID_A),
                name.clone(),
                Optionality::Optional,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            );
            bank.register(prop)?;

            let mut properties = HashMap::new();
            properties.insert("status".into(), RawPropertyBankEntry {
                multi: false,
                spec: RawPropertySpec::String(RawStringSpec::default()),
            });

            let raw = RawPropertyBank {
                version: "1.0".into(),
                properties,
                metadata: crate::schema::raw::RawSchemaMetadata::default(),
            };

            // Use update_from_raw to update only the "status" property
            let changed = vec![name.clone()];
            bank.update_from_raw(&raw, &changed)?;

            let updated_prop =
                bank.get(&name).expect("Expected updated property");

            let expected_id = PropertyId::from_uuid(TEST_PROPERTY_ID_A);
            if updated_prop.id() != expected_id {
                return Err(SchemaError::ValidationFailed(format!(
                    "Expected ID {expected_id}, got {}",
                    updated_prop.id()
                )));
            }

            Ok(())
        }

        /// 3.2-UNIT-011: `property_bank_accessors_cover_names`.
        /// Priority: P1.
        #[test]
        fn property_bank_has() {
            let (bank, _id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            let name = PropertyName::try_new("flag").expect("Valid name");

            assert!(
                bank.has(&name),
                "PropertyBank should contain property by name 'flag'"
            );
        }

        /// 3.2-UNIT-011: `property_bank_accessors_cover_names`.
        /// Priority: P1.
        #[test]
        fn property_bank_get() {
            let (bank, _id) = fixtures::bank_with_property()
                .expect("Valid property bank fixture");

            let name = PropertyName::try_new("flag").expect("Valid name");

            let result = bank.get(&name);
            assert!(result.is_some(), "Get should succeed: {result:?}");
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

        /// Test: `update_from_raw()` incrementally updates changed properties.
        #[test]
        fn update_from_raw_incremental_update() {
            let mut bank = PropertyBank::new();

            // Initial property bank
            let mut raw_properties = HashMap::new();
            raw_properties.insert("title".into(), RawPropertyBankEntry {
                spec: RawPropertySpec::String(RawStringSpec::default()),
                multi: false,
            });
            raw_properties.insert("status".into(), RawPropertyBankEntry {
                spec: RawPropertySpec::String(RawStringSpec::default()),
                multi: false,
            });

            let raw = RawPropertyBank {
                version: RawSchemaVersion::default(),
                properties: raw_properties.clone(),
                metadata: RawSchemaMetadata::default(),
            };

            // Initial load
            let changed = vec![
                PropertyName::try_new("title").unwrap(),
                PropertyName::try_new("status").unwrap(),
            ];
            bank.update_from_raw(&raw, &changed).expect("should update");

            assert_eq!(bank.all().count(), 2);

            // Modify one property
            raw_properties.insert("title".into(), RawPropertyBankEntry {
                spec: RawPropertySpec::String(RawStringSpec::default()),
                multi: true, // Changed to multi
            });

            let raw_updated = RawPropertyBank {
                version: RawSchemaVersion::default(),
                properties: raw_properties,
                metadata: RawSchemaMetadata::default(),
            };

            // Incremental update (only title changed)
            let changed_props = vec![PropertyName::try_new("title").unwrap()];
            bank.update_from_raw(&raw_updated, &changed_props)
                .expect("should update incrementally");

            assert_eq!(bank.all().count(), 2);

            // Verify title was updated
            let title = bank.get(&PropertyName::try_new("title").unwrap());
            assert!(title.is_some());
            assert_eq!(title.unwrap().multiplicity(), Multiplicity::Many);
        }

        /// Test: `update_from_raw()` with empty changed list is no-op.
        #[test]
        fn update_from_raw_empty_changed_is_noop() {
            let mut bank = PropertyBank::new();

            let mut raw_properties = HashMap::new();
            raw_properties.insert("title".into(), RawPropertyBankEntry {
                spec: RawPropertySpec::String(RawStringSpec::default()),
                multi: false,
            });

            let raw = RawPropertyBank {
                version: RawSchemaVersion::default(),
                properties: raw_properties,
                metadata: RawSchemaMetadata::default(),
            };

            let version_before = bank.version();

            // Update with empty changed list
            bank.update_from_raw(&raw, &[]).expect("should succeed");

            assert_eq!(bank.version(), version_before);
            assert_eq!(bank.all().count(), 0);
        }

        /// Test: `update_from_raw()` handles property removal.
        #[test]
        fn update_from_raw_removes_deleted_properties() {
            let mut bank = PropertyBank::new();

            // Initial property bank with two properties
            let mut raw_properties = HashMap::new();
            raw_properties.insert("title".into(), RawPropertyBankEntry {
                spec: RawPropertySpec::String(RawStringSpec::default()),
                multi: false,
            });
            raw_properties.insert("status".into(), RawPropertyBankEntry {
                spec: RawPropertySpec::String(RawStringSpec::default()),
                multi: false,
            });

            let raw = RawPropertyBank {
                version: RawSchemaVersion::default(),
                properties: raw_properties,
                metadata: RawSchemaMetadata::default(),
            };

            let changed = vec![
                PropertyName::try_new("title").unwrap(),
                PropertyName::try_new("status").unwrap(),
            ];
            bank.update_from_raw(&raw, &changed).expect("should update");

            assert_eq!(bank.all().count(), 2);

            // Remove status property
            let mut raw_properties_updated = HashMap::new();
            raw_properties_updated.insert(
                "title".into(),
                RawPropertyBankEntry {
                    spec: RawPropertySpec::String(RawStringSpec::default()),
                    multi: false,
                },
            );

            let raw_updated = RawPropertyBank {
                version: RawSchemaVersion::default(),
                properties: raw_properties_updated,
                metadata: RawSchemaMetadata::default(),
            };

            // Update with status in changed list (it was removed from raw)
            let changed_props = vec![PropertyName::try_new("status").unwrap()];
            bank.update_from_raw(&raw_updated, &changed_props)
                .expect("should update");

            assert_eq!(bank.all().count(), 1);
            assert!(
                bank.get(&PropertyName::try_new("title").unwrap()).is_some()
            );
            assert!(
                bank.get(&PropertyName::try_new("status").unwrap()).is_none()
            );
        }
    }
}
