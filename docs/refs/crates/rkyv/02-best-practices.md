# Best Practices & Effective Usage

To maximize the effectiveness of `rkyv`—especially in storage and performance-critical paths—follow these best practices.

## Use High-Level Serialization APIs
For general use, prefer `rkyv::to_bytes::<Error>(&value)` for simplicity. However, when serializing extensively in a tight loop, reuse allocations to avoid buffer churning (for example, by utilizing `Arena` allocators with `to_bytes_with_alloc`).

## Batch Writes
In storage-backed contexts (e.g., when persisting to Redb), commit and transaction boundaries are expensive. It is much more efficient to batch your serializations and writes into a single transaction rather than hyper-optimizing individual serializations incrementally.

## Isolate to Explicit Storage Models
Do **not** indiscriminately apply `#[derive(Archive)]` over your entire domain model.
- Doing so tightly couples your domain logic to your persistence format.
- Minor refactors in the domain will result in catastrophic breaking changes for your persisted data.
- **Solution**: Restrict `rkyv` usage to explicit DTOs or Storage Records (e.g., `NoteRecord`). Map your domain objects to these DTOs strictly at the storage boundary.

## Leverage `CopyOptimization` Safely
If you have structs composed exclusively of `Copy` primitives and marked `#[repr(C)]`, implement `CopyOptimization::enable()`. This instructs `rkyv` to bypass full piece-by-piece serialization and simply perform a blazing-fast bulk memory copy of the struct.

## Optimize Indirection with `#[with(Inline)]`
By default, `rkyv` may place fields behind relative pointers. Using `#[with(Inline)]` wrapper types forces `rkyv` to store smaller fields sequentially rather than referencing them via a pointer.
- This dramatically improves cache locality for primitive fields.
- Conversely, ensure you keep larger, bulky fields pointer-based to prevent struct bloating and excessive inline copying.
