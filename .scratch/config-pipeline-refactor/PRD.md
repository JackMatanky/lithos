# PRD: Config Pipeline Refactor

**Status**: draft
**Created**: 2026-05-31
**Triage**: ready-for-agent

## Problem Statement

Lithos config processing currently couples legacy view types, generic processor abstractions, and field-hash mechanics that do not align with the desired boundary-change detection model. This makes the pipeline harder to evolve toward deterministic discovery-boundary behavior and clearer module ownership.

## Solution

Refactor the config pipeline into explicit global/local processors with schema-style hash contracts, replacing legacy field-hash plumbing with `ConfigHashView`-based change detection, and modernizing builder orchestration and visibility boundaries.

## User Stories

1. As a config maintainer, I want separate global and local processors, so that context-specific behavior is explicit.
2. As a reliability-focused engineer, I want content and entry hash tracking in config views, so that boundary changes are detected deterministically.
3. As a discovery maintainer, I want boundary-change actions modeled explicitly, so that scan scope decisions are predictable.
4. As a maintainer, I want config exclusion patterns represented as validated types, so that path matching behavior is safer.
5. As an architecture reviewer, I want legacy `ConfigFieldHashes` removed, so that hashing logic follows one consistent model.
6. As a builder maintainer, I want cleaner orchestration boundaries, so that downstream contexts consume stable config specs.
7. As a test author, I want raw-type hash computation methods, so that hash behavior is tested at the right seam.
8. As an operator, I want trace logs through the processor pipeline, so that stale/fresh decisions are diagnosable.
9. As a migration owner, I want compatibility sequencing during view-type transition, so that rollout can be staged safely.
10. As a performance-focused engineer, I want duplicate hash computations reduced, so that config ingestion remains efficient.

## Implementation Decisions

- Introduce `ConfigHashView` as the config analogue of schema hash records (content hash + entry hashes).
- Add hash computation methods on raw config types to produce `ConfigHashView`-compatible data.
- Add/replace config views with `GlobalConfigFileView` and `LocalConfigFileView` that embed metadata and hash state.
- Split processor responsibilities into `GlobalConfigProcessor` and `LocalConfigProcessor` while preserving typestate discipline.
- Remove `ConfigFieldHashes` from processor internals.
- Add a `BoundaryChange` model with required action mapping.
- Add `config/exclusions.rs` with validated exclusion primitives for discovery filtering.
- Refactor builder orchestration to depend on narrower processor/view contracts and reduce unnecessary exposure.
- Add tracing at key processor and orchestration transitions.

## Testing Decisions

- Good tests validate external behavior: stale/fresh classification, boundary-change outputs, processor outcomes, and orchestration decisions.
- Test modules: raw hash computation, ConfigHashView behavior, processor split behavior, boundary action mapping, builder integration, exclusion matching.
- Prior art: schema hash/view tests, config builder tests, and repository seam tests.

## Out of Scope

- Generic event-sourcing infrastructure.
- Root discovery algorithm design.
- Filesystem discovery processor and context routing.
- Parallel orchestration strategy changes.

## Further Notes

- Migration should be staged: introduce new view/hash contracts, wire processors, switch builder/repository paths, then remove legacy artifacts.
