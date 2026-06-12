---
name: fileid-as-universal-identity
status: accepted
date_proposed: 2026-05-28
date_decided: 2026-05-28
stakeholders: [Core Team]
---

# ADR 0001: FileId as Universal Identity for File-Backed Entities

## Context

Lithos previously maintained separate identity types for each context: `SchemaId` for schemas, `NoteId` for notes, and `TemplateId` for templates. Each context maintained its own path-to-ID index (`SCHEMA_ID_BY_PATH`, `NOTE_ID_BY_PATH`) in parallel with the vault's `FILE_ID_BY_PATH` table. This duplication created maintenance overhead and coupling—when a file moved, multiple index tables required coordinated updates.

Schema inheritance was modeled as `SchemaId → SchemaId` edges, despite the fact that schemas are just markdown files with specific structure. The conceptual mismatch (treating file-backed entities as if they had independent identity) complicated the inheritance graph and made it unclear whether identity survived file moves.

The technical forces at play:
- **Source of truth**: Files are the authoritative source for schemas/notes/templates
- **Index duplication**: Three contexts maintaining parallel path→ID mappings
- **Identity stability**: Unclear whether `SchemaId` survived file renames/moves
- **Coupling**: Schema inheritance depends on file relationships, not abstract IDs

## Decision

**We will remove context-specific identity types (`SchemaId`, `NoteId`, `TemplateId`) and use `FileId` as the universal identity for all file-backed entities.**

Schema inheritance graphs will use `FileId` edges (file-to-file relationships). Context-specific path indexes (`SCHEMA_ID_BY_PATH`, `NOTE_ID_BY_PATH`) will be removed—contexts will query discovery's central `FILE_ID_BY_PATH` table instead.

Context aggregates will embed `file_id: FileId` as their primary key:
```rust
// Before
pub struct Schema {
    id: SchemaId,
    name: String,
    // ...
}
pub const SCHEMAS: UuidTable<SchemaId, &[u8]> = UuidTable::new("schemas");

// After
pub struct Schema {
    file_id: FileId,  // Source file identity
    name: String,     // Still unique within schemas
    // ...
}
pub const SCHEMAS: UuidTable<FileId, &[u8]> = UuidTable::new("schemas");
```

Path resolution becomes two-step:
1. Query discovery's `FILE_ID_BY_PATH` → get `FileId`
2. Query context aggregate table using `FileId`

## Alternatives Considered

### Alternative 1: Keep Separate IDs, Maintain Parallel Indexes

**Pros**:
- No migration required
- Context independence (each owns its path index)

**Cons**:
- Index duplication (3+ tables for same data)
- Coordinated updates required (file move = update multiple indexes)
- Identity semantics unclear (does `SchemaId` survive file move?)
- Conceptual mismatch (treating files as if they have abstract identity)

**Why rejected**: The cons outweigh the pros. Maintaining parallel indexes is error-prone and violates DRY. The conceptual mismatch (files vs abstract IDs) creates confusion about identity stability.

### Alternative 2: Composite Identity (FileId + ContextId)

**Pros**:
- Explicit context scoping
- Could support multiple contexts per file (future extensibility)

**Cons**:
- Unnecessary complexity (current design: one context per file)
- Complicates foreign key references
- Still requires path resolution via discovery

**Why rejected**: YAGNI. No current use case for multiple contexts per file. Adding complexity for hypothetical future requirements violates incremental design principles.

## Technical Validation

### Research Findings

- **Inheritance graph analysis** (via GitNexus): Current `InheritanceGraph<SchemaId>` already tracks file paths in node metadata, confirming that identity is file-based despite using `SchemaId` keys.
- **Path index usage** (via GitNexus): Context-specific path indexes are ONLY used for initial lookup—all subsequent operations use the ID. This confirms that path resolution can be externalized to discovery.

### Migration Path

1. Add `file_id: FileId` field to `Schema`/`Note`/`Template` aggregates
2. Dual-write period: write both old ID and new `FileId`
3. Migrate `InheritanceGraph` edges: `(SchemaId, SchemaId)` → `(FileId, FileId)`
4. Remove `SCHEMA_ID_BY_PATH`, `NOTE_ID_BY_PATH` tables
5. Remove old ID fields

## Consequences

- **Positive**:
  - Simpler identity model (one ID type instead of three)
  - Fewer index tables (remove 2-3 context-specific path indexes)
  - Clearer semantics: identity = file identity (survives content changes, not path moves)
  - Single source of truth for path→ID mapping (discovery's `FILE_ID_BY_PATH`)
  - Inheritance graph accurately models reality (file-to-file relationships)

- **Negative**:
  - Migration complexity: existing references must be updated
  - Read-only coupling: contexts depend on discovery for path resolution (acceptable because discovery is a prerequisite)
  - Two-step path resolution (query discovery → query context) vs one-step (direct context path index)

- **Risks**:
  - Migration requires careful coordination (dual-write period to avoid data loss)
  - If discovery's `FILE_ID_BY_PATH` becomes corrupted, all contexts lose path resolution (mitigated by event sourcing: can rebuild from file views)

## References

- PRD: `.scratch/centralized-discovery-processor/PRD.md` (Section 3: Identity & Paths)
- Handoff: `/var/folders/9w/3qn47_qj3m9b27gkxwr5_k9m0000gn/T/opencode/handoff-centralized-discovery-continued.md`
- GitNexus analysis: Current schema builder path resolution patterns
