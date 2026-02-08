---
name: 'step-03-document-crate'
description: 'Generate crate-level documentation (lib.rs/main.rs)'
nextStepFile: './step-04-document-modules.md'
---

# Step 3: Document Crate Level (lib.rs / main.rs)

## STEP GOAL:

Generate RFC 1574 compliant crate-level documentation using inner doc comments (`//!`).

## MANDATORY EXECUTION RULES (READ FIRST):

### Universal Rules:

- 📖 Read the complete step file before taking any action
- ✅ Speak in `{communication_language}`
- 🎯 Use inner doc comments `//!` ONLY for crate level

### Role Reinforcement:

- ✅ You are a rustdoc specialist creating crate documentation
- ✅ Follow RFC 1574 crate-level conventions exactly

### Step-Specific Rules:

- 🎯 MUST use `//!` syntax (inner docs)
- 🎯 First line is the summary (appears in search)
- 🎯 Include: Summary, Features, Usage, Layout
- 🚫 NEVER use `///` for crate-level docs

## EXECUTION PROTOCOLS:

- 🎯 Follow the MANDATORY SEQUENCE exactly
- 📝 Generate documentation in proper format
- 💾 Append to output file

## CONTEXT BOUNDARIES:

- Available context: Analysis from Step 2, output file
- Focus: Crate-level documentation only
- Limits: Do not document modules/types yet

## MANDATORY SEQUENCE

### 1. Generate Crate Summary Line

**CRITICAL:** First line is the summary sentence (no highly technical jargon).

Format:
```rust
//! [One-line summary of crate purpose]
```

Example:
```rust
//! Fast and easy queue abstraction for async workloads.
```

### 2. Generate Detailed Description

Add detailed description explaining:
- What problem this crate solves
- Why use this crate over alternatives
- Key benefits/features

Format:
```rust
//!
//! [Detailed description - 1-3 paragraphs]
//! [Explain the "big What" and "big How"]
```

### 3. Generate Features Section

If applicable, list features:

Format:
```rust
//! # Features
//!
//! - [Feature 1]: [Brief description]
//! - [Feature 2]: [Brief description]
```

### 4. Generate Usage Section

**REQUIRED:** Provide copy-paste ready example:

Format:
```rust
//! # Usage
//!
//! ```
//! use [crate_name]::[main_type];
//!
//! [working example code]
//! ```
```

Guidelines for usage example:
- Include `use` statement
- Show realistic, working code
- Add inline comments explaining key points
- Use `?` not `unwrap()` for fallible operations
- Hide error handling boilerplate with `#` if needed:

```rust
//! # Usage
//!
//! ```
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use my_crate::MyType;
//!
//! let instance = MyType::new()?;
//! instance.do_work()?;
//! # Ok(())
//! # }
//! ```
```

### 5. Generate Layout Section

Document module structure:

Format:
```rust
//! # Layout
//!
//! At the top level, we have [main components].
//! There are submodules for [specific purposes]:
//! - [`module_name`] - [Brief description]
//!
//! [`module_name`]: module_name/index.html
```

### 6. Present Generated Documentation

Show {user_name} the complete crate documentation:

"**Crate-Level Documentation Generated**

```rust
[SHOW COMPLETE //! DOCUMENTATION]
```

**RFC 1574 Compliance Check:**
- ✅ Inner doc comments (`//!`)
- ✅ Summary line first
- ✅ Features section (if applicable)
- ✅ Usage example with `use` statement
- ✅ Layout overview
- ✅ Reference-style links for modules

Does this look correct? Any adjustments needed?"

### 7. Present MENU OPTIONS

Display: "**Select an Option:** [A] Advanced Elicitation [P] Party Mode [C] Continue"

#### Menu Handling Logic:

- IF A: Execute {advancedElicitationTask}
- IF P: Execute {partyModeWorkflow}
- IF C:
  - Update {outputFile}: append crate documentation under "## Crate Documentation" section
  - Update frontmatter: stepsCompleted: step-03-document-crate
  - Then load, read entire file, then execute {nextStepFile}
- IF Any other comments or queries: help user respond then [Redisplay Menu Options](#7-present-menu-options)

## 🚨 SYSTEM SUCCESS/FAILURE METRICS:

### ✅ SUCCESS:

- Crate documentation generated with `//!`
- Summary line is clear and concise
- Usage example is copy-paste ready
- Layout section documents module structure
- RFC 1574 compliant
- Output file updated

### ❌ SYSTEM FAILURE:

- Using `///` instead of `//!`
- Missing usage example
- No summary line
- Not RFC 1574 compliant
- Output not saved
