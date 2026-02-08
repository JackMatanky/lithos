# Raw* Types: Purpose and Architecture

## Executive Summary

The `Raw*` types in `config/raw.rs` are **serde-only Data Transfer Objects (DTOs)** that serve as the deserialization boundary between configuration files (TOML/YAML/JSON) and validated domain models. They have **zero implementation** by design and exist solely to accept flexible, unvalidated input from configuration files.

## Purpose & Responsibilities

### What Raw* Types ARE

1. **Serde Deserialization Targets**: Direct mapping from TOML/YAML/JSON file structure
2. **Validation Boundary Markers**: Unvalidated input that must be converted via `TryFrom`
3. **Flexible Input Shapes**: Accept optional fields, string enums, and liberal input formats
4. **Figment Extraction Output**: What `config::ingest` produces from Figment providers

### What Raw* Types ARE NOT

1. **Domain Models**: They contain no business logic or behavior
2. **Validated Data**: Fields can be empty, malformed, or contradictory
3. **Persistent Storage Format**: They are NOT stored in the database (validated types use rkyv)
4. **Public API**: Internal implementation detail of the config context

## Architecture Pattern

```
┌─────────────────┐
│ Config File     │  (TOML/YAML/JSON)
│ lithos.toml     │
└────────┬────────┘
         │ serde::Deserialize
         ▼
┌─────────────────┐
│ Raw* Types      │  (config/raw.rs)
│ RawGlobal       │  • All fields Option<T>
│ RawVault        │  • String enums
│ RawFrontmatter  │  • No validation
└────────┬────────┘  • No methods
         │ TryFrom<Raw*>
         │ (validation happens here)
         ▼
┌─────────────────┐
│ Domain Types    │  (global.rs, vault.rs, frontmatter.rs, paths.rs)
│ Global          │  • Validated invariants
│ Vault           │  • Typed enums (LogLevel)
│ Frontmatter     │  • Non-empty strings
└────────┬────────┘  • Relative paths
         │ rkyv::Archive
         ▼
┌─────────────────┐
│ Database (redb) │  (zero-copy archived bytes)
└─────────────────┘
```

## Current Implementation Analysis

### Raw* Types (zero implementation)

```rust
// config/raw.rs - DTOs only, no impl blocks
pub struct RawGlobal {
    pub filesystem: Option<RawGlobalPaths>,
    pub frontmatter: Option<RawFrontmatter>,
    pub logging: Option<RawLogging>,
    // ... all fields optional
}

pub struct RawFrontmatter {
    pub alias_key: Option<String>,        // Could be empty
    pub date_created_key: Option<String>, // Could be empty
    // ... no validation
}

pub struct RawLogging {
    pub log_level: Option<String>,  // Could be "invalid"
}
```

### Validation (TryFrom implementations)

Validation happens in `TryFrom<Raw*>` implementations spread across domain modules:

| Raw Type | Validated Type | Location | Validation |
|----------|---------------|----------|------------|
| `RawFrontmatter` | `Frontmatter` | `frontmatter.rs` | Non-empty keys |
| `RawLogging` | `Logging` | `types.rs` | Enum validation |
| `RawSchemaPaths` | `Schema` | `paths.rs` | Path validation |
| `RawTemplatePaths` | `Template` | `paths.rs` | Path validation |
| `RawGlobal` | `Global` | `global.rs` | Aggregates above |
| `RawVault` | `Vault` | `vault.rs` | Aggregates above |
| `RawTaskConfig` | `TaskConfig` | `task.rs` | Complex validation |

## Design Rationale (from 001-config-models.md)

### Why Separate Raw from Domain?

1. **Flexible Parsing**: TOML files can have optional fields, typos, wrong types
2. **Clear Validation Boundary**: `TryFrom` is the explicit validation gate
3. **Error Messages**: Can report which file field failed validation
4. **Default Handling**: Optional fields use `None` → apply defaults during conversion
5. **Serialization Flexibility**: Raw types can evolve without breaking domain invariants

### Design Quote (001-config-models.md Section 3.2.1)

> **Rationale**: Separating Raw types from validated domain types allows:
> - Flexible TOML parsing (optional fields, string enums)
> - Clear validation boundaries (`TryFrom` conversion)
> - Independent evolution of file format vs domain model
> - Explicit error handling at the boundary

## Data Flow Example

```rust
// 1. File on disk
// lithos.toml:
// [frontmatter]
// alias_key = ""           # Invalid (empty string)
// title_key = "title"      # Valid

// 2. Deserialized into Raw (accepts anything)
let raw: RawFrontmatter = toml::from_str(file_content)?;
// raw.alias_key = Some("".to_string())  ← INVALID but accepted
// raw.title_key = Some("title".to_string())

// 3. Validation happens in TryFrom
let frontmatter = Frontmatter::try_from(raw)?;
// ❌ Error: ConfigError::ValidationFailed {
//      field: "alias_key",
//      message: "alias_key cannot be empty"
//    }

// 4. If valid, persisted to database as rkyv bytes
db.put(&config.to_bytes())?;
```

## Current State Assessment

### ✅ Working as Designed

1. Raw types are pure DTOs (no methods)
2. All fields are `Option<T>` for flexible parsing
3. Validation happens in `TryFrom` implementations
4. Clear separation of concerns (parsing ≠ validation)

### ⚠️ Architecture Concerns

1. **Scattered TryFrom Implementations**:
   - `TryFrom<RawFrontmatter>` in `frontmatter.rs`
   - `TryFrom<RawSchemaPaths>` in `paths.rs`
   - `TryFrom<RawLogging>` in `types.rs`
   - **Problem**: Raw types and their validation are in different files
   - **Alternative**: Could co-locate Raw + Domain + Validation per concern

2. **No Validation in Raw Types**:
   - This is **intentional** per design
   - But could add methods like `fn validate(&self) -> Result<(), ConfigError>`
   - Would allow early validation before conversion

3. **Duplication of Defaults**:
   - Defaults defined in domain types (`Frontmatter::default()`)
   - Then applied during `TryFrom` conversion
   - Could centralize default logic

## Recommended Next Steps

### Option 1: Keep Current Design (Minimal Changes)

**Pros**: Matches design spec exactly, clear separation
**Cons**: Validation scattered across multiple files

**Changes**:
- Add documentation to `raw.rs` explaining this architecture
- Keep `TryFrom` implementations where they are
- Accept that Raw and Domain types live separately

### Option 2: Co-locate Raw + Domain + Validation

**Pros**: All related code together, easier to maintain
**Cons**: Deviates from current design, larger refactor

**Structure**:
```
config/
├── frontmatter.rs
│   ├── RawFrontmatter (moved from raw.rs)
│   ├── Frontmatter
│   └── impl TryFrom<RawFrontmatter> for Frontmatter
├── paths.rs
│   ├── RawSchemaPaths, RawTemplatePaths (moved from raw.rs)
│   ├── Schema, Template
│   └── impl TryFrom conversions
├── types.rs
│   ├── RawLogging (moved from raw.rs)
│   ├── Logging, LogLevel
│   └── impl TryFrom<RawLogging> for Logging
└── raw.rs (removed - all types moved to domain modules)
```

### Option 3: Add Validation Methods to Raw Types

**Pros**: Early validation possible, self-documenting
**Cons**: Adds behavior to DTOs (not pure data)

**Example**:
```rust
impl RawFrontmatter {
    /// Pre-validate before conversion (optional check)
    pub fn validate_shape(&self) -> Result<(), ConfigError> {
        if let Some(ref key) = self.alias_key {
            if key.is_empty() {
                return Err(ConfigError::ValidationFailed { ... });
            }
        }
        Ok(())
    }
}
```

## Conclusion

The `Raw*` types are **correctly implemented as pure serde DTOs** per the design specification. They are meant to be "bare" - no methods, just data structures that mirror configuration file shapes.

The architecture is sound, but there's a tension between:
- **Design**: Separation of parsing (raw.rs) from domain (other files)
- **Maintainability**: Co-locating related types for easier maintenance

**Recommendation**: Add comprehensive documentation to `raw.rs` explaining this pattern, but keep the current structure as it matches the design spec. Consider co-location in a future refactor only if maintenance pain increases.
