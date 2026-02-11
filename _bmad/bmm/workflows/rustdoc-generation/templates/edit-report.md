---
name: "template-edit-report"
description: "Template for rustdoc edit reports"
---

# Rustdoc Edit Report Template

```markdown
---
project: {project_name}
targetFile: [file edited]
editType: [add/improve/fix/specific/review]
date: [current date]
status: [success/partial/failure]
changes:
  added: [count]
  improved: [count]
  fixed: [count]
verification:
  syntax: [pass/fail]
  examples: [pass/fail]
  applied: [verified/unverified]
---

# Rustdoc Edit Report

## Target Information

**File:** [file_path]
**Edit Type:** [add/improve/fix/specific/review]
**Date:** [date]

## Changes Applied

### Added Documentation
- [Item]: [Description of what was added]

### Improved Documentation
- [Item]: [Description of improvements]

### Fixed Issues
- [Item]: [Description of fixes applied]

## Verification Results

### Syntax Verification: [PASS/FAIL]
- `cargo doc` execution: [result]
- CommonMark compliance: [result]

### Example Verification: [PASS/FAIL]
- `cargo test --doc` execution: [result]
- Examples compile: [result]

### Application Verification: [VERIFIED/UNVERIFIED]
- Doc comments present in file: [result]
- Line numbers of applied docs: [list]

## Next Steps

1. Review the modified file
2. Run `cargo doc --open` to preview changes
3. Run `cargo test --doc` to verify examples
4. Consider addressing any remaining validation warnings
```
