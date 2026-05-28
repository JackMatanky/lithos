---
name: discovery-table-ownership
status: accepted
date_proposed: 2026-05-28
date_decided: 2026-05-28
stakeholders: [Core Team]
---

# ADR 0003: Discovery Owns All Vault Tables

## Context

After centralizing discovery, we must decide which database tables are owned (written) by discovery vs context processors. The vault module currently maintains nine tables:

**Primary Views**:
- `FILE_VIEWS` - File metadata
- `DIR_VIEWS` - Directory metadata

**Path Indexes**:
- `FILE_ID_BY_PATH` - Path → FileId lookup
- `DIR_ID_BY_PATH` - Path → DirId lookup
- ~~`PATH_BY_FILE_ID`~~ - Removed (path now in FileView)
- ~~`PATH_BY_DIR_ID`~~ - Removed (path now in DirView)

**Query Optimization Indexes**:
- `FILE_IDS_BY_BASENAME` - Wikilink resolution (e.g., `[[note-name]]`)
- `FILE_IDS_BY_PARENT` - Tree traversal queries
- `FILE_IDS_BY_FORMAT` - Format filtering (e.g., all markdown files)

The technical forces at play:
- **Single-writer consistency**: redb supports MVCC reads but benefits from minimizing write contention
- **Index coherence**: Secondary indexes must stay in sync with primary views
- **Context coupling**: If contexts write to vault tables, they become coupled to vault's schema
- **Query performance**: Some indexes benefit multiple contexts (e.g., basename lookup for notes AND schemas)

## Decision

**We will enforce that discovery is the ONLY writer to all vault tables. Contexts are read-only consumers.**

This is enforced at compile-time via segregated repository traits:

```rust
// Read operations (contexts query this)
pub trait DiscoveryReadRepository {
    fn find_file_by_path(&self, path: &PathKey) -> Result<Option<FileView>>;
    fn find_file_by_id(&self, id: FileId) -> Result<Option<FileView>>;
    fn find_files_by_basename(&self, name: &str) -> Result<Vec<FileView>>;
    fn find_files_by_parent(&self, parent: DirId) -> Result<Vec<FileView>>;
    fn find_files_by_format(&self, format: FileFormat) -> Result<Vec<FileView>>;
}

// Write operations (ONLY discovery processor)
pub trait DiscoveryWriteRepository {
    fn persist_file_views(&self, files: &[FileView]) -> Result<()>;
    fn persist_dir_views(&self, dirs: &[DirView]) -> Result<()>;
    fn delete_files(&self, ids: &[FileId]) -> Result<()>;
    fn delete_dirs(&self, ids: &[DirId]) -> Result<()>;
}

// Unified (convenience trait)
pub trait DiscoveryRepository: DiscoveryReadRepository + DiscoveryWriteRepository {}
```

Context repositories **extend** `DiscoveryReadRepository` for read-only vault access:

```rust
pub trait SchemaReadRepository: DiscoveryReadRepository {
    fn find_by_file_id(&self, file_id: FileId) -> Result<Option<Schema>>;
    fn find_by_name(&self, name: &str) -> Result<Option<Schema>>;
}

pub trait SchemaWriteRepository {
    fn save(&self, schema: &Schema, file_id: FileId) -> Result<()>;
    fn delete_by_file_id(&self, file_id: FileId) -> Result<()>;
}
```

All vault tables (including basename/parent/format indexes) are discovery-owned. These indexes are general-purpose—multiple contexts benefit from them.

## Alternatives Considered

### Alternative 1: Context-Specific Indexes

**Pros**:
- Contexts own all their data (full independence)
- No shared tables (simpler reasoning about ownership)

**Cons**:
- Index duplication (e.g., Schema and Note both need basename lookup for wikilinks)
- Coordinated updates (file move = update discovery + schema + note indexes)
- Violates DRY principle

**Why rejected**: Basename/parent/format indexes are general-purpose queries that benefit multiple contexts. Duplicating these indexes wastes storage and creates maintenance overhead.

### Alternative 2: Shared Write Access

**Pros**:
- Contexts can optimize their own index updates
- No dependency on discovery for index maintenance

**Cons**:
- Write contention (multiple contexts updating same tables)
- Index coherence risk (contexts could corrupt vault state)
- Violates single-writer principle
- Hard to reason about invariants (who guarantees index consistency?)

**Why rejected**: Shared writes create coordination overhead and corruption risk. redb's MVCC is optimized for single-writer, many-reader workloads. Multiple writers would block each other, negating parallelization benefits.

### Alternative 3: Context-Maintained, Discovery-Owned

**Pros**:
- Contexts "know best" which indexes they need
- Discovery acts as coordinator, not implementer

**Cons**:
- Complex coordination protocol (contexts request index updates via discovery)
- Async update risk (index lags behind context state)
- Violates cohesion principle (discovery logic scattered across contexts)

**Why rejected**: Coordination overhead outweighs benefits. Discovery already scans the filesystem—it naturally produces the data needed for all indexes. Letting contexts trigger index updates adds complexity without clear benefit.

## Technical Validation

### Research Findings

- **GitNexus analysis**: Current vault repository already maintains all indexes atomically within the same write transaction, confirming that single-writer updates are the existing pattern.
- **redb MVCC benchmarks**: Single-writer with parallel readers is the optimal access pattern for redb (confirmed in `.scratch/pipeline-restartability-research.md`).

### Index Usage Patterns (via GitNexus)

```
FILE_IDS_BY_BASENAME:
  - Note context: Wikilink resolution ([[note-name]])
  - Schema context: Schema reference lookup (extends: base)
  - Template context: Template inclusion ({% include template-name %})

FILE_IDS_BY_PARENT:
  - Note context: Child note queries (all notes in daily/)
  - Template context: Template directory traversal
  - Query layer: Tree navigation

FILE_IDS_BY_FORMAT:
  - Schema context: Find all schema files (*.toml, *.json)
  - Note context: Find all note files (*.md)
  - Query layer: Format filtering
```

All three indexes are used by multiple contexts, confirming they are general-purpose infrastructure.

## Consequences

- **Positive**:
  - Single-writer consistency (no coordination overhead, no corruption risk)
  - Clear ownership boundary (discovery = filesystem state, contexts = domain state)
  - No index duplication (basename/parent/format indexes shared by all contexts)
  - Compile-time enforcement (read-only access via trait bounds)
  - Simpler reasoning (vault tables updated exactly once per discovery run)

- **Negative**:
  - Read-only coupling: Contexts depend on discovery for filesystem queries. This is acceptable because discovery is a prerequisite phase.
  - Context processors cannot optimize vault indexes for their specific use cases. This is acceptable because indexes are general-purpose.

- **Risks**:
  - If discovery's indexes become corrupted, all contexts lose query capability. Mitigated by event sourcing: can rebuild vault tables from `FILE_VIEWS` + `DIR_VIEWS`.

## References

- PRD: `.scratch/centralized-discovery-processor/PRD.md` (Section 9: Table Ownership & Repository Pattern)
- Handoff: `/var/folders/9w/3qn47_qj3m9b27gkxwr5_k9m0000gn/T/opencode/handoff-centralized-discovery-continued.md` (Question 7)
- Current Vault Tables: `lithos-core/src/vault/storage/tables.rs`
- GitNexus Analysis: Index usage patterns across contexts
