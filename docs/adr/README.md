# Architectural Decision Record (ADR) Process

This guide documents the process for proposing, reviewing, and maintaining architectural decisions in the Lithos project.

## Why ADRs?

We use ADRs to capture the "why" behind significant architectural choices. This prevents "architectural drift" and helps new team members understand the constraints and trade-offs that shaped the system.

## When to Create an ADR

Create an ADR for any decision that:
- Affects multiple crates or hexagonal layers.
- Introduces a new major dependency.
- Changes a core implementation pattern (e.g., switching from `async` to threads).
- Defines a significant data format or communication protocol.

## The ADR Lifecycle

1. **Drafting**: Use `docs/adr/template.md` to create a new ADR. Number it sequentially (`NNNN-name.md`).
2. **Review**: Submit a PR. The team reviews for:
    - **Completeness**: All template sections filled.
    - **Technical Validation**: Strong research and analysis of alternatives.
    - **Consequences**: Realistic assessment of both pros and cons.
3. **Approval**: Once consensus is reached, update status to `Accepted`.
4. **Implementation**: The decision is executed in code. Update `Status Tracking` once implemented.

## Validation Tooling

- **Validate Format**: Run `mise run validate-adrs` to ensure template compliance.
- **Check Metrics**: Run `./scripts/adr-metrics.sh` to see the current state of the architecture library.

## Template Standards

Every ADR MUST include:
- **Metadata**: Status, Date, Stakeholders.
- **Context**: The "Problem" and constraints.
- **Decision**: The "Solution" and how it works.
- **Alternatives Considered**: At least one other option and why it wasn't chosen.
- **Technical Validation**: Research findings and performance/compatibility analysis.
- **Consequences**: Both Positive and Negative (be honest!).
- **Status Tracking**: Timeline of the decision.
