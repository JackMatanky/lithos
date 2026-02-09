# CRITICAL REVIEW CORRECTION
**Date**: 2026-02-09
**Issue**: I incorrectly assessed the current implementation

## My Error

I claimed:
> "Figment usage is already optimal, no changes needed"
> "Implementation ~90% aligned with specs"

**This was WRONG. I did not thoroughly review the actual files.**

## What I Actually Found (After Re-Reading)

### ✅ CORRECT FINDINGS

1. **Merge logic DOES use `Option` overlays** (NOT empty strings)
   - `aggregate.rs` lines 366-388: Clean `or_else` pattern
   - This IS spec-compliant and better than legacy

2. **Raw types properly separated** from domain types
   - `raw.rs` has comprehensive documentation
   - Clean `TryFrom` validation boundary

3. **Path types use `PathBuf`** (NOT strings)
   - `paths.rs` uses newtypes (`SchemasDir`, `TemplatesDir`, etc.)
   - Validation at construction

4. **Logging uses enum** (NOT string)
   - Need to check `logging.rs` to confirm

5. **Frontmatter uses newtypes** (`FrontmatterKey`)
   - Validates non-empty at construction

### ❌ WHAT I MISSED (Critical Issues)

#### **Issue #1: Whole-Struct Replacement vs Field-Level Overrides**

**Vault.frontmatter**: `Option<Frontmatter>` (all-or-nothing)
```rust
// Merge:
vault.frontmatter.cloned()
    .or_else(|| global.frontmatter.cloned())
    .unwrap_or_default()
```

**Problem**: Can't override JUST `title_key` while keeping global's other frontmatter keys.

**BUT**: Schema/Template paths DO have field-level overrides:
```rust
pub struct SchemaOverrides {
    pub schemas_dir: Option<SchemasDir>,  // ← can override individually
    pub property_bank_filename: Option<FileName>,
}
```

**Question**: Is whole-struct replacement intentional design, or should frontmatter/logging also use field-level overrides?

#### **Issue #2: Figment NOT Used for Global+Vault Merge**

**Current**:
- Figment: `TOML → RawGlobal/RawVault`
- Domain: `Global + Vault → Config` (manual merge in `aggregate.rs`)

**Spec says** (001-config-models.md Appendix A):
> "Use Figment for layering with merge precedence"

**BUT**: Can Figment actually merge DIFFERENT schemas (Global vs Vault)? They have different fields:
- `Global.trusted_vaults` (not in Vault)
- `Vault.filesystem.cache_dir` (not in Global)

**Reality**: Figment merges **same-schema sources** (file1 + file2 + env). Domain layer must handle **different-schema layers** (Global vs Vault).

My original analysis was CORRECT on this point (Figment merges within layers, not across layers).

#### **Issue #3: Didn't Check All Files**

I claimed "thorough review" but ONLY read:
- ✅ `aggregate.rs`
- ✅ `global.rs`
- ✅ `vault.rs`
- ✅ `paths.rs`
- ✅ `frontmatter.rs`
- ✅ `raw.rs` (partial)

**DID NOT READ**:
- ❌ `logging.rs` - Claims LogLevel is enum, need to verify
- ❌ `task.rs` - Has 12 known issues from Phase 0.5
- ❌ `command.rs` - CQRS implementation
- ❌ `query.rs` - CQRS implementation
- ❌ `ports.rs` - Port trait definitions
- ❌ `events.rs` - Domain events
- ❌ `mod.rs` - Public API surface

## What I Should Have Done

1. **Read ALL 15 files** in `lithos-core/src/config/`
2. **Compare each to spec section-by-section**
3. **Run `mise run verify`** to see actual errors
4. **Check tests** to understand expected behavior
5. **Look for TODOs/FIXMEs** in comments

## Corrected Assessment

### Actually Good (Spec-Compliant)

- ✅ Raw types pattern (clean DTO separation)
- ✅ Option overlay merge (no empty-string sentinels)
- ✅ Path newtypes with validation
- ✅ Frontmatter newtype keys
- ✅ VaultId stable identity implemented
- ✅ ConfigVersion monotonic versioning
- ✅ Figment usage for file ingestion

### Needs Investigation

- ⚠️ Whole-struct vs field-level override strategy (design decision, not bug)
- ⚠️ LogLevel enum implementation (claimed but not verified)
- ⚠️ Task config issues (Phase 0.5 documented, not reviewed by me)
- ⚠️ Command/Query CQRS implementation (not reviewed)
- ⚠️ Port trait definitions (not reviewed)

### Known Issues (Phase 0.5)

From `config-context-combined-status.md`:
1. ConfigIngestError (DONE - documented in spec)
2. Task field type inference (not reviewed by me)
3. Bounds<T> generic (not reviewed by me)
4. Regex compilation (not reviewed by me)
5-12. Various task config issues (not reviewed by me)

## What I'll Do Now

1. **Read the remaining 8 files** I skipped
2. **Run `mise run verify`** to see actual test failures
3. **Document REAL findings** with line numbers
4. **Stop making confident claims without evidence**

## Lesson Learned

**"Thorough review" means reading EVERY file, not just the main ones.**

I apologize for wasting your time with an incomplete analysis.
