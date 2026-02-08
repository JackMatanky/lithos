---
name: 'step-01-assess'
description: 'Assess existing rustdoc for editing'
nextStepFile: '../steps-e/step-02-apply-edit.md'
---

# Step 1: Assess Edit Target (Edit Mode)

## STEP GOAL:

Identify which rustdoc needs editing and load the target file.

## MANDATORY EXECUTION RULES (READ FIRST):

### Universal Rules:

- 📖 Read the complete step file before taking any action
- ✅ Speak in `{communication_language}`
- 🎯 Identify specific edit targets

### Role Reinforcement:

- ✅ You are a rustdoc editor
- ✅ Collaborative approach with {user_name}

### Step-Specific Rules:

- 🎯 Must identify specific doc comments to edit
- 🎯 Understand what needs improvement
- 🚫 Do not edit until target is confirmed

## EXECUTION PROTOCOLS:

- 🎯 Follow the MANDATORY SEQUENCE exactly
- 📖 Load target file
- ✅ Confirm edit scope with user

## CONTEXT BOUNDARIES:

- Available context: Target path from initialization
- Focus: Identify edit targets
- Limits: No edits yet

## MANDATORY SEQUENCE

### 1. Load Target File

Read the Rust file at the target path provided during initialization.

### 2. Analyze Existing Documentation

Identify all doc comments in the file:
- Count `//!` (crate/module level)
- Count `///` (item level)
- Identify which items are documented
- Note any obvious issues

### 3. Present Options to User

Ask {user_name}:

"**Edit Mode: Rustdoc Documentation**

I've loaded `[file_path]`. Here's what I found:

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

Please select: [A]dd / [I]mprove / [F]ix / [S]pecific / [R]eview"

### 4. Handle Selection

**IF A (Add missing):**
- List undocumented items
- Ask which to document
- Proceed to edit step

**IF I (Improve existing):**
- Show existing docs
- Ask what needs improvement
- Proceed to edit step

**IF F (Fix issues):**
- List RFC 1574 violations
- Proceed to edit step

**IF S (Specific item):**
- Ask which item
- Load that item's context
- Proceed to edit step

**IF R (Review all):**
- Present all docs systematically
- Proceed to edit step

### 5. Confirm Target

Confirm with {user_name}:

"**Edit Target Confirmed**

Editing: [target description]
File: [file_path]

Ready to proceed with edits?"

### 6. Load Edit Step

Load next step: `{nextStepFile}`

## 🚨 SYSTEM SUCCESS/FAILURE METRICS:

### ✅ SUCCESS:

- Target file loaded
- Edit scope identified
- User confirmed target
- Next step loaded

### ❌ SYSTEM FAILURE:

- Target not identified
- User not consulted
- Proceeding without confirmation
