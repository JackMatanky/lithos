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
- [ ] `From<StructuredFileFormat> for FileFormat` is implemented with `Yml -> FileFormat::Yaml`.
- [ ] Tests verify precedence rank ordering, extension mapping, and conversion semantics.

## Blocked by

None - can start immediately.

## Implementation notes

- Keep `StructuredFileFormat` as a selector type for discovery and candidate resolution.
- Keep `FileFormat` as parser/storage type; do not change parser dispatch signatures.
- Do not introduce reverse conversion from `FileFormat` to `StructuredFileFormat` (lossy for YAML/YML).
