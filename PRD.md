## Problem Statement

Lithos currently repeats filesystem discovery logic across multiple contexts (notably Schema and Config), while Vault already maintains file and directory identity tables. This duplication increases maintenance cost, creates inconsistent discovery behavior, and makes it harder to evolve freshness checks and indexing safely.

The project needs a unified discovery engine that starts from filesystem primitives, persists canonical file/directory identity once, and then lets each context perform context-specific processing without re-implementing base discovery.

## Solution

Refactor the existing Vault module into the initial base of a discovery module (incrementally, without a full module move in this session). The discovery typestate processor will become the shared filesystem discovery engine and will:

- Run scoped scans using configurable scan input.
- Compare scan results against persisted views to classify freshness.
- Persist only deltas (new, stale metadata updates, deletions) rather than rewriting all records.
- Return a discovery result contract for context-specific processors.

Context processors (Schema, Note, Template, Config) remain standalone and consume discovery results as their first stage, then continue with context-specific parsing, hashing, validation, and persistence.

## User Stories

1. As a Lithos maintainer, I want one base discovery engine, so that file discovery behavior is consistent across contexts.
2. As a Lithos maintainer, I want to avoid duplicate scan code in Schema and Config, so that refactors are safer and faster.
3. As a Schema processor maintainer, I want discovery to provide canonical file identity, so that SchemaId can be replaced by FileId.
4. As a Note processor maintainer, I want discovery classifications (new/stale/unchanged/deleted), so that note ingestion can skip unnecessary work.
5. As a Template processor maintainer, I want indexed file metadata query support, so that template querying can evolve toward Obsidian-like behavior.
6. As a Config processor maintainer, I want config discovery to run first in orchestrated flows, so that downstream processors use resolved configuration.
7. As a performance-focused engineer, I want delta persistence in discovery, so that indexing avoids full table rewrites.
8. As a cross-platform user, I want safe normalized storage keys and valid filesystem read paths, so that indexing works consistently on all OSes.
9. As an architecture reviewer, I want context processors to stay standalone after discovery, so that downstream stages can vary independently.
10. As a concurrency-focused engineer, I want independent context processors after discovery, so that compatible processors can run in parallel.
11. As a reliability-focused engineer, I want deletion detection in base discovery, so that stale records are pruned deterministically.
12. As a schema maintainer, I want property-bank and schema files to be filtered from shared discovery results, so that schema logic focuses only on domain concerns.
13. As a config maintainer, I want structured-file freshness checks in config-owned views, so that config can preserve semantic hash behavior.
14. As a note/template maintainer, I want lightweight content freshness checks, so that unstructured files can use simpler hash records.
15. As a persistence maintainer, I want read/write repository seams preserved, so that storage adapters remain testable and swappable.
16. As a developer onboarding to Lithos, I want discovery responsibilities clearly separated from context parsing responsibilities, so that module boundaries are easier to understand.
17. As a test author, I want deterministic discovery result classification, so that tests can validate behavior without relying on implementation details.
18. As a future refactor owner, I want this change staged incrementally from Vault, so that migration risk stays manageable.

## Implementation Decisions

- Discovery remains an incremental refactor from the current Vault module first; full module renaming/re-homing is deferred.
- The discovery typestate processor lives in the discovery layer and is the shared base filesystem discovery engine.
- Discovery processing is scoped by scan input and should support partial or targeted scans.
- Discovery includes a comparison phase and classifies records by freshness; it does not only scan-and-write blindly.
- Discovery persists deltas and should call batch repository operations for efficient writes/deletes.
- Context processors remain standalone and consume discovery results as input to their own pipelines.
- FileId and DirId become canonical identity across contexts for file-backed entities.
- SchemaId and NoteId are intended to become unnecessary for file identity.
- Parsing remains context-specific and is not owned by base discovery.
- File-level content hashing is context-owned for freshness checks; discovery should not force file content hashing in FileView.
- Structured contexts (Schema/Config) require both content hash and entry/property hash indexing in their own view models.
- Hash capability contracts are crate-private and based on support hash primitives, using traits:
  - HasContentHash
  - HasContentHashMut
  - HasEntryHashes
  - HasEntryHashesMut
- Discovery result should preserve enough path information for safe file reads and must not drop useful scanner information.
- Discovery result representation should begin with a flat classification model and remain extensible.
- Basename index can be removed from the general discovery concern; retained indexes are path, parent, format, and primary views.

## Testing Decisions

- Good tests validate external behavior at module seams (scan/classify/persist/result), not internal implementation detail.
- Discovery tests should cover:
  - Correct classification (new/stale/unchanged/deleted).
  - Delta persistence (batch save/delete only where needed).
  - Path safety and root scoping guarantees.
  - Deterministic ordering and stable result output where expected.
- Context processor tests should cover behavior when fed discovery results (not direct scanner mocks unless needed).
- Hash trait tests should validate:
  - Content-only hash record behavior.
  - Entry/property hash diff behavior.
  - Mutating trait behavior consistency.
- Prior art in codebase includes typestate processor tests, repository seam tests, and scanner/path validation tests; new tests should follow those conventions.

## Out of Scope

- Final renaming or relocation from Vault module to a fully separate discovery module namespace.
- Full redesign of all repository/table architecture across the entire codebase.
- Public API exposure of hash traits beyond crate-private boundaries.
- Immediate removal of all old context discovery code in a single change.
- Any UI/CLI redesign unrelated to orchestration ordering.

## Further Notes

- Config-first orchestration is required in composed processing runs.
- Standalone context processors are preferred to keep downstream stages independent and enable selective parallel execution.
- A follow-up design session is expected to finalize long-term architecture for table ownership and repository seams after this initial refactor draft.
