# Components Index

`rkyv`'s architecture allows deep customization of serialization and deserialization via highly composable components.

## Serializer Traits

Serializers are mutable objects that implement specific traits to support different serialization requirements.

1.  **`Positional`**: Provides the current offset of the "write head". Essential because relative pointers need to know their relative distance to objects.
2.  **`Writer`**: Consumes bytes and writes them out.
    *   *Write-forward*: Never backtracks, allowing streaming directly to disk/network.
    *   Implementations: `Vec<u8>`, `AlignedVec` (highly-aligned byte vector, the default), `Buffer` (for `no_std` stack allocations), and `IoWriter` (adapter for `std::io::Write`).
3.  **`Allocator`**: Provides temporary memory allocation during serialization (e.g., `Vec` needs temporary space for its resolvers). Reduces slow system allocations by reusing memory (e.g., via an `ArenaHandle`).
4.  **`Sharing`**: Provides mutable state to track shared pointers (`Rc`, `Arc`) ensuring they are de-duplicated during serialization.

**Pre-packaged Serializers**:
*   `HighSerializer`: Default balance of flexibility and performance.
*   `LowSerializer`: For `no_std`, uses no allocations.

## Deserializer Traits

1.  **`Pooling`**: Mirrors `Sharing`. Controls whether deserialized shared pointers are pooled together (reusing memory) or cloned/unpooled.

## Allocation Tracking

You can track synthetic metrics and memory usage during serialization using `AllocationTracker`.

```rust
use rkyv::ser::allocator::{ArenaHandle, AllocationTracker};

let tracker = AllocationTracker::new(ArenaHandle::new());
// Use tracker inside a custom Serializer...
```
After serialization, `into_stats()` retrieves `AllocationStats`, useful for debugging or pre-allocating exact buffer sizes.
