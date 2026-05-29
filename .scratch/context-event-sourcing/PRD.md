# PRD: Context-Specific Event Sourcing & Pipeline Restartability

**Status**: draft
**Created**: 2026-05-30
**Depends On**: `.scratch/centralized-discovery-processor/PRD.md` (generic event infrastructure)

---

## Problem Statement

Context processors (Schema, Note, Template, Config) are long-running pipelines that process hundreds or thousands of files. When a process crashes mid-pipeline (e.g., OOM, SIGKILL, power failure), all previously completed work is lost and must be redone on restart.

**Current Behavior**: Discovery scans 1000 files → Schema processes 300 files → **CRASH** → On restart, Schema must re-process all 1000 files (700 new + 300 already completed) because there's no record of what was already done.

**Impact**:
- **Development frustration**: Developers lose 5-10 minutes of work on every crash during iteration
- **Production risk**: Long-running indexing jobs (e.g., initial vault scan of 10,000 files) must complete in one uninterrupted run
- **Debugging difficulty**: No audit trail of which files succeeded, failed, or where the pipeline stalled

The generic event sourcing infrastructure (`EventId`, `EventTable`, `EventStore` trait) was established in the centralized discovery processor PRD. This PRD completes the event sourcing implementation by adding domain-specific event modeling for all context processors.

## Solution

Implement **context-specific event sourcing** for Schema, Note, Template, and Config processors using the shared event infrastructure. Each processor will:

1. Define domain-specific event enums tracking typestate transitions
2. Implement projector patterns to rehydrate pending work from event logs
3. Emit events at each typestate transition (atomic with state writes)
4. Resume processing from rehydrated state after crashes
5. Compact event logs after successful completion

This enables **zero work lost** on crash—all completed files are preserved, and only pending files are processed on restart.

## User Stories

1. As a Schema processor developer, I want schema file processing to be resumable after crashes, so that I don't lose 5-10 minutes of work on every OOM during iteration.
2. As a Note processor developer, I want note ingestion to resume from the last completed note after a crash, so that batch imports of 1000+ notes don't fail catastrophically.
3. As a Template processor developer, I want template compilation to resume after crashes, so that large template libraries don't require uninterrupted processing runs.
4. As a Config processor developer, I want config resolution to be resumable, so that multi-file config hydration survives crashes.
5. As a debugging engineer, I want to inspect event logs to see which files completed, failed, or stalled, so that I can diagnose pipeline hangs.
6. As a performance engineer, I want to see event timestamps for each typestate transition, so that I can identify bottlenecks in the processing pipeline.
7. As a reliability engineer, I want event logs to be compacted after successful runs, so that event tables don't grow unbounded over time.
8. As a context processor maintainer, I want events to be emitted atomically with state writes, so that there's no inconsistency between persisted state and event history.
9. As an architecture reviewer, I want context event logs to be isolated, so that a Schema crash cannot corrupt Note event logs.
10. As a testing engineer, I want to test crash recovery by simulating failures mid-pipeline, so that I can verify resumption logic works correctly.
11. As a Schema processor user, I want to know which schema files failed parsing and why, so that I can fix validation errors without re-running the entire pipeline.
12. As a Note processor user, I want to resume note ingestion from the last batch boundary after a crash, so that I don't re-ingest thousands of already-processed notes.
13. As a Template processor user, I want template compilation to track which templates depend on which schemas, so that template recompilation can be optimized based on schema changes.
14. As a Config processor user, I want to see which config files were successfully loaded vs failed, so that I can diagnose config errors without re-running full discovery.
15. As a maintainer, I want event compaction to respect context dependencies (Config → Discovery → Schema/Note → Template), so that events are only deleted when all downstream consumers have completed.
16. As a developer, I want projector patterns to be consistent across all contexts, so that I can understand resumption logic by reading one example.
17. As a schema maintainer, I want to track property bank reference expansion as a separate event, so that I can debug inheritance resolution failures.
18. As a note maintainer, I want to track frontmatter parsing as a separate event, so that I can diagnose YAML syntax errors.
19. As a template maintainer, I want to track template compilation stages (parsing, validation, linking), so that I can identify which stage failed.
20. As a config maintainer, I want to track global vs local config loading separately, so that I can diagnose which config source failed.

## Implementation Decisions

### 1. Event Type Definitions

Each context defines a domain-specific event enum tracking its typestate pipeline lifecycle.

#### Schema Event Types

```rust
/// Schema processor events track the full processing lifecycle from discovery to persistence.
#[derive(Archive, Deserialize, Serialize)]
#[rkyv(derive(CheckBytes))]
pub enum SchemaEvent {
    /// File discovered (routed from DiscoveryResult)
    Discovered {
        file_id: FileId,
        path: PathKey,
        discovered_at: SystemTime,
    },

    /// Raw file parsing started
    ParseStarted {
        file_id: FileId,
        started_at: SystemTime,
    },

    /// Raw file parsed into RawSchema
    Parsed {
        file_id: FileId,
        parsed_at: SystemTime,
    },

    /// Property bank references expanded
    PropertyBankReferenceExpanded {
        file_id: FileId,
        reference_count: usize,
        expanded_at: SystemTime,
    },

    /// Inheritance resolution started
    InheritanceStarted {
        file_id: FileId,
        parent_count: usize,
        started_at: SystemTime,
    },

    /// Inheritance resolution completed
    InheritanceResolved {
        file_id: FileId,
        resolved_at: SystemTime,
    },

    /// Schema persisted to SCHEMAS table
    SchemaPersisted {
        file_id: FileId,
        persisted_at: SystemTime,
    },

    /// Processing completed successfully
    Completed {
        file_id: FileId,
        completed_at: SystemTime,
    },

    /// Processing failed with error
    Failed {
        file_id: FileId,
        error: String,
        failed_at: SystemTime,
    },
}
```

#### Note Event Types

```rust
/// Note processor events track parsing, frontmatter extraction, and persistence.
#[derive(Archive, Deserialize, Serialize)]
#[rkyv(derive(CheckBytes))]
pub enum NoteEvent {
    /// File discovered (routed from DiscoveryResult)
    Discovered {
        file_id: FileId,
        path: PathKey,
        discovered_at: SystemTime,
    },

    /// Raw markdown parsing started
    ParseStarted {
        file_id: FileId,
        started_at: SystemTime,
    },

    /// Frontmatter extracted
    FrontmatterParsed {
        file_id: FileId,
        has_frontmatter: bool,
        parsed_at: SystemTime,
    },

    /// Schema validation started (if frontmatter present)
    SchemaValidationStarted {
        file_id: FileId,
        schema_name: Option<String>,
        started_at: SystemTime,
    },

    /// Schema validation completed
    SchemaValidated {
        file_id: FileId,
        validated_at: SystemTime,
    },

    /// Note persisted to NOTES table
    NotePersisted {
        file_id: FileId,
        persisted_at: SystemTime,
    },

    /// Processing completed successfully
    Completed {
        file_id: FileId,
        completed_at: SystemTime,
    },

    /// Processing failed with error
    Failed {
        file_id: FileId,
        error: String,
        failed_at: SystemTime,
    },
}
```

#### Template Event Types

```rust
/// Template processor events track compilation stages and schema dependency resolution.
#[derive(Archive, Deserialize, Serialize)]
#[rkyv(derive(CheckBytes))]
pub enum TemplateEvent {
    /// File discovered (routed from DiscoveryResult)
    Discovered {
        file_id: FileId,
        path: PathKey,
        discovered_at: SystemTime,
    },

    /// Template parsing started
    ParseStarted {
        file_id: FileId,
        started_at: SystemTime,
    },

    /// Template parsed into AST
    Parsed {
        file_id: FileId,
        parsed_at: SystemTime,
    },

    /// Schema dependencies resolved
    SchemaDependenciesResolved {
        file_id: FileId,
        schema_count: usize,
        resolved_at: SystemTime,
    },

    /// Note dependencies resolved
    NoteDependenciesResolved {
        file_id: FileId,
        note_count: usize,
        resolved_at: SystemTime,
    },

    /// Template compiled and validated
    Compiled {
        file_id: FileId,
        compiled_at: SystemTime,
    },

    /// Template persisted to TEMPLATES table
    TemplatePersisted {
        file_id: FileId,
        persisted_at: SystemTime,
    },

    /// Processing completed successfully
    Completed {
        file_id: FileId,
        completed_at: SystemTime,
    },

    /// Processing failed with error
    Failed {
        file_id: FileId,
        error: String,
        failed_at: SystemTime,
    },
}
```

#### Config Event Types

```rust
/// Config processor events track multi-file config discovery and hydration.
#[derive(Archive, Deserialize, Serialize)]
#[rkyv(derive(CheckBytes))]
pub enum ConfigEvent {
    /// Global config discovered
    GlobalConfigDiscovered {
        location: GlobalConfigLocation,
        format: StructuredFileFormat,
        discovered_at: SystemTime,
    },

    /// Local config discovered
    LocalConfigDiscovered {
        location: LocalConfigLocation,
        format: StructuredFileFormat,
        discovered_at: SystemTime,
    },

    /// Config file parsing started
    ParseStarted {
        location: ConfigLocation,
        started_at: SystemTime,
    },

    /// Config file parsed
    Parsed {
        location: ConfigLocation,
        parsed_at: SystemTime,
    },

    /// Config validation started
    ValidationStarted {
        location: ConfigLocation,
        started_at: SystemTime,
    },

    /// Config validation completed
    Validated {
        location: ConfigLocation,
        validated_at: SystemTime,
    },

    /// Config merged into final Config
    Merged {
        location: ConfigLocation,
        merged_at: SystemTime,
    },

    /// Config persisted to CONFIG_VIEWS
    Persisted {
        location: ConfigLocation,
        persisted_at: SystemTime,
    },

    /// Config loading completed successfully
    Completed {
        completed_at: SystemTime,
    },

    /// Config loading failed with error
    Failed {
        location: ConfigLocation,
        error: String,
        failed_at: SystemTime,
    },
}
```

### 2. Projector Patterns

Each context implements a projector to rehydrate pending work from event logs.

#### Schema Projector

```rust
/// Rehydrated state from schema events for resumption.
pub struct PendingSchemaState {
    pub pending: HashMap<FileId, PathKey>,
    pub in_progress: HashMap<FileId, SchemaStage>,  // Track which stage file is in
    pub completed: HashSet<FileId>,
    pub failed: HashMap<FileId, String>,  // FileId -> error
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaStage {
    Discovered,
    Parsing,
    Parsed,
    ExpandingPropertyBankReferences,
    ResolvingInheritance,
    Persisting,
}

impl PendingSchemaState {
    pub fn from_events(events: &[SchemaEvent]) -> Self {
        let mut state = Self {
            pending: HashMap::new(),
            in_progress: HashMap::new(),
            completed: HashSet::new(),
            failed: HashMap::new(),
        };

        for event in events {
            match event {
                SchemaEvent::Discovered { file_id, path, .. } => {
                    state.pending.insert(*file_id, path.clone());
                    state.in_progress.insert(*file_id, SchemaStage::Discovered);
                }
                SchemaEvent::ParseStarted { file_id, .. } => {
                    state.in_progress.insert(*file_id, SchemaStage::Parsing);
                }
                SchemaEvent::Parsed { file_id, .. } => {
                    state.in_progress.insert(*file_id, SchemaStage::Parsed);
                }
                SchemaEvent::PropertyBankReferenceExpanded { file_id, .. } => {
                    state.in_progress.insert(*file_id, SchemaStage::ExpandingPropertyBankReferences);
                }
                SchemaEvent::InheritanceStarted { file_id, .. } => {
                    state.in_progress.insert(*file_id, SchemaStage::ResolvingInheritance);
                }
                SchemaEvent::SchemaPersisted { file_id, .. } => {
                    state.in_progress.insert(*file_id, SchemaStage::Persisting);
                }
                SchemaEvent::Completed { file_id, .. } => {
                    state.pending.remove(file_id);
                    state.in_progress.remove(file_id);
                    state.completed.insert(*file_id);
                }
                SchemaEvent::Failed { file_id, error, .. } => {
                    state.pending.remove(file_id);
                    state.in_progress.remove(file_id);
                    state.failed.insert(*file_id, error.clone());
                }
                _ => {}
            }
        }

        state
    }

    pub fn has_pending_work(&self) -> bool {
        !self.pending.is_empty()
    }

    pub fn pending_files(&self) -> Vec<FileId> {
        self.pending.keys().copied().collect()
    }

    /// Returns files that were in-progress at crash (may be partially processed)
    pub fn in_progress_files(&self) -> Vec<(FileId, SchemaStage)> {
        self.in_progress.iter().map(|(id, stage)| (*id, *stage)).collect()
    }
}
```

#### Note Projector

```rust
/// Rehydrated state from note events for resumption.
pub struct PendingNoteState {
    pub pending: HashMap<FileId, PathKey>,
    pub completed: HashSet<FileId>,
    pub failed: HashMap<FileId, String>,
}

impl PendingNoteState {
    pub fn from_events(events: &[NoteEvent]) -> Self {
        let mut state = Self {
            pending: HashMap::new(),
            completed: HashSet::new(),
            failed: HashMap::new(),
        };

        for event in events {
            match event {
                NoteEvent::Discovered { file_id, path, .. } => {
                    state.pending.insert(*file_id, path.clone());
                }
                NoteEvent::Completed { file_id, .. } => {
                    state.pending.remove(file_id);
                    state.completed.insert(*file_id);
                }
                NoteEvent::Failed { file_id, error, .. } => {
                    state.pending.remove(file_id);
                    state.failed.insert(*file_id, error.clone());
                }
                _ => {}
            }
        }

        state
    }

    pub fn has_pending_work(&self) -> bool {
        !self.pending.is_empty()
    }

    pub fn pending_files(&self) -> Vec<FileId> {
        self.pending.keys().copied().collect()
    }
}
```

#### Template Projector

```rust
/// Rehydrated state from template events for resumption.
pub struct PendingTemplateState {
    pub pending: HashMap<FileId, PathKey>,
    pub completed: HashSet<FileId>,
    pub failed: HashMap<FileId, String>,
}

impl PendingTemplateState {
    pub fn from_events(events: &[TemplateEvent]) -> Self {
        let mut state = Self {
            pending: HashMap::new(),
            completed: HashSet::new(),
            failed: HashMap::new(),
        };

        for event in events {
            match event {
                TemplateEvent::Discovered { file_id, path, .. } => {
                    state.pending.insert(*file_id, path.clone());
                }
                TemplateEvent::Completed { file_id, .. } => {
                    state.pending.remove(file_id);
                    state.completed.insert(*file_id);
                }
                TemplateEvent::Failed { file_id, error, .. } => {
                    state.pending.remove(file_id);
                    state.failed.insert(*file_id, error.clone());
                }
                _ => {}
            }
        }

        state
    }

    pub fn has_pending_work(&self) -> bool {
        !self.pending.is_empty()
    }

    pub fn pending_files(&self) -> Vec<FileId> {
        self.pending.keys().copied().collect()
    }
}
```

#### Config Projector

```rust
/// Rehydrated state from config events for resumption.
pub struct PendingConfigState {
    pub pending_global: HashSet<GlobalConfigLocation>,
    pub pending_local: HashSet<LocalConfigLocation>,
    pub completed_global: HashSet<GlobalConfigLocation>,
    pub completed_local: HashSet<LocalConfigLocation>,
    pub failed: HashMap<ConfigLocation, String>,
}

impl PendingConfigState {
    pub fn from_events(events: &[ConfigEvent]) -> Self {
        let mut state = Self {
            pending_global: HashSet::new(),
            pending_local: HashSet::new(),
            completed_global: HashSet::new(),
            completed_local: HashSet::new(),
            failed: HashMap::new(),
        };

        for event in events {
            match event {
                ConfigEvent::GlobalConfigDiscovered { location, .. } => {
                    state.pending_global.insert(location.clone());
                }
                ConfigEvent::LocalConfigDiscovered { location, .. } => {
                    state.pending_local.insert(*location);
                }
                ConfigEvent::Persisted { location, .. } => {
                    match location {
                        ConfigLocation::Global(loc) => {
                            state.pending_global.remove(loc);
                            state.completed_global.insert(loc.clone());
                        }
                        ConfigLocation::Local(loc) => {
                            state.pending_local.remove(loc);
                            state.completed_local.insert(*loc);
                        }
                    }
                }
                ConfigEvent::Failed { location, error, .. } => {
                    state.failed.insert(location.clone(), error.clone());
                }
                _ => {}
            }
        }

        state
    }

    pub fn has_pending_work(&self) -> bool {
        !self.pending_global.is_empty() || !self.pending_local.is_empty()
    }
}
```

### 3. Event Emission Strategy

Events are emitted **atomically** with state writes within the same write transaction. This follows the typestate-driven embedded commits pattern established in the discovery PRD.

**Pattern**:
```rust
impl SchemaProcessor<Parsed, Review> {
    pub fn analyze(self, repo: &impl Repository) -> Result<SchemaProcessor<Analyzed, Review>, Error> {
        // 1. CPU-bound work (no database lock)
        let analysis_result = self.analyze_properties()?;

        // 2. Atomic state + event write (single transaction)
        repo.write(|txn| {
            // Insert analyzed state
            repo.save_schema_state(txn, &analysis_result)?;

            // Append event (same transaction)
            repo.append_event(txn, &SchemaEvent::PropertyBankReferenceExpanded {
                file_id: self.file_id,
                reference_count: analysis_result.reference_count,
                expanded_at: SystemTime::now(),
            })?;

            Ok(())
        })?;

        // 3. Typestate transition
        Ok(SchemaProcessor::new_analyzed(analysis_result))
    }
}
```

**Key Properties**:
- State insert + event append in SAME transaction (atomicity)
- CPU work BEFORE transaction (minimize lock duration)
- Typestate transition AFTER transaction (preserves type safety)

### 4. Resumption Flow

Each processor checks for pending work on startup and resumes from rehydrated state.

**Example: Schema Processor Resumption**

```rust
pub fn run_schema_processor(
    routed_files: Vec<DiscoveredFile>,
    config: &Config,
    repo: &impl Repository,
) -> Result<(), Error> {
    // 1. Rehydrate state from event log
    let events = repo.load_all_events()?;
    let pending_state = PendingSchemaState::from_events(&events);

    // 2. Determine which files need processing
    let files_to_process: Vec<FileId> = if pending_state.has_pending_work() {
        // Resume: process only pending files
        info!("Resuming schema processing: {} pending, {} completed, {} failed",
            pending_state.pending.len(),
            pending_state.completed.len(),
            pending_state.failed.len()
        );
        pending_state.pending_files()
    } else {
        // Fresh run: emit Discovered events for all routed files
        for file in &routed_files {
            repo.write(|txn| {
                repo.append_event(txn, &SchemaEvent::Discovered {
                    file_id: file.id,
                    path: file.view.path().clone(),
                    discovered_at: SystemTime::now(),
                })?;
                Ok(())
            })?;
        }
        routed_files.iter().map(|f| f.id).collect()
    };

    // 3. Process files
    for file_id in files_to_process {
        match process_schema_file(file_id, config, repo) {
            Ok(_) => {
                repo.write(|txn| {
                    repo.append_event(txn, &SchemaEvent::Completed {
                        file_id,
                        completed_at: SystemTime::now(),
                    })?;
                    Ok(())
                })?;
            }
            Err(e) => {
                repo.write(|txn| {
                    repo.append_event(txn, &SchemaEvent::Failed {
                        file_id,
                        error: e.to_string(),
                        failed_at: SystemTime::now(),
                    })?;
                    Ok(())
                })?;
            }
        }
    }

    Ok(())
}
```

### 5. Event Compaction Strategy

Event logs are compacted after successful processor runs to prevent unbounded growth.

**Compaction Timing** (respects dependency graph):

```
Config → Discovery → {Schema, Note} → Template
```

**Compaction Order**:
1. **Schema/Note**: Immediate compaction after processor completes (independent)
2. **Template**: Compaction after Schema/Note complete (depends on both)
3. **Discovery**: Compaction after all contexts complete (all depend on discovery)
4. **Config**: Compaction after discovery completes (discovery depends on config)

**Compaction Implementation**:

```rust
/// Compact event log by deleting events for completed/failed files.
///
/// Called by orchestrator after processor completes successfully.
pub fn compact_schema_events(
    repo: &impl EventStore<Event = SchemaEvent>,
    completed_file_ids: &[FileId],
) -> Result<(), Error> {
    // 1. Load all events
    let all_events = repo.load_all_events()?;

    // 2. Identify event IDs to delete (events for completed files)
    let event_ids_to_delete: Vec<EventId> = all_events
        .iter()
        .enumerate()
        .filter_map(|(idx, event)| {
            let file_id = match event {
                SchemaEvent::Discovered { file_id, .. } => Some(file_id),
                SchemaEvent::Completed { file_id, .. } => Some(file_id),
                SchemaEvent::Failed { file_id, .. } => Some(file_id),
                _ => None,
            };

            file_id.and_then(|fid| {
                completed_file_ids.contains(fid).then(|| EventId(idx as u64))
            })
        })
        .collect();

    // 3. Delete events in a transaction
    repo.write(|txn| {
        repo.compact_events(txn, &event_ids_to_delete)?;
        Ok(())
    })?;

    Ok(())
}
```

**Orchestrator Integration**:

```rust
// After Phase 5 context processing completes
SchemaProcessor::process(routed_files.schemas, &config, &schema_repo)?;
compact_schema_events(&schema_repo, &completed_file_ids)?;  // Immediate

NoteProcessor::process(routed_files.notes, &config, &note_repo)?;
compact_note_events(&note_repo, &completed_file_ids)?;  // Immediate

TemplateProcessor::process(routed_files.templates, &config, &template_repo)?;
compact_template_events(&template_repo, &completed_file_ids)?;  // After Schema/Note

// After all contexts complete
compact_discovery_events(&discovery_repo, &all_file_ids)?;  // After all contexts
compact_config_events(&config_repo, &config_file_ids)?;     // After discovery
```

### 6. Module Structure

**Event Types** (per context):
- `schema/events.rs` - `SchemaEvent` enum
- `note/events.rs` - `NoteEvent` enum
- `template/events.rs` - `TemplateEvent` enum
- `config/events.rs` - `ConfigEvent` enum

**Projectors** (per context):
- `schema/projector.rs` - `PendingSchemaState`
- `note/projector.rs` - `PendingNoteState`
- `template/projector.rs` - `PendingTemplateState`
- `config/projector.rs` - `PendingConfigState`

**Repository Extensions** (per context):
- Existing `Repository` traits extended to implement `EventStore` (already done in discovery PRD)
- No new modules needed (uses existing repository pattern)

**Orchestrator**:
- `orchestrator.rs` - Resumption logic, compaction coordination
- Uses projectors to rehydrate state on startup
- Coordinates compaction after each processor completes

### 7. Serialization Strategy

All events use **rkyv** via the `ArchivedEntity` trait (established in discovery PRD).

**Requirements**:
- Events MUST derive `Archive + Serialize + Deserialize`
- Events MUST derive `#[rkyv(derive(CheckBytes))]` for validation
- Events are serialized via `ArchivedEntity::to_bytes()` (handles alignment)
- Events are deserialized via `ArchivedEntity::from_bytes()` (validates checksums)

**Example**:
```rust
#[derive(Archive, Deserialize, Serialize)]
#[rkyv(derive(CheckBytes))]
pub enum SchemaEvent {
    // ... variants
}

impl ArchivedEntity for SchemaEvent {}  // Blanket impl from db/codec.rs
```

### 8. Error Handling

**Event Append Failures**: If event append fails during transaction, the entire transaction rolls back (state + event both lost). This maintains consistency.

**Event Load Failures**: If event loading fails during rehydration (e.g., corrupt event log), the processor MUST fail fast with a clear error message. Do NOT fall back to full re-processing—require manual intervention.

**Compaction Failures**: If compaction fails, log a warning but DO NOT halt processing. Event logs will grow unbounded until compaction succeeds.

## Testing Decisions

### What Makes a Good Test

- **Test external behavior**: Verify processor resumes from correct state after simulated crash
- **NOT implementation details**: Don't assert on specific event types emitted (test state projector output instead)
- **Test crash scenarios**: Simulate crashes at each typestate transition
- **Test compaction**: Verify event logs are cleaned up after successful runs

### Modules to Test

1. **Projectors** (high priority):
   - `schema/projector.rs` - Test `PendingSchemaState::from_events()` correctly identifies pending/completed/failed files
   - `note/projector.rs` - Test `PendingNoteState::from_events()`
   - `template/projector.rs` - Test `PendingTemplateState::from_events()`
   - `config/projector.rs` - Test `PendingConfigState::from_events()`

2. **Resumption Logic** (high priority):
   - Test processor resumes from pending state (not full re-scan)
   - Test processor emits Discovered events on fresh run
   - Test processor skips completed files on resume

3. **Compaction** (medium priority):
   - Test event logs are compacted after successful run
   - Test compaction respects dependency order
   - Test compaction does NOT delete pending/failed file events

4. **Event Emission** (low priority):
   - Test events are emitted atomically with state writes
   - Test transaction rollback prevents orphaned events

### Prior Art

- **Discovery processor tests** (future): Same patterns apply to context processors
- **Property bank processor tests**: `schema/property_bank_processor.rs` tests typestate transitions
- **Repository tests**: `db/repository_test.rs` tests transaction boundaries

### Test Strategy

**Unit Tests** (per projector):
```rust
#[test]
fn pending_schema_state_from_events_identifies_completed_files() {
    let events = vec![
        SchemaEvent::Discovered { file_id: FileId(1), path: "a.toml".into(), discovered_at: now() },
        SchemaEvent::Completed { file_id: FileId(1), completed_at: now() },
        SchemaEvent::Discovered { file_id: FileId(2), path: "b.toml".into(), discovered_at: now() },
    ];

    let state = PendingSchemaState::from_events(&events);

    assert!(state.completed.contains(&FileId(1)));
    assert!(state.pending.contains_key(&FileId(2)));
    assert_eq!(state.pending_files(), vec![FileId(2)]);
}
```

**Integration Tests** (crash simulation):
```rust
#[test]
fn schema_processor_resumes_after_crash() {
    // 1. Process 3 files, crash after 2
    let files = vec![file1, file2, file3];
    SchemaProcessor::process_with_crash_after(files, 2)?;

    // 2. Restart processor
    let events = schema_repo.load_all_events()?;
    let pending_state = PendingSchemaState::from_events(&events);

    // 3. Verify only 1 file pending (file3)
    assert_eq!(pending_state.pending_files(), vec![file3.id]);
    assert_eq!(pending_state.completed.len(), 2);
}
```

## Out of Scope

1. **Event Schema Evolution**: How to handle adding/removing event variants after initial deployment (deferred to future PRD)
2. **Event Log Archival**: Long-term storage of completed event logs for audit trails (deferred)
3. **Cross-Repository Event Correlation**: Querying events across multiple contexts (deferred)
4. **Event-Driven Incremental Processing**: Using events to trigger incremental updates (deferred to file watcher PRD)
5. **Event Log Replay for Debugging**: Replaying event logs to reproduce bugs (deferred to debugging tools PRD)
6. **Parallel Processor Execution**: Contexts still run sequentially (parallel execution in future PRD)
7. **Property Bank Processor Event Sourcing**: Property bank has different lifecycle (separate PRD if needed)

## Further Notes

### Relationship to Discovery PRD

This PRD depends on and extends the generic event infrastructure established in `.scratch/centralized-discovery-processor/PRD.md`:

**Provided by Discovery PRD**:
- `EventId` newtype (monotonic u64)
- `EventTable<V>` wrapper
- `EventStore` trait
- `EVENT_SEQUENCES` table
- Transactional EventId allocation strategy
- ArchivedEntity serialization pattern

**Added by This PRD**:
- Domain-specific event enums (SchemaEvent, NoteEvent, TemplateEvent, ConfigEvent)
- Projector patterns (PendingSchemaState, etc.)
- Resumption logic
- Compaction coordination
- Orchestrator integration

### ADR Alignment

This PRD implements **ADR 0004: Context-Specific Event Sourcing** with the following refinements:

**Changes from ADR**:
- EventStore is now a trait (not struct with AtomicU64)
- EventId allocation is transactional (EVENT_SEQUENCES table, not AtomicU64)
- Dependency graph corrected: `Config → Discovery → {Schema, Note} → Template` (Template depends on Schema+Note)
- Serialization uses ArchivedEntity trait (not raw bincode)

**Preserved from ADR**:
- Context-specific event tables (bounded context isolation)
- Intermediate event tracking (full lifecycle, not just Start/Complete)
- Projector pattern for state rehydration
- Dependency-aware cleanup timing

### Implementation Phases

**Phase 1: Schema Processor** (highest priority)
- Most complex pipeline (inheritance, property bank)
- Reference implementation for other contexts
- Validates event infrastructure patterns

**Phase 2: Note Processor** (high priority)
- Simpler pipeline (parsing, frontmatter, validation)
- Tests projector pattern consistency

**Phase 3: Config Processor** (medium priority)
- Different lifecycle (multi-file aggregation)
- Tests non-FileId event correlation

**Phase 4: Template Processor** (low priority)
- Depends on Schema+Note completion
- Tests dependency-aware compaction

**Phase 5: Orchestrator Integration** (after all contexts)
- Resumption coordination
- Compaction coordination
- Crash simulation tests
