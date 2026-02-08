---
name: 'step-02-analyze'
description: 'Deep analysis of code for documentation requirements'
nextStepFile: './step-03-document-crate.md'
---

# Step 2: Analyze Documentation Requirements

## STEP GOAL:

Perform deep analysis of each component to identify documentation requirements, special sections needed, and cross-references.

## MANDATORY EXECUTION RULES (READ FIRST):

### Universal Rules:

- 📖 Read the complete step file before taking any action
- ✅ Speak in `{communication_language}`
- 🎯 Follow RFC 1574 conventions for all analysis

### Role Reinforcement:

- ✅ You are a rustdoc specialist analyzing for documentation needs
- ✅ Consider WHAT, HOW, and edge cases for each component

### Step-Specific Rules:

- 🎯 Analyze each component type according to RFC 1574
- 🎯 Identify required sections: Examples, Panics, Errors, Safety
- 🎯 Note cross-references between types
- 🚫 DO NOT write documentation yet

## EXECUTION PROTOCOLS:

- 🎯 Follow the MANDATORY SEQUENCE exactly
- 📊 Create analysis matrix for all components
- 🔗 Map intra-doc links needed

## CONTEXT BOUNDARIES:

- Available context: Code loaded in Step 1, output file with frontmatter
- Focus: Analysis only, no documentation writing
- Limits: Analysis must be complete before proceeding

## MANDATORY SEQUENCE

### 1. Analyze Crate-Level Requirements

Document in analysis notes:

**Crate Documentation Needs:**
- [ ] One-line summary (WHAT is this crate?)
- [ ] Detailed description (HOW does it help users?)
- [ ] Features list (if applicable)
- [ ] Usage example (copy-paste ready)
- [ ] Layout/structure overview

### 2. Analyze Module Requirements

For each module:
- [ ] One-line summary
- [ ] Relationship to parent/child modules
- [ ] Key types/functions exposed
- [ ] Cross-references to related modules

### 3. Analyze Struct Requirements

For each struct, identify:

**Required Documentation:**
- [ ] One-line summary (what it represents)
- [ ] When to use it
- [ ] Field documentation needs

**Special Sections Required:**
- [ ] # Examples (REQUIRED)
- [ ] # Panics (if methods can panic)
- [ ] # Errors (if methods return Result)
- [ ] Non-exhaustive? (#[non_exhaustive])

**Field Analysis:**
For each public field:
- [ ] Does the name explain itself?
- [ ] Are there invariants/constraints?
- [ ] Should it link to other types?

### 4. Analyze Enum Requirements

For each enum, identify:

**Required Documentation:**
- [ ] One-line summary (what it represents)
- [ ] When each variant is used

**Variant Analysis:**
For each variant:
- [ ] Clear description
- [ ] If variant has data, what does data represent?
- [ ] Examples for both common and edge cases

**Special Sections Required:**
- [ ] # Examples (REQUIRED - show match patterns)
- [ ] Non-exhaustive considerations

### 5. Analyze Function/Method Requirements

For each function/method, identify:

**Required Documentation:**
- [ ] One-line summary (third-person singular: "Returns", "Converts")
- [ ] Detailed explanation if behavior is non-obvious

**Special Sections Required:**
- [ ] # Examples (REQUIRED)
- [ ] # Panics (REQUIRED if edge cases cause panic)
- [ ] # Errors (REQUIRED for Result return types)
- [ ] # Safety (REQUIRED for unsafe functions)

**Documentation Anti-Patterns to AVOID:**
- [ ] Repeating type signature in docs
- [ ] Documenting the obvious
- [ ] Using "Parameters:" or "Returns:" sections (not RFC 1574 compliant)

### 6. Analyze Trait Requirements

For each trait, identify:

**Required Documentation:**
- [ ] One-line summary (what behavior it enables)
- [ ] Contract for implementors
- [ ] When should types implement this?

**Method Analysis:**
For each trait method:
- [ ] What does it do?
- [ ] Required vs provided method?
- [ ] Panic conditions
- [ ] Error conditions

### 7. Map Cross-References

Identify all intra-doc links needed:
- [ ] Types referenced in function signatures
- [ ] Related types users should know about
- [ ] Traits and their implementors

### 8. Present Analysis Summary

Present to {user_name}:

"**Documentation Analysis Complete**

Here's what each component needs:

**Crate Level:**
- Standard sections: Summary, Features, Usage

**Modules ([count]):**
- Each needs: Summary, high-level overview

**Structs ([count]):**
- All need: Summary, Examples
- With Panics section: [list]
- With Errors section: [list]

**Enums ([count]):**
- All need: Summary, variant docs, Examples
- Edge case examples needed: [list]

**Functions ([count]):**
- All need: Summary, Examples
- With Panics: [count]
- With Errors: [count]
- Unsafe (require Safety): [count]

**Traits ([count]):**
- All need: Summary, contract docs, Examples

**Cross-references to create:** [count] links

Ready to begin documentation generation?"

### 9. Present MENU OPTIONS

Display: "**Select an Option:** [A] Advanced Elicitation [P] Party Mode [C] Continue"

#### Menu Handling Logic:

- IF A: Execute {advancedElicitationTask}
- IF P: Execute {partyModeWorkflow}
- IF C: Update {outputFile} frontmatter (stepsCompleted: step-02-analyze), append analysis notes, then load, read entire file, then execute {nextStepFile}
- IF Any other comments or queries: help user respond then [Redisplay Menu Options](#9-present-menu-options)

## 🚨 SYSTEM SUCCESS/FAILURE METRICS:

### ✅ SUCCESS:

- All components analyzed
- Special sections identified per RFC 1574
- Cross-references mapped
- Analysis documented in output file
- Frontmatter updated

### ❌ SYSTEM FAILURE:

- Missing analysis for any component
- Not identifying required special sections
- No cross-reference mapping
- Proceeding without analysis completion
