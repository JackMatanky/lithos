# rkyv Reference Guide

This folder contains documentation and best practices for using the `rkyv` (archive) zero-copy deserialization framework within the Lithos project. It serves as a persistent reference derived from official documentation and project-specific guidelines.

## Contents

1. [Core Concepts & Zero-Copy Patterns](01-core-concepts.md)
2. [Best Practices & Effective Usage](02-best-practices.md)
3. [Validation (`access` vs `access_unchecked`)](03-validation.md)
4. [Format Control & Compatibility](04-format-control.md)
5. [Pitfalls, Alignment Issues, & Anti-Patterns](05-pitfalls-and-patterns.md)

## TL;DR

- **Zero-Copy**: `rkyv` accesses memory directly via `Archived<T>`.
- **Validation Boundary**: Untrusted data requires `rkyv::access`, trusted internal caches can use `rkyv::access_unchecked`.
- **API Pattern**: Use closure-based zero-copy extraction (`with_archived`) over returning `Archived` guards to avoid self-referencing struct issues.
- **DTO Boundaries**: Confine `#[derive(Archive)]` to explicit storage DTOs; avoid placing it ubiquitously across domain models.
