//! PropertyBank domain aggregate for centralized property registration.
//!
//! Provides name-indexed property lookup with singleton persistence.

use std::time::SystemTime;

use rkyv::{Archive, Deserialize, Serialize, with::AsUnixTime};

use super::{
    error::SchemaError,
    property::{Property, PropertyMap, PropertyName},
    raw::RawPropertyBank,
};

/// Registry of reusable Property definitions keyed by name.
///
/// The `PropertyBank` acts as a singleton registry with persisted state.
/// It is loaded first at program start and stored on modification.
///
/// # Storage Strategy
///
/// The `PropertyBank` is a singleton registry persisted by the adapter layer.
/// - **Lifecycle**: Loaded once at startup, persisted on modification
/// - **Storage**: `property_bank` singleton table
///
/// # Examples
///
/// ```
/// # use lithos_core::schema::bank::PropertyBank;
/// # use lithos_core::schema::property::{
/// #     Multiplicity, Optionality, Property, PropertyId, PropertyMap,
/// #     PropertyName,
/// # };
/// # use lithos_core::schema::property_spec::{PropertySpec, BoolSpec};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut bank = PropertyBank::new();
/// let name = PropertyName::try_new("is_active")?;
/// let spec = PropertySpec::Bool(BoolSpec::default());
/// let id = PropertyId::new();
/// let property =
///     Property::new(id, Optionality::Required, Multiplicity::Single, spec);
///
/// let mut properties = PropertyMap::new();
/// properties.insert(name.clone(), property);
/// let bank = PropertyBank::from(properties);
/// assert!(bank.has(&name), "Bank should contain property name");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PropertyBank {
    /// Registered properties keyed by name (`PropertyMap` for O(1) lookup).
    properties: PropertyMap,
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
            properties: PropertyMap::new(),
            recorded_at: SystemTime::now(),
        }
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
        self.properties.has(name)
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

    #[inline]
    pub(super) fn set_properties(&mut self) -> &mut PropertyMap {
        &mut self.properties
    }

    #[inline]
    pub(super) fn set_recorded_at(&mut self) -> &mut SystemTime {
        &mut self.recorded_at
    }
}

impl Default for PropertyBank {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<PropertyMap> for PropertyBank {
    #[inline]
    fn from(properties: PropertyMap) -> Self {
        Self {
            properties,
            recorded_at: SystemTime::now(),
        }
    }
}

impl TryFrom<RawPropertyBank> for PropertyBank {
    type Error = SchemaError;

    /// Build a `PropertyBank` from raw vault data with fresh IDs.
    ///
    /// All properties get newly generated IDs (no ID preservation).
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
        let properties = PropertyMap::try_from(raw.into_properties())?;
        Ok(PropertyBank::from(properties))
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
    use super::{
        super::{
            property::{
                Multiplicity, Optionality, Property, PropertyId, PropertyName,
            },
            property_spec::{BoolSpec, PropertySpec},
        },
        *,
    };

    mod fixtures {
        use super::*;

        pub fn bank_with_property()
        -> Result<(PropertyBank, PropertyId), SchemaError> {
            let mut bank = PropertyBank::new();
            let name = PropertyName::try_new("flag")?;
            let property = Property::new(
                PropertyId::new(),
                Optionality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            );
            let id = property.id();
            bank.properties.insert(name, property);
            Ok((bank, id))
        }
    }

    mod property_bank {
        use super::*;
        use crate::schema::{
            property::{
                Multiplicity, Optionality, Property, PropertyId, PropertyName,
            },
            property_spec::{PropertySpec, StringSpec},
        };

        /// 3.3-UNIT-023: `insert_is_idempotent_on_identical_name`.
        /// Priority: P1.
        #[test]
        fn insert_is_idempotent_on_identical_name() {
            // GIVEN: a PropertyBank and an existing property
            let mut bank = PropertyBank::new();
            let spec = PropertySpec::String(StringSpec::default());
            let name = PropertyName::try_new("test").expect("Valid name");
            let prop = Property::new(
                PropertyId::new(),
                Optionality::Optional,
                Multiplicity::Single,
                spec,
            );

            // WHEN: inserting the same property twice
            bank.properties.insert(name.clone(), prop.clone());
            bank.properties.insert(name, prop);

            // THEN: the count remains 1
            let count = bank.all().count();
            assert_eq!(count, 1, "Expected 1 property after identical inserts");
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
    }
}
