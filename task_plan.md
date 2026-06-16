# Task: Redesign Schema Error Module

Redesign `lithos-core/src/schema/error.rs` to align with Hexagonal Architecture and remove duplication with `fs` context errors.

## !!! CRITICAL: NO IMPLEMENTATION YET !!!
**Your primary directive is to finalize the planning documentation. Do NOT start implementation until the planning phase is verified and explicitly approved by the user.**

## Goal
A pure domain error core (`SchemaError`) surrounded by port-specific and orchestration errors, utilizing existing `fs` error types where appropriate.

## Phases
- [x] Phase 1: Initialize Planning <!-- id: 0 -->
- [x] Phase 2: Finalize Planning & Verify Enums (STRICT: NO CODE) <!-- id: 1 -->
- [x] Phase 3: Domain Error Redefinition (`SchemaNameError`, `PropertyNameError`) <!-- id: 2 -->
    - [x] Redefine `SchemaNameError` with `NameIsEmpty`, `NameExceedsMaxLength`, `ContainsInvalidCharacters`, `RegexCompilationFailed`
    - [x] Redefine `PropertyNameError` with `NameIsEmpty`, `NameExceedsMaxLength`, `ContainsInvalidCharacters`, `RegexCompilationFailed`
    - [x] Remove `SchemaSyntaxError` and flatten into `SchemaError`
    - [x] Update all callers of `SchemaNameError`, `PropertyNameError`, and `SchemaSyntaxError`
- [x] Phase 4: Property Specification & Value Error Refactor (REQUIRES USER CONSENT) <!-- id: 3 -->
    - [x] Define specialized spec errors (`StringSpecError`, `NumberSpecError`, `DateSpecError`, `FileSpecError`)
    - [x] Define specialized value validation errors (`StringValueValidationError`, `NumberValueValidationError`, `DateValueValidationError`, `FileValueValidationError`)
    - [x] Refactor `PropertySpecError` to wrap specialized spec errors using `#[error(transparent)]`
    - [x] Refactor `PropertyValueError` to wrap specialized value validation errors and `IncorrectPrimitiveType`
    - [x] Update callers in `property_spec/*.rs` and `property.rs`
    - [x] Verify with build and tests
- [x] Phase 5: Reference & Map Error Refactor <!-- id: 4 -->
    - [x] Perform impact analysis on `PropertyRefError` and `PropertyBankError`
    - [x] Refactor `PropertyRefError` variants in `error.rs`
    - [x] Implement `PropertyBuilderError` in `error.rs`
    - [x] Rename `PropertyBankError` to `PropertyMapError` and update variants
    - [x] Update `SchemaError` umbrella and mapping logic
    - [x] Update all callers and fix breaking changes
    - [x] Verify with `mise run build` and `mise run test --package lithos-core`
- [ ] Phase 6: Cleanup & Orchestration Redesign <!-- id: 5 -->
    - [x] Phase 6a: Ingestion Refactor <!-- id: 7 -->
        - [x] Create `SchemaReadError` wrapping `crate::fs::error::ReadError`
        - [x] Create `SchemaParseError` wrapping `crate::fs::error::ParseError`
        - [x] Update `SchemaIngestionError` to consolidate file/parse logic
    - [x] Phase 6b: Resolution Refactor <!-- id: 8 -->
        - [x] Refine `SchemaInheritanceError` (graph cycles, missing nodes)
        - [x] Refine `SchemaResolutionError` (semantic conflicts, name duplicates)
        - [x] Ensure `Box<str>` usage and clean up redundant variants in callers
        - [x] Update all callers of inheritance/resolution errors
    - [ ] Phase 6c: Orchestration Redesign <!-- id: 9 -->
        - [x] Task 6c.1: String Transition & Type Hardening <!-- id: 9 -->
            - [x] 6c.1.1: Replace `Box<str>` with `String` in `error.rs`
            - [x] 6c.1.2: Propagate `String` transition through `schema` module (raw types, processors, views)
        - [x] Task 6c.2: Circularity & Repository Refactor
            - [x] 6c.2.1: Remove `Domain(SchemaError)` from `SchemaRepositoryError`
            - [x] 6c.2.2: Audit and fix Repository usages in `schema/storage/` and `schema_processor.rs`
        - [ ] Task 6c.3: Orchestration Layer Redesign (Incremental Migration)
            - [x] 6c.3.1: Define `SchemaBuilderError` and update `SchemaError` umbrella
            - [x] 6c.3.2: Migrate `SchemaDiscovery` & `SchemaDelta` (Low complexity)
            - [x] 6c.3.3: Migrate `PropertyBankProcessor` & `BaseSchemaProcessor` (Medium complexity)
            - [ ] 6c.3.4: Migrate `SchemaProcessor` (High complexity - migration chunking)
        - [ ] Task 6c.4: Legacy Cleanup & Final Verification
            - [ ] 6c.4.1: Remove `SchemaIngestionError` & `SchemaLoaderError`
            - [ ] 6c.4.2: Final Audit of Error Mapping & Documentation
    - [ ] Phase 7: Verification <!-- id: 6 -->

## Strategy
1. **Remove Duplication**: Replace internal `SchemaReadError` and `SchemaParseError` by importing `ParseError` and `ReadError` from `crate::fs::error` and mapping them via `SchemaIngestionError`.
2. **Domain Purity**: `SchemaError` remains the central umbrella for semantic failures.
3. **Hexagonal Ports**: `SchemaRepositoryError` handles outbound persistence failures.
4. **Service Layer**: `SchemaBuilderError` orchestrates pipeline failures.
5. **Commit Strategy**: Stage and commit changes frequently (after every task or logical sub-step) using the `/caveman-commit` skill to ensure a clean, incremental history.
