# Findings

## 2026-05-10
### rkyv Research
* **Core Concepts**: `rkyv` provides zero-copy deserialization by mapping `Archived<T>` directly onto memory. It uses traits like `Archive` and `Portable`. Supports zero-copy collections and deduplicates shared pointers natively.
* **Best Practices**: Use high-level APIs like `to_bytes`. Batch writes for Redb. Avoid ubiquitous `#[derive(Archive)]` over domain models; use explicit DTOs. Leverage `CopyOptimization` safely. Use `#[with(Inline)]` to optimize cache locality.
* **Validation**:
  * `rkyv::access`: Validates invariants. Use for network protocols, untrusted user data, file formats loaded from disk.
  * `rkyv::access_unchecked`: Bypasses validation. Use ONLY for trusted internal data or pre-validated paths.
* **Format Control**: Treat endianness, alignment, and pointer width as strict persisted-format contracts. E.g., `aligned` runs faster in-memory, but `unaligned` is recommended for mmap files.
* **Pitfalls**: Alignment traps with Mmap. Accidental deserialization overheads using `rkyv::deserialize(archived)`. Self-referential structs when returning guards (use closure-based extraction `with_archived`). Cyclic graphs are not natively handled well.
