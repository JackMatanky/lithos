---
name: "step-01-load-context"
description: "Load config and target scope for unit testing"
nextStepFile: "./step-02-inventory.md"
outputFile: "{output_folder}/tea-rust/unit-test-plan.md"
templateFile: "../templates/unit-plan-template.md"
knowledgeIndex: "{project-root}/_bmad/custom/tea-rust/knowledge/tea-rust-index.csv"
configSource: "{project-root}/_bmad/custom/tea-rust/config.yaml"
---

# Step 1: Load Context

## STEP GOAL:
Load configuration and determine unit test target scope.

## MANDATORY SEQUENCE
1. Load config
2. Identify target file/module
3. Determine scope (file/module/multi-module)
4. Expand scope if dependencies require it
5. Record scope expansion rationale in output
6. Load relevant Rust knowledge
7. Initialize output from template

## Menu
Select: [C] Continue
