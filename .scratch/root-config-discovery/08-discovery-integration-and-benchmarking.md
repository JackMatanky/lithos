---
title: 08-discovery-integration-and-benchmarking
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-06-08
---

## Type

AFK

## Labels

- root-config-discovery
- ready-for-agent

## Parent

- `.scratch/root-config-discovery/PRD.md`

## Status

**Supersedes**: `.scratch/root-config-discovery/08-phase-2-local-config-discovery-superseded.md`

## What to build

Implement integration testing and benchmarking for the `lithos-core/src/discovery` module.

The previous issue (08) focused on implementing the module itself, which has now been completed via the "Modular Engine" refactor (Issue 06). This issue covers the remaining verification requirements.

> **Note**: Benchmarks were scoped out. All discovery types are `pub(crate)` (crate-internal), so `benches/discovery.rs` would have no access to them. Benchmarking would require either public API exposure or a non-`#[cfg(test)]` bench helper module — deferred to a future issue.

## Acceptance criteria

- [ ] **Discovery module visibility reduced**: `pub mod discovery` → `pub(crate) mod discovery` in `lib.rs` to match actual public API surface (all types are `pub(crate)`).
- [ ] **Integration Testing Implementation**:
    - [ ] Create `src/discovery/tests.rs` (crate-internal `#[cfg(test)]` module — `tests/` cannot access `pub(crate)` types).
    - [ ] Implement tests using `tempfile` to create isolated filesystem trees.
    - [ ] **Testing Scenarios**:
        - **Ascending Walk**: Verify walking up parent directories works across multiple levels; **Rationale**: Core discovery mechanism.
        - **Ceiling Boundary Enforcement**: Verify walk stops correctly at configured ceilings; **Rationale**: Security/isolation boundary.
        - **Symlink Cycle Detection**: Verify probe handles self-referential symlinks gracefully without panic; **Rationale**: Prevent process hangs.
        - **Policy Precedence**: Verify Explicit > Env > Walk order; **Rationale**: Essential user-facing behavior.
        - **Ambiguity Handling**: Verify multiple markers return expected winner + alternatives; **Rationale**: Prevent hidden configuration selection.
- [ ] Documentation updated to reflect the new testing structure.

## Agent Brief

**Category:** enhancement
**Summary:** Implement integration tests and benchmarks for the `discovery` module to ensure stability and monitor performance regressions for filesystem traversal.

**Current behavior:**
The `discovery` module lacks comprehensive integration tests exercising real filesystem behavior (symlinks, complex directory structures, ceiling boundaries) and performance benchmarks for traversal and probing logic.

**Desired behavior:**
Implement integration tests in `src/discovery/tests.rs` (crate-internal, `#[cfg(test)]` module) following the project's integration testing standards. Benchmarks are deferred.

**Key interfaces:**
- `DiscoveryEngine` — core orchestrator to be tested
- `BoundedAscent` — traversal logic (not `AscendingWalker`, which was a stale name in CONTEXT.md)
- `VaultRootProbe` — probing logic
- `DiscoveryPolicy` — precedence logic

**Acceptance criteria:**
- [ ] Create `src/discovery/tests.rs` (not `tests/discovery.rs` — `pub(crate)` types are inaccessible from external crate tests).
- [ ] **Integration Tests**:
    - [ ] `tempfile` based filesystem tests for: Ascending Walk (multi-level), Ceiling Enforcement, Symlink Cycle Detection, Policy Precedence (Explicit > Env > Walk), Ambiguity Handling.
- [ ] Ensure all tests pass using `mise run test:unit:core`.

**Out of scope:**
- Modifying core discovery logic (unless bugs are found).
- Mocking or trait-based filesystems.
- Benchmarks (deferred — all types are `pub(crate)`, so `benches/` cannot access them without public API promotion).

## Implementation Notes

### Files created/modified

| File | Change |
|------|--------|
| `lithos-core/src/lib.rs:21` | `pub mod discovery` → `pub(crate) mod discovery` — module was exporting all items as `pub(crate)` but the module itself was `pub`, making it a dead public module |
| `lithos-core/src/discovery/mod.rs:65` | Added `#[cfg(test)] pub(crate) mod tests;` — gates the test module behind `cfg(test)` so it compiles away in release builds |
| `lithos-core/src/discovery/tests.rs` | **New file** — 6 integration tests using `tempfile` for real filesystem I/O |
| `lithos-core/src/discovery/CONTEXT.md:27` | Fixed stale `AscendingWalker` → `BoundedAscent` |

### Test coverage

| Test | What it exercises | Rationale |
|------|-------------------|-----------|
| `ascending_walk_resolves_marker_across_multiple_levels` | Creates `a/b/c/` nested hierarchy, places marker at `b/`, call starts from `c/` | Core discovery mechanism — walk must iterate parent dirs |
| `ceiling_stops_above_marker` | Start at `a/b/c/`, ceiling at `a/b/`, marker at `a/` | Security boundary — walk must not escape ceiling |
| `symlink_cycle_probe_does_not_panic` | Symlink `link -> ../link` creates self-reference | Robustness — panic protection for malformed FS |
| `explicit_flag_takes_precedence_over_env_and_walk` | Flag, env var, and walk source all present | Policy engine — user intent must win |
| `env_var_takes_precedence_over_ascending_walk` | Env var set, no flag, marker exists higher up | Policy engine — env overrides default walk |
| `multiple_markers_return_winner_and_alternatives` | `.lithos.toml`, `.lithos.yaml`, `.lithos.json` in same dir | Ambiguity — explicit TOML wins, JSON is alternative |

### Key decisions

1. **Crate-internal `#[cfg(test)]` over `tests/discovery.rs`**: Rust's `tests/` directory compiles as an external crate, which cannot access `pub(crate)` items. Since all discovery types are `pub(crate)` (phase-1 seams with `#[allow(dead_code)]`), widening visibility just for testing was rejected per community consensus. The test module lives inside `src/discovery/` and is conditionally compiled with `#[cfg(test)]`.

2. **Benchmarks not implemented**: Same visibility barrier — `benches/` is also an external crate. Beyond that, benchmarking traversal code without real integration into Config (which provides the DiscoveryEngine wiring) would produce misleading results. Deferred to a future issue.

3. **Real filesystem via `tempfile` over mock/trait-based**: The discovery code uses `std::fs` directly (no trait abstraction). `tempfile::TempDir` creates real temporary directories, avoiding the need for filesystem abstraction and keeping tests authentic.

4. **`pub mod discovery` → `pub(crate) mod discovery`**: The module was exported as `pub` but all its items were `pub(crate)`, making it a "dead public module" — visible externally but with no usable API. Corrected to match actual public surface.

### Deviations from plan

- **Test file location**: Changed from `tests/discovery.rs` to `src/discovery/tests.rs` (visibility constraint discovered during implementation)
- **Stale type name**: Fixed `AscendingWalker` → `BoundedAscent` in CONTEXT.md (referenced a renamed type)
- **Benchmarks**: Scoped out entirely (visibility barrier + lack of Config wiring meant benchmarks would be premature)

### Verification

```
mise run verify     — fmt ✓, lint ✓, all tests ✓ (1589 unit, 49 integration)
```

## Blocked by

- `.scratch/root-config-discovery/06-discovery-cleanup-and-integration.md`
