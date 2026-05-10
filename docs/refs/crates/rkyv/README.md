# rkyv Reference Guide

**Version:** 0.8.x
**Official Docs:** [https://docs.rs/rkyv/latest/rkyv/](https://docs.rs/rkyv/latest/rkyv/)
**Guide:** [https://rkyv.org/](https://rkyv.org/)
**Repository:** [https://github.com/rkyv/rkyv](https://github.com/rkyv/rkyv)
**License:** MIT

This folder contains documentation and best practices for using the `rkyv` (archive) zero-copy deserialization framework within the Lithos project. It serves as a persistent reference derived from official documentation and project-specific guidelines.

## Contents

1. [Core Concepts & Zero-Copy Patterns](01-core-concepts.md)
2. [Components Index](02-components.md)
3. [Best Practices & Effective Usage](03-best-practices.md)
4. [Validation (`access` vs `access_unchecked`)](04-validation.md)
5. [Format Control & Compatibility](05-format-control.md)
6. [Pitfalls, Alignment Issues, & Anti-Patterns](06-pitfalls-and-patterns.md)
7. [Integrations (redb, mmap, etc.)](07-integrations.md)

## Applicability: rkyv vs Serde

- **Use `rkyv` when:** Performance and load times are your absolute top priority, especially for large datasets, IPC, or embedded databases (like `redb`). `rkyv` skips the parsing step entirely by mapping bytes directly into memory.
- **Use `serde` when:** You need cross-language interoperability, human-readable formats (JSON, TOML), or dynamic/schema-evolving capabilities, where the CPU overhead of parsing is acceptable.

## TL;DR

- **Zero-Copy**: `rkyv` accesses memory directly via `Archived<T>`.
- **Validation Boundary**: Untrusted data requires `rkyv::access`, trusted internal caches can use `rkyv::access_unchecked`.
- **API Pattern**: Use closure-based zero-copy extraction (`with_archived`) over returning `Archived` guards to avoid self-referencing struct issues.
- **DTO Boundaries**: Confine `#[derive(Archive)]` to explicit storage DTOs; avoid placing it ubiquitously across domain models.
