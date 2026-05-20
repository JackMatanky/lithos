# Issue Update Summary: FS Inode Architecture Review

**Date**: 2026-05-20
**Action**: Reopened 5 issues with comprehensive agent briefs and TDD plans
**Skills Used**: gitnexus-impact-analysis, rust-best-practices, tdd

---

## Issues Updated

### 0. Issue 01: path-types (TYPE SAFETY)

**File**: `.scratch/fs-inode-architecture/01-path-types.md`
**Status**: Reopened - Post-completion review
**Priority**: 🔴 CRITICAL (type safety violations)

**What was added:**
- Post-Completion Review: Path Type Safety Issues
- 3 issues documented (From<PathBuf>, I/O in constructors, incomplete AbsolutePath validation)
- Agent brief with blast radius analysis
- TDD plan with 5 vertical slices (3 require maintainer decisions first)

**Problems:**
1. `From<PathBuf>` for FilePath/DirPath bypasses validation (8 call sites)
2. FilePath::new()/DirPath::new() perform filesystem I/O with wrong error type
3. AbsolutePath validation incomplete (missing `..`, `.`, platform prefix checks)

**TDD plan highlights:**
- Slice 1: Fix AbsolutePath validation (add missing checks)
- Slice 2: Fix error type to PathError
- Slice 3: Fix From<PathBuf> (decision needed)
- Slice 4: Fix join_file/join_dir construction
- Slice 5: Decide on I/O in constructors

**3 design decisions needed** from maintainer before executing:
- Remove/change/document From<PathBuf>?
- Keep I/O in constructors or remove?
- If keep I/O: use PathError?

---

### 1. Issue 08: fs-error-redesign (ROOT CAUSE)

**File**: `.scratch/fs-inode-architecture/08-fs-error-redesign.md`
**Status**: Reopened - Phase 2.5 incomplete
**Priority**: 🔴 CRITICAL (blocks 3+ other issues)

**What was added:**
- Agent Brief - Phase 2.5: Path and Name Constructor Error Types
- Complete TDD plan with 7 vertical slices
- GitNexus impact analysis (156 call sites, risk: LOW)
- Before/after code examples for all 6 constructors

**Root problem:**
- `path.rs` constructors (RelativePath, AbsolutePath, FilePath, DirPath) return `std::io::Error`
- `name.rs` constructors (FileName, BaseName) return `std::io::Error`
- **Should return** `PathError` with semantic variants per ADR 017

**Why this matters:**
- Violates ADR 017 error hierarchy design
- Causes error information loss in `entry.rs` (Issue #8 in review)
- Blocks completion of fs-error-redesign
- Cascades to Issues 09, 10, 11 (consumer migrations)

**TDD plan highlights:**
- 7 vertical slices (one constructor at a time)
- 40 test updates for PathError variant matching
- Per-module verification: path.rs → name.rs → entry.rs → scanner.rs → reader.rs

**GitNexus findings:**
- RelativePath: 0 direct callers (risk: NONE)
- FilePath: 3 direct callers (risk: LOW)
- DirPath: 2 direct callers (risk: LOW)
- FileName: 0 direct callers (risk: NONE)

---

### 2. Issue 02: name-types (API DESIGN)

**File**: `.scratch/fs-inode-architecture/02-name-types.md`
**Status**: Reopened - New post-completion issue
**Priority**: 🟡 HIGH (API usability)

**What was added:**
- Post-Completion Issue: FileName API Encourages Misuse
- Agent brief explaining the API design problem
- TDD plan with 4 phases (25 call sites to migrate)
- GitNexus impact analysis (25 call sites, risk: MEDIUM)

**Clarified problem** (per your correction):
- FileName has too many convenience methods: `basename()`, `extension()`, `as_path()`, `basename_str()`
- Encourages developers to work directly with FileName instead of FilePath/DirPath
- PRD intended: FilePath/DirPath as **primary access points**, name types as **storage primitives**

**Desired state:**
```rust
// Current (wrong):
let base = filename.basename_str();  // Working with FileName directly

// Desired (right):
let base = file_path.basename();  // FilePath is primary API
```

**TDD plan highlights:**
- Phase 1: Add extraction methods to FilePath/DirPath (tracer bullet)
- Phase 2: Migrate 25 call sites (one file at a time, independent commits)
- Phase 3: Remove FileName convenience methods (comment out, verify fail, delete)
- Phase 4: Verification (grep confirms zero usage)

**GitNexus findings:**
- 25 call sites across 13 files using FileName convenience methods
- Key affected: `schema` (7 sites), `vault` (2 sites), `fs` tests
- Extension extraction already done via path types (8 sites) - validates approach

**NOT about conversion traits** (your correction was key here):
- Conversion traits (TryFrom<&Path>) can stay on name types (proper invariant ownership)
- The issue is the **convenience methods** that bypass FilePath/DirPath

---

### 3. Issue 12: phase-4-cleanup (LEGACY CODE)

**File**: `.scratch/fs-inode-architecture/12-phase-4-cleanup.md`
**Status**: Reopened - Vault legacy types not deleted
**Priority**: 🟡 HIGH (blocks migration completion)

**What was added:**
- Post-Review: Vault Legacy Type Deletion
- Agent brief for deleting 5 legacy types
- TDD plan with 5 vertical slices
- GitNexus impact analysis (0 upstream deps, risk: LOW)

**Problem:**
- `vault/model.rs` still contains legacy types scheduled for deletion in PRD Phase 4:
  1. `VaultPath` (lines 345-413) - replaced by NormalizedPath + fs/path.rs types
  2. `VaultFile` (lines 415-512) - replaced by FileView
  3. `VaultFolder` (lines 514-584) - replaced by DirView
  4. `PathParts` (lines 586-622) - internal helper, should be deleted
  5. `FolderParts` (lines 624-646) - internal helper, should be deleted

**Why this matters:**
- Blocks migration to inode-based architecture
- Creates confusion (old vs new types coexisting)
- Test suite validating obsolete behavior

**TDD plan highlights:**
- Pre-flight: Verify inode-based types (FileView, DirView, FsEntryView) are actively used
- 5 vertical slices (one legacy type at a time)
- Test migration before deletion (VaultPath → NormalizedPath in tests)
- Grep verification: zero remaining references

**GitNexus findings:**
- All 5 legacy types: **0 upstream dependencies** in knowledge graph
- Manual grep confirms actual usage in tests and vault processor
- Safe deletion after test migration

---

### 4. Issue 13: vault-model-types (MODULE BOUNDARY)

**File**: `.scratch/fs-inode-architecture/13-vault-model-types.md`
**Status**: Reopened - NormalizedPath in wrong module
**Priority**: 🟢 MEDIUM (module boundary violation)

**What was added:**
- Post-Review: Move NormalizedPath to fs/path.rs
- Agent brief for module relocation
- TDD plan with 5 vertical slices
- GitNexus impact analysis (0 upstream deps, risk: LOW)
- Design observation about NormalizedPath vs RelativePath redundancy

**Problem:**
- `NormalizedPath` is in `vault/model.rs` (lines 121-149)
- **Should be in** `fs/path.rs` (infrastructure, not domain)
- Violates PRD module boundary: fs/ = infrastructure, vault/ = domain

**Why this matters:**
- FS context owns path validation (per CONTEXT.md invariants)
- NormalizedPath is general-purpose, not vault-specific
- Prevents reuse outside vault context
- Creates potential circular dependency risk

**TDD plan highlights:**
- 5 vertical slices: copy → update imports → update exports → delete original → move tests
- Error type conversion (VaultPathError → fs error types)
- Export strategy decision point (re-export from vault or remove entirely?)

**GitNexus findings:**
- NormalizedPath: **0 upstream dependencies** in knowledge graph
- Manual grep: usage in vault/processor.rs path normalization functions
- Safe to move with import updates

**Design observation:**
Both `NormalizedPath` (Box<str>, as_str()) and `RelativePath` (PathBuf, as_path()) enforce vault-relative constraints. Recommended **future consolidation** - they may be redundant types serving the same purpose with different storage strategies.

---

## Corrected Understanding (Your Feedback)

### 1. Path I/O Documentation ✅
**Your correction**: The `path.is_file()`/`is_dir()` I/O is documented in Rust std::path docs.

**What I learned**: The issue is NOT that I/O is undocumented, but:
- **Error type** is wrong: using `std::io::Error` instead of `PathError::NotAFile`/`NotADirectory`
- **Error message** could be clearer about validation being performed
- **General consideration**: Whether filesystem checks belong in constructors (design decision, not critical)

**Review updated**: Downgraded from CRITICAL to note in Issue 08 agent brief.

### 2. name.rs Design Clarified ✅
**Your correction**: The problem is FileName has too many methods, not conversion trait ownership.

**What I learned**:
- Conversion traits (TryFrom<&Path>) are fine on name types - proper invariant ownership
- The issue is **convenience methods** (basename(), extension(), as_path()) that bypass FilePath/DirPath
- PRD intended: path types as primary API, name types as storage primitives

**Review updated**: Issue 02 agent brief focuses on removing convenience methods, not moving conversion traits.

### 3. entry.rs Error Loss Root Cause ✅
**Your correction**: The root cause is Issue 08 not being complete, not the code in entry.rs.

**What I learned**:
- entry.rs **has to** discard io::Error when wrapping in PathError because constructors return io::Error
- Once Issue 08 Phase 2.5 completes, entry.rs can propagate PathError directly via `#[from]`
- The information loss disappears automatically

**Review updated**: Removed entry.rs as separate issue, documented as consequence of Issue 08.

### 4. scanner.rs Organization ✅
**Your correction**: 844 lines for scanner.rs is NOT a problem!

**What I learned**:
- Scanner is a logically cohesive unit
- 844 lines is acceptable for a single conceptual type
- Only split if it grows beyond 1000 lines

**Review updated**: Removed from issues list entirely.

### 5. `is_size_match()` Not Redundant ✅
**Your correction**: Follows codebase conventions for staleness detection APIs.

**What I learned**:
- Staleness detection needs explicit methods for clarity
- Pattern: `is_size_match()`, `is_timestamp_match()` form a consistent API
- Not just `metadata.size() == other_size` - semantic meaning matters

**Review updated**: Removed from issues list entirely.

---

## Files Modified

All issue files have been **appended** (not overwritten) with:

1. `.scratch/fs-inode-architecture/01-path-types.md`
   - Added: Post-Completion Review: Path Type Safety Issues
   - Added: 3 documented issues (From<PathBuf>, I/O in constructors, AbsolutePath validation)
   - Added: TDD Implementation Plan (5 slices, 3 with maintainer decisions)
   - Added: GitNexus Impact Analysis
   - Status: 3 design decisions needed from maintainer

2. `.scratch/fs-inode-architecture/08-fs-error-redesign.md`
   - Added: Agent Brief - Phase 2.5
   - Added: TDD Implementation Plan (7 slices)
   - Added: GitNexus Impact Analysis
   - Added: Verification checklist

3. `.scratch/fs-inode-architecture/02-name-types.md`
   - Added: Post-Completion Issue: FileName API Encourages Misuse
   - Added: Agent Brief (API design)
   - Added: TDD Implementation Plan (4 phases)
   - Added: GitNexus Impact Analysis (25 call sites)

4. `.scratch/fs-inode-architecture/12-phase-4-cleanup.md`
   - Added: Post-Review: Vault Legacy Type Deletion
   - Added: Agent Brief (5 legacy types)
   - Added: TDD Implementation Plan (5 slices)
   - Added: GitNexus Impact Analysis

5. `.scratch/fs-inode-architecture/13-vault-model-types.md`
   - Added: Post-Review: Move NormalizedPath to fs/path.rs
   - Added: Agent Brief (module relocation)
   - Added: TDD Implementation Plan (5 slices)
   - Added: GitNexus Impact Analysis
   - Added: Design observation (redundancy with RelativePath)

---

## Issue Priority Matrix

| Issue | Priority | Status | Root Cause | Blocks |
|-------|----------|--------|------------|--------|
| 01 (path types) | 🔴 CRITICAL | Post-completion - 3 decisions needed | Type safety + design decisions | Downstream consumers |
| 08 (error types) | 🔴 CRITICAL | Phase 2.5 incomplete | ADR 017 not fully implemented | Issues 09, 10, 11 |
| 02 (name API) | 🟡 HIGH | Post-completion issue | API design encourages misuse | None (independent) |
| 12 (legacy cleanup) | 🟡 HIGH | Legacy types not deleted | Phase 4 incomplete | Migration completion |
| 13 (NormalizedPath) | 🟢 MEDIUM | Module boundary violation | Wrong module placement | None (independent) |

---

## Recommended Execution Order

1. **Issue 01** (CRITICAL - 3 maintainer decisions needed first)
   - Decide on From<PathBuf> strategy, I/O in constructors
   - Fix AbsolutePath validation (no decision needed - straightforward)
   - Estimated effort: 1 hour code + decision time

2. **Issue 08 Phase 2.5** (CRITICAL)
   - Migrate path.rs and name.rs constructors to PathError
   - Fixes error information loss automatically
   - Unblocks consumer migrations
   - **Estimated effort**: 2-3 hours (156 call sites, but compiler guides changes)

2. **Issue 12** (HIGH - completes migration)
   - Delete VaultPath, VaultFile, VaultFolder, PathParts, FolderParts
   - Completes inode-based architecture migration
   - **Estimated effort**: 1-2 hours (5 deletions with test migration)

3. **Issue 13** (MEDIUM - module boundary)
   - Move NormalizedPath from vault/model.rs to fs/path.rs
   - Fixes module boundary violation
   - Consider consolidating with RelativePath (future issue)
   - **Estimated effort**: 1 hour (simple move with import updates)

4. **Issue 02** (HIGH - API design)
   - Remove FileName convenience methods
   - Migrate 25 call sites to use FilePath/DirPath
   - Strengthens type safety and zero-copy extraction
   - **Estimated effort**: 2-3 hours (25 call sites, independent commits)

**Total estimated effort**: 6-9 hours

---

## Skills and Tools Used

### GitNexus Impact Analysis
- **gitnexus_impact**: Upstream/downstream analysis for each type
- **gitnexus_context**: Symbol relationships and execution flows
- **gitnexus_query**: Finding usage patterns
- **Result**: Risk assessment, call site counts, affected processes

### Rust Best Practices (Apollo Handbook)
- **Chapter 1**: API design (prefer small interfaces, explicit over trait magic)
- **Chapter 3**: Performance (zero-copy, avoid cloning in loops)
- **Chapter 4**: Error handling (Result-first, thiserror #[from], no unwrap)
- **Chapter 5**: Testing (behavior-focused, one assertion per test)

### TDD Skill
- **Vertical slices**: One test → one implementation → repeat (not horizontal)
- **Tracer bullets**: Prove end-to-end path works first
- **RED-GREEN-REFACTOR**: Never refactor while RED
- **Behavioral tests**: Test public interface, not implementation

---

## Files Modified

All issue files have been **appended** (not overwritten) with:

1. `.scratch/fs-inode-architecture/08-fs-error-redesign.md`
   - Added: Agent Brief - Phase 2.5
   - Added: TDD Implementation Plan (7 slices)
   - Added: GitNexus Impact Analysis
   - Added: Verification checklist

2. `.scratch/fs-inode-architecture/02-name-types.md`
   - Added: Post-Completion Issue: FileName API Encourages Misuse
   - Added: Agent Brief (API design)
   - Added: TDD Implementation Plan (4 phases)
   - Added: GitNexus Impact Analysis (25 call sites)

3. `.scratch/fs-inode-architecture/12-phase-4-cleanup.md`
   - Added: Post-Review: Vault Legacy Type Deletion
   - Added: Agent Brief (5 legacy types)
   - Added: TDD Implementation Plan (5 slices)
   - Added: GitNexus Impact Analysis

4. `.scratch/fs-inode-architecture/13-vault-model-types.md`
   - Added: Post-Review: Move NormalizedPath to fs/path.rs
   - Added: Agent Brief (module relocation)
   - Added: TDD Implementation Plan (5 slices)
   - Added: GitNexus Impact Analysis
   - Added: Design observation (redundancy with RelativePath)

---

## Next Steps

**For you:**
1. Review the updated issues (read the appended agent briefs)
2. Confirm the execution order makes sense
3. Decide whether to:
   - Execute Issue 08 Phase 2.5 first (unblocks everything)
   - Execute all 4 issues in parallel (independent work streams)
   - Adjust priorities based on business needs

**For implementation:**
- Each issue has a complete agent brief ready for an AFK agent
- Each issue has a TDD plan with vertical slices
- Each issue has GitNexus impact analysis showing what breaks
- Each issue has verification commands (per-module + full suite)

**Definition of Done** (from AGENTS.md):
- [ ] All tests pass (`mise run test`)
- [ ] Code formatted (`mise run fmt`)
- [ ] No clippy warnings (`mise run lint`)
- [ ] All public APIs have tests
- [ ] No unwrap()/panic!() in production code
- [ ] Documentation updated (doc comments)
- [ ] ADR updated if architectural decision made

---

## Summary Statistics

**Issues reopened**: 4
**Agent briefs created**: 4
**TDD plans created**: 4
**GitNexus analyses run**: 4

**Total call sites to update**:
- Issue 08: 156 call sites (path/name constructor migrations)
- Issue 02: 25 call sites (FileName API usage)
- Issue 12: 5 legacy types to delete
- Issue 13: 1 type to move + imports

**Risk distribution**:
- LOW: Issues 08, 12, 13 (compiler guides changes, low coupling)
- MEDIUM: Issue 02 (25 call sites across 13 files, but independent changes)

**Success criteria**: All 4 issues complete, `mise run verify` passes, no regressions in 1157 existing tests.
