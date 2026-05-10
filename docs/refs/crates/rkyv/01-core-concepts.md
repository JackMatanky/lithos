# Core Concepts & Zero-Copy Patterns

`rkyv` (archive) is a zero-copy deserialization framework for Rust. Unlike traditional serialization (like `serde`), which parses bytes into an in-memory data structure (allocating memory and consuming CPU), `rkyv` structures its encoded representation to be exactly the same as the in-memory representation. This allows access to the data via pointer offsets and casts, doing virtually no work.

## Total vs. Partial Zero-Copy

*   **Partial Zero-Copy** (e.g., Serde + Bincode): Borrows `&str` or `&[u8]` from the buffer, but still parses the structure, allocating objects like `Vec`s and handling endianness at runtime.
*   **Total Zero-Copy** (rkyv): Guarantees no data is copied and no parsing work is done. The buffer itself *is* the data structure.

## Architecture and Core Traits

`rkyv` uses relative pointers (`RelPtr`) instead of absolute pointers. Absolute pointers hold an exact memory address, meaning if the buffer is loaded at a different address (e.g., due to ASLR or a new process), the pointers dangle. Relative pointers store the offset between the pointer and the data, making them position-independent.

The core functionality is split into three traits (and their `Unsized` variants):

1.  **`Archive`**: Defines the archived representation of a type (e.g., `String` -> `ArchivedString`).
2.  **`Serialize`**: Calculates bookkeeping data (Resolvers) needed to lay out the archived type. For instance, the length and offset of a string's characters.
3.  **`Deserialize`**: Converts an archived type back into the original Rust type (traditional deserialization, allocating memory). Often unnecessary unless mutation is required.

### Resolvers

When archiving a type (e.g., a tuple `(String, String)`), the bytes for both strings must be written to the buffer before the tuple itself is finalized to prevent interleaved bytes. Resolvers carry this intermediate state between the serialization step (writing the dependencies) and the resolve step (finalizing the parent object).

## Shared Pointers (`Rc`, `Arc`)

`rkyv` preserves shared pointers during serialization.
*   **Serialization**: An `Rc` is serialized on first encounter. Subsequent `Rc`s pointing to the same data reuse the address.
*   **Deserialization**: `Pooling` controls whether shared pointers are duplicated or pooled together.
*   **Validation Restriction**: Shared pointers pointing to the same object will fail validation if they are unsized to different types (e.g., `[T; N]` vs `[T]`).

## Unsized Types and Trait Objects

Unsized types (like `str`, `[T]`) require metadata. In `rkyv`, the metadata (like length or a vtable) is archived separately from the relative pointer. Trait objects (`dyn Trait`) are supported via the `rkyv_dyn` sister crate using the `#[archive_dyn]` macro.
