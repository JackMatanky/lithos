# Obsidian Metadata Menu Reference

Source digest: `docs/refs/digests/obsidian_mdelobelle-metadatamenu-digest.txt`

This reference captures Metadata Menu concepts that inform Lithos schema design:
field typing, fileClass mapping, indexed paths for nested fields, and API
operations for reading/writing metadata in notes.

## Core Purpose

- Metadata Menu manages metadata in **frontmatter** and **inline fields**
  (`field:: value` Dataview syntax).
- It focuses on data quality: typed fields, validation, and controlled editing.
- Field definitions can be **global presets** or **fileClass-specific**.

## Field Types (schema surface)

The plugin assigns a type to each field. Types define validation and controls.

Primitive:

- `Input` (free text; default)
- `Boolean`
- `Number` (optional min/max/step)
- `Date`, `DateTime`, `Time`

Selection:

- `Select` (single choice)
- `Multi` (multi choice)
- `Cycle` (next value from list)

File references:

- `File`, `MultiFile` (links to notes)
- `Media`, `MultiMedia` (links to media)

Computed/lookup:

- `Lookup` (query other notes, store result)
- `Formula` (JS function over note fields)

Canvas:

- `Canvas`
- `Canvas Group`
- `Canvas Group Link`

Structured:

- `JSON`, `YAML`
- `Object` (parent field)
- `Object List` (list of objects)

Note: Metadata Menu supports nested fields via Object/Object List. Lithos MVP
remains flat-only, but nested fields should remain in scope for future schema
iterations.

## Field Definitions and Precedence

- A field definition includes:
  - `name`, `type`, `id`, `options`, and `path` for nesting.
- Definitions live in:
  - **Preset fields** (plugin settings, global)
  - **FileClass fields** (stored in fileClass notes)
- Precedence: **fileClass definition overrides preset** for the same field.
- Field names are **case-sensitive** and **unique per nesting level**.
- Some types cannot be nested: `Lookup`, `Formula`, `Canvas`, `Canvas Group`,
  `Canvas Group Link`.

## FileClasses (schema per-context)

- A **fileClass** is a note stored in a configured folder.
- The file name is the class name; subfolders are used in identifiers
  (e.g., `fileClass: area/projects/Project`).
- FileClass frontmatter stores:
  - fileClass settings
  - field definitions for that class
- Files can map to multiple fileClasses.
- If multiple fileClasses define the same field name, a priority order applies
  (frontmatter mapping order has precedence).

## Indexed Path (nested fields)

Metadata Menu identifies nested fields using an **indexedPath**.

Composition:

- Each field has a unique `id`.
- `indexedPath` is built by joining parent `id`s with `____`, including
  list indices in brackets, and ending with the field `id`.

Example (object list nesting):

- Field IDs:
  - `Employees` (ObjectList) id: `dx8Mth`
  - `Name` id: `7r1kwd` (child of Employees)
  - `Contact Info` (Object) id: `Y0dsfZ` (child of Employees)
  - `email` id: `hRlSsW` (child of Contact Info)

- Indexed paths:
  - `dx8Mth[0]____7r1kwd`
  - `dx8Mth[0]____Y0dsfZ____hRlSsW`

This indexedPath is the key for API operations that target nested fields.
Lithos MVP does not support nested fields, but the indexedPath concept remains
relevant for future hierarchical metadata support.

## API Surface (MetadataMenu.api)

Primary use is to read and write metadata programmatically.

- `getValues(fileOrPath, attribute)` (deprecated)
  - Returns all values for a field name.

- `getValuesForIndexedPath(fileOrPath, indexedPath)`
  - Returns the value of a specific indexedPath field instance.

- `postValues(fileOrPath, payload, lineNumber?, after?, asList?, asBlockquote?)`
  - Writes values by indexedPath.
  - If field does not exist:
    - inserted at `lineNumber` if provided,
    - otherwise added to frontmatter.

- `postNamedFieldsValues(fileOrPath, payload, lineNumber?, after?, asList?, asBlockquote?)`
  - Same as postValues but targets by field name instead of indexedPath.

- `fileFields(fileOrPath)`
  - Returns a map of indexedPath -> field info:
    - `value`, `fileClassName`, `ignoreInMenu`, `isValid`, `options`,
      `sourceType`, `type`, `id`, `indexedPath`.

- `namedFileFields(fileOrPath)`
  - Same as fileFields but uses named indexedPath keys.

- `insertMissingFields(fileOrPath, lineNumber, after, asList, asBlockquote, fileClassName?)`
  - Inserts fields that are defined in fileClass but missing in the file.

## Controls and Editing Flow

Fields can be edited via:

- Autocompletion in editor (`:` in frontmatter, `::` for inline fields)
- Context menus (links, file explorer, calendar, note)
- Command palette
- Metadata Menu modal (per-note field editor)
- Dataview table integration (`fieldModifier` API)

Bulk edits:

- FileClass table view and codeblocks allow multi-file editing.

## Type Behaviors (selected highlights)

- **Select / Multi / Cycle**
  - Options can be defined from:
    - a note path (each line as an option)
    - a JS function returning a list
    - settings-managed list

- **Date/DateTime/Time**
  - Uses moment.js formats.
  - Supports shift intervals and cycle-based interval fields.
  - Optionally inserts links to date notes.

- **File/MultiFile**
  - Options from DataviewJS `dv.pages(...)` queries.
  - Supports alias function and custom sort function.

- **Lookup**
  - Executes a DataviewJS query to find pages.
  - Matches by a “related field” to the source note.
  - Writes persistent results into the note (can be published).
  - Supports built-in summaries (Sum/Count/CountAll/Average/Max/Min) or
    custom functions over the DataArray.

- **Formula**
  - JS function over fields; can auto-update on vault changes.

- **Object / Object List**
  - Parent types that enable nested fields; only valid in frontmatter.

## Lithos Alignment Notes

- FileClass is a concrete model for **schema-by-context**.
- IndexedPath provides a stable, hierarchical addressing scheme for nested
  data; Lithos schema module can adopt a similar path identity.
- Field types and options map cleanly to a schema definition system:
  - validation rules
  - UI/editor controls
  - computed/derived fields (Lookup, Formula)
- API methods emphasize **id-based targeting** over name-based targeting,
  which is critical once nesting and duplication exist.
- Metadata Menu includes nested field types (Object/Object List), but Lithos
  MVP is flat-only. Keep nested semantics in mind for later expansions.

## Appendix A: Field Types -> Options and Controls

This table summarizes the main configurable options and the UI controls used
to edit each type.

- Input
  - Options: template tokens `{{...}}` for guided input
  - Controls: field modal (text area), command palette, note modal, dataview

- Boolean
  - Options: none
  - Controls: modal toggle, command palette toggle, dataview toggle

- Number
  - Options: `min`, `max`, `step`
  - Controls: modal input + increment/decrement, dataview in-cell

- Select
  - Options: values from note, JS function, or static list
  - Controls: modal select, inline suggestor, dataview modal

- Multi
  - Options: values from note, JS function, or static list
  - Controls: modal multi-select, inline suggestor (comma), dataview modal

- Cycle
  - Options: values list; `cycle begins by null`
  - Controls: command palette and modal “next value”, inline suggestor

- File / MultiFile
  - Options: DataviewJS `dv.pages(...)` query, alias function, sort function
  - Controls: modal file picker, inline suggestor, dataview modal

- Media / MultiMedia
  - Options: media folders, embed size, list vs cards display
  - Controls: modal picker

- Date / DateTime / Time
  - Options: moment format, shift interval, interval cycle field
  - Date: optional link insertion and link path template
  - Controls: modal date/time picker, shift controls

- Lookup
  - Options: DataviewJS query, related field name, output type
  - Output types: links list, indented list, built-in summaries, custom
  - Controls: update lookup command (if not auto-update)

- Formula
  - Options: JS expression, auto-update on vault change
  - Controls: update command (if not auto-update)

- JSON / YAML
  - Options: none
  - Controls: modal editor

- Object / Object List
  - Options: none (parent type)
  - Controls: modal to edit child fields, add/remove list items

## Appendix B: Metadata Menu -> Lithos Schema Mapping

Suggested conceptual mapping for Lithos schema module design.

- Field Definition
  - Metadata Menu: `{ name, type, id, options, path }`
  - Lithos: `Property` struct (base definition)
  - Notes: Property + Property Specs drive concrete field typing

- Field Type
  - Metadata Menu: `Input`, `Select`, `Object`, etc.
  - Lithos: `PropertySpec` (type + constraints + UI/control hints)

- FileClass
  - Metadata Menu: file-based class with fields + settings
  - Lithos: schema file that assembles Properties + Property Specs

- IndexedPath
  - Metadata Menu: hierarchical `id` path with list indices
  - Lithos: `SchemaName` or `SchemaId` identity for schema references

- Preset Fields
  - Metadata Menu: global settings in `data.json`
  - Lithos: Property Bank (shared Properties for schema reference)

- File Mapping
  - Metadata Menu: fileClass mapping via frontmatter
  - Lithos: schema assignment via frontmatter or config-based mapping rules

- Validation
  - Metadata Menu: per-field validation based on type + options
  - Lithos: raw parse -> `Property` + `PropertySpec` validation

- Lookup / Formula
  - Metadata Menu: computed, persisted fields
  - Lithos: derived field pipeline; explicitly model evaluation phase and
    persistence into projection cache

## Appendix C: Schema Inheritance with `extends` and `excludes`

### Overview

Metadata Menu fileClasses support single-parent inheritance via the `extends` field and selective field exclusion via the `excludes` array. This enables schema reuse and composition without duplication.

**Key Characteristics**:
- Single-parent inheritance (no multiple inheritance)
- Transitive inheritance (grandparent fields inherited)
- Field override by name (child redefines parent field)
- Selective field exclusion from any ancestor
- Cycle detection prevents infinite loops

### Schema File Syntax

```yaml
# physics.md (child fileClass)
---
extends: course           # Single parent reference (fileClass name)
excludes: [grade, fees]   # Array of field names to exclude from ancestors
fields:
  - name: lecture         # Own field definition
    type: Select
    id: abc123
    options:
      - "0": Mechanics
      - "1": Optics
      - "2": Electricity

  - name: type            # Override parent's 'type' field
    type: Select
    id: def456
    options:
      - "0": "at school"
      - "1": "online"
    # Note: parent had 3 options, child has only 2
---

Body content (ignored by plugin)
```

### Inheritance Resolution Algorithm

**Phase 1: Build Ancestor Chain** (one-time, global index)

```
1. For each fileClass, read frontmatter `extends` field
2. Build immediate parent map: fileClass -> parent
3. Recursively expand to full ancestor chain:
   - Start with [parent]
   - Add parent's parent, parent's parent's parent, etc.
   - Stop when cycle detected (ancestor == current fileClass)

Example:
  physics extends course
  course extends base

  Result:
    physics -> [course, base]  # Full chain
    course -> [base]
```

**Phase 2: Resolve Excludes** (per fileClass load)

```
1. Parse `excludes` array from frontmatter (string or array)
2. Build exclusion list from ALL ancestors:
   - For each ancestor in chain:
     - Get ancestor's field definitions
     - If field.name in excludes list:
       - Add to excluded_fields (deduplicate)

Result: Set of FileClassAttribute to exclude
```

**Phase 3: Merge Fields** (per fileClass load)

```
1. Start with empty attributes list
2. Add own fields first (from this fileClass's frontmatter)
3. For each ancestor (in order: parent, grandparent, ...):
   - Get ancestor's field definitions
   - Filter out excluded fields
   - For each ancestor field:
     - If field.name not already in attributes:
       - Add to attributes
     # Note: First occurrence wins (child override)

Result: Merged list of fields for fileClass
```

### Exclude Scope: Any Ancestor

Unlike simple parent-child inheritance, `excludes` filters fields from **any ancestor**, not just the immediate parent.

**Example**:
```yaml
# base.md
fields:
  - name: id
  - name: created_at
  - name: internal_ref

# course.md
extends: base
fields:
  - name: teacher
  - name: grade

# physics.md
extends: course
excludes: [internal_ref, grade]  # Excludes from grandparent AND parent
fields:
  - name: lecture
```

**Result for physics**:
- From grandparent (base): `id`, `created_at` (excludes `internal_ref`)
- From parent (course): `teacher` (excludes `grade`)
- From self (physics): `lecture`
- **Final**: `[id, created_at, teacher, lecture]`

### Field Override Semantics

When a child defines a field with the same name as an ancestor, the child's definition **fully replaces** the parent's definition.

**Override Behavior**:
- Match by field `name` (case-sensitive)
- Child definition used (parent definition ignored)
- No partial merge (e.g., can't keep parent options and add more)
- Type can change (parent `Select` → child `Multi`)

**Example**:
```yaml
# Parent (course.md)
fields:
  - name: status
    type: Select
    options:
      - "0": Active
      - "1": Completed
      - "2": Dropped

# Child (physics.md)
extends: course
fields:
  - name: status      # Same name = override
    type: Multi       # Different type allowed
    options:
      - "0": Active
      - "1": Completed
    # Only 2 options (narrowed from parent)
```

**Result**: Child's `status` field is used (Multi with 2 options). Parent's `status` (Select with 3 options) is ignored.

### Exclude vs Override

Two ways to handle unwanted parent fields:

| Approach | Syntax | Result | Use Case |
|----------|--------|--------|----------|
| **Exclude** | `excludes: [fieldName]` | Field not inherited | Don't want field at all |
| **Override** | Redefine field | Child definition used | Want field with different config |

**Example**:
```yaml
extends: course
excludes: [grade]     # Exclude: don't want 'grade' field
fields:
  - name: teacher     # Override: want 'teacher' but with different options
    type: MultiFile   # Changed from File to MultiFile
    options:
      query: "dv.pages('#instructor')"
```

### Cycle Detection

Cycles are detected during ancestor chain building:

**Algorithm**:
```
while building ancestor chain:
  if next_ancestor == current_fileClass:
    stop recursion  # Cycle detected
```

**Example**:
```yaml
# a.md
extends: b

# b.md
extends: a
```

**Result**:
- a -> [b] (stops when detecting 'a' again)
- b -> [a] (stops when detecting 'b' again)

**Behavior**: Cycle breaks the chain but doesn't error. Each fileClass gets one level of inheritance only.

### Edge Cases

**1. Missing Parent**

```yaml
extends: nonexistent
```

**Behavior**: Treated as root class (no inheritance). No error thrown.

**2. Exclude Non-Existent Field**

```yaml
extends: course
excludes: [nonexistent, grade]
```

**Behavior**: `nonexistent` silently ignored. Only `grade` excluded if it exists.

**3. Deep Inheritance Chain**

```yaml
# specific.md extends medium
# medium.md extends general
# general.md extends base
```

**Behavior**: All ancestors tracked (`specific -> [medium, general, base]`). Excludes filter entire chain.

**4. Multiple FileClasses per Note**

```yaml
# note.md
fileClass: [physics, chemistry]
```

**Behavior**: Both fileClasses contribute fields. If both define same field name, priority determined by array order (first wins).

**5. Exclude + Override Same Field**

```yaml
extends: course
excludes: [status]
fields:
  - name: status  # Define own 'status'
```

**Behavior**: Child's `status` used. Exclude applies to **parent's** status, not child's own definition.

### Implementation Notes

**Ancestor Chain Storage**:
- Built once at plugin startup
- Stored in global index: `Map<fileClassName, ancestorNames[]>`
- O(1) lookup after initial build

**Field Merge Priority**:
1. Child's own fields (first in merge order)
2. Parent's fields (if not excluded, if name not in child)
3. Grandparent's fields (if not excluded, if name not already present)
4. ... (continue up ancestor chain)

**Name Collision Resolution**:
- First occurrence wins
- Child defines first → child's definition used
- If child doesn't define, parent's used
- If parent doesn't define, grandparent's used
- Etc.

**Performance**:
- Ancestor chain: O(n) build per fileClass, O(1) lookup
- Field merge: O(m × a) where m = fields per ancestor, a = ancestor count
- Typical chains are shallow (1-3 levels), so performance is acceptable

### Lithos Implications

**Current Gaps**:
1. ❌ Exclude scope limited to parent only (should extend to all ancestors)
2. ❌ Ancestor chain rebuilt each load (should be cached in RawSchemaView)
3. ❌ Override semantics unclear/untested (should document and validate)
4. ⚠️ Missing parent = error (consider warning mode like Metadata Menu)

**Recommended Changes**:
1. Expand `excludes` to filter any ancestor field (not just parent)
2. Cache ancestor chain in `RawSchemaView.ancestors: Vec<SchemaName>`
3. Document override behavior in ADR (full replace vs partial merge)
4. Add tests for deep inheritance (3+ levels)
5. Add tests for exclude from grandparent
6. Add tests for field override with type change

**Priority**: High - inheritance is core to schema reuse. Metadata Menu's approach is mature and well-tested.
