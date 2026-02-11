---
name: 'step-02-comprehensive-review'
description: 'Comprehensive adversarial review of rustdoc documentation'
nextStepFile: '../steps-r/step-03-edge-case-testing.md'
---

# Step 2: Comprehensive Review (Review Mode)

## STEP GOAL:

Conduct detailed adversarial review across all quality categories to identify every possible issue, inconsistency, and improvement opportunity.

## MANDATORY EXECUTION RULES (READ FIRST):

### Universal Rules:

- 📖 Read the complete step file before taking any action
- ✅ Speak in `{communication_language}`
- 🎯 Maintain adversarial reviewer mindset

### Role Reinforcement:

- ✅ You are a senior documentation reviewer
- ✅ Be thorough and ruthless in finding issues
- ✅ Think like a user having problems

### Step-Specific Rules:

- 🎯 Review all quality categories systematically
- 🎯 Document specific issues with locations
- 🎯 Categorize by severity and impact

## EXECUTION PROTOCOLS:

- 🎯 Follow the MANDATORY SEQUENCE exactly
- 🔍 Conduct systematic review by category
- 📊 Document all findings with specific details
- ✅ Update review findings as you proceed

## CONTEXT BOUNDARIES:

- Available context: Target file and initial assessment
- Focus: Comprehensive quality review
- Limits: Review only (no fixes yet)

## MANDATORY SEQUENCE

### 1. Content and Clarity Review

**Summaries and Descriptions:**
- [ ] Is every summary crystal clear to moderately experienced Rust developers?
- [ ] Are there ambiguous terms or jargon that could confuse users?
- [ ] Does documentation explain *why* something exists, not just *what* it is?
- [ ] Are there assumptions about user knowledge that should be clarified?
- [ ] Document specific locations and examples of unclear content

**Examples Analysis:**
- [ ] Do examples demonstrate realistic, useful scenarios?
- [ ] Are examples self-contained and copy-paste ready?
- [ ] Do examples cover common use cases AND edge cases?
- [ ] Are there missing examples that users would likely need?
- [ ] Do examples follow project conventions and patterns?
- [ ] Note any examples that don't compile or are confusing

**Instructions and Guidance:**
- [ ] Are "how to use" instructions actionable and clear?
- [ ] Are there missing safety considerations or gotchas?
- [ ] Are performance characteristics documented where relevant?
- [ ] Are backward compatibility or migration considerations noted?
- [ ] Identify missing guidance users would need

### 2. Technical Accuracy Review

**Code Examples Verification:**
- [ ] Do all examples actually compile? (`cargo test --doc`)
- [ ] Are type signatures correct and current?
- [ ] Are imports and use statements accurate?
- [ ] Do examples follow current API surface?
- [ ] Document any compilation failures with specific errors

**API Documentation Consistency:**
- [ ] Do parameter types match actual function signatures?
- [ ] Are return types accurately documented?
- [ ] Are lifetime parameters correctly explained?
- [ ] Are trait bounds and constraints properly documented?
- [ ] Note any inconsistencies between docs and code

**Error Handling Documentation:**
- [ ] Are all possible error variants documented?
- [ ] Are error conditions that users might encounter explained?
- [ ] Are recovery strategies for errors provided?
- [ ] Are panic conditions thoroughly documented?
- [ ] Identify missing error handling information

### 3. User Experience and Navigation Review

**Discoverability:**
- [ ] Can users easily find information they need?
- [ ] Are intra-doc links working and pointing to right places?
- [ ] Is related functionality cross-referenced effectively?
- [ ] Are module-level overviews helpful for navigation?
- [ ] Note any navigation difficulties or missing cross-references

**Progressive Disclosure:**
- [ ] Does documentation provide basic usage first, then advanced details?
- [ ] Are complex topics broken down into digestible pieces?
- [ ] Is there a clear learning path from simple to complex usage?
- [ ] Identify any overwhelming or poorly structured information

**Context and Use Cases:**
- [ ] Are real-world use scenarios provided?
- [ ] Is the motivation for design choices explained?
- [ ] Are common anti-patterns or mistakes highlighted and explained?
- [ ] Note any missing context or use case information

### 4. Quality and Polish Review

**Language and Tone:**
- [ ] Is language consistent throughout documentation?
- [ ] Is tone appropriate for target audience?
- [ ] Are there grammatical errors, typos, or awkward phrasing?
- [ ] Are technical terms used correctly and consistently?
- [ ] Document specific language issues with corrections needed

**Formatting and Structure:**
- [ ] Are code blocks properly formatted with language tags?
- [ ] Is markdown formatting correct and rendering properly?
- [ ] Are headings logical and hierarchical?
- [ ] Are lists, tables, and other formatting elements used effectively?
- [ ] Note any formatting problems that affect readability

**Completeness Gaps:**
- [ ] Are there undocumented public items?
- [ ] Are missing sections (Examples, Panics, Errors, Safety) actually needed?
- [ ] Are there undocumented invariants or constraints users should know?
- [ ] Are performance characteristics or resource usage documented?
- [ ] Identify any information gaps that would hurt users

### 5. Document Review Findings

Update comprehensive findings document:

```markdown
## Comprehensive Review Findings

### Content and Clarity Issues
[C-001] [Issue Title]
**Location:** [file.rs:line]
**Category:** Content
**Impact:** How this affects users
**Description:** Detailed explanation of the problem

### Technical Accuracy Issues
[T-001] [Issue Title]
**Location:** [file.rs:line]
**Category:** Technical
**Impact:** Potential for misuse or errors
**Description:** Technical inaccuracy or inconsistency

### User Experience Issues
[X-001] [Issue Title]
**Location:** [file.rs:line]
**Category:** User Experience
**Impact:** Difficulty finding or using information
**Description:** Navigation or discoverability problem

### Quality and Polish Issues
[Q-001] [Issue Title]
**Location:** [file.rs:line]
**Category:** Quality
**Impact:** Professional appearance and clarity
**Description:** Language, formatting, or consistency issue
```

### 6. Present Review Progress

Show {user_name}:

"**Comprehensive Review Progress!**

**Review Categories Completed:**
- ✅ Content and Clarity Review
- ✅ Technical Accuracy Review
- ✅ User Experience Review
- ✅ Quality and Polish Review

**Issues Found So Far:**
- Content Issues: [count]
- Technical Issues: [count]
- User Experience Issues: [count]
- Quality Issues: [count]
- **Total Issues:** [count]

**Next:** Edge case and stress testing

Continue with edge case analysis?

**Options:**
- **[C]ontinue** - Proceed to edge case testing
- **[S]ummary** - See detailed findings so far
- **[F]ocus** - Deep dive into specific category
- **[Q]uit** - End review session"

### 7. Handle User Choice

**IF C:**
- Load next step: `{nextStepFile}`
- Begin edge case testing

**IF S:**
- Show detailed findings by category
- Return to choice menu

**IF F:**
- Ask which category to explore
- Provide detailed breakdown of that category
- Return to choice menu

**IF Q:**
- Save current findings and end session

## 🚨 SYSTEM SUCCESS/FAILURE METRICS:

### ✅ SUCCESS:

- All four review categories completed systematically
- Issues documented with specific locations and impacts
- Findings categorized by type and severity
- User presented with review progress and options

### ❌ SYSTEM FAILURE:

- Review categories not completed systematically
- Issues not documented with specific details
- No categorization of findings
- User not consulted on next steps
