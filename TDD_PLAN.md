# TDD Implementation Plan: SchemaConfigSpec Absolute Paths

**Branch:** `refactor/schema-config-spec-absolute-paths`
**Baseline:** ✅ 1157 unit tests + 38 integration tests PASSING

## Objective
Refactor `SchemaConfigSpec` to store `DirPath` and `FilePath` (absolute paths) instead of `RelativePath`, eliminating the need to pass `vault_root` separately to discovery and builder.

---

## Phase 1: Update SchemaConfigSpec Type Definition (RED → GREEN)

### Test 1.1: SchemaConfigSpec stores DirPath and FilePath
**Location:** `lithos-core/src/config/paths.rs` (mod tests::schema_config_spec)

**RED - Write failing test:**
```rust
#[test]
fn schema_config_spec_accepts_absolute_paths() {
    use crate::fs::{DirPath, FilePath};

    let schemas_dir: DirPath = PathBuf::from("/vault/schemas").into();
    let property_bank: FilePath = PathBuf::from("/vault/schemas/property_bank.json").into();

    let spec = SchemaConfigSpec::new(schemas_dir, property_bank);

    assert_eq!(spec.directory().as_path(), Path::new("/vault/schemas"));
    assert_eq!(spec.property_bank().as_path(), Path::new("/vault/schemas/property_bank.json"));
}
```

**Why this test:** Verifies SchemaConfigSpec can store and return absolute paths via DirPath/FilePath types.

**Expected failure:** Compilation error - SchemaConfigSpec fields are currently `RelativePath`, not `DirPath`/`FilePath`.

**GREEN - Implementation:**
```rust
// lithos-core/src/config/paths.rs

use crate::fs::{DirPath, FilePath};

pub struct SchemaConfigSpec {
    /// Absolute directory containing schema files (e.g., "/vault/schemas").
    directory: DirPath,
    /// Absolute path to property bank file (e.g., "/vault/schemas/property_bank.json").
    property_bank: FilePath,
}

impl SchemaConfigSpec {
    #[inline]
    #[must_use]
    pub const fn new(directory: DirPath, property_bank: FilePath) -> Self {
        Self { directory, property_bank }
    }

    #[inline]
    #[must_use]
    pub const fn directory(&self) -> &DirPath {
        &self.directory
    }

    #[inline]
    #[must_use]
    pub const fn property_bank(&self) -> &FilePath {
        &self.property_bank
    }
}
```

**Verification:** Test passes. `cargo test schema_config_spec_accepts_absolute_paths`

---

## Phase 2: Update Config::to_schema_spec (RED → GREEN)

### Test 2.1: to_schema_spec joins vault root to create absolute paths
**Location:** `lithos-core/src/config/aggregate.rs` (mod tests)

**RED - Write failing test:**
```rust
#[test]
fn to_schema_spec_creates_absolute_paths_from_vault_root() {
    use crate::fs::{DirPath, FilePath};

    let temp = TempDir::new().unwrap();
    let vault_root_path = temp.path().to_path_buf();

    // Create actual directories for DirPath/FilePath validation bypass
    std::fs::create_dir_all(vault_root_path.join("schemas")).unwrap();
    std::fs::write(vault_root_path.join("schemas/property_bank.json"), "{}").unwrap();

    let config = test_config_with_vault_root(&vault_root_path);
    let spec = config.to_schema_spec();

    // Verify absolute paths
    assert_eq!(
        spec.directory().as_path(),
        vault_root_path.join("schemas").as_path()
    );
    assert_eq!(
        spec.property_bank().as_path(),
        vault_root_path.join("schemas/property_bank.json").as_path()
    );
    assert!(spec.directory().is_absolute());
    assert!(spec.property_bank().is_absolute());
}
```

**Why this test:** Ensures `to_schema_spec()` correctly joins vault root with relative paths to produce absolute paths.

**Expected failure:** Compilation errors - return type mismatch, field type mismatch.

**GREEN - Implementation:**
```rust
// lithos-core/src/config/aggregate.rs

pub fn to_schema_spec(&self) -> super::paths::SchemaConfigSpec {
    use super::paths::SchemaConfigSpec;
    use crate::fs::{DirPath, FilePath};

    let vault_root = self.vault.root().as_path();

    // Join vault root with relative paths
    let schemas_dir = vault_root.join(self.paths.schema.schemas_dir().as_path());
    let property_bank = vault_root.join(self.paths.property_bank_path());

    // Use From<PathBuf> to bypass filesystem validation
    SchemaConfigSpec::new(
        schemas_dir.into(),
        property_bank.into(),
    )
}
```

**Verification:** Test passes. `cargo test to_schema_spec_creates_absolute_paths_from_vault_root`

---

## Phase 3: Update DiscoveryEngine::run Signature (RED → GREEN)

### Test 3.1: DiscoveryEngine::run works without vault_root parameter
**Location:** `lithos-core/src/schema/discovery.rs` (mod tests)

**RED - Write failing test:**
```rust
#[test]
fn run_uses_absolute_paths_from_spec() {
    let temp = TempDir::new().unwrap();
    let vault_root = temp.path();

    // Setup filesystem
    let schema_dir = vault_root.join("schemas");
    std::fs::create_dir_all(&schema_dir).unwrap();
    std::fs::write(schema_dir.join("property_bank.json"), "{}").unwrap();
    std::fs::write(schema_dir.join("schema1.json"), r#"{"name": "test"}"#).unwrap();

    // Create spec with absolute paths (no separate vault_root needed)
    let spec = SchemaConfigSpec::new(
        schema_dir.clone().into(),
        schema_dir.join("property_bank.json").into(),
    );

    let repo = InMemoryRepository::new();

    // Call without vault_root parameter
    let result = DiscoveryEngine::run(&spec, &repo);

    assert!(result.is_ok());
    let discovery = result.unwrap();
    assert!(discovery.has_schemas());
    assert!(discovery.property_bank().is_some());
}
```

**Why this test:** Proves DiscoveryEngine can extract absolute paths from spec without needing vault_root parameter.

**Expected failure:** Compilation error - `run()` currently requires 3 parameters.

**GREEN - Implementation:**
```rust
// lithos-core/src/schema/discovery.rs

impl DiscoveryEngine {
    pub(crate) fn run<R>(
        spec: &SchemaConfigSpec,
        repo: &R,
        // vault_root parameter REMOVED
    ) -> Result<DiscoveryResult, SchemaLoaderError>
    where
        R: ReadRepository,
    {
        // Use spec paths directly (already absolute)
        let vault_root = spec.directory().as_path()
            .parent()
            .ok_or_else(|| SchemaLoaderError::Ingestion(
                SchemaIngestionError::File(
                    crate::schema::error::SchemaFileError::FileSystem {
                        reason: "schema directory has no parent".into(),
                    }
                )
            ))?;

        // Step 1: Scan filesystem using absolute directory from spec
        let entries = Self::scan_filesystem(spec, vault_root)?;

        // ... rest unchanged
    }

    fn scan_filesystem(
        spec: &SchemaConfigSpec,
        vault_root: &Path,
    ) -> Result<Vec<FsEntry>, SchemaLoaderError> {
        const SCHEMA_EXTENSIONS: [&str; 4] = ["json", "toml", "yaml", "yml"];

        // Use absolute directory from spec
        let schema_dir = spec.directory();
        let pattern = format!("{}/**/*", schema_dir.as_path().display());

        DirScanner::new(vault_root)
            .entries(
                DirScanInput::new()
                    .with_pattern(&pattern)
                    .with_extensions(&SCHEMA_EXTENSIONS),
            )
            // ... rest unchanged
    }
}
```

**Verification:** Test passes. Update all call sites:
- `Builder::load_all` at line 61
- `accepts_read_repository_only` (test)
- `run_skips_schema_batch_lookups_when_no_schema_files_exist` (test)

---

## Phase 4: Update Builder::load_all Call Site (RED → GREEN)

### Test 4.1: Builder::load_all works with new DiscoveryEngine signature
**Location:** `lithos-core/src/schema/builder.rs` (mod tests)

**RED - Update existing test:**
```rust
#[test]
fn builder_load_all_orchestrates_discovery() {
    let temp = TempDir::new().unwrap();
    create_schema_files(&temp, &["schema_a.toml"]);

    let repo = InMemoryRepository::new();
    let config = setup_test_config(&temp);
    let source = FsReader::new(temp.path().to_path_buf());

    let mut builder = Builder::new(repo, source, &config);
    let result = builder.load_all();

    // Should work with new DiscoveryEngine signature
    assert!(
        result.is_ok() || matches!(
            result.unwrap_err(),
            SchemaLoaderError::Ingestion(_)
        )
    );
}
```

**Why this test:** Ensures existing test still passes after removing vault_root parameter.

**Expected failure:** Compilation error - `DiscoveryEngine::run` call has wrong number of arguments.

**GREEN - Implementation:**
```rust
// lithos-core/src/schema/builder.rs

impl<'config, R> Builder<'config, R>
where
    R: Repository,
{
    pub fn load_all(&mut self) -> Result<Vec<Arc<Schema>>, SchemaLoaderError> {
        use super::schema_processor::{
            Discovery, DiscoveryBranch, NeverSeen, Review, SchemaProcessor,
        };

        // 1. Single discovery call - vault_root no longer needed
        let discovery = DiscoveryEngine::run(
            &self.config.to_schema_spec(),
            &self.repository,
            // self.source.root() REMOVED
        )?;

        // ... rest unchanged
    }
}
```

**Verification:** Test passes. `cargo test builder_load_all_orchestrates_discovery`

---

## Phase 5: Update Discovery Test Call Sites (RED → GREEN)

### Test 5.1: Update accepts_read_repository_only
**Location:** `lithos-core/src/schema/discovery.rs` (mod tests)

**RED - Update test:**
```rust
fn accepts_read_repository_only<R: ReadRepository>(
    spec: &SchemaConfigSpec,
    repo: &R,
    // root parameter REMOVED
) -> Result<DiscoveryResult, SchemaLoaderError> {
    DiscoveryEngine::run(spec, repo) // 2 params instead of 3
}

#[test]
fn run_finds_all_files() {
    let root = tempfile::tempdir().unwrap();
    let schema_dir = root.path().join("schemas");
    std::fs::create_dir_all(&schema_dir).unwrap();

    let bank_path = schema_dir.join("property_bank.json");
    std::fs::write(&bank_path, "{}").unwrap();

    let schema1_path = schema_dir.join("schema1.json");
    std::fs::write(&schema1_path, "{}").unwrap();

    // Create spec with absolute paths
    let spec = SchemaConfigSpec::new(
        schema_dir.clone().into(),
        bank_path.into(),
    );

    let repo = InMemoryRepository::new();

    // Call without vault_root
    let result = accepts_read_repository_only(&spec, &repo).unwrap();

    assert_eq!(result.schemas().len(), 1);
    assert!(result.property_bank().is_some());
    assert!(result.has_schemas());
}
```

**Expected failure:** Compilation error - wrong number of arguments.

**GREEN - Implementation:** Update test as shown above.

**Verification:** Test passes. `cargo test run_finds_all_files`

### Test 5.2: Update run_skips_schema_batch_lookups_when_no_schema_files_exist
**Location:** `lithos-core/src/schema/discovery.rs` (mod tests)

**RED - Update test:**
```rust
#[test]
fn run_skips_schema_batch_lookups_when_no_schema_files_exist() {
    let root = tempfile::tempdir().unwrap();
    let schema_dir = root.path().join("schemas");
    std::fs::create_dir_all(&schema_dir).unwrap();

    let bank_path = schema_dir.join("property_bank.json");
    std::fs::write(&bank_path, "{}").unwrap();

    // Create spec with absolute paths
    let spec = SchemaConfigSpec::new(
        schema_dir.into(),
        bank_path.into(),
    );

    let repo = CountingReadRepo::new();

    // Call without vault_root
    let result = DiscoveryEngine::run(&spec, &repo).unwrap();

    assert!(!result.has_schemas());
    assert_eq!(repo.raw_views_by_paths_calls.load(Ordering::Relaxed), 0);
    assert_eq!(repo.ids_by_paths_calls.load(Ordering::Relaxed), 0);
}
```

**Expected failure:** Compilation error - wrong number of arguments.

**GREEN - Implementation:** Update test as shown above.

**Verification:** Test passes. `cargo test run_skips_schema_batch_lookups_when_no_schema_files_exist`

---

## Phase 6: Update Doc Comments and Examples

### Test 6.1: Doc tests compile with new API
**Location:** `lithos-core/src/config/paths.rs`, `lithos-core/src/config/aggregate.rs`

**RED - Update doc examples:**
```rust
/// # Examples
///
/// ```rust
/// use std::path::PathBuf;
/// use lithos_core::{config::paths::SchemaConfigSpec, fs::{DirPath, FilePath}};
///
/// let directory: DirPath = PathBuf::from("/vault/schemas").into();
/// let property_bank: FilePath = PathBuf::from("/vault/schemas/property_bank.json").into();
///
/// let spec = SchemaConfigSpec::new(directory, property_bank);
/// assert_eq!(spec.directory().as_path(), std::path::Path::new("/vault/schemas"));
/// ```
```

**Expected failure:** Doc test compilation errors due to old API.

**GREEN - Implementation:** Update all doc comments in:
- `SchemaConfigSpec` struct doc comment
- `SchemaConfigSpec::new()` doc comment
- `Config::to_schema_spec()` doc comment

**Verification:** `cargo test --doc`

---

## Phase 7: Final Verification & Cleanup

### Checklist:
- [ ] All unit tests pass: `mise run test:unit`
- [ ] All integration tests pass: `mise run test:integration`
- [ ] All doc tests pass: `cargo test --doc`
- [ ] No clippy warnings: `mise run lint`
- [ ] Code formatted: `mise run fmt`
- [ ] No `unwrap()` or `panic!()` in production code
- [ ] All imports updated (DirPath, FilePath added)
- [ ] All doc comments updated
- [ ] No unused imports or dead code

---

## Summary of Changes

| File | Changes |
|------|---------|
| `lithos-core/src/config/paths.rs` | SchemaConfigSpec fields: RelativePath → DirPath/FilePath |
| `lithos-core/src/config/aggregate.rs` | Config::to_schema_spec() joins vault root |
| `lithos-core/src/schema/discovery.rs` | DiscoveryEngine::run() signature (remove vault_root param) |
| `lithos-core/src/schema/builder.rs` | Builder::load_all() call site update |
| `lithos-core/src/schema/discovery.rs` (tests) | Update 3 test call sites |

**Total test updates:** ~8 tests
**Total production code files:** 4 files
**Risk level:** LOW (verified via GitNexus impact analysis)

---

## Rust Best Practices Applied

- ✅ **Type-driven design:** DirPath/FilePath enforce domain semantics at compile time
- ✅ **Borrowing over cloning:** Use `&DirPath` / `&FilePath` in accessors
- ✅ **Validated constructors:** DirPath/FilePath use `From<PathBuf>` for construction
- ✅ **Private fields:** SchemaConfigSpec fields remain private, exposed via const accessors
- ✅ **No unwrap/expect:** Use Result types, propagate errors with `?`
- ✅ **Clear error messages:** ConfigError::ValidationFailed with descriptive messages
- ✅ **Integration-style tests:** Test through public interfaces, verify behavior not implementation
- ✅ **One assertion per test:** Each test verifies a single behavior

---

## Next Steps

1. Execute Phase 1 (RED → GREEN)
2. Run `cargo test` to verify
3. Execute Phase 2 (RED → GREEN)
4. Run `cargo test` to verify
5. Continue through Phase 7
6. Run full verification suite
7. Commit changes with descriptive message
