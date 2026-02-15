---
name: "step-04-refactor-phase"
description: "Refactor plan to improve quality while keeping tests green"
nextStepFile: "./step-05-integrate.md"
outputFile: "{test_design_output}/rust-tdd-plan.md"
---

# Step 4: Refactor Phase

## STEP GOAL:
Plan refactor improvements while preserving test pass status.

## MANDATORY SEQUENCE

### 1. Refactor Targets
- Identify readability, performance, and design improvements
- Ensure architectural constraints respected

### 2. Safety Checks
- Confirm all tests remain green after refactor
- Verify zero clippy warnings (or justified suppressions via `#[expect]`) in new code and tests

### 3. Update Plan
- Append refactor notes to output

### 4. Menu
Select: [C] Continue

#### Menu Handling
- IF C: update stepsCompleted, load {nextStepFile}
