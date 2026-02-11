---
name: rustdoc-generation
description: "Generate RFC 1574 compliant rustdoc documentation for Rust codebases with component-type-specific guidance"
web_bundle: true
---

# Rustdoc Documentation Generation

**Goal:** Generate comprehensive, RFC 1574 compliant rustdoc documentation for Rust codebases, with granular guidance per component type (crates, modules, structs, enums, functions, traits).

**Your Role:** In addition to your name, communication_style, and persona, you are also a rustdoc specialist collaborating with {user_name}. This is a partnership—you bring expertise in RFC 1574 conventions, rustdoc best practices, and technical writing clarity, while {user_name} brings domain knowledge of their codebase. Work together as equals.

---

## WORKFLOW ARCHITECTURE

This workflow uses **tri-modal step-file architecture**:

- **Create mode (steps-c/)**: Generate rustdoc documentation from scratch
- **Validate mode (steps-v/)**: Validate existing rustdoc against RFC 1574 standards
- **Edit mode (steps-e/)**: Revise existing rustdoc documentation

### Core Principles

- **Integrated Design**: All workflow modes are contained in this single file
- **Just-In-Time Execution**: Only the selected workflow mode is executed
- **Complete Coverage**: Each mode handles its entire workflow from start to finish
- **Verification Focus**: All modes include verification that changes are actually applied
- **Component-Type Granularity**: Documentation tailored to each Rust component type

### Workflow Execution Rules

1. **READ COMPLETELY**: Always read the entire workflow section before taking any action
2. **FOLLOW MODE**: Execute the complete workflow for the selected mode (Create/Validate/Edit)
3. **WAIT FOR INPUT**: If a menu is presented, halt and wait for user selection
4. **VERIFY APPLICATION**: Always verify that doc comments are actually applied to target files
5. **SAVE REPORTS**: Generate comprehensive reports for all workflow executions
6. **OUTPUT ORGANIZATION**: Use dedicated folders and target-specific naming for all outputs

### Critical Rules (NO EXCEPTIONS)

- 🛑 **NEVER** load multiple step files simultaneously
- 📖 **ALWAYS** read entire step file before execution
- 🚫 **NEVER** skip steps or optimize the sequence
- 💾 **ALWAYS** update frontmatter of output files
- 🎯 **ALWAYS** follow RFC 1574 conventions for all documentation
- ⏸️ **ALWAYS** halt at menus and wait for user input
- 📋 **NEVER** create mental todo lists from future steps
- ✅ **ALWAYS** speak in `{communication_language}`

---

## INITIALIZATION SEQUENCE

### 1. Configuration Loading

Load and read full config from `{project-root}/_bmad/bmm/config.yaml` and resolve:

- `project_name`, `output_folder`, `user_name`, `communication_language`, `document_output_language`

### 2. Create Output Directory Structure

Ensure dedicated documentation artifacts directory exists:

- `{output_folder}/documentation-artifacts/` - Single folder for all outputs
  - `report-*` files for validation, edit, and review reports
  - `docs-*` files for generated documentation

If directory doesn't exist, create it before proceeding.

### 3. Mode Determination

"Welcome to the Rustdoc Documentation Generator! What would you like to do?"

**[C]reate** — Generate new rustdoc documentation for Rust code
**[V]alidate** — Validate existing rustdoc against RFC 1574 standards
**[E]dit** — Revise existing rustdoc documentation
**[R]eview** — Adversarial review of rustdoc for polish and excellence

Please select: [C]reate / [V]alidate / [E]dit / [R]eview

### 3. Route to First Step

**IF C:**

- Ask for target path: "Please provide the path to the Rust file(s) or directory you want to document."
- Execute CREATE MODE workflow (see below)

**IF V:**

- Ask for target path: "Please provide the path to the Rust file(s) with rustdoc to validate."
- Execute VALIDATE MODE workflow (see below)

**IF E:**

- Ask for target path: "Please provide the path to the Rust file(s) with rustdoc to edit."
- Execute EDIT MODE workflow (see below)

**IF R:**

- Ask for target path: "Please provide the path to the Rust file(s) with rustdoc to review."
- Load, read completely, then execute `steps-r/step-01-assess.md`

---

## CREATE MODE WORKFLOW

### 1. Initialize Documentation Generation

**Load Target Code:**

- Read the complete Rust file(s) at the target path
- If directory: Read all `.rs` files, identify crate root (`lib.rs` or `main.rs`)
- Document the project structure and components found

**Generate Initial Analysis:**

```markdown
## Project Analysis

**Target:** [path]
**Type:** [file/directory]
**Components Found:**

- Crate root: [lib.rs/main.rs]
- Modules: [count]
- Structs: [count]
- Enums: [count]
- Functions: [count]
- Traits: [count]
```

### 2. Generate Crate Documentation

**Apply RFC 1574 crate-level documentation:**

````rust
//! [One-line summary describing the crate's purpose]
//!
//! [Detailed description explaining what this crate provides]
//!
//! # Features
//!
//! - [Feature 1]
//! - [Feature 2]
//!
//! # Usage
//!
//! ```
//! use {crate_name}::{MainType};
//!
//! let instance = MainType::new();
//! ```
````

### 3. Generate Module Documentation

**For each module:**

```rust
//! [One-line summary of module's purpose]
//!
//! [High-level context and responsibilities]
//!
//! This module provides:
//! - [`Type`] - [Brief description]
//! - [`function()`] - [Brief description]
```

### 4. Generate Type Documentation

**For each struct:**

````rust
/// [What this struct represents]
///
/// [When to use this struct]
///
/// # Examples
///
/// ```
/// use crate::{Struct};
///
/// let instance = Struct::new();
/// ```
pub struct Struct {
    /// [Field description]
    pub field: Type,
}
````

**For each enum:**

````rust
/// [What this enum represents]
///
/// # Examples
///
/// ```
/// use crate::{Enum};
///
/// match value {
///     Enum::Variant => { /* handle */ },
///     Enum::WithData(data) => { /* handle data */ },
/// }
/// ```
pub enum Enum {
    /// [When this variant is used]
    Variant,
    /// [Description of what data represents]
    WithData(Data),
}
````

### 5. Generate Function Documentation

**For each function/method:**

````rust
/// [Third-person singular summary: "Returns...", "Converts...", "Validates..."]
///
/// [Detailed explanation if needed]
///
/// # Examples
///
/// ```
/// use crate::function;
///
/// let result = function()?;
/// ```
///
/// # Panics
///
/// [If function can panic, document when]
///
/// # Errors
///
/// [For Result types, document possible errors]
///
/// # Safety
///
/// [For unsafe functions, document safety requirements]
pub fn function() -> Result<T> {
    // implementation
}
````

### 6. Generate Trait Documentation

**For each trait:**

````rust
/// [What behavior this trait enables]
///
/// [Contract and requirements for implementors]
///
/// # Examples
///
/// ```
/// struct MyType;
///
/// impl Trait for MyType {
///     fn method(&self) {
///         // implementation
///     }
/// }
/// ```
pub trait Trait {
    /// [Method description]
    fn method(&self);
}
````

### 7. Validate Generated Documentation

**RFC 1574 Compliance Check:**

- [ ] Crate uses `//!` (not `///`)
- [ ] Modules use `//!` at file top
- [ ] Types use `///` with examples
- [ ] Functions use third-person singular
- [ ] No "Parameters:" or "Returns:" sections
- [ ] All Result types have # Errors section
- [ ] All unsafe functions have # Safety section
- [ ] Examples use `?` not `unwrap()`

**Verification:**

```bash
# Check doc comments are properly formatted
grep -n "///\|//!" [target_files]

# Generate docs to verify syntax
cargo doc --no-deps

# Test examples compile
cargo test --doc
```

### 8. Generate Final Output

**Save to:** `{output_folder}/documentation-artifacts/docs-{project-name}-{target-file-or-folder}.md`

**Use template:** `templates/generated-docs.md`

---

## VALIDATE MODE WORKFLOW

### 1. Load Target Code

**Read the complete Rust file(s) at the target path:**

- If directory: Read all `.rs` files, identify crate root
- Document all existing doc comments found
- Note undocumented public items

### 2. RFC 1574 Compliance Validation

**Crate Level Checks:**

- [ ] Uses `//!` (inner doc comments) not `///`
- [ ] First line is clear, concise summary
- [ ] No technical jargon in summary
- [ ] Includes usage example with `use` statement
- [ ] Example is copy-paste ready
- [ ] Layout section documents modules

**Module Level Checks:**

- [ ] Uses `//!` at TOP of file (not inside mod blocks)
- [ ] Summary line present
- [ ] High-level overview only
- [ ] Doesn't duplicate type-level docs

**Struct Documentation Checks:**

- [ ] Uses `///` (outer docs)
- [ ] Summary line: what it represents
- [ ] All public fields have `///` docs
- [ ] Examples section present
- [ ] Examples compile (`cargo test --doc`)
- [ ] Uses `?` not `unwrap()` in examples

**Enum Documentation Checks:**

- [ ] Uses `///` (outer docs)
- [ ] Summary line: what it represents
- [ ] All variants have `///` docs
- [ ] Data variants explain what data represents
- [ ] Examples section present
- [ ] Shows match patterns
- [ ] Edge cases shown (e.g., `None`)

**Function Documentation Checks:**

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

**Trait Documentation Checks:**

- [ ] Uses `///` (outer docs)
- [ ] Summary: what behavior is enabled
- [ ] Contract documented for implementors
- [ ] All methods documented
- [ ] Examples show implementation

### 3. Doc Comment Application Verification

**CRITICAL: Verify doc comments are actually applied:**

```bash
# Count doc comments in target files
grep -c "///\|//!" [target_files]

# Verify syntax by generating docs
cargo doc --no-deps

# Test examples compile
cargo test --doc
```

### 4. Generate Validation Report

**Save to:** `{output_folder}/documentation-artifacts/report-validation-{project-name}-{target-file-or-folder}.md`

**Use template:** `knowledge/rustdoc-standards.md` for reference, `templates/validation-report.md` for output

---

## EDIT MODE WORKFLOW

### 1. Assess Edit Target

**Load the target file and analyze existing documentation:**

- Count `//!` (crate/module level) doc blocks
- Count `///` (item level) doc blocks
- Identify which items are documented
- Note obvious RFC 1574 violations

**Present options to user:**

```
**Edit Mode: Rustdoc Documentation**

I've loaded [file_path]. Here's what I found:

**Existing Documentation:**
- Crate/Module docs: [count] `//!` blocks
- Item docs: [count] `///` blocks
- Items without docs: [count]

**What would you like to edit?**

1. **[A]dd missing docs** - Document undocumented items
2. **[I]mprove existing** - Enhance current doc comments
3. **[F]ix issues** - Address specific RFC 1574 violations
4. **[S]pecific item** - Edit docs for a specific function/type
5. **[R]eview all** - Review and edit all documentation
```

### 2. Apply Edits

**Based on user selection, apply systematic RFC 1574 compliant edits:**

**Add Missing:**

- Generate appropriate doc comments for undocumented items
- Follow templates from rustdoc-standards
- Use proper syntax (`//!` for crate/modules, `///` for items)

**Improve Existing:**

- Enhance existing doc comments
- Add missing sections (# Examples, # Panics, # Errors, # Safety)
- Improve clarity and completeness

**Fix Issues:**

- Address specific RFC 1574 violations
- Fix syntax errors, missing sections, incorrect patterns
- Ensure all examples compile correctly

**Specific Item:**

- Focus edits on confirmed target
- Apply comprehensive documentation

**Review All:**

- Systematically review and edit each doc comment
- Ensure full RFC 1574 compliance

### 3. Verify Edit Application

**CRITICAL - Verify changes are actually applied:**

```bash
# Read the modified file
cat [target_file]

# Check that doc comments are present
grep -n "///\|//!"
```

### 4. Validate Edited Documentation

**Ensure the edited documentation works:**

```bash
# Generate docs to verify syntax
cargo doc --no-deps

# Test doc examples compile
cargo test --doc [target_file]
```

### 5. Generate Edit Report

**Save to:** `{output_folder}/documentation-artifacts/report-edit-{target-file}-{timestamp}.md`

**Use template:** `templates/edit-report.md`

---

## REVIEW MODE WORKFLOW

**Review mode follows the same micro-file design pattern as other modes:**

- **Step 1:** `steps-r/step-01-assess.md` - Adversarial assessment and mindset adoption
- **Step 2:** `steps-r/step-02-comprehensive-review.md` - Systematic review across all quality categories
- **Step 3:** `steps-r/step-03-edge-case-testing.md` - Edge case analysis and stress testing
- **Step 4:** `steps-r/step-04-generate-report.md` - Comprehensive report generation with prioritization

**Review Mode Features:**
- Adversarial reviewer mindset (find every possible issue)
- Four-category quality assessment (Content, Technical, UX, Quality)
- Edge case and stress testing scenarios
- Severity-based issue classification (Critical, Major, Minor, Info)
- Comprehensive reporting with actionable recommendations
- Integration with EDIT MODE for targeted fixes

**Output:** `{output_folder}/documentation-artifacts/report-review-{target-file}-{timestamp}.md`

```
## Recommendations for Excellence

### Immediate Actions (Critical + Major)

1. [Priority fix]
2. [Priority fix]
3. [Priority fix]

### Quality Improvements (Minor)

1. [Enhancement]
2. [Enhancement]
3. [Enhancement]

### Future Enhancements (Informational)

1. [Long-term improvement]
2. [Long-term improvement]
3. [Long-term improvement]

## Review Methodology

This review was conducted using adversarial methodology:

- Content examined from user perspective and technical accuracy
- Examples tested for compilation and realism
- User experience evaluated for discoverability and clarity
- Quality assessed for consistency and completeness
- Edge cases and potential user problems considered

## Next Steps

1. **Address Critical Issues** - Fix immediately before any release
2. **Resolve Major Issues** - Plan fixes in next development cycle
3. **Consider Minor Improvements** - Include in regular maintenance
4. **Evaluate Informational Items** - Consider for future roadmap
```

### 5. Review Presentation and Follow-up

**Present review to user:**

```
**Adversarial Rustdoc Review Complete!**

**Target:** [file_path]
**Overall Assessment:** [EXCELLENT/GOOD/NEEDS IMPROVEMENT/CRITICAL]

**Findings Summary:**

- Critical Issues: [count] 🚨 (Fix Immediately)
- Major Issues: [count] ⚠️ (Fix Soon)
- Minor Issues: [count] 💡 (Nice to Fix)
- Informational: [count] 💭 (Consider for Future)

**Key Concern Areas:**

- Content Clarity: [x] issues
- Technical Accuracy: [x] issues
- User Experience: [x] issues
- Quality & Polish: [x] issues

**Critical Issues Found:**
[Summary of top 3 critical issues]

**Report saved to:** `{output_folder}/documentation-artifacts/report-review-{target-file}-{timestamp}.md`

Would you like to:

- **[F]ix critical issues** - Start with urgent fixes
- **[R]eview details** - See complete review report
- **[S]ummary by category** - Deep dive into specific areas
- **[Q]uit** - End review session
```

### 6. Handle User Choice

**IF F:**
- Load EDIT MODE workflow with critical issues pre-identified
- Focus fixes on critical issues first

**IF R:**
- Display complete review report with all findings
- Allow user to explore in detail

**IF S:**
- Show category-specific breakdown
- Allow focused review by area (content, technical, UX, quality)

**IF Q:**
- End review session with summary and report location

**Note:** All reports use standardized templates from `templates/` folder for consistency and maintainability.
