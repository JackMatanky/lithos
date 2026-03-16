# MetadataMenu Extends/Excludes Analysis

## Executive Summary

MetadataMenu (Obsidian plugin) implements a mature schema inheritance system with `extends` and `excludes` that we can learn from for Lithos. Key insights:

1. **Ancestor chain resolution**: Builds complete ancestor list recursively with cycle detection
2. **Excludes accumulate**: Each level can exclude fields from ANY ancestor (not just parent)
3. **Field override by name**: Child can redefine inherited fields completely
4. **Priority-based field merging**: Clear conflict resolution order
5. **Lazy ancestor calculation**: Builds ancestry map once, reuses for all queries

## Architecture Overview

### Data Model

```typescript
// FileClass structure
class FileClass {
  name: string;
  attributes: FileClassAttribute[]; // Own + inherited fields
  options: {
    parent?: FileClass; // Single parent (no multiple inheritance)
    excludes?: FileClassAttribute[]; // Fields to exclude from ancestors
    // ... other options
  };
}

// Ancestry tracking (global index)
fileClassesAncestors: Map<string, string[]>;
// Example: "physics" -> ["course", "base"]  (ordered: immediate parent first)
```

### Frontmatter Syntax

```yaml
# physics.md fileClass
---
extends: course # Single parent reference
excludes: [grade, fees] # Array of field names to exclude
fields:
  - name: lecture
    type: Select
    options: ["Mechanics", "Optics"]
  - name: type # Override parent's 'type' field
    type: Select
    options: ["at school", "online"] # Fewer options than parent
---
```

## Core Resolution Algorithm

### 1. Build Ancestor Chain (One-Time, Global)

**Location**: `FieldIndex.ts` - `buildFileClassesAncestors()`

**Algorithm**:

```typescript
// Phase 1: Initialize with immediate parent
for each fileClass:
  parent = frontmatter.extends
  if parent_file_exists:
    ancestors[fileClass] = [parent]
  else:
    ancestors[fileClass] = []

// Phase 2: Recursively expand to full chain
for each fileClass with ancestors:
  getAncestorsRecursively(fileClass)

function getAncestorsRecursively(fileClassName):
  ancestors = ancestors[fileClassName]
  if ancestors.length > 0:
    lastAncestor = ancestors.last()
    lastAncestorParent = ancestors[lastAncestor][0]  // Get immediate parent

    if lastAncestorParent && lastAncestorParent != fileClassName:  // Cycle check
      ancestors[fileClassName].push(lastAncestorParent)
      getAncestorsRecursively(fileClassName)  // Continue up the chain
```

**Example**:

```
physics extends course
course extends base

After phase 1:
  physics -> [course]
  course -> [base]

After phase 2 (recursive expansion):
  physics -> [course, base]  // Full chain
  course -> [base]
```

**Key Insight**: Cycle detection via simple check: `lastAncestorParent !== fileClassName`

### 2. Resolve Excludes (Per FileClass Load)

**Location**: `fileClass.ts` - `getFileClassOptions()`

**Algorithm**:

```typescript
// Step 1: Parse excludes from frontmatter
excludedNames = getExcludedFieldsFromFrontmatter(frontmatter.excludes)
// Handles: string ("field1,field2") or array (["field1", "field2"])

// Step 2: Build excludes list from ALL ancestors
excludes = []
for each ancestorName in ancestors[this.fileClass]:
  for each attribute in fileClass[ancestorName].attributes:
    if attribute.name in excludedNames:
      if attribute.name not in excludes:  // Deduplicate
        excludes.push(attribute)

// Result: FileClassAttribute[] of excluded fields
```

**Key Insight**: Excludes apply to ANY ancestor, not just immediate parent. The child's `excludes` array filters fields from the entire inheritance chain.

### 3. Merge Attributes (Per FileClass Load)

**Location**: `fileClass.ts` - `buildAttributes()`

**Algorithm**:

```typescript
// Step 1: Load excludes list (from step 2 above)
excludedFields = getExcludedFieldsFromFrontmatter(frontmatter.excludes)

// Step 2: Build map of ancestor attributes
ancestorsAttributes = Map<fileClassName, attributes[]>

// Add own attributes first
ancestorsAttributes.set(this.name,
  getFileClassAttributes(this, excludedFields))

// Add ancestor attributes
for each ancestorName in ancestors:
  ancestorExcludes = frontmatter[ancestor].excludes  // Cascade excludes
  excludedFields.push(...ancestorExcludes)

  ancestorsAttributes.set(ancestorName,
    getFileClassAttributes(ancestor, excludedFields))

// Step 3: Merge with priority (own fields first = override)
this.attributes = []
for each [fileClassName, fileClassAttributes] in ancestorsAttributes:
  for each attr in fileClassAttributes:
    if attr.name not in this.attributes.map(a => a.name):  // Name-based deduplication
      this.attributes.push(attr)
```

**Key Insights**:

1. **Own fields first** = they override inherited fields with same name
2. **Excludes cascade down**: Each level's excludes filter all descendants
3. **Name-based merge**: Simple deduplication by field name (not by ID or type)
4. **No field ID conflicts**: Fields are identified by name, not UUID

### 4. Field Priority Resolution

When multiple fileClasses are mapped to same note:

**Location**: Documentation - "File mapping"

**Priority Order** (highest to lowest):

1. `fileClass` in frontmatter (explicit)
2. Tag match
3. Path match
4. Bookmark group match
5. Query match
6. Global fileClass
7. Preset fields (plugin settings)

**Key Insight**: Explicit wins over implicit. Multiple fileClasses can contribute fields (additive), but conflicts resolved by priority.

## Edge Cases Handled

### 1. Circular Inheritance

**Detection**: `lastAncestorParent !== fileClassName`

**Behavior**: Stops recursion when cycle detected

**Example**:

```
A extends B
B extends A

Result:
  A -> [B]      // Stops when detecting A again
  B -> [A]      // Stops when detecting B again
```

**Lithos Implication**: We already have cycle detection in `Extender::build()` - verify it handles this case.

### 2. Missing Parent

**Detection**: Check if parent file exists

**Behavior**: Set ancestors to empty array `[]`, continue without parent

**Example**:

```yaml
extends: nonexistent
```

Result: Treated as root class (no inheritance)

**Lithos Implication**: We return error in loader. Consider: should missing parent be warning instead?

### 3. Exclude Non-Existent Field

**Behavior**: Silently ignored (no error)

**Algorithm**: Filter only matches existing ancestor fields

**Example**:

```yaml
extends: course
excludes: [nonexistent, grade] # Only 'grade' excluded if exists
```

**Lithos Implication**: Current implementation errors on invalid excludes. Consider: should we be lenient?

### 4. Override With Different Type

**Behavior**: Fully replaces field definition (not merge)

**Example**:

```yaml
# Parent (course)
fields:
  - name: type
    type: MultiSelect
    options: ["A", "B", "C", "D"]

# Child (physics)
fields:
  - name: type
    type: Select        # Different type!
    options: ["A", "B"]  # Subset of options
```

Result: Child's definition completely replaces parent's.

**Lithos Implication**: We need to decide:

- Allow type changes? (Current: probably not validated)
- Allow option narrowing? (Current: we support excludes for properties)
- Full replace or merge semantics?

### 5. Deep Inheritance Chain

**Example**:

```
specific extends medium
medium extends general
general extends base
```

**Behavior**: All ancestors tracked, excludes filter entire chain

**Performance**: O(n) ancestor lookup after initial build (map lookup)

**Lithos Implication**: Our `Extender` builds tree but doesn't persist ancestor list. Consider: should we cache this?

## Differences from Lithos Schema System

| Aspect                 | MetadataMenu                        | Lithos (Current)                        |
| ---------------------- | ----------------------------------- | --------------------------------------- |
| **Inheritance**        | Single parent via `extends`         | Single parent via `extends` field       |
| **Excludes**           | Array of field names                | Array of `PropertyName`                 |
| **Exclude scope**      | Any ancestor field                  | Only parent fields                      |
| **Ancestor tracking**  | Persistent map in index             | Built per-load in `Extender`            |
| **Field merge**        | Name-based deduplication            | ID-based with rkyv                      |
| **Override semantics** | Full replacement by name            | Not explicitly handled                  |
| **Missing parent**     | Warning, continue                   | Error, abort load                       |
| **Circular detection** | Name-based recursion check          | DAG validation in `Extender`            |
| **Multiple parents**   | No                                  | No                                      |
| **Field priority**     | First occurrence wins (child first) | Schema inheritance + property bank refs |

## Recommendations for Lithos

### 1. **Expand Exclude Scope** (High Priority)

**Current**: Excludes only filter parent fields
**Proposed**: Excludes filter any ancestor field

**Implementation**:

```rust
// In Resolver::resolve_schema()
fn filter_inherited_properties(
    &self,
    schema: &Schema,
    ancestors: &[SchemaId],  // Full chain, not just parent
    excludes: &[PropertyName],
) -> Vec<Property> {
    let mut inherited = Vec::new();

    // Walk ancestors in reverse (root first)
    for ancestor_id in ancestors.iter().rev() {
        let ancestor_props = self.get_schema_properties(ancestor_id);
        for prop in ancestor_props {
            if !excludes.contains(&prop.name()) {
                // Merge if not already present (child override)
                if !inherited.iter().any(|p| p.name() == prop.name()) {
                    inherited.push(prop.clone());
                }
            }
        }
    }

    inherited
}
```

**Benefits**:

- More flexible schema design
- Can exclude fields from any level (grandparent, etc.)
- Matches user mental model

### 2. **Cache Ancestor Chains** (Medium Priority)

**Current**: `Extender::build()` creates tree each load
**Proposed**: Persist ancestor chains in `RawSchemaView`

**Implementation**:

```rust
// In views/raw.rs
pub struct RawSchemaView {
    file_path: Box<str>,
    extends: Option<SchemaName>,
    excludes: Vec<PropertyName>,
    ancestors: Vec<SchemaName>,  // NEW: Full chain
    versions: RingBuffer<RawFileVersion, 5>,
}
```

**Benefits**:

- O(1) ancestor lookup for incremental resolution
- Easier to detect cycles (check if name in ancestors)
- Supports "affected schemas" query (find all descendants)

**Trade-offs**:

- Must invalidate on parent schema changes
- Slightly more storage (ancestor list per schema)

### 3. **Explicit Override Validation** (Low Priority)

**Current**: Unclear if child can change property type/spec
**Proposed**: Validate override compatibility

**Options**:

```rust
enum OverridePolicy {
    FullReplace,    // MetadataMenu approach - any change allowed
    TypeMatch,      // Type must match, can change options
    SpecMatch,      // Type and spec structure must match
}
```

**Recommendation**: Start with `FullReplace` (most flexible), add validation later if needed.

### 4. **Lenient Parent Resolution** (Low Priority)

**Current**: Missing parent = error, abort load
**Proposed**: Missing parent = warning, continue as root class

**Implementation**:

```rust
// In Loader::load()
match self.resolve_parent(schema_name) {
    Ok(parent) => parent,
    Err(e) => {
        self.emit_warning(format!("Parent not found: {}", e));
        None  // Continue without parent
    }
}
```

**Benefits**:

- More resilient to vault restructuring
- Easier incremental schema development
- Matches MetadataMenu behavior

**Trade-offs**:

- Silent failures harder to debug
- May hide typos in parent names

### 5. **Exclude Cascade** (Optional)

**Current**: Each schema's excludes only applies to itself
**Proposed**: Excludes cascade to children

**Example**:

```yaml
# course.yaml
extends: base
excludes: [internal_id]  # Exclude from course AND all children

# physics.yaml
extends: course
# Automatically excludes 'internal_id' too
```

**MetadataMenu Approach**: Reads each level's excludes during merge

**Recommendation**: Nice-to-have, but adds complexity. Defer unless requested by users.

## Testing Implications

Based on MetadataMenu's approach, we should add tests for:

1. **Deep inheritance** (3+ levels)
   - Schema A extends B extends C extends D
   - Verify property merge order

2. **Exclude from grandparent**
   - Child excludes property from grandparent (not just parent)
   - Verify excluded field not in final schema

3. **Override with different type**
   - Parent has `status: Select`, child has `status: MultiSelect`
   - Verify child's definition wins

4. **Multiple excludes**
   - Exclude multiple fields: `excludes: [a, b, c]`
   - Verify all excluded

5. **Exclude + Override**
   - Exclude parent field `x`, but define own field `x`
   - Verify child's `x` is used (exclude doesn't apply to own fields)

6. **Ancestor cache invalidation**
   - Change parent's parent (grandparent)
   - Verify children pick up new ancestor chain

## Implementation Priority

### Phase 1: Core Improvements (2-3 days)

1. ✅ Ancestor chain tracking in `RawSchemaView`
2. ✅ Expand exclude scope to all ancestors
3. ✅ Add deep inheritance tests

### Phase 2: Robustness (1-2 days)

4. ⚠️ Lenient parent resolution (warning vs error)
5. ⚠️ Add override validation tests
6. ⚠️ Document override semantics in ADR

### Phase 3: Nice-to-Have (Optional)

7. ❓ Exclude cascade to children
8. ❓ Override policy configuration
9. ❓ Performance benchmarks for deep chains

## Conclusion

MetadataMenu's inheritance system is mature and well-tested. Key learnings:

1. **Cache ancestor chains** - Don't recompute every time
2. **Excludes apply broadly** - Filter any ancestor, not just parent
3. **Name-based merge** - Simple and predictable
4. **Lenient resolution** - Warn on issues, don't fail

Our current system is solid but could benefit from:

- Expanded exclude scope (ancestors, not just parent)
- Cached ancestor chains for performance
- More comprehensive tests for edge cases

**Next Step**: Update `REFACTOR_PLAN_SCHEMA_AUDIT.md` with these findings and add to implementation checklist.
