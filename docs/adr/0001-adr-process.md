---
name: adr-review-process-and-validation-tooling
status: accepted
stakeholders: [Jack (Developer), Architects]
date_proposed: 2026-01-11
date_decided: 2026-01-11
date_implemented: 2026-01-11
---

# ADR 001: ADR Review Process and Validation Tooling

## Context

As the Lithos project grows, we need a structured way to document and review architectural decisions. The current ADRs were created during the planning phase but lack a formal review process and automated validation. We need to ensure that all future decisions are thoroughly reviewed for completeness, correctness, and consistency.

## Decision

We will implement a formal ADR review process and validation tooling. This ADR itself records the decision and defines the process:

### 1. ADR Lifecycle

ADRs follow a standard lifecycle to ensure transparency and accountability:

- **Proposed**: The ADR is drafted and open for discussion.
- **Accepted**: The decision has been reviewed and approved.
- **Superseded**: A newer ADR replaces this decision.
- **Deprecated**: The decision is no longer relevant but preserved for historical context.
- **Rejected**: The proposal was reviewed but not accepted.

### 2. Review Checklist

Every ADR must be evaluated against:

- **Completeness**: Uses standard template, all required sections filled.
- **Correctness**: Accurate technical information, valid assumptions.
- **Consistency**: Aligns with existing principles, follows naming convention (`NNNN-name.md`).
- **Understandability**: Clear rationale, readable by new team members.
- **Feasibility**: Practical to implement within constraints.

### 3. Tooling and Integration

- **Validation**: The `validate-adrs` mise task checks for format and required sections.
- **Mise**: Integrated as `mise run validate-adrs`.
- **Pre-commit**: Added as a quality gate for all commits.
- **CI/CD**: Enforced in the GitHub Actions pipeline.

## Alternatives Considered

### Manual Review Only

- **Pros**: Low overhead.
- **Cons**: Prone to human error, inconsistent formatting.

### Using an ADR Management Tool (e.g., adr-log)

- **Pros**: Specialized tool.
- **Cons**: Adds another dependency, less flexible than custom scripting.

## Technical Validation

### Research Findings

- **MADR Standard**: MADR (Markdown Architectural Decision Records) is the industry standard for lightweight ADRs.
- **Shell Scripting for Validation**: Shell scripts provide the most portable and zero-dependency way to enforce documentation standards in a Git-first workflow.

### Compatibility & Performance

- **Hexagonal Alignment**: Supports architectural integrity by ensuring all cross-crate decisions are documented.
- **Performance Impact**: Validation script runs in <50ms, negligible impact on pre-commit or CI time.

## Consequences

- **Positive**: Improved architectural integrity, automated enforcement of documentation standards.
- **Negative**: Slight overhead in creating and reviewing ADRs.
