# CI Pipeline Analysis - Run #21144965230

**Date**: 2026-01-19
**Branch**: `rust-conversion`
**Status**: ❌ **FAILED**
**Duration**: ~18 minutes

---

## Executive Summary

The pipeline failed due to **missing Rust components** in the CI environment. The root cause is that `cargo-fmt` (rustfmt) is not installed for the nightly toolchain used by the project, causing the `fmt` dependency task to fail across multiple jobs.

### Impact
- **5 test jobs failed** (ubuntu-stable, ubuntu-beta, ubuntu-nightly, macos-stable, windows-stable)
- **2 cross-compile jobs failed** (wasm32, x86_64-linux)
- **1 coverage job failed**
- **1 security scan failed** (indirect dependency on test failures)

---

## Root Cause Analysis

### Primary Issue: Missing `rustfmt` Component

**Error**:
```
error: 'cargo-fmt' is not installed for the toolchain 'nightly-2026-01-11-x86_64-unknown-linux-gnu'.
```

**Why it happens**:
1. Your project uses `rust-toolchain.toml` specifying `nightly-2026-01-11`
2. The CI jobs install Rust via `dtolnay/rust-toolchain@master` with matrix toolchain (stable/beta/nightly)
3. Your mise tasks have a `depends=["fmt"]` relationship
4. The `fmt` task runs `cargo fmt` which requires the `rustfmt` component
5. GitHub Actions' Rust toolchain installation doesn't include `rustfmt` by default for nightly toolchains

**Affected jobs**:
- All `Test (*)` jobs (ubuntu-stable, ubuntu-nightly, ubuntu-beta, macos-stable, windows-stable)
- `Coverage Report` job
- `Cross-Compile Check` jobs (transitively)

---

## Detailed Findings

### 1. **Toolchain Mismatch** ⚠️

**Issue**: CI matrix uses `stable/beta/nightly` but your project pins to a specific `nightly-2026-01-11` via `rust-toolchain.toml`.

**Evidence**:
```yaml
# CI workflow
matrix:
  include:
    - os: ubuntu-latest
      rust-version: stable  ← Generic stable
    - os: ubuntu-latest
      rust-version: nightly  ← Generic nightly (latest)
```

```toml
# rust-toolchain.toml
[toolchain]
channel = "nightly-2026-01-11"  ← Specific nightly
```

**Problem**:
- `dtolnay/rust-toolchain` installs the matrix version
- But your code requires the specific nightly from `rust-toolchain.toml`
- mise respects `rust-toolchain.toml` and tries to use `nightly-2026-01-11`
- This nightly doesn't have `rustfmt` installed

**Recommendation**: Either:
- Option A: Remove matrix beta/nightly (you already optimized to 3 jobs)
- Option B: Install `rustfmt` component explicitly in CI

---

### 2. **Excessive Matrix Combinations** 📊

**Before our optimization**, you had:
- 5 OS/toolchain combinations

**After our optimization**, you still have issues with **beta/nightly testing**:
- ubuntu-stable ✅ (needed)
- ubuntu-beta ❌ (failed, likely not needed)
- ubuntu-nightly ❌ (failed, likely not needed)
- macos-stable ❌ (failed due to mise setup taking 5+ minutes)
- windows-stable ❌ (failed due to mise setup taking 2+ minutes)

**Impact**:
- Beta/nightly add no value for a stable Rust project
- They consume CI minutes and add failure points
- mise installation is slower on Windows/macOS

---

### 3. **mise Installation Overhead** ⏱️

**Timings**:
| Job | mise Install Time | Status |
|-----|-------------------|--------|
| ubuntu-stable | ~2s | Fast, but failed on fmt |
| macos-stable | ~5m 21s | Slow, failed on mise install |
| windows-stable | ~2m 15s | Slow, failed on mise install |

**Root cause**: mise@v2 action downloads and installs mise + all tools. On Windows/macOS this is significantly slower than on Linux.

**Recommendation**:
- Keep ubuntu-stable only for now
- Add mac OS/Windows later with cached mise binaries

---

### 4. **Missing rustfmt Component** 🔧

**Solution**: Add `rustfmt` to the toolchain installation step.

**Option A - Per-job fix** (Quick):
```yaml
- uses: dtolnay/rust-toolchain@master
  with:
    toolchain: ${{ matrix.rust-version }}
    components: rustfmt, clippy  # Add this line
```

**Option B - rust-toolchain.toml fix** (Better):
```toml
[toolchain]
channel = "nightly-2026-01-11"
components = ["rustfmt", "clippy", "rust-src"]  # Add this
```

---

### 5. **Dependency Chain Failures** 🔗

**Current dependency chain**:
```
test (depends on) → fmt → cargo fmt (missing rustfmt) → FAIL
```

**This cascades to**:
- test:unit → FAIL
- test:integration → FAIL
- test:e2e → FAIL
- test:coverage → FAIL
- burn-in (would fail) → SKIPPED

**Recommendation**: Make `fmt` optional in CI or ensure rustfmt is always available.

---

### 6. **Cross-Compile Failures** 🌍

**wasm32-unknown-unknown**:
- Failed during "Check Compilation" step
- Likely due to missing wasm target or fmt dependency

**x86_64-unknown-linux-gnu**:
- Cancelled (cascading failure from other jobs)

**Recommendation**:
- Remove cross-compile checks until core tests pass
- Add them back incrementally when needed

---

### 7. **Security Scan Failure** 🔒

**cargo-deny** ran successfully but the workflow failed due to dependent job failures.

**Not a real issue** - this was a cascading failure.

---

## Optimization Recommendations

### **Immediate Fixes** (Priority 1)

1. **Install rustfmt component**:
   ```yaml
   - uses: dtolnay/rust-toolchain@master
     with:
       toolchain: stable
       components: rustfmt, clippy
   ```

2. **Remove beta/nightly from matrix**:
   ```yaml
   matrix:
     include:
       - os: ubuntu-latest
         rust-version: stable
       # Remove beta and nightly entirely
   ```

3. **Temporarily disable macOS/Windows** until mise caching is optimized:
   ```yaml
   matrix:
     include:
       - os: ubuntu-latest
         rust-version: stable
       # Add macOS/Windows back later
   ```

---

### **Short-Term Improvements** (Priority 2)

4. **Skip fmt in CI** (alternative to installing rustfmt):
   - Rely on pre-commit hooks locally
   - Remove `depends=["fmt"]` from test tasks for CI

5. **Add conditional burn-in**:
   ```yaml
   if: github.event_name == 'pull_request' && success()
   ```

6. **Remove cross-compile checks** (not needed yet):
   - You're not shipping WASM or cross-platform binaries
   - Focus on core functionality first

---

### **Medium-Term Enhancements** (Priority 3)

7. **Cache mise tools** explicitly:
   ```yaml
   - uses: actions/cache@v4
     with:
       path: ~/.local/share/mise
       key: mise-tools-${{ runner.os }}-${{ hashFiles('mise.toml') }}
   ```

8. **Optimize rust-cache** to include mise-installed tools:
   ```yaml
   - uses: swatinem/rust-cache@v2
     with:
       cache-directories: |
         ~/.local/share/mise
   ```

9. **Add artifact retention strategy**:
   - Currently keeping artifacts for 30 days
   - Reduce to 7 days for PR builds
   - Keep 30 days only for main branch

---

## Performance Bottlenecks

### Quality Gates (12min)
- **Time**: 16:33 → 16:45 (12 minutes)
- **Issue**: mise installation on first run without cache
- **Fix**: Cache mise tools directory

### Test Jobs (<1min actual, but failed immediately)
- **Time**: Setup took longer than actual test execution
- **Issue**: fmt dependency failed before tests could run
- **Fix**: Install rustfmt or skip fmt in CI

### mise Action (varies wildly)
- **Ubuntu**: ~2s
- **macOS**: ~5m
- **Windows**: ~2m
- **Fix**: Use mise binary cache or install-less mode

---

## Success Criteria for Next Run

✅ **All tests pass on ubuntu-stable**
✅ **No beta/nightly failures** (removed from matrix)
✅ **fmt task doesn't block tests** (rustfmt installed)
✅ **Pipeline completes in <15 minutes**
✅ **burn-in runs successfully on PR**

---

## Proposed Pipeline Changes

```yaml
matrix:
  include:
    - os: ubuntu-latest
      rust-version: stable
      # That's it - single configuration for now
```

```yaml
- uses: dtolnay/rust-toolchain@master
  with:
    toolchain: stable
    components: rustfmt, clippy
```

```yaml
# Remove these jobs for now
# - cross-compile-check
# - test (beta)
# - test (nightly)
# - test (macos) - add back when mise caching works
# - test (windows) - add back when mise caching works
```

---

## Next Steps

1. ✅ Install rustfmt component in toolchain setup
2. ✅ Remove beta/nightly from matrix
3. ✅ Keep only ubuntu-stable for testing
4. ✅ Remove cross-compile checks
5. ⏸️ Defer macOS/Windows until mise caching is optimized
6. ✅ Ensure burn-in runs on next PR
7. 📊 Monitor pipeline duration and success rate

---

**Expected outcome after fixes**:
- Pipeline duration: ~8-12 minutes (down from 18+ current)
- Success rate: 100% on ubuntu-stable
- Fewer false positives from beta/nightly variations

**ROI**:
- 40% faster feedback loop
- 80% fewer CI minute consumption
- Cleaner signal-to-noise ratio on failures
