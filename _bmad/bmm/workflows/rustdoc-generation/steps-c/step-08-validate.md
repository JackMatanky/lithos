---
name: 'step-08-validate'
description: 'Validate all generated documentation'
nextStepFile: './step-09-finalize.md'
---

# Step 8: Validate Documentation

## STEP GOAL:

Validate all generated rustdoc documentation against RFC 1574 standards and project requirements.

## MANDATORY EXECUTION RULES (READ FIRST):

### Universal Rules:

- 📖 Read the complete step file before taking any action
- ✅ Speak in `{communication_language}`
- 🎯 Validate against RFC 1574 strictly

### Role Reinforcement:

- ✅ You are a rustdoc validator ensuring compliance
- ✅ Component-type granularity for validation

### Step-Specific Rules:

- 🎯 Checklist validation per component type
- 🎯 Identify all violations
- 🎯 Suggest specific fixes

## EXECUTION PROTOCOLS:

- 🎯 Follow the MANDATORY SEQUENCE exactly
- ✅ Use validation checklist
- 📊 Generate validation report

## CONTEXT BOUNDARIES:

- Available context: All documentation generated in previous steps
- Focus: Validation and quality assurance
- Limits: Do not edit, only identify issues

## MANDATORY SEQUENCE

### 1. Load Documentation Standards

Load and read: `{project-root}/_bmad/_memory/tech-writer-sidecar/documentation-standards.md`

Also reference internal knowledge of RFC 1574 conventions.

### 2. Validate Crate-Level Documentation

Checklist for crate docs:
- [ ] Uses `//!` (inner doc comments)
- [ ] Summary line is clear and concise
- [ ] Detailed description explains purpose
- [ ] Features section (if applicable)
- [ ] Usage example is copy-paste ready
- [ ] Layout section documents module structure
- [ ] Reference-style links for modules

### 3. Validate Module-Level Documentation

Checklist for each module:
- [ ] Uses `//!` at module top (not inside mod blocks)
- [ ] Summary line is clear
- [ ] High-level overview only
- [ ] Does not duplicate type-level docs
- [ ] Cross-references to related modules

### 4. Validate Struct Documentation

Checklist for each struct:
- [ ] Uses `///` (outer doc comments)
- [ ] Summary line: what it represents
- [ ] All public fields have inline `///` docs
- [ ] Examples section present
- [ ] Examples use `?` not `unwrap()` where possible
- [ ] Panics section (if applicable)
- [ ] Errors section (if applicable)
- [ ] Intra-doc links for related types

### 5. Validate Enum Documentation

Checklist for each enum:
- [ ] Uses `///` (outer doc comments)
- [ ] Summary line: what it represents
- [ ] All variants have inline `///` docs
- [ ] Data variants explain what data represents
- [ ] Examples section present
- [ ] Both common and edge case examples (e.g., `None`)
- [ ] Match patterns shown in examples

### 6. Validate Function Documentation

Checklist for each function:
- [ ] Uses `///` (outer doc comments)
- [ ] Summary line: third-person singular ("Returns", "Converts")
- [ ] Does NOT use "Parameters:" section
- [ ] Does NOT use "Returns:" section
- [ ] Examples section present
- [ ] Examples include `use` statement
- [ ] Panics section (if applicable)
- [ ] Errors section (for Result return types)
- [ ] Safety section (REQUIRED for unsafe functions)
- [ ] Examples use `?` not `unwrap()` where possible

### 7. Validate Trait Documentation

Checklist for each trait:
- [ ] Uses `///` (outer doc comments)
- [ ] Summary line: what behavior is enabled
- [ ] Contract documented for implementors
- [ ] All required methods documented
- [ ] All provided methods documented
- [ ] Examples show complete implementation
- [ ] Panics documented for trait methods (if applicable)
- [ ] Errors documented for trait methods (if applicable)

### 8. Validate Markdown Formatting

Checklist for all docs:
- [ ] CommonMark compliant
- [ ] Code blocks have language identifiers
- [ ] Reference-style links preferred
- [ ] Intra-doc links use ``[`Type`]`` format
- [ ] Headers use proper hierarchy (no skipping levels)

### 9. Generate Validation Report

Create validation report in output file:

```markdown
## Validation Report

### Summary
- **Total Components:** [count]
- **Passed:** [count]
- **Issues Found:** [count]
- **RFC 1574 Compliance:** [percentage]%

### Component-Level Results

#### Crate Level
- Status: [✅/❌]
- Issues: [list or "None"]

#### Modules
- [Module Name]: [✅/❌] - [issues if any]

#### Structs
- [Struct Name]: [✅/❌] - [issues if any]

#### Enums
- [Enum Name]: [✅/❌] - [issues if any]

#### Functions
- [Function Name]: [✅/❌] - [issues if any]

#### Traits
- [Trait Name]: [✅/❌] - [issues if any]

### Critical Issues (Must Fix)
1. [Issue description and fix suggestion]

### Warnings (Should Fix)
1. [Issue description and fix suggestion]

### Recommendations
1. [Improvement suggestion]
```

### 10. Present Validation Results

Show {user_name} the validation summary:

"**Validation Complete**

**Summary:**
- Total Components: [count]
- RFC 1574 Compliance: [percentage]%
- Critical Issues: [count]
- Warnings: [count]

**Critical Issues Found:**
[list]

**Warnings:**
[list]

How would you like to proceed?
- **[F]ix Issues** - Go back and fix identified issues
- **[P]roceed** - Continue to finalization (fix issues later)
- **[R]eview Details** - See full validation report"

### 11. Handle User Choice

**IF F (Fix Issues):**
- Load `steps-e/step-01-fix-validation.md` (edit mode for fixes)

**IF P (Proceed):**
- Continue to next step

**IF R (Review Details):**
- Show full validation report from output file
- Then redisplay the choice menu

### 12. Present MENU OPTIONS

Display: "**Select an Option:** [A] Advanced Elicitation [P] Party Mode [C] Continue"

#### Menu Handling Logic:

- IF A: Execute {advancedElicitationTask}
- IF P: Execute {partyModeWorkflow}
- IF C:
  - Update {outputFile} frontmatter: stepsCompleted: step-08-validate
  - Then load, read entire file, then execute {nextStepFile}
- IF Any other comments or queries: help user respond then [Redisplay Menu Options](#12-present-menu-options)

## 🚨 SYSTEM SUCCESS/FAILURE METRICS:

### ✅ SUCCESS:

- All components validated against RFC 1574
- Validation report generated
- Issues categorized (critical/warning)
- Specific fix suggestions provided
- Frontmatter updated

### ❌ SYSTEM FAILURE:

- Missing validation for any component type
- No validation report
- Issues not categorized
- No fix suggestions
