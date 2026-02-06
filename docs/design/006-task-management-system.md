---
feature: Configuration-Driven Task System with List Entities
status: Draft
author: James (AI Dev Agent)
ticket: N/A
date_created: 2026-01-30
tags: [tasks, lists, markdown, config, domain, epic-11, epic-12]
---

# Tech Spec: Configuration-Driven Task System with List Entities

> **Note**: See `docs/design/README.md` for usage instructions and T-Shirt sizing.

Related specs:

- [docs/design/001-config-models.md](001-config-models.md) (config model/value contracts used by task schema and metadata)
- [docs/design/002-config-cqrs.md](002-config-cqrs.md) (how config layers and merged config are persisted/retrieved)

## 1. Problem Space (The "Why")

### 1.1 Context & Background

**Current State**: The existing task representation is too simplistic for real-world task management workflows:

- Only tracks checkbox status (`[x]`, `[ ]`, `[-]`)
- No inline metadata support (`[key:: value]` fields)
- Cannot differentiate between simple checklists and rich tasks
- Missing temporal tracking (due dates, reminders, completion timestamps)
- No support for custom task types or user-defined metadata

**User Context**: Obsidian users leverage plugins like Dataview and Tasks to create rich task management systems with:
- Custom metadata fields (`[priority:: 1]`, `[project:: work]`)
- Temporal emoji markers (⏰ reminder, 📅 due, ✅ completed)
- Task type classification (action items, meetings, research tasks)
- Configurable status symbols beyond basic complete/incomplete

**User Example** (current workflow that fails):
```markdown
- [x] #task Convert SuperMemo to Anki [type:: action_item]
      [time_start:: 14:00] [time_end:: 15:00] [duration_est:: 150]
      ⏰ 2024-07-16 12:25 ➕ 2024-07-16 📅 2024-07-16 ✅ 2024-07-16
```

**Architectural Gap**: pulldown-cmark parses **all list types** (ordered, unordered, task lists), but Lithos only models tasks. Missing the `List` structural entity creates an impedance mismatch between parser events and domain model.

**Why Now**: Epic 11 (Query Service) and Epic 12 (Templates) need rich task metadata for:
- Task queries: `lithos tasks list --overdue --priority 1`
- Template functions: `{% for task in query.tasks_due_today() %}`
- Reports: Generate weekly task summaries

**Related Decisions**:
- [ADR 008](../adr/008-markdown-parsing.md): pulldown-cmark integration
- [Epic 11](../../_bmad-output/planning-artifacts/epics/epic-11-query-service-knowledge-graph-mvp-core.md): Query service
- [Epic 12](../../_bmad-output/planning-artifacts/epics/): Template system (TBD)

### 1.2 Goals & Non-Goals

**Goals**:
1. **Configuration-Driven Metadata**: Users define task schema in vault config (no hardcoded fields)
2. **Type Safety**: Validate metadata types at parse time (enums, bounds, date formats)
3. **Structural Completeness**: Model all list types (ordered, unordered, checkbox) from pulldown-cmark
4. **Task Promotion**: Auto-promote checkbox items to Task entities based on config rules
5. **Dataview Parity**: Support inline metadata (`[key:: value]`) and emoji temporal markers
6. **Query Integration**: Tasks queryable via Epic 11 (`find_tasks_by_field()`, `find_overdue_tasks()`)
7. **Template Access**: Tasks accessible in templates (`{{ task.metadata.get_field("priority") }}`)
8. **Zero Breaking Changes**: Existing task parsing continues to work

**Non-Goals**:
- **Recurring tasks**: No cron-style scheduling (`[repeat:: daily]`) in initial version
- **Task dependencies**: No graph-based dependencies (`[blocked-by:: task-id]`)
- **Time tracking**: No stopwatch/timer CLI integration
- **Subtask nesting**: Parse nested checkboxes as flat items (no parent-child relationships)
- **Pattern extraction**: NO regex extraction from task names (explicit metadata only)
- **Real-time sync**: Task updates require re-indexing (no live file watching in MVP)

### 1.3 Constraints (The Hard Limits)

**Architecture (current)**:
- **Sync-first core**: all task parsing, promotion, and validation in `lithos-core` is synchronous.
- **Context boundaries**:
    - Task/list domain types live in the **note context** (they are note content semantics + structure).
    - Task configuration lives in the **config context** (vault-config driven).
    - Template exposure is owned by the **template context** + CLI/app boundary.
    - Note/schema/template contexts must not import each other; importing config is allowed (config is cross-cutting).
- **No parser types in domain**: pulldown-cmark is adapter/infrastructure; domain types store only validated values (strings, offsets, ids).

**Zero-Copy Performance**:
- Task metadata values use `SettingValue` (**config-owned**) because the task system is explicitly config-defined (field types, enums, bounds).
- Parser must use `&str` slices, not allocate for every metadata field
- Target: Parse 1000 tasks in <100ms (inline with Epic 10 indexing goals)

**Backward Compatibility**:
- Existing checkboxes without metadata remain valid
- Parser must emit both `List` (structural) and `Task` (promoted) entities
- Config defaults must match current behavior (e.g., `[x]` → Complete)

**Data Integrity**:
- Invalid metadata values (bad enum, out-of-bounds) → parse error, not silent failure
- Unconfigured fields stored as `SettingValue::String` (forward compatibility)
- Task IDs must be UUID v7 for stable identity across file renames

**Epic Integration**:
- Must integrate with the core query surfaces (Epic 11 direction): tasks are indexed in redb and retrievable via note/query read models.
- Must integrate with template rendering (Epic 12 direction): tasks are accessible via the template context.

**No External State**:
- Task status changes tracked via file modifications only
- No separate task database (tasks are derived from note content during indexing)

## 2. Guide-Level Explanation (The "What")

### 2.1 User/Dev Experience

#### User Perspective: Configuring Tasks

**Step 1: Define Task Schema in Vault Config**

Users create `.lithos/lithos.toml` in their vault:

```toml
[task]
enabled = true

# Promotion rules - what makes a checkbox a "task"
promotion.tags = ["#task", "#todo", "#action-item"]
promotion.auto_promote_with_metadata = true

# Custom status symbols
[task.status]
complete = "x"
incomplete = " "
cancelled = "-"
in_progress = ">"
waiting = "?"

# Temporal markers (emojis + keywords)
[task.temporal.reminder]
emoji = "⏰"
keyword = "reminder"
field = "reminder"
format = "%Y-%m-%d %H:%M"
type = "datetime"

[task.temporal.due]
emoji = "📅"
keyword = "due"
field = "due_date"
format = "%Y-%m-%d"
type = "date"

[task.temporal.completed]
emoji = "✅"
keyword = "completed"
field = "completed_at"
format = "%Y-%m-%d"
type = "date"

# Custom metadata fields
[task.metadata.type]
keyword = "type"
field = "task_type"
type = "enum"
values = ["action_item", "reminder", "meeting", "research"]

[task.metadata.priority]
keyword = "priority"
field = "priority"
type = "integer"
min = 0
max = 10

[task.metadata.project]
keyword = "project"
field = "project_name"
type = "string"

# Query optimization
[task.indexing]
indexed_fields = ["due_date", "priority", "project_name", "task_type"]
```

**Step 2: Write Tasks with Metadata**

In any note:

```markdown
# My Tasks

- [ ] #task Review pull request [priority:: 1] [project:: lithos] 📅 2026-02-01
- [>] #task Implement cache layer [type:: action_item] [priority:: 2] ⏰ 2026-01-31 14:00
- [x] #todo Buy groceries [type:: reminder] ✅ 2026-01-30

## Simple Checklists (not promoted to tasks)
- [ ] Remember to call mom
- [x] Water plants
```

**Step 3: Query Tasks via CLI**

```bash
# List all incomplete tasks
lithos tasks list --status incomplete

# Find overdue tasks
lithos tasks overdue

# Query by metadata
lithos tasks list --field priority --value 1
lithos tasks list --field project_name --value lithos

# Tasks due today
lithos tasks due --when today
```

**Step 4: Use Tasks in Templates**

```jinja
{# templates/weekly-review.md #}
# Weekly Review - {{ date.now().format("%Y-W%V") }}

## Overdue Tasks
{% for task in query.tasks_overdue() %}
- [{{ task.status_symbol }}] {{ task.text }}
  - Priority: {{ task.metadata.get_number("priority") | default(0) }}
  - Due: {{ task.metadata.get_date("due_date") | date("%Y-%m-%d") }}
{% endfor %}

## This Week's Completed Tasks
{% for task in query.tasks_completed_this_week() %}
- ✅ {{ task.text }} ({{ task.metadata.get_string("project_name") | default("Personal") }})
{% endfor %}

## Tasks by Project
{% for project in query.projects_with_tasks() %}
### {{ project }}
{% for task in query.tasks_by_field("project_name", project) %}
- [{{ task.status_symbol }}] {{ task.text }} (P{{ task.metadata.get_number("priority") }})
{% endfor %}
{% endfor %}
```

**Step 5: Create Validated Tasks in Templates**

```jinja
{# templates/daily-tasks.md #}
# Tasks for {{ date.now().format("%Y-%m-%d") }}

{# Config-validated task creation with static values #}
{{ tasks.format_checkbox(
    text="Review pull request",
    status="x",
    metadata={
        "priority": 1,
        "project_name": "lithos",
        "due_date": "2026-02-01"
    }
) }}

{# Bulk task generation #}
{% set daily_items = ["Code review", "Update docs", "Run tests"] %}
{% for item in daily_items %}
{{ tasks.format_checkbox(
    text=item,
    status=" ",
    metadata={"priority": loop.index, "project_name": "lithos"}
) }}
{% endfor %}

{# Interactive task creation using prompts/suggesters #}
{# Prompts and suggesters are standard template functions #}
{% set task_text = prompt("Task description?") %}
{% set task_priority = prompt("Priority (0-10)?", type="number") %}
{% set task_project = suggester(
    "Select project",
    query.unique_field_values("project_name")
) %}
{% set task_type = suggester(
    "Task type",
    config.task.metadata.type.values
) %}
{% set due_date = prompt("Due date (YYYY-MM-DD)?", default=date.today()) %}

{{ tasks.format_checkbox(
    text=task_text,
    status=" ",
    metadata={
        "priority": task_priority,
        "project_name": task_project,
        "type": task_type,
        "due_date": due_date
    }
) }}
```

#### Developer Perspective: Domain Model

**Creating a Task from Config**

```rust
use lithos_core::note::task::{Task, StatusName};
use lithos_core::config::types::TaskConfig;

// Config loaded from vault
let config: TaskConfig = vault_config.task;

// Parse task from markdown checkbox
let raw_text = "#task Implement feature [priority:: 1] [project:: lithos]";
let task = Task::from_checkbox(
    raw_text,
    StatusName("incomplete".into()),
    42, // source-byte offset (start of list item)
    &config,
)?;

// Access metadata
assert_eq!(task.text(), "Implement feature");
assert_eq!(task.metadata.get_number("priority"), Some(1.0));
assert_eq!(task.metadata.get_string("project_name"), Some("lithos"));
```

**Query Service Integration (Epic 11)**

```rust
This spec does not prescribe an async application-layer `QueryService` API.

The core requirement is that task projections/indexes support fast lookups by configured fields.
Queries remain sync-first in `lithos-core`; any async orchestration belongs at the CLI/app edge.
```

**List Entity (Structural Model)**

```rust
use lithos_core::note::list::{List, ListType, ListItem};
use lithos_core::note::task::StatusName;

// Parser creates List entities for document structure
let list = List {
    list_type: ListType::Unordered,
    items: vec![
        ListItem::Plain {
            text: "Regular bullet".into(),
            position: 10,
        },
        ListItem::Checkbox {
            text: "#task Do work [priority:: 1]".into(),
            status: StatusName("incomplete".into()),
            position: 30,
            task_id: Some(task_uuid), // Links to promoted Task
        },
    ],
    depth: 0,
};

// Both list and promoted task stored in Note
note.add_list(list);
note.add_task(task); // Promoted from checkbox
```

### 2.2 Mental Model

**Three-Layer Hierarchy**:

```
┌─────────────────────────────────────────────────────────────┐
│ Document Structure (Lists)                                  │
│ - Captures markdown syntax (ordered/unordered/checkbox)     │
│ - Preserves position, nesting, all items                    │
│ - Stored in: Note::lists                                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ Promotion (config-driven)
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Rich Tasks (Promoted Checkboxes)                            │
│ - Has #task tag OR inline metadata                          │
│ - Queryable, indexed, template-accessible                   │
│ - Stored in: Note::tasks                                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ References
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Task Metadata (Config-Defined Schema)                       │
│ - User-defined fields (priority, project, type)             │
│ - Temporal markers (due, reminder, completed)               │
│ - Stored as: HashMap<String, SettingValue>                  │
└─────────────────────────────────────────────────────────────┘

Raw → Domain conversion (critical to avoid `Stored*` types):

- **Raw (adapter/parser)**: extract inline metadata as `&str` slices and map symbols/keywords using `TaskConfig`.
- **Domain (note context)**: store validated outputs in task/list types using lean owned strings (`Box<str>`) and config-owned `SettingValue` for typed metadata.
- **Persistence (db adapters)**: store domain types directly by default; only introduce `StoredTask` / `StoredList` if profiling shows the domain shape is inefficient for rkyv/redb.
```

**Key Concepts**:

1. **List = Structure, Task = Semantics**
   - Lists are "what markdown wrote"
   - Tasks are "what the user meant" (extracted via config rules)

2. **Promotion = Config-Driven Transformation**
   - Simple checkbox → List only
   - Checkbox with `#task` tag → List + Task
   - Checkbox with metadata → List + Task (if auto_promote enabled)

3. **Metadata = User Schema**
   - Not hardcoded in domain model
   - Defined per vault in config
   - Validated at parse time against config schema

4. **Status Symbols = User Preference**
   - `[x]` might mean "complete" in one vault
   - `[>]` might mean "in_progress" in another vault
   - Config maps symbols → semantic enum values

**Think of it like**:
- **List** = HTML `<ul>` or `<ol>` tag (structure)
- **Task** = Application-level "TODO item" (meaning)
- **TaskConfig** = JSON Schema for task metadata

## 3. Detailed Design (The "How")

### 3.1 System Architecture

#### Component Diagram

```mermaid
graph TB
    subgraph "Adapters Layer (SPI)"
        Parser[Markdown Parser<br/>pulldown-cmark]
        Storage[Redb Storage]
    end

    subgraph "Domain Layer"
        Note[Note Aggregate]
        List[List Entity]
        Task[Task Entity]
        TaskMeta[TaskMetadata]
        TaskConfig[TaskConfig]
    end

    subgraph "Application Layer"
        IndexSvc[Indexing Service<br/>Epic 10]
        QuerySvc[Query Service<br/>Epic 11]
        TmplSvc[Template Service<br/>Epic 12]
    end

    Parser -->|emits list events| List
    Parser -->|checks promotion| Task
    List -->|references| Task
    Task -->|contains| TaskMeta
    TaskConfig -->|validates| TaskMeta
    TaskConfig -->|defines promotion| Task

    Note -->|owns| List
    Note -->|owns| Task

    IndexSvc -->|parses notes| Parser
    IndexSvc -->|stores| Storage

    QuerySvc -->|reads| Storage
    QuerySvc -->|returns| Task

    TmplSvc -->|queries| QuerySvc
    TmplSvc -->|renders| Task
```

### 3.2 Data Models

#### Core Domain Entities

```rust
// lithos-core/src/note/list.rs

/// Represents a markdown list (ordered, unordered, or checkbox).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct List {
    /// Type of list
    list_type: ListType,
    /// Items in this list
    items: Vec<ListItem>,
    /// Nesting depth (0 = top-level)
    depth: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ListType {
    /// Numbered list (1., 2., 3.)
    Ordered { start: u64 },
    /// Bullet list (-, *, +)
    Unordered,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ListItem {
    /// Plain list item
    Plain {
        text: Box<str>,
        position: usize,
    },
    /// Checkbox item (may be promoted to Task)
    Checkbox {
        text: Box<str>,
        status: StatusName,
        position: usize,
        /// If promoted to Task, stores task ID for linkage
        task_id: Option<Uuid>,
    },
}
```

```rust
// lithos-core/src/note/task.rs

/// Rich task entity with user-configured metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Task {
    /// UUID v7 for stable identity
    id: Uuid,
    /// Clean task text (before first metadata marker)
    text: Box<str>,
    /// Status name (semantic), resolved from a single-character symbol via config.
    status: StatusName,
    /// First-class temporal attributes.
    ///
    /// These are *not* dynamic metadata: they have dedicated fields for simpler
    /// query/index/template usage.
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    due_at: Option<chrono::DateTime<chrono::Utc>>,
    reminder_at: Option<chrono::DateTime<chrono::Utc>>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Source-byte offset in document (start of list item)
    position: usize,
    /// Tags extracted from text
    tags: Vec<Tag>,
    /// Dynamic metadata (config-defined schema)
    metadata: TaskMetadata,
}

/// Status name (semantic identifier).
///
/// Constraint: ASCII alphanumeric plus `_` and `-`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StatusName(Box<str>);

/// Status symbol (single character) as it appears in markdown `[<symbol>]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StatusSymbol(char);
```

```rust
// lithos-core/src/note/task.rs (continued)

use crate::config::types::SettingValue;

/// Task metadata using config-defined schema.
///
/// Reuses SettingValue from config system for type flexibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TaskMetadata {
    /// All fields (temporal + custom) as SettingValue
    /// Field names come from TaskConfig definitions
    fields: HashMap<Box<str>, SettingValue>,
}

impl TaskMetadata {
    /// Get field by name.
    pub fn get(&self, field: &str) -> Option<&SettingValue> {
        self.fields.get(field)
    }

    /// Get string field (common case).
    pub fn get_string(&self, field: &str) -> Option<&str> {
        match self.get(field)? {
            SettingValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get number field.
    pub fn get_number(&self, field: &str) -> Option<f64> {
        match self.get(field)? {
            SettingValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get date field.
    pub fn get_date(&self, field: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        match self.get(field)? {
            SettingValue::Date(d) => Some(*d),
            _ => None,
        }
    }
}
```

#### Configuration Schema

```rust
// lithos-core/src/config/types.rs (additions)

/// User-defined task metadata schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TaskConfig {
    /// Enable task parsing
    pub enabled: bool,
    /// Promotion rules
    pub promotion: PromotionRules,
    /// Status mapping between semantic names and markdown checkbox symbols.
    pub status: CheckboxStatus,
    /// Unified field definitions (metadata + temporal markers).
    ///
    /// Map key is the canonical *field name* as stored in `TaskMetadata.fields`.
    pub fields: HashMap<Box<str>, TaskFieldSpecDef>,
    /// Indexed fields for query optimization
    pub indexed_fields: Vec<Box<str>>,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            promotion: PromotionRules::default(),
            status: CheckboxStatus::default(),
            fields: HashMap::new(),
            indexed_fields: vec![],
        }
    }
}

/// Mapping between semantic status names (used throughout the domain) and the
/// single-character symbol used in markdown `[<symbol>]`.
///
/// This replaces a hardcoded `CheckboxStatus` enum: status is vault-configurable,
/// while queries/indexing/templates operate on stable semantic names.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CheckboxStatus {
    pub by_name: HashMap<StatusName, StatusSymbol>,
    pub by_symbol: HashMap<StatusSymbol, StatusName>,
}

impl CheckboxStatus {
    pub fn symbol_for_name(&self, name: &StatusName) -> Option<StatusSymbol> {
        self.by_name.get(name).copied()
    }

    pub fn name_for_symbol(&self, symbol: StatusSymbol) -> Option<&StatusName> {
        self.by_symbol.get(&symbol)
    }
}

impl Default for CheckboxStatus {
    fn default() -> Self {
        let mut by_name = HashMap::new();
        by_name.insert(StatusName("complete".into()), StatusSymbol('x'));
        by_name.insert(StatusName("incomplete".into()), StatusSymbol(' '));
        by_name.insert(StatusName("cancelled".into()), StatusSymbol('-'));

        let by_symbol = by_name
            .iter()
            .map(|(name, symbol)| (*symbol, name.clone()))
            .collect();

        Self { by_name, by_symbol }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PromotionRules {
    /// Tags that trigger task promotion
    pub tags: Vec<Box<str>>,
    /// Auto-promote checkboxes with any metadata
    pub auto_promote_with_metadata: bool,
}

impl Default for PromotionRules {
    fn default() -> Self {
        Self {
            tags: vec!["#task".to_owned()],
            auto_promote_with_metadata: true,
        }
    }
}

/// Unified field spec definition.
///
/// The `type` tag determines which additional fields are legal, mirroring the
/// style of schema `PropertySpecDef`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum TaskFieldSpecDef {
    // --- General metadata fields (inline: [keyword:: value]) ---
    String {
        keyword: Box<str>,
        pattern: Option<Box<str>>,
        unit: Option<Box<str>>,
    },
    Integer {
        keyword: Box<str>,
        min: Option<f64>,
        max: Option<f64>,
        unit: Option<Box<str>>,
    },
    Float {
        keyword: Box<str>,
        min: Option<f64>,
        max: Option<f64>,
        unit: Option<Box<str>>,
    },
    Boolean {
        keyword: Box<str>,
    },
    Enum {
        keyword: Box<str>,
        values: Vec<String>,
        unit: Option<Box<str>>,
    },
    Time {
        keyword: Box<str>,
        format: Box<str>,
        unit: Option<Box<str>>,
    },

    // --- First-class temporal fields (populate Task.*_at) ---
    Created {
        emoji: Box<str>,
        keyword: Box<str>,
        format: Box<str>,
    },
    Due {
        emoji: Box<str>,
        keyword: Box<str>,
        format: Box<str>,
    },
    Reminder {
        emoji: Box<str>,
        keyword: Box<str>,
        format: Box<str>,
    },
    Completed {
        emoji: Box<str>,
        keyword: Box<str>,
        format: Box<str>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ValidationRules {
    /// Min value (for numbers)
    pub min: Option<f64>,
    /// Max value (for numbers)
    pub max: Option<f64>,
    /// Regex pattern (for strings)
    pub pattern: Option<Box<str>>,
    /// Unit label (e.g., "minutes")
    pub unit: Option<Box<str>>,
}
```

#### Storage Schema (Redb Tables)

```rust
// New tables for Epic 11 query support

// Primary task storage (composite key: note UUID + position)
// Foreign key relationship: Uuid references notes table
// Position is a source-byte offset within the note (start of the list item)
// (pulldown-cmark provides source byte ranges via Parser::into_offset_iter).
// When note is deleted → cascade delete all tasks with that note_uuid
tasks: Table<(Uuid, u64), ArchivedTask>
//             ^^^^  ^^^-- Position in source file (stable within file)
//             +--------- Note UUID (foreign key to notes:Table<Uuid, Note>)

// Query indexes (configured via task.indexing.indexed_fields)
// All indexes store task references as (note_uuid, position) tuples
tasks_by_due_date: Table<i64, Vec<(Uuid, u64)>>  // timestamp → task refs
tasks_by_priority: Table<i64, Vec<(Uuid, u64)>>  // priority → task refs
tasks_by_project: Table<String, Vec<(Uuid, u64)>> // project → task refs
tasks_by_status: Table<String, Vec<(Uuid, u64)>> // StatusName → task refs

// Generic metadata index (for non-indexed fields)
tasks_metadata: Table<(String, String), Vec<(Uuid, u64)>>  // (field, value) → task refs

// Referential integrity:
// - Tasks are derived from markdown during indexing (source of truth = markdown)
// - Note UUID is stable across file renames (moving tasks.md preserves task IDs)
// - Query "all tasks in note X" = scan tasks table where key.0 == note_uuid
// - Deleting note triggers cleanup of all tasks, list items, and index entries
```

### 3.3 Component & Interface Specifications

This section is the canonical contract for the core components involved in parsing, modeling, indexing, and exposing tasks.

#### Component: Markdown Parsing + Promotion (adapter)

- **Responsibility**: Convert pulldown-cmark events into note-domain `List` entities and (optionally) promoted `Task` entities.
- **Public Interface** (pseudocode; exact types may vary by adapter boundary):
    - `parse_note(markdown: &str, task_config: &TaskConfig) -> Result<Note, ParseError>`
        - _Behavior_:
            - builds `List` structures by handling balanced `Event::Start(Tag::List/Tag::Item)` and `Event::End(TagEnd::List/TagEnd::Item)` events.
            - identifies checkbox items via:
                - `Event::TaskListMarker(bool)` (only emitted when `Options::ENABLE_TASKLISTS` is enabled), and
                - adapter-owned parsing for custom status symbols (e.g. `[>]`, `[-]`, `[?]`, `[d]`) because `TaskListMarker` only conveys checked/unchecked.
            - computes positions as byte offsets from `Parser::into_offset_iter()` ranges (store `range.start` as the domain `position`).
            - performs deterministic promotion checks based on `TaskConfig`.
        - _Outputs_: always emits `List` entities; emits `Task` entities only when promotion rules match.
        - _Errors_:
            - task-specific validation failures (invalid metadata, invalid temporal format, unknown status symbol when strict)
- **State/Invariants**:
    - Domain types MUST NOT store pulldown-cmark types.
    - Promotion MUST be deterministic and config-driven.

#### Component: `TaskConfig` (config context)

- **Responsibility**: Define task promotion rules, status mapping (`CheckboxStatus`), field schema (`TaskFieldSpecDef`), and indexing policy.
- **Public Interface**:
    - `TaskConfig::default() -> TaskConfig`
        - _Behavior_: provides backward-compatible defaults (e.g., `x/ /-` mapping).
    - (Conceptual) `validate(&self) -> Result<(), ConfigError>`
        - _Behavior_: rejects invalid schema definitions early (e.g., enum defs with empty allowed values, invalid bounds).
        - _Errors_: schema invalid; inconsistent `indexed_fields`; duplicated keywords.
- **State/Invariants**:
    - Keyword → field mapping is the authority for `[keyword:: value]` parsing.
    - Indexed fields are an explicit user trade-off (memory/speed).

#### Component: `Task` (note context)

- **Responsibility**: Represent a promoted, queryable task derived from a checkbox item.
- **Public Interface**:
    - `Task::from_checkbox(raw_text: &str, status: StatusName, position: usize, config: &TaskConfig) -> Result<Task, DomainError>`
        - _Behavior_: extracts tags, computes clean display `text`, parses and validates metadata, assigns UUID v7.
        - _Errors_: invalid metadata value/format; schema mismatch.
    - `Task::should_promote(raw_text: &str, config: &TaskConfig) -> bool`
        - _Behavior_: checks promotion tags and/or metadata/temporal markers depending on config.
- **State/Invariants**:
    - `id` is stable for a given derived task instance (see constraints + migration notes).
    - `metadata` keys are field names from config (not raw keywords).

#### Component: `List` / `ListItem` (note context)

- **Responsibility**: Preserve markdown list structure (ordered/unordered/checkbox items) regardless of promotion.
- **Public Interface**:
    - `List::new(list_type: ListType) -> List`
    - `List::add_item(&mut self, item: ListItem)`
- **State/Invariants**:
    - For checkbox items, `task_id` is set iff a `Task` was promoted from that checkbox.
    - `depth` is structural nesting depth only (no semantic parent/child tasks in MVP).

#### Component: Task Indexing + Query Projection (storage/query boundary)

- **Responsibility**: Persist tasks in redb and optionally maintain field indexes as configured.
- **Public Interface** (conceptual; Epic 11 decides final surface):
    - `write_tasks(note_id: Uuid, tasks: &[Task], cfg: &TaskConfig) -> Result<(), StorageError>`
    - `query_tasks_by_field(field: &str, value: &SettingValue) -> Result<Vec<TaskRef>, QueryError>`
        - _Behavior_: uses field index when configured, otherwise falls back to scan.
- **State/Invariants**:
    - Tasks are derived; storage is a projection. Deleting a note deletes derived tasks.

#### Component: Template Task Formatting (template context)

- **Responsibility**: Provide an ergonomic, config-validated way to generate syntactically correct task checkboxes from templates.
- **Public Interface**:
    - `tasks.format_checkbox(text: String, status: &str, metadata: HashMap<String, SettingValue>) -> Result<String, TemplateError>`
        - _Behavior_: validates status + metadata against current vault config and returns a formatted markdown checkbox line.
        - _Errors_: invalid enum/format/out-of-bounds; invalid temporal format.
        - _Notes_: full pseudocode is kept in Appendix A.

### 3.4 Integration & Data Flow

#### Data Flow: Markdown → Task Entity

```mermaid
sequenceDiagram
    participant MD as Markdown File
    participant P as Parser (Adapter)
    participant TC as TaskConfig
    participant L as List Entity
    participant T as Task Entity
    participant N as Note Aggregate

    MD->>P: "- [x] #task Do work [priority:: 1]"
    P->>P: Parse pulldown-cmark events
    P->>L: Create ListItem::Checkbox
    P->>TC: Check promotion rules
    TC->>P: Matches "#task" tag → Promote
    P->>T: Task::from_checkbox(text, status, config)
    T->>TC: Parse metadata fields
    TC->>T: Validate priority (0-10)
    T->>T: Extract name: "Do work"
    T->>N: Add to tasks collection
    L->>L: Store task_id reference
    L->>N: Add to lists collection
```

#### Epic Integration Architecture

```mermaid
graph LR
    subgraph "Epic 10: Indexing"
        Parser[Parser] --> Note[Note + Lists + Tasks]
        Note --> Redb[(Redb Storage)]
    end

    subgraph "Epic 11: Query"
        Redb --> TaskIndex[Task Indexes]
        TaskIndex --> Query[QueryService]
        Query --> TaskAPI[Task Query API]
    end

    subgraph "Epic 12: Templates"
        TaskAPI --> TmplCtx[Template Context]
        TmplCtx --> MiniJinja[MiniJinja Engine]
    end

    subgraph "Epic 14: CLI"
        TaskAPI --> TasksCLI[lithos tasks]
    end
```

#### Events/Messages

This design is synchronous and in-process; “events” here refer to conceptual handoffs between components.

- `ParsedList { list: List }`
- `ParsedCheckboxItem { raw_text, status_symbol, position }`
- `PromotedTask { task: Task, list_item_task_id: Uuid }`
- `TaskValidationFailed { field, value, reason }`
- `IndexedTask { note_id, task_id, indexed_fields }`

#### Dependencies

- Markdown parsing: `pulldown-cmark` (adapter layer)
    - `Options::ENABLE_TASKLISTS` must be enabled to receive `Event::TaskListMarker(bool)`.
    - `Parser::into_offset_iter()` yields `(Event, Range<usize>)` source byte ranges that can be used for stable in-note positions.
    - `pulldown_cmark::utils::TextMergeWithOffset` can merge consecutive text events while preserving ranges.
- Persistence/query: `redb` (projection storage)
- Serialization: `rkyv` (persisted-bytes contract applies)
- Dates/times: `chrono` (format parsing/validation)
- Logging/metrics: `tracing`

### 3.5 Core Logic & Algorithms

#### Algorithm: Task Promotion Decision

```rust
impl Task {
    /// Check if checkbox should be promoted to Task.
    pub fn should_promote(text: &str, config: &TaskConfig) -> bool {
        // Check 1: Has promotion tag?
        if config.promotion.tags.iter().any(|tag| text.contains(tag)) {
            return true;
        }

        // Check 2: Has inline metadata (if auto-promote enabled)?
        if config.promotion.auto_promote_with_metadata {
            // Check for any [keyword:: pattern
            for (_field_name, spec) in &config.fields {
                if let Some(keyword) = keyword_for_field_spec(spec) {
                    if text.contains(&format!("[{}::", keyword)) {
                        return true;
                    }
                }

                match spec {
                    TaskFieldSpecDef::Created { emoji, .. }
                    | TaskFieldSpecDef::Due { emoji, .. }
                    | TaskFieldSpecDef::Reminder { emoji, .. }
                    | TaskFieldSpecDef::Completed { emoji, .. } => {
                        if text.contains(emoji.as_ref()) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }

        false
    }
}
```

#### Algorithm: Task Name Extraction

```rust
impl Task {
    /// Extract clean task name (before first metadata marker).
    fn extract_task_name(raw_text: &str, config: &TaskConfig) -> String {
        let mut text = raw_text.trim();

        // Remove promotion tags
        for tag in &config.promotion.tags {
            text = text.trim_start_matches(tag).trim();
        }

        // Find first metadata position
        let metadata_start = Self::find_metadata_start(text, config);

        // Task name = everything before metadata
        let task_name = if let Some(pos) = metadata_start {
            &text[..pos]
        } else {
            text
        }.trim();

        task_name.to_owned()
    }

    fn find_metadata_start(text: &str, config: &TaskConfig) -> Option<usize> {
        let mut positions = Vec::new();

        // Check for inline fields: [keyword::
        for (_field_name, spec) in &config.fields {
            if let Some(keyword) = keyword_for_field_spec(spec) {
                let pattern = format!("[{}::", keyword);
                if let Some(pos) = text.find(&pattern) {
                    positions.push(pos);
                }
            }

            match spec {
                TaskFieldSpecDef::Created { emoji, .. }
                | TaskFieldSpecDef::Due { emoji, .. }
                | TaskFieldSpecDef::Reminder { emoji, .. }
                | TaskFieldSpecDef::Completed { emoji, .. } => {
                    if let Some(pos) = text.find(emoji.as_ref()) {
                        positions.push(pos);
                    }
                }
                _ => {}
            }
        }

        positions.into_iter().min()
    }
}
```

#### Algorithm: Metadata Parsing

```rust
impl Task {
    /// Parse metadata from text using config schema.
    fn parse_metadata(
        text: &str,
        config: &TaskConfig,
    ) -> Result<TaskMetadata, DomainError> {
        let mut fields = HashMap::new();

        // Parse configured (non-temporal) metadata fields.
        //
        // Temporal fields (created/due/reminder/completed) are first-class on `Task`
        // and are parsed separately into `Task.*_at`.
        for (field_name, spec) in &config.fields {
            if matches!(
                spec,
                TaskFieldSpecDef::Created { .. }
                    | TaskFieldSpecDef::Due { .. }
                    | TaskFieldSpecDef::Reminder { .. }
                    | TaskFieldSpecDef::Completed { .. }
            ) {
                continue;
            }

            if let Some(value) = Self::extract_field_by_spec(text, spec)? {
                fields.insert(field_name.clone(), value);
            }
        }

        // Parse unconfigured fields (forward compatibility)
        for (key, raw_value) in Self::extract_all_inline_fields(text) {
            if !fields.contains_key(&key) {
                fields.insert(key, SettingValue::String(raw_value));
            }
        }

        Ok(TaskMetadata { fields })
    }

    fn extract_field_by_spec(
        text: &str,
        def: &TaskFieldSpecDef,
    ) -> Result<Option<SettingValue>, DomainError> {
        let keyword = keyword_for_field_spec(def).unwrap_or("");
        if keyword.is_empty() {
            return Ok(None);
        }

        // Pattern: [keyword:: value]
        let pattern = format!("[{}::", keyword);
        let Some(start) = text.find(&pattern) else {
            return Ok(None);
        };

        let value_start = start + pattern.len();
        let value_end = text[value_start..]
            .find(']')
            .map(|i| value_start + i)
            .ok_or_else(|| DomainError::ValidationFailed(
                format!("Unclosed metadata field: {}", def.keyword)
            ))?;

        let raw_value = text[value_start..value_end].trim();

        // Parse according to type
        let value = match def {
            TaskFieldSpecDef::String { .. } | TaskFieldSpecDef::Time { .. } => {
                SettingValue::String(raw_value.to_owned())
            }
            TaskFieldSpecDef::Integer { .. } => {
                let num = raw_value.parse::<i64>().map_err(|_| {
                    DomainError::ValidationFailed(format!("Invalid integer: {}", raw_value))
                })?;
                SettingValue::Number(num as f64)
            }
            TaskFieldSpecDef::Float { .. } => {
                let num = raw_value.parse::<f64>().map_err(|_| {
                    DomainError::ValidationFailed(format!("Invalid float: {}", raw_value))
                })?;
                SettingValue::Number(num)
            }
            TaskFieldSpecDef::Boolean { .. } => {
                let b = raw_value.parse::<bool>().map_err(|_| {
                    DomainError::ValidationFailed(format!("Invalid boolean: {}", raw_value))
                })?;
                SettingValue::Boolean(b)
            }
            TaskFieldSpecDef::Enum { values, .. } => {
                if !values.contains(&raw_value.to_owned()) {
                    return Err(DomainError::ValidationFailed(format!(
                        "Invalid enum value: {}. Allowed: {:?}",
                        raw_value, values
                    )));
                }
                SettingValue::String(raw_value.to_owned())
            }
            TaskFieldSpecDef::Created { .. }
            | TaskFieldSpecDef::Due { .. }
            | TaskFieldSpecDef::Reminder { .. }
            | TaskFieldSpecDef::Completed { .. } => {
                // Temporal is first-class on Task; not stored in TaskMetadata.
                return Ok(None);
            }
        };

        // Spec-level note: validation (bounds/pattern/unit) is encoded per variant.
        // This pseudocode omits full validation for brevity.

        Ok(Some(value))
    }

    fn validate_value(
        value: &SettingValue,
        rules: &ValidationRules,
    ) -> Result<(), DomainError> {
        if let SettingValue::Number(n) = value {
            if let Some(min) = rules.min {
                if *n < min {
                    return Err(DomainError::ValidationFailed(
                        format!("Value {} below minimum {}", n, min)
                    ));
                }
            }
            if let Some(max) = rules.max {
                if *n > max {
                    return Err(DomainError::ValidationFailed(
                        format!("Value {} above maximum {}", n, max)
                    ));
                }
            }
        }
        Ok(())
    }
}
```

#### Algorithm: Template Task Formatting (Config-Validated)

The full formatting + validation pseudocode is in Appendix A to keep Section 3.5 focused on the note-domain logic.

#### Parser Integration Flow

The parser integration pseudocode is in Appendix B.

## 4. Alternatives & Decisions (The "Divergence")

### 4.1 Tactical Decisions

#### Decision: Reuse `SettingValue` for Metadata Storage

**Context**: Need flexible type system for user-defined task metadata.

**Choice**: Use existing `SettingValue` enum from config system.

**Alternatives Considered**:
- **Custom `MetadataValue` enum**: Duplicate type definitions (String, Number, Boolean, Date). **Rejected** - violates DRY, adds complexity.
- **Generic `serde_json::Value`**: Too permissive, no compile-time type checking. **Rejected** - want domain-specific types.
- **Trait-based polymorphism**: `Box<dyn MetadataValue>`. **Rejected** - not `Serialize`, adds heap allocations.

**Rationale**: `SettingValue` already has:
- All needed types (String, Number, Boolean, Date, Array, Object)
- Serde serialization
- Debug masking for sensitive data (Encrypted variant)
- Conversion traits (`From<T>` impls)

**Trade-off**: Couples task metadata to config types, but they're both in domain layer (no boundary violation).

---

#### Decision: No Regex Pattern Extraction from Task Names

**Context**: User requested removal of brittle `_action_item` suffix extraction.

**Choice**: All metadata MUST be explicit via `[key:: value]` syntax.

**Alternatives Considered**:
- **Regex patterns in config**: Allow users to define patterns like `_([a-z_]+)$` to extract metadata. **Rejected** - brittle, couples naming to data.
- **NLP/AI extraction**: "Detect task type from natural language". **Rejected** - unreliable, requires ML models.

**Rationale**:
- **Explicit > Implicit**: `[type:: action_item]` is self-documenting
- **Validation**: Enum types catch typos at parse time
- **Discoverability**: Config shows available fields
- **No Hidden Coupling**: Task name is just a name, not a data container

**Migration**: Users update `_action_item` suffix to `[type:: action_item]` field.

---

#### Decision: User-Configurable Status Symbols

**Context**: Different users use different checkbox symbols (GTD: `[>]`, Waiting: `[?]`).

**Choice**: Status symbols defined in config, mapped to semantic enum values.

**Alternatives Considered**:
- **Hardcoded symbols**: Only `[x]`, `[ ]`, `[-]`. **Rejected** - not flexible.
- **Free-form symbols**: Any symbol allowed, no validation. **Rejected** - breaks queries (how to filter "complete" if symbol varies?).

**Rationale**:
- **Flexibility**: Each vault defines its own symbol set
- **Semantic Stability**: Queries/indexing use `StatusName` (semantic), not the raw symbol
- **Backward Compat**: Default symbols match existing behavior

**Trade-off**: Config must be loaded before parsing. Acceptable - config already needed for metadata schema.

---

#### Decision: Task Promotion via Config Rules

**Context**: Not all checkboxes are tasks (simple checklists vs. rich tasks).

**Choice**: Promote checkbox to Task when:
1. Has promotion tag (`#task` by default), OR
2. Has inline metadata (if `auto_promote_with_metadata` enabled)

**Alternatives Considered**:
- **Promote all checkboxes**: Simple, but pollutes task queries with grocery lists. **Rejected**.
- **Manual promotion only**: Require `#task` tag always. **Rejected** - UX burden.
- **Separate syntax**: Use different checkbox symbol for tasks (`[t]`). **Rejected** - non-standard markdown.

**Rationale**:
- **Opt-in semantics**: User explicitly marks tasks
- **Metadata implies intent**: If checkbox has `[priority:: 1]`, it's clearly a task
- **Zero false positives**: Simple checklists stay simple

---

#### Decision: Store Task Reference in ListItem

**Context**: Need bidirectional link between List structure and promoted Task.

**Choice**: `ListItem::Checkbox` has optional `task_id: Option<Uuid>`.

**Alternatives Considered**:
- **Separate linkage table**: `HashMap<ListItemPosition, TaskId>`. **Rejected** - adds complexity.
- **Task has list reference**: `Task::list_id`. **Rejected** - violates aggregate boundaries (Task doesn't own List).
- **No linkage**: Keep them independent. **Rejected** - can't navigate from list to task.

**Rationale**:
- List owns the structural relationship
- Task can be queried independently
- Parser can atomically create both and link them

---

#### Decision: Template Task Formatter with Config Validation

**Context**: Users need to generate tasks in templates without manual syntax errors or invalid metadata.

**Choice**: Provide `tasks.format_checkbox(text, status, metadata)` function that validates all fields against vault config.

**Alternatives Considered**:
- **Raw string concatenation**: Let users write `"- [x] #task {{ text }} [priority:: {{ p }}]"` manually. **Rejected** - no validation, brittle to config changes.
- **Separate validation function**: `tasks.validate()` called after formatting. **Rejected** - errors happen after template renders, not during.
- **Auto-correct invalid values**: Silently fix out-of-bounds priority (11 → 10). **Rejected** - masks user errors, breaks expectations.

**Rationale**:
- **Early Errors**: Template fails at render time if metadata invalid (better than silently creating broken tasks)
- **Config Alignment**: Function uses same validation as parser (DRY principle)
- **Discoverability**: Template errors show allowed values/formats ("priority must be 0-10")
- **Forward Compat**: Unknown fields stored as strings (graceful degradation)
- **Interoperability**: Accepts output from prompts/suggesters without special handling (standard template variables)

**Integration Pattern**:
```jinja
{# Prompts and suggesters return primitive types (string, number, etc.) #}
{% set priority = prompt("Priority?", type="number") %}  {# Returns: number #}
{% set project = suggester("Project", projects) %}        {# Returns: string #}

{# format_checkbox validates and formats them #}
{{ tasks.format_checkbox(
    text="My task",
    metadata={"priority": priority, "project_name": project}
) }}
```

**Trade-off**: Templates require access to `TaskConfig` (loaded before render). Acceptable - config already needed for queries.

---

#### Decision: No Nested Task Hierarchies (MVP)

**Context**: Checkboxes can be nested in markdown.

**Choice**: Parse nested checkboxes as flat items (depth stored in List, but no parent-child in Task).

**Alternatives Considered**:
- **Task parent/child graph**: `Task::subtasks: Vec<Task>`. **Rejected** - complex queries, circular refs.
- **Dependency graph**: `Task::depends_on: Vec<Uuid>`. **Rejected** - future feature, not MVP.

**Rationale**:
- **Structural nesting ≠ semantic dependency**: Indented checkbox might just be formatting
- **Query simplicity**: Flat task list easier to filter/sort
- **Future extension**: Can add `parent_task_id` later without breaking changes

---

#### Decision: IndexedFields in Config for Performance

**Context**: Task queries need fast lookups by metadata fields.

**Choice**: Config specifies `indexed_fields: ["priority", "due_date", "project_name"]`. Parser creates Redb tables for indexed fields.

**Alternatives Considered**:
- **Index all fields**: Create table for every field. **Rejected** - memory waste for rarely-queried fields.
- **Auto-detect hot fields**: Index based on query frequency. **Rejected** - requires stats collection, cold-start problem.
- **No indexes**: Scan all tasks for every query. **Rejected** - violates NFR1 (<500ms queries).

**Rationale**:
- **User control**: Power users index heavily-queried fields
- **Memory/speed trade-off**: User decides based on vault size
- **Explicit config**: No hidden performance cliffs

## 5. Operational Readiness (The "Reality Check")

### 5.1 Observability

**Metrics** (via `tracing`):

```rust
// Task parsing metrics
#[tracing::instrument(level = "debug", skip(config))]
fn parse_tasks(markdown: &str, config: &TaskConfig) -> Vec<Task> {
    let start = Instant::now();
    let result = /* parsing */;

    tracing::debug!(
        task_count = result.len(),
        duration_ms = start.elapsed().as_millis(),
        "Tasks parsed"
    );

    result
}

// Task promotion decision
tracing::debug!(
    text = %checkbox_text,
    has_tag = has_promotion_tag,
    has_metadata = has_inline_metadata,
    promoted = should_promote,
    "Promotion decision"
);

// Metadata validation errors
tracing::warn!(
    field = %field_name,
    value = %raw_value,
    error = %validation_error,
    "Metadata validation failed"
);

// Query performance
#[tracing::instrument(level = "debug")]
fn find_tasks_by_field(field: &str, value: &SettingValue) -> Result<Vec<Task>> {
    tracing::debug!(
        field = %field,
        cache_hit = is_indexed,
        "Task query executed"
    );
}
```

**Logs**:
- `DEBUG`: Task parsing decisions (promotion, metadata extraction)
- `INFO`: Batch parse completion (1000 tasks indexed)
- `WARN`: Validation failures (bad enum values, out-of-bounds)
- `ERROR`: Parse failures (unclosed brackets, invalid UTF-8)

**Health Checks**:
- Task parse success rate (should be >99%)
- Average parse time per task (<1ms target)
- Metadata validation failure rate (monitor for config issues)

### 5.2 Migration Strategy

**Phase 1: Add Domain Entities** (Non-Breaking)
- Add `List`, `ListItem`, `TaskMetadata` to domain
- Add `TaskConfig` to config types
- No parser changes yet
- **Validation**: Unit tests pass, no regressions

**Phase 2: Update Parser** (Dual-Write)
- Parser emits both old `Task` and new `List` entities
- `Note::tasks` and `Note::lists` coexist
- Old code continues using `Note::tasks` directly
- **Validation**: Existing tests pass, new tests for `List`

**Phase 3: Add Config Support**
- Load `TaskConfig` from vault config
- Parser uses config for promotion/metadata
- Default config matches current behavior (backward compat)
- **Validation**: Parse existing vaults, compare output to Phase 2

**Phase 4: Epic 11 Integration**
- Add task query methods to `QueryService`
- Add Redb indexes for configured fields
- CLI commands for task queries
- **Validation**: Query smoke tests, performance benchmarks

**Phase 5: Epic 12 Integration**
- Add task functions to template context
- Document template API
- Example templates for task reports
- **Validation**: Template rendering tests

**Rollback Strategy**:
- Each phase is a separate PR with feature flag
- Can disable task system via config: `task.enabled = false`
- Falls back to old simple checkbox parsing

### 5.3 Security & Privacy

**No New Attack Surface**:
- Tasks are parsed from user-owned markdown files (same trust level as existing notes)
- No external network requests for task data
- No task-specific authentication/authorization (vault-level permissions apply)

**Validation Protections**:
- Enum validation prevents SQL-injection-style attacks in queries (`[status:: '; DROP TABLE--]` rejected)
- Bounds validation prevents resource exhaustion (`[priority:: 999999999]` rejected)
- Format validation prevents datetime parsing attacks

**PII Considerations**:
- Task metadata may contain sensitive info (`[client:: John Doe]`)
- Already covered by vault encryption (Epic 5)
- Query results use same access control as notes

**Rate Limiting**:
- Task parsing bounded by file size limits (existing vault validation)
- Query result sets limited by Epic 11 pagination (default 1000 tasks)

## 6. Pre-Mortem (The "Inversion")

> Assume it is 6 months from now and this task system failed. Why?

### Risk 1: Config Schema Chaos

**Scenario**: Users create incompatible task configs across vaults. Task exports fail when moving notes between vaults with different schemas.

**Mitigation**:
- Validate config on load (error early if invalid enum values, missing required fields)
- Document config migration guide
- Tool: `lithos config validate-tasks` checks schema consistency
- Unknown fields gracefully stored as `SettingValue::String` (forward compat)

### Risk 2: Query Performance Degradation

**Scenario**: Users index 50+ fields, creating hundreds of Redb tables. Queries slow down, indexing takes minutes.

**Mitigation**:
- Document recommended max indexed fields (5-10)
- Warn if `indexed_fields.len() > 10` on config load
- Benchmark with large datasets (10k+ tasks) before shipping
- Epic 11 pagination prevents loading all tasks into memory

### Risk 3: Metadata Parsing Ambiguity

**Scenario**: User writes `[time:: 14:00]` but config has no `time` field. System stores as custom field, but query fails because field name differs from intent.

**Mitigation**:
- Parser warns for unrecognized fields: `tracing::warn!("Unknown field: time")`
- Tool: `lithos tasks validate` checks all tasks against current config
- Config documentation with field examples

### Risk 4: Status Symbol Collisions

**Scenario**: User configures `[>]` as "in_progress" but another vault uses `[>]` for "forwarded". Notes moved between vaults have wrong status.

**Mitigation**:
- Export/import tool validates status mappings
- CLI: `lithos tasks remap-status --from '>:forwarded' --to '>:in_progress'`
- Documentation: Recommend stable symbols across vaults

### Risk 5: Template Complexity Explosion

**Scenario**: Users create deeply nested task queries in templates. Template rendering takes seconds.

**Mitigation**:
- Epic 12 template timeout (5s default)
- Document query best practices (filter before iterating)
- Caching: Memoize expensive task queries during template render

### Risk 6: Emoji Parsing Inconsistency

**Scenario**: Emoji rendering differs across editors (Vim vs VS Code). Parser fails to detect temporal markers.

**Mitigation**:
- Use Unicode codepoint matching, not visual rendering
- Test suite includes various emoji encodings
- Config allows both emoji AND keyword syntax: `⏰ 2026-01-30` OR `[reminder:: 2026-01-30]`

## 7. Critique & Refinement Log

| Date       | Critique / Issue                                                                 | Resolution                                                                                                   |
| :--------- | :------------------------------------------------------------------------------- | :----------------------------------------------------------------------------------------------------------- |
| 2026-01-30 | "Pattern extraction from task names is brittle"                                 | **RESOLVED**: Removed regex patterns. All metadata must be explicit via `[key:: value]`.                    |
| 2026-01-30 | "Custom MetadataValue enum duplicates SettingValue"                             | **RESOLVED**: Reuse `SettingValue` from config system. Avoid type duplication.                              |
| 2026-01-30 | "Hardcoded status symbols limit user flexibility"                               | **RESOLVED**: Status symbols configurable per vault. Semantic enum values stable for queries.               |
| 2026-01-30 | "No clear distinction between structural lists and semantic tasks"              | **RESOLVED**: Separate `List` (structure) and `Task` (promoted) entities. Linked via `task_id`.             |
| 2026-01-30 | "How does parser know which metadata fields exist?"                              | **RESOLVED**: `TaskConfig` loaded before parsing. Config defines all valid fields.                          |
| 2026-01-30 | "What if task has metadata but no promotion tag?"                                | **RESOLVED**: `auto_promote_with_metadata` config flag. Default: true (metadata implies task).              |
| 2026-01-30 | "Performance concern: Parsing 1000s of tasks with complex metadata"             | **DESIGN NOTE**: Target <100ms for 1000 tasks. Use `&str` slices, lazy metadata parsing if needed.          |
| 2026-01-30 | "Nested tasks (subtasks) not modeled"                                           | **ACCEPTED LIMITATION**: MVP treats all tasks as flat. Depth stored in `List` but not in `Task` hierarchy. |
| 2026-01-30 | "Unknown metadata fields - fail or store?"                                       | **RESOLVED**: Store as `SettingValue::String` for forward compatibility. Warn in logs.                      |
| 2026-01-30 | "How to migrate existing simple tasks to new system?"                            | **RESOLVED**: Dual-write parser (Phase 2). Default config matches old behavior (backward compat).           |
| 2026-01-30 | "Config validation - when to reject invalid schemas?"                            | **RESOLVED**: Validate on load. Enum values checked, bounds validated. Early error prevents bad parses.     |
| 2026-01-30 | "Task query index strategy - index everything or selective?"                     | **RESOLVED**: User configures `indexed_fields`. Parser creates Redb tables for specified fields only.       |
| 2026-01-30 | "Emoji temporal markers - Unicode normalization issues?"                         | **DESIGN NOTE**: Use codepoint matching. Test with various encodings. Allow keyword fallback.               |
| 2026-01-30 | "Template task creation needs prompts/suggesters for metadata values"            | **RESOLVED**: `format_checkbox()` accepts output from standard template prompt/suggester functions. No special integration needed - works with any template variable source. |
| 2026-01-30 | "How do templates get metadata options for suggesters (e.g., existing projects)?" | **RESOLVED**: Epic 11 `query.unique_field_values(field)` returns list of distinct values for suggester options. Config enum values available via `config.task.metadata.<field>.values`. |
| 2026-01-30 | "Task ID stability when file moves/renames"                                      | **RESOLVED**: Use UUID v7 (time-ordered). ID assigned at parse time, stable unless task text changes.       |
| 2026-01-30 | "How to handle task status changes without separate state store?"               | **ACCEPTED LIMITATION**: Status is in markdown file. Changes require file edit + re-index. No live updates. |
| TBD        | _Post-implementation reviews will update this table_                             |                                                                                                              |

---

## 8. References

- pulldown-cmark: https://docs.rs/pulldown-cmark/
- redb: https://docs.rs/redb/
- rkyv: https://docs.rs/rkyv/
- tracing: https://docs.rs/tracing/
- chrono: https://docs.rs/chrono/

## Appendix A: Template Task Formatting (Config-Validated)

```rust
impl TaskTemplateContext {
    /// Format task checkbox with config validation.
    ///
    /// Used in templates to generate syntactically correct task checkboxes
    /// with guaranteed-valid metadata based on current vault config.
    ///
    /// Accepts standard template variables from any source:
    /// - Static values: `text="My task"`
    /// - Prompts: `text=prompt("Description?")`
    /// - Suggesters: `metadata={"project": suggester("Pick", projects)}`
    /// - Query results: `metadata={"priority": task.metadata.priority}`
    fn format_checkbox(
        &self,
        text: String,
        status: &str,
        metadata: HashMap<String, SettingValue>,
    ) -> Result<String, TemplateError> {
        let config = self.vault_config.task();

        // 1. Validate status symbol
        let symbol = status.chars().next().ok_or(TemplateError::InvalidStatus {
            symbol: status.to_owned(),
            allowed: config
                .status
                .by_symbol
                .keys()
                .map(|s| s.0.to_string())
                .collect(),
        })?;

        if config.status.name_for_symbol(StatusSymbol(symbol)).is_none() {
            return Err(TemplateError::InvalidStatus {
                symbol: status.to_owned(),
                allowed: config
                    .status
                    .by_symbol
                    .keys()
                    .map(|s| s.0.to_string())
                    .collect(),
            });
        }

        // 2. Validate all metadata fields against config
        let mut validated_fields = HashMap::new();
        let mut temporal_markers = Vec::new();

        for (key, value) in metadata {
            match config.fields.get(key.as_str()) {
                Some(TaskFieldSpecDef::Created { emoji, format, .. })
                | Some(TaskFieldSpecDef::Due { emoji, format, .. })
                | Some(TaskFieldSpecDef::Reminder { emoji, format, .. })
                | Some(TaskFieldSpecDef::Completed { emoji, format, .. }) => {
                    let formatted = validate_temporal_value(&value, format)?;
                    temporal_markers.push((emoji.clone(), formatted));
                    validated_fields.insert(key, value);
                }
                Some(field_def) => {
                    validate_metadata_field(&value, field_def)?;
                    validated_fields.insert(key, value);
                }
                None => {
                    // Unknown field - store as string (forward compat)
                    tracing::warn!(field = %key, "Template referenced unknown metadata field");
                    validated_fields.insert(key, value);
                }
            }
        }

        // 3. Build formatted checkbox
        let mut parts = vec![];
        parts.push(format!("- [{}]", status));
        if !config.promotion.tags.is_empty() {
            parts.push(config.promotion.tags[0].to_string());
        }
        parts.push(text);

        // Inline metadata fields (non-temporal)
        for (key, value) in &validated_fields {
            match config.fields.get(key.as_str()) {
                Some(TaskFieldSpecDef::Created { .. }
                | TaskFieldSpecDef::Due { .. }
                | TaskFieldSpecDef::Reminder { .. }
                | TaskFieldSpecDef::Completed { .. }) => {
                    // rendered as emoji marker
                }
                Some(spec) => {
                    let keyword = keyword_for_field_spec(spec).unwrap_or(key.as_str());
                    parts.push(format!("[{}:: {}]", keyword, format_setting_value(value)));
                }
                None => {
                    parts.push(format!("[{}:: {}]", key, format_setting_value(value)));
                }
            }
        }

        // Temporal markers at end
        for (emoji, date_str) in temporal_markers {
            parts.push(format!("{} {}", emoji, date_str));
        }

        Ok(parts.join(" "))
    }
}

fn validate_temporal_value(
    value: &SettingValue,
    format: &str,
) -> Result<String, TemplateError> {
    let SettingValue::String(s) = value else {
        return Err(TemplateError::TypeMismatch {
            field: "temporal".to_owned(),
            expected: format!("string matching {}", format),
            actual: format!("{:?}", value),
        });
    };

    // Spec-level contract: formats are chrono-compatible, and parsing rules
    // are defined by config per field.
    if chrono::NaiveDateTime::parse_from_str(s, format).is_ok()
        || chrono::NaiveDate::parse_from_str(s, format).is_ok()
    {
        return Ok(s.clone());
    }

    Err(TemplateError::InvalidDateTimeFormat {
        value: s.clone(),
        expected_format: format.to_owned(),
    })
}

fn validate_metadata_field(
    value: &SettingValue,
    field_def: &TaskFieldSpecDef,
) -> Result<(), TemplateError> {
    match field_def {
        TaskFieldSpecDef::Enum { .. } => {
            if !matches!(value, SettingValue::String(_)) {
                return Err(TemplateError::TypeMismatch {
                    field: "enum".to_owned(),
                    expected: "string (enum)".to_owned(),
                    actual: format!("{:?}", value),
                });
            }
        }
        TaskFieldSpecDef::Integer { .. } | TaskFieldSpecDef::Float { .. } => {
            if !matches!(value, SettingValue::Number(_)) {
                return Err(TemplateError::TypeMismatch {
                    field: "number".to_owned(),
                    expected: "number".to_owned(),
                    actual: format!("{:?}", value),
                });
            }
        }
        TaskFieldSpecDef::Boolean { .. } => {
            if !matches!(value, SettingValue::Boolean(_)) {
                return Err(TemplateError::TypeMismatch {
                    field: "boolean".to_owned(),
                    expected: "boolean".to_owned(),
                    actual: format!("{:?}", value),
                });
            }
        }
        TaskFieldSpecDef::String { .. } | TaskFieldSpecDef::Time { .. } => {
            if !matches!(value, SettingValue::String(_)) {
                return Err(TemplateError::TypeMismatch {
                    field: "string".to_owned(),
                    expected: "string".to_owned(),
                    actual: format!("{:?}", value),
                });
            }
        }
        TaskFieldSpecDef::Created { .. }
        | TaskFieldSpecDef::Due { .. }
        | TaskFieldSpecDef::Reminder { .. }
        | TaskFieldSpecDef::Completed { .. } => {
            // Temporal fields are validated via validate_temporal_value()
        }
    };
    Ok(())
}
```

## Appendix B: Parser Integration Flow

```rust
// Pseudocode for parser (adapters layer)

impl MarkdownParser {
    fn parse_note(&self, markdown: &str, config: &TaskConfig) -> Result<Note, ParseError> {
        let mut note = Note::new(uuid, path)?;

        // Task list markers are *opt-in* in pulldown-cmark.
        // Even with ENABLE_TASKLISTS, Event::TaskListMarker(bool) only supports [ ] and [x].
        // Custom status symbols (e.g. [>], [-], [?], [d]) must be parsed by the adapter.
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TASKLISTS);

        let iter = Parser::new_ext(markdown, options).into_offset_iter();
        let iter = pulldown_cmark::utils::TextMergeWithOffset::new(iter);

        let mut current_list: Option<List> = None;
        let mut current_item: Option<ItemBuilder> = None;

        for (event, range) in iter {
            match event {
                Event::Start(Tag::List(start_num)) => {
                    current_list = Some(List::new(
                        if let Some(start) = start_num {
                            ListType::Ordered { start }
                        } else {
                            ListType::Unordered
                        },
                        range.start,
                    ));
                }

                Event::Start(Tag::Item) => {
                    current_item = Some(ItemBuilder {
                        position: range.start,
                        checked: None,
                        text: String::new(),
                    });
                }

                Event::TaskListMarker(checked) => {
                    if let Some(item) = current_item.as_mut() {
                        item.checked = Some(checked);
                    }
                }

                Event::Text(t) | Event::Code(t) => {
                    if let Some(item) = current_item.as_mut() {
                        item.text.push_str(&t);
                    }
                }

                Event::SoftBreak | Event::HardBreak => {
                    if let Some(item) = current_item.as_mut() {
                        item.text.push('\n');
                    }
                }

                Event::End(TagEnd::Item) => {
                    let Some(item) = current_item.take() else {
                        continue;
                    };

                    // Adapter determines the status symbol:
                    // - If pulldown-cmark emitted TaskListMarker, map checked/unchecked to config symbols.
                    // - Otherwise, attempt to detect a configured custom symbol from the item text prefix.
                    let status_symbol = match item.checked {
                        Some(true) => Some('x'),
                        Some(false) => Some(' '),
                        None => detect_custom_status_symbol(&item.text, config),
                    };

                    if let Some(symbol) = status_symbol {
                        let Some(status_name) = config
                            .status
                            .name_for_symbol(StatusSymbol(symbol))
                            .cloned()
                        else {
                            // Unknown status symbol: treat as non-checkbox or error depending on strictness.
                            continue;
                        };

                        let normalized_text = strip_checkbox_prefix(&item.text, symbol);

                        let (list_item, promoted_task) = if Task::should_promote(&normalized_text, config) {
                            let task = Task::from_checkbox(normalized_text.as_str(), status_name.clone(), item.position, config)?;
                            (
                                ListItem::Checkbox {
                                    text: normalized_text.into(),
                                    status: status_name,
                                    position: item.position,
                                    task_id: Some(task.id()),
                                },
                                Some(task),
                            )
                        } else {
                            (
                                ListItem::Checkbox {
                                    text: normalized_text.into(),
                                    status: status_name,
                                    position: item.position,
                                    task_id: None,
                                },
                                None,
                            )
                        };

                        if let Some(list) = current_list.as_mut() {
                            list.add_item(list_item);
                        }
                        if let Some(task) = promoted_task {
                            note.add_task(task);
                        }
                    } else {
                        // Not a checkbox item → plain list item.
                        if let Some(list) = current_list.as_mut() {
                            list.add_item(ListItem::Plain {
                                text: item.text.into(),
                                position: item.position,
                            });
                        }
                    }
                }

                Event::End(TagEnd::List(_)) => {
                    if let Some(list) = current_list.take() {
                        note.add_list(list);
                    }
                }

                _ => {}
            }
        }

        Ok(note)
    }
}

struct ItemBuilder {
    position: usize,
    checked: Option<bool>,
    text: String,
}

fn detect_custom_status_symbol(text: &str, config: &TaskConfig) -> Option<char> {
    // Pseudocode: detect "[<symbol>]" prefix based on configured symbols.
    // (pulldown-cmark only recognizes [ ] and [x] as task list markers.)
    for symbol in config.status.by_symbol.keys() {
        let needle = format!("[{}]", symbol.0);
        if text.trim_start().starts_with(&needle) {
            return Some(symbol.0);
        }
    }
    None
}

fn strip_checkbox_prefix(text: &str, symbol: char) -> String {
    // Pseudocode: remove leading "[<symbol>]" from the human-visible text.
    let needle = format!("[{}]", symbol);
    text.trim_start()
        .strip_prefix(&needle)
        .map(|rest| rest.trim_start().to_owned())
        .unwrap_or_else(|| text.to_owned())
}
```

## Appendix C: Redb Schema DDL

```rust
// Pseudocode for Redb table creation

fn create_task_tables(db: &Database, config: &TaskConfig) -> Result<()> {
    let txn = db.begin_write()?;

    // Primary task storage
    let _ = txn.open_table::<(Uuid, u64), ArchivedTask>("tasks")?;

    // Status index (always created)
    let _ = txn.open_table::<String, Vec<(Uuid, u64)>>("tasks_by_status")?;

    // Dynamic indexes based on config
    for field_name in &config.indexed_fields {
        let table_name = format!("tasks_by_{}", field_name);

        // Field type determines index key type
        let field_def = config.fields.get(field_name.as_ref());

        match field_def {
            Some(TaskFieldSpecDef::Integer { .. } | TaskFieldSpecDef::Float { .. }) => {
                let _ = txn.open_table::<i64, Vec<(Uuid, u64)>>(&table_name)?;
            }
            Some(
                TaskFieldSpecDef::Created { .. }
                | TaskFieldSpecDef::Due { .. }
                | TaskFieldSpecDef::Reminder { .. }
                | TaskFieldSpecDef::Completed { .. },
            ) => {
                let _ = txn.open_table::<i64, Vec<(Uuid, u64)>>(&table_name)?; // timestamp
            }
            _ => {
                let _ = txn.open_table::<String, Vec<(Uuid, u64)>>(&table_name)?;
            }
        };
    }

    txn.commit()?;
    Ok(())
}
```

## Appendix D: Related ADRs (To Be Created)

- **ADR 00XX**: Task metadata schema validation strategy
- **ADR 00XX**: Task query index selection algorithm
- **ADR 00XX**: Template context API for task access

---

## Appendix E: Status & Next Steps

**Status**: Draft (awaiting review)

**Next Steps**:
1. Review with architect (validate hexagonal boundaries)
2. Performance engineer review (confirm <100ms parse target feasible)
3. Epic 11/12 team review (confirm API contracts)
4. Create implementation stories
5. Update status to "Approved" after consensus

## Appendix F: Config Example (Full)

See Section 2.1 for complete `lithos.toml` example.
