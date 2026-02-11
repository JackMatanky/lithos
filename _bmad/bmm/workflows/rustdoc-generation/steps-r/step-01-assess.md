---
name: 'step-01-assess'
description: 'Adversarial assessment of rustdoc documentation'
nextStepFile: '../steps-r/step-02-comprehensive-review.md'
---

# Step 1: Adversarial Assessment (Review Mode)

## STEP GOAL:

Adopt adversarial reviewer mindset and conduct initial assessment of target documentation for quality, clarity, and completeness issues.

## MANDATORY EXECUTION RULES (READ FIRST):

### Universal Rules:

- 📖 Read the complete step file before taking any action
- ✅ Speak in `{communication_language}`
- 🎯 Adopt adversarial reviewer mindset

### Role Reinforcement:

- ✅ You are a senior documentation reviewer
- ✅ Find every possible issue and improvement opportunity
- ✅ Think: "What could make this confusing, incorrect, or incomplete?"

### Step-Specific Rules:

- 🎯 Be ruthless in identifying even minor issues
- 🎯 Consider user perspective above all
- 🎯 Look for problems users might actually encounter

## EXECUTION PROTOCOLS:

- 🎯 Follow the MANDATORY SEQUENCE exactly
- 🔍 Load and analyze target documentation
- 📊 Document initial assessment findings
- ✅ Confirm assessment scope with user

## CONTEXT BOUNDARIES:

- Available context: Target path from initialization
- Focus: Initial adversarial assessment
- Limits: Assessment only (no fixes yet)

## MANDATORY SEQUENCE

### 1. Load Target Documentation

Read the complete Rust file(s) at the target path:
- If directory: Read all `.rs` files, identify crate root
- Analyze all existing documentation thoroughly
- Document the intended audience and use patterns

### 2. Adopt Adversarial Reviewer Mindset

**You are now acting as a senior documentation reviewer with these principles:**

**Adversarial Mindset:**
- Find every possible issue, inconsistency, and improvement opportunity
- Think: "What could make this documentation confusing, incorrect, or incomplete?"
- Be ruthless in identifying even minor issues that users might encounter
- Consider worst-case user scenarios and confusion points

**User-Centric Focus:**
- How would a moderately experienced Rust developer interpret this?
- What assumptions are being made about user knowledge?
- What questions would users have after reading this?
- Where could users get stuck or frustrated?

**Quality Standards:**
- Is this documentation truly excellent or just "good enough"?
- Are there any ambiguities that could lead to misuse?
- Are examples realistic and helpful?
- Is the information complete and accurate?

### 3. Initial Quality Assessment

**Quick Assessment Categories:**

**Content Clarity:**
- Are summaries crystal clear?
- Are there ambiguous terms or jargon?
- Does documentation explain *why* things exist?

**Technical Accuracy:**
- Do examples actually compile?
- Are type signatures correct?
- Are error conditions documented?

**User Experience:**
- Can users find what they need?
- Are examples realistic and useful?
- Is navigation clear?

**Quality & Polish:**
- Is language consistent?
- Are there typos or grammatical errors?
- Is formatting correct?

### 4. Document Initial Findings

Create initial assessment summary:

```markdown
## Initial Adversarial Assessment

**Target:** [file_path]
**Assessment Date:** [date]
**Reviewer Mindset:** Adversarial - Find all possible issues

### Quick Quality Scan
- **Content Clarity:** [Excellent/Good/Fair/Poor]
- **Technical Accuracy:** [Excellent/Good/Fair/Poor]
- **User Experience:** [Excellent/Good/Fair/Poor]
- **Quality & Polish:** [Excellent/Good/Fair/Poor]

### Immediate Concerns
- [Top 3 most obvious issues]
- [Areas that need deep investigation]
- [Potential user problems]

### Assessment Scope
Based on initial scan, comprehensive review should focus on:
- [Priority area 1]
- [Priority area 2]
- [Priority area 3]
```

### 5. Present Assessment to User

Show {user_name}:

"**Adversarial Assessment Complete!**

**Target:** [file_path]
**Initial Quality Assessment:**
- Content Clarity: [rating]
- Technical Accuracy: [rating]
- User Experience: [rating]
- Quality & Polish: [rating]

**Immediate Concerns Identified:**
- [Top concern 1]
- [Top concern 2]
- [Top concern 3]

**Recommended Review Focus:**
- [Priority area 1]
- [Priority area 2]
- [Priority area 3]

Ready to proceed with comprehensive adversarial review?

**Options:**
- **[C]ontinue** - Begin comprehensive review
- **[F]ocus** - Specify review priorities
- **[S]ummary** - See detailed initial assessment
- **[Q]uit** - End review session"

### 6. Handle User Choice

**IF C:**
- Load next step: `{nextStepFile}`
- Begin comprehensive review

**IF F:**
- Ask user to specify priorities
- Adjust review focus accordingly
- Then load next step

**IF S:**
- Show detailed initial assessment
- Return to choice menu

**IF Q:**
- End review session gracefully

## 🚨 SYSTEM SUCCESS/FAILURE METRICS:

### ✅ SUCCESS:

- Target documentation loaded completely
- Adversarial mindset adopted
- Initial assessment conducted
- Quality categories evaluated
- User presented with assessment summary

### ❌ SYSTEM FAILURE:

- Target not loaded completely
- Adversarial mindset not adopted
- No initial assessment conducted
- User not consulted on next steps
