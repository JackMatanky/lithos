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

use std::collections::{BTreeSet, HashMap, HashSet};

use crate::schema::{
    error::{SchemaIngestionError, SchemaLoaderError},
    property::{PropertyMap, PropertyName},
    raw::{
        RawPropertyBank, RawSchema,
        property::{
            RawProperty, RawPropertyBankEntry, RawPropertyInline,
            RawPropertyMap, RawPropertyRef,
        },
    },
    views::metadata::HashMetadata,
};

type PropertyHashes = HashMap<PropertyName, [u8; 32]>;
type PropertyDiffParts<T> =
    (HashMap<PropertyName, T>, Vec<PropertyName>, PropertyHashes);
type PropertyBankDiffResult =
    Result<(PropertyBankDelta, PropertyHashes), SchemaLoaderError>;

/// Delta for schema-level `excludes` lists.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ExcludesDelta {
    /// Property names present in the new excludes list but not the old list.
    added: Vec<PropertyName>,
    /// Property names present in the old excludes list but not the new list.
    removed: Vec<PropertyName>,
}

impl ExcludesDelta {
    /// Returns `true` when no excludes changes exist.
    #[inline]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }

    /// Returns the union of all changed exclude names.
    #[inline]
    #[must_use]
    pub(crate) fn changed_names(&self) -> HashSet<PropertyName> {
        let mut changed = HashSet::with_capacity(
            self.added.len().saturating_add(self.removed.len()),
        );
        changed.extend(self.added.iter().cloned());
        changed.extend(self.removed.iter().cloned());
        changed
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
        let old_set: BTreeSet<PropertyName> =
            old_excludes.iter().cloned().collect();
        let new_set: BTreeSet<PropertyName> =
            new_excludes.iter().cloned().collect();

        let added = new_set.difference(&old_set).cloned().collect();
        let removed = old_set.difference(&new_set).cloned().collect();

        Self {
            added,
            removed,
        }
    }
}

/// Typed property upserts for schema raw properties.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct SchemaPropertyUpserts {
    /// Inline property definitions that are new or changed.
    inline: HashMap<PropertyName, RawPropertyInline>,
    /// Property-bank references that are new or changed.
    refs: HashMap<PropertyName, RawPropertyRef>,
}

/// Delta for schema properties.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct SchemaPropertyDelta {
    /// New/changed properties split by raw variant.
    upserts: SchemaPropertyUpserts,
    /// Removed property names.
    removed: Vec<PropertyName>,
}

impl SchemaPropertyDelta {
    #[inline]
    #[must_use]
    pub(crate) fn from_parts(
        upserts: SchemaPropertyUpserts,
        removed: Vec<PropertyName>,
    ) -> Self {
        Self {
            upserts,
            removed,
        }
    }

    /// Returns `true` when no property changes exist.
    #[inline]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.upserts.inline.is_empty()
            && self.upserts.refs.is_empty()
            && self.removed.is_empty()
    }

    #[inline]
    #[must_use]
    pub(crate) fn removed(&self) -> &[PropertyName] {
        &self.removed
    }

    #[inline]
    #[must_use]
    pub(crate) fn is_upsert_name(&self, name: &PropertyName) -> bool {
        self.upserts.inline.contains_key(name)
            || self.upserts.refs.contains_key(name)
    }
}

/// Delta for property-bank entries.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PropertyBankDelta {
    /// New/changed property-bank entries.
    upserts: PropertyMap,
    /// Names for changed entries (added/updated), retained for quick set
    /// builds.
    upsert_names: Vec<PropertyName>,
    /// Removed property names.
    removals: Vec<PropertyName>,
}

impl PropertyBankDelta {
    /// Returns `true` when no property-bank changes exist.
    #[inline]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.upserts.is_empty() && self.removals.is_empty()
    }

    /// Returns changed/added entries.
    #[inline]
    #[must_use]
    pub(crate) fn upserts(&self) -> &PropertyMap {
        &self.upserts
    }

    /// Returns the union of changed names (upserts + removals).
    #[inline]
    #[must_use]
    pub(crate) fn changed_names(&self) -> HashSet<PropertyName> {
        let mut names = HashSet::with_capacity(
            self.upsert_names.len().saturating_add(self.removals.len()),
        );
        names.extend(self.upsert_names.iter().cloned());
        names.extend(self.removals.iter().cloned());
        names
    }

    /// Returns removed property names.
    ///
    /// Names are sorted deterministically.
    #[inline]
    #[must_use]
    pub(crate) fn removals(&self) -> &[PropertyName] {
        &self.removals
    }
}

/// Generic differ over a raw property map and previous hash snapshot.
///
/// This is the core engine used by both schema and property-bank delta flows.
pub(crate) struct PropertyDiffer<'data, T> {
    properties: &'data RawPropertyMap<T>,
    previous_hashes: &'data PropertyHashes,
}

impl<'data, T> PropertyDiffer<'data, T> {
    /// Creates a differ from any raw property map.
    #[inline]
    #[must_use]
    fn for_map(
        properties: &'data RawPropertyMap<T>,
        previous_hashes: &'data PropertyHashes,
    ) -> Self {
        Self {
            properties,
            previous_hashes,
        }
    }
}

impl<'data> PropertyDiffer<'data, RawProperty> {
    /// Creates a differ for a raw schema's properties.
    #[inline]
    #[must_use]
    pub(crate) fn for_schema(
        schema: &'data RawSchema,
        previous_hashes: &'data PropertyHashes,
    ) -> Self {
        Self::for_map(schema.properties(), previous_hashes)
    }

    /// Computes a schema-specific property delta.
    #[inline]
    #[must_use]
    pub(crate) fn diff_schema(&self) -> SchemaPropertyDelta {
        let property_diff = self.diff();
        let (upserts, removed, _current_hashes) = property_diff.into_parts();
        let mut typed_upserts = SchemaPropertyUpserts::default();

        let mut upsert_entries: Vec<_> = upserts.into_iter().collect();
        upsert_entries.sort_by(|left, right| left.0.cmp(&right.0));
        for (name, entry) in upsert_entries {
            match entry {
                RawProperty::Inline(inline) => {
                    typed_upserts.inline.insert(name, inline);
                }
                RawProperty::Ref(r#ref) => {
                    typed_upserts.refs.insert(name, r#ref);
                }
            }
        }

        SchemaPropertyDelta::from_parts(typed_upserts, removed)
    }
}

impl<'data> PropertyDiffer<'data, RawPropertyBankEntry> {
    /// Creates a differ for a raw property bank's entries.
    #[inline]
    #[must_use]
    pub(crate) fn for_property_bank(
        bank: &'data RawPropertyBank,
        previous_hashes: &'data PropertyHashes,
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
    pub(crate) fn diff_property_bank(&self) -> PropertyBankDiffResult {
        let property_diff = self.diff();
        let (upserts, removals, property_hashes) = property_diff.into_parts();
        let upsert_names = upserts.keys().cloned().collect::<Vec<_>>();

        let upserts = PropertyMap::try_from(upserts).map_err(|error| {
            SchemaLoaderError::Ingestion(SchemaIngestionError::Schema {
                path: std::path::PathBuf::from("property_bank"),
                source: error,
            })
        })?;

        Ok((
            PropertyBankDelta {
                upserts,
                upsert_names,
                removals,
            },
            property_hashes,
        ))
    }
}

/// Internal generic diff payload used by the shared diff engine.
#[derive(Debug, Clone, PartialEq)]
struct PropertyDiff<T> {
    upserts: HashMap<PropertyName, T>,
    removed: Vec<PropertyName>,
    current_hashes: HashMap<PropertyName, [u8; 32]>,
}

impl<T> PropertyDiff<T> {
    /// Consumes the diff and returns all computed parts.
    #[inline]
    #[must_use]
    fn into_parts(self) -> PropertyDiffParts<T> {
        (self.upserts, self.removed, self.current_hashes)
    }
}

impl<T> PropertyDiffer<'_, T>
where
    T: Clone + serde::Serialize + std::fmt::Debug,
{
    /// Computes generic map-level diff results.
    ///
    /// Algorithm:
    /// 1. hash current entries,
    /// 2. record new/changed entries as upserts,
    /// 3. compute removed names from previous hash keys,
    /// 4. sort removals deterministically.
    fn diff(&self) -> PropertyDiff<T> {
        let mut current_hashes = HashMap::with_capacity(
            self.properties.len().max(self.previous_hashes.len()),
        );
        let mut upserts = HashMap::new();

        for (name, entry) in self.properties {
            let hash = HashMetadata::hash_entry(entry);
            current_hashes.insert(name.clone(), hash);
            if self.previous_hashes.get(name) != Some(&hash) {
                upserts.insert(name.clone(), entry.clone());
            }
        }

        let mut removed = self
            .previous_hashes
            .keys()
            .filter(|name| !current_hashes.contains_key(*name))
            .cloned()
            .collect::<Vec<_>>();
        removed.sort();

        PropertyDiff {
            upserts,
            removed,
            current_hashes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{
        raw::{RawFileTimes, RawPropertyBank},
        views::HashMetadata,
    };

    #[test]
    fn excludes_delta_is_deterministic_and_tracks_changes() {
        let old = vec![
            PropertyName::try_new("a").expect("valid property name"),
            PropertyName::try_new("b").expect("valid property name"),
        ];
        let new = vec![
            PropertyName::try_new("b").expect("valid property name"),
            PropertyName::try_new("c").expect("valid property name"),
        ];

        let delta = ExcludesDelta::from_slices(&old, &new);

        assert_eq!(delta.added.len(), 1);
        assert_eq!(delta.removed.len(), 1);
        assert_eq!(delta.added.first().map(PropertyName::as_str), Some("c"));
        assert_eq!(delta.removed.first().map(PropertyName::as_str), Some("a"));
        assert!(delta.changed_names().contains(
            &PropertyName::try_new("a").expect("valid property name")
        ));
    }

    #[test]
    fn property_differ_supports_raw_property_map() {
        let map: RawPropertyMap<RawProperty> = serde_json::from_str(
            r##"{
              "flag": {"type": "bool"},
              "name": {"$ref": "#property_bank/name"}
            }"##,
        )
        .expect("valid raw property map");

        let previous_hashes = HashMap::new();
        let diff = PropertyDiffer::for_map(&map, &previous_hashes).diff();
        let (upserts, removed, hashes) = diff.into_parts();
        assert_eq!(upserts.len(), 2);
        assert!(removed.is_empty());
        assert_eq!(hashes.len(), 2);
    }

    #[test]
    fn property_differ_supports_raw_property_bank() {
        let bank: RawPropertyBank =
            serde_json::from_value::<RawPropertyBank>(serde_json::json!({
                "$version": "1.0",
                "properties": {
                    "title": {"type": "string"}
                }
            }))
            .expect("valid property bank")
            .with_file_times(RawFileTimes {
                created_at: None,
                modified_at: None,
            });

        let previous_hashes = HashMap::new();
        let diff =
            PropertyDiffer::for_property_bank(&bank, &previous_hashes).diff();
        let (upserts, removed, hashes) = diff.into_parts();
        assert_eq!(upserts.len(), 1);
        assert!(removed.is_empty());
        assert_eq!(hashes.len(), 1);
    }

    #[test]
    fn diff_properties_reuses_property_differ_core() {
        let raw: RawSchema =
            serde_json::from_value::<RawSchema>(serde_json::json!({
                "$version": "1.0",
                "properties": {
                    "title": {"type": "string"}
                }
            }))
            .expect("valid schema")
            .with_name("note".into())
            .with_file_times(RawFileTimes {
                created_at: None,
                modified_at: None,
            });

        let mut previous_hashes = HashMap::new();
        previous_hashes.insert(
            PropertyName::try_new("title").expect("valid property name"),
            HashMetadata::hash_entry(&serde_json::json!({"type":"bool"})),
        );

        let delta =
            PropertyDiffer::for_schema(&raw, &previous_hashes).diff_schema();
        assert!(!delta.is_empty());
        assert_eq!(delta.upserts.inline.len(), 1);
        assert!(delta.upserts.refs.is_empty());
    }
}
