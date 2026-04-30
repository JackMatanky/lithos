# Fix Plan: visitor.rs `expect` attributes without reasons

## Problem

Clippy is failing on 9 instances of `#[expect(unused_variables)]` in `visitor.rs` because the attributes don't specify a `reason`.

All occurrences are on **trait method definitions** in the `BlockVisitor` trait. These are method signatures in the trait definition, not implementations, so by design they may not "use" all parameters.

## Lint Rule

Clippy requires `#[expect(...)]` to include a `reason = "..."` clause explaining why the expectation is needed.

## Root Cause Analysis

Looking at lines 95-206, the pattern is:

| Line | Method | Parameter Marked Unused |
|------|--------|-------------------------|
| 95   | `visit_paragraph` | `block`, `depth` |
| 105  | `visit_heading` | `block`, `level`, `depth` |
| 121  | `visit_code_block` | `block`, `language`, `depth` |
| 137  | `visit_frontmatter` | `block`, `format`, `depth` |
| 152  | `visit_thematic_break` | `block`, `depth` |
| 165  | `visit_blockquote` | `block`, `depth` |
| 179  | `visit_list` | `block`, `kind`, `depth` |
| 199  | `visit_list_item` | `block`, `is_task`, `depth` |

**Why is this happening?**

The trait method signatures declare parameters, but in the trait definition (not implementations) we don't actually use them. Clippy flags this because:
1. Trait method signatures define the interface contract
2. Parameters are part of that contract, but unused in the signature itself
3. The `#[expect(unused_variables)]` was added previously but the lint rule changed to require `reason`

**However**: These are ALL DEFAULT-IMPLEMENTED trait methods. The default implementations are empty (just `{}`), so they legitimately don't use the parameters.

## Fix Options

### Option A: Remove `#[expect]` entirely (Recommended)
Since these are trait method signatures with DEFAULT empty implementations, the parameters are intentionally unused. The `#[expect]` was added incorrectly - trait method parameters don't need to be "used" in the signature.

**Rationale**: The clippy warning `unused_variables` is meant for actual variables in function bodies. In trait method signatures, the parameters are part of the interface contract and will only be used when implementors provide concrete implementations.

### Option B: Add `reason = "..."` to each
Keep the `#[expect]` and add a reason like `reason = "trait method has default empty impl"`.

## Recommended Fix

**Option A**: Remove all 9 instances of `#[expect(unused_variables)]` from the trait method signatures.

The trait methods have empty default implementations `{}`, so the parameters are legitimately unused by design. The `#[expect]` was incorrectly applied to trait method signatures.

## Implementation

```rust
// Line 95 - REMOVE these 2 lines:
#[expect(unused_variables)]
fn visit_paragraph(&mut self, block: &Block<'source>, depth: u32) {}

// Replace with:
fn visit_paragraph(&mut self, block: &Block<'source>, depth: u32) {}

// Repeat for lines 105, 121, 137, 152, 165, 179, 199
```

Total: 9 edits (remove one line each - the `#[expect(...)]` line)

## Risk Assessment

- **Low risk**: Removing unused attribute that was incorrectly applied
- **No behavioral change**: Trait methods remain empty defaults
- **Precedent**: This pattern is common in Rust trait definitions

## Verification

After fix:
```bash
cargo clippy --lib -p lithos-core 2>&1 | grep -E "^error:"
# Should return: error: could not compile... (no actual errors after fix)
```

Wait - looking at this more carefully:

The issue is that these are EMPTY DEFAULT implementations on a TRAIT. In trait definitions, you don't have a function body that "uses" variables - you're just declaring the interface.

But wait - there ARE implementations in the `impl BlockVisitor for BlockCounter` etc. Those DO use the parameters (see lines 256-315). So the trait signature is correct.

The fix is simply: REMOVE the `#[expect(unused_variables)]` from the trait definition lines. The default impls don't need it, and clippy is complaining about the expect itself not having a reason.
