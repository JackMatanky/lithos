# Obsidian Dataview Reference

Source digest: `docs/refs/digests/obsidian_blacksmithgu-obsidian-dataview-digest.txt`

This reference summarizes Dataview behavior that impacts Lithos data modeling,
indexing, and query semantics.

## Dataview Scope and Guarantees

- Dataview is a live index and query engine over a vault.
- It is designed for display and calculation, not editing.
  - Exception: task checkboxes can be toggled from Dataview views.
- JavaScript queries run with full plugin privileges (file and network access).
  - DQL queries are sandboxed and cannot perform destructive actions.

## Data Sources and Field Definition

Dataview only indexes metadata, not arbitrary paragraph content.

### Primary sources

- YAML frontmatter (Obsidian standard) at top of file.
- Inline fields inside Markdown content.
- Implicit fields derived from file metadata, tags, links, lists, and tasks.

### Inline field syntax

Inline fields are `Key:: Value` pairs.

Examples:

```markdown
Basic Field:: Value
**Bold Field**:: Nice!
I would rate this a [rating:: 9]!
You can also write [field:: inline fields]; multiple [field2:: on the same line].
This will not show the (longKeyIDontNeedWhenReading:: key).
```

Rules:

- Inline fields on their own line can omit brackets.
- Inline fields embedded in sentences or tasks must use bracket syntax.
- Parenthesis syntax hides the key in reader mode but still indexes the field.

### Field key normalization

- Keys with spaces are normalized to lowercase with dashes (e.g. `Basic Field`
  becomes `basic-field`).
- Keys with capitalization can be referenced by their sanitized form (all
  lowercase) to avoid casing differences.
- Formatting tokens are removed from keys (e.g. `**Bold Field**` becomes
  `bold-field`).

### Emoji and non-latin keys

- UTF-8 keys are allowed.
- Emoji keys must use bracket syntax `[🎅:: value]`.
- Emoji codepoints can differ by OS, which can affect lookup behavior.

## Field Types

Dataview assigns a type to each field and renders/filters based on type.

- Text: default if no other type matches.
- Number: numeric values (integers, floats, negatives).
- Boolean: `true` or `false`.
- Date: ISO8601 strings (e.g. `YYYY-MM`, `YYYY-MM-DD`, `YYYY-MM-DDTHH:mm:ssZ`).
  - Date properties are available: `year`, `month`, `day`, `hour`, `minute`,
    `second`, `millisecond`, `weekday`, `week`, `weekyear`.
- Duration: `<time> <unit>` strings (e.g. `6 hours`, `4min`, `6hr7min`).
- Link: `[[Page]]` or `[[Page|Display]]`.
  - In frontmatter, links must be quoted: `key: "[[Link]]"`.
- List: multi-value fields.
  - YAML lists in frontmatter.
  - Comma-separated in inline fields; text values must be quoted.
- Object: YAML map with nested fields in frontmatter.

Rules for lists:

- Repeated keys in a file are collected into a list of values.

## Implicit Fields (Page-Level)

Implicit fields are exposed under `file.*`:

- `file.name`: note title
- `file.folder`: folder path
- `file.path`: full path including filename
- `file.ext`: file extension (usually `md`)
- `file.link`: link to the file
- `file.size`: file size in bytes
- `file.ctime` / `file.cday`: created timestamp / date
- `file.mtime` / `file.mday`: modified timestamp / date
- `file.tags`: all tags, including expanded subtags
- `file.etags`: explicit tags only (no subtag expansion)
- `file.inlinks`: incoming links
- `file.outlinks`: outgoing links
- `file.aliases`: frontmatter aliases
- `file.tasks`: all tasks in the file
- `file.lists`: all list items (including tasks)
- `file.frontmatter`: raw key/value list for frontmatter
- `file.day`: parsed date from filename or date field
- `file.starred`: true if bookmarked

## Task and List Item Fields

Tasks and list items inherit fields from their parent page and add task-specific
implicit fields:

- `status`: completion status character (space, `x`, or custom)
- `checked`: true if status is not empty
- `completed`: true only when status is `x`
- `fullyCompleted`: true if task and all subtasks completed
- `text`: raw task text including inline fields
- `visual`: rendered task text (can be overridden in DataviewJS)
- `line`: line number in file
- `lineCount`: number of lines for the task
- `path`: path to the file (same as `file.path`)
- `section`: link to containing section
- `tags`: tags inside task text
- `outlinks`: links inside task text
- `link`: link to closest linkable block near the task
- `children`: subtasks or sublists
- `task`: true if task, false if regular list item
- `annotated`: true if inline fields present
- `parent`: line number of parent task, or null for root
- `blockId`: block id (from `^blockId`) or null

### Task field shorthands (emoji dates)

Dataview supports Tasks plugin date shorthands, mapped to textual fields:

- `due`: `🗓️YYYY-MM-DD`
- `completion`: `✅YYYY-MM-DD`
- `created`: `➕YYYY-MM-DD`
- `start`: `🛫YYYY-MM-DD`
- `scheduled`: `⏳YYYY-MM-DD`

## Query Modes

Dataview supports four query modes:

1) DQL (Dataview Query Language): pipeline-style, SQL-like syntax.
2) Inline expressions: `= this.file.name` style DQL expressions.
3) DataviewJS: JavaScript API with full index access and render helpers.
4) Inline JS expressions: `$= dv.current().file.mtime` style.

## Query Structure (DQL)

A DQL query consists of:

- Exactly one query type (e.g. LIST, TABLE, TASK, CALENDAR).
- Optional FROM statement to define a source (tag, folder, compound source).
- Optional filtering/sort/group commands.

Example:

```dataview
TABLE file.name AS "File", rating AS "Rating"
FROM #book
```

## DataviewJS API (Core Concepts)

DataviewJS provides a `dv` object:

- `dv.current()` returns page info for the current file.
- `dv.pages(source)` returns a data array of page objects.
- `dv.page(path)` resolves a path or link to a page object.
- Rendering helpers: `dv.list`, `dv.taskList`, `dv.table`, `dv.header`,
  `dv.paragraph`, `dv.span`.
- `dv.execute` and `dv.executeJs` run queries inside a JS block.
- `dv.view(path, input)` loads a JS view from a vault path and executes it.
  - Path is vault-root relative and cannot start with a dot directory.

## Data Model Shapes (reference)

These shapes are descriptive; Dataview page objects are plain JS objects.

### Page object (conceptual)

```js
{
  // user-defined fields from frontmatter + inline fields
  <field>: <value>,

  // implicit page fields
  file: {
    name: string,
    folder: string,
    path: string,
    ext: string,
    link: Link,
    size: number,
    ctime: Date,
    cday: Date,
    mtime: Date,
    mday: Date,
    tags: list<string>,
    etags: list<string>,
    inlinks: list<Link>,
    outlinks: list<Link>,
    aliases: list<string>,
    tasks: list<Task>,
    lists: list<ListItem>,
    frontmatter: list<string>,
    day: Date | null,
    starred: boolean
  }
}
```

### Task (conceptual)

```js
{
  status: string,
  checked: boolean,
  completed: boolean,
  fullyCompleted: boolean,
  text: string,
  visual: string,
  line: number,
  lineCount: number,
  path: string,
  section: Link,
  tags: list<string>,
  outlinks: list<Link>,
  link: Link,
  children: list<Task>,
  task: boolean,
  annotated: boolean,
  parent: number | null,
  blockId: string | null,

  // inherits page fields + user-defined fields
  file: <page file fields>,
  <field>: <value>
}
```

### Link (conceptual)

```js
{
  path: string,
  display?: string,
  embed?: boolean,
  subpath?: string,
  type?: "file" | "header" | "block"
}
```

## Query Semantics and Methods (DQL)

### Query Types

- `LIST`: bullet list of page links or group keys; supports one optional value.
- `TABLE`: tabular view with zero or more columns; supports `AS` headers.
- `TASK`: interactive task list; operates at task level; can mutate tasks.
- `CALENDAR`: calendar dots for a date field; SORT/GROUP have no effect.

### Data Commands (executed top-to-bottom)

- `FROM`: source selector (tags, folders, files, links). Only once, immediately
  after query type.
- `WHERE`: filter by boolean expression (can appear multiple times).
- `SORT`: sort by one or multiple fields, with direction.
- `GROUP BY`: group by a field or computed expression; yields `rows` arrays.
- `FLATTEN`: expand list field into individual rows, optionally aliasing.
- `LIMIT`: cap results to N.

Notes:

- DQL is pipeline-based; each command transforms the current result set.
- Multiple WHERE/GROUP/SORT commands are allowed and executed in order.

### Sources

- Tag source: `#tag` (includes subtags).
- Folder source: `"path/to/folder"` (no trailing slash).
- File source: `"path/to/file"` (folder vs file ambiguity resolved by extension).
- Link source: `[[note]]` (incoming links), `outgoing([[note]])` (outgoing links).
- Combine with `and`, `or`, `-` (negation), and parentheses for precedence.
- `[[]]` or `[[#]]` reference the current file.

### Expressions

Expressions are any values usable in WHERE, fields, and calculations:

- Literals: numbers, strings, dates, durations, lists, objects, links.
- Fields: direct field access or normalized keys with dashes.
- Arithmetic: `+ - * / %`
- Comparisons: `= != < > <= >=`
- Strings: concatenation with `+`, repetition with `*`.
- Indexing: `obj.key`, `obj["key"]`, `list[index]`.
- Link indexing: `[[Page]].field` to access page fields by link.
- Lambdas: `(x) => expression` for map/filter/reduce.

### Function Families (DQL)

Dataview functions are vectorized by default (apply to lists too).

Constructors:

- `object(key1, value1, ...)`
- `list(value1, value2, ...)` / `array(...)`
- `date(any)` / `date(text, format)`
- `dur(any)`
- `number(string)`
- `string(any)`
- `link(path, [display])`
- `embed(link, [embed?])`
- `elink(url, [display])`
- `typeof(any)`

Numeric:

- `round(number, [digits])`, `trunc(number)`, `floor(number)`, `ceil(number)`
- `min(...)`, `max(...)`, `sum(array)`, `product(array)`, `average(array)`
- `reduce(array, operand)`, `minby(array, func)`, `maxby(array, func)`

Collections and logic:

- `contains`, `icontains`, `econtains`, `containsword`
- `extract(object, key1, key2, ...)`
- `sort(list)`, `reverse(list)`, `length(object|array)`
- `nonnull(array)`, `firstvalue(array)`
- `all(array|args, [predicate])`, `any(array|args, [predicate])`,
  `none(array, [predicate])`
- `join(array, [delimiter])`, `filter(array, predicate)`, `unique(array)`
- `map(array, func)`, `flat(array, [depth])`, `slice(array, [start], [end])`

Strings:

- `regextest`, `regexmatch`, `regexreplace`, `replace`
- `lower`, `upper`, `split`, `startswith`, `endswith`
- `padleft`, `padright`, `substring`, `truncate`

Utility:

- `default(field, value)` / `ldefault(list, value)`
- `display(value)`
- `choice(bool, left, right)`
- `hash(seed, [text], [variant])`
- `striptime(date)`
- `dateformat(date, format)`, `durationformat(duration, format)`
- `currencyformat(number, currency)`
- `localtime(date)`
- `meta(link)` with `display`, `embed`, `path`, `subpath`, `type`

### DataArray Methods (DataviewJS)

DataArray is a proxied array with immutable transforms and field swizzling.

- Transform: `where`, `filter`, `map`, `flatMap`, `mutate`
- Order: `sort`, `groupBy`, `distinct`, `limit`, `slice`, `concat`
- Queries: `find`, `findIndex`, `includes`, `indexOf`
- Aggregates: `sum`, `avg`, `min`, `max`, `every`, `some`, `none`
- Access: `first`, `last`, `to(key)`, `expand(key)`, `array()`
- Field swizzling: `dataArray.field` maps and flattens field across elements.

## Lithos Alignment Notes

## Lithos Alignment Notes

- Treat frontmatter and inline fields as first-class metadata sources.
- Preserve key normalization rules (spaces, casing, formatting tokens).
- Model implicit fields for pages, lists, and tasks to keep parity.
- Include task shorthand emoji mappings in the parser.
- Treat Dataview as a read-only indexer with a second-phase view layer; align
  with Lithos projection-cache design rather than primary storage.
- Align query semantics with DQL pipeline execution and function vectorization.

## Appendix A: Function Catalog (compact)

Each entry lists the canonical signature and a short note. Functions are
vectorized unless otherwise noted.

Constructors and coercion:

- `object(key1, value1, ...)` create object from alternating key/value pairs
- `list(value1, value2, ...)` / `array(...)` create list
- `date(any)` parse date from string/date/link
- `date(text, format)` parse date with Luxon format tokens
- `dur(any)` parse duration
- `number(string)` extract first number from text
- `string(any)` coerce to string (dates/durations formatted)
- `link(path, [display])` internal link
- `embed(link, [embed?])` mark link as embed
- `elink(url, [display])` external link
- `typeof(any)` return type string

Numeric:

- `round(number, [digits])`
- `trunc(number)`
- `floor(number)`
- `ceil(number)`
- `min(a, b, ...)` / `min(array)`
- `max(a, b, ...)` / `max(array)`
- `sum(array)`
- `product(array)`
- `average(array)`
- `reduce(array, operand)` operands: `+ - * / & |`
- `minby(array, func)`
- `maxby(array, func)`

Collections and logic:

- `contains(object|list|string, value)`
- `icontains(object|list|string, value)` case-insensitive
- `econtains(object|list|string, value)` exact match for list/object keys
- `containsword(list|string, value)` exact word match, case-insensitive
- `extract(object, key1, key2, ...)`
- `sort(list)`
- `reverse(list)`
- `length(object|array)`
- `nonnull(array)`
- `firstvalue(array)`
- `all(array|args, [predicate])`
- `any(array|args, [predicate])`
- `none(array, [predicate])`
- `join(array, [delimiter])`
- `filter(array, predicate)`
- `unique(array)`
- `map(array, func)`
- `flat(array, [depth])`
- `slice(array, [start], [end])`

Strings and regex:

- `regextest(pattern, string)` partial match
- `regexmatch(pattern, string)` full-string match
- `regexreplace(string, pattern, replacement)`
- `replace(string, pattern, replacement)`
- `lower(string)`
- `upper(string)`
- `split(string, delimiter, [limit])` delimiter is regex
- `startswith(string, prefix)`
- `endswith(string, suffix)`
- `padleft(string, length, [padding])`
- `padright(string, length, [padding])`
- `substring(string, start, [end])`
- `truncate(string, length, [suffix])`

Utilities:

- `default(field, value)` vectorized default
- `ldefault(list, value)` non-vectorized default
- `display(value)` render display text for links/markdown
- `choice(bool, left, right)`
- `hash(seed, [text], [variant])` stable hash for randomization
- `striptime(date)`
- `dateformat(date, format)` returns string
- `durationformat(duration, format)`
- `currencyformat(number, currency)` locale-aware
- `localtime(date)`
- `meta(link)` returns link metadata: `display`, `embed`, `path`, `subpath`, `type`
