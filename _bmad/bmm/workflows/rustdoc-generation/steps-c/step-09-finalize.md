---
name: 'step-09-finalize'
description: 'Finalize and output complete documentation'
---

# Step 9: Finalize Documentation

## STEP GOAL:

Finalize the rustdoc generation workflow, provide complete output, and summarize deliverables.

## MANDATORY EXECUTION RULES (READ FIRST):

### Universal Rules:

- 📖 Read the complete step file before taking any action
- ✅ Speak in `{communication_language}`
- 🎯 Provide complete, actionable output

### Role Reinforcement:

- ✅ You are a rustdoc specialist completing the workflow
- ✅ Ensure deliverables are clear and usable

### Step-Specific Rules:

- 🎯 Provide final documentation in usable format
- 🎯 Summarize what was accomplished
- 🎯 Give clear next steps

## EXECUTION PROTOCOLS:

- 🎯 Follow the MANDATORY SEQUENCE exactly
- 📝 Finalize output file
- ✅ Mark workflow as complete

## CONTEXT BOUNDARIES:

- Available context: All documentation generated and validated
- Focus: Finalization and delivery
- Limits: This is the final step

## MANDATORY SEQUENCE

### 1. Generate Final Output Summary

Update the output file with final section:

```markdown
## Final Output Summary

### Documentation Generated

**Crate Level:**
- Location: [lib.rs/main.rs path]
- Lines: [count]
- Status: [✅/⚠️]

**Modules:**
- [count] modules documented
- List: [module names]

**Structs:**
- [count] structs documented
- List: [struct names]

**Enums:**
- [count] enums documented
- List: [enum names]

**Functions:**
- [count] functions documented
- [count] methods documented

**Traits:**
- [count] traits documented
- List: [trait names]

### Compliance Summary

- **RFC 1574 Compliance:** [percentage]%
- **Critical Issues:** [count] remaining
- **Warnings:** [count] remaining

### Files Modified/Created

1. [Original source file] - [description of changes]
2. [Output documentation file] - [description]

### How to Use This Documentation

1. **Review the generated doc comments** in the output file
2. **Copy documentation** into your source files
3. **Run `cargo doc`** to generate HTML documentation
4. **Run `cargo test --doc`** to verify all examples compile
5. **Address any remaining validation issues**

### Next Steps

**Immediate:**
- [ ] Copy documentation into source files
- [ ] Run `cargo doc --open` to preview
- [ ] Run `cargo test --doc` to verify examples

**Optional:**
- [ ] Address remaining validation warnings
- [ ] Add additional examples for complex scenarios
- [ ] Set `#![deny(missing_docs)]` in lib.rs
```

### 2. Provide Complete Documentation

Ensure the output file contains all generated documentation organized as:

```
## Crate Documentation
[//! docs]

## Module Documentation
[//! docs for each module]

## Type Documentation
[/// docs for structs and enums]

## Function Documentation
[/// docs for functions and methods]

## Trait Documentation
[/// docs for traits]

## Validation Report
[validation results]

## Final Output Summary
[summary]
```

### 3. Update Frontmatter

Mark workflow as complete:

```yaml
---
project: {project_name}
created: [date]
completed: [current date]
status: complete
stepsCompleted:
  - step-01-init
  - step-02-analyze
  - step-03-document-crate
  - step-04-document-modules
  - step-05-document-types
  - step-06-document-functions
  - step-07-document-traits
  - step-08-validate
  - step-09-finalize
targetPath: [path]
components:
  crate: [true/false]
  modules: [count]
  structs: [count]
  enums: [count]
  functions: [count]
  traits: [count]
  unsafe: [count]
compliance:
  rfc1574: [percentage]%
  criticalIssues: [count]
  warnings: [count]
---
```

### 4. Present Final Summary to User

Display completion message:

"---

# 🎉 Rustdoc Generation Complete!

**Project:** {project_name}
**Status:** ✅ Complete

## What Was Generated

📦 **Crate Documentation** - Inner docs (`//!`) for [lib.rs/main.rs]
📁 **Module Documentation** - [count] modules documented
🏗️ **Struct Documentation** - [count] structs with field docs
🔀 **Enum Documentation** - [count] enums with variant docs
⚡ **Function Documentation** - [count] functions/methods
🔷 **Trait Documentation** - [count] traits with contracts

## RFC 1574 Compliance

**Overall Compliance:** [percentage]%
- ✅ Crate docs use `//!` correctly
- ✅ Module docs use `//!` at file top
- ✅ Type docs use `///` with examples
- ✅ Function docs use third-person singular
- ✅ Special sections (Panics/Errors/Safety) included

## Output Location

All documentation saved to:
`{output_folder}/rustdoc-{project_name}.md`

## Next Steps

1. **Copy the generated `///` and `//!` comments** into your Rust source files
2. **Run `cargo doc --open`** to preview the generated HTML
3. **Run `cargo test --doc`** to verify all examples compile
4. **Address any remaining validation issues** from the report

## Additional Resources

- **RFC 1574:** https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.html
- **Rustdoc Book:** https://doc.rust-lang.org/rustdoc/
- **API Guidelines:** https://rust-lang.github.io/api-guidelines/

---

Would you like to:
- **[V]alidate** the documentation again
- **[E]dit** specific sections
- **[Q]uit** - End the workflow
"

### 5. Handle User Choice

**IF V:**
- Return to validation step

**IF E:**
- Load edit mode workflow

**IF Q:**
- End workflow gracefully

## 🚨 SYSTEM SUCCESS/FAILURE METRICS:

### ✅ SUCCESS:

- All documentation generated and organized
- Output file updated with final summary
- Frontmatter marked as complete
- User provided with clear next steps
- All steps tracked in frontmatter

### ❌ SYSTEM FAILURE:

- Missing documentation sections
- Output file not properly organized
- Frontmatter not updated
- No clear next steps provided
