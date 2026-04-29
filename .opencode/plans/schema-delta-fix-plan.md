# Schema Delta Fix Plan

## Context

The `SchemaPropertyUpserts` struct in `delta.rs` stores raw types (`RawPropertyInline`, `RawPropertyRef`), but these values are **never consumed** during incremental updates. The processor only uses `contains_upsert(name)` as a membership check against `expanded_by_id`.

This breaks the incremental update strategy because:

1. `expanded_by_id` expands ALL properties (even unchanged ones)
2. The delta's raw values are discarded
3. True incremental update (only applying changed properties) is not achieved

Additionally, `type PropertyHashes = RawPropertyMapHash` is an unnecessary alias.

## Goals

1. **Remove `PropertyHashes` alias** - Use `RawPropertyMapHash` directly
2. **Fix `SchemaPropertyUpserts`** to enable true incremental updates by storing domain types

## Plan

### Step 1: Remove `PropertyHashes` alias

**File**: `lithos-core/src/schema/delta.rs`

- Delete line 30: `type PropertyHashes = RawPropertyMapHash;`
- Replace all usages of `PropertyHashes` with `RawPropertyMapHash`:
  - Line 33-34: `PropertyBankDeltaResult` type signature
  - Line 313, 323, 337, 375: `PropertyDeltaEngine` struct and impls
  - Line 406: `PropertyChangeSet` struct
  - Line 414: `into_parts()` return type

### Step 2: Redesign `SchemaPropertyUpserts` to store domain types

**Current state** (broken for incremental updates):

```rust
pub(crate) struct SchemaPropertyUpserts {
    inline: HashMap<PropertyName, RawPropertyInline>,
    refs: HashMap<PropertyName, RawPropertyRef>,
}
```

**New design** (enables true incremental updates):

```rust
pub(crate) struct SchemaPropertyUpserts {
    /// Changed properties that are ready to apply (already expanded/resolved)
    properties: PropertyMap,
}
```

**Rationale**:

- `PropertyBankDelta` already stores domain type (`PropertyMap`) - this makes them consistent
- The `diff_schema()` method needs to expand/resolve refs, similar to how `expanded_by_id` works
- The processor can then directly apply `properties` without re-expanding everything

### Step 3: Update `diff_schema()` to return domain types

**File**: `lithos-core/src/schema/delta.rs:349-366`

Current flow:

```rust
pub(crate) fn diff_schema(&self) -> SchemaPropertyDelta {
    let change_set = self.compute_change_set();
    let (upserts, removals, _current_hashes) = change_set.into_parts();
    let mut typed_upserts = SchemaPropertyUpserts::default();

    for (name, entry) in upserts {
        match entry {
            RawProperty::Inline(inline) => {
                typed_upserts.inline.insert(name, inline);
            }
            RawProperty::Ref(r#ref) => {
                typed_upserts.refs.insert(name, r#ref);
            }
        }
    }

    SchemaPropertyDelta::new(typed_upserts, removals)
}
```

**Problem**: Cannot convert to domain types without a `RefExpander` and property bank.

**Solution**: Change the API to accept an expander:

```rust
pub(crate) fn diff_schema(
    &self,
    expander: &RefExpander,
    property_bank: &PropertyBank,
) -> Result<SchemaPropertyDelta, SchemaLoaderError> {
    let change_set = self.compute_change_set();
    let (upserts, removals, _current_hashes) = change_set.into_parts();

    // Convert raw upserts to domain types
    let mut properties = PropertyMap::new();

    for (name, entry) in upserts {
        match entry {
            RawProperty::Inline(inline) => {
                let prop = Property::try_from(inline)
                    .map_err(SchemaLoaderError::Resolution)?;
                properties.insert(name, prop);
            }
            RawProperty::Ref(r#ref) => {
                // Need to expand this ref
                let expanded = expander.expand_single(&r#ref, property_bank)?;
                properties.insert(name, expanded);
            }
        }
    }

    let upserts = SchemaPropertyUpserts { properties };
    SchemaPropertyDelta::new(upserts, removals)
}
```

**Wait** - this changes the function signature significantly. Let me reconsider...

### Step 3 (Revised): Simpler approach

Looking more carefully at the flow:

1. `diff_schema()` is called in `schema_processor.rs:1922-1924`
2. At that point, we have access to `property_bank` and can create a `RefExpander`
3. The expanded properties are computed later in `expanded_by_id`

**Better approach**: Move the expansion into `diff_schema()` by passing the necessary context.

### Step 4: Update call site in `schema_processor.rs`

**File**: `lithos-core/src/schema/schema_processor.rs:1917-1924`

Current:

```rust
let old_property_hashes = payload
    .view
    .current()
    .map_or(&empty_hashes, |v| v.hashes().properties());
let property_delta =
    PropertyDeltaEngine::for_schema(&payload.raw, old_property_hashes)
        .diff_schema();
```

New (requires passing expander and property_bank):

```rust
let old_property_hashes = payload
    .view
    .current()
    .map_or(&empty_hashes, |v| v.hashes().properties());

let expander = RefExpander::new(property_bank.clone());
let property_delta =
    PropertyDeltaEngine::for_schema(&payload.raw, old_property_hashes)
        .diff_schema(&expander)?;
```

### Step 5: Update processor to use delta directly

**File**: `lithos-core/src/schema/schema_processor.rs:2467-2517`

Current (broken - re-expands everything):

```rust
(ExtendsChangeKind::Unchanged, Some(delta)) => {
    let schema = fetched_by_id.get(&id).cloned()...;
    let expanded = expanded_by_id.get(&id)...;

    let mut properties = schema.properties().clone();
    for (name, prop) in expanded {
        if delta.contains_upsert(name) {
            properties.insert(name.clone(), prop.clone());
        }
    }
    // ...
}
```

New (true incremental - only applies changed properties):

```rust
(ExtendsChangeKind::Unchanged, Some(delta)) => {
    let schema = fetched_by_id.get(&id).cloned()...;

    let mut properties = schema.properties().clone();

    // Apply only the changed properties from the delta
    for (name, prop) in delta.upserts().iter() {
        properties.insert(name.clone(), prop.clone());
    }
    for name in delta.removals() {
        properties.remove(name);
    }
    // ...
}
```

### Step 6: Remove `expanded_by_id` dependency for incremental case

The `expanded_by_id` map is still needed for the `ExtendsChangeKind::Rewired` case (where inheritance changed), but NOT for the `Unchanged` case.

We can optimize by:

1. Only computing `expanded_by_id` for schemas that need full expansion (extends changed)
2. Using delta's upserts directly for incremental updates

### Step 7: Update `SchemaPropertyUpserts` API

Remove methods that are no longer needed:

- `inline()` - returns `&HashMap<PropertyName, RawPropertyInline>` (gone)
- `refs()` - returns `&HashMap<PropertyName, RawPropertyRef>` (gone)
- `contains_inline()` - (gone)
- `contains_ref()` - (gone)

Add new methods:

```rust
impl SchemaPropertyUpserts {
    pub(crate) fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&PropertyName, &Property)> {
        self.properties.iter()
    }

    pub(crate) fn contains_name(&self, name: &PropertyName) -> bool {
        self.properties.contains_key(name)
    }
}
```

## Files to Modify

1. `lithos-core/src/schema/delta.rs`
   - Remove `PropertyHashes` alias
   - Replace all usages with `RawPropertyMapHash`
   - Redesign `SchemaPropertyUpserts` to store `PropertyMap`
   - Update `diff_schema()` to accept expander and return domain types
   - Update `SchemaPropertyDelta` API methods

2. `lithos-core/src/schema/schema_processor.rs`
   - Update `diff_schema()` call site to pass expander
   - Update `construct_schema_incremental()` to use delta directly
   - Remove `expanded_by_id` usage for incremental case

3. `lithos-core/src/schema/delta.rs` (tests)
   - Update tests to use new `SchemaPropertyUpserts` API

## Verification

1. `mise run fmt` - Format code
2. `mise run lint` - Check clippy warnings
3. `mise run test:unit:core` - Run unit tests
4. `mise run verify` - Full quality gate
