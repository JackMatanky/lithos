# Core Concepts & Zero-Copy Patterns

`rkyv` (archive) is a zero-copy deserialization framework. It allows you to directly access serialized data without the overhead of traditional parsing/deserialization.

## Direct Memory Access
Instead of parsing bytes into a newly allocated native structure (e.g., using `serde`), `rkyv` creates an `Archived<T>` type representation that can be mapped directly onto a byte buffer. This enables immediate data access with zero heap allocation upon read.

## The `Archive` Trait
The `Archive` trait is the core mechanism powering `rkyv`. An `Archived` type must implement the `Portable` trait to guarantee a stable layout and byte representation across different environments.

## Archived Types vs Native Types
It is important to understand that `Archived<T>` is a fundamentally distinct type from `T`.
- Zero-copy access works exclusively through `Archived<T>`.
- Standard "deserialization" (converting `Archived<T>` back to `T`) is typically an anti-pattern unless you need to mutate the data or pass it to an API requiring the native type.

## Zero-Copy Collections
`rkyv` provides equivalents to standard collections that avoid heap allocation when accessed:
- **`ArchivedVec`**: Grants `O(1)` indexing directly from the underlying bytes.
- **`ArchivedString`**: Allows seamless access as an `&str`.
- **`ArchivedHashMap` & `ArchivedBTreeMap`**: Provide zero-copy key/value lookups.

## Shared Pointers
`rkyv` naturally supports deduplication. If multiple elements reference the same `Arc<T>` during the serialization process, the archived format preserves that sharing without replicating the underlying bytes, leading to smaller payloads and maintaining topological structure.
