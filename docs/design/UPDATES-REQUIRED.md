# Design Spec Updates Required

## Status: Ready for Implementation

This document tracks all required updates to design specs following the task management system split and architectural refinements.

---

## ✅ Completed

- [x] **003-config-task.md** - Created (new)
- [x] **007-note-list-task.md** - Created (new)
- [x] **012-template-task-integration.md** - Created (new)
- [x] **docs/operations/clean-slate-protocol.md** - Created (new)

---

## 🔴 Critical Updates (Block Implementation)

### **001-config-models.md**

**Changes**:
1. **Remove SettingValue** (lines 28, 50-53):
   - Delete all references to `SettingValue` as universal config type
   - Note: SettingValue concept eliminated in favor of context-specific newtypes

2. **Add Newtype Pattern Section** (after line 160):
   ```markdown
   #### Config Value Newtypes (Type-Driven Design)

   Config values use newtypes to enforce invariants:

   | Type | Backing | Purpose | Rules |
   |------|---------|---------|-------|
   | `LogLevel` | enum | Logging verbosity | Error/Warn/Info/Debug/Trace |
   | `NotesDir` | `PathBuf` | Notes directory path | Vault-relative, non-empty |
   | `SchemasDir` | `PathBuf` | Schemas directory path | Vault-relative, non-empty |
   | `TemplatesDir` | `PathBuf` | Templates directory path | Vault-relative, non-empty |
   ```

3. **Add Raw Types Section** (after line 200):
   ```markdown
   #### Raw Input Types (Serde-Friendly)

   Config files are deserialized into Raw types before validation:

   - `RawGlobal` - Global config file shape (TOML)
   - `RawVault` - Vault config file shape (TOML)
   - `RawPaths` - Path configuration (tolerant to missing values)
   - `RawLogging` - Logging configuration (string log level)

   Conversion: `RawGlobal::try_into() -> Result<Global, ConfigError>`
   ```

4. **Update cross-references**:
   - Line 15: Change `006-task-management-system.md` → `003-config-task.md`
   - Add reference to clean-slate protocol in Section 5.2

**Priority**: Critical (blocks config implementation)

---

### **002-config-cqrs.md**

**Changes**:
1. **Update port trait names** (line 77-92):
   ```rust
   // OLD
   use lithos_core::config::ports::{Command as _, Query as _};

   // NEW
   use lithos_core::config::ports::{Command, Query};  // ✅ Qualified imports
   ```

2. **Split error types** (add after line 60):
   ```markdown
   #### Error Type Strategy

   CQRS operations use split error types:

   **ConfigCommandError**:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum ConfigCommandError {
       #[error("Domain error: {0}")]
       Domain(#[from] ConfigError),

       #[error("Storage error: {0}")]
       Storage(#[from] DbError),
   }
   ```

   **ConfigQueryError**:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum ConfigQueryError {
       #[error("Storage error: {0}")]
       Storage(#[from] DbError),

       #[error("Data corruption: {0}")]
       Corruption(String),
   }
   ```

3. **Update cross-references**:
   - Line 16: Change `006-task-management-system.md` → `003-config-task.md`
   - Add reference to clean-slate protocol in Section 5.2

**Priority**: Critical (blocks CQRS implementation)

---

### **004-note-models.md**

**Changes**:
1. **Add FieldValue section** (insert after line 50):
   ```markdown
   ### 3.2.1 FieldValue (Shared Primitive)

   **Purpose**: Runtime representation of note metadata values (shared by frontmatter and task metadata).

   **Location**: `note/value.rs` (NEW - extracted from frontmatter)

   **Shape**:
   ```rust
   #[derive(Debug, Clone, PartialEq)]
   pub enum FieldValue {
       String(String),
       Number(f64),
       Boolean(bool),
       Date(chrono::DateTime<chrono::Utc>),
       Array(Vec<FieldValue>),
       Object(HashMap<String, FieldValue>),
   }
   ```

   **Used by**:
   - `Frontmatter` (YAML/TOML parsed values)
   - `TaskMetadata` (inline `[key:: value]` fields)

   **Rationale**: Single value primitive for all note metadata avoids duplication.
   ```

2. **Update Frontmatter section** (around line 100):
   ```markdown
   - Change: `fields: HashMap<String, FrontmatterValue>`
   - To: `fields: HashMap<String, FieldValue>`  // ✅ Uses shared FieldValue
   ```

3. **Add NotePath validation examples** (Section 3.3 or new subsection):
   ```markdown
   #### Valid NotePath Examples

   NotePath represents validated markdown file paths in the vault.

   ```rust
   // ✅ Valid note paths
   NotePath::try_from("notes/foo.md")?;
   NotePath::try_from("a.md")?;  // root-level
   NotePath::try_from("deep/nested/path.md")?;

   // ❌ Invalid
   NotePath::try_from("/notes/foo.md")?;  // Absolute → Err(AbsolutePath)
   NotePath::try_from("../foo.md")?;       // Traversal → Err(PathTraversal)
   NotePath::try_from("notes//foo.md")?;   // Double slash → normalize first
   NotePath::try_from("notes/foo.txt")?;   // Wrong extension → Err(NotMarkdown)
   ```

4. **Update file structure** (Section 3.6):
   ```markdown
   note/
   ├── mod.rs
   ├── aggregate.rs      // Note
   ├── value.rs          // FieldValue (NEW - shared primitive)
   ├── frontmatter.rs    // Frontmatter (uses FieldValue)
   ├── task.rs           // Task, TaskMetadata (uses FieldValue) - added by 007
   ├── list.rs           // List, ListItem - added by 007
   └── ports.rs          // Query, Command traits
   ```

5. **Update cross-references**:
   - Add reference to `007-note-list-task.md` for Task/List integration
   - Add reference to clean-slate protocol

**Priority**: Critical (blocks note implementation)

**Context Boundary Note**: `NotePath` is note-domain specific. Do NOT confuse with `VaultRelPath` (schema context, used in FileSpec validation).

---

### **005-note-cqrs.md**

**Changes**:
1. **Update port trait names** (lines 88-89):
   ```rust
   // OLD
   use lithos_core::note::ports::{Command as _, Query as _};

   // NEW
   use lithos_core::note::ports::{Command, Query};  // ✅ Qualified
   ```

2. **Split error types** (add after line 50):
   ```markdown
   #### Error Type Strategy

   **NoteCommandError**:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum NoteCommandError {
       #[error("Domain error: {0}")]
       Domain(#[from] NoteError),

       #[error("Storage error: {0}")]
       Storage(#[from] DbError),
   }
   ```

   **NoteQueryError**:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum NoteQueryError {
       #[error("Storage error: {0}")]
       Storage(#[from] DbError),

       #[error("Data corruption: {0}")]
       Corruption(String),
   }
   ```

3. **Add reference to clean-slate protocol** (Section 5.2):
   ```markdown
   See [Clean-Slate Protocol](../../operations/clean-slate-protocol.md) for reindex procedures.
   ```

**Priority**: Critical (blocks CQRS implementation)

---

### **006-note-frontmatter.md**

**Changes**:
1. **Update to use FieldValue** (throughout):
   - Change all references from local `FrontmatterValue` types to `note::value::FieldValue`
   - Update imports to `use super::value::FieldValue;`

2. **Add conversion section**:
   ```markdown
   ### Conversion from YAML/TOML

   Frontmatter parsing converts YAML/TOML to FieldValue:

   ```rust
   impl Frontmatter {
       pub fn from_yaml(yaml: &str) -> Result<Self, FrontmatterError> {
           let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml)?;
           let fields = convert_yaml_to_field_values(yaml_value)?;
           Ok(Frontmatter { fields })
       }
   }

   fn convert_yaml_to_field_values(
       yaml: serde_yaml::Value
   ) -> Result<HashMap<String, FieldValue>, FrontmatterError> {
       // Convert serde_yaml::Value → FieldValue
   }
   ```

3. **Reference FieldValue location**:
   ```markdown
   FieldValue is defined in `note/value.rs` and shared with task metadata.
   ```

**Priority**: Critical (blocks frontmatter implementation)

---

### **009-schema-cqrs.md**

**Changes**:
1. **Rename port traits** (lines 78-79):
   ```rust
   // OLD
   use lithos_core::schema::{command, query};

   // NEW
   use lithos_core::schema::ports::{Command, Query};  // ✅ Qualified
   ```

2. **Split error types** (add after line 60):
   ```markdown
   #### Error Type Strategy

   **SchemaCommandError**:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum SchemaCommandError {
       #[error("Domain error: {0}")]
       Domain(#[from] SchemaError),

       #[error("Storage error: {0}")]
       Storage(#[from] DbError),
   }
   ```

   **SchemaQueryError**:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum SchemaQueryError {
       #[error("Storage error: {0}")]
       Storage(#[from] DbError),

       #[error("Data corruption: {0}")]
       Corruption(String),
   }
   ```

3. **Add clean-slate reference** (Section 5.2)

**Priority**: Critical (blocks schema CQRS)

---

## 🟡 Medium Priority (Before Implementation Start)

### **011-property-spec.md**

**Changes**:
1. **Add validation examples for VaultRelPath** (Section 3.5.2):
   ```markdown
   #### Valid VaultRelPath Examples (FileSpec Validation)

   VaultRelPath is used **only** for schema FileSpec directory restrictions.

   ```rust
   // ✅ Valid vault-relative directory paths
   VaultRelPath::try_from("notes/")?;
   VaultRelPath::try_from("attachments/images/")?;
   VaultRelPath::try_from(".")?;  // Root vault directory

   // ❌ Invalid
   VaultRelPath::try_from("/absolute/path/")?;   // Absolute → Err(AbsolutePath)
   VaultRelPath::try_from("../parent/")?;        // Traversal → Err(PathTraversal)
   VaultRelPath::try_from("")?;                  // Empty → Err(EmptyPath)
   VaultRelPath::try_from("C:\\Windows\\")?;     // Windows prefix → Err(AbsolutePath)
   ```

**Note**: `NotePath` (note file paths) is defined in note context (004-note-models.md), not schema context.

2. **Add regex cache pseudocode** (Section 3.5.3):
   ```rust
   fn get_cached_regex(pattern: &str) -> Result<Arc<Regex>, SchemaError> {
       static CACHE: OnceLock<RwLock<HashMap<String, Arc<Regex>>>> = OnceLock::new();
       let cache = CACHE.get_or_init(|| RwLock::new(HashMap::new()));

       // Fast path: read lock
       {
           let lock = cache.read().unwrap_or_else(PoisonError::into_inner);
           if let Some(regex) = lock.get(pattern) {
               return Ok(Arc::clone(regex));
           }
       }

       // Slow path: compile without holding lock
       let compiled = Regex::new(pattern)
           .map_err(|e| SchemaError::InvalidRegex(pattern.to_owned(), e))?;
       let arc = Arc::new(compiled);

       // Insert with write lock (check-before-insert)
       {
           let mut lock = cache.write().unwrap_or_else(PoisonError::into_inner);
           Ok(Arc::clone(lock.entry(pattern.to_owned()).or_insert(arc)))
       }
   }
   ```

3. **Document string length semantics**:
   ```markdown
   #### String Length Semantics

   **Current**: Length measured in **UTF-8 bytes** (`value.len()`)

   **Rationale**: Simple, fast, matches Rust native string API

   **Limitation**: Multi-byte Unicode characters count as multiple "units"

   **Example**:
   ```rust
   let text = "café";  // é is 2 bytes
   assert_eq!(text.len(), 5);  // bytes: c a f é(2)
   assert_eq!(text.chars().count(), 4);  // chars: c a f é
   ```

   **Future Consideration**: Add `chars().count()` option if user expectations demand it.
   ```

**Priority**: Medium (nice-to-have before implementation)

**Context Boundary Note**: `VaultRelPath` is schema-domain specific (FileSpec validation). Do NOT confuse with `NotePath` (note context, actual note file paths).

---

## 🟢 Low Priority (Can Defer)

### **All Specs - Cross-Reference Updates**

**Changes**:
- Update all references from `006-task-management-system.md` to split specs:
  - `003-config-task.md` (config aspects)
  - `007-note-list-task.md` (domain models)
  - `012-template-task-integration.md` (template integration)

**Files Affected**:
- 001, 002, 003, 004, 005 (any that reference task spec)

**Priority**: Low (consistency polish)

---

### **Template Specs - Future Work**

**Missing Specs** (defer to Epic 12):
- `template-models.md` - Core template domain types
- `template-cqrs.md` - Template persistence (if needed)

**Note**: 012 covers task integration, but not core template system

**Priority**: Defer (Epic 12 not started)

---

## Implementation Plan

### Phase 1: Critical Updates (This Session)
1. ✅ Create clean-slate protocol
2. ⏳ Update 001-config-models (remove SettingValue, add newtypes)
3. ⏳ Update 002-config-cqrs (split errors, fix ports)
4. ⏳ Update 004-note-models (add FieldValue)
5. ⏳ Update 005-note-cqrs (split errors, fix ports)
6. ⏳ Update 006-note-frontmatter (use FieldValue)
7. ⏳ Update 009-schema-cqrs (split errors, fix ports)

### Phase 2: Medium Priority (Next Session)
8. Update 011-property-spec (add examples)

### Phase 3: Polish (Ongoing)
9. Fix all cross-references
10. Consistency review

---

## Validation Checklist

Before marking spec as "Ready for Implementation":

- [ ] All SettingValue references removed
- [ ] Port traits use qualified imports (`config::ports::Query`)
- [ ] Error types split into Command/Query variants
- [ ] FieldValue documented in note context
- [ ] Cross-references point to correct files
- [ ] Clean-slate protocol referenced in migration sections
- [ ] All code examples compile (conceptually)
- [ ] Mermaid diagrams render correctly

---

## Notes

- **006-task-management-system.md**: Preserved as-is (historical reference)
- **All new specs (003/007/012)**: Already compliant with standards
- **Clean-slate protocol**: Created and ready for Epic 10/11 integration
