# Architectural Decision Record (ADR) Process

This guide documents the process for proposing, reviewing, and maintaining architectural decisions in the Lithos project.

## Two-Tier Documentation Strategy

We use two types of documents to separate **Strategic Constraints** from **Tactical Implementation**.

| Feature | **ADR** (Architectural Decision Record) | **Tech Spec** (Design Document) |
| :--- | :--- | :--- |
| **Scope** | Cross-cutting, System-wide | Feature-scoped, Component-specific |
| **Why write it?** | To record a **significant design choice** that is hard to reverse. | To plan **how to build** a specific feature or module. |
| **Lifecycle** | **Immutable**. Status changes (Proposed -> Accepted -> Superseded). | **Living** during dev. Matches code. Archived after shipping. |
| **Template** | `docs/adr/template.md` | `docs/design/template.md` |

### Decision Matrix: Which one do I write?

| Question | If YES, write an **ADR** | If YES, write a **Tech Spec** |
| :--- | :--- | :--- |
| **Reversibility** | Is this decision **expensive** to reverse later? | Can we easily refactor this later? |
| **Impact** | Does this affect **multiple components** or layers? | Is this isolated to a **single component**? |
| **Constraint** | Does this establish a **rule** other devs must follow? | Is this just describing **logic flow**? |
| **Content** | Are we choosing a **technology or pattern**? | Are we defining **structs or functions**? |

## When to Create an ADR

Create an ADR only for **Architecturally Significant Decisions** (ASDs). An ASD typically:
- **Has a high cost of reversal** (e.g., choosing a primary database, defining a wire protocol).
- **Involves a significant trade-off** (e.g., prioritizing write-throughput over read-consistency).
- **Deviates from established patterns** (e.g., "We are using a synchronous call here despite our async-first policy because...").

### When NOT to Create an ADR
- **Routine Implementation**: Adding a new feature that follows existing patterns (Use a **Tech Spec**).
- **Bug Fixes**: Unless the fix requires a fundamental architectural shift.
- **Refactoring**: Cleaning up code without changing system boundaries or behaviors.

## The ADR Lifecycle

ADRs are **immutable documents**. Once accepted, they are not updated. If a decision changes, a new ADR is created to **supersede** the old one.

1. **Drafting**: Use `docs/adr/template.md`. Number sequentially (`NNNN-name.md`).
2. **Review**: Submit a PR. The team reviews for:
    - **Context**: Is the problem clearly stated?
    - **Honesty**: Are the negative consequences listed?
    - **Evidence**: Is the decision backed by research, benchmarks, or citations?
    - **Alternatives**: Did we steelman the options we rejected?
3. **Status Transitions**:
    - `Proposed`: Under review.
    - `Accepted`: Approved and active.
    - `Rejected`: Reviewed but not adopted (valuable history).
    - `Superseded`: Replaced by a newer ADR (e.g., "Superseded by 0012").
    - `Deprecated`: The decision no longer applies due to system evolution.

## Validation Tooling

- **Validate Format**: `mise run adr:validate`
- **Check Metrics**: `mise run adr:metrics`

## Template Standards

Every ADR MUST focus on **Strategic Intent** over implementation details.
- **Context**: The forces at play (business goals, technical constraints).
- **Decision**: The specific path chosen.
- **Technical Validation**: Proof that this works (benchmarks, prototypes, citations).
- **Consequences**: The resulting context (good, bad, and neutral).
