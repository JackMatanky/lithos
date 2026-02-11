---
name: "template-validation-report"
description: "Template for rustdoc validation reports"
---

# Rustdoc Validation Report Template

````markdown
---
project: {project_name}
target: [target_path]
date: [current date]
mode: validate
compliance:
  rfc1574: [percentage]%
  components:
    crate: [pass/fail]
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
**RFC 1574 Compliance:** [percentage]%
**Status:** [PASS / NEEDS IMPROVEMENT / FAIL]

### Component Summary

| Component | Count | Pass | Fail | Compliance |
| --------- | ----- | ---- | ---- | ---------- |
| Crate     | 1     | [x]  | [x]  | [x]%       |
| Modules   | [n]   | [x]  | [x]  | [x]%       |
| Structs   | [n]   | [x]  | [x]  | [x]%       |
| Enums     | [n]   | [x]  | [x]  | [x]%       |
| Functions | [n]   | [x]  | [x]  | [x]%       |
| Traits    | [n]   | [x]  | [x]  | [x]%       |

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
````
