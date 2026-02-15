# Rust TDD Workflow

**Workflow:** `tea-rust-tdd`
**Version:** 1.0 (Step-File Architecture)

---

## Overview

Guide new Rust implementations through red-green-refactor while enforcing Lithos architecture, Rust-specific testing practices, adaptive scope expansion, and quality-first criteria.

---

## WORKFLOW ARCHITECTURE

This workflow uses **step-file architecture**:

- **Micro-file Design**: Each step is self-contained
- **JIT Loading**: Only the current step file is in memory
- **Sequential Enforcement**: Execute steps in order

---

## INITIALIZATION SEQUENCE

### 1. Configuration Loading

From `workflow.yaml`, resolve:

- `config_source`, `output_folder`, `user_name`, `communication_language`, `document_output_language`, `date`
- `target_scope`, `story_path`

### 2. Scope and Context

- Detect scope from target and dependencies (file/module/multi-module/project)
- Expand context when ports or cross-cutting infrastructure are touched
- Load knowledge fragments relevant to the detected scope

### 3. First Step

Load, read completely, and execute:
`{project-root}/_bmad/custom/tea-rust/workflows/testarch/rust-tdd/steps-c/step-01-load-context.md`
