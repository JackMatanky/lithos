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

## Schema Inheritance with `extends` and `excludes`

FileClasses support single-parent inheritance via the `extends` field and selective field exclusion via the `excludes` array.

### Basic Syntax

```yaml
# physics.md (child fileClass)
---
extends: course # Single parent reference (fileClass name)
excludes: [grade, fees] # Array of field names to exclude from ancestors
fields:
  - name: lecture # Own field definition
    type: Select
    id: abc123
    options:
      - "0": Mechanics
      - "1": Optics

  - name: type # Override parent's 'type' field
    type: Select
    id: def456
    options:
      - "0": "at school"
      - "1": "online"
---
```

### Key Characteristics

- **Single-parent inheritance** - No multiple inheritance
- **Transitive inheritance** - Grandparent fields inherited automatically
- **Field override by name** - Child redefines parent field completely
- **Exclude scope: any ancestor** - Can exclude fields from parent, grandparent, etc.
- **Cycle detection** - Prevents infinite loops in inheritance chains

### Inheritance Resolution

**Phase 1: Build Ancestor Chain** (one-time, global)

For each fileClass:
  1. Read frontmatter 'extends' field
  2. Build immediate parent map
  3. Recursively expand to full ancestor chain
  4. Stop when cycle detected (ancestor == current fileClass)

Example:
```
physics extends course
course extends base

Result:
  physics -> [course, base]  # Full chain
  course -> [base]
```

**Phase 2: Resolve Excludes** (per fileClass load)

```
1. Parse 'excludes' array from frontmatter
2. Build exclusion list from ALL ancestors:
   - For each ancestor in chain
   - Get ancestor's field definitions
   - If field.name in excludes: add to excluded list

Key: Excludes filter any ancestor, not just immediate parent
```

**Phase 3: Merge Fields** (per fileClass load)

```
1. Start with own fields (child's definitions)
2. For each ancestor (parent, grandparent, ...):
   - Get ancestor's fields
   - Filter out excluded fields
   - Add fields not already present (by name)

Result: Child fields win, then parent, then grandparent, etc.
```

### Exclude Scope Example

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
- From self: `lecture`
- **Final fields**: `[id, created_at, teacher, lecture]`

### Field Override Semantics

When a child defines a field with the same name as an ancestor, the child's definition **fully replaces** the parent's.

**Override behavior**:

- Match by field `name` (case-sensitive)
- Child definition used, parent definition ignored
- No partial merge (can't keep parent options and add more)
- Type can change (parent `Select` → child `Multi`)
- Options can narrow (parent has 3 choices, child has 2)

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
  - name: status      # Same name = full override
    type: Multi       # Different type allowed
    options:
      - "0": Active
      - "1": Completed
    # Only 2 options (narrowed from parent)
```

**Result**: Child's `status` used (Multi with 2 options). Parent's `status` ignored.

### Exclude vs Override

Two ways to handle unwanted parent fields:

| Approach     | Syntax                  | Result                | Use Case                         |
| ------------ | ----------------------- | --------------------- | -------------------------------- |
| **Exclude**  | `excludes: [fieldName]` | Field not inherited   | Don't want field at all          |
| **Override** | Redefine field          | Child definition used | Want field with different config |

**Example**:

```yaml
extends: course
excludes: [grade] # Exclude: don't want 'grade' field
fields:
  - name: teacher # Override: want 'teacher' but with different options
    type: MultiFile # Changed from File to MultiFile
    options:
      query: "dv.pages('#instructor')"
```

### Edge Cases

**Missing Parent**

```yaml
extends: nonexistent
```

Behavior: Treated as root class (no inheritance). No error.

**Exclude Non-Existent Field**

```yaml
excludes: [nonexistent, grade]
```

Behavior: `nonexistent` silently ignored. Only `grade` excluded if exists.

**Cycle Detection**

```yaml
# a.md extends b
# b.md extends a
```

Behavior: Each gets one level `(a → [b], b → [a])`. Cycle breaks chain.

**Deep Inheritance**

```yaml
# specific → medium → general → base
```

Behavior: All ancestors tracked. Excludes filter entire chain.

**Multiple FileClasses per Note**

```yaml
# note.md
fileClass: [physics, chemistry]
```

Behavior: Both fileClasses contribute fields. If both define same field name, priority determined by array order (first wins).

**Exclude + Override Same Field**

```yaml
extends: course
excludes: [status]
fields:
  - name: status # Define own 'status'
```

Behavior: Child's `status` used. Exclude applies to **parent's** status, not child's own definition.

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

**Performance**:

- Ancestor chain: O(n) build per fileClass, O(1) lookup
- Field merge: O(m × a) where m = fields per ancestor, a = ancestor count
- Typical chains are shallow (1-3 levels), so performance is acceptable

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

## Appendix C: Lithos vs MetadataMenu Inheritance Comparison

**See**: `INHERITANCE_COMPREHENSIVE_ANALYSIS.md` for detailed source code analysis.

This appendix compares MetadataMenu's inheritance implementation with Lithos to understand design trade-offs and identify optimization opportunities.

### Summary of Findings (2026-03-16)

After thorough source code review of both systems:

**✅ Lithos Already Supports**:

- Excludes filtering properties from **all ancestors** (grandparent, great-grandparent, etc.)
- Topological resolution ensures parent properties include full inheritance chain
- Property override by name match
- Cycle detection (actually MORE robust than MetadataMenu - uses DFS + Kahn's algorithm)
- Depth tracking with max limit (10 levels - MetadataMenu lacks this)

**❌ Key Difference**:

- **Ancestor caching**: MetadataMenu caches `fileClassesAncestors` map globally; Lithos rebuilds tree each load

### Architecture Comparison

| Feature                | MetadataMenu            | Lithos                   | Winner                      |
| ---------------------- | ----------------------- | ------------------------ | --------------------------- |
| **Exclude Scope**      | ✅ All ancestors        | ✅ All ancestors         | TIE                         |
| **Cycle Detection**    | ⚠️ Name comparison only | ✅ DFS + Kahn's          | **Lithos**                  |
| **Depth Tracking**     | ❌ Not explicit         | ✅ NodeDepth type        | **Lithos**                  |
| **Depth Limit**        | ❌ No limit             | ✅ Max 10 levels         | **Lithos**                  |
| **Ancestor Caching**   | ✅ Global map           | ❌ Rebuild each load     | **MetadataMenu**            |
| **Merge Algorithm**    | Name-based filter loop  | Two-pointer sorted merge | **Lithos** (more efficient) |
| **Override Semantics** | Full replacement        | Full replacement         | TIE                         |

### How Lithos Achieves Ancestor Excludes (Without Explicit Ancestor Cache)

**Key Insight**: Topological ordering + resolution caching makes explicit ancestor tracking unnecessary for correctness.

**Example hierarchy**: `base → course → physics`

- base properties: `[id, created_at, internal_ref]`
- course properties: `[title]`
- physics excludes: `[internal_ref]` ← from grandparent!

**Resolution order** (via Kahn's algorithm):

1. **base** (depth 1):

   ```rust
   parent_props = []  // No parent
   merged = [id, created_at, internal_ref]
   resolved_cache["base"] = Schema { properties: [id, created_at, internal_ref] }
   ```

2. **course** (depth 2):

   ```rust
   parent_props = resolved_cache["base"].properties  // ← Full chain already here
   parent_props = [id, created_at, internal_ref]
   merged = [created_at, id, internal_ref, title]  // Sorted merge
   resolved_cache["course"] = Schema { properties: [created_at, id, internal_ref, title] }
   ```

3. **physics** (depth 3):
   ```rust
   parent_props = resolved_cache["course"].properties  // ← Includes grandparent!
   parent_props = [created_at, id, internal_ref, title]
   excludes = ["internal_ref"]
   merged = merge_properties(parent_props, [], excludes)
   // Result: [created_at, id, title]  ✅ internal_ref excluded!
   ```

**Why this works**:

- Parent's resolved properties **already contain** everything from ancestors
- Topological order guarantees parent processed before child
- `merge_properties()` filters against full inherited set

### MetadataMenu's Approach: Explicit Ancestor Caching

**File**: `src/index/FieldIndex.ts`

```typescript
// Global cache built once per index
fileClassesAncestors: Map<string, string[]>
// Example: "physics" → ["course", "base"]

// Phase 1: Init with immediate parent
for each fileClass:
  parent = frontmatter.extends
  if parent exists:
    ancestors[fileClass] = [parent]

// Phase 2: Recursive expansion
function getAncestorsRecursively(fileClassName):
  ancestors = ancestors[fileClassName]
  lastAncestor = ancestors.last()
  lastAncestorParent = ancestors[lastAncestor][0]

  if lastAncestorParent && lastAncestorParent != fileClassName:  // Cycle check
    ancestors[fileClassName].push(lastAncestorParent)
    getAncestorsRecursively(fileClassName)  // Continue up chain
```

**Benefits**:

- O(1) ancestor lookup (vs O(depth) for topological walk)
- Enables "find all descendants" queries
- Incremental resolution can check if ancestors changed
- Built once, reused for all resolutions

**Trade-offs**:

- Must invalidate when parent changes
- Extra storage (~8-40 bytes per schema for typical 1-5 ancestor chains)
- More complex cache invalidation logic

### Optimization Opportunity: Ancestor Caching

**Status**: Not yet implemented; under consideration for performance optimization.

**Trade-off Analysis**: See `INHERITANCE_COMPREHENSIVE_ANALYSIS.md` Part 6 for detailed comparison.

**Current Lithos Approach** (Topological Resolution):

- Rebuild `SchemaTree` on each load via `Extender::build()`
- Benefits: Simpler, no cache invalidation, always fresh
- Drawbacks: O(depth) for ancestor chain walk, redundant computation

**MetadataMenu Approach** (Cached Ancestors):

- Store `ancestors: Vec<SchemaName>` globally
- Build once, reuse until parent changes
- Benefits: O(1) lookup, supports "find descendants" queries
- Drawbacks: Cache invalidation complexity, staleness tracking

**Potential Implementation** (if performance becomes issue):

```rust
// In RawSchemaView (persisted):
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct RawSchemaView {
    extends: Option<SchemaName>,
    excludes: Vec<PropertyName>,
    ancestors: Vec<SchemaName>,  // NEW: [parent, grandparent, ...]
    ancestors_hash: u64,          // NEW: Hash of parent chain for staleness check
}

// In Loader:
fn is_ancestors_stale(&self, schema: &RawSchemaView) -> bool {
    let parent = schema.extends.as_ref()?;
    let parent_view = self.repo.find_raw_schema_by_name(parent)?;

    // Check if parent's chain changed
    let current_hash = compute_ancestor_hash(parent, &parent_view.ancestors);
    current_hash != schema.ancestors_hash
}
```

**Recommendation**: **Defer until profiling shows need** (premature optimization).

For typical vaults (<1000 schemas, <5 depth), topological resolution overhead is negligible (<10ms).

### Override Semantics (Documented)

**Current Lithos Behavior** (matches MetadataMenu):

| Aspect                 | Lithos                       | MetadataMenu     | Notes                                       |
| ---------------------- | ---------------------------- | ---------------- | ------------------------------------------- |
| **Override trigger**   | Name match                   | Name match       | Case-sensitive `PropertyName` comparison    |
| **Override scope**     | Full replacement             | Full replacement | Child property completely replaces parent   |
| **Type compatibility** | Can change                   | Can change       | No type constraint; child can redefine spec |
| **Multiplicity**       | Can change                   | Can change       | Optional → Required, Single → Multi allowed |
| **Priority**           | Child > Parent > Grandparent | Same             | First occurrence by topological order wins  |

**Implementation** (from `resolver.rs:merge_properties`):

```rust
match p.name().cmp(c.name()) {
    Ordering::Equal => {
        // Same name: child overrides parent
        result.push(c.clone());  // ← Child wins, parent discarded
        p_iter.next();
        c_iter.next();
    }
}
```

**Rules**:

1. **Child property with same `PropertyName` fully replaces parent's property**
2. Type can change (Select → Multi, String → Number, etc.)
3. Spec can change (3 options → 2 options, range narrowing, etc.)
4. No validation at merge time (validate at raw→domain boundary)
5. First occurrence wins (child > parent > grandparent)

### Test Coverage (Added 2026-03-16)

**Status**: ✅ Comprehensive inheritance tests added to `resolver.rs`

Tests verifying exclude scope across ancestors:

1. ✅ `exclude_grandparent_property()` - Excludes from 3-level chain (base → course → physics)
2. ✅ `exclude_great_grandparent_property()` - Excludes from 4-level chain
3. ✅ `mixed_excludes_at_multiple_levels()` - Multiple schemas with different excludes
4. ✅ `child_override_beats_parent()` - Name-based override verification
5. ✅ `inheritance_depth_limit_exceeded()` - Max depth enforcement (11 levels rejected)

**All tests passing** - confirms Lithos correctly handles ancestor excludes without explicit caching.

### MetadataMenu's Ancestor Caching Strategy

**Context**: MetadataMenu caches ancestor chains in a global map to avoid rebuilding inheritance trees on every resolution.

**Implementation** (`src/index/FieldIndex.ts`):

```typescript
// Global cache: schema name → ancestor list
fileClassesAncestors: Map<string, string[]>
// Example: "physics" → ["course", "base"]

// Build process (two phases):
// 1. Initialize with immediate parent from frontmatter
// 2. Recursively expand to include full chain

function getAncestorsRecursively(fileClassName):
  ancestors = ancestors[fileClassName]
  lastAncestor = ancestors.last()
  lastAncestorParent = ancestors[lastAncestor][0]

  if lastAncestorParent && lastAncestorParent != fileClassName:  // Cycle check
    ancestors[fileClassName].push(lastAncestorParent)
    getAncestorsRecursively(fileClassName)
```

**Characteristics**:

- **Performance**: O(1) ancestor lookup vs O(depth) for tree walk
- **Staleness**: Must invalidate when parent's `extends` field changes
- **Storage**: ~8-40 bytes per schema (typical 1-5 ancestor chains)
- **Complexity**: Requires cache invalidation logic

### Lithos Current Approach vs MetadataMenu

| Aspect                 | Lithos                        | MetadataMenu                |
| ---------------------- | ----------------------------- | --------------------------- |
| **Strategy**           | Topological resolution        | Cached ancestor map         |
| **Rebuild frequency**  | Each load                     | Once (until parent changes) |
| **Complexity**         | Lower (no cache invalidation) | Higher (staleness tracking) |
| **Performance**        | O(depth) per schema           | O(1) per schema             |
| **Correctness**        | Always fresh                  | Requires invalidation logic |
| **Memory**             | Transient tree                | Persistent map              |
| **Cycle detection**    | ✅ DFS + Kahn's algorithm     | ⚠️ Name comparison only     |
| **Depth limit**        | ✅ Max 10 levels              | ❌ No explicit limit        |

### Error Handling Differences

**MetadataMenu approach**:

```typescript
// Missing parent = warning, continue as root
if (!parentClass) {
  console.warn(`Parent class not found: ${extends}`);
  return null;  // Treat as root
}
```

**Lithos approach**:

```rust
// Missing parent = error, abort load
let parent = self.resolve_parent(schema_name)?;
```

**Trade-offs**:

| Approach | Benefits | Drawbacks |
|----------|----------|-----------|
| **Lenient** (MetadataMenu) | Resilient to typos, easier incremental development | Silent failures, may hide errors |
| **Strict** (Lithos) | Explicit errors, guaranteed consistency | Requires correct parent references upfront |

### Summary of Key Findings

**✅ Lithos Already Implements** (verified via tests):

1. Excludes filtering properties from **all ancestors** (grandparent, great-grandparent, etc.)
2. Property override by name match (child wins)
3. Topological ordering ensures correct resolution order
4. Cycle detection (more robust - uses DFS + Kahn's algorithm vs name comparison)
5. Depth tracking with max limit (10 levels - MetadataMenu lacks this)

**❌ Key Difference**:

- **Ancestor caching**: MetadataMenu caches `fileClassesAncestors` map globally; Lithos rebuilds tree each load

**Performance consideration**: For typical vaults (<1000 schemas, <5 depth), topological resolution overhead is negligible (<10ms). Caching becomes beneficial for larger vaults or frequent resolution cycles.
