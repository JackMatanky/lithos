# Property Delta Unification Plan

## Executive Summary

This plan implements a unified `PropertyDelta` type to replace both `SchemaPropertyDelta` and `PropertyBankDelta`, enabling **true incremental updates** for schemas by eagerly resolving property references during delta computation rather than deferring resolution to application time.

## Problem Statement

### Current Architecture Issues

1.  **Duplicate Delta Types**: Both `PropertyBankDelta` and `SchemaPropertyDelta` track property changes, but store different data types:
    *   `PropertyBankDelta` stores resolved `PropertyMap` (domain types).
    *   `SchemaPropertyDelta` stores raw `RawPropertyInline` and `RawPropertyRef` (unresolved types).
2.  **Broken Incremental Updates**: In `schema_processor.rs`, even when only 1 property changes, the system:
    *   Detects the change (correctly).
    *   Stores only that 1 property in the delta as a raw type.
    *   Then re-expands **ALL** properties for the schema (`expanded_by_id`, lines 2287-2326).
    *   Filters the full expansion using the delta's changed-names filter.
    *   **Result**: No actual performance benefit from incremental updates.
3.  **Unnecessary Type Alias**: `type PropertyHashes = RawPropertyMapHash` at `delta.rs:30` adds indirection for minimal benefit.

### Root Cause

`SchemaPropertyDelta` was designed with the assumption that property references need resolution later. However, the resolution context (`RefExpander` and `PropertyBank`) IS available during the analysis phase when `diff_schema()` is called. Eager resolution was always possible but never implemented.

---

## Design

### Unified `PropertyDelta` Struct

Location: `lithos-core/src/schema/delta.rs`

```rust
/// Unified delta type for property changes in both schemas and property banks.
///
/// # Design Rationale
///
/// - Stores `PropertyMap` (domain types) for direct application.
/// - Enables true incremental updates without full property re-expansion.
/// - Unifies the delta API across both processors.
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
    pub(crate) fn new(upserts: PropertyMap, mut removals: Vec<PropertyName>) -> Self {
        removals.sort();
        removals.dedup();
        Self { upserts, removals }
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
        self.upserts.contains_key(name)
    }

    /// Returns an iterator over changed names (upsert entries + removals).
    #[inline]
    pub(crate) fn iter_changed(&self) -> impl Iterator<Item = &PropertyName> {
        self.upserts.keys().chain(self.removals.iter())
    }

    /// Returns the union of changed names as a new set.
    ///
    /// This allocates a new `HashSet` on each call.
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
        names.extend(self.removals.into_iter());
        names
    }
}
```

### Shared Result Type for Property Bank

The property bank processor needs to return both the delta AND the new property hashes (for caching). We preserve this behavior with:

```rust
type PropertyDeltaResult = Result<(PropertyDelta, RawPropertyMapHash), SchemaLoaderError>;
```

### Engine API Changes

#### `diff_schema()` New Signature

```rust
/// Computes a schema-specific property delta using the given ref expander.
///
/// # Errors
///
/// Returns [`SchemaLoaderError`] when changed entries cannot be converted
/// into a validated [`PropertyMap`].
pub(crate) fn diff_schema(
    &self,
    expander: &RefExpander,
) -> Result<PropertyDelta, SchemaLoaderError> {
    let change_set = self.compute_change_set();
    let (raw_upserts, removals, current_hashes) = change_set.into_parts();

    // Eagerly resolve all raw upserts to domain types
    let mut resolved_upserts = PropertyMap::new();
    for (name, entry) in raw_upserts {
        let property = match entry {
            RawProperty::Inline(inline) => {
                Property::try_from(inline).map_err(|e| {
                    SchemaLoaderError::Resolution(e.into())
                })?
            }
            RawProperty::Ref(r#ref) => {
                expander.expand_property(r#ref).map_err(|e| {
                    SchemaLoaderError::Resolution(e)
                })?
            }
        };
        resolved_upserts.insert(name, property);
    }

    Ok(PropertyDelta::new(resolved_upserts, removals))
}
```

---

## Implementation Phases

### Phase 1: Cleanup and Type Definition

**Location**: `lithos-core/src/schema/delta.rs`

1.  **Remove the alias** (Line 30):
    *   Delete: `type PropertyHashes = RawPropertyMapHash;`
2.  **Replace usages** (6 total):
    *   Line 31-32: `PropertyChangeSetParts<T>` type alias
    *   Line 33-34: `PropertyBankDeltaResult` type alias
    *   Line 313: `PropertyDeltaEngine` struct field
    *   Line 323: `for_map` method argument
    *   Line 337: `for_schema` method argument
    *   Line 375: `for_property_bank` method argument
    *   Line 406: `PropertyChangeSet` struct field
3.  **Define `PropertyDelta`** (after Line 113, near other public types):
    *   Implement the struct and all methods defined in the Design section above.
4.  **Update Engine Methods**:
    *   `diff_property_bank` should return `PropertyDeltaResult`.
    *   `diff_schema` should accept `&RefExpander` and return `Result<PropertyDelta, SchemaLoaderError>`.
5.  **Delete obsolete types** (after Phase 2):
    *   `SchemaPropertyUpserts` (Lines 117-163)
    *   `SchemaPropertyDelta` (Lines 166-221)
    *   `PropertyBankDelta` (Lines 225-306)

### Phase 2: Update `RefExpander` API

**Location**: `lithos-core/src/schema/expander.rs`

1.  **Expose single-property expansion** (Line 90):
    *   Change `fn expand_property` visibility from private to `pub(crate)`.

### Phase 3: Update Property Bank Processor

**Location**: `lithos-core/src/schema/property_bank_processor.rs`

1.  **Update import**:
    *   Change import from `PropertyBankDelta` to `PropertyDelta`.
2.  **Update `Changed` struct** (Line 662):
    ```rust
    struct Changed {
        raw: RawPropertyBank,
        delta: PropertyDelta,  // Was: PropertyBankDelta
        raw_hash: HashRecord,
    }
    ```
3.  **Update `diff_property_bank` call**:
    *   Now returns `PropertyDelta` directly (unchanged return pattern).
4.  **Update `apply_delta`** (Line 773):
    ```rust
    fn apply_delta(&self, bank: &mut PropertyBank) {
        if self.status.delta.is_empty() {
            return;
        }
        let existing = bank.set_properties();
        // delta.upserts() is already PropertyMap!
        for (name, property) in self.status.delta.upserts() {
            existing.insert(name, property);
        }
        for name in self.status.delta.removals() {
            existing.remove(name);
        }
        *bank.set_recorded_at() = SystemTime::now();
    }
    ```

### Phase 4: Update Schema Processor

**Location**: `lithos-core/src/schema/schema_processor.rs`

1.  **Update imports**:
    *   Change import from `SchemaPropertyDelta` to `PropertyDelta`.
    *   Add import for `RefExpander`.
2.  **Update `RebuildNodePayload`** (Line 479):
    ```rust
    pub(crate) struct RebuildNodePayload {
        // ...
        property_delta: Option<PropertyDelta>,  // Was: Option<SchemaPropertyDelta>
    }
    ```
3.  **Update `UpdateNodePayload`** (Line 489):
    ```rust
    pub(crate) struct UpdateNodePayload {
        // ...
        property_delta: PropertyDelta,  // Was: SchemaPropertyDelta
    }
    ```
4.  **Update `analyze_properties`** (Lines 1917-1925):
    ```rust
    let empty_hashes = RawPropertyMapHash::default();
    let old_property_hashes = payload.view.current()
        .map_or(&empty_hashes, |v| v.hashes().properties());

    // Create expander to resolve refs during delta computation
    let expander = RefExpander::new(property_bank);
    let property_delta = PropertyDeltaEngine::for_schema(&payload.raw, old_property_hashes)
        .diff_schema(&expander)?;
    ```
5.  **Optimize `expanded_by_id` computation**:
    *   Only expand properties for schemas where `ExtendsChangeKind` is NOT `Unchanged`:
    ```rust
    let expanded_by_id: HashMap<SchemaId, PropertyMap> = /* only for needs_full_expansion_ids */ {
        // ... existing logic ...
    };
    ```
6.  **Update `construct_schema_incremental`**:
    *   For `ExtendsChangeKind::Unchanged`:
    ```rust
    (ExtendsChangeKind::Unchanged, Some(delta)) => {
        let schema = fetched_by_id.get(&id).cloned()
            .or_else(|| ...)?;

        let mut properties = schema.properties().clone();

        // Apply ONLY the changed properties from delta (no full expansion!)
        for (name, prop) in delta.upserts() {
            properties.insert(name.clone(), prop.clone());
        }
        for name in delta.removals() {
            properties.remove(name);
        }

        // ... rest of schema construction ...
    }
    ```
    *   For `Rewired`/`RootToChild` inheritance changes (still needs full expansion).

---

## File Impact Summary

| File                                   | Lines     | Changes Description                                       |
| -------------------------------------- | -------- | ------------------------------------------------------ |
| `lithos-core/src/schema/delta.rs`            | ~400     | Remove alias, define `PropertyDelta`, update engine methods    |
| `lithos-core/src/schema/expander.rs`           | 1        | Expose `expand_property` as `pub(crate)`               |
| `lithos-core/src/schema/schema_processor.rs` | ~100     | Update payloads, analysis, and incremental construction |
| `lithos-core/src/schema/property_bank_processor.rs` | ~30      | Use `PropertyDelta` instead of `PropertyBankDelta` |
| (Tests in all files)                      | ~150     | Adapt tests to new API                          |

---

## Verification

1.  `mise run fmt` - Format code.
2.  `mise run lint` - Check for clippy warnings (unused imports, type mismatches).
3.  `mise run test:unit:core` - Run all 876+ unit tests.
4.  `mise run test:integration` - Run all 33 integration tests.
5.  `mise run verify` - Full quality gate.

---

## Expected Benefits

1.  **True Incremental Updates**: Only resolve/expand changed properties, not all properties.
2.  **Unified API**: Single `PropertyDelta` concept for both processors.
3.  **Reduced Code**: Eliminates duplicate delta logic.
4.  **Performance**: Significant improvement for schemas with many properties but few changes.
