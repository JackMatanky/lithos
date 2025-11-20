# Hybrid Performance & Configuration Guide

Date: 2025-11-20

This document fulfills Story 3.31 requirements by consolidating hybrid architecture, performance benchmarks, configuration options, and query optimization guidance for the BoltDB + SQLite vault indexing engine.

## 1. Hybrid Architecture Overview

- Hybrid storage: BoltDB for hot-path lookups (<1 ms) and SQLite for deep-path metadata queries (<50 ms).
- Query routing aligns with Story 3.23: ByPath/ByBasename/ByAlias → BoltDB, frontmatter queries → SQLite MetadataQueryPort.
- Storage write coordination relies on CacheUnitOfWork (Story 3.22) to keep both stores consistent.

## 2. Performance Benchmarks

| Operation | Target | Representative Result | Notes |
| --------- | ------ | --------------------- | ----- |
| BoltDB ByPath/ByBasename/ByAlias | < 1 ms | TBD after Feature 15 benchmarks (Story 3.20) | Record actual numbers after benchmark suite runs |
| BoltDB Persist/Delete | < 10 ms | TBD | Capture per Story 3.20 Feature 15 |
| SQLite FileClassQuery / FrontmatterQuery | < 50 ms (1000 notes) | TBD | Measure via Story 3.21 Feature 4 |
| CLI Template Query end-to-end | < 100 ms | TBD | Validate in Story 3.30 E2E suite |

> Update the "Representative Result" column once benchmark evidence is captured (see Story 3.20 Performance Targets).

## 3. Configuration Guide

- `FileClassKey` (Story 3.29): default `"fileClass"`; supports custom keys like `"type"`, `"category"`, `"kind"`. Update `lithos.yaml` or env var to change.
- `CacheDir`, `SQLitePath`, `BoltDBPath`: ensure writable locations; used by DI wiring in Story 3.30.
- `file_class_key` variations must be exercised in Story 3.30 E2E tests.

## 4. Query Optimization Tips

1. Prefer BoltDB-backed lookups (ByPath/Basename/Alias) when possible to stay under 1 ms.
2. For complex frontmatter filters, define schema-driven views via Story 3.21 to benefit from SQLite indexing.
3. Monitor staleness logs (Story 3.23) to detect when incremental refresh is required; stale entries degrade query accuracy.
4. Use CacheUnitOfWork to batch writes and reduce transaction overhead (<10% per Story 3.22).

## 5. References

- Story 3.20: BoltDB hot cache implementation
- Story 3.21: SQLite deep storage & schema-driven views
- Story 3.22: Storage write coordination
- Story 3.23: QueryService hybrid routing
- Story 3.29: FileClassKey configuration
- Story 3.30: DI wiring and production-scale validation
- Story 3.30 (Event-Driven): EventBus integration for CQRS
