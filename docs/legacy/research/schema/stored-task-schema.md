# StoredTask Schema and Task Index Tables

This document defines a concrete draft schema for `StoredTask` and the
corresponding task index tables. It is a storage-focused specification intended
to guide implementation in the note context.

Goals
- Provide a task-level projection to enable fast task queries.
- Keep Notes as the source of truth (projection can be rebuilt).
- Support Dataview/Tasks-style metadata, dates, and status semantics.

StoredTask (projection record)

Identity and location
- task_id: TaskId (note::task::TaskId)
- note_id: NoteId (note::identity::NoteId)
- path: Box<str> (note path)
- heading: Option<Heading> (note::structure::Heading)
- position: SourceLocation (note::position::SourceLocation)
- block_id: Option<Box<str>> (if present in source)

Status
- status_symbol: StatusSymbol (config::task::StatusSymbol)
- status_name: StatusName (config::task::StatusName)

Content
- text: TaskText (note::task::TaskText)
- tags: Vec<Tag> (note::tag::Tag)

Dates
- created_at: Option<TaskTimestamp>
- due_at: Option<TaskTimestamp>
- start_at: Option<TaskTimestamp>
- scheduled_at: Option<TaskTimestamp>
- done_at: Option<TaskTimestamp>
- cancelled_at: Option<TaskTimestamp>

Relationships
- parent_task_id: Option<TaskId>
- depends_on: Vec<Box<str>> (ids referenced by dependsOn)

Notes
- StoredTask should reuse note value types where they are rkyv-safe and
  infallible to deserialize (Tag, TaskText, Heading, SourceByteOffset,
  TaskTimestamp).
- All timestamps are seconds since Unix epoch (UTC).
- TaskId should be stable for a task line and derived from parsing rules.
- StoredTask is an adapter type for persistence; domain Task remains canonical.

Table definitions (redb)

Projection table
- TASKS: TableDefinition<&str, &[u8]>
  - key: task_id as string
  - value: rkyv bytes for StoredTask

Metadata index
- TASKS_BY_METADATA: MultimapTableDefinition<&str, &str>
  - key: field + "\0" + typed_value
  - value: task_id as string

Typed value encoding
- string: "s:" + raw
- number: "n:" + JSON number
- date: "d:" + epoch seconds
- boolean (if needed): "b:true" / "b:false"
- arrays: stored as multiple entries (one per element)
- objects: stored as a JSON string under type "o:"

Date indexes
- TASKS_BY_CREATED_DATE
- TASKS_BY_DUE_DATE
- TASKS_BY_START_DATE
- TASKS_BY_SCHEDULED_DATE
- TASKS_BY_DONE_DATE
- TASKS_BY_CANCELLED_DATE
- Optional: TASKS_BY_HAPPENS_DATE

Each date index
- TableDefinition: MultimapTableDefinition<&str, &str>
- key: date string (ISO-8601 or epoch string; pick one and keep stable)
- value: task_id as string

Dependency index (conditional)
- TASKS_BY_DEPENDS_ON: MultimapTableDefinition<&str, &str>
  - key: dependsOn id string
  - value: task_id as string
- Only created when `task.dependencies.enabled = true`.

Key format guidance
- All table keys should be stable, ASCII-safe strings.
- Field names are stored as-is (case-sensitive); normalization should happen at
  parse/config time.

Query API alignment
- Provide a single query entrypoint for dates:
  - list_by_task_date(kind, range)
- Metadata queries should resolve the expected type from config; unknown fields
  are treated as strings.
