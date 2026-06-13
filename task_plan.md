# Task Plan: Discovery Context Contract Slice

## Goal
Implement the first vertical slice of the discovery module redesign from `.scratch/root-config-discovery/10-bootstrap-context-discovery-contracts.md`: Discovery-owned input/output/report/error/policy contracts plus a minimal app-owned Bootstrapper context acquisition seam.

## Current Phase
Complete

## Phases

### Phase 1: Orient and Bound Scope
- [x] Read issue, ADR/domain docs, graph/report context, and current discovery/bootstrapper code.
- [x] Identify public interfaces and target symbols for impact analysis.
- **Status:** complete

### Phase 2: Discovery Contracts via TDD
- [x] Add one failing behavior test for `DiscoveryContext`, `DiscoveryFlags`, and `DiscoveryEnv` construction.
- [x] Implement minimal contract types.
- [x] Add one failing behavior test for `CandidatePath` and `DiscoveryResult`.
- [x] Implement minimal output contract types.
- [x] Add one failing behavior test for `DiscoveryReport` metadata.
- [x] Implement report-only metadata types.
- [x] Add one failing behavior test for fatal discovery error taxonomy and policy names.
- [x] Implement minimal error and policy contracts.
- **Status:** complete

### Phase 3: Bootstrapper Context Seam via TDD
- [x] Add one failing behavior test proving app-owned sources build a `DiscoveryContext` without invoking DiscoveryService or Config.
- [x] Implement the minimal Bootstrapper seam and test source injection.
- **Status:** complete

### Phase 4: Verification
- [x] Run formatting.
- [x] Run targeted/unit tests for the touched crate.
- [x] Run lint if feasible.
- [x] Run full project tests.
- [x] Run GitNexus `detect_changes` before completion.
- **Status:** complete

## Decisions Made
| Decision | Rationale |
|----------|-----------|
| Use TDD vertical slices | Required by task and keeps the new boundary behavior-driven rather than shape-only. |
| Keep discovery execution out of scope | Issue explicitly limits this slice to contracts and context acquisition. |
| Add new contract modules alongside legacy engine/probe | Keeps this first slice minimal while preserving existing discovery behavior. |

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
