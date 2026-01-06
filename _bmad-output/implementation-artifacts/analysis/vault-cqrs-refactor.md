# Vault Package CQRS Refactor: Eliminating God Objects

## Current Problem: VaultIndexer God Object

**Current VaultIndexer (650 lines)** mixes **Commands + Queries + Events**:

```go
type VaultIndexer struct {
    // Mixed responsibilities
    scanner       spi.VaultScannerPort     // Read
    processor     *VaultProcessor          // Write
    cacheWriter   *persistence.CacheWriter // Write
    // ... more
}

// Commands (Write Operations)
func (v *VaultIndexer) Build(ctx) error                    // ✅ Command
func (v *VaultIndexer) Refresh(ctx, since) error          // ✅ Command
func (v *VaultIndexer) reconcileDeletions(ctx) []domain.Note // ✅ Command

// Queries (Read Operations)
func (v *VaultIndexer) validateCacheState(ctx, ...)       // ❌ Query in Command object
func (v *VaultIndexer) collectVaultState(ctx, ...)        // ❌ Query in Command object
func (v *VaultIndexer) collectCacheState(ctx, ...)        // ❌ Query in Command object
func (v *VaultIndexer) findInconsistencies(...)           // ❌ Query in Command object

// Events (Communication)
func (v *VaultIndexer) handleCommandIssuedEvent(...)      // ✅ Event
func (v *VaultIndexer) publishIndexingCompleteEvent(...)  // ✅ Event
```

**Issues:**
- ❌ **CQRS Violation**: Commands and Queries mixed in same object
- ❌ **God Object**: 650 lines handling disparate concerns
- ❌ **Tight Coupling**: Read logic tied to write logic
- ❌ **Test Complexity**: Hard to test queries independently of commands

## Proposed CQRS Architecture

**Separate Commands, Queries, and Events:**

```
internal/app/vault/
├── commands/           # Write operations
│   ├── index_command.go       (~150 lines)
│   ├── refresh_command.go     (~120 lines)
│   └── reconcile_command.go   (~80 lines)
├── queries/            # Read operations
│   ├── cache_state_query.go   (~120 lines)
│   ├── vault_state_query.go   (~100 lines)
│   └── index_stats_query.go   (~80 lines)
├── events/             # Communication
│   ├── handlers.go            (~100 lines)
│   └── publishers.go          (~80 lines)
└── orchestrator.go     # Thin orchestrator (~100 lines)
```

### Commands (Write Operations)

**IndexCommand** - Full vault indexing
```go
type IndexCommand struct {
    scanner     spi.VaultScannerPort
    processor   *VaultProcessor
    cacheWriter *persistence.CacheWriter
    validator   *CacheValidator
}

func (c *IndexCommand) Execute(ctx) (metrics.IndexStats, error) {
    // Pure write logic: scan → process → cache → validate
    files := c.scanner.ScanAll(ctx)
    notes, metadata := c.processor.ProcessBatch(ctx, files)
    c.cacheWriter.WriteBatch(ctx, notes, metadata)
    return c.validator.ValidateState(ctx, files, notes)
}
```

**RefreshCommand** - Incremental updates
```go
type RefreshCommand struct {
    scanner     spi.VaultScannerPort
    processor   *VaultProcessor
    cacheWriter *persistence.CacheWriter
    validator   *CacheValidator
}

func (c *RefreshCommand) Execute(ctx, since) error {
    // Pure write logic: find changes → process → cache
    modified := c.scanner.ScanModified(ctx, since)
    notes, metadata := c.processor.ProcessBatch(ctx, modified)
    return c.cacheWriter.WriteBatch(ctx, notes, metadata)
}
```

**ReconcileCommand** - Cache cleanup
```go
type ReconcileCommand struct {
    cacheStateQuery *CacheStateQuery
    cacheWriter     *persistence.CacheWriter
}

func (c *ReconcileCommand) Execute(ctx) error {
    // Pure write logic: find orphans → delete
    orphans := c.cacheStateQuery.FindOrphanedEntries(ctx)
    return c.cacheWriter.DeleteBatch(ctx, orphans)
}
```

### Queries (Read Operations)

**CacheStateQuery** - Cache inspection
```go
type CacheStateQuery struct {
    cacheReader spi.CacheReaderPort
}

func (q *CacheStateQuery) GetAllEntries(ctx) ([]domain.Note, error) {
    // Pure read: return cache contents
}

func (q *CacheStateQuery) FindOrphanedEntries(ctx) ([]string, error) {
    // Pure read: compare vault vs cache
}
```

**VaultStateQuery** - Vault inspection
```go
type VaultStateQuery struct {
    scanner spi.VaultScannerPort
}

func (q *VaultStateQuery) GetAllFiles(ctx) ([]dto.VaultFile, error) {
    // Pure read: scan vault
}

func (q *VaultStateQuery) GetModifiedSince(ctx, since) ([]dto.VaultFile, error) {
    // Pure read: find changes
}
```

**IndexStatsQuery** - Statistics
```go
type IndexStatsQuery struct {
    cacheStateQuery *CacheStateQuery
    vaultStateQuery *VaultStateQuery
}

func (q *IndexStatsQuery) GetStats(ctx) (metrics.IndexStats, error) {
    // Pure read: aggregate statistics
}
```

### Events (Communication)

**Event Handlers** - React to events
```go
type IndexingEventHandler struct {
    indexCommand *IndexCommand
    refreshCommand *RefreshCommand
}

func (h *IndexingEventHandler) HandleCommandIssued(ctx, event) error {
    switch event.CommandType() {
    case "index":
        return h.indexCommand.Execute(ctx)
    case "refresh":
        return h.refreshCommand.Execute(ctx, event.Since())
    }
}
```

**Event Publishers** - Publish events
```go
type IndexingEventPublisher struct {
    eventBus events.EventBus
}

func (p *IndexingEventPublisher) PublishCompleted(ctx, stats) error {
    event := events.VaultIndexingCompletedEvent{Stats: stats}
    return p.eventBus.Publish(ctx, event)
}
```

### Thin Orchestrator

**VaultOrchestrator** - Coordinates commands/queries/events
```go
type VaultOrchestrator struct {
    indexCommand   *IndexCommand
    refreshCommand *RefreshCommand
    reconcileCommand *ReconcileCommand
    indexStatsQuery *IndexStatsQuery
    eventPublisher *IndexingEventPublisher
}

func (o *VaultOrchestrator) Index(ctx) (metrics.IndexStats, error) {
    stats, err := o.indexCommand.Execute(ctx)
    if err != nil {
        return metrics.IndexStats{}, err
    }
    o.eventPublisher.PublishCompleted(ctx, stats)
    return stats, nil
}

func (o *VaultOrchestrator) GetStats(ctx) (metrics.IndexStats, error) {
    return o.indexStatsQuery.GetStats(ctx)
}
```

## Benefits of CQRS Refactor

### Maintainability
- ✅ **Single Responsibility**: Commands only write, Queries only read
- ✅ **Focused Components**: Each ~80-150 lines with clear purpose
- ✅ **Easy Testing**: Commands and Queries tested independently
- ✅ **Clear Dependencies**: Commands depend on Queries, not vice versa

### Reusability
- ✅ **Query Reuse**: CacheStateQuery usable by monitoring tools
- ✅ **Command Composition**: Commands can be combined for complex workflows
- ✅ **Event Flexibility**: Handlers can be swapped for different behaviors

### Testability
- ✅ **Unit Testing**: Each Command/Query tested in isolation
- ✅ **Mock Boundaries**: Clear interfaces between C/Q/E
- ✅ **Integration Testing**: Orchestrator tests command composition

### Performance
- ✅ **Concurrent C/Q**: Commands and Queries can run concurrently
- ✅ **Optimized Reads**: Query optimization independent of commands
- ✅ **Event-Driven**: Async processing via events

### Scalability
- ✅ **Horizontal Scaling**: Commands and Queries scale independently
- ✅ **CQRS Databases**: Different storage for Commands vs Queries
- ✅ **Event Sourcing**: Commands generate events for Query updates

## Implementation Strategy

### Phase 1: Extract Queries (Low Risk)
1. Create `vault/queries/cache_state_query.go`
2. Move `validateCacheState()`, `collectCacheState()` logic
3. Update VaultIndexer to use CacheStateQuery
4. Test: All existing tests pass

### Phase 2: Extract Commands (Medium Risk)
1. Create `vault/commands/index_command.go`
2. Move Build() logic to IndexCommand.Execute()
3. Update orchestrator to use commands
4. Test: Full integration test suite

### Phase 3: Extract Events (Low Risk)
1. Create `vault/events/handlers.go`
2. Move event handling logic
3. Update orchestrator
4. Test: Event-driven scenarios

### Phase 4: Thin Orchestrator (Final)
1. Replace VaultIndexer with VaultOrchestrator
2. Update command layer to use new interface
3. Remove old VaultIndexer
4. Test: Full system integration

## Risk Mitigation

### Testing Strategy
- **Unit Tests**: Each Command/Query tested independently
- **Integration Tests**: Orchestrator command composition
- **End-to-End**: Full vault indexing workflows
- **Regression Tests**: Existing CLI commands unchanged

### Rollback Plan
- Keep old VaultIndexer interface during transition
- Gradual migration: one method at a time
- Feature flags if needed
- Database backups before changes

### Performance Monitoring
- Benchmark before/after each phase
- Monitor memory usage (Queries might cache results)
- Alert on performance regressions >5%

## Success Criteria

- ✅ **VaultIndexer eliminated**: No god object
- ✅ **CQRS Boundaries**: Clear C/Q/E separation
- ✅ **Component Size**: All components <200 lines
- ✅ **Test Coverage**: >90% for all new components
- ✅ **Performance**: No regression in indexing speed
- ✅ **API Compatibility**: Existing CLI unchanged

## Questions for Discussion

1. **Query Result Caching**: Should Queries cache results for performance?
2. **Event Sourcing**: Should Commands generate events for Query updates?
3. **Database Segregation**: Separate databases for Commands vs Queries?
4. **Migration Strategy**: Big bang vs incremental migration?
5. **Interface Design**: How to maintain backward compatibility?
