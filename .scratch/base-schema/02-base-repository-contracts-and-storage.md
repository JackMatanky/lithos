---
title: 02-base-repository-contracts-and-storage
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-06-01
---

## Type

AFK

## Labels

- base-schema
- ready-for-agent

## Parent

- `.scratch/base-schema/PRD.md`

## What to build

Add persistence seams for base schemas across repository traits and storage adapters so the processor can save/load base state by `SchemaId`.

This slice wires the read/write contract end-to-end for `BaseSchema` in repository traits, in-memory test repository, and redb table definitions.

## Acceptance criteria

- [ ] `ReadRepository`/`WriteRepository` expose `get_base_schema`, `find_base_schemas_by_ids`, `save_base_schema`, and `delete_base_schema`.
- [ ] `lithos-core/src/schema/storage/tables.rs` defines `BASE_SCHEMA_BY_ID` table.
- [ ] `lithos-core/src/schema/storage/read.rs` implements base-schema read methods.
- [ ] `lithos-core/src/schema/storage/write.rs` implements base-schema write/delete methods.
- [ ] `InMemoryRepository` supports the same base-schema methods for tests.
- [ ] Repository round-trip tests prove save/get/find/delete behavior and deterministic ID lookups.

## Blocked by

- `.scratch/base-schema/01-base-domain-and-deltas.md`
