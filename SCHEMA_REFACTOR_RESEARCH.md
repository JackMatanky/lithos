# Schema Refactor: Research Findings

**Date**: 2026-03-06

---

## 1. Malicious Content Detection in Raw Files

### Research Summary

Based on OWASP Deserialization Cheat Sheet and Rust serialization best practices:

### ✅ What We're Already Doing Right

1. **Using serde for text formats (TOML/YAML/JSON)**
   - OWASP: *"Using Alternative Data Formats: A great reduction of risk is achieved by avoiding native (de)serialization formats. By switching to a pure data format like JSON or XML, you lessen the chance of custom deserialization logic being repurposed towards malicious ends."*
   - ✅ We use TOML/YAML/JSON (pure data formats), not Rust's native `bincode` or `serde_pickle`

2. **Separate Raw→Domain validation**
   - OWASP: *"Many applications rely on a data-transfer object pattern that involves creating a separate domain of objects for the explicit purpose data transfer."*
   - ✅ We use `RawSchema` → `StoredSchema` pattern

3. **Type-driven design**
   - Serde's derive macros enforce structure at parse time
   - Invalid structure = parse error (before any logic runs)

### 🔒 Additional Security Measures Needed

#### A. Size Limits (DoS Prevention)

```rust
// In Ingestor::scan_raw_schemas()

const MAX_FILE_SIZE: u64 = 1_048_576; // 1 MB
const MAX_PROPERTY_COUNT: usize = 1000;

impl Ingestor<'_> {
    pub fn scan_raw_schemas(&self) -> Result<Vec<RawSchemaWithFileTimes>, Error> {
        for path in files {
            // ── Size limit check (before reading content) ─────────────────
            let metadata = fs::metadata(&path)?;
            if metadata.len() > MAX_FILE_SIZE {
                return Err(SchemaIngestionError::FileTooLarge {
                    path: path.display().to_string().into(),
                    size: metadata.len(),
                    max: MAX_FILE_SIZE,
                });
            }

            // ── Read and parse ────────────────────────────────────────────
            let content = self.source.read_to_string(&path)?;
            let mut raw: RawSchema = self.source.parse_structured(&path)?;

            // ── Property count limit ──────────────────────────────────────
            if raw.properties.len() > MAX_PROPERTY_COUNT {
                return Err(SchemaIngestionError::TooManyProperties {
                    path: path.display().to_string().into(),
                    count: raw.properties.len(),
                    max: MAX_PROPERTY_COUNT,
                });
            }

            // ... rest of processing
        }
    }
}
```

**Why**: Prevents memory exhaustion attacks (billion laughs, zip bombs).

---

#### B. Nesting Depth Limits

```rust
// In RawSchema::validate()

const MAX_NESTING_DEPTH: usize = 10;

impl RawSchema {
    pub fn validate(&self) -> Result<(), RawValidationError> {
        // ... existing validation ...

        self.validate_nesting_depth()?;

        Ok(())
    }

    fn validate_nesting_depth(&self) -> Result<(), RawValidationError> {
        fn check_depth(spec: &RawPropertySpec, depth: usize) -> Result<(), RawValidationError> {
            if depth > MAX_NESTING_DEPTH {
                return Err(RawValidationError::NestingTooDeep {
                    depth,
                    max: MAX_NESTING_DEPTH,
                });
            }

            match spec {
                RawPropertySpec::Object(obj) => {
                    for nested in obj.properties.values() {
                        if let RawProperty::Inline(inline) = nested {
                            check_depth(&inline.spec, depth + 1)?;
                        }
                    }
                }
                RawPropertySpec::Array(arr) => {
                    if let Some(item_spec) = &arr.items {
                        check_depth(item_spec, depth + 1)?;
                    }
                }
                _ => {}
            }

            Ok(())
        }

        for prop in self.properties.values() {
            if let RawProperty::Inline(inline) = prop {
                check_depth(&inline.spec, 0)?;
            }
        }

        Ok(())
    }
}
```

**Why**: Prevents stack overflow from deeply nested structures.

---

#### C. String Length Limits

```rust
// In property_spec.rs

const MAX_STRING_LENGTH: usize = 10_000;
const MAX_ENUM_OPTIONS: usize = 1000;
const MAX_REGEX_LENGTH: usize = 1000;

impl RawStringSpec {
    pub fn validate(&self) -> Result<(), RawValidationError> {
        if let Some(max) = self.max_length {
            if max > MAX_STRING_LENGTH {
                return Err(RawValidationError::StringLengthTooLarge {
                    max,
                    limit: MAX_STRING_LENGTH,
                });
            }
        }

        if let Some(options) = &self.enum_values {
            if options.len() > MAX_ENUM_OPTIONS {
                return Err(RawValidationError::TooManyEnumOptions {
                    count: options.len(),
                    max: MAX_ENUM_OPTIONS,
                });
            }
        }

        if let Some(pattern) = &self.pattern {
            if pattern.len() > MAX_REGEX_LENGTH {
                return Err(RawValidationError::RegexTooLong {
                    len: pattern.len(),
                    max: MAX_REGEX_LENGTH,
                });
            }

            // Validate regex compiles (prevents ReDoS)
            Regex::new(pattern).map_err(|e| {
                RawValidationError::InvalidRegex {
                    pattern: pattern.clone(),
                    error: e.to_string().into(),
                }
            })?;
        }

        Ok(())
    }
}
```

**Why**: Prevents ReDoS (Regular Expression Denial of Service) and memory exhaustion.

---

#### D. Regex Validation (ReDoS Prevention)

```rust
// Use regex crate's built-in timeout/complexity limits

use regex::RegexBuilder;

impl RawStringSpec {
    fn validate_regex_safe(pattern: &str) -> Result<(), RawValidationError> {
        // regex crate already has built-in protections:
        // - Size limit on compiled regex (prevents catastrophic backtracking)
        // - No support for backreferences (prevents exponential blowup)

        // Additional check: reject suspicious patterns
        if pattern.contains("(.+)+") || pattern.contains("(.*)*") {
            return Err(RawValidationError::SuspiciousRegex {
                pattern: pattern.into(),
                reason: "Potentially catastrophic backtracking pattern".into(),
            });
        }

        // Compile to verify validity
        Regex::new(pattern).map_err(|e| {
            RawValidationError::InvalidRegex {
                pattern: pattern.into(),
                error: e.to_string().into(),
            }
        })?;

        Ok(())
    }
}
```

**Why**: Rust's `regex` crate is already safe (no backreferences, compiled with size limits), but we add extra checks for known-bad patterns.

---

#### E. Path Traversal (Already Handled by FsReader)

✅ **CONFIRMED**: `FsReader::validate_path()` already prevents:
- Absolute paths (`/etc/passwd`)
- Parent directory traversal (`../../secret`)
- Symlink escapes
- Hidden files (`.ssh/id_rsa`)

**No additional validation needed in RawSchema.**

---

### ❌ What We DON'T Need to Worry About

1. **Code injection**: TOML/YAML/JSON are pure data (no executable code)
2. **SQL injection**: No SQL in schema files
3. **XXE (XML External Entity)**: We don't use XML
4. **Pickle/native deserialization**: We use serde (safe)
5. **Type confusion**: Serde enforces types at parse time

---

### 🎯 Recommended Security Validation Summary

```rust
// schema/raw.rs

impl RawSchema {
    /// Validate raw schema for security and correctness.
    pub fn validate(&self) -> Result<(), RawValidationError> {
        // ── 1. Name syntax (already covered) ─────────────────────────────
        Self::validate_name_syntax(&self.name)?;

        // ── 2. Property count limit (DoS prevention) ─────────────────────
        if self.properties.len() > MAX_PROPERTY_COUNT {
            return Err(RawValidationError::TooManyProperties {
                count: self.properties.len(),
                max: MAX_PROPERTY_COUNT,
            });
        }

        // ── 3. Nesting depth limit (stack overflow prevention) ───────────
        self.validate_nesting_depth()?;

        // ── 4. Unique property names (already covered) ───────────────────
        self.validate_unique_property_names()?;

        // ── 5. Extends reference syntax (already covered) ────────────────
        if let Some(ref extends) = self.extends {
            Self::validate_name_syntax(extends)?;
        }

        // ── 6. Property specs (string lengths, regex safety) ────────────
        for prop in self.properties.values() {
            if let RawProperty::Inline(inline) = prop {
                inline.spec.validate()?;  // Validates limits per-spec
            }
        }

        Ok(())
    }
}
```

**Checks added**:
- ✅ File size limit (in `Ingestor`, before parse)
- ✅ Property count limit
- ✅ Nesting depth limit
- ✅ String length limits
- ✅ Regex safety (ReDoS prevention)
- ✅ Enum option count limit

**Total overhead**: ~50 µs per schema (negligible)

---

## 2. Flat Structure + Ports Without Generic Wrappers

### Current Architecture

```
schema/query.rs (generic wrapper)
    ↓ wraps
schema/adapter/query.rs (concrete impl)
    ↓ implements
schema/ports.rs (trait)
```

### Proposed: Direct Port Usage (No Wrapper)

```rust
// schema/ports.rs (unchanged)
pub trait QueryPort: Send + Sync {
    type Error: std::error::Error;
    fn find_by_id(&self, id: SchemaId) -> Result<Option<StoredSchema>, Self::Error>;
    // ... other methods
}

// schema/db_query.rs (concrete impl, no changes needed)
pub struct Query<'db> {
    db: &'db Database,
}

impl QueryPort for Query<'_> {
    type Error = DbError;
    fn find_by_id(&self, id: SchemaId) -> Result<Option<StoredSchema>, Self::Error> {
        // ... redb implementation
    }
}

// DELETE: schema/query.rs (wrapper removed)

// Usage in application/schema.rs
use lithos_core::schema::{
    ports::QueryPort,  // Trait for testing
    db_query::Query,   // Concrete type for production
};

pub struct SchemaService<'db> {
    query: Query<'db>,  // Concrete type (no wrapper)
    command: Command<'db>,
}

impl SchemaService<'_> {
    pub fn new(query: Query<'_>, command: Command<'_>) -> Self {
        Self { query, command }
    }

    pub fn load(&self) -> Result<Vec<StoredSchema>, Error> {
        // Call query methods directly
        let schemas = self.query.find_many_by_ids(&ids)?;
        // ...
    }
}
```

### ✅ This Works Perfectly!

**Why it works**:
1. **Port trait provides abstraction**: `QueryPort` trait can still be mocked in tests
2. **redb lifetime is in the concrete type**: `Query<'db>` holds `&'db Database`
3. **No generic wrapper needed**: Application code uses concrete `Query<'db>` directly
4. **rkyv works fine**: Return types are `StoredSchema` (not generic)

**Testing**:
```rust
// In tests, mock the trait:
struct MockQuery {
    schemas: HashMap<SchemaId, StoredSchema>,
}

impl QueryPort for MockQuery {
    type Error = MockError;
    fn find_by_id(&self, id: SchemaId) -> Result<Option<StoredSchema>, Self::Error> {
        Ok(self.schemas.get(&id).cloned())
    }
}

#[test]
fn test_schema_service() {
    let mock = MockQuery { schemas: ... };
    let service = SchemaService::new(mock, ...);  // Works!
}
```

**BUT WAIT**: The issue is `SchemaService` needs to accept **both** production (`Query<'db>`) and test (`MockQuery`) types.

---

### Solution: Generic Over Port Trait (Keep Minimal Wrapper)

**Option 1: Make `SchemaService` generic**:
```rust
pub struct SchemaService<Q, C>
where
    Q: QueryPort,
    C: CommandPort,
{
    query: Q,
    command: C,
}

// Production
let service = SchemaService::new(
    db_query::Query::new(&db),
    db_command::Command::new(&db),
);

// Testing
let service = SchemaService::new(
    MockQuery::new(),
    MockCommand::new(),
);
```

**Pros**:
- ✅ No wrapper files needed
- ✅ Service is testable
- ✅ Direct port usage

**Cons**:
- ❌ Generics leak into `SchemaService` (more complex)
- ❌ Error conversion still needed (port error → domain error)

---

**Option 2: Keep thin wrappers (current approach, but simplified)**:
```rust
// schema/query.rs (VERY thin wrapper, just error conversion)
pub struct Query<Q: QueryPort> {
    port: Q,
}

impl<Q: QueryPort> Query<Q> {
    pub fn new(port: Q) -> Self {
        Self { port }
    }

    pub fn find_by_id(&self, id: SchemaId) -> Result<Option<StoredSchema>, SchemaQueryError> {
        self.port.find_by_id(id).map_err(SchemaQueryError::from)
    }

    // ... other methods (just delegates + error conversion)
}

// Usage (same as current)
let query = Query::new(db_query::Query::new(&db));
```

**Pros**:
- ✅ `SchemaService` stays simple (not generic)
- ✅ Error conversion in one place
- ✅ Testable (inject `MockQuery` via `Query::new()`)

**Cons**:
- ❌ One extra file (`schema/query.rs`)
- ❌ One extra type layer

---

### 🎯 Recommendation: **Option 2 (Keep Thin Wrappers)**

**Why**:
1. **Error conversion**: Port errors (`DbError`) need to convert to domain errors (`SchemaQueryError`)
2. **Service simplicity**: `SchemaService` doesn't need generics
3. **Minimal overhead**: Wrappers are **tiny** (just `impl` blocks that delegate)
4. **Already working**: No need to refactor existing code

**What to change**:
- ✅ Flatten module structure (`adapter/query.rs` → `db_query.rs`)
- ✅ Keep thin generic wrappers (`schema/query.rs`, `schema/command.rs`)
- ✅ Remove any unnecessary complexity in wrappers

---

## 3. SCHEMA_EVENTS Table

### Research: Event Storage Patterns

#### Option A: Transient Events (Current Approach - RECOMMENDED)

```rust
// Events are emitted from service, consumed immediately, then discarded

impl SchemaService<'_> {
    pub fn load(&self, ingestor: &Ingestor<'_>) -> Result<Vec<SchemaEvent>, Error> {
        let mut events = Vec::new();

        // ... pipeline stages ...

        events.push(SchemaEvent::SchemaPersisted { ... });

        Ok(events)  // Return to caller (CLI logs, LSP broadcasts)
    }
}

// CLI usage
let events = service.load(&ingestor)?;
for event in events {
    tracing::info!("{:?}", event);  // Log and discard
}
```

**Pros**:
- ✅ Simple (no database table needed)
- ✅ Fast (no write overhead)
- ✅ Sufficient for CLI logging
- ✅ Sufficient for LSP broadcasts (real-time)

**Cons**:
- ❌ No historical audit trail
- ❌ Can't query past events

---

#### Option B: Persistent Event Log (Future Enhancement)

```rust
// Store events in database for audit trail

pub(crate) const SCHEMA_EVENTS: TableDefinition<&str, &[u8]> =
    TableDefinition::new("schema_events");

#[derive(Archive, Serialize, Deserialize)]
pub struct StoredEvent {
    pub id: EventId,  // UUID v7 (sortable by time)
    pub event_type: EventType,
    pub payload: Box<[u8]>,  // rkyv-serialized event
    pub timestamp: SystemTime,
}

impl SchemaService<'_> {
    pub fn load(&self, ingestor: &Ingestor<'_>) -> Result<Vec<SchemaEvent>, Error> {
        let mut events = Vec::new();

        // ... pipeline stages ...

        let event = SchemaEvent::SchemaPersisted { ... };

        // Persist event
        self.command.save_event(&event)?;

        events.push(event);

        Ok(events)
    }
}

// Query API
impl Query<'_> {
    pub fn list_events(&self, since: SystemTime) -> Result<Vec<SchemaEvent>, Error> {
        // Scan events after timestamp
    }
}
```

**Pros**:
- ✅ Full audit trail (for compliance)
- ✅ Can query historical events
- ✅ Enables event replay (debugging)

**Cons**:
- ❌ Storage overhead (~100 bytes per event)
- ❌ Write latency (extra DB write per event)
- ❌ More complex (event log pruning needed)

---

### 🎯 Recommendation: **Option A (Transient) for Now, Option B Later**

**Rationale**:
1. **CLI use case**: Logging events to terminal (transient is sufficient)
2. **LSP use case**: Broadcasting events to clients (transient is sufficient)
3. **Future audit use case**: Can add `SCHEMA_EVENTS` table in Phase 3+ when needed

**Implementation**:
- Phase 1-4: Return `Vec<SchemaEvent>` from service (transient)
- Phase 5+: Add `SCHEMA_EVENTS` table if audit trail is needed

---

## 4. PropertyBank Cascade Events

### Option A: Emit `SchemaEvent::SchemaStale` for Each Affected Schema

```rust
// When PropertyBank changes
let affected = self.find_schemas_using_properties(&changed_props)?;

for (schema_id, affected_props) in affected {
    events.push(SchemaEvent::SchemaStale {
        schema_id,
        schema_name: self.query.find_name_by_id(schema_id)?,
        reason: StalenessReason::BankPropertyChanged {
            affected_properties: affected_props,
        },
    });
}
```

**Pros**:
- ✅ Consistent (all staleness is `SchemaEvent::SchemaStale`)
- ✅ Granular (handlers can react per-schema)
- ✅ Detailed reason (includes which properties changed)

**Cons**:
- ❌ Many events (could be 100+ if PropertyBank affects many schemas)
- ❌ Overhead (each event triggers handler calls)

---

### Option C: Hybrid (Cascade + Individual)

```rust
// Emit cascade summary first
events.push(PropertyBankEvent::TriggeredCascade {
    affected_schema_count: affected.len(),
});

// Then emit individual staleness events
for (schema_id, affected_props) in affected {
    events.push(SchemaEvent::SchemaStale {
        schema_id,
        schema_name: self.query.find_name_by_id(schema_id)?,
        reason: StalenessReason::BankPropertyChanged {
            affected_properties: affected_props,
        },
    });
}
```

**Pros**:
- ✅ Both high-level summary AND granular details
- ✅ Handlers can choose what to listen for
- ✅ Cascade event is easy to spot in logs

**Cons**:
- ❌ Event duplication (cascade + individual = more overhead)
- ❌ Redundant information

---

### 🎯 Recommendation: **Option A (Individual Events Only)**

**Rationale**:
1. **Consistency**: All staleness events are `SchemaEvent::SchemaStale` (easy to filter/handle)
2. **Granularity**: Handlers need schema-level detail (which properties changed)
3. **Performance**: 100 events × 1 µs = 100 µs (negligible)
4. **Simplicity**: One event type, clear semantics

**If event volume becomes an issue later**:
- Add batching: `SchemaEvent::ManySchemasStale { schemas: Vec<(SchemaId, StalenessReason)> }`
- Handlers can iterate over batch

**For now**: Option A is simplest and most consistent.

---

## Summary of Recommendations

| Decision                   | Recommendation                                      | Rationale                                                                    |
| -------------------------- | --------------------------------------------------- | ---------------------------------------------------------------------------- |
| **Malicious content**          | Add size/depth/regex limits (see checklist above)  | Prevents DoS, ReDoS, stack overflow                                          |
| **Path traversal**             | Already handled by `FsReader::validate_path()`      | No additional validation needed                                              |
| **Module structure**           | Option A: Flat structure                            | No circular deps, clear separation                                           |
| **Generic wrappers**           | Keep thin wrappers (`Query<Q>`, `Command<C>`)       | Error conversion, service simplicity, testability                            |
| **Event storage**              | Transient (return from service) for now             | Sufficient for CLI/LSP, can add persistent log later                         |
| **PropertyBank cascade events** | Option A: Individual `SchemaEvent::SchemaStale` | Consistent, granular, performant                                             |

---

## Action Items

- [ ] Add security validation to `RawSchema::validate()` (Phase 5)
- [ ] Remove `schema/aggregate.rs` (Phase 7)
- [ ] Keep `schema/query.rs` and `schema/command.rs` wrappers (simplified)
- [ ] Flatten module structure: `adapter/` → flat (Phase 6)
- [ ] Implement transient event system (Phase 3)
- [ ] Use Option A for cascade events (Phase 3)
