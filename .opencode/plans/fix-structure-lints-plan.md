# Fix Plan: structure.rs linter warnings

## Overview

Two types of warnings to fix in `structure.rs`:

| Type | Location | Severity | Fix Complexity |
|------|----------|----------|--------------|
| Field visibility | Lines 60, 62 | Warning | Refactor |
| Arithmetic | Lines 502, 523 | **Error** | Quick |

The **arithmetic errors are blocking** (prevent compilation). The field visibility warnings are not blocking but should be addressed.

---

## Part A: Arithmetic Errors (Quick Fix) - BLOCKING

### Problem

```
error: arithmetic operation that can potentially result in unexpected side-effects
  --> lithos-core/src/note/parser/structure.rs:502:54
  --> lithos-core/src/note/parser/structure.rs:523:54
```

Lines 502 and 523 have `depth + 1` which can overflow for large depth values (u32).

### Fix

Replace `depth + 1` with `depth.saturating_add(1)`:

```rust
// Line 502: change from:
Self::walk_block(child, visitor, depth + 1);
// to:
Self::walk_block(child, visitor, depth.saturating_add(1));

// Line 523: change from:
Self::walk_block(child, visitor, depth + 1);
// to:
Self::walk_block(child, visitor, depth.saturating_add(1));
```

### Risk

- **Low**: Saturating arithmetic is the correct behavior for depth tracking
- No semantic change for normal depth values (0-1000)
- Precedent: This is standard Rust idiom for overflow-safe increment

---

## Part B: Field Visibility (Refactor - DEFERRED)

### Problem

```
warning: scoped visibility modifier on a field
  --> lithos-core/src/note/parser/structure.rs:60:5
60 |     pub(crate) kind: BlockKind<'source>,
   |     ^^^^^^^^^^
   = help: consider making the field private and adding a scoped visibility method for it

warning: scoped visibility modifier on a field
  --> lithos-core/src/note/parser/structure.rs:62:5
62 |     pub(crate) span: SourceByteRange,
```

### Current State

```rust
pub(crate) struct Block<'source> {
    pub(crate) kind: BlockKind<'source>,      // Line 60
    pub(crate) span: SourceByteRange,       // Line 62
}
```

Clippy recommends: Make fields private, add accessor methods.

### Fix (Deferred - Requires Review)

1. Change fields to private:
```rust
pub(crate) struct Block<'source> {
    kind: BlockKind<'source>,
    span: SourceByteRange,
}
```

2. Add accessor methods in `impl Block<'source>`:
```rust
/// Returns the block kind (type and content)
#[inline]
pub(crate) fn kind(&self) -> &BlockKind<'source> {
    &self.kind
}

/// Returns the source byte span
#[inline]
pub(crate) fn span(&self) -> &SourceByteRange {
    &self.span
}
```

3. Update all internal usages (grep shows 3 patterns: lines 408, 469, 828)

### Risk Analysis

- **Medium**: Need to verify all internal usages work with borrow
- **Breaking**: If any external code (tests) uses `block.kind` directly, will need updating
- Current code already uses `&block.kind` pattern correctly (see line 469)

---

## Verification

After arithmetic fix:
```bash
cargo clippy --lib -p lithos-core 2>&1 | grep -E "^error:"
# Should return empty or only the field visibility warnings
```

---

## Recommendation

**Fix the arithmetic errors (lines 502, 523) immediately** - they are blocking.

**Defer the field visibility refactor (lines 60, 62)** until next review cycle, because:
1. It's not blocking
2. Requires careful review of all usages
3. The accessor methods need to return borrowed references, which may have downstream implications
