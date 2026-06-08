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

The previous issue (08) focused on implementing the module itself, which has now been completed via the "Modular Engine" refactor (Issue 06). This issue covers the remaining verification and performance-monitoring requirements.

## Acceptance criteria

- [ ] **Integration Testing Implementation**:
    - [ ] Create `tests/discovery.rs`.
    - [ ] Implement tests using `tempfile` to create isolated filesystem trees.
    - [ ] **Testing Scenarios**:
        - **Ascending Walk**: Verify walking up parent directories works across multiple levels; **Rationale**: Core discovery mechanism.
        - **Ceiling Boundary Enforcement**: Verify walk stops correctly at configured ceilings; **Rationale**: Security/isolation boundary.
        - **Symlink Cycle Detection**: Verify protection against infinite loops; **Rationale**: Prevent process hangs.
        - **Policy Precedence**: Verify Explicit > Env > Walk order; **Rationale**: Essential user-facing behavior.
        - **Ambiguity Handling**: Verify multiple markers return expected winner + alternatives; **Rationale**: Prevent hidden configuration selection.
- [ ] **Benchmarking Implementation**:
    - [ ] Create `benches/discovery.rs` using `criterion`.
    - [ ] **Benchmarking Scenarios**:
        - **Cold-Cache Traversal**: Walk from a deep subdirectory in a large, nested structure; **Rationale**: Measures worst-case latency for standard usage.
        - **Directory Probing**: Probing directories with varying marker file counts; **Rationale**: Measures per-directory I/O overhead.
        - **Precedence Resolution**: `DiscoveryEngine` overhead with competing sources; **Rationale**: Validates efficient policy matching.
- [ ] Documentation updated to reflect the new testing/benchmarking structure.

## Agent Brief

**Category:** enhancement
**Summary:** Implement integration tests and benchmarks for the `discovery` module to ensure stability and monitor performance regressions for filesystem traversal.

**Testing Strategy:**
Use `tempfile` for creating real, isolated directory trees. This ensures accurate reproduction of OS-level filesystem behavior, which is critical for discovery logic (canonicalization, symlinks, permissions).

**Benchmarking Strategy:**
Use `criterion` to measure:
1. Traversal speed (AscendingWalker performance).
2. Probing cost (Directory-level I/O).
3. Overall `DiscoveryEngine` overhead.

**Out of scope:**
- Modifying core discovery logic (except to fix bugs discovered during testing).
- Mocking or trait-based filesystems (stick to `tempfile`).

## Blocked by

- `.scratch/root-config-discovery/06-discovery-cleanup-and-integration.md`
