---
name: 'step-01-init'
description: 'Initialize rustdoc workflow and load target code'
nextStepFile: './step-02-analyze.md'
---

# Step 1: Initialize Rustdoc Generation

## STEP GOAL:

Initialize the workflow, load the target Rust code, and establish documentation scope with {user_name}.

## MANDATORY EXECUTION RULES (READ FIRST):

### Universal Rules:

- 📖 Read the complete step file before taking any action
- ✅ Speak in `{communication_language}`
- 🛑 NEVER generate documentation without understanding the codebase

### Role Reinforcement:

- ✅ You are a rustdoc specialist and technical writer
- ✅ Follow RFC 1574 conventions religiously
- ✅ Component-type granularity is critical

### Step-Specific Rules:

- 🎯 MUST understand the codebase before documenting
- 🚫 DO NOT start writing doc comments yet
- 💬 Engage in collaborative discovery with {user_name}

## EXECUTION PROTOCOLS:

- 🎯 Follow the MANDATORY SEQUENCE exactly
- 📖 Load and analyze code structure before any documentation
- 💾 Create output file with frontmatter to track progress

## CONTEXT BOUNDARIES:

- Available context: target file path from initialization
- Focus: Code loading and scope definition
- Limits: No documentation generation yet
- Dependencies: Valid Rust file path

## MANDATORY SEQUENCE

### 1. Load Target Code

**CRITICAL:** Read the complete Rust file(s) at the target path provided during initialization.

If a directory was provided:
- List all `.rs` files
- Identify the crate root (`lib.rs` or `main.rs`)
- Note all module files

### 2. Analyze Code Structure

Analyze the loaded code and identify:

**Crate-Level Components:**
- [ ] Crate name and purpose
- [ ] Module hierarchy
- [ ] Public API surface

**Module-Level Components:**
- [ ] Each module's purpose
- [ ] Public items per module

**Type-Level Components:**
- [ ] Structs (with fields)
- [ ] Enums (with variants)
- [ ] Type aliases
- [ ] Constants

**Function-Level Components:**
- [ ] Public functions
- [ ] Methods (impl blocks)
- [ ] Unsafe functions (mark specially)

**Trait-Level Components:**
- [ ] Trait definitions
- [ ] Required methods
- [ ] Provided methods

### 3. Confirm Scope with User

Present findings to {user_name}:

"I've analyzed your codebase. Here's what I found:

**Crate:** [crate name]
**Modules:** [count] modules identified
**Structs:** [count] structs
**Enums:** [count] enums
**Functions:** [count] public functions
**Traits:** [count] traits
**Unsafe Items:** [count] unsafe functions

**Documentation Plan:**
1. Document crate-level (lib.rs/main.rs)
2. Document each module
3. Document types (structs, enums)
4. Document functions and methods
5. Document traits
6. Validate all documentation

Does this scope look correct? Would you like to:
- **[P]roceed** with all components
- **[E]xclude** certain items (specify which)
- **[F]ocus** on specific components only"

### 4. Create Output Tracking File

Create the output file at: `{output_folder}/rustdoc-{project_name}.md`

With frontmatter:
```yaml
---
project: {project_name}
created: [current date]
status: in-progress
stepsCompleted:
  - step-01-init
targetPath: [path provided]
components:
  crate: [true/false]
  modules: [count]
  structs: [count]
  enums: [count]
  functions: [count]
  traits: [count]
  unsafe: [count]
---
```

### 5. Present MENU OPTIONS

Display: "**Select an Option:** [A] Advanced Elicitation [P] Party Mode [C] Continue"

#### Menu Handling Logic:

- IF A: Execute {advancedElicitationTask}
- IF P: Execute {partyModeWorkflow}
- IF C: Update {outputFile} frontmatter with confirmed scope, then load, read entire file, then execute {nextStepFile}
- IF Any other comments or queries: help user respond then [Redisplay Menu Options](#5-present-menu-options)

#### EXECUTION RULES:

- ALWAYS halt and wait for user input after presenting menu
- ONLY proceed to next step when user selects 'C'
- After other menu items execution, redisplay the menu

## 🚨 SYSTEM SUCCESS/FAILURE METRICS:

### ✅ SUCCESS:

- Target code loaded and analyzed
- Component inventory complete
- Scope confirmed with user
- Output tracking file created
- Frontmatter initialized

### ❌ SYSTEM FAILURE:

- Proceeding without loading target code
- Missing component inventory
- No scope confirmation
- Output file not created
- Skipping to documentation phase
