# Best Practices & Effective Usage

When building domain models and database layers with `rkyv` (especially in Lithos), adhere to these best practices.

## 1. The `with_archived` Pattern (Closure-based Zero-Copy Extraction)

**Rule:** Never attempt to return an `Archived<T>` or a zero-copy guard struct directly if it requires referencing a temporarily mapped buffer. This leads to self-referential structs.
**Solution:** Use higher-order functions (closures) to extract the needed data while the borrow is valid.

```rust
// ✅ GOOD: Closure-based extraction
pub fn with_archived<F, R>(&self, id: Id, f: F) -> Result<Option<R>, Error>
where
    F: for<'a> FnOnce(&'a Archived<T>) -> R,
{
    let bytes = self.db.get_bytes(id)?;
    let archived = unsafe { rkyv::access_unchecked::<Archived<T>>(&bytes) };
    Ok(Some(f(archived)))
}

// ❌ BAD: Attempting to return a Guard
// pub fn get_archived(&self, id: Id) -> Result<Option<Guard>, Error>;
```

## 2. Treat `Archived<T>` as Free Optimization

`rkyv` automatically generates `Archived*` types that mirror your domain structures. You rarely need to create custom `*View` types.
*   Let `rkyv` do the work: `ArchivedString` and `ArchivedVec` are optimized natively.
*   Introduce manual `*View` structs *only* if profiling demonstrates that the domain shape is deeply inefficient for database queries.

## 3. Deserialization is Usually Unnecessary

`Deserialize` allocates memory and re-creates standard Rust types.
*   If you just need to *read* data to answer a query (e.g., checking a name, computing a sum, mapping to an LSP response), do it directly on the `Archived<T>` via the `with_archived` pattern.
*   Only call `.deserialize(&mut deserializer)` if you intend to deeply mutate the object and save it back.

## 4. Derived Default Impls are Efficient

`rkyv` provides optimal `Archive` implementations for standard library types. For instance, `ArchivedString` uses small-string optimizations to save space. Trust the default derives unless you are writing custom serialization for FFI or highly exotic types.
