# Sprint Change Proposal: Epic 1 Architectural Realignment

**Date:** 2025-10-19
**Author:** Sarah (Product Owner)

## 1. Identified Issue Summary

A post-completion review of `Epic 1: Foundational CLI & Static Template Engine` revealed that the implementation did not follow the project's mandated Hexagonal Architecture. The resulting code is a monolith, directly conflicting with the documented architectural principles in `docs/architecture/`.

## 2. Epic Impact Summary

-   **Epic 1:** The implementation is considered a technical dead end and cannot serve as the foundation for future work.
-   **Epic 2:** `Epic 2: Hexagonal Architecture Realignment` was created to refactor Epic 1. As we have decided to re-implement rather than refactor, Epic 2 is now obsolete.
-   **Future Epics (3-6):** All subsequent epics are blocked until a correctly architected foundation is in place.

## 3. Artifact Adjustment Needs

-   The code implemented for the stories in Epic 1 must be discarded.
-   The stories within Epic 1 (1.1 - 1.9) should be returned to a "To Do" state in the backlog.
-   The document `docs/prd/epic-2-hexagonal-architecture-realignment.md` should be marked as "Obsolete".

## 4. Recommended Path Forward

We will **Rollback & Re-implement**. The implementation for the stories in Epic 1 will be discarded, and the development work will be redone from scratch, adhering strictly to the hexagonal architecture from the outset. The existing story definitions and acceptance criteria from Epic 1 remain valid and will be reused.

## 5. High-Level Action Plan

1.  **Backlog Adjustment (PO/SM):**
    -   Reset the status of stories 1.1 through 1.9 to "To Do".
    -   Archive or delete `docs/prd/epic-2-hexagonal-architecture-realignment.md`.
2.  **Code Rollback (Dev):**
    -   Remove the incorrect Go code that was written for the initial implementation of Epic 1.
3.  **Re-implementation (Dev):**
    -   The Dev agent will begin work on story 1.1, implementing it and all subsequent stories in Epic 1 according to the definitions in `docs/architecture/`.

## 6. Agent Handoff Plan

-   This proposal serves as the directive for the **Product Owner (PO)** and **Scrum Master (SM)** to adjust the project backlog.
-   Once the backlog is reset, the **Full Stack Developer (dev)** agent is cleared to begin the re-implementation of Epic 1.
