# Lithos Allocation Optimization Plan
**Generated**: 2026-02-11
**Status**: Implementation Ready

## Executive Summary

Comprehensive analysis identified **156 allocation sites** across the codebase, with **27 CRITICAL allocations in hot paths**. This research validates and extends the findings in `TODO_NOTE_OPTIMIZATIONS.md` with system-wide patterns.

### Key Findings

The issues identified in `TODO_NOTE_OPTIMIZATIONS.md` are **valid and extend throughout the entire codebase**:

1. **Database Layer (P0 - CRITICAL)**: Every DB operation allocates formatted keys
   - **Impact**: 100% of all read/write operations (15 allocation sites)
   - **Cost**: 36-100 bytes per operation
   - **Solution**: Replace `format!()` with `write!()` + pre-allocated buffers

2. **UUID Conversions (P0 - CRITICAL)**: Every ID-based lookup allocates 36-byte strings
   - **Impact**: All template/note/schema queries by ID (8 allocation sites)
   - **Cost**: 36 bytes × operations frequency
   - **Solution**: Add UUID-native database methods

3. **Command/Query Allocations** - Pattern exists everywhere (not just note module)
   - ✅ **CONFIRMED**: The note module analysis was correct
   - **Decision**: Accept as architectural constraint (redb transaction model)
   - Applies to all command modules throughout the codebase

4. **Template Composition (P0 - HOT PATH)**: Clones all template names into HashMap
   - **Impact**: Every template composition
   - **Cost**: ~20-50 bytes per template × template count
   - **Solution**: Use borrowed HashMap keys (`HashMap<&str, &Template>`)

5. **API Design (P1)**: Constructors force caller allocations (14 locations)
   - **Impact**: Ergonomics + unnecessary allocations at call sites
   - **Cost**: Variable
   - **Solution**: Change `new(name: String)` → `new(name: &str)` (Rust idiom)

### Impact Potential
- **Database layer optimization**: 30-40 fewer allocations per note operation
- **Query optimization**: 10-20 fewer allocations per query
- **API cleanup**: Improved ergonomics + reduced caller allocations
- **Total estimated reduction**: 50-80% of allocations in hot paths

### Implementation Effort
- **P0 tasks** (critical hot paths): 6-8 hours
- **P1 tasks** (high value): 5-6 hours
- **Total**: 12-16 hours for complete optimization

---

## 🔴 P0: CRITICAL (Hot Path - Immediate Action Required)

### 1. Database Key Formatting - HIGHEST IMPACT

**Impact**: **100% of all database operations**

#### Current State

Every database operation allocates a formatted key string:

```rust
// db/mod.rs:136, 202, 270, 305
pub fn get<V, F, R>(&self, table: &str, key: &str, f: F) -> Result<Option<R>, DbError> {
    let namespaced_key = format!("{table}:{key}");  // ❌ EVERY DB READ/WRITE
    // ...
}

// db/mod.rs:374, 403, 432
pub fn multimap_insert(&self, key: &str, value: &str) -> Result<(), DbError> {
    let namespaced_key = format!("multimap:{key}");  // ❌ EVERY MULTIMAP OP
    // ...
}

// db/batch.rs:48, 66, 91, 116 - Same pattern in batch operations
```

**15 allocation sites total**

#### Locations

| File                   | Lines              | Pattern                               | Operations Affected          |
| ---------------------- | ------------------ | ------------------------------------- | ---------------------------- |
| `db/mod.rs`            | 136, 202, 270, 305 | `format!("{table}:{key}")`            | get, put, delete, get_owned  |
| `db/mod.rs`            | 374, 403, 432      | `format!("multimap:{key}")`           | multimap insert/remove/range |
| `db/mod.rs`            | 465                | `format!("{table}:")`                 | scan_prefix                  |
| `db/batch.rs`          | 48, 66             | `format!("{table}:{key}")`            | batch put/delete             |
| `db/batch.rs`          | 91, 116            | `format!("multimap:{key}")`           | batch multimap ops           |
| `db/config_adapter.rs` | 85, 213            | `format!("{}:{}", vault_id, version)` | config storage               |

#### Solution Options

**Option A: Pre-allocated thread-local buffer** (Recommended)

```rust
use std::fmt::Write;

thread_local! {
    static KEY_BUFFER: RefCell<String> = RefCell::new(String::with_capacity(128));
}

fn format_key(table: &str, key: &str) -> String {
    KEY_BUFFER.with(|buf| {
        let mut buffer = buf.borrow_mut();
        buffer.clear();
        write!(&mut buffer, "{table}:{key}").unwrap();
        buffer.clone()  // One allocation, but reuses buffer
    })
}
```

**Option B: Stack-allocated buffer with write!()** (Zero allocation)

```rust
use std::fmt::Write;

pub fn get<V, F, R>(&self, table: &str, key: &str, f: F) -> Result<Option<R>, DbError> {
    // Pre-allocate to exact size
    let mut namespaced_key = String::with_capacity(table.len() + key.len() + 1);
    write!(&mut namespaced_key, "{table}:{key}").unwrap();
    // ...
}
```

**Option C: Change DB API to accept tuples** (Breaking change)

```rust
// Store raw table:key in redb, format only once
pub fn get<V, F, R>(&self, table: &str, key: &str, f: F) -> Result<Option<R>, DbError> {
    let combined = (table, key);  // Pass tuple to redb layer
    // redb layer formats internally once
}
```

#### Recommended Approach

**Option B** - Inline `write!()` with pre-allocation

- Zero-copy for read operations
- No thread-local complexity
- Minimal code change
- Already partially done in `note/query.rs:326-338`

#### Estimated Impact

- **36-100 bytes saved per database operation**
- Affects: Every note/template/schema/config query and command
- ROI: **HIGHEST** - single change affects entire system

---

### 2. UUID to String Conversions

**Impact**: Every ID-based query/command operation

#### Current State

```rust
// template/query.rs:40
fn find_by_id(&self, id: Uuid) -> Result<Option<Template>, TemplateError> {
    let id_str = id.to_string();  // ❌ 36-byte allocation
    self.db.get_owned::<Template>("templates", &id_str)
}

// Similar in:
// - note/query.rs:162, 356 (find_by_id, with_archived_by_id)
// - note/command.rs:95, 116, 147 (create, delete, update)
// - template/command.rs:36, 56, 86 (create, delete, update)
```

**8 allocation sites total**

#### Locations

| File                  | Lines        | Context                             | Hot/Cold Path            |
| --------------------- | ------------ | ----------------------------------- | ------------------------ |
| `note/query.rs`       | 162, 356     | `find_by_id`, `with_archived_by_id` | 🔥 HOT - read queries    |
| `note/command.rs`     | 95, 116, 147 | `create`, `delete`, `update`        | ⚠️ WARM - write commands |
| `template/query.rs`   | 40           | `find_by_id`                        | 🔥 HOT - read query      |
| `template/command.rs` | 36, 56, 86   | `create`, `delete`, `update`        | ⚠️ WARM - write commands |

#### Solution Options

**Option A: Add UUID-native DB methods** (Recommended)

```rust
// In db/mod.rs
impl Database {
    pub fn get_by_uuid<V, F, R>(&self, table: &str, id: Uuid, f: F)
        -> Result<Option<R>, DbError>
    where
        V: Archive,
        F: FnOnce(&V::Archived) -> R,
    {
        use std::fmt::Write;
        let mut key = String::with_capacity(table.len() + 37);  // table + ":" + 36-char UUID
        write!(&mut key, "{table}:{id}").unwrap();

        // Use existing get() implementation
        self.get_raw(&key, f)
    }

    pub fn put_by_uuid<V>(&self, table: &str, id: Uuid, value: &V) -> Result<(), DbError>
    where
        V: Serialize<Strategy<Serialize, Serialize>>,
    {
        use std::fmt::Write;
        let mut key = String::with_capacity(table.len() + 37);
        write!(&mut key, "{table}:{id}").unwrap();
        self.put_raw(&key, value)
    }
}
```

**Option B: Use uuid::fmt::Hyphenated with stack buffer**

```rust
fn find_by_id(&self, id: Uuid) -> Result<Option<Template>, TemplateError> {
    use uuid::fmt::Hyphenated;
    let mut buffer = [0u8; 36];
    let id_str = Hyphenated::from_uuid(id).encode_lower(&mut buffer);
    self.db.get_owned::<Template>("templates", id_str)
}
```

**Option C: Store UUIDs as bytes (rkyv-encoded)**

```rust
// Change DB schema to store raw UUID bytes instead of strings
// Requires migration
```

#### Recommended Approach

**Option A** - Add UUID-native methods

- Clean API
- Centralizes formatting logic
- Easy to optimize later
- No breaking changes

#### Estimated Impact

- **36 bytes saved per UUID-based operation**
- Affects: All ID-based queries (high frequency)
- ROI: **HIGH** - clean solution, significant savings

---

### 3. Note Command Index Data Extraction

**Impact**: Every note update/delete operation

#### Current State

```rust
// note/command.rs:70-83
fn get_note_index_data(&self, id_str: &str)
    -> Result<Option<(String, Vec<String>)>, NoteCommandError> {
    self.db.get::<Note, _, (String, Vec<String>)>(
        "notes",
        id_str,
        |archived| {
            let path = archived.path().as_str().to_owned();  // ❌ Allocates full path
            let tags: Vec<String> = archived
                .tags()
                .iter()
                .map(|t| t.full_path().as_str().to_owned())  // ❌ Allocates N tag strings
                .collect();
            (path, tags)
        },
    )
}
```

**2 allocation sites** (but high multiplier: 1 path + N tags per operation)

#### Why It Allocates

1. Read transaction creates archived data with limited lifetime (closure scope)
2. Index updates require a separate write transaction
3. Cannot borrow archived data across transaction boundary
4. Must extract owned data before read transaction ends

#### Architectural Constraint

As noted in `TODO_NOTE_OPTIMIZATIONS.md`, this is a **fundamental constraint** of the redb transaction model:

- Attempted fix: add `WriteBatch::get()` to read within write transaction
- Result: Rust borrowing rules prevent calling `batch.multimap_remove()` while inside `batch.get()` closure

#### Solution Options

**Option A: Accept as-is** (Current recommendation in TODO_NOTE_OPTIMIZATIONS.md)

- Allocations are necessary given redb transaction model
- Impact is limited to write operations (not read-heavy workload)
- Typical cost: ~50 bytes path + ~10-20 tags × 20 bytes = ~250-450 bytes per operation

**Option B: Use Cow<str> for conditional allocation**

```rust
// For path change detection, compare before allocating
let needs_path_update = self.db.get::<Note, _, bool>(
    "notes",
    id_str,
    |archived| archived.path().as_str() != new_note.path().as_str()
)?;

if needs_path_update.unwrap_or(true) {
    // Only allocate if path actually changed
    let old_path = self.db.get::<Note, _, String>(/*...*/)?;
    // ... update path index
}
```

**Option C: Restructure index storage** (Major refactor)

- Store indexes as separate tables read in write transaction
- Complexity: HIGH
- Benefit: Eliminates ~200-400 bytes per write operation

**Option D: Batch multiple operations**

- If updating multiple notes, batch them
- Only helps for bulk operations

#### Recommended Approach

**Option A** - Accept as architectural constraint

- Allocations only occur on writes (cold path relative to reads)
- Cost is acceptable (~250-450 bytes per note mutation)
- Alternative requires major DB architecture changes

#### Estimated Impact

- **N/A** - Accept as necessary cost
- Alternative approaches have poor cost/benefit ratio
- Document as architectural constraint

---

### 4. Template Resolution HashMap Allocation

**Impact**: Every template composition operation

#### Current State

```rust
// template/query.rs:89-97
fn resolve(&self, composition: &Composition) -> Result<Template, TemplateError> {
    let base = self
        .find_by_name(&composition.base_template)
        .and_then(|opt| opt.ok_or_else(|| TemplateError::NotFound(composition.base_template.clone())))?;

    let all_templates_list = self.list()?;
    let all_templates: HashMap<String, Template> = all_templates_list
        .into_iter()
        .map(|t| (t.name().to_owned(), t))  // ❌ Clones every template name
        .collect();

    Template::compose(&base, composition, &all_templates)
}
```

**2 allocation sites**:

1. Line 89: `composition.base_template.clone()` (error path)
2. Line 96: `t.name().to_owned()` (every template in system)

#### Locations

| File                | Line | Context              | Data Allocated                 |
| ------------------- | ---- | -------------------- | ------------------------------ |
| `template/query.rs` | 89   | Error construction   | Base template name (cold path) |
| `template/query.rs` | 96   | HashMap construction | ALL template names (hot path)  |

#### Solution Options

**Option A: Use borrowed HashMap** (Recommended)

```rust
fn resolve(&self, composition: &Composition) -> Result<Template, TemplateError> {
    let base = self
        .find_by_name(&composition.base_template)
        .and_then(|opt| opt.ok_or_else(|| TemplateError::NotFound(composition.base_template.clone())))?;

    let all_templates_list = self.list()?;

    // Build HashMap with borrowed keys
    let all_templates: HashMap<&str, &Template> = all_templates_list
        .iter()
        .map(|t| (t.name().as_str(), t))  // ✅ Zero-copy
        .collect();

    Template::compose(&base, composition, &all_templates)  // ❌ Requires API change
}
```

Requires changing `Template::compose()` signature:

```rust
// template/aggregate.rs
pub fn compose(
    base: &Self,
    composition: &Composition,
    all_templates: &HashMap<&str, &Template>,  // Changed from HashMap<String, Template>
) -> Result<Self, TemplateError> {
    // ...
}
```

**Option B: Use BTreeMap with &str keys**

```rust
// Similar to Option A, but with BTreeMap for deterministic iteration
let all_templates: BTreeMap<&str, &Template> = /* ... */;
```

**Option C: Pass Vec and search linearly**

```rust
// For small template counts (<100), linear search may be faster than HashMap construction
Template::compose(&base, composition, &all_templates_list)
```

#### Recommended Approach

**Option A** - Use borrowed HashMap keys

- Zero-copy for template names
- Requires signature change to `Template::compose()`
- Clean solution with no allocations

#### Estimated Impact

- **~20-50 bytes per template name saved**
- If 20 templates: ~400-1000 bytes saved per composition
- ROI: **HIGH** - composition is a common operation

---

### 5. Numeric to String Conversions in Note Queries

**Impact**: Every task date/priority query

#### Current State

```rust
// note/query.rs:202
fn find_by_task_completed_date(&self, completed_date: i64) -> Result<Vec<Note>, NoteQueryError> {
    self.find_notes_by_task_index("tasks_by_completed_date", &completed_date.to_string())  // ❌
}

// note/query.rs:221
fn find_by_task_created_date(&self, created_date: i64) -> Result<Vec<Note>, NoteQueryError> {
    self.find_notes_by_task_index("tasks_by_created_date", &created_date.to_string())  // ❌
}

// note/query.rs:237
fn find_by_task_due_date(&self, due_date: i64) -> Result<Vec<Note>, NoteQueryError> {
    self.find_notes_by_task_index("tasks_by_due_date", &due_date.to_string())  // ❌
}

// note/query.rs:254
fn find_by_task_priority(&self, priority: u8) -> Result<Vec<Note>, NoteQueryError> {
    self.find_notes_by_task_index("tasks_by_priority", &priority.to_string())  // ❌
}
```

**4 allocation sites**

#### Locations

| File            | Line | Method                        | Index Type    | Typical Size |
| --------------- | ---- | ----------------------------- | ------------- | ------------ |
| `note/query.rs` | 202  | `find_by_task_completed_date` | i64 timestamp | 10-19 bytes  |
| `note/query.rs` | 221  | `find_by_task_created_date`   | i64 timestamp | 10-19 bytes  |
| `note/query.rs` | 237  | `find_by_task_due_date`       | i64 timestamp | 10-19 bytes  |
| `note/query.rs` | 254  | `find_by_task_priority`       | u8 priority   | 1-3 bytes    |

#### Solution Options

**Option A: Use itoa crate with stack buffer** (Recommended)

```rust
fn find_by_task_completed_date(&self, completed_date: i64) -> Result<Vec<Note>, NoteQueryError> {
    let mut buffer = itoa::Buffer::new();
    let completed_date_str = buffer.format(completed_date);  // ✅ Stack-allocated
    self.find_notes_by_task_index("tasks_by_completed_date", completed_date_str)
}
```

**Option B: Inline format with write!()**

```rust
fn find_by_task_completed_date(&self, completed_date: i64) -> Result<Vec<Note>, NoteQueryError> {
    use std::fmt::Write;
    let mut key = String::with_capacity(20);  // Max i64 is 19 chars + null
    write!(&mut key, "{completed_date}").unwrap();
    self.find_notes_by_task_index("tasks_by_completed_date", &key)
}
```

**Option C: Store indexes as native types** (Breaking change)

```rust
// Change index storage to use raw i64/u8 instead of string keys
// Requires redb table with typed keys or custom serialization
```

#### Recommended Approach

**Option A** - Use `itoa` crate

- Zero-allocation integer formatting
- Fast and well-tested
- Minimal code change
- `itoa` already in dependency tree (via `serde_json`)

#### Estimated Impact

- **10-19 bytes saved per date query**
- **1-3 bytes saved per priority query**
- ROI: **MEDIUM-HIGH** - simple fix, moderate frequency

---

## 🟠 P1: HIGH PRIORITY (Write Path + API Design)

### 6. Constructor API Design - String vs &str

**Impact**: Forces caller allocations, poor ergonomics

#### Current State - Multiple constructors force `String` allocation

**Schema constructors:**

```rust
// schema/aggregate.rs:532
impl SchemaName {
    pub fn new(name: String) -> Result<Self, SchemaError> {  // ❌
        // ...
    }
}

// schema/aggregate.rs:215
impl TryFrom<String> for SchemaName {  // ❌
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

// schema/property.rs:266
impl PropertyName {
    pub fn new(name: String) -> Result<Self, SchemaError> {  // ❌
        // ...
    }
}

// schema/property.rs:119
impl TryFrom<String> for PropertyName {  // ❌
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
```

**Event constructors:**

```rust
// template/events.rs:44
impl TemplateCreated {
    pub fn new(id: Uuid, name: String, timestamp: i64) -> Self {  // ❌
        Self { id, name, timestamp }
    }
}

// schema/events.rs:110
impl SchemaCreated {
    pub fn new(id: Uuid, name: String, timestamp: i64) -> Self {  // ❌
        Self { id, name, timestamp }
    }
}

// config/events.rs:69
impl ConfigUpdated {
    pub fn new(source: String, timestamp: i64) -> Self {  // ❌
        Self { source, timestamp }
    }
}
```

**Other constructors:**

```rust
// note/aggregate.rs:396
impl TryFrom<String> for NotePath {  // ❌
    fn try_from(value: String) -> Result<Self, Self::Error> {
        // ...
    }
}

// schema/property_spec.rs:394
impl DateSpec {
    pub fn try_new(format: String) -> Result<Self, SchemaError> {  // ❌
        // ...
    }
}
```

**14 allocation-forcing APIs total**

#### Rust Idiom Violation

Per `AGENTS.md` - Rust idioms section:

> Prefer borrowed arguments in APIs: take `&str`, `&Path`, slices, and `&T` instead of `String`/`PathBuf`/owned types unless ownership is required.

Standard library examples:

```rust
// ✅ std::path::Path::new()
pub fn new<S: AsRef<OsStr> + ?Sized>(s: &S) -> &Path

// ✅ String::from()
pub fn from(s: &str) -> String

// ❌ Would be weird
pub fn from(s: String) -> String
```

#### Solution - Change All to Accept `&str`

**Schema constructors:**

```rust
impl SchemaName {
    pub fn new(name: &str) -> Result<Self, SchemaError> {  // ✅
        // Internal validation, then allocate
        // ...
        Ok(Self(name.into()))
    }
}

impl TryFrom<&str> for SchemaName {  // ✅
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

// Keep String variant for convenience
impl TryFrom<String> for SchemaName {
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}
```

**Event constructors:**

```rust
impl TemplateCreated {
    pub fn new(id: Uuid, name: &str, timestamp: i64) -> Self {  // ✅
        Self {
            id,
            name: name.into(),  // Allocate here, not at call site
            timestamp,
        }
    }
}
```

#### Locations to Change

| File                      | Line | Current Signature           | New Signature                     |
| ------------------------- | ---- | --------------------------- | --------------------------------- |
| `schema/aggregate.rs`     | 532  | `new(name: String)`         | `new(name: &str)`                 |
| `schema/aggregate.rs`     | 215  | `TryFrom<String>`           | `TryFrom<&str>` (keep String too) |
| `schema/property.rs`      | 266  | `new(name: String)`         | `new(name: &str)`                 |
| `schema/property.rs`      | 119  | `TryFrom<String>`           | `TryFrom<&str>` (keep String too) |
| `schema/property_spec.rs` | 394  | `try_new(format: String)`   | `try_new(format: &str)`           |
| `note/aggregate.rs`       | 396  | `TryFrom<String>`           | `TryFrom<&str>` (keep String too) |
| `template/events.rs`      | 44   | `new(id, name: String, ts)` | `new(id, name: &str, ts)`         |
| `schema/events.rs`        | 110  | `new(id, name: String, ts)` | `new(id, name: &str, ts)`         |
| `config/events.rs`        | 69   | `new(source: String, ts)`   | `new(source: &str, ts)`           |

Plus 5 more in config/ modules

#### Call Site Updates Required

Must update all call sites that currently do:

```rust
SomeName::new(my_string.to_owned())  // ❌ Before
SomeName::new(&my_string)             // ✅ After
```

Search for patterns:

- `.to_owned()` before constructor calls
- `.clone()` before constructor calls
- Unnecessary `format!()` to build strings for constructors

#### Estimated Impact

- **Reduces allocations at call sites** (DRY principle)
- **Better ergonomics** (idiomatic Rust)
- Effort: 2-3 hours (need to update call sites)
- ROI: **MEDIUM-HIGH** - improves API quality, moderate perf gain

---

### 7. Template Command Index Operations

**Impact**: Every template create/update/delete

#### Current State

```rust
// template/command.rs:35-45
fn create(&self, template: &Template) -> Result<(), TemplateError> {
    let id_str = template.id().to_string();   // ❌ 36 bytes
    let name = template.name().to_owned();    // ❌ ~20-50 bytes

    self.db.put("templates", &id_str, template)?;
    self.db.multimap_insert("template_name_to_id", &name, &id_str)?;
    Ok(())
}

// template/command.rs:55-68
fn delete(&self, id: Uuid) -> Result<(), TemplateError> {
    let id_str = id.to_string();  // ❌

    if let Some(template) = self.db.get_owned::<Template>("templates", &id_str)? {
        let name = template.name().to_owned();  // ❌
        self.db.multimap_remove("template_name_to_id", &name, &id_str)?;
        self.db.delete("templates", &id_str)?;
    }
    Ok(())
}

// template/command.rs:85-102
fn update(&self, template: &Template) -> Result<(), TemplateError> {
    let id_str = template.id().to_string();  // ❌
    let new_name = template.name().to_owned();  // ❌

    if let Some(old_template) = self.db.get_owned::<Template>("templates", &id_str)? {
        let old_name = old_template.name().to_owned();  // ❌

        if old_name != new_name {
            self.db.multimap_remove("template_name_to_id", &old_name, &id_str)?;
            self.db.multimap_insert("template_name_to_id", &new_name, &id_str)?;
        }
    }

    self.db.put("templates", &id_str, template)?;
    Ok(())
}
```

**7 allocation sites in template commands**

#### Dependencies

This issue is **dependent on fixes #2 (UUID methods) and #1 (DB key formatting)**.

Once those are fixed:

```rust
fn create(&self, template: &Template) -> Result<(), TemplateError> {
    // ✅ Zero-copy with get_by_uuid() and optimized DB layer
    self.db.put_by_uuid("templates", template.id(), template)?;
    self.db.multimap_insert_with_uuid("template_name_to_id", template.name().as_str(), template.id())?;
    Ok(())
}
```

#### Recommended Approach

**Defer until P0 fixes complete** - these allocations will be eliminated by dependency fixes

---

## 🟡 P2: MEDIUM PRIORITY (Optimization Opportunities)

### 8. Error Construction with format!()

**Impact**: Error paths only (cold path)

#### Current State

89 locations use `format!()` for error messages:

```rust
// Example from schema/property_spec.rs:335
Err(SchemaError::ValidationFailed(format!(
    "Value {actual:?} has wrong type, expected {expected}"
)))
```

#### Analysis

- **Status**: ✅ ACCEPTABLE
- Error paths are cold (only execute on failure)
- Allocation cost is acceptable in error cases
- `format!()` provides good error messages
- Alternative (`Box<str>`) provides minimal benefit

#### Recommendation

**Accept as-is** - error paths are cold, cost is acceptable

---

### 9. Multimap Value Iteration

**Impact**: Scanning multimap entries

#### Current State

```rust
// db/mod.rs:442
pub fn multimap_range(&self, key: &str) -> Result<Vec<String>, DbError> {
    // ...
    for value in values {
        let val: &str = value.value();
        result.push(val.to_owned());  // ❌ Allocates each value
    }
    Ok(result)
}
```

#### Analysis

This is **necessary allocation** - must return owned data from transaction scope.
Alternative would be to pass callback (like `get()`), but that complicates API.

#### Recommendation

**Accept as-is** - necessary given transaction lifetime constraints

---

## 🟢 P3: LOW PRIORITY (Accept or Defer)

### 10. Test Code Allocations

**Impact**: Test code only

#### Examples

```rust
// schema/query.rs:92
test_schema.name.to_owned()

// template/command.rs:133
"default".to_owned()
```

**Status**: ✅ ACCEPTABLE - test code is not production

---

### 11. Error Path in Template Resolution

**Impact**: Error path only

#### Current State

```rust
// template/query.rs:89
.ok_or_else(|| TemplateError::NotFound(composition.base_template.clone()))?;
```

**Status**: ✅ ACCEPTABLE - error path (cold)

---

## 📊 Implementation Roadmap

### Phase 1: Database Infrastructure (P0) - Highest ROI

**Estimated Time**: 4-6 hours
**Expected Reduction**: 30-40 allocations per note operation

1. ✅ **Task 2**: Optimize database key formatting
   - Files: `db/mod.rs`, `db/batch.rs`
   - Change: Use `write!()` with pre-allocated buffer instead of `format!()`
   - Impact: 100% of DB operations
   - Validation: Run benchmarks before/after

2. ✅ **Task 3**: Add UUID-native DB methods
   - Files: `db/mod.rs`, `db/batch.rs`
   - Add: `get_by_uuid()`, `put_by_uuid()`, `delete_by_uuid()`, `multimap_*_uuid()` methods
   - Update: All query.rs and command.rs to use new methods
   - Impact: All ID-based operations
   - Validation: All tests pass

3. ✅ **Task 5**: Fix template resolution HashMap
   - File: `template/query.rs:89-97`, `template/aggregate.rs`
   - Change: Use `HashMap<&str, &Template>` with borrowed keys
   - Impact: Template composition
   - Validation: Template tests pass

### Phase 2: Query Hot Paths (P0/P1)

**Estimated Time**: 2-3 hours
**Expected Reduction**: 10-20 allocations per query

4. ✅ **Task 6**: Fix numeric to string conversions in note queries
   - File: `note/query.rs:202, 221, 237, 254`
   - Change: Use `itoa` crate with stack buffer
   - Impact: Task date/priority queries
   - Validation: Note query tests pass

5. ✅ **Task 4**: Document note command allocation constraint
   - File: `note/command.rs:70-83`
   - Action: Add rustdoc comment explaining why allocations are necessary
   - Impact: Documentation only
   - Validation: cargo doc

### Phase 3: API Surface Cleanup (P1)

**Estimated Time**: 3-4 hours
**Expected Reduction**: Improved ergonomics + reduced caller allocations

6. ✅ **Task 7**: Change constructor APIs from String to &str
   - Files: All aggregate.rs, events.rs, property.rs files
   - Change: `new(name: String)` → `new(name: &str)`
   - Change: `TryFrom<String>` → `TryFrom<&str>` (keep String variant)
   - Impact: API ergonomics, reduced caller allocations
   - Validation: Full test suite

### Phase 4: Verification (P0)

**Estimated Time**: 1 hour

7. ✅ **Task 8**: Run full verification suite
   - Run: `mise run verify` (fmt + lint + tests + ADR)
   - Run: `mise run test:bench` (benchmarks)
   - Compare: Before/after benchmark results
   - Validate: No regressions, performance improvements visible

---

## 📈 Success Metrics

### Quantitative Goals

- [ ] Database layer: 30-40 fewer allocations per note operation
- [ ] Query layer: 10-20 fewer allocations per query
- [ ] Template composition: ~400-1000 bytes saved per composition
- [ ] Overall: 50-80% reduction in hot path allocations

### Qualitative Goals

- [ ] Idiomatic Rust APIs (`&str` parameters)
- [ ] Clear documentation of architectural constraints
- [ ] Zero regressions in functionality
- [ ] Maintained or improved benchmark performance

---

## 🔍 Validation Strategy

### Before Implementation

1. Run baseline benchmarks: `mise run test:bench`
2. Save results to `benchmarks_before.txt`
3. Note key metrics: note parse time, query time, command time

### During Implementation

1. Run tests after each phase: `mise run test`
2. Check for regressions: `mise run verify`
3. Profile if needed: `cargo flamegraph`

### After Implementation

1. Run full verification: `mise run verify`
2. Run benchmarks: `mise run test:bench`
3. Compare against baseline
4. Document improvements in commit message

---

## 🚧 Known Constraints (Accept As-Is)

### 1. Note Command Index Data Extraction (command.rs:70-83)

**Constraint**: redb transaction lifetime prevents zero-copy across read → write boundary
**Decision**: Accept 250-450 bytes allocation per note mutation
**Rationale**: Write operations are cold path relative to reads; alternative requires major architecture change

### 2. Error Path Allocations (89 locations)

**Constraint**: Error messages need owned strings for propagation
**Decision**: Accept `format!()` in error construction
**Rationale**: Error paths are cold; good error messages more valuable than allocation savings

### 3. Multimap Iteration (db/mod.rs:442)

**Constraint**: Must return owned data from transaction scope
**Decision**: Accept `.to_owned()` for each multimap value
**Rationale**: Necessary given transaction lifetime constraints

---

## 🎯 Priority Summary

| Priority          | Tasks                  | Estimated Time | Expected Impact             |
| ----------------- | ---------------------- | -------------- | --------------------------- |
| **P0** (Critical) | Tasks 2, 3, 4, 5, 6, 8 | 7-10 hours     | 50-80% allocation reduction |
| **P1** (High)     | Task 7                 | 3-4 hours      | API quality + moderate perf |
| **P2** (Medium)   | None (all accepted)    | 0 hours        | N/A                         |
| **P3** (Low)      | None (all accepted)    | 0 hours        | N/A                         |

**Total estimated effort**: 10-14 hours for all P0-P1 tasks

---

## 📋 Related Documents

- `TODO_NOTE_OPTIMIZATIONS.md` - Note module specific optimizations (partially complete)
- `AGENTS.md` - Rust idioms and coding standards
- `docs/refs/rust/idioms.md` - Deeper Rust idiom rationale
- `docs/adr/` - Architectural decision records

---

**Next Steps**:

1. Review and approve this plan
2. Begin Phase 1 (Database Infrastructure)
3. Validate after each phase
4. Update this document with actual results
5. Create ADR if architectural decisions made
