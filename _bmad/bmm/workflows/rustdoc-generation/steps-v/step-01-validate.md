---
name: 'step-01-validate'
description: 'Validate existing rustdoc against RFC 1574 standards'
---

# Step 1: Validate Rustdoc (Validate Mode)

## STEP GOAL:

Validate existing rustdoc documentation against RFC 1574 standards and generate a comprehensive validation report.

## MANDATORY EXECUTION RULES (READ FIRST):

### Universal Rules:

- 📖 Read the complete step file before taking any action
- ✅ Speak in `{communication_language}`
- 🎯 Strict RFC 1574 compliance checking

### Role Reinforcement:

- ✅ You are a rustdoc validator
- ✅ Identify ALL violations, no matter how small

### Step-Specific Rules:

- 🎯 Component-type granularity for validation
- 🎯 Categorize issues: Critical vs Warning vs Info
- 🎯 Provide specific line numbers and fix suggestions

## EXECUTION PROTOCOLS:

- 🎯 Follow the MANDATORY SEQUENCE exactly
- 📊 Generate detailed validation report
- 💾 Save report to output file

## CONTEXT BOUNDARIES:

- Available context: Target path from initialization
- Focus: Validation only (no edits)
- Limits: Read-only analysis

## MANDATORY SEQUENCE

### 1. Load Target Code

Read the complete Rust file(s) at the target path.

If a directory:
- Read all `.rs` files
- Identify crate root (`lib.rs` or `main.rs`)

### 2. Load Documentation Standards

Read: `{project-root}/_bmad/_memory/tech-writer-sidecar/documentation-standards.md`
Read: `{project-root}/_bmad/bmm/workflows/rustdoc-generation/knowledge/rustdoc-standards.md`

### 3. Validate Crate-Level Documentation

Check the crate root file:

**RFC 1574 Checks:**
- [ ] Uses `//!` (inner doc comments) not `///`
- [ ] First line is a clear, concise summary
- [ ] No highly technical jargon in summary
- [ ] Includes usage example
- [ ] Example is copy-paste ready
- [ ] Includes `use` statement in example
- [ ] Layout section documents modules

**Common Violations:**
- Using `///` instead of `//!` for crate docs
- Missing usage example
- Summary too long or technical
- No layout overview

### 4. Validate Module Documentation

For each module file:

**RFC 1574 Checks:**
- [ ] Uses `//!` at TOP of file (not inside mod blocks)
- [ ] Summary line present
- [ ] High-level overview only
- [ ] Doesn't duplicate type-level docs

**Common Violations:**
- Using `//!` inside `mod { }` blocks
- Module docs too detailed
- No summary line

### 5. Validate Struct Documentation

For each struct:

**RFC 1574 Checks:**
- [ ] Uses `///` (outer docs)
- [ ] Summary line: what it represents
- [ ] All public fields have `///` docs
- [ ] Examples section present
- [ ] Examples compile (use `cargo test --doc`)
- [ ] Uses `?` not `unwrap()` in examples

**Common Violations:**
- Missing field documentation
- No examples section
- Summary describes fields instead of purpose
- Using "Returns:" or "Parameters:" sections

### 6. Validate Enum Documentation

For each enum:

**RFC 1574 Checks:**
- [ ] Uses `///` (outer docs)
- [ ] Summary line: what it represents
- [ ] All variants have `///` docs
- [ ] Data variants explain what data represents
- [ ] Examples section present
- [ ] Shows match patterns
- [ ] Edge cases shown (e.g., `None`)

**Common Violations:**
- Missing variant documentation
- No examples for edge cases
- No match pattern examples

### 7. Validate Function Documentation

For each function/method:

**RFC 1574 Checks:**
- [ ] Uses `///` (outer docs)
- [ ] Summary line: third-person singular ("Returns", "Converts")
- [ ] Does NOT use "Parameters:" section
- [ ] Does NOT use "Returns:" section
- [ ] Examples section present
- [ ] Examples include `use` statement
- [ ] Panics section (if applicable)
- [ ] Errors section (for Result types)
- [ ] Safety section (REQUIRED for unsafe)
- [ ] Examples use `?` not `unwrap()`

**Common Violations:**
- Using "Parameters:" or "Returns:" sections
- Not using third-person singular
- Missing Panics section
- Missing Safety section for unsafe
- Missing Errors section for Result
- Using `unwrap()` in examples

### 8. Validate Trait Documentation

For each trait:

**RFC 1574 Checks:**
- [ ] Uses `///` (outer docs)
- [ ] Summary: what behavior is enabled
- [ ] Contract documented for implementors
- [ ] All methods documented
- [ ] Examples show implementation

**Common Violations:**
- Missing contract documentation
- No implementation example
- Methods not documented

### 9. Validate Doc Comment Application

CRITICAL: Verify that doc comments are actually applied to source files:

**Application Verification:**
- [ ] Check target files contain expected `///` and `//!` comments
- [ ] Verify line numbers match expected locations
- [ ] Confirm no unintended side effects
- [ ] Check compilation still succeeds

**Verification Commands:**
```bash
# Count doc comments in target
grep -c "///\|//!" {target_file}

# Generate docs to verify syntax
cargo doc --no-deps

# Test examples compile
cargo test --doc
```

### 10. Validate Markdown Formatting

For all documentation:

**Checks:**
- [ ] CommonMark compliant
- [ ] Code blocks have language identifiers
- [ ] Intra-doc links use ``[`Type`]`` format
- [ ] Reference-style links preferred
- [ ] Proper heading hierarchy

### 11. Generate Validation Report

Create comprehensive report at: `{output_folder}/rustdoc-reports/rustdoc-validation-{project-name}-{target-file-or-folder}.md`

```yaml
---
project: {project_name}
date: [current date]
targetPath: [path validated]
status: [pass/fail/partial]
compliance:
  rfc1574: [percentage]%
  components:
    crate: [✅/❌]
    modules: [count pass/fail]
    structs: [count pass/fail]
    enums: [count pass/fail]
    functions: [count pass/fail]
    traits: [count pass/fail]
  verification:
    docCommentsApplied: [✅/❌]
    syntaxValid: [✅/❌]
    examplesCompile: [✅/❌]
---

# Rustdoc Validation Report

## Executive Summary

**Target:** [path]
**Date:** [date]
**RFC 1574 Compliance:** [percentage]%
**Status:** [PASS / NEEDS IMPROVEMENT / FAIL]

### Component Summary

| Component | Count | Pass | Fail | Compliance |
|-----------|-------|------|------|------------|
| Crate | 1 | [x] | [x] | [x]% |
| Modules | [n] | [x] | [x] | [x]% |
| Structs | [n] | [x] | [x] | [x]% |
| Enums | [n] | [x] | [x] | [x]% |
| Functions | [n] | [x] | [x] | [x]% |
| Traits | [n] | [x] | [x] | [x]% |

## Verification Status

### Doc Comment Application
- **Status:** [PASS/FAIL]
- **Doc comments found:** [count]
- **Expected:** [count]
- **Missing:** [count]

### Syntax Verification
- **Status:** [PASS/FAIL]
- **`cargo doc` result:** [success/failure]
- **Compilation errors:** [count]

### Example Compilation
- **Status:** [PASS/FAIL]
- **`cargo test --doc` result:** [success/failure]
- **Failed examples:** [count]

## Critical Issues (Must Fix)

### [Component]: [Name]
**Location:** [file.rs:line]
**Issue:** [Description]
**RFC 1574 Rule:** [Which rule is violated]
**Fix:** [Specific suggestion]

## Warnings (Should Fix)

### [Component]: [Name]
**Location:** [file.rs:line]
**Issue:** [Description]
**Recommendation:** [Suggestion]

## Recommendations

1. [Improvement suggestion]
2. [Improvement suggestion]

## Next Steps

1. Address critical issues
2. Review warnings
3. Run `cargo doc` to preview
4. Run `cargo test --doc` to verify examples
```

### 12. Present Validation Results

Show {user_name}:

"**Validation Complete!**

**Summary:**
- RFC 1574 Compliance: [percentage]%
- Critical Issues: [count]
- Warnings: [count]

**By Component:**
- Crate: [status]
- Modules: [pass]/[total]
- Structs: [pass]/[total]
- Enums: [pass]/[total]
- Functions: [pass]/[total]
- Traits: [pass]/[total]

**Report saved to:**
`{output_folder}/rustdoc-reports/rustdoc-validation-{project-name}-{target-file-or-folder}.md`

What would you like to do?
- **[F]ix Issues** - Edit mode to fix identified issues
- **[D]etails** - See full validation report
- **[Q]uit** - End validation"

### 13. Handle User Choice

**IF F:**
- Load `steps-e/step-01-assess.md` (edit mode)

**IF D:**
- Show full validation report
- Redisplay choice menu

**IF Q:**
- End workflow gracefully

## 🚨 SYSTEM SUCCESS/FAILURE METRICS:

### ✅ SUCCESS:

- All components validated
- RFC 1574 compliance measured
- Validation report generated
- Issues categorized (critical/warning)
- Specific fix suggestions provided

### ❌ SYSTEM FAILURE:

- Missing component validation
- No compliance measurement
- No validation report
- Issues not categorized
