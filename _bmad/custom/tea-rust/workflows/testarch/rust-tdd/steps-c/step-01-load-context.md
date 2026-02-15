---
name: "step-01-load-context"
description: "Load config, determine scope, and gather story/requirements for Rust TDD"
nextStepFile: "./step-02-red-phase.md"
outputFile: "{output_folder}/tea-rust/tdd-plan.md"
templateFile: "../templates/tdd-plan-template.md"
knowledgeIndex: "{project-root}/_bmad/custom/tea-rust/knowledge/tea-rust-index.csv"
configSource: "{project-root}/_bmad/custom/tea-rust/config.yaml"
---

# Step 1: Load Context

## STEP GOAL:
Load configuration, determine scope, and gather story/requirements for TDD.

## MANDATORY EXECUTION RULES (READ FIRST):
- 🛑 NEVER generate content without user input
- 📖 Read the entire step file before acting
- ✅ Speak in {communication_language}

## MANDATORY SEQUENCE

### 1. Load Config
- Read {configSource}
- Store {user_name}, {communication_language}, {output_folder}

### 2. Determine Scope
- Ask for target if not provided
- Determine file/module/multi-module/project scope
- Expand scope if dependencies require it (ports, infra, or cross-context boundaries)
- Record any scope expansion rationale in output

### 3. Gather Inputs
- Ask for story/requirements file if available
- If none, capture intent and acceptance criteria

### 4. Load Knowledge
- Use {knowledgeIndex} to load relevant Rust testing fragments
- Prioritize architecture + anti-patterns when scope crosses contexts

### 5. Initialize Output
- Create output file from template
- Record scope and inputs

### 6. Menu
Select: [C] Continue

#### Menu Handling
- IF C: update stepsCompleted, load {nextStepFile}
