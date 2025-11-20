# Hybrid BoltDB + SQLite Architecture Guide

This guide accompanies Story 3.31 and explains how BoltDB (hot cache) and SQLite (deep storage) work together.

## Storage Responsibilities
- **BoltDB:** Sub-millisecond lookups for path/basename/alias queries (Stories 3.20, 3.23).
- **SQLite:** Schema-driven views for frontmatter queries (Story 3.21).
- **CacheUnitOfWork:** Ensures both stores remain consistent (Story 3.22).

## Query Routing
- Hot path: ByPath / ByBasename / ByAlias (BoltDB via CacheReaderPort).
- Deep path: FileClassQuery / FrontmatterQuery (SQLite MetadataQueryPort).
- Staleness detection occurs before returning results (Story 3.23).

## Event/CQRS Integration
- Event-driven architecture (Story 3.30) ensures QueryService stays read-only and receives rebuild notifications via events.

## References
- docs/architecture/components.md (VaultIndexer, QueryService)
- docs/architecture/performance-guide.md
