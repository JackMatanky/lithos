# Pitfalls, Alignment Issues, & Anti-Patterns

When integrating `rkyv` into performance-critical applications, beware of these common anti-patterns.

## Alignment Traps with Mmap
If you read data out of memory-mapped files without utilizing the `unaligned` feature flag in `rkyv`, you risk causing CPU traps or panics. OS-level memory mapping (`mmap`) guarantees page-level alignment, but it does *not* guarantee strict struct alignment inside the arbitrary byte layouts of the file.

## Accidental Deserialization Overheads
Calling `rkyv::deserialize(archived)` negates almost all benefits of using `rkyv`.
- **Why**: Deserializing forces the allocation of standard native types (e.g., allocating a heap `String` out of an `ArchivedString`).
- **Rule**: Fall back to `deserialize` exclusively as an escape hatch for code paths that must logically mutate the data. For all read-only paths, access fields directly via `Archived<T>`.

```rust
let archived: &ArchivedTransaction = ...;

// ANTI-PATTERN: Heap allocation!
let deserialized: Transaction = rkyv::deserialize(archived).unwrap();

// GOOD: Zero-copy access
let amount = archived.amount;
```

## Self-Referential Types vs Closures
A common mistake when writing Repository traits is attempting to return an `Archived` guard directly from a data store (like `redb`). This creates impossible lifetime requirements and self-referential struct issues in Rust because the reference to `Archived` cannot outlive the database transaction guard.

**Anti-Pattern (Avoid):**
```rust
// Fails compilation because the DB Guard is dropped at the end of the function
fn get_archived(&self, id: Id) -> Result<Option<&Archived<T>>, Error>;
```

**Best Practice (Zero-Copy Closure):**
Instead, utilize a closure-based zero-copy extraction pattern. This keeps the lifetime of the borrow contained entirely within the method execution.
```rust
fn with_archived<F, R>(&self, id: Id, f: F) -> Result<Option<R>, Error>
where
    F: for<'a> FnOnce(&'a Archived<T>) -> R
{
    self.db.get::<T, _, _>(TABLE, id, |archived| {
        f(archived) // Access happens here while DB transaction is held
    })
}
```

## Cyclic Graphs
`rkyv`'s out-of-the-box shared pointers do not natively handle cyclic data structures well. Attempting to serialize a cyclic graph (e.g., `A` points to `B`, and `B` points to `A` via `Arc`) can lead to infinite recursion during serialization. It requires specific custom bounds and extra care to break cycles if they are present in your domain model.
