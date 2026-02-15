---
name: "step-01-load-context"
description: "Load config and determine benchmark scope"
nextStepFile: "./step-02-design.md"
outputFile: "{output_folder}/tea-rust/benchmark-plan.md"
templateFile: "../templates/benchmark-plan-template.md"
knowledgeIndex: "{project-root}/_bmad/custom/tea-rust/knowledge/tea-rust-index.csv"
configSource: "{project-root}/_bmad/custom/tea-rust/config.yaml"
---

# Step 1: Load Context

## STEP GOAL:
Identify benchmark targets and scope.

## MANDATORY SEQUENCE
1. Load config
2. Identify benchmark targets
3. Determine scope and baselines
4. Expand scope if dependencies require it
5. Record scope expansion rationale in output
6. Load knowledge
7. Initialize output

## Menu
Select: [C] Continue
