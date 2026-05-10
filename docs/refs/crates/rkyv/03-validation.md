# Validation (`access` vs `access_unchecked`)

Validation is critical when memory-mapping byte buffers directly to structs. Treating random bytes as an `Archived<T>` without verification can lead to undefined behavior, panics, and severe memory safety vulnerabilities.

## `access` (Safe API)
The `rkyv::access` API mathematically validates the buffer against the invariants of the `Archived<T>` struct before returning a reference.
- **What it checks**: Pointer alignment, boundary limits, UTF-8 validity for strings, and valid enum discriminants.
- **When to use**: *Always* use this at system boundaries. This includes data arriving over network protocols, untrusted user inputs, and file formats loaded directly from disk.
- **Note**: This runtime validation requires the `bytecheck` feature to be enabled.

## `access_unchecked` (Unsafe API)
The `rkyv::access_unchecked` API completely bypasses all validation overhead, yielding instantaneous memory access.
- **When to use**: ONLY for strictly trusted internal data or pre-validated paths. It is invaluable for high-performance internal caches where the source of the bytes is absolutely guaranteed (e.g., data you wrote into memory during the same session).
- **Warning**: Using this on untrusted or corrupted data invokes undefined behavior immediately.

## Validation Scaling
The computational cost of `access` scales linearly with the complexity of your struct.
- **Best Practice**: Validate exclusively at the system boundary (e.g., upon reading from disk or network). Once validated, cache the reference or buffer and use `access_unchecked` (if strictly controlled) for downstream access. Do not repeatedly validate the same buffer.
