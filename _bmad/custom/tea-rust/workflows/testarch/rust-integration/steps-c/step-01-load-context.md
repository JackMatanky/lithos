---
name: "step-01-load-context"
description: "Load config and identify integration scope"
nextStepFile: "./step-02-scenarios.md"
outputFile: "{output_folder}/tea-rust/integration-plan.md"
templateFile: "../templates/integration-plan-template.md"
knowledgeIndex: "{project-root}/_bmad/custom/tea-rust/knowledge/tea-rust-index.csv"
configSource: "{project-root}/_bmad/custom/tea-rust/config.yaml"
---

# Step 1: Load Context

## STEP GOAL:
Identify components, boundaries, and required context.

## MANDATORY SEQUENCE
1. Load config
2. Identify components and boundaries
3. Determine scope (module/multi-module/project)
4. Expand scope if dependencies require it
5. Record scope expansion rationale in output
6. Load knowledge
7. Initialize output

## Menu
Select: [C] Continue
