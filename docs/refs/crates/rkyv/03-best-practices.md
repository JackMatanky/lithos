# Best Practices & Effective Usage

To maximize the effectiveness of `rkyv`—especially in storage and performance-critical paths—follow these best practices.

## Use High-Level Serialization APIs
For general use, prefer `rkyv::to_bytes::<Error>(&value)` for simplicity. However, when serializing extensively in a tight loop, reuse allocations to avoid buffer churning (for example, by utilizing `Arena` allocators with `to_bytes_with_alloc`).

```rust
// Standard usage
let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&my_struct).unwrap();

// Advanced allocator usage (for tight loops)
use rkyv::{api::high::to_bytes_with_alloc, ser::allocator::Arena};
let mut arena = Arena::new();
let bytes = to_bytes_with_alloc::<_, rkyv::rancor::Error>(&my_struct, arena.acquire()).unwrap();
```

## Batch Writes
In storage-backed contexts (e.g., when persisting to `redb`), commit and transaction boundaries are expensive. It is much more efficient to batch your serializations and writes into a single transaction rather than hyper-optimizing individual serializations incrementally.

## Lithos Guideline: Storage Model Isolation
In `lithos-core`, **do not** indiscriminately apply `#[derive(Archive)]` over your entire domain model (e.g., `Note` or `Schema`).
- Doing so tightly couples your core business logic to your persistence format.
- Minor refactors in the domain will result in catastrophic breaking changes for your persisted data.
- **Solution**: Restrict `rkyv` usage to explicit storage DTOs (e.g., inside `db_table.rs` or `views.rs`). The `Repository` implementations must map domain objects to these DTOs strictly at the storage boundary.

## Leverage `CopyOptimization` Safely
If you have structs composed exclusively of `Copy` primitives and marked `#[repr(C)]`, implement `CopyOptimization::enable()`. This instructs `rkyv` to bypass full piece-by-piece serialization and simply perform a blazing-fast bulk memory copy of the struct.

```rust
#[derive(rkyv::Archive, rkyv::Serialize)]
#[repr(C)]
struct Vector3 { x: f32, y: f32, z: f32 }

impl rkyv::Archive for Vector3 {
    type Archived = Self;
    type Resolver = ();
    // Safe because the type is `Copy`, `#[repr(C)]`, and has no pointer fields
    const COPY_OPTIMIZATION: rkyv::CopyOptimization<Self> = rkyv::CopyOptimization::enable();
    // ...
}
```

## Optimize Indirection with `#[with(Inline)]`
By default, `rkyv` may place fields behind relative pointers. Using `#[with(Inline)]` wrapper types forces `rkyv` to store smaller fields sequentially rather than referencing them via a pointer.
**Ref:** [rkyv Wrapper Types Guide](https://rkyv.org/derive-macro-features/wrapper-types.html)

- This dramatically improves cache locality for primitive fields.
- Conversely, ensure you keep larger, bulky fields pointer-based to prevent struct bloating and excessive inline copying.

```rust
use rkyv::{Archive, Serialize};
use rkyv::with::Inline;

#[derive(Archive, Serialize)]
struct Record {
    #[with(Inline)]
    id: String,  // Stored inline, no relative pointer lookup overhead

    payload: Vec<u8> // Stored as standard pointer
}
```
