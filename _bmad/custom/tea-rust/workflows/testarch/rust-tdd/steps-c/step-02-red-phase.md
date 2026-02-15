---
name: "step-02-red-phase"
description: "Define red-phase tests with GWT comments"
nextStepFile: "./step-03-green-phase.md"
outputFile: "{output_folder}/tea-rust/tdd-plan.md"
---

# Step 2: Red Phase (Failing Tests)

## STEP GOAL:
Define failing tests with GWT comments and edge-case coverage.

## MANDATORY SEQUENCE

### 1. Test Scenarios
- Identify happy path, edge cases, and error paths
- Ensure all public components are covered
 - Flag risk hotspots (CQRS ports, zero-copy boundaries, validation constructors)

### 2. GWT Comment Plan
- For each test, define Given/When/Then comments
- No doc comments in unit tests
- Ensure comments map to concrete inputs/outputs

### 3. Capture in Plan
- Append scenarios and GWT plan to output

### 4. Menu
Select: [C] Continue

#### Menu Handling
- IF C: update stepsCompleted, load {nextStepFile}
