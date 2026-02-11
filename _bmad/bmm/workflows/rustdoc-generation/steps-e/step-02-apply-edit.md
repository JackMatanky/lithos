---
name: 'step-02-apply-edit'
description: 'Apply rustdoc edits with RFC 1574 compliance'
---

# Step 2: Apply Edits (Edit Mode)

## STEP GOAL:

Apply specific rustdoc edits while maintaining RFC 1574 compliance and verify changes are actually applied to target files.

## MANDATORY EXECUTION RULES (READ FIRST):

### Universal Rules:

- 📖 Read the complete step file before taking any action
- ✅ Speak in `{communication_language}`
- 🎯 Follow RFC 1574 conventions exactly
- 🔍 Verify edits are actually applied

### Role Reinforcement:

- ✅ You are a rustdoc specialist
- ✅ Apply precise, targeted edits
- ✅ Ensure no unintended side effects

### Step-Specific Rules:

- 🎯 Apply ONLY confirmed edits
- 🎯 Verify file changes after editing
- 🎯 Validate edited docs compile correctly

## EXECUTION PROTOCOLS:

- 🎯 Follow the MANDATORY SEQUENCE exactly
- ✅ Apply edits systematically
- 🔍 Verify file integrity after changes
- 📊 Generate edit report

## CONTEXT BOUNDARIES:

- Available context: Target file and edit scope from previous step
- Focus: Apply and verify edits
- Limits: Only modify confirmed targets

## MANDATORY SEQUENCE

### 1. Load Context

Load the target file and edit scope confirmed in the previous step.

### 2. Apply Edits Systematically

Based on the edit type selected in step 1:

**IF Add missing:**
- For each undocumented item, generate appropriate doc comments
- Follow RFC 1574 templates from `templates/rustdoc-standards.md`
- Ensure proper syntax (`//!` for crate/modules, `///` for items)

**IF Improve existing:**
- Enhance existing doc comments following RFC 1574
- Add missing sections (# Examples, # Panics, # Errors, # Safety)
- Improve clarity and completeness

**IF Fix issues:**
- Address specific RFC 1574 violations
- Fix syntax errors, missing sections, incorrect patterns
- Ensure all examples compile correctly

**IF Specific item:**
- Focus edits on the confirmed target item
- Apply comprehensive documentation to that item only

**IF Review all:**
- Systematically review and edit each doc comment
- Ensure full RFC 1574 compliance across the file

### 3. Verify Edit Application

CRITICAL STEP - Verify that changes are actually applied:

```bash
# Read the modified file
cat {target_file}

# Check that doc comments are present
grep -n "///\|//!"
```

### 4. Validate Documentation Compiles

Ensure the documentation actually works:

```bash
# Generate docs to verify syntax
cargo doc --no-deps

# Test doc examples compile
cargo test --doc {target_file}
```

### 5. Generate Edit Report

Create detailed edit report at: `{output_folder}/rustdoc-reports/rustdoc-edit-{target-file}-{timestamp}.md`

```yaml
---
project: {project_name}
date: [current date]
targetFile: [file edited]
editType: [add/improve/fix/specific/review]
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
```

### 6. Present Results to User

Show {user_name}:

"**Edit Complete!**

**Target:** [file_path]
**Changes Applied:**
- Added: [count] doc comments
- Improved: [count] existing docs
- Fixed: [count] RFC 1574 violations

**Verification Results:**
- Syntax check: [PASS/FAIL]
- Example compilation: [PASS/FAIL]
- Application verified: [YES/NO]

**Report saved to:** `{output_folder}/rustdoc-reports/rustdoc-edit-{target-file}-{timestamp}.md`

Would you like to:
- **[V]iew** the modified file
- **[R]eport** - See detailed edit report
- **[C]ontinue** - Edit another item
- **[Q]uit** - End edit session"

### 7. Handle User Choice

**IF V:**
- Display the modified file with changes highlighted
- Return to choice menu

**IF R:**
- Show complete edit report
- Return to choice menu

**IF C:**
- Return to step 1 to identify new edit targets

**IF Q:**
- End edit session gracefully

## 🚨 SYSTEM SUCCESS/FAILURE METRICS:

### ✅ SUCCESS:

- All confirmed edits applied
- Edits verified in target file
- Documentation compiles correctly
- Examples test successfully
- Edit report generated

### ❌ SYSTEM FAILURE:

- Edits not applied to file
- Documentation fails to compile
- Examples don't compile
- No verification performed
- No edit report generated
