# redb API & Trait Reference

This document provides a detailed technical reference for the core traits and types in `redb`. Understanding these is essential for building high-performance, memory-safe persistence layers.

---

## 1. The `Value` Trait (Serialization)
The `Value` trait defines how Rust types are mapped to database bytes. It is the heart of `redb`'s zero-copy architecture.

```rust
pub trait Value: Debug {
    type SelfType<'a>: Debug + 'a where Self: 'a;
    type AsBytes<'a>: AsRef<[u8]> + 'a where Self: 'a;

    fn fixed_width() -> Option<usize>;
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a> where Self: 'a;
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> where Self: 'b;
    fn type_name() -> TypeName;
}
```

### Key Associated Types
- **`SelfType<'a>`**: The type returned when reading from the database.
  - For `String`, this is `String` (owned).
  - For `&str`, this is `&'a str` (zero-copy view).
- **`AsBytes<'a>`**: The intermediate type used during serialization. Often a byte array or slice.

### Implementing for Complex Types
When using zero-copy frameworks like `rkyv`, `SelfType` should be the `Archived` version of your struct.
> **Source:** [redb/src/types.rs](https://github.com/cberner/redb/blob/master/src/types.rs)

---

## 2. The `Key` Trait (Ordering)
`Key` extends `Value` by adding a comparison method. `redb` maintains its B-trees in the order defined by this trait.

```rust
pub trait Key: Value {
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering;
}
```

### Requirements
- **Total Order:** The implementation must provide a consistent total order across all possible byte representations.
- **Lexicographical Default:** Most primitive implementations (`u64`, `&str`) use standard lexicographical or numeric comparison.
> **Source:** [redb::Key API](https://docs.rs/redb/latest/redb/trait.Key.html)

---

## 3. Table Types & Hierarchy
`redb` uses a strict separation between read-only and read-write access through its transaction model.

### `ReadableTable` Trait
Common interface for both `ReadOnlyTable` and `Table`.
- `get(key)`: Returns `Result<Option<AccessGuard<V>>>`.
- `range(range)`: Returns an iterator over a range of keys.
- `len()`: Returns the number of entries (O(1) in v2+).
- `iter()`: Full table scan.

### `Table` (Read-Write)
Accessible only within a `WriteTransaction`.
- `insert(key, value)`: Inserts or updates a value.
- `remove(key)`: Deletes a value.
- `insert_reserve(key, size)`: **Performance Tip:** Reserves space for a value and returns a mutable guard to write directly into.

### `MultimapTable`
Specialized table for many-to-one relationships.
- `insert(key, value)`: Adds a value to the set of values associated with the key.
- `get(key)`: Returns an iterator over all values for the key.
> **Source:** [redb/src/table.rs](https://github.com/cberner/redb/blob/master/src/table.rs)

---

## 4. `AccessGuard` Internals
`AccessGuard<'a, V>` is a smart pointer that manages the lifetime of a reference to a database page.

### Mechanics
1. **Lifetime Binding:** The guard is bound to the lifetime of the `Transaction`.
2. **Page Pinning:** While the guard is alive, the underlying B-tree page is "pinned." Even if a writer modifies the tree (Copy-on-Write), the old page will not be reclaimed by the allocator until all guards referencing it are dropped.
3. **Zero-Copy Access:** Calling `.value()` simply executes `Value::from_bytes` on the memory-mapped slice, typically involving zero allocations.

### 🔴 Critical Warning
**Do not hold `AccessGuard`s longer than necessary.** Long-lived guards in read transactions prevent the database from reclaiming freed pages, leading to rapid file growth.
> **Source:** [redb/src/tree_store/page_manager.rs](https://github.com/cberner/redb/blob/master/src/tree_store/page_manager.rs)

---

## 5. B-Tree Layout (Internal)
`redb` stores data in fixed-size pages.

| Page Type | Contents |
| :--- | :--- |
| **Branch** | Contains child page pointers, checksums, and separator keys. |
| **Leaf** | Contains the actual key-value data. |

- **Checksums:** Every branch and leaf is protected by an **XXH3_128** checksum.
- **Copy-on-Write:** Modifications never overwrite existing pages. A new page is allocated, and the path to the root is updated.
> **Source:** [redb Design - B-Tree Pages](https://github.com/cberner/redb/blob/master/docs/design.md#b-tree-pages)
