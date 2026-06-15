# Task: Redesign Schema Error Module

Redesign `lithos-core/src/schema/error.rs` to align with Hexagonal Architecture and remove duplication with `fs` context errors.

## !!! CRITICAL: NO IMPLEMENTATION YET !!!
**Your primary directive is to finalize the planning documentation. Do NOT start implementation until the planning phase is verified and explicitly approved by the user.**

## Goal
A pure domain error core (`SchemaError`) surrounded by port-specific and orchestration errors, utilizing existing `fs` error types where appropriate.

## Phases
- [x] Phase 1: Initialize Planning <!-- id: 0 -->
- [x] Phase 2: Finalize Planning & Verify Enums (STRICT: NO CODE) <!-- id: 1 -->
- [in_progress] Phase 3: Domain Error Redefinition (`SchemaNameError`, `PropertyNameError`) <!-- id: 2 -->
- [ ] Phase 4: Property Specification & Value Error Refactor <!-- id: 3 -->
- [ ] Phase 5: Reference & Map Error Refactor <!-- id: 4 -->
- [ ] Phase 6: Cleanup & Orchestration Redesign <!-- id: 5 -->
- [ ] Phase 7: Verification <!-- id: 6 -->

## Strategy
1. **Remove Duplication**: Replace internal `SchemaFileError` and `SchemaParseError` by importing `ParseError` and `ReadError` from `crate::fs::error` and mapping them via `SchemaIngestionError`.
2. **Domain Purity**: `SchemaError` remains the central umbrella for semantic failures.
3. **Hexagonal Ports**: `SchemaRepositoryError` handles outbound persistence failures.
4. **Service Layer**: `SchemaBuilderError` orchestrates pipeline failures.
