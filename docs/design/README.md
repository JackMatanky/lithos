# Tech Spec (Design Doc) Process

This directory contains **Technical Specifications** (Tech Specs). These documents capture the *tactical* design of features before we write code.

## Why write a Tech Spec?

> "Cheapest place to fix a bug is on the whiteboard."

We write specs to:
1.  **Force "Working Backwards"**: Design the API/Experience before the Implementation.
2.  **Catch "Impossible" Requirements**: Identify constraints (Allocations, Threading) early.
3.  **Align the Team**: Agree on the *interface* so parallel work is possible.

## How to use the Template

The `template.md` is comprehensive, but **not every feature needs every section**. Use this "T-Shirt Sizing" guide to right-size your doc.

### 👕 Small Feature (The "One-Pager")
*Use for: Internal refactors, minor API additions.*
*   **Keep**: 1. Problem Space, 2. Guide-Level Explanation (API), 3. Detailed Design.
*   **Delete**: Alternatives, Pre-Mortem, Operational Readiness.

### 👚 Medium Feature (The Standard)
*Use for: New Modules, Database Schema changes, Performance work.*
*   **Keep**: All sections.
*   **Merge**: You can merge "Operational Readiness" into "Detailed Design" if simple.

### 🧥 Large Feature (The System)
*Use for: Distributed Systems, Critical Path changes, Public APIs.*
*   **Keep**: Everything. **Crucial**: Do not skip "Pre-Mortem" or "Alternatives".

---

## 🤖 AI-Assisted Workflow (The "Easy Mode")

The template is designed to be filled by an AI agent acting as a Principal Engineer. Don't stare at a blank page.

### The "Design Partner" Prompt

Copy-paste this prompt to your AI agent to start a high-quality spec:

```text
Act as a Principal Software Architect. I need to write a Tech Spec for a new feature: [FEATURE NAME/DESCRIPTION].

Your goal is to help me "Work Backwards" from the user experience.

1.  **Read**: `docs/design/template.md`.
2.  **Interview Me**: Ask 3-5 sharp questions to clarify the "Problem Space" (Constraints, Non-Goals) and the "User Experience" (How should the API feel?).
3.  **Draft**: Once I answer, draft the "Guide-Level Explanation" (Section 2) and "Detailed Design" (Section 3).
4.  **Critique**: After drafting, assume the persona of a hostile SRE. Identify 3 ways this design could fail in production (The "Pre-Mortem").
5.  **Output**: Fill the template with our agreed design.

Do not ask me to fill the sections. You fill them based on my answers.
```

## The Lifecycle

1.  **Draft**: Create `docs/design/NNN-feature-name.md` using the template.
2.  **Critique**: Use the "Critique Log" in the template. Challenge your own assumptions.
    *   *Prompt*: "Critique this design. What is the biggest performance bottleneck?"
3.  **Approve**: Once the "Critique Log" shows resolution, move Status to `Approved`.
4.  **Implement**: Write the code.
5.  **Archive**: Once shipped, mark Status as `Implemented`. You do *not* need to keep this doc in sync with code forever. It is a historical plan.
