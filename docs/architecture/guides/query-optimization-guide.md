# Query Optimization Guide (Story 3.31)

## Hot vs Deep Path Decisions
- Prefer BoltDB (hot path) for any exact path, basename, or alias query.
- Use SQLite frontmatter views when filtering on schema fields or performing complex aggregations.

## Staleness & Refresh
- Monitor staleness warnings (Story 3.23) to schedule incremental refreshes via VaultIndexer.Refresh(since).

## Indexing Tips
- Keep schemas lean: define only the fields needed for querying to minimize SQLite view overhead.
- Batch writes through CacheUnitOfWork (Story 3.22) to reduce transaction overhead and lock contention.

## Configuration knobs
- `FileClassKey` customization (Story 3.29) allows matching non-standard vault metadata; ensure E2E tests cover each key variant.

## Performance Monitoring
- Capture benchmark data from `docs/architecture/performance-guide.md` and update as new results are collected.
