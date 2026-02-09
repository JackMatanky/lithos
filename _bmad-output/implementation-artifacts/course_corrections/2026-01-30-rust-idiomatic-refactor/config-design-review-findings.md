# Config Context Design Review Findings

**Date**: 2026-02-09
**Scope**: Comprehensive review of design specs vs implementation + Figment optimization analysis

## Executive Summary

Completed thorough review of:

1. **Design Specs**: 001-config-models.md, 002-config-cqrs.md, 003-config-task.md
2. **Implementation**: `lithos-core/src/config/*`
3. **Figment Usage**: Research on optimal patterns for layered configuration

**Key Finding**: Current implementation is **largely aligned** with specs. Identified **12 misalignments** (documented in `config-context-combined-status.md` Phase 0.5), but **Figment usage is already optimal** for our use case.

---

## Design Spec Review Results

### 001-config-models.md ✅ (Well-Aligned)

**Adherence**: ~90% aligned

**Key Recommendations from Spec**:

- ✅ Use `Option` overlays instead of empty-string sentinels (implemented in raw types)
- ✅ Type-driven newtypes (`LogLevel`, `VaultId`, etc.) (mostly implemented)
- ✅ Figment for layering with `merge` precedence (correctly implemented)
- ✅ Raw types separate from domain types (clean separation in `raw.rs`)
- ⚠️ Some path types still `String` instead of `PathBuf` (see Phase 0.5)

**Strong Points**:

- Clean separation: `Raw* → TryFrom → Domain → [Stored*]`
- Figment confined to `config::ingest` (adapter boundary)
- Validation happens at construction (newtype pattern)

**Minor Gaps**:

- Vault identity (`VaultId`) not yet fully implemented (noted in spec as future work)
- Some path operations use string formatting instead of `PathBuf::join` (Phase 0.5 task)

---

### 002-config-cqrs.md ✅ (Now Aligned)

**Adherence**: 100% aligned (after today's spec update)

**Changes Made**:

- ✅ Added `ConfigIngestError` variant to spec (matches implementation)
- ✅ Documented rationale for three-tier error taxonomy

**Error Taxonomy** (validated):

```rust
// Infrastructure errors (adapter boundary failures)
ConfigIngestError    → TOML parsing, Figment extraction
DbError             → Storage layer failures

// Domain errors (business rule violations)
ConfigError         → Empty paths, invalid enums, constraint violations

// CQRS wrappers
ConfigCommandError  → Domain | Storage | Ingest
ConfigQueryError    → Storage | Corruption
```

**Strong Points**:

- Split CQRS error types prevent domain/storage confusion
- `ConfigIngestError` properly isolates Figment errors at adapter boundary
- Versioned merged config read model architecture well-designed (not yet implemented)

**Implementation Status**:

- ✅ Basic CQRS structure in place
- ⏳ Versioned merged config (future work, not blocking)
- ⏳ VaultId-based persistence (future work, path-based for now)

---

### 003-config-task.md ⚠️ (Some Divergences)

**Adherence**: ~75% aligned (12 misalignments identified)

**Critical Issues** (Phase 0.5):

1. **Type inference UX**: Spec says `#[serde(untagged)]` for type inference (no `type=` key), implementation uses tagged format
2. **Performance**: Spec says compile regex at config load (`Arc<Regex>`), implementation stores `Option<String>`
3. **DRY**: Spec says `Bounds<T>` generic, implementation has separate `IntegerBounds`/`FloatBounds`
4. **API surface**: Spec says `validate_raw_value` is private helper, implementation exposes it publicly

**See Phase 0.5 in `config-context-combined-status.md` for complete list**

**Strong Points**:

- TaskConfig newtype pattern correctly applied
- First-class date fields with emoji support (Obsidian compatibility)
- Validation-in-construction enforced

---

## Figment Usage Analysis

### Current Implementation Review

**What We're Doing**:

```rust
// ingest_global()
Figment::from(Serialized::defaults(RawGlobal::default()))
    .merge(Toml::file(path))  // if exists
    .extract()

// ingest_vault()
Figment::from(Serialized::defaults(RawVault::default()))
    .merge(Toml::file(vault_root/.lithos/lithos.toml))  // if exists
    .extract()
```

**Verdict**: ✅ **Already optimal for our use case**

### Figment Best Practices Validated

| Practice                                                 | Lithos Implementation                            | Status  |
| :------------------------------------------------------- | :----------------------------------------------- | :------ |
| **Use `Serialized::defaults` for programmatic defaults** | ✅ `RawGlobal::default()`, `RawVault::default()` | Correct |
| **Use `merge` for overrides (incoming wins)**            | ✅ File overrides defaults                       | Correct |
| **Use `join` for fallbacks (existing wins)**             | ❌ Not needed (we only have defaults + file)     | N/A     |
| **Avoid `#[serde(flatten)]`** (breaks error attribution) | ✅ No flatten used                               | Correct |
| **Let Figment handle missing files gracefully**          | ✅ Check `path.exists()` before merge            | Correct |
| **Extract into Raw types, validate separately**          | ✅ `Raw* → TryFrom → Domain`                     | Correct |

### Figment Features We're NOT Using (Intentionally)

#### Profiles (`select`, `nested`)

```rust
// What we could do:
Figment::from(Toml::file("config.toml").nested())
    .select("production")  // vs "dev", "staging"
```

**Why we don't need it**:

- Global vs Vault layers are NOT profiles (they're distinct data sources)
- We don't have environment-specific config (dev/staging/prod)
- If we add environment support later, we'd use profiles for **operational context** (not vault identity)

**Spec guidance** (001-config-models.md, Appendix A.3.2):

> "Profiles work well for runtime context (dev/test/prod), NOT for vault identity (vaults are domain instances)"

**Recommendation**: Don't add profiles unless we need operational contexts (rare for CLI tool)

#### Environment Variables (`Env::prefixed`)

```rust
// What we could do:
Figment::from(Serialized::defaults(...))
    .merge(Toml::file(...))
    .merge(Env::prefixed("LITHOS_").split("_"))  // LITHOS_LOGGING_LEVEL → logging.level
```

**Why we don't need it**:

- Current pattern uses single env var: `LITHOS_GLOBAL_CONFIG` (path to file)
- Per-field env overrides add complexity without user request
- TOML files are the intended UX (not dozens of env vars)

**Recommendation**: Keep current pattern unless users request env var overrides

#### Array Concatenation (`admerge`)

```rust
// What we could do:
figment.admerge(provider)  // Concatenate arrays instead of replacing
```

**Why we don't need it**:

- Replace semantics are correct for our config (vault overrides global entirely)
- No use case for "extend global list with vault additions"

**Spec guidance** (001-config-models.md, Appendix A.3.5):

> "Default to **replace** for arrays/lists in config overrides"

**Recommendation**: Don't use `admerge` (replace semantics are correct)

---

## Optimization Opportunities

### None Found for Figment Usage

Our ingest implementation is **already following Figment best practices**:

1. ✅ Defaults via `Serialized::defaults`
2. ✅ Merge precedence for overrides
3. ✅ Extract into Raw types (validation deferred to `TryFrom`)
4. ✅ Figment errors wrapped at boundary (`ConfigIngestError`)
5. ✅ No premature complexity (profiles, env vars, admerge)

### Code Simplification: None Needed

**Current pattern is minimal and correct**:

```rust
pub fn ingest_global() -> Result<RawGlobal, ConfigIngestError> {
    let mut figment = Figment::from(Serialized::defaults(RawGlobal::default()));

    if let Some(path) = global_config_path_from_env()
        && path.exists()
    {
        figment = figment.merge(Toml::file(path));
    }

    figment.extract().map_err(ConfigIngestError::from)
}
```

**Why not simplify further?**

- We could remove `if path.exists()` check (Figment handles missing files), but **explicit check is clearer**
- We could inline `global_config_path_from_env()`, but **separation is clearer**
- We could remove `mut figment`, but **builder pattern is idiomatic**

**Recommendation**: Keep as-is (clarity > brevity)

---

## Recommendations

### Immediate (Phase 0.5 Tasks)

1. **Fix Task Config Type Inference** (003 spec, UX regression)
   - Change `RawTaskFieldSpec` to `#[serde(untagged)]`
   - Remove `type=` key requirement from user configs

2. **Consolidate Bounds Types** (003 spec, DRY violation)
   - Introduce `Bounds<T>` generic enum
   - Replace `IntegerBounds`/`FloatBounds` with `Bounds<i64>`/`Bounds<f64>`

3. **Compile Regex at Config Load** (003 spec, performance)
   - Store `Arc<Regex>` in `TaskFieldSpec::String`, not `Option<String>`

4. **Make validate_raw_value Private** (003 spec, API surface)
   - Change to `pub(crate)` or remove entirely

### Future (Not Blocking)

1. **VaultId-based persistence** (001/002 specs)
   - Implement stable vault identity (not path-based keys)
   - Requires vault ID discovery/persistence strategy

2. **Versioned merged config read model** (002 spec)
   - Enable config rollback
   - Requires `ConfigVersion`, version history retention policy

3. **Path types instead of strings** (001 spec)
   - Use `PathBuf` for path operations (not string formatting)
   - Use `join` semantics instead of string concatenation

### Non-Recommendations (Explicitly Avoid)

1. **Don't add Figment profiles** unless we need operational contexts (dev/prod)
2. **Don't add env var overrides** unless users request it
3. **Don't use `admerge`** (replace semantics are correct)
4. **Don't remove `path.exists()` check** (clarity > terseness)

---

## Conclusion

**Figment Usage**: ✅ Already optimal, no changes needed

**Design Alignment**: ⚠️ 12 minor issues in Phase 0.5, mostly in task config

**Next Steps**:

1. ✅ ConfigIngestError documented in spec (DONE)
2. ⏭️ Address Phase 0.5 issues (12 tasks in `config-context-combined-status.md`)
3. ⏭️ Run `mise run verify` after each fix

**Overall**: Implementation quality is high. Figment usage demonstrates understanding of best practices. Phase 0.5 tasks are refinements, not major rework.
