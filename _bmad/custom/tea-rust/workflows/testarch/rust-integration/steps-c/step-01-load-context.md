---
name: "step-01-load-context"
description: "Load config and identify integration scope"
outputFile: "{integration_output}/rust-integration-plan.md"
nextStepFile: "./step-02-scenarios.md"
knowledgeIndex: "{project-root}/_bmad/custom/tea-rust/knowledge/tea-rust-index.csv"
configSource: "{project-root}/_bmad/custom/tea-rust/config.yaml"
---

# Step 1: Load Context

## STEP GOAL:
Identify components, boundaries, and required context for orchestration.

## MANDATORY SEQUENCE
1. Load config
2. Identify components and boundaries
3. Determine scope (module/multi-module/project)
4. Expand scope if dependencies require it
5. Record scope expansion rationale in output
6. Load knowledge
7. Summarize initial scope to the user

## Menu
Select: [C] Continue
