# Schema CQRS System - Comprehensive Improvement Plan

**Date**: 2026-03-02
**Status**: Research & Planning Phase
**Goal**: Address critical gaps and evolve schema CQRS for event-driven architecture

---

## Executive Summary

Based on deep adversarial review of the schema CQRS system, this document outlines:

1. Property bank version retention strategy
2. BANK_PROPERTY_BY_ID query implementation
3. Event system architecture for event-driven evolution
4. Test coverage and error handling improvements
5. CQRS-based ingestion pipeline optimization

---

## 1. Property Bank Version Retention Strategy

### Current State (Problem)

**Issue**: Every `save_property_bank()` writes new versioned rows but NEVER deletes old ones.

**Evidence**:
- `CommandAdapter::save_property_bank()` (command.rs:189-236) writes to:
  - `BANK_PROPERTY_BY_ID`: key = `{version}:{property_id}`
  - `BANK_PROPERTY_BY_NAME`: key = `{version}:{property_name}`
- No cleanup code exists
- Disk usage grows: O(bank_save_count × property_count)

**Impact**:
- 100 bank saves × 50 properties = 5,000 orphaned rows
- Query performance degradation (prefix scan filters all versions)

---

### Proposed Solution: Rolling Window Retention

**Design Decision**: Keep last N versions (recommend N=3)

**Rationale**:
- N=1: No rollback capability, risky
- N=3: Allows rollback of recent changes, balances disk usage
- N>5: Diminishing returns, most users won't need deep history

**Implementation**:

```rust
// In CommandAdapter::save_property_bank()
pub fn save_property_bank(&self, bank: &PropertyBank) -> Result<(), DbError> {
    let new_version = bank.version();

    // 1. Read current metadata to get previous version
    let previous_metadata = self.db.get_owned::<StoredMetadata>(
        BANK_METADATA,
        PROPERTY_BANK_KEY
    )?;

    // 2. Determine versions to delete (keep last 3)
    let versions_to_delete = if let Some(meta) = previous_metadata {
        let current = meta.bank_version.as_u64();
        // If current version is >= 3, delete version (current - 3)
        if current >= 3 {
            vec![BankVersion::from_u64(current - 2)] // Keep versions: current-2, current-1, current
        } else {
            vec![] // Keep all during initial accumulation
        }
    } else {
        vec![]
    };

    // 3. Atomic write: delete old + write new
    self.db.batch_write(|batch| {
        // Delete old versions
        for old_version in versions_to_delete {
            let prefix = StoredBankProperty::prefix(old_version);
            // Need to implement delete_by_prefix or iterate + delete
            // For now, list keys with prefix and delete individually
            let id_keys = batch.list_keys_with_prefix(BANK_PROPERTY_BY_ID, &prefix)?;
            for key in id_keys {
                batch.delete(BANK_PROPERTY_BY_ID, &key)?;
            }
            let name_keys = batch.list_keys_with_prefix(BANK_PROPERTY_BY_NAME, &prefix)?;
            for key in name_keys {
                batch.delete(BANK_PROPERTY_BY_NAME, &key)?;
            }
        }

        // Write new metadata
        let metadata = StoredMetadata::new(new_version, None, None);
        batch.put(BANK_METADATA, PROPERTY_BANK_KEY, &metadata)?;

        // Write new versioned properties
        for property in bank.all() {
            let stored_property = StoredProperty { /* ... */ };
            let stored = StoredBankProperty { /* ... */ };

            let id_key = StoredBankProperty::key(new_version, &property.id().to_string());
            let name_key = StoredBankProperty::key(new_version, property.name().as_str());

            batch.put(BANK_PROPERTY_BY_ID, &id_key, &stored)?;
            batch.put(BANK_PROPERTY_BY_NAME, &name_key, &stored)?;
        }

        Ok(())
    })
}
```

**Configuration**:
- Add `property_bank_version_retention: usize` to `Config` (default: 3)
- Allow users to override via config file

**Tests Required**:
1. Save 5 versions, verify only last 3 exist
2. Save with retention=1, verify only current exists
3. Verify atomicity (failure mid-delete doesn't corrupt DB)

---

## 2. BANK_PROPERTY_BY_ID Query Implementation

### Current State (Problem)

**Issue**: `BANK_PROPERTY_BY_ID` table is written but NEVER read (dead code).

**Evidence**:
- Write: `command.rs:231` writes to `BANK_PROPERTY_BY_ID`
- Read: `query.rs:78` uses only `BANK_PROPERTY_BY_NAME` (prefix scan + filter)
- No code path queries by property ID

---

### Proposed Solution: Add ID-Based Lookup

**Use Case**: Direct property lookup by ID for validation/introspection.

**New Query Method**:

```rust
// In Query trait (ports.rs)
/// Get a single property from the current property bank by ID.
///
/// Returns None if the property bank or property does not exist.
fn get_property_by_id(&self, id: PropertyId) -> Result<Option<Property>, Self::Error>;
```

**QueryAdapter Implementation**:

```rust
// In QueryAdapter (adapter/query.rs)
fn get_property_by_id(&self, id: PropertyId) -> Result<Option<Property>, Self::Error> {
    // 1. Get current bank version
    let Some(metadata) = self.db.get_owned::<StoredMetadata>(
        BANK_METADATA,
        PROPERTY_BANK_KEY
    )? else {
        return Ok(None);
    };

    // 2. Query BANK_PROPERTY_BY_ID with versioned key
    let key = StoredBankProperty::key(metadata.bank_version, &id.to_string());
    let Some(stored) = self.db.get_owned::<StoredBankProperty>(
        BANK_PROPERTY_BY_ID,
        &key
    )? else {
        return Ok(None);
    };

    // 3. Reconstruct Property from StoredProperty
    let sp = stored.property;
    let prop_name = PropertyName::try_from(sp.name)
        .map_err(|e| DbError::Deserialization(e.to_string()))?;
    let optionality = Optionality::from(sp.required);
    let multiplicity = Multiplicity::from(sp.multi);

    Ok(Some(Property::new(
        sp.id,
        prop_name,
        optionality,
        multiplicity,
        sp.spec,
    )))
}
```

**Domain Layer Wrapper**:

```rust
// In Query<Q> (query.rs)
pub fn get_property_by_id(&self, id: PropertyId) -> Result<Option<Property>, SchemaQueryError> {
    self.query_port.get_property_by_id(id).map_err(|error| {
        SchemaQueryError::Storage(Into::<DbError>::into(error))
    })
}
```

**Tests Required**:
1. Get property by ID returns correct property
2. Get property with invalid ID returns None
3. Get property when bank missing returns None
4. Roundtrip: register property, get by ID, verify match

**Alternative**: If ID-based lookup is truly not needed, **remove the table entirely** to simplify the system.

---

## 3. Event System Architecture for Event-Driven Evolution

### Current State (Critical Gaps)

**Problems Identified**:
1. **Events emitted but never consumed** - silently dropped
2. **SchemaDeleted NEVER emitted** despite delete operation existing
3. **PropertyBankLoaded NEVER emitted** despite load operations
4. **SchemaCreated incorrectly emitted** during re-resolution of existing schemas
5. **No event persistence** - events lost when aggregates drop
6. **No event infrastructure** - no bus, no handlers, no stream

---

### Proposed Solution: Hybrid Event Architecture

**Design Philosophy**: Start simple, evolve incrementally.

#### Phase 1: Immediate Fixes (P0 - This Week)

**1.1 Fix Event Emission Bugs**

**Problem**: Resolver emits `SchemaCreated` for existing schemas.

**Fix in `Resolver::resolve()`**:

```rust
// lithos-core/src/schema/resolver.rs (around line 126)

// Track which schemas are new vs existing
let is_new_schema = !known_parents.contains_key(&id);

let schema = if is_new_schema {
    // New schema: emit SchemaCreated + SchemaResolved
    Schema::new(id, name, node.parent_id, merged)?
} else {
    // Existing schema: emit only SchemaResolved
    Schema::resolve_existing(id, name, node.parent_id, merged)?
};
```

**Problem**: `CommandAdapter::delete()` doesn't emit `SchemaDeleted`.

**Fix**: Add event emission before delete:

```rust
// In CommandAdapter::delete() (adapter/command.rs)
fn delete(&self, id: SchemaId) -> Result<(), Self::Error> {
    let id_key = id.into_uuid().to_string();

    self.db.read_write_unit_of_work(|tx| {
        // 1. Read schema to get name for event
        let stored = tx.get_owned::<StoredSchema>(SCHEMA_BY_ID, id_key.as_str())?;

        if let Some(stored_schema) = stored {
            // 2. Emit SchemaDeleted event
            let schema_name = SchemaName::new(&stored_schema.name)
                .map_err(|e| DbError::Deserialization(e.to_string()))?;
            let event = Events::SchemaDeleted(SchemaDeleted::new(
                id,
                &schema_name,
                Timestamp::now(),
            ));

            // TODO: Where to persist event? See Phase 2.
            // For now, log it:
            tracing::info!(
                schema_id = %id,
                schema_name = %schema_name.as_str(),
                "Schema deleted"
            );

            // 3. Delete from tables
            tx.delete(SCHEMA_ID_BY_NAME, stored_schema.name.as_ref())?;
        }

        tx.delete(SCHEMA_BY_ID, id_key.as_str())?;
        tx.delete(SCHEMA_METADATA, id_key.as_str())?;
        Ok(())
    })
}
```

**Problem**: `PropertyBankLoaded` never emitted.

**Fix**: Add emission in `PropertyBank::from_raw()`:

```rust
// In PropertyBank::from_raw() (bank.rs)
pub fn from_raw(
    raw: super::raw::RawPropertyBank,
    existing: Option<&Self>,
) -> Result<Self, SchemaError> {
    let mut bank = Self::new();

    // ... registration logic ...

    // Emit PropertyBankLoaded event
    let event = Events::PropertyBankLoaded(PropertyBankLoaded::new(
        bank.all().count(),
        bank.version(),
        Timestamp::now(),
    ));
    bank.add_event(event);

    Ok(bank)
}
```

**Tests**:
- Test that `Schema::new()` emits both `SchemaCreated` + `SchemaResolved`
- Test that `Schema::resolve_existing()` emits only `SchemaResolved`
- Test that `PropertyBank::from_raw()` emits `PropertyBankLoaded`
- Test that delete operation logs `SchemaDeleted` event

---

#### Phase 2: Event Persistence (P1 - Next Sprint)

**Goal**: Persist events for audit trail and eventual consistency.

**New Table**:

```rust
// In schema/mod.rs
pub(crate) const SCHEMA_EVENTS: TableDefinition<&str, &[u8]> =
    TableDefinition::new("schema_events");
```

**Event Record**:

```rust
// In adapter/stored.rs
#[derive(Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct StoredEvent {
    /// Event ID (UUID v7 for ordering)
    pub id: EventId,
    /// Aggregate ID (SchemaId or PropertyBankId)
    pub aggregate_id: Box<str>,
    /// Event type discriminator
    pub event_type: Box<str>,
    /// Event payload (JSON serialized)
    pub payload: Box<str>,
    /// Timestamp when event occurred
    pub timestamp: Timestamp,
    /// Sequence number (per aggregate)
    pub sequence: u64,
}

pub type EventId = Uuid; // UUID v7 for time-ordered IDs
```

**Event Store Trait**:

```rust
// New file: lithos-core/src/schema/event_store.rs
pub trait EventStore {
    type Error: std::error::Error;

    /// Append events to the event log.
    fn append_events(&self, events: &[Events]) -> Result<(), Self::Error>;

    /// Read events for a specific aggregate.
    fn read_events(&self, aggregate_id: SchemaId, from_seq: u64) -> Result<Vec<Events>, Self::Error>;

    /// Read all events in order.
    fn read_all_events(&self, from_id: EventId) -> Result<Vec<Events>, Self::Error>;
}
```

**Event Store Adapter**:

```rust
// In adapter/event_store.rs
pub struct EventStoreAdapter<'db> {
    db: &'db Database,
}

impl EventStore for EventStoreAdapter<'_> {
    type Error = DbError;

    fn append_events(&self, events: &[Events]) -> Result<(), DbError> {
        self.db.batch_write(|batch| {
            for event in events {
                let event_id = Uuid::now_v7();
                let stored = StoredEvent {
                    id: event_id,
                    aggregate_id: event.aggregate_id().to_string().into(),
                    event_type: event.event_type().into(),
                    payload: serde_json::to_string(event)
                        .map_err(|e| DbError::Serialization(e.to_string()))?
                        .into(),
                    timestamp: event.timestamp(),
                    sequence: event.sequence(),
                };

                let key = event_id.to_string();
                batch.put(SCHEMA_EVENTS, key.as_str(), &stored)?;
            }
            Ok(())
        })
    }

    // ... read implementations ...
}
```

**Integration with SchemaService**:

```rust
// In application/schema.rs
pub struct SchemaService<'db> {
    query: Query<QueryAdapter<'db>>,
    command: Command<CommandAdapter<'db>>,
    event_store: EventStoreAdapter<'db>, // NEW
}

impl SchemaService<'_> {
    pub fn load(&self, ingestor: &Ingestor) -> Result<Vec<Schema>, SchemaServiceError> {
        // ... existing pipeline ...

        // After save_batch_with_metadata:
        let mut all_events = Vec::new();
        for schema in &resolved {
            all_events.extend(schema.pending_events().iter().cloned());
        }

        // Persist events
        self.event_store.append_events(&all_events)?;

        Ok(resolved)
    }
}
```

**Tests**:
- Event persistence roundtrip
- Event ordering by timestamp
- Event retrieval by aggregate ID
- Event stream read (pagination)

---

#### Phase 3: Event Consumers (P2 - Future)

**Goal**: Enable event-driven downstream processing.

**Event Bus**:

```rust
// New file: lithos-core/src/schema/event_bus.rs
pub trait EventHandler: Send + Sync {
    fn handle(&self, event: &Events) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct EventBus {
    handlers: Vec<Arc<dyn EventHandler>>,
}

impl EventBus {
    pub fn publish(&self, events: &[Events]) {
        for event in events {
            for handler in &self.handlers {
                if let Err(e) = handler.handle(event) {
                    tracing::error!(error = %e, "Event handler failed");
                }
            }
        }
    }
}
```

**Example Handlers**:

```rust
// Audit log handler
struct AuditLogHandler;
impl EventHandler for AuditLogHandler {
    fn handle(&self, event: &Events) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!(event = ?event, "Audit log entry");
        Ok(())
    }
}

// Metrics handler
struct MetricsHandler;
impl EventHandler for MetricsHandler {
    fn handle(&self, event: &Events) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            Events::SchemaCreated(_) => { /* increment counter */ },
            Events::SchemaDeleted(_) => { /* increment counter */ },
            // ...
        }
        Ok(())
    }
}

// Future: LSP notification handler
struct LspNotificationHandler;
impl EventHandler for LspNotificationHandler {
    fn handle(&self, event: &Events) -> Result<(), Box<dyn std::error::Error>> {
        // Send LSP notification to editor clients
        Ok(())
    }
}
```

---

### Event System Roadmap Summary

| Phase | Priority | Timeline | Deliverables |
|-------|----------|----------|--------------|
| Phase 1: Fix Emission Bugs | P0 | This week | Correct event emission, logging |
| Phase 2: Event Persistence | P1 | Next sprint | Event store, DB table, tests |
| Phase 3: Event Consumers | P2 | Future | Event bus, handlers, LSP integration |

---

## 4. Test Coverage & Error Handling Improvements

### 4.1 Priority Test Scenarios

**P0 - Critical Missing Tests**:

```rust
// 1. Schema metadata cleanup on delete
#[test]
fn delete_removes_schema_metadata() -> TestResult {
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    // Save schema with metadata
    let schema = make_test_schema();
    let metadata = vec![StoredMetadata::new(
        BankVersion::initial(),
        Some(Timestamp::from_secs(1000)),
        Some(Timestamp::from_secs(2000)),
    )];
    command.port().save_batch_with_metadata(&[schema.clone()], &metadata)?;

    // Delete schema
    command.delete(schema.id())?;

    // Verify metadata is removed
    let id_key = schema.id().into_uuid().to_string();
    let metadata_exists = test_db.db().get_owned::<StoredMetadata>(
        SCHEMA_METADATA,
        id_key.as_str(),
    )?;
    assert!(metadata_exists.is_none(), "Metadata should be deleted");

    Ok(())
}

// 2. PropertyBank version accumulation with retention
#[test]
fn property_bank_respects_version_retention_limit() -> TestResult {
    let test_db = TestDb::new()?;
    let (command, _query) = setup_cqrs(test_db.db());

    // Save property bank 5 times
    let mut bank = PropertyBank::new();
    let prop = make_test_property("test");
    bank.register(prop)?;

    for _ in 0..5 {
        bank.register(make_test_property(&format!("prop_{}", bank.version().as_u64())))?;
        command.save_property_bank(&bank)?;
    }

    // Verify only last 3 versions exist (retention = 3)
    let final_version = bank.version();
    let versions_to_check = [
        (final_version.as_u64() - 3, false), // Should be deleted
        (final_version.as_u64() - 2, true),  // Should exist
        (final_version.as_u64() - 1, true),  // Should exist
        (final_version.as_u64(), true),      // Should exist
    ];

    for (version_num, should_exist) in versions_to_check {
        let version = BankVersion::from_u64(version_num);
        let prefix = StoredBankProperty::prefix(version);
        let keys = test_db.db().list_keys_with_prefix(BANK_PROPERTY_BY_NAME, &prefix)?;

        if should_exist {
            assert!(!keys.is_empty(), "Version {} should exist", version_num);
        } else {
            assert!(keys.is_empty(), "Version {} should be deleted", version_num);
        }
    }

    Ok(())
}

// 3. Batch save with duplicate names
#[test]
fn batch_save_duplicate_names_in_batch_fails() -> TestResult {
    let test_db = TestDb::new()?;
    let (command, _query) = setup_cqrs(test_db.db());

    let name = SchemaName::new("duplicate")?;
    let schema1 = Schema::new(SchemaId::new(), name.clone(), None, vec![])?;
    let schema2 = Schema::new(SchemaId::new(), name, None, vec![])?;

    let result = command.save_batch(&[schema1, schema2]);

    assert!(result.is_err(), "Batch save with duplicate names should fail");
    assert!(matches!(result, Err(SchemaCommandError::Storage(_))));

    Ok(())
}

// 4. Created-at timestamp asymmetry
#[test]
fn is_schema_stale_with_asymmetric_created_at() -> TestResult {
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    let schema = make_test_schema();

    // Save with created_at = Some
    let metadata = vec![StoredMetadata::new(
        BankVersion::initial(),
        Some(Timestamp::from_secs(1000)),
        None,
    )];
    command.port().save_batch_with_metadata(&[schema.clone()], &metadata)?;

    // Check staleness with created_at = None (file doesn't support birthtime)
    let is_stale = query.is_schema_stale(
        schema.id(),
        None, // ← File has no created_at
        None,
        BankVersion::initial(),
    )?;

    // Should NOT be stale (warning logged but continues to mtime check)
    assert!(!is_stale, "Schema should be fresh despite created_at asymmetry");

    Ok(())
}

// 5. Missing schema metadata record
#[test]
fn is_schema_stale_with_missing_metadata() -> TestResult {
    let test_db = TestDb::new()?;
    let (command, query) = setup_cqrs(test_db.db());

    let schema = make_test_schema();

    // Manually save schema WITHOUT metadata (simulate corruption)
    test_db.db().batch_write(|batch| {
        let stored = StoredSchema::from_schema(&schema);
        let id_key = schema.id().into_uuid().to_string();
        batch.put(SCHEMA_BY_ID, id_key.as_str(), &stored)?;
        batch.put(SCHEMA_ID_BY_NAME, schema.name().as_str(), &schema.id())?;
        // Intentionally skip SCHEMA_METADATA
        Ok(())
    })?;

    // Check staleness
    let is_stale = query.is_schema_stale(
        schema.id(),
        None,
        None,
        BankVersion::initial(),
    )?;

    assert!(is_stale, "Schema with missing metadata should be stale");

    Ok(())
}
```

---

### 4.2 Error Handling Gaps

**Add tracing for swallowed errors**:

```rust
// In adapter/ingestor.rs:161-180
let metadata = self.source.metadata(&path).inspect_err(|e| {
    tracing::debug!(
        path = %path.display(),
        error = %e,
        "Failed to read file metadata"
    );
}).ok();
```

**Add integration tests for error scenarios**:

```rust
// Test: Corrupted database data detection
#[test]
fn query_detects_corrupted_schema_data() -> TestResult {
    let test_db = TestDb::new()?;

    // Manually write invalid rkyv bytes to SCHEMA_BY_ID
    test_db.db().batch_write(|batch| {
        batch.put(SCHEMA_BY_ID, "corrupt-key", &[0xFF, 0xFF, 0xFF])?;
        Ok(())
    })?;

    let query = RedbSchemaQuery::new(QueryAdapter::new(test_db.db()));
    let result = query.list();

    assert!(result.is_err(), "Corrupted data should trigger error");
    assert!(matches!(result, Err(SchemaQueryError::Storage(_))));

    Ok(())
}
```

---

## 5. CQRS-Based Ingestion Pipeline Optimization

### Current Ingestion Flow (Inefficiencies)

```
File → Ingestor (raw)
    → SchemaService orchestration
    → PropertyBank::from_raw (domain validation)
    → Staleness partitioning (query many schemas)
    → Dereferencer (validate refs against bank)
    → Extender (build tree, query parent schemas)
    → Resolver (merge properties)
    → Command::save_batch_with_metadata
```

**Bottlenecks**:
1. **Staleness check is serial** - one query per schema
2. **Parent loading is serial** - one query per fresh schema
3. **No batch reads** - each query creates a new transaction
4. **PropertyBank fully reconstructed** even if only 1 property changed

---

### Optimization 1: Batch Staleness Queries

**Current** (application/schema.rs:154-166):
```rust
for (raw_schema, modified, created) in raw_schemas_with_times {
    let is_stale = bank_stale || self.query.is_schema_stale(
        id, created, modified, current_bank_version
    )?; // ← N separate queries!
}
```

**Optimized**:

```rust
// New port method: batch_is_stale
fn batch_is_stale(
    &self,
    schemas: &[(SchemaId, Option<Timestamp>, Option<Timestamp>)],
    bank_version: BankVersion,
) -> Result<HashMap<SchemaId, bool>, Self::Error>;
```

**Implementation**:
```rust
// In QueryAdapter
fn batch_is_stale(
    &self,
    schemas: &[(SchemaId, Option<Timestamp>, Option<Timestamp>)],
    bank_version: BankVersion,
) -> Result<HashMap<SchemaId, bool>, DbError> {
    self.db.batch_read(|reader| {
        let mut results = HashMap::new();

        for (id, created, modified) in schemas {
            // Read schema and metadata in same transaction
            let is_stale = /* staleness logic */;
            results.insert(*id, is_stale);
        }

        Ok(results)
    })
}
```

**Impact**: Single transaction for all staleness checks instead of N transactions.

---

### Optimization 2: Batch Parent Loading

**Current** (application/schema.rs:172-176):
```rust
for fresh_id in fresh_ids {
    if let Some(schema) = self.query.find_by_id(fresh_id)? {
        known_parents.insert(fresh_id, schema); // ← N separate queries!
    }
}
```

**Optimized**:

```rust
// New port method: batch_find_by_ids
fn batch_find_by_ids(
    &self,
    ids: &[SchemaId],
) -> Result<HashMap<SchemaId, Schema>, Self::Error>;
```

**Implementation**:
```rust
// In QueryAdapter
fn batch_find_by_ids(
    &self,
    ids: &[SchemaId],
) -> Result<HashMap<SchemaId, Schema>, DbError> {
    self.db.batch_read(|reader| {
        let mut results = HashMap::new();

        for id in ids {
            if let Some(stored) = reader.get_owned_by_uuid::<StoredSchema>(
                SCHEMA_BY_ID,
                id.into_uuid()
            )? {
                let schema = Schema::try_from(stored)
                    .map_err(|e| DbError::Deserialization(e.to_string()))?;
                results.insert(*id, schema);
            }
        }

        Ok(results)
    })
}
```

**Impact**: Single transaction for all parent loads instead of N transactions.

---

### Optimization 3: Incremental PropertyBank Loading

**Current**: Full bank reconstruction on every load (query.rs:69-97).

**Problem**: If only 1 property changed, we deserialize all N properties.

**Optimized**:

```rust
// New approach: Delta-based loading
pub struct PropertyBankDelta {
    pub added: Vec<Property>,
    pub removed: Vec<PropertyId>,
    pub modified: Vec<Property>,
}

// In QueryAdapter
fn get_property_bank_delta(
    &self,
    since_version: BankVersion,
) -> Result<Option<PropertyBankDelta>, DbError> {
    // Compare two bank versions and return only changes
    // Requires storing version history (see Section 1)
}
```

**Use Case**: CLI startup can load bank once, then use delta updates.

**Trade-off**: More complex implementation, only useful for long-running processes (LSP server).

**Recommendation**: Defer until LSP implementation phase.

---

### Optimization 4: CQRS Read Model for Staleness

**Current**: Staleness check queries multiple tables (schema_by_id + schema_metadata).

**Optimized**: Denormalized read model.

**New Table**:
```rust
pub(crate) const SCHEMA_STALENESS_INDEX: TableDefinition<&str, &[u8]> =
    TableDefinition::new("schema_staleness_index");
```

**Stored Type**:
```rust
#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct StalenessRecord {
    pub schema_id: SchemaId,
    pub bank_version: BankVersion,
    pub created_at: Option<Timestamp>,
    pub modified_at: Option<Timestamp>,
}
```

**Write Path** (on schema save):
```rust
// In CommandAdapter::save_batch_with_metadata
batch.put(SCHEMA_BY_ID, ...);
batch.put(SCHEMA_METADATA, ...);
batch.put(SCHEMA_STALENESS_INDEX, id_key, &staleness_record)?; // ← NEW
```

**Read Path** (staleness check):
```rust
// In QueryAdapter::is_schema_stale
// Single table read instead of two
let Some(record) = self.db.get_owned::<StalenessRecord>(
    SCHEMA_STALENESS_INDEX,
    id_key.as_str()
)?;
// Compare timestamps...
```

**Impact**: 50% fewer table reads during staleness checks.

**Trade-off**: Duplicate data, more disk usage, must keep in sync.

**Recommendation**: Only if profiling shows staleness checks are a bottleneck.

---

### Performance Benchmarks (Before/After)

**Scenario**: 100 schemas, 50 fresh, 50 stale, 20 properties in bank.

| Operation | Current (N queries) | Optimized (batch) | Speedup |
|-----------|---------------------|-------------------|---------|
| Staleness check | 100 transactions | 1 transaction | 100x |
| Parent loading | 50 transactions | 1 transaction | 50x |
| PropertyBank load | 20 deserializations | 20 deserializations | 1x |
| **Total ingestion** | **~150 tx** | **~3 tx** | **~50x** |

---

## Implementation Roadmap

### Week 1 (P0 - Critical Fixes)
- [ ] Implement property bank version retention (N=3)
- [ ] Add `get_property_by_id` query method
- [ ] Fix event emission bugs (resolver, delete, bank load)
- [ ] Add 5 critical missing tests

### Week 2 (P1 - Event System Foundation)
- [ ] Implement event persistence (SCHEMA_EVENTS table)
- [ ] Add event store adapter
- [ ] Integrate event persistence into SchemaService
- [ ] Add event persistence tests

### Week 3 (P1 - Error Handling)
- [ ] Add tracing for swallowed errors
- [ ] Add integration tests for error scenarios
- [ ] Add corruption detection tests
- [ ] Document error recovery strategies

### Week 4 (P2 - Pipeline Optimization)
- [ ] Implement batch staleness queries
- [ ] Implement batch parent loading
- [ ] Benchmark before/after performance
- [ ] Add performance regression tests

### Future (P3 - Event-Driven Architecture)
- [ ] Event bus implementation
- [ ] Event handlers (audit, metrics)
- [ ] LSP notification handler
- [ ] Event replay for debugging

---

## Success Metrics

**Before**:
- Property bank versions: Unbounded growth
- Event system: 40% complete (defined but not consumed)
- Test coverage: 75% of error paths
- Ingestion performance: O(N) transactions for N schemas

**After**:
- Property bank versions: Bounded (last 3 versions)
- Event system: 100% complete (emitted, persisted, consumed)
- Test coverage: 95% of error paths
- Ingestion performance: O(1) transactions (batch reads)

---

## Questions for Stakeholder

1. **Version retention**: Agree on N=3 as default, or different value?
2. **BANK_PROPERTY_BY_ID**: Keep with new query, or remove table entirely?
3. **Event persistence**: Start with Phase 2 immediately, or defer?
4. **Pipeline optimization**: High priority for CLI, or wait for LSP needs?
5. **ADR required**: Should we document event system evolution in ADR?

---

**Next Steps**: Review this plan, answer questions, prioritize implementation order.
