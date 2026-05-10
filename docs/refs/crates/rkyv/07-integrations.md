# Integrations

## Integration with `redb`

`redb` is a zero-copy embedded key-value store, making it a perfect companion for `rkyv`. However, `redb` relies heavily on RAII `AccessGuard`s.

### The Orphan Rule and `redb::Value`
Due to Rust's orphan rules, you cannot implement `redb::Value` directly for `rkyv`'s `Archived<T>` or for your domain type `T` using `rkyv` underneath without a wrapper.
You must implement `redb::Value` via local newtypes or wrappers to handle the encoding/decoding.

### Guards and Zero-Copy
`redb`'s `AccessGuard` borrows the transaction/table. You cannot return an `Archived<T>` alongside the `AccessGuard` because `Archived<T>` references bytes owned by the guard.
This is why the `with_archived` closure pattern (documented in [Best Practices](05-best-practices.md)) is strictly enforced. It ensures the `Archived<T>` reference cannot outlive the `AccessGuard`.

## Integration with Memory Mapped Files (`mmap`)

Because `rkyv` uses relative pointers (`RelPtr`), it is fully compatible with memory-mapped files.
*   Relative pointers do not require write access to be fixed up.
*   You can `mmap` a file read-only and immediately cast to the `Archived` representation.
*   This makes `rkyv` uniquely suited for multi-gigabyte datasets, as the OS handles paging parts of the graph into memory lazily.
