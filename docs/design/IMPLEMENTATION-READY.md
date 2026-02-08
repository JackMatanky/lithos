# Design Specs: Implementation Ready Status

## ✅ Fully Complete & Ready

These specs are ready for immediate implementation with no changes needed:

1. **006a-config-task.md** - Task configuration (Config context)
   - ✅ Type inference via `#[serde(untagged)]`
   - ✅ First-class date fields with emoji support
   - ✅ TaskFieldKeyword newtype validation
   - ✅ Unified DateTime type
2. **006b-note-list-task.md** - List and Task entities (Note context)
3. **006c-template-task-integration.md** - Template task integration
4. **docs/operations/clean-slate-protocol.md** - Reindex procedures

---

## ⏳ Updates In Progress

The following specs have been identified for updates but are **not yet complete** due to session constraints. All required changes are documented in `UPDATES-REQUIRED.md`.

### Critical Priority (Blocks Implementation)

1. **001-config-models.md**
   - Status: Partially updated (cross-references fixed)
   - Remaining: Remove SettingValue references (lines 50-52, 340, 403), add newtype section, add Raw types section

2. **002-config-cqrs.md**
   - Status: Not started
   - Remaining: Update port names, split errors, fix cross-references

3. **003-note-models.md**
   - Status: Not started
   - Remaining: Add FieldValue section, update Frontmatter, add NotePath examples, fix cross-references

4. **004-note-cqrs.md**
   - Status: Not started
   - Remaining: Update port names, split errors, add clean-slate reference

5. **005-note-frontmatter.md**
   - Status: Not started
   - Remaining: Use FieldValue from value.rs, add conversion section

6. **008-schema-cqrs.md**
   - Status: Not started
   - Remaining: Rename port traits, split errors, add clean-slate reference

### Medium Priority

7. **010-property-spec.md**
   - Status: Not started
   - Remaining: Add VaultRelPath validation examples, regex cache pseudocode, string length semantics

---

## 📋 Next Session Action Plan

### Step 1: Complete Critical Updates (Priority Order)

Run these updates in sequence:

```bash
# 1. Finish 001-config-models.md
#    - Remove SettingValue (4 occurrences)
#    - Add newtype pattern section after line 160
#    - Add Raw types section after line 200

# 2. Update 002-config-cqrs.md
#    - Change ports imports (line 77)
#    - Add split error types section (after line 60)
#    - Fix cross-reference (line 16)

# 3. Update 003-note-models.md
#    - Add FieldValue section (after line 50)
#    - Update Frontmatter to use FieldValue
#    - Add NotePath examples
#    - Update file structure diagram
#    - Fix cross-references

# 4. Update 004-note-cqrs.md
#    - Update port imports (line 88-89)
#    - Add split error types (after line 50)
#    - Add clean-slate reference (Section 5.2)

# 5. Update 005-note-frontmatter.md
#    - Replace local types with note::value::FieldValue
#    - Add YAML/TOML conversion section
#    - Reference value.rs location

# 6. Update 008-schema-cqrs.md
#    - Rename port traits (line 78-79)
#    - Add split error types (after line 60)
#    - Add clean-slate reference

# 7. Update 010-property-spec.md
#    - Add VaultRelPath examples (Section 3.5.2)
#    - Add regex cache pseudocode (Section 3.5.3)
#    - Document string length semantics
```

### Step 2: Verification

After all updates:

```bash
# Check all cross-references are valid
grep -r "006-task-management-system" docs/design/*.md
# Should return: 0 results (all updated to 006a/b/c)

# Check SettingValue removed from config specs
grep "SettingValue" docs/design/001-config-models.md docs/design/002-config-cqrs.md
# Should return: 0 results in context of "universal type"

# Check FieldValue added to note specs
grep "FieldValue" docs/design/003-note-models.md docs/design/005-note-frontmatter.md
# Should return: Multiple results showing usage

# Check port naming consistency
grep "ports::{Query, Command}" docs/design/002-config-cqrs.md docs/design/004-note-cqrs.md docs/design/008-schema-cqrs.md
# Should return: Consistent qualified imports

# Check error splits
grep "CommandError\\|QueryError" docs/design/002-config-cqrs.md docs/design/004-note-cqrs.md docs/design/008-schema-cqrs.md
# Should return: Split error types in all CQRS specs
```

### Step 3: Final Polish

1. Run consistency check across all specs
2. Verify all mermaid diagrams render
3. Check all code examples for syntax
4. Update UPDATES-REQUIRED.md to mark completed items

---

## 📝 Detailed Change Script

For reference, here are the exact changes needed (use UPDATES-REQUIRED.md for full details):

### 001-config-models.md

**Line 50-52**: Delete
```markdown
- `SettingValue` is **owned by the config context**...
- Other contexts may _consume_ `SettingValue`...
- If a bounded context needs its own dynamic value model...
```

**After line 160**: Insert
```markdown
#### Config Value Newtypes (Type-Driven Design)

Config uses newtypes to enforce invariants at construction:

**Path Types**:
- `NotesDir(PathBuf)` - Vault-relative notes directory
- `SchemasDir(PathBuf)` - Vault-relative schemas directory
- `TemplatesDir(PathBuf)` - Vault-relative templates directory

**Validation**: All paths validated as vault-relative (no absolute, no traversal)

**Enum Types**:
```rust
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
```

**Rationale**: Newtypes make invalid states unrepresentable (e.g., cannot construct `NotesDir("/absolute/path")`).
```

**After line 200**: Insert
```markdown
#### Raw Input Types (Serde-Friendly)

Config files deserialized into Raw types before domain validation:

```rust
// config/raw.rs

#[derive(Deserialize)]
pub struct RawGlobal {
    pub paths: Option<RawPaths>,
    pub logging: Option<RawLogging>,
    pub frontmatter: Option<RawFrontmatter>,
    // ...
}

#[derive(Deserialize)]
pub struct RawPaths {
    pub notes: Option<String>,
    pub schemas: Option<String>,
    pub templates: Option<String>,
}

#[derive(Deserialize)]
pub struct RawLogging {
    pub log_level: Option<String>,  // "info", "debug", etc.
    pub log_file: Option<String>,
}
```

**Conversion**:
```rust
impl Global {
    pub fn from_raw(raw: RawGlobal) -> Result<Self, ConfigError> {
        // Validate and convert to domain types
    }
}
```
```

**Line 340**: Delete entire line mentioning SettingValue

**Line 403**: Delete or replace with generic encrypted field guidance

---

## 🎯 Success Criteria

All specs updated when:

- [ ] No references to SettingValue as universal type
- [ ] All CQRS specs use qualified port imports
- [ ] All CQRS specs have split Command/Query error types
- [ ] FieldValue documented in note context (003, 005)
- [ ] NotePath examples in 003, VaultRelPath examples in 010
- [ ] All cross-references point to 006a/b/c (not 006)
- [ ] Clean-slate protocol referenced in all CQRS migration sections
- [ ] All code examples use correct newtype patterns

---

## 📚 Reference Documents

- **UPDATES-REQUIRED.md** - Full detail of all required changes
- **006a/b/c specs** - Template for correct patterns
- **clean-slate-protocol.md** - Migration procedures

---

**Next Session**: Start with Step 1, updating specs in priority order. Use UPDATES-REQUIRED.md as the authoritative change list.
