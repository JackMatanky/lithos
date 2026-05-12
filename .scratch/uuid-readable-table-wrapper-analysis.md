# UUID ReadableTable Wrapper Analysis

## Executive Summary

**Recommendation: Do NOT create wrapper types implementing `ReadableTable`.**

Instead, use **extension traits** that add UUID-specific convenience methods to `redb::Table` and `redb::MultimapTable`.

## ReadableTable Trait Methods

From https://docs.rs/redb/latest/redb/trait.ReadableTable.html:

```rust
pub trait ReadableTable<K: Key + 'static, V: Value + 'static>: ReadableTableMetadata {
    // Required methods
    fn get<'a>(
        &self,
        key: impl Borrow<K::SelfType<'a>>,
    ) -> Result<Option<AccessGuard<'_, V>>>;

    fn range<'a, KR>(
        &self,
        range: impl RangeBounds<KR> + 'a,
    ) -> Result<Range<'_, K, V>>
       where KR: Borrow<K::SelfType<'a>> + 'a;

    fn first(&self) -> Result<Option<(AccessGuard<'_, K>, AccessGuard<'_, V>)>>;

    fn last(&self) -> Result<Option<(AccessGuard<'_, K>, AccessGuard<'_, V>)>>;

    // Provided method
    fn iter(&self) -> Result<Range<'_, K, V>> { ... }
}
```

## ReadableMultimapTable Trait Methods

From https://docs.rs/redb/latest/redb/trait.ReadableMultimapTable.html:

```rust
pub trait ReadableMultimapTable<K: Key + 'static, V: Key + 'static>: ReadableTableMetadata {
    // Required methods
    fn get<'a>(
        &self,
        key: impl Borrow<K::SelfType<'a>>,
    ) -> Result<MultimapValue<'_, V>>;

    fn range<'a, KR>(
        &self,
        range: impl RangeBounds<KR> + 'a,
    ) -> Result<MultimapRange<'_, K, V>>
       where KR: Borrow<K::SelfType<'a>> + 'a;

    // Provided method
    fn iter(&self) -> Result<MultimapRange<'_, K, V>> {
        // ...
    }
}
```

## Current Architecture

The codebase already has:

1. **Table Definition Wrappers** (`db/table.rs`):
   - `UuidTable<K, V>` - Wraps `TableDefinition<'static, K, V>`
   - `UuidMultimap<K, V>` - Wraps `MultimapTableDefinition<'static, K, V>`
   - These are **const constructible** and **zero-cost**

2. **UUID Key Implementation** (`db/uuid.rs`):
   - `impl_redb_uuid!` macro implements `redb::Key` and `redb::Value` for UUID wrapper types
   - `UuidV7DbType` marker trait for type safety

3. **Current Usage Pattern**:
```rust
// Define table (compile-time constant)
pub const SCHEMAS: UuidTable<SchemaId, &[u8]> = UuidTable::new("schemas_v2");

// Open table and use it
let mut table = tx.inner.open_table(SCHEMAS.definition())?;
table.insert(schema.id(), bytes.as_slice())?;

let table = tx.inner.open_table(SCHEMAS.definition())?;
let Some(guard) = table.get(id)? else { return Ok(None); };
```

The opened `table` is a `redb::Table<SchemaId, &[u8]>` which already implements `ReadableTable<SchemaId, &[u8]>`.

## Proposed Wrapper Approach

```rust
struct UuidReadableTable<'txn, K: UuidV7DbType, V: Value> {
    inner: redb::Table<'txn, K, V>
}

impl<'txn, K: UuidV7DbType, V: Value> ReadableTable for UuidReadableTable<'txn, K, V> {
    fn get<'a>(&self, key: impl Borrow<K::SelfType<'a>>) -> Result<Option<AccessGuard<'_, V>>> {
        self.inner.get(key)
    }

    fn range<'a, KR>(&self, range: impl RangeBounds<KR> + 'a) -> Result<Range<'_, K, V>>
    where KR: Borrow<K::SelfType<'a>> + 'a {
        self.inner.range(range)
    }

    fn first(&self) -> Result<Option<(AccessGuard<'_, K>, AccessGuard<'_, V>)>> {
        self.inner.first()
    }

    fn last(&self) -> Result<Option<(AccessGuard<'_, K>, AccessGuard<'_, V>)>> {
        self.inner.last()
    }
}
```

## Analysis: Wrapper vs Extension Trait

### Problems with Wrapper Type

1. **No Boilerplate Reduction**: The wrapper just delegates every method to `inner`, adding zero value.

2. **Extra Wrapping Cost**: Users must convert from `redb::Table` to `UuidReadableTable`:
   ```rust
   // Before (current)
   let table = tx.inner.open_table(SCHEMAS.definition())?;
   table.get(id)?

   // After (wrapper)
   let table = tx.inner.open_table(SCHEMAS.definition())?;
   let wrapped = UuidReadableTable { inner: table };
   wrapped.get(id)?
   ```

3. **Trait Object Issues**: `ReadableTable` is **NOT dyn compatible** according to the docs:
   ```
   ### Dyn Compatibility
   This trait is **not** dyn compatible.
   ```

   This means you cannot use `&dyn ReadableTable<K, V>` or `Box<dyn ReadableTable<K, V>>`, which kills most abstraction benefits.

4. **Lifetime Complexity**: The wrapper adds another lifetime parameter that must be threaded through all call sites.

5. **Breaking Existing Ergonomics**: `redb::Table` already implements `ReadableTable`. Adding a wrapper forces users to choose between two APIs for the same operations.

### Alternative: Extension Trait

Instead, add convenience methods directly to existing types:

```rust
/// Extension trait for UUID-keyed tables.
pub trait UuidTableExt<K: UuidV7DbType, V: Value> {
    /// Insert with UUID key (avoids explicit ID wrapping in some contexts).
    fn insert_uuid(&mut self, id: K, value: V::SelfType<'_>) -> Result<(), redb::Error>;

    /// Batch get multiple UUIDs efficiently.
    fn get_many(&self, ids: &[K]) -> Result<Vec<Option<AccessGuard<'_, V>>>, redb::Error>;
}

impl<'txn, K: UuidV7DbType, V: Value> UuidTableExt<K, V> for redb::Table<'txn, K, V> {
    fn insert_uuid(&mut self, id: K, value: V::SelfType<'_>) -> Result<(), redb::Error> {
        self.insert(id, value)
    }

    fn get_many(&self, ids: &[K]) -> Result<Vec<Option<AccessGuard<'_, V>>>, redb::Error> {
        ids.iter().map(|id| self.get(*id)).collect()
    }
}
```

**Benefits:**
- No wrapping overhead
- Works with existing `redb::Table` instances
- Adds genuinely useful functionality (batch operations)
- Users can import the trait only when needed
- No lifetime complexity
- Compatible with existing code

## Before/After Comparison

### Current Usage (No Wrapper)

```rust
// Define table
pub const SCHEMAS: UuidTable<SchemaId, &[u8]> = UuidTable::new("schemas_v2");

// Write
self.store.write(|tx| {
    let mut table = tx.inner.open_table(SCHEMAS.definition())?;
    table.insert(schema.id(), bytes.as_slice())?;
    Ok(())
})

// Read
self.store.read(|tx| {
    let table = tx.inner.open_table(SCHEMAS.definition())?;
    let Some(guard) = table.get(id)? else { return Ok(None); };
    // use guard...
})
```

### With Wrapper Type (NOT RECOMMENDED)

```rust
// Define table (same)
pub const SCHEMAS: UuidTable<SchemaId, &[u8]> = UuidTable::new("schemas_v2");

// Write - NO CHANGE, wrapper doesn't help with mutable operations
self.store.write(|tx| {
    let mut table = tx.inner.open_table(SCHEMAS.definition())?;
    let mut wrapped = UuidReadableTable { inner: table }; // Extra step!
    wrapped.insert(schema.id(), bytes.as_slice())?; // Same call
    Ok(())
})

// Read - NO BENEFIT
self.store.read(|tx| {
    let table = tx.inner.open_table(SCHEMAS.definition())?;
    let wrapped = UuidReadableTable { inner: table }; // Extra step!
    let Some(guard) = wrapped.get(id)? else { return Ok(None); }; // Same call
    // use guard...
})
```

### With Extension Trait (RECOMMENDED)

```rust
// Define table (same)
pub const SCHEMAS: UuidTable<SchemaId, &[u8]> = UuidTable::new("schemas_v2");

// Write - NEW: batch operations
use crate::db::UuidTableExt;

self.store.write(|tx| {
    let mut table = tx.inner.open_table(SCHEMAS.definition())?;

    // NEW: Batch insert multiple schemas efficiently
    for schema in schemas {
        table.insert_uuid(schema.id(), bytes.as_slice())?;
    }
    Ok(())
})

// Read - NEW: batch lookups
self.store.read(|tx| {
    let table = tx.inner.open_table(SCHEMAS.definition())?;

    // NEW: Get multiple schemas in one call
    let results = table.get_many(&[id1, id2, id3])?;
    // process results...
})
```

## Real-World Example: Schema Lookup

### Current Pattern (Good)

```rust
fn find_schema_by_id(&self, id: SchemaId) -> Result<Option<Schema>, SchemaStorageV2Error> {
    self.store.read(|tx| {
        let table = match tx.inner.open_table(SCHEMAS.definition()) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        let Some(guard) = table.get(id)? else {
            return Ok(None);
        };

        // Zero-copy deserialization
        let bytes: &[u8] = guard.value();
        let archived = rkyv::access::<rkyv::Archived<Schema>, _>(bytes)?;
        let schema = rkyv::deserialize::<Schema, _>(archived)?;
        Ok(Some(schema))
    })
}
```

### With Wrapper (No Improvement)

```rust
fn find_schema_by_id(&self, id: SchemaId) -> Result<Option<Schema>, SchemaStorageV2Error> {
    self.store.read(|tx| {
        let table = match tx.inner.open_table(SCHEMAS.definition()) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        // EXTRA STEP: Wrap the table
        let wrapped = UuidReadableTable { inner: table };

        // SAME API as before, no benefit
        let Some(guard) = wrapped.get(id)? else {
            return Ok(None);
        };

        // Same deserialization...
        let bytes: &[u8] = guard.value();
        let archived = rkyv::access::<rkyv::Archived<Schema>, _>(bytes)?;
        let schema = rkyv::deserialize::<Schema, _>(archived)?;
        Ok(Some(schema))
    })
}
```

### With Extension Trait (Actual Improvement)

```rust
fn find_schemas_by_ids(&self, ids: &[SchemaId]) -> Result<Vec<Schema>, SchemaStorageV2Error> {
    use crate::db::UuidTableExt;

    self.store.read(|tx| {
        let table = match tx.inner.open_table(SCHEMAS.definition()) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };

        // NEW: Batch lookup multiple schemas at once
        let guards = table.get_many(ids)?;

        let mut schemas = Vec::with_capacity(ids.len());
        for guard_opt in guards {
            if let Some(guard) = guard_opt {
                let bytes: &[u8] = guard.value();
                let archived = rkyv::access::<rkyv::Archived<Schema>, _>(bytes)?;
                let schema = rkyv::deserialize::<Schema, _>(archived)?;
                schemas.push(schema);
            }
        }

        Ok(schemas)
    })
}
```

## ReadableMultimapTable: Same Conclusion

For `ReadableMultimapTable`, the analysis is identical:

1. **Wrapper adds no value**: Just delegates to `inner.get()`, `inner.range()`, `inner.iter()`
2. **Extension trait is better**:

```rust
pub trait UuidMultimapExt<K: UuidV7DbType, V: Key> {
    /// Get all values for multiple keys at once.
    fn get_many_multimap(&self, keys: &[K]) -> Result<Vec<Vec<V>>, redb::Error>;
}

impl<'txn, K: UuidV7DbType, V: Key> UuidMultimapExt<K, V>
    for redb::MultimapTable<'txn, K, V>
{
    fn get_many_multimap(&self, keys: &[K]) -> Result<Vec<Vec<V>>, redb::Error> {
        keys.iter()
            .map(|key| {
                self.get(*key)?
                    .map(|iter| iter.collect::<Result<Vec<_>, _>>())
                    .transpose()
            })
            .collect()
    }
}
```

## Conclusion

**Do NOT implement wrapper types.**

The `ReadableTable` and `ReadableMultimapTable` traits are already implemented by `redb::Table` and `redb::MultimapTable`. Creating wrapper types that re-implement these traits provides zero benefit while adding:
- Wrapping boilerplate at every call site
- Lifetime complexity
- API duplication
- No new functionality

**Instead, use extension traits** to add genuinely useful UUID-specific operations like batch lookups, which would actually reduce boilerplate in real-world usage.

## Recommendation Summary

| Approach | Verdict | Reason |
|----------|---------|--------|
| Wrapper implementing `ReadableTable` | ❌ **No** | Pure delegation, no value, extra wrapping cost |
| Wrapper implementing `ReadableMultimapTable` | ❌ **No** | Same as above |
| Extension trait with useful methods | ✅ **Yes** | Adds actual functionality (batch ops), no wrapping overhead |
| Keep current architecture | ✅ **Yes** | Already optimal for single operations |

The current architecture with `UuidTable` and `UuidMultimap` as **table definition wrappers** is correct. The opened `redb::Table<K, V>` already has everything needed. Only add extension traits if batch operations become a common pattern.
