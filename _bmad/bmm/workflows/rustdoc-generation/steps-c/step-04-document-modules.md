---
name: 'step-04-document-modules'
description: 'Generate module-level documentation'
nextStepFile: './step-05-document-types.md'
---

# Step 4: Document Modules

## STEP GOAL:

Generate RFC 1574 compliant module-level documentation using inner doc comments (`//!`).

## MANDATORY EXECUTION RULES (READ FIRST):

### Universal Rules:

- 📖 Read the complete step file before taking any action
- ✅ Speak in `{communication_language}`
- 🎯 Use inner doc comments `//!` for module level

### Role Reinforcement:

- ✅ You are a rustdoc specialist creating module documentation
- ✅ Module docs should be BROAD summaries, not detailed

### Step-Specific Rules:

- 🎯 MUST use `//!` syntax at module top
- 🎯 High-level summary only
- 🎯 Each type in module documents itself
- 🚫 NEVER use `//!` inside `mod` blocks

## EXECUTION PROTOCOLS:

- 🎯 Follow the MANDATORY SEQUENCE exactly
- 📝 Generate module docs in proper format
- 💾 Append to output file

## CONTEXT BOUNDARIES:

- Available context: Analysis from Step 2, crate docs complete
- Focus: Module-level documentation only
- Limits: Do not duplicate type-level docs

## MANDATORY SEQUENCE

### 1. Generate Module Summary Line

**CRITICAL:** First line is the summary sentence.

Format:
```rust
//! [One-line summary of module purpose]
```

Example:
```rust
//! Functions for working with filesystem paths and path components.
```

### 2. Generate Module Context

Add broader context:

Format:
```rust
//!
//! [How this module fits in the crate]
//! [When would users use this module?]
```

### 3. Generate Module Overview

List key items (broad strokes only):

Format:
```rust
//! This module provides:
//! - [`StructName`] - [Brief description]
//! - [`enum_name`] - [Brief description]
//! - [`function_name`] - [Brief description]
```

**CRITICAL RULE:** Keep module docs brief. Each item should document itself fully.

### 4. Generate Cross-References

Reference related modules if applicable:

Format:
```rust
//!
//! For [related functionality], see the [`other_module`] module.
//!
//! [`other_module`]: ../other_module/index.html
```

### 5. Module Documentation Placement

**CRITICAL:** Place `//!` comments at the TOP of the module file, BEFORE any items:

Correct:
```rust
//! Module summary here
//!
//! More details...

pub struct MyStruct;
```

**INCORRECT:** Do NOT use `//!` inside mod blocks:
```rust
// WRONG:
mod my_module {
    //! This module...
}

// CORRECT:
/// This module contains tests
mod my_module {
    // items...
}
```

### 6. Present Generated Documentation

Show {user_name} module documentation for each module:

"**Module Documentation Generated**

**Module: [module_name]**
```rust
[SHOW //! DOCUMENTATION]
```

**RFC 1574 Compliance Check:**
- ✅ Inner doc comments (`//!`) at module top
- ✅ Summary line first
- ✅ High-level overview only
- ✅ Types will document themselves
- ✅ Cross-references to related modules

Review each module documentation. Any adjustments needed?"

### 7. Present MENU OPTIONS

Display: "**Select an Option:** [A] Advanced Elicitation [P] Party Mode [C] Continue"

#### Menu Handling Logic:

- IF A: Execute {advancedElicitationTask}
- IF P: Execute {partyModeWorkflow}
- IF C:
  - Update {outputFile}: append module documentation under "## Module Documentation" section
  - Update frontmatter: stepsCompleted: step-04-document-modules
  - Then load, read entire file, then execute {nextStepFile}
- IF Any other comments or queries: help user respond then [Redisplay Menu Options](#7-present-menu-options)

## 🚨 SYSTEM SUCCESS/FAILURE METRICS:

### ✅ SUCCESS:

- All modules documented with `//!`
- Summary lines are clear and concise
- High-level overviews only
- No duplication of type-level docs
- RFC 1574 compliant
- Output file updated

### ❌ SYSTEM FAILURE:

- Using `///` instead of `//!` for modules
- Module docs too detailed (duplicating type docs)
- Using `//!` inside mod blocks
- Missing summary lines
- Not RFC 1574 compliant
