---
feature: Benchmarks + rkyv reality notes
status: Draft # Options: Draft, In Review, Approved, Implemented, Archived
author: Jack Matanky (drafted with GitHub Copilot)
date_created: 2026-02-03
tags: [benchmarks, rkyv, redb, performance, projections]
---

# Benchmarks + rkyv Reality Notes

This document captures the key takeaways and implementation guidance from the
benchmarking and rkyv discussion, grounded in Lithos’ architecture and redb
constraints.

## 1. Terminology (keep claims honest)

- **Zero-copy** (strict): no allocation and no memcpy from storage buffers.
- **Zero-deserialize**: we avoid building an owned runtime model; we compute a
  small owned result `R` from archived bytes.

Lithos’ hot-path goal is typically “zero-deserialize.” Strict zero-copy may not
be achievable with redb unless alignment guarantees are established.

## 2. redb + rkyv constraint: alignment

- redb does not guarantee that `&[u8]` returned from a table is suitably aligned
  for rkyv’s archived references.
- rkyv’s safe access (`rkyv::access` + bytecheck validation) may therefore
  require copying bytes into an aligned buffer first.

Implication:

- Hot-path reads are still valuable (no full deserialization), but they may pay
  an **alignment copy** cost.
- Benchmarks must measure the *real* cost (including alignment staging), not an
  idealized “pure zero-copy” path.

## 3. rkyv “unaligned” format-control lever

rkyv provides a format-control option that can make archived primitives
1-aligned (“unaligned”).

Tradeoff:

- This changes the persisted format contract.
- Adopting it requires an explicit migration story and should be treated as a
  storage-format decision.

## 4. Workload shape: steady-state is read-heavy

Lithos’ steady-state interaction model (CLI/LSP-style queries, alias
resolution, metadata lookups, schema/property resolution) is read-heavy.
Writes are bursty (indexing, rebuilds, migrations).

Therefore:

- Prioritize **read latency** for “instant queries”.
- Use projections/indexes to avoid deserializing whole aggregates.

## 5. Projections: read-optimized indexes for instant queries

The general query primitive for Obsidian-like metadata is:

- “frontmatter field *K* contains value *V*”.

Specializations (usually also frontmatter-driven):

- aliases (often array of strings)
- file_class (string)
- title (string)

Recommended projection tables (logical):

- `alias_to_id`: alias -> note id
- `folder_to_id`: folder path -> note id
- `file_class_to_id`: file_class -> note id
- `frontmatter_kv_to_id`: composite(frontmatter_key, value) -> note id

Notes:

- `alias_to_id` and `file_class_to_id` are conceptually special cases of
  `frontmatter_kv_to_id`, but are worth keeping explicit if they are hot and
  central to UX.
- Frontmatter field names should be treated as config-driven (do not hard-code
  `aliases`, `file_class`, `title`).

## 6. Benchmark implementation plan (Lithos-specific)

### 6.1 Principles

- Separate **setup** from measurement: dataset creation and indexing should not
  be in the timed loop.
- Use benchmarks that model real queries:
  - alias resolution
  - list files in folder
  - list by file_class
  - generic frontmatter field contains value
  - schema property lookup
- Include a “slow path” baseline (scan + filter) to quantify why projections
  exist.

### 6.2 Benchmark tiers to measure

For each query, measure the tiers you will actually ship:

1) **Index-only**: multimap lookup returning ids/paths.
2) **Index + materialize**: follow ids to load notes when a command needs full
   objects.
3) **Archived compute**: closure-based archived reads computing a small `R`.
4) **Owned**: `get_owned` (cold path baseline for mutations).

### 6.3 Suggested Criterion structure

- `lithos-core/benches/db_perf.rs`: keep as the storage micro-benchmark suite.
- Add a new suite for end-to-end query shapes (example names):
  - `lithos-core/benches/instant_queries.rs`
  - `lithos-core/benches/schema_queries.rs`

Each suite should:

- Build a synthetic dataset (e.g., 1k then 10k notes) with realistic
  distributions.
- Include hot-key vs cold-key cases (aliases with 1 match vs 50 matches).
- Report throughput in “queries/sec” and optionally “notes/sec” for indexing.

## 7. Maximize rkyv value without derive-everything

- Keep `serde` primarily at *external boundaries* (configs, user-facing
  formats, import/export).
- Use rkyv for *persistence* and *hot reads*.
- Avoid coupling domain ergonomics to storage encoding by:
  - keeping projection/index tables storage-shaped (strings / byte keys), and
  - exposing “archived compute” helpers on concrete query types (not `dyn`
    port traits).

## 8. Next decisions (explicit)

- Decide whether to accept alignment staging copies as the steady-state cost, or
  adopt a persisted-format change (e.g., rkyv unaligned) with a migration plan.
- Implement projections only for queries that are confirmed hot via
  benchmarks; avoid speculative indexes.
