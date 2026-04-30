//! Shared delta computation utilities for schema ingestion.
//!
//! This module centralizes change detection for raw schema/property-bank inputs
//! against previously stored property hashes.
//!
//! Key guarantees:
//! - Hash-based comparison is used consistently for all property maps.
//! - Removed property names are sorted for deterministic downstream behavior.
//! - Excludes deltas are computed with deterministic set-diff semantics.
//! - The module is pure computation (no filesystem or repository I/O).

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::{
    schema::{
        error::{SchemaIngestionError, SchemaLoaderError},
        expander::RefExpander,
        property::{PropertyMap, PropertyName},
        raw::{
            RawPropertyBank, RawPropertyInline, RawPropertyMap, RawSchema,
            property::{RawProperty, RawPropertyBankEntry},
        },
        views::RawPropertyMapHash,
    },
    support::hash::Blake3Hash,
};

type PropertyChangeSetParts<T> =
    (HashMap<PropertyName, T>, Vec<PropertyName>, RawPropertyMapHash);
type PropertyDeltaResult =
    Result<(PropertyDelta, RawPropertyMapHash), SchemaLoaderError>;

/// Delta for schema-level `excludes` lists.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ExcludesDelta {
    /// Property names present in the new excludes list but not the old list.
    added: Vec<PropertyName>,
    /// Property names present in the old excludes list but not the new list.
    removals: Vec<PropertyName>,
}

impl ExcludesDelta {
    /// Creates a new excludes delta with normalized vectors.
    ///
    /// Both `added` and `removals` will be sorted and deduplicated.
    #[inline]
    #[must_use]
    #[expect(dead_code, reason = "API constructor for future use")]
    pub(crate) fn new(
        mut added: Vec<PropertyName>,
        mut removals: Vec<PropertyName>,
    ) -> Self {
        added.sort();
        added.dedup();
        removals.sort();
        removals.dedup();
        Self {
            added,
            removals,
        }
    }

    /// Returns `true` when no excludes changes exist.
    #[inline]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removals.is_empty()
    }

    /// Returns property names that were added to the excludes list.
    #[inline]
    #[must_use]
    #[cfg_attr(not(test), expect(dead_code, reason = "API accessor"))]
    pub(crate) fn added(&self) -> &[PropertyName] {
        &self.added
    }

    /// Returns property names that were removed from the excludes list.
    #[inline]
    #[must_use]
    #[cfg_attr(not(test), expect(dead_code, reason = "API accessor"))]
    pub(crate) fn removals(&self) -> &[PropertyName] {
        &self.removals
    }

    /// Builds an excludes delta from old/new slices.
    ///
    /// Uses ordered-set diffing to keep output deterministic.
    #[inline]
    #[must_use]
    pub(crate) fn from_slices(
        old_excludes: &[PropertyName],
        new_excludes: &[PropertyName],
    ) -> Self {
        let mut old_sorted: Vec<_> = old_excludes.to_vec();
        let mut new_sorted: Vec<_> = new_excludes.to_vec();

        // Sort + dedup (matches PropertyDelta::new() pattern)
        old_sorted.sort();
        old_sorted.dedup();
        new_sorted.sort();
        new_sorted.dedup();

        // Compute added: in new but not in old
        let added: Vec<_> = new_sorted
            .iter()
            .filter(|n| old_sorted.binary_search(n).is_err())
            .cloned()
            .collect();

        // Compute removals: in old but not in new
        let removals: Vec<_> = old_sorted
            .iter()
            .filter(|n| new_sorted.binary_search(n).is_err())
            .cloned()
            .collect();

        Self {
            added,
            removals,
        }
    }
}

/// Unified delta type for property changes in both schemas and property banks.
///
/// Stores resolved `PropertyMap` (domain types) for direct application.
/// Enables true incremental updates without full property re-expansion.
///
/// # Design Rationale
///
/// - `PropertyBankDelta` and `SchemaPropertyDelta` are unified into this type.
/// - Stores `PropertyMap` for direct application to existing schemas.
/// - Enables true incremental update: apply changed properties without full
///   re-expansion.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct PropertyDelta {
    /// New/changed properties (resolved to domain types).
    upserts: PropertyMap,
    /// Removed property names (sorted deterministically).
    removals: Vec<PropertyName>,
}

impl PropertyDelta {
    /// Creates a new property delta with normalized removals.
    ///
    /// The `removals` vector will be sorted and deduplicated.
    #[inline]
    #[must_use]
    pub(crate) fn new(
        upserts: PropertyMap,
        mut removals: Vec<PropertyName>,
    ) -> Self {
        removals.sort();
        removals.dedup();
        Self {
            upserts,
            removals,
        }
    }

    /// Returns `true` when no property changes exist.
    #[inline]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.upserts.is_empty() && self.removals.is_empty()
    }

    /// Returns the upsert map.
    #[inline]
    #[must_use]
    pub(crate) fn upserts(&self) -> &PropertyMap {
        &self.upserts
    }

    /// Returns removed property names (sorted deterministically).
    #[inline]
    #[must_use]
    pub(crate) fn removals(&self) -> &[PropertyName] {
        &self.removals
    }

    /// Returns `true` if the given name has an upsert.
    #[inline]
    #[must_use]
    pub(crate) fn contains_upsert(&self, name: &PropertyName) -> bool {
        self.upserts.has(name)
    }

    /// Returns the union of changed names as a new set (test-only).
    #[cfg(test)]
    #[inline]
    #[must_use]
    pub(crate) fn to_changed_name_set(&self) -> HashSet<PropertyName> {
        let mut names = HashSet::with_capacity(
            self.upserts.len().saturating_add(self.removals.len()),
        );
        names.extend(self.upserts.keys().cloned());
        names.extend(self.removals.iter().cloned());
        names
    }

    /// Consumes self and returns the union of changed names as a new set.
    ///
    /// This takes ownership to avoid cloning the upsert map's keys.
    #[inline]
    #[must_use]
    pub(crate) fn into_changed_name_set(self) -> HashSet<PropertyName> {
        let mut names = HashSet::with_capacity(
            self.upserts.len().saturating_add(self.removals.len()),
        );
        names.extend(self.upserts.into_keys());
        names.extend(self.removals);
        names
    }
}

/// Generic delta engine over a raw property map and previous hash snapshot.
///
/// This is the core engine used by both schema and property-bank delta flows.
pub(crate) struct PropertyDeltaEngine<'data, T> {
    properties: &'data RawPropertyMap<T>,
    previous_hashes: &'data RawPropertyMapHash,
}

impl<'data, T> PropertyDeltaEngine<'data, T> {
    /// Creates an engine from any raw property map.
    #[inline]
    #[must_use]
    fn for_map(
        properties: &'data RawPropertyMap<T>,
        previous_hashes: &'data RawPropertyMapHash,
    ) -> Self {
        Self {
            properties,
            previous_hashes,
        }
    }
}

impl<'data> PropertyDeltaEngine<'data, RawProperty> {
    /// Creates an engine for a raw schema's properties.
    #[inline]
    #[must_use]
    pub(crate) fn for_schema(
        schema: &'data RawSchema,
        previous_hashes: &'data RawPropertyMapHash,
    ) -> Self {
        Self::for_map(schema.properties(), previous_hashes)
    }

    /// Computes a schema-specific property delta using the given ref expander.
    ///
    /// This enables eager resolution of property references during delta
    /// computation, avoiding the need for full property re-expansion during
    /// application.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] when changed entries cannot be converted
    /// into a validated [`PropertyMap`].
    #[inline]
    pub(crate) fn diff_schema(
        &self,
        expander: &RefExpander,
    ) -> Result<PropertyDelta, SchemaLoaderError> {
        let change_set = self.compute_change_set();
        let (raw_upserts, removals, _current_hashes) = change_set.into_parts();

        let mut resolved_upserts = PropertyMap::new();
        let mut inline_map: HashMap<PropertyName, RawPropertyInline> =
            HashMap::new();

        #[expect(
            clippy::iter_over_hash_type,
            reason = "Property map iteration order does not affect delta \
                      semantics"
        )]
        for (name, entry) in raw_upserts {
            match entry {
                RawProperty::Inline(inline) => {
                    inline_map.insert(name.clone(), inline);
                }
                RawProperty::Ref(r#ref) => {
                    let property = expander
                        .expand_property(&r#ref)
                        .map_err(SchemaLoaderError::Resolution)?;
                    resolved_upserts.insert(name, property);
                }
            }
        }

        if !inline_map.is_empty() {
            let converted = PropertyMap::try_from(inline_map)
                .map_err(SchemaLoaderError::Resolution)?;
            for (name, property) in converted {
                resolved_upserts.insert(name, property);
            }
        }

        Ok(PropertyDelta::new(resolved_upserts, removals))
    }
}

impl<'data> PropertyDeltaEngine<'data, RawPropertyBankEntry> {
    /// Creates an engine for a raw property bank's entries.
    #[inline]
    #[must_use]
    pub(crate) fn for_property_bank(
        bank: &'data RawPropertyBank,
        previous_hashes: &'data RawPropertyMapHash,
    ) -> Self {
        Self::for_map(bank.properties(), previous_hashes)
    }

    /// Computes a property-bank delta and current property hashes.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaLoaderError`] when changed entries cannot be converted
    /// into a validated [`PropertyMap`].
    #[inline]
    pub(crate) fn diff_property_bank(&self) -> PropertyDeltaResult {
        let change_set = self.compute_change_set();
        let (raw_upserts, removals, property_hashes) = change_set.into_parts();

        let upserts = PropertyMap::try_from(raw_upserts).map_err(|error| {
            SchemaLoaderError::Ingestion(SchemaIngestionError::Schema {
                path: PathBuf::from("property_bank"),
                source: error,
            })
        })?;

        Ok((PropertyDelta::new(upserts, removals), property_hashes))
    }
}

/// Internal generic change set used by the shared delta engine.
struct PropertyChangeSet<T> {
    upserts: HashMap<PropertyName, T>,
    removals: Vec<PropertyName>,
    current_hashes: RawPropertyMapHash,
}

impl<T> PropertyChangeSet<T> {
    /// Consumes the change set and returns all computed parts.
    #[inline]
    #[must_use]
    fn into_parts(self) -> PropertyChangeSetParts<T> {
        (self.upserts, self.removals, self.current_hashes)
    }
}

impl<T> PropertyDeltaEngine<'_, T>
where
    T: Clone + serde::Serialize + std::fmt::Debug,
{
    /// Computes generic map-level change set results.
    ///
    /// Algorithm:
    /// 1. hash current entries,
    /// 2. record new/changed entries as upserts,
    /// 3. compute removed names from previous hash keys,
    /// 4. sort removals deterministically.
    fn compute_change_set(&self) -> PropertyChangeSet<T> {
        let mut current_hashes = RawPropertyMapHash::default();
        let mut upserts = HashMap::new();

        let entries: Vec<_> = self.properties.iter().collect();
        for (name, entry) in entries {
            let hash = Blake3Hash::compute_json(entry);
            current_hashes.insert(name.clone(), hash);
            if self.previous_hashes.get(name) != Some(&hash) {
                upserts.insert(name.clone(), entry.clone());
            }
        }

        let mut removals = self
            .previous_hashes
            .keys()
            .filter(|name| !current_hashes.contains_key(name))
            .cloned()
            .collect::<Vec<_>>();
        removals.sort();

        PropertyChangeSet {
            upserts,
            removals,
            current_hashes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod fixtures {
        use super::*;
        use crate::schema::raw::RawPropertyBank;

        pub(crate) fn name(s: &str) -> PropertyName {
            PropertyName::try_new(s).expect("valid test property name")
        }

        pub(crate) fn inline_string() -> RawPropertyInline {
            serde_json::from_value(serde_json::json!({"type": "string"}))
                .expect("valid inline string")
        }

        pub(crate) fn property_bank_fixture(
            names: &[&str],
        ) -> (RawPropertyBank, RawPropertyMapHash) {
            let mut properties = serde_json::Map::new();
            let mut hashes = RawPropertyMapHash::default();

            for name_str in names {
                let p_name = name(name_str);
                let entry_value = serde_json::json!({"type": "string"});
                let entry = RawPropertyBankEntry(inline_string());
                hashes
                    .as_inner_mut()
                    .insert(p_name.clone(), Blake3Hash::compute_json(&entry));
                properties.insert((*name_str).to_owned(), entry_value);
            }

            let bank_json = serde_json::json!({
                "$version": "1.0",
                "properties": properties
            });
            let bank: RawPropertyBank =
                serde_json::from_value(bank_json).expect("valid bank fixture");

            (bank, hashes)
        }
    }

    mod excludes_delta {
        use super::{fixtures, *};

        #[test]
        fn should_compute_diff_correctly_when_slices_overlap() {
            let old = vec![fixtures::name("a"), fixtures::name("b")];
            let new = vec![fixtures::name("b"), fixtures::name("c")];

            let delta = ExcludesDelta::from_slices(&old, &new);

            assert_eq!(
                delta.added().len(),
                1,
                "Expected 1 added property, found {:?}",
                delta.added()
            );
            assert_eq!(
                delta.removals().len(),
                1,
                "Expected 1 removed property, found {:?}",
                delta.removals()
            );
            assert_eq!(
                delta
                    .added()
                    .first()
                    .expect("added should have 1 element")
                    .as_str(),
                "c"
            );
            assert_eq!(
                delta
                    .removals()
                    .first()
                    .expect("removals should have 1 element")
                    .as_str(),
                "a"
            );
        }

        #[test]
        fn should_report_is_empty_when_no_changes_exist() {
            let names = vec![fixtures::name("a")];
            let delta = ExcludesDelta::from_slices(&names, &names);
            assert!(
                delta.is_empty(),
                "Delta should be empty when slices match"
            );
        }

        #[test]
        fn should_iterate_all_changed_names() {
            let old = vec![fixtures::name("removed")];
            let new = vec![fixtures::name("added")];
            let delta = ExcludesDelta::from_slices(&old, &new);

            let mut changed: HashSet<_> =
                delta.added().iter().cloned().collect();
            changed.extend(delta.removals().iter().cloned());

            assert!(
                changed.contains(&fixtures::name("removed")),
                "Missing removal"
            );
            assert!(
                changed.contains(&fixtures::name("added")),
                "Missing addition"
            );
        }

        #[test]
        fn should_convert_to_set_correctly() {
            let old = vec![fixtures::name("a")];
            let new = vec![fixtures::name("b")];
            let delta = ExcludesDelta::from_slices(&old, &new);

            let mut set: HashSet<_> = delta.added().iter().cloned().collect();
            set.extend(delta.removals().iter().cloned());

            assert_eq!(
                set.len(),
                2,
                "Set should contain both added and removed"
            );
        }
    }

    mod schema_property_delta {
        use super::{fixtures, *};
        use crate::schema::{
            property::{Multiplicity, Optionality, Property, PropertyId},
            property_spec::{PropertySpec, StringSpec},
        };

        #[test]
        fn should_normalize_removals_when_unsorted_or_duplicate() {
            let upserts = PropertyMap::new();
            let removals = vec![
                fixtures::name("b"),
                fixtures::name("a"),
                fixtures::name("b"),
            ];

            let delta = PropertyDelta::new(upserts, removals);
            let normalized = delta.removals();

            assert_eq!(
                normalized.len(),
                2,
                "Expected deduplicated removals, found {normalized:?}"
            );
            assert_eq!(
                normalized
                    .first()
                    .expect("normalized should have 2 elements")
                    .as_str(),
                "a",
                "Removals should be sorted"
            );
            assert_eq!(
                normalized
                    .get(1)
                    .expect("normalized should have 2 elements")
                    .as_str(),
                "b",
                "Removals should be sorted"
            );
        }

        #[test]
        fn should_detect_upsert_regardless_of_type() {
            let mut upserts = PropertyMap::new();
            let name = fixtures::name("prop");
            let property = Property::new(
                PropertyId::new(),
                Optionality::Optional,
                Multiplicity::Single,
                PropertySpec::String(StringSpec::default()),
            );
            upserts.insert(name.clone(), property);

            let delta = PropertyDelta::new(upserts, Vec::new());
            assert!(delta.contains_upsert(&name));
        }
    }

    mod property_delta {
        use super::{fixtures, *};
        use crate::schema::{
            property::{Multiplicity, Optionality, Property, PropertyId},
            property_spec::{PropertySpec, StringSpec},
        };

        fn property_fixture() -> Property {
            Property::new(
                PropertyId::new(),
                Optionality::Optional,
                Multiplicity::Single,
                PropertySpec::String(StringSpec::default()),
            )
        }

        #[test]
        fn should_support_borrowed_and_owned_set_conversion() {
            let mut map = HashMap::new();
            map.insert(fixtures::name("a"), property_fixture());
            let upserts = PropertyMap::from(map);
            let removals = vec![fixtures::name("b")];

            let delta = PropertyDelta::new(upserts, removals);

            assert!(delta.to_changed_name_set().contains(&fixtures::name("a")));
            assert!(delta.to_changed_name_set().contains(&fixtures::name("b")));

            let into_set = delta.into_changed_name_set();
            assert!(into_set.contains(&fixtures::name("a")));
            assert!(into_set.contains(&fixtures::name("b")));
        }
    }

    mod engine {
        use super::{fixtures, *};

        #[test]
        fn should_detect_changed_entries_via_hash_mismatch() {
            let map_raw = serde_json::json!({
                "prop": {"type": "string"}
            });
            let map: RawPropertyMap<RawProperty> =
                serde_json::from_value(map_raw).expect("valid map");

            // Previous hash for same name but different content (bool type)
            let old_hash = Blake3Hash::compute_json(&serde_json::json!({
                "type": "bool"
            }));
            let mut previous_hashes = RawPropertyMapHash::default();
            previous_hashes
                .as_inner_mut()
                .insert(fixtures::name("prop"), old_hash);

            let engine = PropertyDeltaEngine::for_map(&map, &previous_hashes);
            let change_set = engine.compute_change_set();

            assert!(
                change_set.upserts.contains_key(&fixtures::name("prop")),
                "Changed property should be in upserts"
            );
            assert!(
                change_set.removals.is_empty(),
                "No properties were removed"
            );
        }

        #[test]
        fn should_identify_removals_when_keys_disappear() {
            let map: RawPropertyMap<RawProperty> =
                serde_json::from_value(serde_json::json!({}))
                    .expect("empty map");

            let mut previous_hashes = RawPropertyMapHash::default();
            previous_hashes
                .as_inner_mut()
                .insert(fixtures::name("old"), Blake3Hash::new([0u8; 32]));

            let engine = PropertyDeltaEngine::for_map(&map, &previous_hashes);
            let change_set = engine.compute_change_set();

            assert_eq!(change_set.removals.len(), 1);
            assert_eq!(
                change_set
                    .removals
                    .first()
                    .expect("removals should have 1 element")
                    .as_str(),
                "old"
            );
        }

        #[test]
        fn should_ignore_entries_with_matching_hashes() {
            let entry_json = serde_json::json!({"type": "string"});
            let entry: RawProperty =
                serde_json::from_value(entry_json.clone()).expect("valid");
            let hash = Blake3Hash::compute_json(&entry);

            let map_json = serde_json::json!({
                "stable": entry_json
            });
            let map: RawPropertyMap<RawProperty> =
                serde_json::from_value(map_json).expect("valid map");

            let mut previous_hashes = RawPropertyMapHash::default();
            previous_hashes.insert(fixtures::name("stable"), hash);

            let engine = PropertyDeltaEngine::for_map(&map, &previous_hashes);
            let change_set = engine.compute_change_set();

            assert!(
                change_set.upserts.is_empty(),
                "Unchanged property should not be in upserts"
            );
        }

        #[test]
        fn should_convert_bank_entries_to_validated_properties() {
            let (bank, hashes) = fixtures::property_bank_fixture(&["a"]);

            // Force a change in "a" by providing an empty previous hash set
            let empty_hashes = RawPropertyMapHash::default();
            let engine =
                PropertyDeltaEngine::for_property_bank(&bank, &empty_hashes);
            let result =
                engine.diff_property_bank().expect("diff should succeed");

            let (delta, new_hashes) = result;
            assert!(delta.upserts().has(&fixtures::name("a")));
            assert!(new_hashes.contains_key(&fixtures::name("a")));
            assert_eq!(
                new_hashes.get(&fixtures::name("a")),
                hashes.get(&fixtures::name("a"))
            );
        }
    }
}
