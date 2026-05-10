# Pitfalls, Alignment Issues, & Anti-Patterns

## 1. The "Extra Prefix" Misalignment

`rkyv` expects the buffer to be properly aligned for the root object type. A common pitfall is storing `rkyv` bytes prefixed by custom length headers or database metadata (like a 4-byte `u32` length).

```text
[ 4 bytes length ] [ ... rkyv buffer ... ]
```

If the `rkyv` buffer requires 8-byte alignment, but you read it directly from offset `4`, you will cause an alignment violation on ARM/RISC architectures (and performance penalties on x86).
**Fix**: Ensure your buffer slicing accounts for alignment, or use `AlignedVec` for in-memory manipulation before persisting.

## 2. Trailing Padding

`rkyv` lays objects out in a depth-first order from the leaves to the root, meaning the root object is at the *exact end* of the buffer.
Functions like `rkyv::access` rely on the end of the buffer being tight to the end of the data. If you pad the end of the buffer (e.g., for block-aligned storage), `rkyv::access` may miscalculate the location of the root object.

## 3. Modifying Format Control Flags

As mentioned in [Format Control](02-format-control.md), changing endianness, alignment, or pointer width features in `Cargo.toml` **will invalidate all your existing stored data**. Treat these flags as an immutable database contract.

## 4. `unsafe` Transmutation

**Anti-Pattern**: Using `std::mem::transmute` to convert from `&[u8]` to an `Archived<T>`.
**Fix**: Always use `rkyv::access` (for untrusted) or `rkyv::access_unchecked` (for trusted). They correctly handle pointer math and root object location calculation. `transmute` will fail because the root object is at the end of the buffer, not the beginning.
