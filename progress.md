# Progress: Schema Error Redesign

## Status
- Phase 1: Planning initialized - COMPLETED
- Phase 2: Finalize Planning & Verify Enums - COMPLETED
- Phase 3: Domain Error Redefinition (SchemaNameError, PropertyNameError) - COMPLETED
- Phase 4: Property Specification & Value Error Refactor - COMPLETED
- Phase 5: Reference & Map Error Refactor - COMPLETED
- Phase 6: In progress
    - Phase 6a: Ingestion Refactor - COMPLETED
    - Phase 6b: Resolution Refactor - COMPLETED
- Phase 6c: Orchestration Redesign - IN_PROGRESS
    - Task 6c.1: String Transition & Type Hardening - COMPLETED
    - Task 6c.2: Circularity & Repository Refactor - COMPLETED
    - Task 6c.3: Orchestration Layer Redesign - IN_PROGRESS
        - 6c.3.1: Define Orchestration Structures - COMPLETED
    - Task 6c.4: Legacy Cleanup & Final Verification - PENDING
- Phase 7: Verification - PENDING

## Activity Log
### 2026-06-16 (Current Session)
- [6c.3.1] Defined `SchemaBuilderError` in `error.rs`.
- [6c.3.1] Updated `SchemaError` umbrella to wrap `SchemaBuilderError` and `SchemaRepositoryError` (boxed).
- [6c.3.1] Implemented manual `Clone` and `PartialEq` for `SchemaError`, `SchemaBuilderError`, and `SchemaReadError` to preserve system-wide observability while wrapping non-cloneable I/O errors.
- [6c.3.1] Fixed breaking changes in `events.rs`, `expander.rs`, and `schema_processor.rs` caused by moving variants and dropping automatic `Clone`/`PartialEq`.
- [6c.3.1] Verified stability with `mise run verify` (1975 tests passing).
- [6c.3.1] Staged and committed changes.
