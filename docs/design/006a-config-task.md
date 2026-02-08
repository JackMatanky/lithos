---
feature: Task Configuration Schema
status: Draft
author: Jack Matanky (with AI assistance)
ticket: TBD
date_created: 2026-02-08
tags: [config, task, schema, validation]
---

# Tech Spec: Task Configuration Schema

## 1. Problem Space (The "Why")

### 1.1 Context & Background

The Lithos task system requires user-configurable metadata schemas to support rich task management workflows. Currently, there is no formal definition of how task metadata fields, status symbols, and promotion tags are configured.

**Current Gap**: Users cannot define:
- Custom metadata fields (priority, project, due dates)
- Custom status symbols beyond `[x]`, `[ ]`, `[-]`
- Which tags trigger task promotion (`#task`, `#todo`, `#action-item`)
- Field validation rules (enum values, numeric bounds, date formats)

**Why Config Context**: Task configuration is **cross-cutting infrastructure** (like paths, logging, frontmatter config). The Note context consumes this configuration to:
- Validate inline task metadata during parsing
- Map status symbols to semantic names
- Determine which checkboxes are promoted to Task entities

**Related Decisions**:
- [ADR 001: Configuration System](../adr/001-configuration-system.md)
- [Epic 11: Query Service](../../_bmad-output/planning-artifacts/epics/epic-11-query-service-knowledge-graph-mvp-core.md)

### 1.2 Goals & Non-Goals

**Goals**:
1. Define a **user-editable TOML schema** for task metadata fields
2. Support **type-safe field validation** (string/integer/float/boolean/enum/date)
3. Enable **custom status symbols** mapped to semantic names
4. Allow **configurable task tags** for promotion rules
5. Provide **indexed field selection** for query optimization
6. **Zero breaking changes**: Default config matches existing behavior

**Non-Goals**:
- Implementing task parsing (Note context responsibility)
- Query service integration (Epic 11)
- Template rendering (Template context responsibility)
- Recurring tasks, dependencies, or time tracking

### 1.3 Constraints (The Hard Limits)

**Architectural**:
- Config context is **cross-cutting infrastructure** (no business logic)
- Validation logic stays **internal to construction** (newtype pattern)
- Config types are **sync-first** (no async validation)
- No imports from Note/Schema/Template contexts (only infrastructure)

**Compatibility**:
- Default config must match current checkbox behavior (`x/ /-` symbols)
- Invalid config files must produce **structured errors** with field names
- Config changes require vault reindexing (no hot-reload in MVP)

**Performance**:
- Config loaded once at startup (no repeated parsing)
- Field specs compiled during config construction (validation cached)
- Target: Config load + validation < 50ms for typical vaults

**Data Integrity**:
- Invalid field specs (empty enum, min > max) rejected at config load
- Unknown fields in user config trigger warnings (forward compatibility)

## 2. Guide-Level Explanation (The "What")

### 2.1 User/Dev Experience

#### User Perspective: Defining Task Schema

**Step 1: Create Vault Config**

Users create `.lithos/lithos.toml`:

```toml
[task]
enabled = true

# Task promotion tags (which tags indicate "this is a task")
task_tags = ["#task", "#todo", "#action-item"]

# Status symbol mapping
[task.status]
complete = "x"
incomplete = " "
cancelled = "-"
in_progress = ">"
waiting = "?"

# Custom metadata fields
[task.fields.priority]
type = "integer"
keyword = "priority"
min = 0
max = 10

[task.fields.project_name]
type = "string"
keyword = "project"
pattern = "^[a-z_]+$"

[task.fields.task_type]
type = "enum"
keyword = "type"
values = ["action_item", "reminder", "meeting", "research"]

[task.fields.due_date]
type = "date"
keyword = "due"
format = "%Y-%m-%d"

# Query optimization
[task.indexing]
indexed_fields = ["due_date", "priority", "project_name", "task_type"]
```

**Step 2: Config Validation**

When Lithos loads the vault:

```bash
$ lithos index --vault my-vault/

✓ Config loaded successfully
  - 4 task fields defined
  - 5 status symbols mapped
  - 3 task tags configured
  - 4 indexed fields
```

**Validation errors are caught early:**

```bash
$ lithos index --vault my-vault/

✗ Config validation failed:
  - task.fields.priority: min (0) cannot be greater than max (-1)
  - task.fields.task_type: enum values cannot be empty
  - task.status: duplicate symbol 'x' mapped to both 'complete' and 'done'
```

#### Developer Perspective: Using TaskConfig

**Loading Config**

```rust
use lithos_core::config::Config;
use lithos_core::config::task::TaskConfig;

// Load and validate vault config
let config = Config::load_vault_config(vault_path)?;

// Access task config (cross-cutting infrastructure)
let task_config: &TaskConfig = &config.task;

assert_eq!(task_config.task_tags().len(), 3);
assert!(task_config.has_task_tag("#task"));
```

**Validating Task Metadata (Note Context Uses This)**

```rust
use serde_json::json;

// Note context calls this during task parsing
let priority_json = json!(5);
let field_value = task_config.parse_field_value("priority", &priority_json)?;

assert_eq!(field_value.as_number(), Some(5.0));

// Validation catches out-of-bounds
let invalid = json!(15);
let err = task_config.parse_field_value("priority", &invalid);
assert!(err.is_err()); // Error: Value 15 exceeds max 10
```

**Status Symbol Mapping**

```rust
use lithos_core::config::task::{StatusSymbol, StatusName};

let symbol = StatusSymbol('x');
let name = task_config.status().name_for_symbol(symbol);

assert_eq!(name, Some(&StatusName::new("complete")?));
```

### 2.2 Mental Model

**Three-Layer Model**:

```
┌─────────────────────────────────────────────────┐
│ User Config (TOML)                              │
│ - Defines field schema, status mapping, tags   │
│ - May be invalid until validated               │
└─────────────────────────────────────────────────┘
                    │
                    │ Raw → Domain Conversion
                    ▼
┌─────────────────────────────────────────────────┐
│ TaskConfig (Validated Domain)                   │
│ - Invariants guaranteed by construction         │
│ - Used by Note context for validation           │
│ - Cross-cutting infrastructure                  │
└─────────────────────────────────────────────────┘
                    │
                    │ Consumed by Note Context
                    ▼
┌─────────────────────────────────────────────────┐
│ Task Parsing & Validation                       │
│ - Validates inline [key:: value] metadata       │
│ - Maps status symbols to semantic names         │
│ - Checks task tags for promotion                │
└─────────────────────────────────────────────────┘
```

**Key Concepts**:

1. **TaskTag = Promotion Vocabulary**: Config defines which tags mean "this is a task"
2. **TaskFieldSpec = Validation Rules**: Config defines what metadata is allowed
3. **CheckboxStatus = Symbol Mapping**: Config maps `[x]` to semantic "complete"
4. **Validation is Private**: Newtype pattern means validation happens at construction

**Think of it like**:
- **TaskConfig** = JSON Schema for task metadata
- **TaskTag** = Allowed promotion keywords
- **CheckboxStatus** = Character encoding map (symbol ↔ meaning)

## 3. Detailed Design (The "How")

### 3.1 System Architecture

```mermaid
graph TB
    subgraph "Config Files (Input)"
        GlobalTOML[global.toml]
        VaultTOML[vault.toml]
    end

    subgraph "Config Context (Domain)"
        RawTaskConfig[RawTaskConfig<br/>serde-friendly]
        TaskConfig[TaskConfig<br/>validated]

        TaskTag[TaskTag<br/>newtype]
        TaskFieldSpec[TaskFieldSpec<br/>validation rules]
        CheckboxStatus[CheckboxStatus<br/>symbol mapping]

        RawTaskConfig --> TaskConfig
        TaskConfig --> TaskTag
        TaskConfig --> TaskFieldSpec
        TaskConfig --> CheckboxStatus
    end

    subgraph "Note Context (Consumer)"
        TaskParsing[Task Parsing]
        MetadataValidation[Metadata Validation]
    end

    GlobalTOML --> RawTaskConfig
    VaultTOML --> RawTaskConfig

    TaskConfig -.read-only.-> TaskParsing
    TaskConfig -.validates via.-> MetadataValidation

    style TaskConfig fill:#e1f5ff
    style TaskParsing fill:#fff4e1
```

### 3.2 Data Models

#### `TaskTag` (Domain)

- **Purpose**: Validated tag that triggers task promotion (e.g., `#task`, `#todo`)
- **Backing**: `Box<str>`
- **Rules**: Must start with `#`, non-empty, ASCII alphanumeric + `_-`
- **Notes**: Allocated once during config load

```rust
pub struct TaskTag(Box<str>);

impl TaskTag {
    fn try_from(s: &str) -> Result<Self, ConfigError> {
        if !s.starts_with('#') {
            return Err(ConfigError::InvalidTaskTag("Must start with '#'"));
        }
        // Additional validation...
        Ok(TaskTag(s.into()))
    }

    pub fn as_ref(&self) -> &str {
        &self.0
    }
}
```

#### `StatusSymbol` (Domain)

- **Purpose**: Single-character checkbox symbol as written in markdown `[<char>]`
- **Backing**: `char`
- **Rules**: Printable ASCII, non-whitespace
- **Notes**: Copy type

```rust
pub struct StatusSymbol(char);

impl StatusSymbol {
    fn try_from(c: char) -> Result<Self, ConfigError> {
        if !c.is_ascii() || c.is_whitespace() {
            return Err(ConfigError::InvalidStatusSymbol(c));
        }
        Ok(StatusSymbol(c))
    }
}
```

#### `StatusName` (Domain)

- **Purpose**: Semantic status identifier used throughout system (e.g., "complete", "in_progress")
- **Backing**: `Box<str>`
- **Rules**: Non-empty, ASCII alphanumeric + `_`, <= 32 chars
- **Notes**: Used in queries, templates, indexing (stable across vaults)

```rust
pub struct StatusName(Box<str>);

impl StatusName {
    fn try_from(s: &str) -> Result<Self, ConfigError> {
        if s.is_empty() || s.len() > 32 {
            return Err(ConfigError::InvalidStatusName("Must be 1-32 chars"));
        }
        // Additional validation...
        Ok(StatusName(s.into()))
    }

    pub fn as_ref(&self) -> &str {
        &self.0
    }
}
```

#### `TaskFieldSpec` (Domain)

- **Purpose**: Validated field specification with type constraints
- **Key rules**: Enforced at construction (min <= max, non-empty enum values, valid regex)
- **Important notes**: Contains compiled regex (cached at config load)
- **Shape**:

```rust
#[derive(Debug, Clone)]
pub enum TaskFieldSpec {
    String {
        keyword: Box<str>,
        pattern: Option<Arc<regex::Regex>>,
    },
    Integer {
        keyword: Box<str>,
        min: Option<i64>,
        max: Option<i64>,
    },
    Float {
        keyword: Box<str>,
        min: Option<f64>,
        max: Option<f64>,
    },
    Boolean {
        keyword: Box<str>,
    },
    Enum {
        keyword: Box<str>,
        values: Vec<Box<str>>,
    },
    Date {
        keyword: Box<str>,
        format: Box<str>,
    },
}
```

#### `CheckboxStatus` (Domain)

- **Purpose**: Bidirectional mapping between status symbols and semantic names
- **Key rules**: No duplicate symbols or names; at least one mapping
- **Shape**:

```rust
#[derive(Debug, Clone)]
pub struct CheckboxStatus {
    by_name: HashMap<StatusName, StatusSymbol>,
    by_symbol: HashMap<StatusSymbol, StatusName>,
}

impl CheckboxStatus {
    pub fn symbol_for_name(&self, name: &StatusName) -> Option<StatusSymbol> {
        self.by_name.get(name).copied()
    }

    pub fn name_for_symbol(&self, symbol: StatusSymbol) -> Option<&StatusName> {
        self.by_symbol.get(&symbol)
    }
}
```

#### `TaskConfig` (Domain, Aggregate)

- **Purpose**: Validated task configuration aggregate
- **Key rules**: All fields validated at construction; no public mutation
- **Important notes**: Cross-cutting infrastructure consumed by Note context
- **Shape**:

```rust
#[derive(Debug, Clone)]
pub struct TaskConfig {
    enabled: bool,
    task_tags: Vec<TaskTag>,
    status: CheckboxStatus,
    fields: HashMap<Box<str>, TaskFieldSpec>,
    indexed_fields: Vec<Box<str>>,
}
```

#### `RawTaskConfig` (Raw/Input)

- **Purpose**: Serde-friendly input shape before validation
- **Notes**: May be invalid; compiled into `TaskConfig`

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct RawTaskConfig {
    pub enabled: Option<bool>,
    pub task_tags: Option<Vec<String>>,
    pub status: Option<HashMap<String, char>>,
    pub fields: Option<HashMap<String, RawTaskFieldSpec>>,
    pub indexing: Option<RawIndexingConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RawTaskFieldSpec {
    String {
        keyword: String,
        pattern: Option<String>,
    },
    Integer {
        keyword: String,
        min: Option<i64>,
        max: Option<i64>,
    },
    Float {
        keyword: String,
        min: Option<f64>,
        max: Option<f64>,
    },
    Boolean {
        keyword: String,
    },
    Enum {
        keyword: String,
        values: Vec<String>,
    },
    Date {
        keyword: String,
        format: String,
    },
}
```

### 3.3 Component & Interface Specifications

#### Component: `TaskConfig`

- **Responsibility**: Validated task configuration aggregate; provides field validation and status mapping for Note context
- **Public Interface**:
  - `TaskConfig::from_raw(raw: RawTaskConfig) -> Result<Self, ConfigError>`
    - _Behavior_: Validates raw config, compiles regex patterns, builds bidirectional status map
    - _Errors_: Invalid field specs, duplicate mappings, empty required fields
  - `task_tags(&self) -> &[TaskTag]`
    - _Behavior_: Returns configured task tags
  - `has_task_tag(&self, text: &str) -> bool`
    - _Behavior_: Checks if text contains any configured task tag
  - `status(&self) -> &CheckboxStatus`
    - _Behavior_: Returns status symbol ↔ name mapping
  - `parse_field_value(&self, field_name: &str, json: &serde_json::Value) -> Result<note::value::FieldValue, ConfigError>`
    - _Behavior_: Validates JSON value against field spec and converts to note-owned `FieldValue`
    - _Errors_: Unknown field, type mismatch, out of bounds, pattern mismatch
  - `indexed_fields(&self) -> &[Box<str>]`
    - _Behavior_: Returns fields marked for indexing
- **State/Invariants**:
  - All regex patterns successfully compiled
  - Status mapping is bijective (1:1)
  - Indexed fields reference valid field names
  - Min <= max for numeric fields
  - Enum values non-empty

#### Component: `TaskFieldSpec`

- **Responsibility**: Validated field specification with compiled constraints
- **Public Interface** (internal to config context):
  - `validate_raw_value(&self, value: &serde_json::Value) -> Result<(), ConfigError>`
    - _Behavior_: Validates JSON value against type and constraints (private helper)
    - _Errors_: Type mismatch, out of bounds, pattern mismatch, invalid enum value
- **State/Invariants**:
  - Keyword is non-empty
  - Regex pattern (if present) is compiled
  - Numeric bounds satisfy min <= max
  - Enum values are non-empty

#### Component: `CheckboxStatus`

- **Responsibility**: Bidirectional mapping between symbols and semantic names
- **Public Interface**:
  - `CheckboxStatus::from_raw(raw: HashMap<String, char>) -> Result<Self, ConfigError>`
    - _Behavior_: Builds bidirectional map, validates uniqueness
    - _Errors_: Duplicate symbols, duplicate names, invalid symbol/name format
  - `symbol_for_name(&self, name: &StatusName) -> Option<StatusSymbol>`
    - _Behavior_: Lookup symbol by semantic name
  - `name_for_symbol(&self, symbol: StatusSymbol) -> Option<&StatusName>`
    - _Behavior_: Lookup semantic name by symbol
- **State/Invariants**:
  - Bijective mapping (no duplicates)
  - At least one mapping exists

### 3.4 Integration & Data Flow

#### Config Loading Flow

```mermaid
sequenceDiagram
    participant User as User Config File
    participant Serde as Serde Deserializer
    participant Raw as RawTaskConfig
    participant Domain as TaskConfig
    participant Note as Note Context

    User->>Serde: Read lithos.toml
    Serde->>Raw: Deserialize
    Raw->>Raw: May be invalid
    Raw->>Domain: TaskConfig::from_raw()
    Domain->>Domain: Validate fields
    Domain->>Domain: Compile regex
    Domain->>Domain: Build status map
    alt Validation Success
        Domain->>Note: Provide validated config
    else Validation Failure
        Domain-->>User: ConfigError with details
    end
```

#### Field Validation Flow (Note Context → Config Context)

```mermaid
sequenceDiagram
    participant Parser as Note Parser
    participant Task as Task Builder
    participant Config as TaskConfig
    participant Spec as TaskFieldSpec

    Parser->>Task: Parse [priority:: 5]
    Task->>Config: parse_field_value("priority", json!(5))
    Config->>Config: Lookup field spec
    Config->>Spec: validate_raw_value(json!(5))
    Spec->>Spec: Check type (integer)
    Spec->>Spec: Check bounds (0 <= 5 <= 10)
    Spec-->>Config: Valid
    Config->>Config: Convert to FieldValue
    Config-->>Task: Ok(FieldValue::Number(5.0))
    Task->>Task: Store in TaskMetadata
```

#### Dependencies

- **Serde**: TOML/YAML deserialization (`serde`, `toml`, `serde_yaml`)
- **Regex**: Pattern validation (`regex` crate, compiled at config load)
- **Chrono**: Date format validation (`chrono::format::strftime`)
- **Note Context**: Exports `FieldValue` type (config converts to this)

### 3.5 Core Logic & Algorithms

#### Algorithm: Field Value Validation

```rust
impl TaskFieldSpec {
    fn validate_raw_value(&self, value: &serde_json::Value) -> Result<(), ConfigError> {
        match self {
            TaskFieldSpec::Integer { keyword, min, max } => {
                let n = value.as_i64()
                    .ok_or(ConfigError::TypeMismatch {
                        field: keyword.clone(),
                        expected: "integer"
                    })?;

                if let Some(min) = min {
                    if n < *min {
                        return Err(ConfigError::OutOfBounds {
                            field: keyword.clone(),
                            value: n,
                            min: Some(*min),
                            max: *max,
                        });
                    }
                }

                if let Some(max) = max {
                    if n > *max {
                        return Err(ConfigError::OutOfBounds {
                            field: keyword.clone(),
                            value: n,
                            min: *min,
                            max: Some(*max),
                        });
                    }
                }

                Ok(())
            }

            TaskFieldSpec::String { keyword, pattern } => {
                let s = value.as_str()
                    .ok_or(ConfigError::TypeMismatch {
                        field: keyword.clone(),
                        expected: "string",
                    })?;

                if let Some(re) = pattern {
                    if !re.is_match(s) {
                        return Err(ConfigError::PatternMismatch {
                            field: keyword.clone(),
                            value: s.to_owned(),
                            pattern: re.as_str().to_owned(),
                        });
                    }
                }

                Ok(())
            }

            TaskFieldSpec::Enum { keyword, values } => {
                let s = value.as_str()
                    .ok_or(ConfigError::TypeMismatch {
                        field: keyword.clone(),
                        expected: "string (enum)",
                    })?;

                if !values.iter().any(|v| v.as_ref() == s) {
                    return Err(ConfigError::InvalidEnumValue {
                        field: keyword.clone(),
                        value: s.to_owned(),
                        allowed: values.iter().map(|v| v.to_string()).collect(),
                    });
                }

                Ok(())
            }

            // ... other variants
        }
    }
}
```

#### Algorithm: Status Mapping Construction

```rust
impl CheckboxStatus {
    fn from_raw(raw: HashMap<String, char>) -> Result<Self, ConfigError> {
        let mut by_name = HashMap::new();
        let mut by_symbol = HashMap::new();

        for (name_str, symbol_char) in raw {
            let name = StatusName::try_from(name_str.as_str())?;
            let symbol = StatusSymbol::try_from(symbol_char)?;

            // Check for duplicate names
            if by_name.contains_key(&name) {
                return Err(ConfigError::DuplicateStatusName(name_str));
            }

            // Check for duplicate symbols
            if by_symbol.contains_key(&symbol) {
                return Err(ConfigError::DuplicateStatusSymbol(symbol_char));
            }

            by_name.insert(name.clone(), symbol);
            by_symbol.insert(symbol, name);
        }

        // Require at least one mapping
        if by_name.is_empty() {
            return Err(ConfigError::EmptyStatusMapping);
        }

        Ok(CheckboxStatus { by_name, by_symbol })
    }
}
```

#### Algorithm: Default Config Generation

```rust
impl Default for TaskConfig {
    fn default() -> Self {
        let task_tags = vec![
            TaskTag::try_from("#task").unwrap(),
        ];

        let status = {
            let mut map = HashMap::new();
            map.insert("complete".to_owned(), 'x');
            map.insert("incomplete".to_owned(), ' ');
            map.insert("cancelled".to_owned(), '-');
            CheckboxStatus::from_raw(map).unwrap()
        };

        TaskConfig {
            enabled: false,
            task_tags,
            status,
            fields: HashMap::new(),
            indexed_fields: vec![],
        }
    }
}
```

## 4. Alternatives & Decisions (The "Divergence")

### 4.1 Tactical Decisions

#### Decision: TaskConfig in Config Context (Not Note Context)

- **Context**: Task configuration could live in note context (where it's consumed)
- **Choice**: Place in config context as cross-cutting infrastructure
- **Alternatives Considered**:
  - _Note context_: Config owned by note. **Rejected** - breaks config layering (global vs vault)
  - _Separate task context_: Dedicated bounded context for tasks. **Rejected** - task is note semantics, config is vocabulary
- **Rationale**: Config is cross-cutting infrastructure (like paths, logging). Note context imports config (allowed), but config never imports note (enforced).

#### Decision: Validation is Private (Newtype Pattern)

- **Context**: Should validation be exposed as public API?
- **Choice**: Validation happens at construction; no public `validate()` methods
- **Alternatives Considered**:
  - _Public validation_: Expose `TaskFieldSpec::validate()`. **Rejected** - violates type-driven design (makes invalid states representable)
  - _Builder pattern_: `TaskConfigBuilder` with staged validation. **Rejected** - unnecessary complexity for config
- **Rationale**: Newtype pattern means construction enforces invariants. Public API operates on validated types only.

#### Decision: Regex Compiled at Config Load (Not Lazy)

- **Context**: Regex compilation is expensive; when to compile?
- **Choice**: Compile all regex patterns during `TaskConfig::from_raw()` (eager)
- **Alternatives Considered**:
  - _Lazy compilation_: Compile on first use. **Rejected** - defers errors, adds runtime overhead
  - _Pre-compiled crate_: Generate Rust code from config. **Rejected** - overkill for user config
- **Rationale**: Config loaded once at startup; eager compilation catches invalid patterns early.

#### Decision: StatusSymbol is char (Not String)

- **Context**: Checkbox symbols are single characters in markdown `[<char>]`
- **Choice**: `StatusSymbol(char)` newtype
- **Alternatives Considered**:
  - _String_: Allow multi-char symbols like `[done]`. **Rejected** - non-standard markdown
  - _Enum_: Hardcoded `StatusSymbol::Complete`. **Rejected** - not user-configurable
- **Rationale**: Aligns with markdown checkbox syntax; keeps domain model lean.

#### Decision: Indexed Fields as Vec (Not Set)

- **Context**: Should indexed fields preserve order?
- **Choice**: `Vec<Box<str>>` (ordered)
- **Alternatives Considered**:
  - _HashSet_: Unordered unique fields. **Rejected** - order matters for query planning
  - _BTreeSet_: Sorted unique fields. **Rejected** - user-specified order is meaningful
- **Rationale**: User controls index priority (first = most important for queries).

## 5. Operational Readiness (The "Reality Check")

### 5.1 Observability

**Metrics** (via `tracing`):

```rust
#[tracing::instrument(level = "debug")]
fn load_task_config(raw: RawTaskConfig) -> Result<TaskConfig, ConfigError> {
    let start = Instant::now();
    let result = TaskConfig::from_raw(raw);

    if let Ok(ref config) = result {
        tracing::info!(
            fields_count = config.fields.len(),
            indexed_count = config.indexed_fields.len(),
            task_tags_count = config.task_tags.len(),
            status_mappings = config.status.by_name.len(),
            duration_ms = start.elapsed().as_millis(),
            "Task config loaded"
        );
    }

    result
}
```

**Logs**:
- `INFO`: Config loaded successfully (field counts, duration)
- `WARN`: Unknown fields in user config (forward compatibility)
- `ERROR`: Validation failures (specific field + reason)

**Health Checks**:
- Config validation success rate (should be 100% after initial setup)
- Regex compilation time (detect expensive patterns)

### 5.2 Migration Strategy

**Phase 1: Add TaskConfig Type** (Non-Breaking)
- Add `config/task.rs` with types
- Add `task: Option<TaskConfig>` to `Config` aggregate
- Default to `TaskConfig::default()` if missing

**Phase 2: Update Config Loaders**
- Deserialize `[task]` section from TOML
- Validate and construct `TaskConfig`
- Emit warnings for unknown fields

**Phase 3: Note Context Integration**
- Note context imports `TaskConfig`
- Use for task parsing (see 006b spec)

**Backward Compatibility**:
- Old vaults without `[task]` section use defaults
- Old checkboxes (`[x]`, `[ ]`, `[-]`) continue to work

### 5.3 Security & Privacy

**Regex DoS Prevention**:
- Limit pattern length (< 256 chars)
- Timeout regex compilation (5 seconds)
- Log expensive patterns for review

**No PII in Config**:
- Field names, enum values are vault-specific (not user data)
- Status symbols are single characters (no PII)

## 6. Pre-Mortem (The "Inversion")

- **Risk**: User defines `task_tags = []` (empty), all checkbox promotion breaks
  - _Mitigation_: Require at least one tag in default config; warn if empty

- **Risk**: Regex pattern is malicious (catastrophic backtracking)
  - _Mitigation_: Limit pattern length, timeout compilation, document safe patterns

- **Risk**: Duplicate field names in config (case-insensitive)
  - _Mitigation_: Case-sensitive field names; document convention (snake_case)

- **Risk**: Status symbol conflicts with markdown syntax (`[>]` is quote)
  - _Mitigation_: Document safe symbols; validate non-whitespace printable ASCII

- **Risk**: Config changes require full reindex (slow for large vaults)
  - _Mitigation_: Document in clean-slate protocol; defer hot-reload to future

## 7. Critique & Refinement Log

| Date       | Critique / Issue                                      | Resolution                                                  |
|:-----------|:------------------------------------------------------|:------------------------------------------------------------|
| 2026-02-08 | "Should TaskConfig own promotion logic?"              | No - promotion logic in Note context; config provides tags  |
| 2026-02-08 | "Should validation be public API?"                    | No - newtype pattern means validation at construction only  |
| 2026-02-08 | "How does Note context get FieldValue?"               | Config converts serde_json::Value → note::value::FieldValue |
| 2026-02-08 | "What if user config has typos in field names?"       | Unknown fields trigger warnings; indexed_fields validated   |

## 8. References

- [Rust API Guidelines: Type Safety](https://rust-lang.github.io/api-guidelines/type-safety.html)
- [Serde: Enum Representations](https://serde.rs/enum-representations.html)
- [Regex Crate Documentation](https://docs.rs/regex/latest/regex/)
- [TOML Specification](https://toml.io/en/)
- [ADR 001: Configuration System](../adr/001-configuration-system.md)
