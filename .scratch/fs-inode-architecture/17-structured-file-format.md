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

- [ ] `StructuredFileFormat` exists in `fs/format.rs` with variants: `Toml`, `Json`, `Yaml`, `Yml`.
- [ ] `StructuredFileFormat::PRECEDENCE` is defined and ordered `Toml > Json > Yaml > Yml`.
- [ ] `StructuredFileFormat` exposes `extension()`, `precedence_rank()`, `from_extension(&str)`, and `from_path(&Path)` helpers.
- [ ] `from_extension(&str)` and `from_path(&Path)` return `Option<StructuredFileFormat>`.
- [ ] Selector matching is ASCII case-insensitive for extension inputs.
- [ ] `From<StructuredFileFormat> for FileFormat` is implemented with `Yml -> FileFormat::Yaml`.
- [ ] Tests verify precedence rank ordering, extension mapping, and conversion semantics.

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
   - precedence ordering and `precedence_rank()`
   - extension mapping and case-insensitive lookup
   - path mapping for known, unknown, and missing extensions
   - one-way conversion semantics into `FileFormat`
2. Implement minimal `StructuredFileFormat` enum + helpers + conversion to satisfy tests.
3. Add discovery tests proving configured property bank path remains absolute winner.
4. Integrate selector use into discovery candidate resolution without changing parse dispatch contracts.
5. Run unit and quality checks.
