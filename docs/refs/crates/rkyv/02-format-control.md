# Format Control & Compatibility

The binary format produced by `rkyv` is strictly tied to its feature flags. **Changing these features changes the on-disk binary format, representing a breaking change for stored data.**

## Feature Flags

*   **Endianness** (`little_endian` / `big_endian`): Controls byte order. Left unspecified, `rkyv` defaults to little-endian.
*   **Alignment** (`aligned` / `unaligned`): Controls whether primitive types have alignment greater than 1. Aligned accesses are faster on modern CPUs. Left unspecified, `rkyv` defaults to aligned.
*   **Pointer Width** (`pointer_width_16`, `pointer_width_32`, `pointer_width_64`): Controls the size of relative pointers. Smaller pointers save space but limit the maximum buffer size. Defaults to 32-bit (supporting buffers up to 4GB).

## Macro Attributes for Format Control

When using `#[derive(Archive, Serialize, Deserialize)]`, you can customize the generated types and behavior:

### `omit_bounds`

By default, `rkyv` generates "perfect derives" (adding `T: Archive` for every generic field `T`). This can cause overflow with recursive types (like `Box<Node>`) or leak private types.
Use `#[rkyv(omit_bounds)]` to disable perfect derive for a field. You can manually add bounds back using:
*   `#[rkyv(archive_bounds(..))]`
*   `#[rkyv(serialize_bounds(..))]`
*   `#[rkyv(deserialize_bounds(..))]`

### Wrapper Types (`with = ..`)

You can customize how a field is archived using a wrapper type without changing the original struct definition.
```rust
#[derive(Archive, Serialize)]
struct Example {
    #[rkyv(with = SomeWrapper)]
    field: i32,
}
```

### Remote Derive (`remote = ..`)

Used to support types from external crates that don't implement `rkyv` traits. You define a local struct mirroring the remote type, annotate it with `#[rkyv(remote = path::to::RemoteType)]`, and provide a `From` impl for deserialization.

```rust
#[derive(Archive, Serialize, Deserialize)]
#[rkyv(remote = external_crate::TheirType)]
struct TheirTypeDef {
    // fields matching external_crate::TheirType
}
```

## Object Order

`rkyv` lays out objects in **depth-first order** from the leaves to the root. The root object is stored at the **end** of the buffer. This deterministic layout means you generally don't need to store the position of the root object; `rkyv::access` assumes the root object ends exactly at the end of the buffer.
