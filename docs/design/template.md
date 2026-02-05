---
feature: [Feature Name]
status: Draft # Options: Draft, In Review, Approved, Implemented, Archived
author: [Name]
ticket: [Link to Issue]
date_created: YYYY-MM-DD
tags: [cache, refactor, performance]
---

# Tech Spec: [Feature Name]

> **Note**: See `docs/design/README.md` for usage instructions.

<!--
How to use this template (keep it lightweight):

1) Start with Section 1 (constraints + goals). If you can’t write 1.3, you’re not ready to design.
2) Write Section 2 (guide-level UX) next. If it’s hard to explain, the design is wrong.
3) Define the core types (3.2) and the component contracts (3.3) before any algorithms.
4) Draw the integration/data flow (3.4). Systems fail at the edges.
5) Use the critique log (7) and pre-mortem (6) to iterate before coding.
-->

## 1. Problem Space (The "Why")

<!--
THE PLANNING PHASE
Before designing, define the box we are playing in.
-->

### 1.1 Context & Background

<!-- What is the current state? Why are we doing this NOW? Link to relevant PRDs/ADRs. -->

### 1.2 Goals & Non-Goals

<!--
*   **Goals**: What MUST happen for this to be a success? (e.g., "Latency < 50ms")
*   **Non-Goals**: What are we explicitly NOT doing? (e.g., "Mobile support")
-->

### 1.3 Constraints (The Hard Limits)

<!--
Unmovable barriers (Budget, Legacy systems, Compliance).
Example: "Must be zero-copy", "Must use existing Postgres".
-->

## 2. Guide-Level Explanation (The "What")

<!--
THE WORKING BACKWARDS PHASE (API First)
Design the interface before the internals. How does the user/developer hold this?
-->

### 2.1 User/Dev Experience

<!--
Show, don't tell. Write the "Tutorial" for this feature.
-   Code snippets of the PUBLIC API.
-   CLI command examples.
-   Configuration examples.
-->

### 2.2 Mental Model

<!-- How should the user think about this concept? (e.g., "It acts like a Git branch") -->

## 3. Detailed Design (The "How")

<!--
THE REFERENCE PHASE
Now defines the implementation.
-->

### 3.1 System Architecture

<!-- Diagrams (Mermaid), Component relationships. -->

### 3.2 Data Models

<!--
Define the types the design is built from.

Put the key structs/enums/newtypes/traits here *before* the components that use them,
so the reader knows what objects can be used in the component/interface contracts.

Examples:
- Struct and enum definitions (Rust-ish pseudocode is fine)
- Database schemas / record layouts (if relevant)
- Serialization shapes (Serde DTOs vs validated domain types)
-->

<!--
"Show me your data structures, and I won't need your code."
-->

<!--
DATA MODELS TEMPLATES (keep it lightweight)

Use one of the templates below:
- Use the **Large struct/enum template** when a type is central or non-trivial.
- Use the **Type table template** when you have many small newtypes/IDs and you don’t want a lot of code blocks.


Template 1 — Large Struct/Enum (designated place for important info)

#### `[TypeName]` ([Domain | Raw/Input | Persistence])

- Keep this short. Only include bullets that matter; delete the rest.
- **Purpose**: One sentence.
- **Key rules**: Invariants + validation rules (and where they’re enforced).
- **Important notes (optional)**: Only if there’s a real footgun.
  - ownership/borrowing expectations (allocates vs borrows)
  - performance constraints (hot path, zero-copy expectations)
  - stability constraints (persisted-bytes contract / migration notes)
- **Shape** (Rust-ish pseudocode, optional):

```rust
pub struct TypeName {
  // fields
}

pub enum TypeKind {
  // variants
}
```


Template 2 — Small Types (newtypes/IDs)

Use one of the formats below. Prefer **Detailed** for 1–3 types; prefer **Compact** when there are many.

Detailed format:
- **Type**: `WidgetName` (Domain)
  - Purpose: one sentence
  - Backing: `Box<str>`
  - Rules: non-empty, <= 64 bytes
  - Notes (optional): allocates

Compact format (one line per type):
- `WidgetId` (Domain, `Uuid`): uniquely identifies a widget. Rules: always valid UUID.
- `WidgetName` (Domain, `Box<str>`): display name shown to users. Rules: non-empty, <= 64 bytes. Notes: allocates.
- `RawWidgetName` (Raw/Input, `String`): user input before validation. Notes: compiled into `WidgetName`.

Table format (only for unit types or single-field newtypes):

Use this table only when each type is effectively one attribute (or none), e.g.:
- `WidgetId(Uuid)`
- `WidgetName(Box<str>)`
- `ArchivedWidgetName(Box<str>)`

| Signature               | Purpose                      | Layer     | Rules                  | Notes                      |
|-------------------------|------------------------------|-----------|------------------------|----------------------------|
| `WidgetId(Uuid)`        | uniquely identifies a widget | Domain    | always valid UUID      | identifier                 |
| `WidgetName(Box<str>)`  | display name shown to users  | Domain    | non-empty, <= 64 bytes | allocates                  |
| `RawWidgetName(String)` | user input before validation | Raw/Input | none (may be invalid)  | compiled into `WidgetName` |

-->

### 3.3 Component & Interface Specifications

<!--
Define the "Contract" of new components (Interfaces/Traits), not just the code.
Focus on Responsibility, Inputs, Outputs, and Invariants.

This is also the canonical place to define the **types** that make the design real:
- new structs/enums/newtypes and what they represent
- traits/ports and their method contracts
- validation rules at construction time (make invalid states unrepresentable)
- ownership/borrowing expectations (what is borrowed vs owned)

If a type is central to the design, define it here even if you repeat it later in 3.4.
-->

#### Component: [Name, e.g., `CacheEngine`]

- **Responsibility**: [What does this own? e.g., "Manages raw byte storage on disk."]
- **Public Interface**:
  - `method(arg: Type) -> Result<Type>`
    - _Behavior_: [What does it do?]
    - _Errors_: [What failures are expected?]
- **State/Invariants**: [e.g., "Must always hold a file lock."]

### 3.4 Integration & Data Flow

<!--
How do components talk to each other?
-->

- **Sequence Diagram**: [Mermaid chart showing the call flow]
- **Events/Messages**: [Schema of events emitted or consumed]
- **Dependencies**: [External services or modules this relies on]

### 3.5 Core Logic & Algorithms

<!-- State machines, Consensus algorithms, Error handling policies. -->

## 4. Alternatives & Decisions (The "Divergence")

<!--
THE DECISION MATRIX
Why is this design better than the others?
-->

### 4.1 Tactical Decisions

<!-- Record specific choices made during design. -->

#### Decision: [e.g., Use B-Tree over HashMap]

- **Context**: We need range queries.
- **Choice**: B-Tree.
- **Alternatives Considered**:
  - _HashMap_: O(1) lookup, but O(N) range. Rejected.
  - _SkipList_: Good for concurrency, but higher memory overhead. Rejected.

## 5. Operational Readiness (The "Reality Check")

<!--
It works on my machine, but will it work in prod?
-->

### 5.1 Observability

<!-- Metrics, Logs, Traces. How do we know it's broken? -->

### 5.2 Migration Strategy

<!-- How do we switch from the old system? (Feature flags, Dual-write). -->

### 5.3 Security & Privacy

<!-- Permissions, Encryption, PII. -->

## 6. Pre-Mortem (The "Inversion")

<!--
Assume it is 6 months from now and this system failed. Why?
-->

- **Risk**: [e.g., "Connection pool exhaustion"]
  - _Mitigation_: [e.g., "Implement circuit breakers"]

## 7. Critique & Refinement Log

<!--
THE VALIDATION PHASE
Document the "Review & Fix" loop.
-->

| Date       | Critique / Issue   | Resolution                             |
|:-----------|:-------------------|:---------------------------------------|
| YYYY-MM-DD | "API is blocking." | "Intentional. See Constraints in 1.3." |

## 8. References

<!--
External references used to justify design decisions.
Examples:
- Rust API Guidelines
- OWASP guidance
- Crate docs (rkyv/redb/regex/chrono)
-->

- [Title](https://example.com)

<!--
Optional: add appendices below when you have implementation snapshots,
migration checklists, or other material you plan to delete later.
-->
