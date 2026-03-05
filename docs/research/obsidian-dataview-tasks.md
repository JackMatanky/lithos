# Obsidian Dataview + Tasks Learnings for Lithos

This document summarizes concrete lessons from the Dataview and Obsidian Tasks plugins and translates them into implementation requirements for Lithos, with a focus on task querying, metadata, and performance at Obsidian-scale vaults.

Sources reviewed
- Dataview docs: data annotation, task metadata, query types, API overview.
- Obsidian Tasks docs: queries, filters, sorting, grouping, global query, statuses, dates, recurrence, dependencies, dataview format compatibility.

Non-negotiable user expectations set by these plugins
1) Tasks are first-class data.
- Tasks are queried, sorted, and grouped independently of notes.
- Queries return actionable tasks, not just notes.
- Query results are expected to be fast and scalable with 100k+ tasks.

2) Metadata is universal and flexible.
- Both inline fields and page metadata are queryable.
- Arbitrary fields are expected to work without manual schema changes.

3) Tasks have rich implicit properties.
- Status, dates, tags, file path, heading/section, line, original line text.
- Task identity (id) and dependency relationships (dependsOn).

4) Editing via query results is normal.
- Users expect to toggle task status from query output and have the source line update.

5) Queries are both simple and expressive.
- A compact query language for daily use.
- An escape hatch for advanced users (Dataview-style API or custom functions).

Dataview learnings (critical behaviors)
1) Inline field model
- Dataview treats bracketed inline fields `key:: value` as primary metadata.
- Fields can appear on task lines and should be indexed just like frontmatter.

2) Task metadata format compatibility
- Dataview uses task field shorthands like `due`, `completion`, `created`, `start`, `scheduled`.
- Tasks plugin reads Dataview inline fields in task lines; users expect interoperability.

3) Task properties expected by users
- Task fields often queried in Dataview include: status, tags, text, line, path, file/section, and links.

Obsidian Tasks learnings (critical behaviors)
1) Task date system
- Emoji signifiers for dates:
  - Due:        📅
  - Scheduled:  ⏳
  - Start:      🛫
  - Created:    ➕
  - Done:       ✅
  - Cancelled:  ❌
- Supports date filters, sorting, grouping, and a computed "happens" date (earliest of start/scheduled/due).

2) Statuses and status types
- Status symbol maps to a status name and a status type.
- Status types: TODO, IN_PROGRESS, ON_HOLD, DONE, CANCELLED, NON_TASK.
- Filters and sorting are driven by status type, not just checkbox state.

3) Recurrence
- Recurrence rules use a dedicated signifier and are parsed into a rule.
- "when done" changes recurrence calculation semantics.
- Date priority matters for recurrence.

4) Dependencies
- Tasks can have an id and dependsOn list.
- Filters like `is blocked` and `is blocking` are core behavior.

5) Query execution model
- Global query / global filter applied to all queries by default.
- Queries support filters, sorting, grouping, limits, and custom functions.

Lithos implementation requirements

1) Task projection model
- Introduce a task projection table (`TASKS`) with a `StoredTask` record.
- `StoredTask` is a projection, not a source of truth. Notes remain canonical.

Minimum `StoredTask` fields
- Identity and location
  - task_id, note_id
  - path, heading/section, line (or byte offset)
  - block_id (optional)
- Status
  - status_symbol, status_name, status_type
- Content
  - text (raw task description)
  - tags (list)
- Dates
  - created_at, due_at, start_at, scheduled_at, done_at, cancelled_at (optional)
- Relationships
  - parent_task_id (optional)
  - depends_on (optional list)

2) Metadata indexing
- Add `TASKS_BY_METADATA` multimap for arbitrary task metadata fields.
- Use a user-facing key of `field:value` (no type annotation required).
- Internally, encode value with a type tag derived from config or parser:
  - string: `s:`
  - number: `n:`
  - date: `d:`
- Key format: `{field}\0{typed_value}`

3) Date indexing
- Separate task date indexes per date type:
  - TASKS_BY_DUE_DATE
  - TASKS_BY_SCHEDULED_DATE
  - TASKS_BY_START_DATE
  - TASKS_BY_CREATED_DATE
  - TASKS_BY_DONE_DATE
  - TASKS_BY_CANCELLED_DATE
- Provide a single query API method `list_by_task_date(kind, range)`.
- Optional but recommended: TASKS_BY_HAPPENS_DATE.

4) Dependency indexing (conditional)
- Add `TASKS_BY_DEPENDS_ON` only when `task.dependencies.enabled = true`.
- Without it, dependency queries are O(N) and should be considered out of scope.

5) Query behavior parity targets
- Filters, sorting, grouping for:
  - status, status.type
  - dates (including "happens")
  - tags, path, heading, file name
  - metadata fields (arbitrary)
- Global query mechanism that is applied unless explicitly bypassed.

6) Compatibility requirements
- Parse Tasks emoji date signifiers.
- Parse Dataview inline fields in task lines `key:: value` with `[]` or `()`.
- Preserve exact source line information to support query-view edits.

Migration plan for task indexes

Phase 0: Add new tables and config
- Add tables: TASKS, TASKS_BY_METADATA, TASKS_BY_*_DATE.
- Add config flag: `task.dependencies.enabled` (default false for safe rollout).

Phase 1: Dual-write projections
- During note indexing, write StoredTask rows and new indexes.
- Keep existing note-based task indexes during transition.

Phase 2: Query migration
- Route task-level queries to TASKS.
- Implement metadata/date queries on new indexes.
- Gate dependency queries on `task.dependencies.enabled`.

Phase 3: Remove legacy indexes
- Remove TASKS_BY_PRIORITY and TASKS_BY_PROJECT (superseded by TASKS_BY_METADATA).

Phase 4: Rebuild path
- Provide a full rebuild path for all task projections.
- Rebuild should be deterministic and recoverable.

Product risk assessment
- Without a task projection, task queries will not be Obsidian-class and will feel slow.
- Without generic metadata indexing, Dataview-style queries are impossible.
- Without status types and "happens" semantics, tasks will behave inconsistently with user expectations.

Recommended next steps
1) Define StoredTask schema (storage DTO) and add TASKS table.
2) Implement TASKS_BY_METADATA with typed internal encoding.
3) Implement per-date indexes and a unified date query API.
4) Add config for dependencies and optional index.
5) Align parsing with Tasks emoji format and Dataview inline fields.
