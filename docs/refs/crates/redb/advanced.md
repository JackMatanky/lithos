# redb Advanced Patterns

This guide explores sophisticated usage patterns for `redb`, focusing on complex data structures, indexing, and zero-copy integration.

---

## 1. Secondary Index Patterns
`redb` does not natively manage secondary indexes. You must implement them within your transactions.

### Pattern: Many-to-One (using `MultimapTable`)
Use `MultimapTable` to map multiple records to a single attribute (e.g., all users with a specific tag).
```rust
const USERS: TableDefinition<&str, &str> = TableDefinition::new("users");
const TAGS: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("tags");

// In a WriteTransaction
let mut user_table = txn.open_table(USERS)?;
let mut tag_index = txn.open_multimap_table(TAGS)?;

user_table.insert("user_1", "Alice")?;
tag_index.insert("admin", "user_1")?; // Map tag to ID
```
> **Source:** [redb::MultimapTable API](https://docs.rs/redb/latest/redb/struct.MultimapTable.html)

---

## 2. Compound Keys & Prefix Queries
`redb` supports tuples as keys (up to 12 elements), enabling multi-dimensional queries.

### Pattern: Multi-dimensional Range Query
To query sensor data by `(timestamp, sensor_id)`:
```rust
const METRICS: TableDefinition<(u64, u32), f64> = TableDefinition::new("metrics");

// Find all data for timestamp 12345 across all sensors
let range = (12345, u32::MIN)..(12345, u32::MAX);
for result in metrics_table.range(range)? {
    let ((ts, id), value) = result?;
}
```
*Note: You can only prefix-query dimensions in the order they appear in the tuple.*
> **Source:** [redb Design - Key Comparisons](https://github.com/cberner/redb/blob/master/docs/design.md#key-end-optional)

---

## 3. Large Values and `insert_reserve`
For values larger than a few kilobytes, `insert_reserve` provides a massive optimization by eliminating temporary allocations.

```rust
// Serialize directly into the database page
let mut guard = table.insert_reserve(key, data_len as u32)?;
let buffer: &mut [u8] = guard.as_mut();
// In-place serialization logic here
```
> **Source:** [redb::Table::insert_reserve](https://docs.rs/redb/latest/redb/struct.Table.html#method.insert_reserve)

---

## 4. Zero-Copy Integration (rkyv)
The most performant way to use `redb` with complex structs is via `rkyv`.

### Recommended Pattern: Closure-based Access
Because `AccessGuard` locks database pages, use a closure to ensure the guard stays alive while you access the archived data.
```rust
pub fn with_archived<F, R>(&self, id: &str, f: F) -> Result<Option<R>, Error>
where
    F: for<'a> FnOnce(&'a ArchivedMyData) -> R,
{
    let txn = self.db.begin_read()?;
    let table = txn.open_table(TABLE)?;
    Ok(table.get(id)?.map(|guard| {
        let archived = unsafe { rkyv::access_unchecked::<MyData>(guard.value()) };
        f(archived)
    }))
}
```
> **Source:** [redb issue #1030: AccessGuard lifetime](https://github.com/cberner/redb/issues/1030)

---

## 5. Atomic Multi-Table Updates
Always perform multiple table updates within a single `WriteTransaction` to ensure atomicity. If any update fails, simply drop the transaction without calling `commit()` to roll back all changes.
