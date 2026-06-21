# Task Plan: Fix clippy lint errors in lithos-core

## Goals
- Fix unused imports/variables. (Checked, seems done by clippy automatically or not needed)
- Fix shadowing in `lithos-core/src/config/builder.rs` tests.
- Fix type complexity in `lithos-core/src/discovery/service.rs`.
- Verify all fixes with `mise run test`.

## Phases
1. [x] Run `mise run lint` and log specific errors in `findings.md`.
2. [x] Fix unused imports/variables. (Fixed by clippy)
3. [x] Fix shadowing in `lithos-core/src/config/builder.rs`.
4. [x] Fix type complexity in `lithos-core/src/discovery/service.rs`.
5. [x] Verify with `mise run test`.
