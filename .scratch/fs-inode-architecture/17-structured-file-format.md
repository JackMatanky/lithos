---
title: 17-structured-file-format
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-05-31
---

## Type

AFK

## Labels

- fs-inode-architecture
- ready-for-agent

## Parent

- `.scratch/centralized-discovery-processor/PRD.md`

## What to build

Introduce a `StructuredFileFormat` selector type as a codebase invariant for structured file discovery, with explicit precedence and stable extension mapping. Keep parser/storage semantics on `FileFormat`, and provide one-way conversion from `StructuredFileFormat` to `FileFormat` for parse-time handoff.

This slice must make structured candidate selection deterministic for `.toml`, `.json`, `.yaml`, and `.yml`, while preserving existing `FileFormat` behavior where `.yaml` and `.yml` are parsed as YAML.

## Acceptance criteria

- [x] `StructuredFileFormat` exists in `fs/format.rs` with variants: `Toml`, `Json`, `Yaml`, `Yml`.
- [x] `StructuredFileFormat::PRECEDENCE` is defined and ordered `Toml > Json > Yaml > Yml`.
- [x] `StructuredFileFormat` exposes `extension()`, `rank()`, `from_extension(&str)`, and `from_path(&Path)` helpers.
- [x] `from_extension(&str)` and `from_path(&Path)` return `Option<StructuredFileFormat>`.
- [x] Selector matching is ASCII case-insensitive for extension inputs.
- [x] `From<StructuredFileFormat> for FileFormat` is implemented with `Yml -> FileFormat::Yaml`.
- [x] Tests verify precedence/rank ordering, extension mapping, and conversion semantics.

## Blocked by

None - can start immediately.

## Implementation notes

- Keep `StructuredFileFormat` as a selector type for discovery and candidate resolution.
- Keep `FileFormat` as parser/storage type; do not change parser dispatch signatures.
- Do not introduce reverse conversion from `FileFormat` to `StructuredFileFormat` (lossy for YAML/YML).
- Discovery must honor configured property bank path as absolute winner when selecting candidates.
- Implement in vertical TDD slices (red-green-refactor), with unit naming/structure per `docs/engineering/testing/unit.md` and `docs/engineering/testing/unit-naming.md`.

## Approved clarifications (2026-05-31)

- `StructuredFileFormat::from_extension(&str)` returns `Option<StructuredFileFormat>`.
- `StructuredFileFormat::from_path(&Path)` returns `Option<StructuredFileFormat>`.
- Extension matching for selector helpers is ASCII case-insensitive.
- Discovery selection keeps configured property bank path as absolute winner.
- `StructuredFileFormat` should mirror `FileFormat` traits where practical (`Archive`, `Serialize`, `Deserialize`) to keep future config/discovery usage flexible.

## TDD execution plan

1. Add/adjust unit tests for `StructuredFileFormat` in `fs/format.rs`:
   - precedence ordering and `rank()`
   - extension mapping and case-insensitive lookup
   - path mapping for known, unknown, and missing extensions
   - one-way conversion semantics into `FileFormat`
2. Implement minimal `StructuredFileFormat` enum + helpers + conversion to satisfy tests.
3. Add discovery tests proving configured property bank path remains absolute winner.
4. Integrate selector use into discovery candidate resolution without changing parse dispatch contracts.
5. Run unit and quality checks.

## Implementation status (2026-05-31)

- Implemented `StructuredFileFormat` and selector helpers in `lithos-core/src/fs/format.rs`.
- Selector rank helper was finalized as `rank()` (renamed from `precedence_rank()`).
- Added `From<StructuredFileFormat> for FileFormat` with `Yml -> FileFormat::Yaml`.
- Discovery extension filtering now derives from `StructuredFileFormat::PRECEDENCE`.
- Added regression coverage ensuring configured property bank path remains absolute winner.
- Normalized `fs/format.rs` unit suite to Structure A with canonical modules (`integrity`, `lookup`, `conversions`, `validation`, `parse`) and verb-first test names.
- Unit + lint + formatting checks pass in the dedicated worktree.

## Implementation commits

- `1d6ddc74` `feat(fs): add structured file format selector`
- `73bd5947` `test(fs): normalize format suite and rename rank`
