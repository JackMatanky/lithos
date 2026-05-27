# Task Plan - Property Bank Processor Test Normalization & Visibility Reduction

## Goal
Improve the unit test suite for `PropertyBankProcessor` by normalizing it to project standards, consolidating redundant integration tests, and reducing the visibility of intermediate typestates.

## Phases
| Phase | Description | Status |
|-------|-------------|--------|
| 1 | Review & Design | pending |
| 2 | Consolidate Integration Tests | pending |
| 3 | Reduce Visibility | pending |
| 4 | Normalize Unit Tests | pending |
| 5 | Verification | pending |

## Decisions
- Move `tests/property_bank_processor.rs` unique logic to `tests/schema_loader.rs` and delete the file.
- Reduce visibility of all intermediate typestates to private in `schema/property_bank_processor.rs`.
- Normalize `schema/property_bank_processor.rs` unit tests to "Structure A" with submodules.
