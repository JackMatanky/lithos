---
name: "step-01-load-context"
description: "Load config and target path for review"
nextStepFile: "./step-02-analyze.md"
outputFile: "{test_review_output}/rust-test-review.md"
templateFile: "../templates/review-report-template.md"
knowledgeIndex: "{project-root}/_bmad/custom/tea-rust/knowledge/tea-rust-index.csv"
configSource: "{project-root}/_bmad/custom/tea-rust/config.yaml"
---

# Step 1: Load Context

## STEP GOAL:
Load configuration, require target path, and load knowledge.

## MANDATORY SEQUENCE
1. Load config
2. Require file/module path
3. Determine scope (single/directory/suite)
4. Expand scope if dependencies require it
5. Record scope expansion rationale in output
6. Load knowledge fragments
7. Initialize output

## Menu
Select: [C] Continue
