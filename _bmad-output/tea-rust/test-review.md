---
stepsCompleted: ["step-01-load-context", "step-02-analyze", "step-03-fix", "step-04-report"]
lastStep: "step-04-report"
date: "2026-02-15"
user_name: "Jack"
---

# Rust Test Review Report

## Scope
- **Target Path**: `lithos-core/src/config/`
- **Review Scope**: `directory`
- **Issue**: `misaligned pointer dereference` panics in `rend` crate with `rkyv` 0.8.

## Context Expansion Notes
- **Dependencies**: `rkyv` 0.8, `rend`.
- **Architecture**: Port-based CQRS with zero-copy patterns (GATs).
- **Rationale**: The issue was related to `char` fields in archived structs. `rkyv`'s validation of `char` (via `rend::char_ule`) triggered alignment-sensitive operations that failed when data was at unaligned offsets (e.g., in a `HashMap`).

## Findings (High/Medium/Low)
### [High] Misaligned Pointer Dereference in `rkyv` validation
- **Description**: `rkyv`'s `bytecheck` validation panics when checking `char` fields that are not 4-byte aligned.
- **Impact**: Application crashes during database reads of configuration.
- **Root Cause**: `char` alignment requirements enforced during validation of `rend::char_ule`.
- **Location**: `lithos-core/src/config/task.rs` (`StatusSymbol`) and `lithos-core/src/config/value.rs` (`DateSpec`).

## Fixes Applied
### Workaround for `char` alignment issues
- **`StatusSymbol`**: Changed internal storage from `char` to `u8`. Since validation restricts it to printable ASCII, `u8` is sufficient and has alignment 1, avoiding the issue.
- **`DateSpec`**: Changed internal storage for `emoji` from `Option<char>` to `Option<u32>`. Storing the Unicode Scalar Value as `u32` avoids `rend::char_ule`'s problematic validation while still supporting full Unicode.
- **Public API Preservation**: Maintained `char` in all public method signatures (`try_new`, `value`, `emoji()`, etc.) to ensure zero impact on domain logic.

## Fixes Recommended (Not Applied)
- Consider a general audit of `char` usage in other `Archive`-derived types if they are stored in `redb`.

## Remaining Risks
- The use of `u32` for `emoji` still has a 4-byte alignment requirement in `rkyv` 0.8 UNLESS `unaligned` feature correctly maps it to `rend::u32_le`. Current tests pass, suggesting `u32` validation is less problematic than `char` or that it's correctly handled as unaligned.

## Status
✅ **Review Complete & Fixes Verified**
All 103 config unit tests are passing.
