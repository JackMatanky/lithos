# Obsidian Tasks Plugin Reference

Source digest: `docs/refs/digests/obsidian_obsidian-tasks-digest.txt`

This reference captures the Tasks plugin data model, task-line semantics, and
query language as used by Obsidian users. It prioritizes behavior that impacts
Lithos parsing, indexing, and query compatibility.

## Task Line Basics

- Tasks are checklist list items in Markdown:
  - `- [ ] task`, `* [ ] task`, `+ [ ] task`, and numbered lists are supported.
- Tasks can be indented, but query results are flat unless `show tree` is used.

## Task Formats (Emoji vs Dataview)

Tasks supports two formats, chosen in settings (one at a time):

- **Tasks Emoji Format** (default)
  - Uses emoji signifiers (e.g., `📅`, `🔁`, `⏫`, `✅`).
- **Dataview Format**
  - Uses inline fields like `[due:: 2023-05-01]` or `(due:: 2023-05-01)`.
  - Tasks reads bracketed inline fields only; when writing, it uses `[]`.
  - In Live Preview, adjacent bracketed fields can be hidden by markdown parsing;
    use two spaces or commas between fields.

Limitations:

- Tasks reads/writes only the selected format.
- Reading mode and query results still display emoji format regardless of
  selected format.
- Field order still matters in task lines.

## Dates on Tasks

Dates use `YYYY-MM-DD` and cannot include times. Invalid dates are ignored by
date searches and must be queried explicitly.

### Work-planning dates (user-provided)

- **Due date**: `📅 2021-04-09` -> `task.due`
- **Scheduled date**: `⏳ 2021-04-09` -> `task.scheduled`
- **Start date**: `🛫 2021-04-09` -> `task.start`
  - Start-date filters include tasks with no start date.

### History dates (auto-managed, settings-driven)

- **Created**: `➕ 2021-04-09` -> `task.created`
- **Done**: `✅ 2021-04-09` -> `task.done`
- **Cancelled**: `❌ 2021-04-09` -> `task.cancelled`

Notes:

- Dates are added by Tasks when enabled in settings (created/done/cancelled).
- Task status changes to `DONE` trigger done-date and recurrence logic.
- No support for date-time values.

## Statuses

Each task has a **status** determined by the single character in `[...]`.

Status fields:

- **symbol**: the character inside brackets
- **name**: label for display/search (customizable)
- **next symbol**: symbol when toggled
- **type**: `TODO`, `IN_PROGRESS`, `ON_HOLD`, `DONE`, `CANCELLED`, `NON_TASK`

Unknown status symbols are mapped to a built-in `Unknown` status:

- name: `Unknown`
- next symbol: `x`
- type: `TODO`

Default filters:

- `done` matches `DONE`, `CANCELLED`, `NON_TASK`
- `not done` matches `TODO`, `IN_PROGRESS`

Status-aware query fields:

- `status.name`, `status.type`
- `sort by status`, `sort by status.name`, `sort by status.type`
- `group by status`, `group by status.name`, `group by status.type`

## Priority

Priority signifiers (highest to lowest):

1. `🔺` highest
2. `⏫` high
3. `🔼` medium
4. none (between medium and low)
5. `🔽` low
6. `⏬️` lowest

Query fields and instructions:

- `priority is (above, below, not)? (lowest, low, none, medium, high, highest)`
- `sort by priority`, `group by priority`, `hide priority`
- `task.priorityNumber`, `task.priorityName`, `task.priorityNameGroupText`

## Recurrence

- Recurrence signifier: `🔁` followed by rule starting with `every`.
- When a recurring task is completed:
  - The current instance is marked done (with done date).
  - A new task is created (by default **one line above** the original).

Key behaviors:

- `when done` makes the next occurrence based on completion date rather than
  the original scheduled/due date.
- Setting: **Remove scheduled date on recurrence** (if Start or Due exists).
- Setting: **Order of the new task** (above/below original).

## On Completion

- Action signifier: `🏁` followed by action string.
- Supported actions:
  - `keep` (default)
  - `delete`
- Dataview format uses `[onCompletion:: action]` instead of emoji.

Warning:

- Do not use `🏁 delete` on tasks with nested list items or child tasks; it can
  leave indented orphan lines that become code blocks.

## Task Dependencies

Dependencies are task-to-task ordering constraints.

- **id** signifier: `🆔 abcdef`
  - Allowed characters: letters, digits, `_`, `-`.
- **depends on** signifier: `⛔ abcdef` (comma-separated ids allowed).

Blocking logic (direct dependencies only):

- A task is **blocking** if it is not done and at least one task depends on it.
- A task is **blocked** if it is not done and depends on any not-done task.

Filters:

- `has id`, `no id`, `id includes <text>`, `id regex matches /.../`
- `has depends on`, `no depends on`
- `is blocking`, `is not blocking`, `is blocked`, `is not blocked`

## Links

Link-aware fields are available only via scripting functions:

- `task.outlinks` (links on task line)
- `task.file.outlinksInBody`, `task.file.outlinksInProperties`, `task.file.outlinks`
- `query.file.outlinksInBody`, `query.file.outlinksInProperties`, `query.file.outlinks`

Limitations:

- No `inlinks` concept yet.
- Embeds are not treated as links.
- `Link.destinationPath` is computed when the file is read and may not update
  after moves without app reload.

## Global Filter vs Global Query

- **Global Filter**: optional string that must be present in a checklist item
  (e.g., `#task`) to be treated as a task.
  - If using a tag for the global filter, subtags are not supported.

- **Global Query**: a query fragment from settings that is prepended to all
  Tasks searches (more flexible than Global Filter).

Query assembly order:

1. Global Query (unless `ignore global query` is present)
2. Query File Defaults (from frontmatter)
3. Tasks block instructions

## Tasks Query Language

Tasks queries are code blocks:

````text
```tasks
not done
due before tomorrow
group by filename
sort by due reverse
limit 100
```
````

Instruction categories:

- **Filters**: status, dates, priority, recurrence, tags, description, file
  properties, dependencies, and custom functions.
- **Sorting**: `sort by ...` and `sort by function`.
- **Grouping**: `group by ...` and `group by function`.
- **Layout**: `hide/show ...` (fields, buttons, tree, urgency).
- **Limiting**: `limit`, `limit groups`.
- **Explain**: `explain` shows parsed query interpretation.
- **Comments**: `# comment` lines.

Case-sensitivity:

- Most instructions are case-insensitive.
- Boolean operators (`AND`, `OR`, `NOT`, `XOR`, `AND NOT`, `OR NOT`) are
  case-sensitive and must be capitalized.
- Regex patterns and scripting expressions are case-sensitive.

### Combining filters

- Combine filters on a single line using delimiters and boolean operators:
  - Delimiters: `(...)`, `[...]`, `{...}`, `"..."` (same type within a line).
  - Operators: `AND`, `OR`, `NOT`, `AND NOT`, `OR NOT`, `XOR`.
- Execution precedence: `NOT` > `XOR` > `AND` > `OR`.

### Line continuations

- A trailing `\` at end of a line continues the instruction on the next line.

## Sorting

Default sort order is always appended to every query:

```text
sort by status.type
sort by urgency
sort by due
sort by priority
sort by path
```

Notes:

- User-provided `sort by` lines take precedence and are applied before defaults.
- `sort by function` allows custom sort keys.

## Grouping

- `group by` creates headings; multiple groupings are allowed.
- `group by function` enables custom grouping keys.
- Status groupings have specific ordering and names (`Done` vs `Todo`).

## Layout

Task elements that can be hidden:

- `id`, `depends on`, `priority`, `cancelled date`, `created date`,
  `start date`, `scheduled date`, `due date`, `done date`, `recurrence rule`,
  `on completion`, `tags`.

Query elements that can be hidden/shown:

- `tree`, `edit button`, `postpone button`, `backlink`, `urgency`, `task count`.

`show tree`:

- Displays nested tasks and list items.
- Child items are shown even if they do not match filters.
- Sorting affects only top-level tasks; children keep file order.

## Limiting

- `limit <number>` (or `limit to <number> tasks`) caps total results.
- `limit groups <number>` caps items per group (ignored without grouping).

## Lithos Alignment Notes

- The Tasks plugin is a strong reference for emoji-based task metadata and
  query expectations in Obsidian ecosystems.
- Support for Dataview format is partial; if Lithos aims for compatibility,
  it should handle both emoji and bracketed inline fields.
- Query composition rules (global query, file defaults, local block) mirror
  how users expect layered filters to behave.

## Appendix A: Task Properties Map (scripting)

These are the primary properties exposed in Tasks scripting and used by
`filter by function`, `sort by function`, and `group by function`.

Status:

- `task.status.name`
- `task.status.type`
- `task.status.typeGroupText`
- `task.status.symbol`
- `task.status.nextSymbol`
- `task.isDone` (boolean)

Dates:

- `task.due`
- `task.scheduled`
- `task.start`
- `task.created`
- `task.done`
- `task.cancelled`
- `task.happens`

Recurrence and completion:

- `task.isRecurring`
- `task.recurrenceRule`
- `task.onCompletion`

Priority and urgency:

- `task.priorityNumber`
- `task.priorityName`
- `task.priorityNameGroupText`
- `task.urgency`

File metadata:

- `task.file.path`
- `task.file.pathWithoutExtension`
- `task.file.root`
- `task.file.folder`
- `task.file.filename`
- `task.file.filenameWithoutExtension`

Headings and identifiers:

- `task.hasHeading`
- `task.heading`
- `task.id`
- `task.dependsOn`
- `task.isBlocked(query.allTasks)`
- `task.isBlocking(query.allTasks)`

Description and tags:

- `task.description`
- `task.descriptionWithoutTags`
- `task.tags`

Links:

- `task.outlinks`
- `task.file.outlinksInProperties`
- `task.file.outlinksInBody`
- `task.file.outlinks`

Source line:

- `task.originalMarkdown`
- `task.lineNumber`

Query file helpers:

- `query.file.path`
- `query.file.pathWithoutExtension`
- `query.file.root`
- `query.file.folder`
- `query.file.filename`
- `query.file.filenameWithoutExtension`
- `query.file.outlinksInProperties`
- `query.file.outlinksInBody`
- `query.file.outlinks`

## Appendix B: Quick Reference (condensed)

Filters:

- Status: `done`, `not done`, `status.name`, `status.type`
- Dates: `due`, `scheduled`, `start`, `created`, `done`, `cancelled`, `happens`
- Recurrence: `is recurring`, `recurrence includes`, `recurrence regex`
- Priority: `priority is (above, below)? ...`
- Dependencies: `has id`, `id includes`, `has depends on`, `is blocked`
- File: `path`, `root`, `folder`, `filename`, `heading`
- Tags: `has tags`, `tag includes`, `tags include`

Sorting:

- `sort by status`, `status.name`, `status.type`
- `sort by due`, `scheduled`, `start`, `created`, `done`, `cancelled`
- `sort by priority`, `urgency`, `path`, `filename`, `folder`, `heading`
- `sort by id`, `recurring`, `recurrence`
- `sort by random`
- `sort by function <expression>`

Grouping:

- `group by status`, `status.name`, `status.type`
- `group by due`, `scheduled`, `start`, `created`, `done`, `cancelled`
- `group by priority`, `urgency`, `path`, `filename`, `folder`, `heading`
- `group by id`, `recurrence`
- `group by function <expression>`

Layout:

- Task elements: `hide id`, `hide depends on`, `hide priority`, `hide tags`,
  `hide due date`, `hide scheduled date`, `hide start date`, `hide done date`,
  `hide created date`, `hide cancelled date`, `hide recurrence rule`,
  `hide on completion`
- Query elements: `hide task count`, `hide backlink`, `hide edit button`,
  `hide postpone button`, `hide toolbar`
- `show urgency`, `show tree`

Other:

- `limit <n>`
- `limit groups <n>`
- `explain`
- `ignore global query`
