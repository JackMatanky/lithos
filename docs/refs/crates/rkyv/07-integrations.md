# Integrations (`redb`, `memmap2`, `moka`)

This guide covers how `rkyv` integrates with other critical components of the Lithos architecture, predominantly embedded databases, memory mapping, and caches.

## `redb` Integration

`redb` is an embedded key-value database used extensively in Lithos. Using `rkyv` inside `redb` yields a fully zero-copy database pipeline.

### The Integration Flow
`rkyv` serializes a struct to a `[u8]` slice (byte array), which natively implements `redb::Value`. Therefore, any `rkyv` output can immediately be stored into `redb`.

```rust
// 1. Serialize the struct into a byte buffer
let view = MyDto::new();
let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&view).unwrap();

// 2. Insert bytes directly as the redb::Value
let mut txn = db.begin_write().unwrap();
{
    let mut table = txn.open_table(MY_TABLE).unwrap();
    table.insert(id_str.as_str(), bytes.as_slice()).unwrap();
}
txn.commit().unwrap();
```

### Implementing Custom `redb::Value` Types
Due to Rust's orphan rules, if you require strongly typed ID columns (like UUIDs) rather than raw `&str` or byte slices, you must wrap them in a local newtype and implement `redb::Value` and `redb::Key` for them manually, as seen in `src/db/uuid.rs`.

### Querying with Zero-Copy
Since `redb` returns an `AccessGuard` protecting the underlying Mmap bytes, you must not return the `rkyv::Archived` reference beyond the function boundary. The standard pattern in Lithos adapters is to use a `with_archived` higher-order function:

```rust
fn with_archived<F, R>(&self, id: Id, f: F) -> Result<Option<R>, DbError>
where
    F: for<'archived> FnOnce(&'archived rkyv::Archived<Template>) -> R,
{
    // The inner closure prevents lifetimes from escaping the Guard
    self.db.get::<Template, _, _>(TEMPLATES, &id.to_string(), |archived| {
        f(archived)
    })
}
```

## `memmap2` Integration

If you need to load very large persisted datasets quickly outside of a database, you can memory-map an `rkyv` file using the OS page cache via `memmap2`.

```rust
use memmap2::MmapOptions;
use std::fs::File;

// Memory-map for zero-copy access
let file = File::open("data.rkyv")?;
let mmap = unsafe { MmapOptions::new().map(&file)? };

// Access without loading into memory
// (Ensure Cargo.toml has `unaligned` feature to prevent mmap alignment panics)
let archived = unsafe { rkyv::access_unchecked::<ArchivedLargeDataset>(&mmap) };
```

## `moka` (Caching) Integration

When using an in-memory cache like `moka`, storing standard Rust structs incurs allocation overhead during cache hydration. Storing `rkyv`-serialized bytes as cache entries provides persistent, low-overhead caching. When integrating:

1. **Serialization**: Run `rkyv::to_bytes` and store the resultant byte buffers in the cache.
2. **Access**: When a cache hit occurs, pull the byte buffer and use `rkyv::access_unchecked` (since you trust your own cache data) to immediately retrieve the `Archived<T>` data.
