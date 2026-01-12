# Version Management Standards

**Owner:** Charlie (Senior Dev)
**Created:** 2026-01-12
**Status:** Active
**Epic:** Epic 2 Story 2.1 Preparation

---

## Overview

This document establishes version management standards for the Lithos project to prevent dependency conflicts, ensure reproducible builds, and provide clear resolution strategies when version issues arise.

**Context from Epic 1:**
- Version conflicts appeared in 30% of stories (3 out of 10)
- Conflicts: rkyv 0.8 feature mismatch, mise/rust-toolchain conflict, cargo-deny 0.19.0 breaking changes
- Root cause: Lack of explicit version management strategy
- Resolution: Establish documented policies and patterns

---

## 1. Rust Toolchain Management

### 1.1 rust-toolchain.toml Configuration

**Purpose:** Ensure deterministic Rust version across development, CI, and production environments.

**Location:** `rust-toolchain.toml` at repository root

**Current Configuration:**
```toml
[toolchain]
channel = "nightly-2024-12-01"
components = ["rustfmt", "clippy"]
targets = ["wasm32-unknown-unknown"]
profile = "default"
```

**Why Nightly:**
- Required for advanced rustfmt features (`imports_granularity`, `group_imports`)
- Pinned to specific nightly date for stability
- Mitigates nightly breakage risk through explicit version control

**Policy:**
- ✅ **DO:** Pin to specific nightly date (e.g., `nightly-2024-12-01`)
- ✅ **DO:** Update quarterly or when needed features become available
- ✅ **DO:** Test updates in CI before merging
- ❌ **DON'T:** Use unpinned `nightly` channel (causes non-deterministic builds)
- ❌ **DON'T:** Switch to stable unless rustfmt features stabilize

### 1.2 Handling mise vs dtolnay/rust-toolchain Conflicts

**Issue Encountered (Story 1.5):**
- CI uses `dtolnay/rust-toolchain` action
- Local development uses `mise` for tool management
- Conflict: Which toolchain takes precedence?

**Resolution Strategy:**
```bash
# In CI workflows, pass mise environment variable
env:
  MISE_RUST_VERSION: ${{ matrix.rust-version }}
```

**Policy:**
- ✅ **DO:** Use `rust-toolchain.toml` as single source of truth
- ✅ **DO:** Configure CI to respect rust-toolchain.toml
- ✅ **DO:** Pass `MISE_RUST_VERSION` when overriding is necessary
- ❌ **DON'T:** Have conflicting version specs in multiple places

---

## 2. Dependency Pinning Strategy

### 2.1 Workspace Dependency Management

**Location:** `Cargo.toml` `[workspace.dependencies]` section

**Pinning Levels:**

1. **Exact Version Pinning** (Use sparingly, only when necessary)
   ```toml
   # When: Breaking changes expected, API instability
   critical-dep = "=1.2.3"
   ```

2. **Caret Requirements** (Default, recommended)
   ```toml
   # Allows: 1.2.3 to <2.0.0 (semver compatible updates)
   tokio = "1.49"
   ```

3. **Tilde Requirements** (Conservative updates)
   ```toml
   # Allows: 1.2.3 to <1.3.0 (patch updates only)
   fragile-dep = "~1.2.3"
   ```

**Policy by Dependency Type:**

| Dependency Type | Strategy | Example | Rationale |
|----------------|----------|---------|-----------|
| **Core Runtime** | Caret | `tokio = "1.49"` | Stable, well-maintained, semver-compliant |
| **CLI Tools** | Caret | `clap = "4.5"` | Active development, backward compatible |
| **Security-Critical** | Exact (temporary) | `cargo-deny = "=0.19.0"` | Breaking changes identified, need explicit migration |
| **Build Tools** | Caret | `criterion = "0.5"` | Dev dependency, less critical |
| **Internal Crates** | Path | `lithos-domain = { path = "..." }` | Full control, version managed at workspace level |

### 2.2 Documenting Version Decisions

**Requirement:** All non-standard version choices MUST be documented in Cargo.toml comments.

**Example:**
```toml
[workspace.dependencies]
# rkyv 0.8 - Removed bytecheck_std feature due to compilation error
# See: Story 1.1 dev notes, resolved 2026-01-11
rkyv = { version = "0.8", features = ["bytecheck", "std"] }

# cargo-deny pinned to 0.19.0 due to breaking config changes
# Migration: Updated deny.toml syntax for 0.19.0 compatibility
# Review: Can upgrade after Epic 2 if 0.20+ stabilizes
cargo-deny = "0.19.0"
```

**Policy:**
- ✅ **DO:** Document WHY specific versions chosen
- ✅ **DO:** Reference story/issue where conflict was resolved
- ✅ **DO:** Note when to review for upgrades
- ✅ **DO:** Explain removed features or workarounds

---

## 3. Version Conflict Resolution Process

### 3.1 When Conflicts Occur

**Step-by-Step Resolution:**

1. **Identify Conflict Source**
   ```bash
   cargo tree -d            # Show duplicate dependencies
   cargo tree -i <crate>    # Show dependency path for specific crate
   ```

2. **Understand Semver Implications**
   - Major version change (1.x → 2.x): Breaking changes expected
   - Minor version change (1.2.x → 1.3.x): New features, backward compatible
   - Patch version change (1.2.3 → 1.2.4): Bug fixes only

3. **Check Changelog and Migration Guides**
   - Review crate's CHANGELOG.md or GitHub releases
   - Look for migration guides or breaking change documentation
   - Check for known issues in GitHub issues

4. **Attempt Resolution (Priority Order)**
   - **Option A:** Update to compatible version
   - **Option B:** Add feature flags to resolve conflict
   - **Option C:** Pin to working version temporarily
   - **Option D:** Find alternative crate (last resort)

5. **Document Resolution**
   - Add comment to Cargo.toml explaining decision
   - Update story dev notes with resolution details
   - Create ADR if decision has architectural impact

6. **Validate Resolution**
   ```bash
   cargo check --all-targets --all-features
   cargo test --workspace
   mise run verify
   ```

### 3.2 Example: rkyv 0.8 Feature Conflict (Story 1.1)

**Conflict:**
```
error: failed to compile rkyv v0.8.x
  feature 'bytecheck_std' not found
```

**Investigation:**
- Checked rkyv 0.8 changelog: `bytecheck_std` renamed to `bytecheck`
- Breaking change in feature naming

**Resolution:**
```toml
# Before (failed)
rkyv = { version = "0.8", features = ["bytecheck_std"] }

# After (working)
rkyv = { version = "0.8", features = ["bytecheck", "std"] }
```

**Documentation:**
- Added comment to Cargo.toml
- Documented in Story 1.1 dev notes
- No ADR needed (minor config fix)

### 3.3 Example: cargo-deny 0.19.0 Breaking Changes (Story 1.6)

**Conflict:**
- cargo-deny 0.19.0 introduced breaking config syntax changes
- `yanked` configuration moved from boolean to explicit enum

**Investigation:**
- Reviewed cargo-deny migration guide
- Identified syntax changes in deny.toml

**Resolution:**
```toml
# deny.toml - Updated syntax for 0.19.0
[advisories]
yanked = "deny"  # Changed from: yanked = true
```

**Documentation:**
- Documented in Story 1.6 dev notes
- Added TODO to review 0.20+ when stable
- Pinned version temporarily in Cargo.toml

---

## 4. Dependency Update Policy

### 4.1 Regular Dependency Audits

**Schedule:** Once per epic (during retrospective or sprint planning)

**Process:**
1. Run `cargo outdated` to identify available updates
2. Review changelogs for major/minor version changes
3. Prioritize security updates and critical bug fixes
4. Test updates in feature branch before merging
5. Update Cargo.lock with `cargo update`

**Commands:**
```bash
# Check for outdated dependencies
cargo outdated

# Update all dependencies respecting Cargo.toml constraints
cargo update

# Update specific dependency
cargo update -p <crate-name>

# Dry run to see what would change
cargo update --dry-run
```

### 4.2 Security Updates

**Priority:** HIGH - Address immediately

**Process:**
1. Monitor `cargo audit` output (integrated in CI via cargo-deny)
2. Review RustSec Advisory Database alerts
3. Update vulnerable dependencies within 48 hours
4. Document security updates in commit messages

**Policy:**
- ✅ **DO:** Prioritize security updates over feature development
- ✅ **DO:** Test thoroughly after security updates
- ✅ **DO:** Document CVE numbers in commit messages
- ❌ **DON'T:** Delay security updates "until next sprint"

### 4.3 Breaking Dependency Updates

**When upgrading dependencies with breaking changes:**

1. **Create Dedicated Story/Task**
   - Don't mix breaking updates with feature work
   - Allocate time for migration and testing

2. **Review Migration Path**
   - Read upgrade guides and breaking change lists
   - Identify affected code in codebase
   - Estimate migration effort

3. **Update Incrementally**
   - Update one major dependency at a time
   - Validate each update before moving to next
   - Commit after each successful migration

4. **Update Documentation**
   - Update ADRs if architectural impact
   - Update internal docs referencing old API
   - Add migration notes to Cargo.toml

---

## 5. Duplicate Dependency Management

### 5.1 Acceptable Duplicates

**Some duplicates are expected and acceptable:**

- Transitive dependencies with incompatible semver requirements
- Build-time vs runtime dependency differences
- Platform-specific variations (windows-sys, etc.)

**Current Known Duplicates (Epic 1):**
- `unicode-width`: Multiple versions due to transitive dependencies
- `windows-sys`: Platform-specific, multiple versions acceptable

**Policy:**
- ✅ **DO:** Document duplicates in deny.toml with skip rules
- ✅ **DO:** Review duplicates each epic for consolidation opportunities
- ✅ **DO:** Prioritize consolidation when easy (< 1 hour effort)
- ❌ **DON'T:** Spend excessive time forcing consolidation

### 5.2 Problematic Duplicates

**Red flags requiring action:**

- Same major version duplicated (e.g., tokio 1.48 and 1.49)
- Bloated binary size due to duplicates
- Conflicting behavior from multiple versions

**Resolution:**
```bash
# Identify duplicate sources
cargo tree -d

# Force unified version if possible
[patch.crates-io]
problematic-crate = { version = "1.2.3" }
```

---

## 6. Tools and Automation

### 6.1 Recommended Tools

| Tool | Purpose | Integration |
|------|---------|-------------|
| **cargo-outdated** | Check for newer dependency versions | Manual, run before epic planning |
| **cargo-deny** | Security auditing, license compliance | CI pipeline (Story 1.6, 1.10) |
| **cargo-audit** | RustSec vulnerability scanning | Integrated via cargo-deny |
| **cargo-tree** | Dependency graph analysis | Manual, for conflict investigation |
| **cargo-udeps** | Detect unused dependencies | Periodic cleanup (not in CI) |

### 6.2 CI Integration

**Automated Checks (from Story 1.10):**
```yaml
# .github/workflows/ci.yml
- name: Security Scan
  uses: EmbarkStudios/cargo-deny-action@v2
  with:
    command: check advisories licenses sources
```

**Policy:**
- ✅ **DO:** Fail CI on security vulnerabilities
- ✅ **DO:** Warn on license compliance issues
- ✅ **DO:** Report duplicate dependencies (informational)
- ❌ **DON'T:** Fail CI on all duplicates (too restrictive)

---

## 7. Lessons from Epic 1

### 7.1 Version Conflicts Encountered

| Story | Conflict | Resolution | Lesson Learned |
|-------|----------|------------|----------------|
| 1.1 | rkyv 0.8 `bytecheck_std` | Removed feature flag, used `bytecheck` + `std` | Read changelogs carefully during updates |
| 1.5 | mise vs rust-toolchain | Pass `MISE_RUST_VERSION` in CI | Single source of truth: rust-toolchain.toml |
| 1.6 | cargo-deny 0.19.0 syntax | Updated deny.toml syntax | Pin tools with breaking changes temporarily |

### 7.2 Preventive Measures for Epic 2+

1. **Before Starting Epic:**
   - Run `cargo outdated` and review
   - Update dependencies proactively in dedicated task
   - Document pinned versions with rationale

2. **During Epic:**
   - Add new dependencies through workspace.dependencies
   - Document version choices in Cargo.toml comments
   - Run `cargo tree -d` weekly to catch duplicates

3. **During Retrospective:**
   - Review version conflicts encountered
   - Update this document with new patterns
   - Plan dependency updates for next epic

---

## 8. Quick Reference

### 8.1 Common Commands

```bash
# Check current versions
cargo tree --depth 1

# Find outdated dependencies
cargo outdated

# Update dependencies
cargo update                    # Update all within Cargo.toml constraints
cargo update -p tokio           # Update specific crate
cargo update --dry-run          # Preview changes

# Analyze duplicates
cargo tree -d                   # Show duplicate dependencies
cargo tree -i serde             # Show why 'serde' is included

# Security auditing
cargo deny check                # Full audit (advisories, licenses, sources)
cargo deny check advisories     # Security vulnerabilities only

# Clean build
cargo clean
cargo check --all-targets --all-features
```

### 8.2 Decision Tree: Which Version Strategy?

```
Is this a new dependency?
├─ Yes → Use caret requirement (e.g., "1.2")
│        Document reason for inclusion
│
└─ No → Is it causing conflicts?
    ├─ Yes → Follow conflict resolution process (Section 3)
    │
    └─ No → Is it security-critical?
        ├─ Yes → Consider exact pinning temporarily
        │        Monitor for updates
        │
        └─ No → Keep existing strategy
                Review quarterly
```

---

## 9. Escalation and Review

### 9.1 When to Create an ADR

**Create ADR when:**
- Dependency choice has architectural implications
- Alternative crates considered (document why chosen)
- Breaking dependency update requires code refactoring
- Version pinning strategy changes workspace-wide

**Example ADRs:**
- ADR 0002: Storage (redb + rkyv) - Documented technology choice
- ADR 0003: Template Engine (minijinja) - Compared alternatives

### 9.2 Review Schedule

**This document should be reviewed:**
- After each epic retrospective
- When version conflicts occur (update patterns)
- Quarterly (ensure policies remain relevant)
- Before major dependency updates

**Owner:** Charlie (Senior Dev)
**Last Updated:** 2026-01-12
**Next Review:** After Epic 2 Retrospective

---

## 10. Nightly Rust Fallback Plan

### 10.1 If Nightly Breaks

**Symptoms:**
- rustfmt fails to compile
- New nightly introduces breaking changes
- CI pipeline fails on nightly channel

**Fallback Strategy:**

1. **Short-term (< 24 hours):**
   ```toml
   # Pin to last known working nightly
   channel = "nightly-2024-11-15"  # Example: previous working version
   ```

2. **Medium-term (< 1 week):**
   - Monitor rustfmt repository for fixes
   - Test newer nightlies in feature branch
   - Document issue in GitHub issue

3. **Long-term (if nightly unstable):**
   - Evaluate stable channel migration
   - Assess impact of losing advanced rustfmt features
   - Consider alternative formatting approaches

**Policy:**
- ✅ **DO:** Keep previous working nightly documented
- ✅ **DO:** Test nightly updates before deployment
- ✅ **DO:** Have rollback plan ready
- ❌ **DON'T:** Panic - pinning provides stability

---

## Appendix: Related Documentation

- [clippy.toml Configuration](../../clippy.toml)
- [rustfmt.toml Configuration](../../rustfmt.toml)
- [deny.toml Security Config](../../deny.toml)
- [rust-toolchain.toml](../../rust-toolchain.toml)
- [CI/CD Pipeline Documentation](../ci-cd.md)
- [ADR 0001: ADR Process](../adr/0001-adr-process.md)
- [Story 1.1 Dev Notes](_bmad-output/implementation-artifacts/stories/1-1-initialize-cargo-workspace-structure.md)
- [Story 1.5 Dev Notes](_bmad-output/implementation-artifacts/stories/1-5-configure-rustfmttoml-with-import-sorting.md)
- [Story 1.6 Dev Notes](_bmad-output/implementation-artifacts/stories/1-6-set-up-denytoml-for-dependency-security-auditing.md)
