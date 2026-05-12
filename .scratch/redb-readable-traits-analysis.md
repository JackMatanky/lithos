# redb ReadableTable and ReadableMultimapTable Traits Analysis

## Executive Summary

The `ReadableTable` and `ReadableMultimapTable` traits in redb are **runtime interfaces for opened table instances**, not for table definitions. Our current `UuidTable`/`UuidMultimap` wrappers wrap `TableDefinition`/`MultimapTableDefinition` (compile-time schema), while the traits apply to `Table<'txn, K, V>`/`MultimapTable<'txn, K, V>` (runtime instances obtained from transactions).

**Recommendation**: Do **not** implement these traits on our wrapper types. Instead, consider creating transaction-scoped wrapper types or helper methods if unified read APIs become necessary.

---

## Trait Method Contracts

### `ReadableTable<K, V>`

**Purpose**: Unified read API for both mutable `Table<'_, K, V>` and immutable `ReadOnlyTable<K, V>`.

**Required Methods**:
- `fn get<'a>(&self, key: impl Borrow<K::SelfType<'a>>) -> Result<Option<AccessGuard<'_, V>>>`
  - Returns value for given key

- `fn range<'a, KR>(&self, range: impl RangeBounds<KR> + 'a) -> Result<Range<'_, K, V>>`
  - Returns double-ended iterator over key range

- `fn first(&self) -> Result<Option<(AccessGuard<'_, K>, AccessGuard<'_, V>)>>`
  - Returns first key-value pair

- `fn last(&self) -> Result<Option<(AccessGuard<'_, K>, AccessGuard<'_, V>)>>`
  - Returns last key-value pair

**Provided Methods**:
- `fn iter(&self) -> Result<Range<'_, K, V>>`
  - Returns double-ended iterator over all elements (defaults to `self.range(..)`)

**Also requires**: `ReadableTableMetadata` (provides `stats()`, `len()`, `is_empty()`)

### `ReadableMultimapTable<K, V>`

**Purpose**: Unified read API for both mutable `MultimapTable<'_, K, V>` and immutable `ReadOnlyMultimapTable<K, V>`.

**Required Methods**:
- `fn get<'a>(&self, key: impl Borrow<K::SelfType<'a>>) -> Result<MultimapValue<'_, V>>`
  - Returns iterator over all values for key (values in ascending order)

- `fn range<'a, KR>(&self, range: impl RangeBounds<KR> + 'a) -> Result<MultimapRange<'_, K, V>>`
  - Returns double-ended iterator over key range

**Provided Methods**:
- `fn iter(&self) -> Result<MultimapRange<'_, K, V>>`
  - Returns double-ended iterator over all elements (defaults to `self.range(..)`)

**Also requires**: `ReadableTableMetadata`

---

## What Types Implement These Traits

### `ReadableTable<K, V>` Implementors

1. **`Table<'txn, K, V>`** (mutable, from `WriteTransaction::open_table()`)
   - Provides both read and write operations
   - Lifetime `'txn` tied to transaction

2. **`ReadOnlyTable<K, V>`** (immutable, from `ReadTransaction::open_table()`)
   - Read-only access, no write methods
   - Lifetime tied to read transaction

### `ReadableMultimapTable<K, V>` Implementors

1. **`MultimapTable<'txn, K, V>`** (mutable, from `WriteTransaction::open_multimap_table()`)
   - Provides both read and write operations
   - Lifetime `'txn` tied to transaction

2. **`ReadOnlyMultimapTable<K, V>`** (immutable, from `ReadTransaction::open_multimap_table()`)
   - Read-only access, no write methods
   - Lifetime tied to read transaction

---

## Why Our Wrappers Cannot Meaningfully Implement These Traits

### Current Design

```rust
pub struct UuidTable<K: UuidV7DbType + 'static, V: Value + 'static> {
    definition: TableDefinition<'static, K, V>,  // ← Compile-time schema
}

pub struct UuidMultimap<K: UuidV7DbType + 'static, V: Key + 'static> {
    definition: MultimapTableDefinition<'static, K, V>,  // ← Compile-time schema
}
```

**Key observation**: These wrap **table definitions** (compile-time schema), not **opened table instances** (runtime data).

### The Semantic Mismatch

| What                | Our Wrappers              | Trait Requirements           |
|---------------------|---------------------------|------------------------------|
| **Wraps**           | `TableDefinition`         | Opened `Table<'txn, K, V>`   |
| **Lifetime**        | `'static`                 | `'txn` (transaction-scoped)  |
| **Purpose**         | Define schema             | Query data                   |
| **Available at**    | Compile-time (const)      | Runtime (after `open_table`) |
| **Operations**      | None (just schema)        | Read operations              |

**Cannot implement because**:
1. Our wrappers have no access to transaction data
2. Methods like `get()` require an opened table instance
3. No `&self` method on `TableDefinition` returns data
4. Traits require lifetime `'_` tied to a transaction we don't have

### Example: Why `get()` Cannot Work

```rust
impl<K: UuidV7DbType + 'static, V: Value + 'static> ReadableTable<K, V>
    for UuidTable<K, V>
{
    fn get<'a>(&self, key: impl Borrow<K::SelfType<'a>>)
        -> Result<Option<AccessGuard<'_, V>>>
    {
        // ❌ Cannot implement: we only have `self.definition` (a schema)
        // We need an opened table from a transaction
        // self.definition has no method to query data
        todo!()
    }
}
```

---

## Alternative Design Options

### Option 1: Transaction-Scoped Wrappers (Recommended)

Create **runtime** wrappers that hold opened table instances:

```rust
/// Runtime wrapper for an opened UUID-keyed table
pub struct OpenedUuidTable<'txn, K: UuidV7DbType + 'static, V: Value + 'static> {
    inner: Table<'txn, K, V>,
}

impl<'txn, K: UuidV7DbType + 'static, V: Value + 'static>
    ReadableTable<K, V> for OpenedUuidTable<'txn, K, V>
{
    fn get<'a>(&self, key: impl Borrow<K::SelfType<'a>>)
        -> Result<Option<AccessGuard<'_, V>>>
    {
        self.inner.get(key)  // ✅ Delegate to redb's Table
    }

    // ... other methods delegate to self.inner
}
```

**When to use**:
- When you want to add domain-specific query methods while preserving the standard trait interface
- When wrapping transaction-scoped logic (e.g., validation, metrics)

**Tradeoffs**:
- ✅ Semantically correct (wraps runtime data)
- ✅ Can implement traits meaningfully
- ❌ Adds one more layer of wrapping
- ❌ Requires explicit `open()` helper or constructor

### Option 2: Extension Trait on Transactions

Add helper methods to transactions rather than wrapping tables:

```rust
pub trait TransactionExt {
    fn open_uuid_table<K: UuidV7DbType + 'static, V: Value + 'static>(
        &self,
        table: &UuidTable<K, V>,
    ) -> Result<Table<'_, K, V>>;
}

impl TransactionExt for WriteTransaction {
    fn open_uuid_table<K: UuidV7DbType + 'static, V: Value + 'static>(
        &self,
        table: &UuidTable<K, V>,
    ) -> Result<Table<'_, K, V>> {
        self.open_table(table.definition())
    }
}
```

**When to use**:
- When you just need ergonomic table opening
- When you don't need custom query logic

**Tradeoffs**:
- ✅ Minimal overhead
- ✅ No new types
- ❌ Cannot implement `ReadableTable` traits (still returns raw `Table<'_, K, V>`)
- ❌ Less encapsulation

### Option 3: Keep Current Design (Status Quo)

Continue using `UuidTable`/`UuidMultimap` as **schema definitions only**:

```rust
const NOTES: UuidTable<NoteId, &[u8]> = UuidTable::new("notes");

// Usage
let table = write_txn.open_table(NOTES.definition())?;
table.get(&note_id)?;
```

**When to use**:
- When the current API is sufficient
- When adding wrappers would be premature abstraction

**Tradeoffs**:
- ✅ Simple, zero overhead
- ✅ Direct access to redb's Table API
- ❌ Cannot implement `ReadableTable` traits
- ❌ No place for domain-specific query logic

---

## Recommendation

**Do not implement `ReadableTable`/`ReadableMultimapTable` on `UuidTable`/`UuidMultimap`.**

### Why Not

1. **Semantic mismatch**: Our wrappers are compile-time schema definitions, not runtime table instances
2. **No transaction context**: Traits require data access that our wrappers don't have
3. **Incorrect lifecycle**: Traits are for transaction-scoped instances, our wrappers are `'static`

### What to Do Instead

- **For most use cases**: Keep the current design (Option 3)
  - Current wrappers serve their purpose: type-safe schema definitions
  - Direct use of redb's `Table<'_, K, V>` is idiomatic and efficient

- **If you need unified read APIs**: Consider Option 1 (transaction-scoped wrappers) in the future
  - Create `OpenedUuidTable<'txn, K, V>` that wraps `Table<'txn, K, V>`
  - Implement `ReadableTable` on that wrapper
  - Only introduce when you have concrete domain-specific query logic to encapsulate

### Key Insight

The `ReadableTable` traits are **redb's internal abstraction** to unify read operations across mutable and immutable table instances. They're not intended as extension points for user wrappers. If you need domain-specific query logic, wrap the **opened table** (runtime), not the **table definition** (compile-time).

---

## References

- [redb::ReadableTable docs](https://docs.rs/redb/latest/redb/trait.ReadableTable.html)
- [redb::ReadableMultimapTable docs](https://docs.rs/redb/latest/redb/trait.ReadableMultimapTable.html)
- [redb::Table docs](https://docs.rs/redb/latest/redb/struct.Table.html)
- [redb::MultimapTable docs](https://docs.rs/redb/latest/redb/struct.MultimapTable.html)
- Current implementation: `lithos-core/src/db/table.rs:25-83`
