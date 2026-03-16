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
extends: course           # Single parent reference (fileClass name)
excludes: [grade, fees]   # Array of field names to exclude from ancestors
fields:
  - name: lecture         # Own field definition
    type: Select
    id: abc123
    options:
      - "0": Mechanics
      - "1": Optics

  - name: type            # Override parent's 'type' field
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

```
For each fileClass:
  1. Read frontmatter 'extends' field
  2. Build immediate parent map
  3. Recursively expand to full ancestor chain
  4. Stop when cycle detected (ancestor == current fileClass)

Example:
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
Behavior: Each gets one level (a → [b], b → [a]). Cycle breaks chain.

**Deep Inheritance**
```yaml
# specific → medium → general → base
```
Behavior: All ancestors tracked. Excludes filter entire chain.

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

## Appendix C: Applying Inheritance Patterns to Lithos

This appendix translates MetadataMenu's inheritance model to Lithos implementation.

### Current Lithos Architecture

**What we have**:
- Single-parent inheritance via `extends` field in schema JSON
- `excludes` array for selective property filtering
- `Extender` module builds inheritance tree
- `Resolver` module merges properties
- Property-level conflict resolution

**Where we differ**:
- Exclude scope: parent-only vs any-ancestor (MetadataMenu)
- Ancestor tracking: per-load rebuild vs cached (MetadataMenu)
- Override semantics: unclear vs documented (MetadataMenu)

### Recommended Implementation Changes

#### 1. Expand Exclude Scope (Priority: HIGH)

**Current Behavior**:
```rust
// Resolver only checks immediate parent's excludes
let excluded_from_parent: HashSet<PropertyName> = schema.excludes();
```

**Target Behavior** (MetadataMenu style):
```rust
// Check excludes against ALL ancestors
fn filter_inherited_properties(
    &self,
    schema: &Schema,
    ancestors: &[SchemaId],  // Full chain: [parent, grandparent, ...]
    excludes: &[PropertyName],
) -> Vec<Property> {
    let mut inherited = Vec::new();

    // Walk ancestors in reverse (root first: grandparent → parent)
    for ancestor_id in ancestors.iter().rev() {
        let ancestor_props = self.get_schema_properties(ancestor_id);

        for prop in ancestor_props {
            // Skip if excluded
            if excludes.contains(&prop.name()) {
                continue;
            }

            // Skip if already present (child override)
            if inherited.iter().any(|p| p.name() == prop.name()) {
                continue;
            }

            inherited.push(prop.clone());
        }
    }

    inherited
}
```

**Benefits**:
- More flexible schema composition
- Can exclude grandparent fields without touching parent
- Matches user mental model ("I don't want X from anywhere")

**Estimate**: 2-4 hours (implementation + tests)

#### 2. Cache Ancestor Chains (Priority: HIGH)

**Current Behavior**:
```rust
// Extender::build() creates SchemaTree each load
pub fn build(&self, raw_schemas: Vec<RawSchema>) -> Result<SchemaTree> {
    // Topological sort, cycle detection, etc.
    // Rebuilt every time
}
```

**Target Behavior** (MetadataMenu style):
```rust
// Store ancestors in RawSchemaView (persisted)
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct RawSchemaView {
    file_path: Box<str>,
    extends: Option<SchemaName>,
    excludes: Vec<PropertyName>,
    ancestors: Vec<SchemaName>,  // NEW: Cached full chain
    versions: RingBuffer<RawFileVersion, 5>,
}

// Build once, reuse forever (until parent changes)
fn build_ancestor_chain(schema_name: &SchemaName) -> Vec<SchemaName> {
    let mut chain = Vec::new();
    let mut current = schema_name.clone();

    while let Some(parent) = get_parent(&current) {
        if chain.contains(&parent) {
            break;  // Cycle detected
        }
        chain.push(parent.clone());
        current = parent;
    }

    chain  // [parent, grandparent, great-grandparent, ...]
}
```

**Benefits**:
- O(1) ancestor lookup (vs O(n) tree traversal)
- Enables "find all descendants" queries
- Incremental resolution can check if ancestor changed
- Simpler cycle detection (check if name in ancestors vec)

**Trade-offs**:
- Must invalidate on parent schema changes
- Slightly more storage (Vec<SchemaName> per schema)

**Estimate**: 4-6 hours (implementation + migration + tests)

#### 3. Document Override Semantics (Priority: MEDIUM)

**Create ADR**: "Schema Property Override Behavior"

**Decision Points**:

| Aspect | Options | Recommendation |
|--------|---------|----------------|
| **Override trigger** | Name match, ID match | Name match (simpler, matches MetadataMenu) |
| **Override scope** | Full replacement, Partial merge | Full replacement (predictable) |
| **Type compatibility** | Must match, Can change | Can change (flexibility) |
| **Spec compatibility** | Must match, Can change | Can change (but validate at boundary) |

**Proposed Rules**:
1. Child property with same `PropertyName` fully replaces parent's
2. Type can change (Select → Multi, String → Number, etc.)
3. Spec can change (3 options → 2 options, range narrowing, etc.)
4. No validation at merge time (validate at raw→domain boundary)
5. First occurrence wins (child > parent > grandparent)

**Example**:
```rust
// Parent: status (Select with 3 options)
// Child: status (Multi with 2 options)
// Result: Child's Multi with 2 options
//
// Implementation:
// 1. Collect child properties first
// 2. For each ancestor property:
//    - If name in child properties: skip (override)
//    - If name in excludes: skip
//    - Else: add to merged list
```

**Estimate**: 2-3 hours (ADR writing + discussion)

#### 4. Add Comprehensive Inheritance Tests (Priority: HIGH)

**Test Scenarios** (from MetadataMenu analysis):

```rust
#[test]
fn deep_inheritance_chain() {
    // specific → medium → general → base (3+ levels)
    // Verify property merge order correct
}

#[test]
fn exclude_from_grandparent() {
    // Child excludes property that exists in grandparent (not parent)
    // Verify excluded field not in final schema
}

#[test]
fn override_with_type_change() {
    // Parent has status: Select
    // Child has status: Multi
    // Verify child's type used
}

#[test]
fn exclude_multiple_fields() {
    // excludes: [a, b, c] from various ancestor levels
    // Verify all excluded
}

#[test]
fn exclude_plus_override_same_field() {
    // excludes: [status]
    // fields: [{ name: status, ... }]
    // Verify child's definition used (exclude doesn't apply to own fields)
}

#[test]
fn ancestor_cache_invalidation() {
    // Change parent's parent (grandparent)
    // Verify children pick up new ancestor chain
    // (Requires loader integration test)
}
```

**Estimate**: 2-4 hours (writing + debugging)

#### 5. Lenient Error Handling (Priority: LOW)

**Current Behavior**:
```rust
// Missing parent = error, abort load
let parent = self.resolve_parent(schema_name)?;
```

**Target Behavior** (MetadataMenu style):
```rust
// Missing parent = warning, continue as root
let parent = match self.resolve_parent(schema_name) {
    Ok(p) => Some(p),
    Err(e) => {
        self.emit_warning(format!("Parent not found: {}", e));
        None  // Treat as root class
    }
};
```

**Benefits**:
- More resilient to vault restructuring
- Easier incremental schema development
- Typos don't crash entire load

**Trade-offs**:
- Silent failures harder to debug
- May hide intentional errors

**Recommendation**: Make configurable via setting (strict vs lenient mode)

**Estimate**: 1-2 hours (implementation + config)

### Implementation Priority

**Phase 1: Core Improvements** (Est. 1 day)
1. ✅ Expand exclude scope to all ancestors (2-4h)
2. ✅ Add comprehensive inheritance tests (2-4h)
3. ✅ Document override semantics in ADR (2-3h)

**Phase 2: Performance** (Est. 4-6 hours)
4. ⚠️ Cache ancestor chains in RawSchemaView (4-6h)

**Phase 3: Polish** (Est. 1-2 hours)
5. ❓ Add lenient error handling config (1-2h)

**Total Estimate**: 1.5-2 days

### Code Examples

**Expanding Exclude Scope**:

Location: `lithos-core/src/schema/resolver.rs`

```rust
// Before (current)
fn merge_properties(schema: &Schema, parent: &Schema) -> Vec<Property> {
    let excludes = schema.excludes();
    let mut props = schema.properties().to_vec();

    for parent_prop in parent.properties() {
        if !excludes.contains(&parent_prop.name()) {
            if !props.iter().any(|p| p.name() == parent_prop.name()) {
                props.push(parent_prop.clone());
            }
        }
    }

    props
}

// After (target)
fn merge_properties(
    schema: &Schema,
    ancestors: &[&Schema],  // All ancestors, not just parent
) -> Vec<Property> {
    let excludes = schema.excludes();
    let mut props = schema.properties().to_vec();

    // Walk ancestors in reverse (grandparent → parent)
    for ancestor in ancestors.iter().rev() {
        for ancestor_prop in ancestor.properties() {
            // Skip if excluded from any ancestor
            if excludes.contains(&ancestor_prop.name()) {
                continue;
            }

            // Skip if already present (override)
            if props.iter().any(|p| p.name() == ancestor_prop.name()) {
                continue;
            }

            props.push(ancestor_prop.clone());
        }
    }

    props
}
```

**Caching Ancestors**:

Location: `lithos-core/src/schema/views/raw.rs`

```rust
// Add to RawSchemaView
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct RawSchemaView {
    file_path: Box<str>,
    extends: Option<SchemaName>,
    excludes: Vec<PropertyName>,
    ancestors: Vec<SchemaName>,  // NEW
    versions: RingBuffer<RawFileVersion, 5>,
}

impl RawSchemaView {
    /// Get full ancestor chain (cached)
    pub fn ancestors(&self) -> &[SchemaName] {
        &self.ancestors
    }

    /// Check if ancestors are stale (parent changed)
    pub fn ancestors_stale(&self, parent_schema: &RawSchemaView) -> bool {
        // If parent's ancestors changed, this schema's ancestors are stale
        if let Some(parent_name) = &self.extends {
            if let Some(first_ancestor) = self.ancestors.first() {
                if first_ancestor != parent_name {
                    return true;  // Parent changed
                }
            }

            // Check if parent's chain changed
            let expected = [parent_name.as_ref()]
                .iter()
                .chain(parent_schema.ancestors().iter())
                .cloned()
                .collect::<Vec<_>>();

            self.ancestors != expected
        } else {
            false  // No parent, never stale
        }
    }
}
```

### Migration Path

1. **Week 1**: Implement expanded exclude scope + tests
2. **Week 2**: Add ancestor caching to RawSchemaView
3. **Week 3**: Write ADR, add lenient error handling
4. **Week 4**: Integration testing, performance benchmarks

**Rollout Strategy**:
- Phase 1 changes are backwards compatible (exclude scope expansion)
- Phase 2 requires RawSchemaView migration (add ancestors field)
- Phase 3 is opt-in (config flag for lenient mode)

### Success Criteria

- [ ] Exclude scope covers all ancestors (not just parent)
- [ ] Ancestor chains cached in RawSchemaView
- [ ] All inheritance tests passing (6+ scenarios)
- [ ] ADR documented override semantics
- [ ] Performance: ancestor lookup O(1) (vs O(n) tree walk)
- [ ] Compatibility: existing schemas load without changes
