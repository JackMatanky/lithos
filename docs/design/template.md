---
feature: [Feature Name]
status: Draft # Options: Draft, In Review, Approved, Implemented, Archived
author: [Name]
ticket: [Link to Issue]
date_created: YYYY-MM-DD
tags: [cache, refactor, performance]
---

# Tech Spec: [Feature Name]

> **Note**: See `docs/design/README.md` for usage instructions and T-Shirt sizing.

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
<!-- Diagrams (Mermaid), Component relationships, Data Flow. -->

### 3.2 Component & Interface Specifications
<!--
Define the "Contract" of new components (Interfaces/Traits), not just the code.
Focus on Responsibility, Inputs, Outputs, and Invariants.
-->

#### Component: [Name, e.g., `CacheEngine`]
*   **Responsibility**: [What does this own? e.g., "Manages raw byte storage on disk."]
*   **Public Interface**:
    *   `method(arg: Type) -> Result<Type>`
        *   *Behavior*: [What does it do?]
        *   *Errors*: [What failures are expected?]
*   **State/Invariants**: [e.g., "Must always hold a file lock."]

### 3.3 Data Models
<!--
"Show me your data structures, and I won't need your code."
Struct definitions, Database schemas, Protobufs.
-->

### 3.4 Core Logic & Algorithms
<!-- State machines, Consensus algorithms, Error handling policies. -->

## 4. Alternatives & Decisions (The "Divergence")
<!--
THE DECISION MATRIX
Why is this design better than the others?
-->

### 4.1 Tactical Decisions
<!-- Record specific choices made during design. -->

#### Decision: [e.g., Use B-Tree over HashMap]
*   **Context**: We need range queries.
*   **Choice**: B-Tree.
*   **Alternatives Considered**:
    *   *HashMap*: O(1) lookup, but O(N) range. Rejected.
    *   *SkipList*: Good for concurrency, but higher memory overhead. Rejected.

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
*   **Risk**: [e.g., "Connection pool exhaustion"]
    *   *Mitigation*: [e.g., "Implement circuit breakers"]

## 7. Critique & Refinement Log
<!--
THE VALIDATION PHASE
Document the "Review & Fix" loop.
-->

| Date | Critique / Issue | Resolution |
| :--- | :--- | :--- |
| YYYY-MM-DD | "API is blocking." | "Intentional. See Constraints in 1.3." |
