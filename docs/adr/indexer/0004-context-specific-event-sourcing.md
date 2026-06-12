---
name: context-specific-event-sourcing
status: accepted
date_proposed: 2026-05-28
date_decided: 2026-05-28
stakeholders: [Core Team]
---

# ADR 0004: Context-Specific Event Sourcing for Pipeline Restartability

## Context

Lithos processors (Discovery, Schema, Note, Template, Config) are long-running pipelines that process hundreds or thousands of files. If the process crashes mid-pipeline (e.g., OOM, SIGKILL, power failure), previously completed work is lost and must be redone on restart.

For example: Discovery scans 1000 files → Schema processes 300 files → **CRASH** → On restart, Schema must re-process all 300 completed files because there's no record of what was already done.

The technical forces at play:
- **Crash recovery**: How to resume processing after unexpected termination?
- **Bounded context isolation**: Schema crash should not corrupt Note event log
- **Event granularity**: Should events track Start/Complete only, or intermediate steps?
- **Performance**: Event logging must not become a write bottleneck
- **Storage overhead**: Event logs must be compacted to avoid unbounded growth

## Decision

**We will implement context-specific event sourcing with shared `EventStore<E>` infrastructure.**

### Architecture Components

1. **Context-Specific Event Tables** (maintains bounded contexts):
   ```rust
   pub const DISCOVERY_EVENTS: Table<u64, &[u8]> = Table::new("discovery_events");
   pub const SCHEMA_EVENTS: Table<u64, &[u8]> = Table::new("schema_events");
   pub const NOTE_EVENTS: Table<u64, &[u8]> = Table::new("note_events");
   pub const TEMPLATE_EVENTS: Table<u64, &[u8]> = Table::new("template_events");
   pub const CONFIG_EVENTS: Table<u64, &[u8]> = Table::new("config_events");
   ```

2. **Generic EventStore** (shared infrastructure in `db` module):
   ```rust
   pub struct EventStore<E> {
       db: Database,
       table_def: TableDefinition<u64, &'static [u8]>,
       next_seq: AtomicU64,  // Monotonic sequence generator
       _event: PhantomData<E>,
   }

   impl<E: Serialize + DeserializeOwned> EventStore<E> {
       pub fn append(&self, event: &E) -> Result<u64, DbError>;
       pub fn append_batch(&self, events: &[E]) -> Result<Vec<u64>, DbError>;
       pub fn load_all(&self) -> Result<Vec<E>, DbError>;
       pub fn compact(&self, completed_file_ids: &[FileId]) -> Result<(), DbError>;
   }
   ```

3. **Intermediate Event Tracking** (full lifecycle, not just Start/Complete):
   ```rust
   pub enum SchemaEvent {
       Discovered { file_id: FileId, path: PathKey, discovered_at: SystemTime },
       ParseStarted { file_id: FileId, started_at: SystemTime },
       Parsed { file_id: FileId, parsed_at: SystemTime },
       PropertyBankReferenceExpanded { file_id: FileId, expanded_at: SystemTime },
       InheritanceResolved { file_id: FileId, parent_count: usize, resolved_at: SystemTime },
       SchemaPersisted { file_id: FileId, persisted_at: SystemTime },
       Completed { file_id: FileId, completed_at: SystemTime },
       Failed { file_id: FileId, error: String, failed_at: SystemTime },
   }
   ```

4. **Projector Pattern** (rehydrates state from events):
   ```rust
   pub struct PendingSchemaState {
       pub pending: HashMap<FileId, PathKey>,
       pub completed: HashSet<FileId>,
       pub failed: HashSet<FileId>,
   }

   impl PendingSchemaState {
       pub fn from_events(events: &[SchemaEvent]) -> Self {
           // Fold events into current state
       }

       pub fn pending_files(&self) -> Vec<FileId> {
           self.pending.keys().copied().collect()
       }
   }
   ```

### Batch Performance

**Discovery**: Batch commits every N=100 files
```rust
const BATCH_SIZE: usize = 100;
for batch in files.chunks(BATCH_SIZE) {
    repository.persist_discovery_batch(batch)?;
    event_store.append_batch(batch_events)?;  // Atomic with view persistence
}
```

**Context Processing**: Per-file event logging
```rust
for file in pending_files {
    match process_schema_file(file) {
        Ok(_) => event_store.append(&SchemaEvent::Completed { file_id })?,
        Err(e) => event_store.append(&SchemaEvent::Failed { file_id, error })?,
    }
}
```

### Dependency-Aware Cleanup

Cleanup timing respects context dependencies (`Discovery → Config → {Schema, Note, Template}`):

```rust
// Immediate cleanup (independent contexts)
schema_event_store.compact(&completed_file_ids)?;
note_event_store.compact(&completed_file_ids)?;
template_event_store.compact(&completed_file_ids)?;

// Deferred cleanup (after dependents complete)
config_event_store.compact(&completed_file_ids)?;    // After Schema/Note/Template
discovery_event_store.compact(&completed_file_ids)?; // After ALL contexts
```

### Event Table Primary Key

**Key Type**: `u64` (monotonically increasing sequence number)
- Ensures strict chronological ordering
- Enables deterministic replay during state rehydration
- No race conditions (atomic increment via `AtomicU64`)

## Alternatives Considered

### Alternative 1: Checkpoint-Only (Start/Complete Events)

**Pros**:
- Minimal storage overhead (2 events per file: Start + Complete/Failed)
- Simple state rehydration (only need to track completed files)

**Cons**:
- No debugging granularity (cannot see where pipeline hung)
- Cannot detect partial progress within a file
- Loses audit trail of intermediate steps

**Why rejected**: Intermediate events provide valuable debugging information. When a pipeline hangs, we can see exactly which typestate transition it stalled on (e.g., "stuck in PropertyBankReferenceExpanded"). The storage overhead is acceptable (~100 bytes per event × 5-7 events per file = <1KB per file).

### Alternative 2: Shared Event Table with Context Discriminator

**Pros**:
- Single event log (simpler storage layout)
- Easy to query cross-context event timeline

**Cons**:
- Violates bounded context boundaries
- Schema crash could corrupt Note events (shared table)
- Harder to compact (must filter by context discriminator)
- Coupling across contexts (all share same event schema)

**Why rejected**: Bounded context isolation is a core architectural principle. Each context owns its event log, ensuring crashes/corruption are isolated. Shared tables create coupling that makes contexts harder to evolve independently.

### Alternative 3: Lightweight Journaling (Redo Log Pattern)

**Pros**:
- Minimal write overhead (append-only redo log)
- Database-native durability (redb's transaction log)

**Cons**:
- No intermediate event tracking (only final mutations logged)
- Loses typestate transition audit trail
- Harder to inspect state (must replay mutations, not read events)
- Not idempotent (reapplying mutations can corrupt state)

**Why rejected**: Event sourcing provides better auditability and debuggability. Events are idempotent (replaying `Completed { file_id }` multiple times is safe), whereas redo log mutations are not. The performance overhead is acceptable given redb's append-only write performance.

## Technical Validation

### Research Findings

- **redb append-only benchmarks** (`.scratch/pipeline-restartability-research.md`): Append-only writes are 1-2ms per 100-file batch, confirming that event logging is not a bottleneck.
- **Existing processor patterns** (via GitNexus): Property bank processor already uses checkpoint pattern (`PropertyBankDelta`), confirming that partial success tracking is a real need.

### Recovery Scenario (Example)

**Scenario**: Discovery scans 1000 files → Schema processes 300 files → **CRASH** → Restart

**Recovery Process**:
1. **Rehydrate State**: Load all `SchemaEvent` from `SCHEMA_EVENTS` table
2. **Project State**: `PendingSchemaState::from_events()` identifies:
   - 300 completed files (via `Completed` events)
   - 700 pending files (via `Discovered` but no `Completed/Failed`)
3. **Resume Processing**: Process only the 700 pending files
4. **Emit Events**: Append `Completed/Failed` events as files finish
5. **Compact Log**: After all files complete, delete events for completed/failed files

**Result**: Zero work lost. Schema processor resumes from exact point of failure.

## Consequences

- **Positive**:
  - Zero work lost on crash (all completed work preserved via events)
  - Full audit trail (can inspect pipeline history for debugging)
  - Bounded context isolation (context crashes don't corrupt other event logs)
  - Natural typestate fit (events emitted at typestate transitions)
  - Deterministic replay (monotonic sequence keys ensure chronological ordering)

- **Negative**:
  - Storage overhead (~1KB per file for 5-7 events)
  - Write amplification (per-file event logging in contexts)
  - Compaction required (event logs grow unbounded without cleanup)
  - Migration complexity (existing processors must be retrofitted)

- **Risks**:
  - Event log compaction failure → unbounded growth. Mitigated by dependency-aware cleanup (always compact after context completion).
  - Event schema evolution (adding/removing event types). Mitigated by versioned deserialization (bincode supports schema evolution).

## References

- PRD: `.scratch/centralized-discovery-processor/PRD.md` (Section 6: Pipeline Resilience & Restartability)
- Research: `.scratch/pipeline-restartability-research.md`
- Handoff: `/var/folders/9w/3qn47_qj3m9b27gkxwr5_k9m0000gn/T/opencode/handoff-centralized-discovery-continued.md` (Question 4)
- Property Bank Processor: `lithos-core/src/schema/property_bank_processor.rs` (checkpoint pattern reference)
